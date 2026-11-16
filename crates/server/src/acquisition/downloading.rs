use super::*;

/// Project recognized, selected video files into their eventual library. No media is
/// published here: the scanner/import journal remains the only completed-file source.
pub(super) async fn sync(a: &App) -> Result<()> {
    // A title correction unlinks its source without rewriting watch history.
    sqlx::query("DELETE FROM acquisition_stream_items WHERE EXISTS(SELECT 1 FROM acquisition_downloads d WHERE d.id=download_id AND (d.state='removed' OR d.title_id IS NULL OR d.title_id!=acquisition_stream_items.title_id))")
        .execute(&a.pool).await.map_err(db)?;
    let rows = sqlx::query("SELECT d.id,d.title_id,d.data,t.data title FROM acquisition_downloads d JOIN acquisition_titles t ON t.id=d.title_id WHERE d.state IN ('queued','checking','downloading','paused','seeding','completed') ORDER BY d.created_at LIMIT 1000")
        .fetch_all(&a.pool).await.map_err(db)?;
    for row in rows {
        let data = decode(row.get("data"))?;
        let title = decode(row.get("title"))?;
        let download: String = row.get("id");
        let title_id: String = row.get("title_id");
        let Some(target) =
            destinations::for_title(a, &title_id, title["kind"].as_str().unwrap_or("movie"))
                .await?
        else {
            continue;
        };
        let registered:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM libraries l JOIN library_roots r ON r.library_id=l.id LEFT JOIN library_options o ON o.library_id=l.id WHERE l.id=? AND r.path=? AND COALESCE(o.enabled,1)=1 AND l.kind=?)")
            .bind(&target.library_id).bind(&target.root).bind(if title["kind"]=="tv"{"tv"}else{"movies"}).fetch_one(&a.pool).await.map_err(db)?;
        if !registered {
            continue;
        }
        let files: Vec<&Value> = data["files"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|f| {
                f["path"].as_str().is_some_and(auto_import::is_video)
                    && data["input"]["selected_files"]
                        .as_array()
                        .is_none_or(|selected| selected.contains(&f["index"]))
            })
            .collect();
        if title["kind"] == "movie" && files.len() != 1 {
            continue;
        }
        for file in files {
            let Some(index) = file["index"].as_i64() else {
                continue;
            };
            let imported: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM acquisition_imports WHERE download_id=? AND file_index=? AND state='scanned')")
                .bind(&download).bind(index).fetch_one(&a.pool).await.map_err(db)?;
            if imported && data["reimport_requested"] != true {
                continue;
            }
            let mapped: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM acquisition_stream_items WHERE download_id=? AND file_index=?)")
                .bind(&download).bind(index).fetch_one(&a.pool).await.map_err(db)?;
            if mapped {
                continue;
            }
            let coverage = if title["kind"] == "tv" {
                auto_import::file_coverage(file["path"].as_str().unwrap_or_default())?
            } else {
                Vec::new()
            };
            if title["kind"] == "tv" && coverage.is_empty() {
                continue;
            }
            let episode = if let Some(first) = coverage.first() {
                let raw: Option<String> = sqlx::query_scalar("SELECT data FROM acquisition_episodes WHERE title_id=? AND season=? AND episode=?")
                    .bind(&title_id).bind(first.season).bind(first.episode).fetch_optional(&a.pool).await.map_err(db)?;
                let Some(raw) = raw else { continue };
                Some(decode(&raw)?)
            } else {
                None
            };
            let _permit = match a.acquisition.import_jobs.clone().try_acquire_owned() {
                Ok(permit) => permit,
                Err(_) => return Ok(()),
            };
            let mut tx = a.pool.begin_with("BEGIN IMMEDIATE").await.map_err(db)?;
            // Do not duplicate installed titles or attach a second release to an
            // identity already playing from another download.
            let candidates = sqlx::query("SELECT i.id,i.data,EXISTS(SELECT 1 FROM acquisition_stream_items s WHERE s.item_id=i.id) mapped FROM items i JOIN item_metadata m ON m.item_id=i.id WHERE i.library_id=? AND CAST(json_extract(m.data,'$.provider_ids.tmdb') AS INTEGER)=?")
                .bind(&target.library_id).bind(title["provider_id"].as_i64()).fetch_all(&mut *tx).await.map_err(db)?;
            let candidates: Vec<_> = candidates
                .iter()
                .filter(|r| {
                    let Ok(item) = decode(r.get("data")) else {
                        return false;
                    };
                    if title["kind"] == "movie" {
                        item["kind"] == "movie"
                    } else {
                        item["kind"] == "episode"
                            && coverage.first().is_some_and(|e| {
                                item["season_number"] == e.season
                                    && item["episode_number"] == e.episode
                            })
                    }
                })
                .collect();
            if candidates.len() > 1
                || candidates.iter().any(|r| {
                    r.get::<bool, _>("mapped")
                        || decode(r.get("data")).is_ok_and(|i| i["file_available"] != false)
                })
            {
                continue;
            }
            let existing = candidates.first().map(|r| r.get::<String, _>("id"));
            let id = existing
                .clone()
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
            if existing.is_none() {
                let first = coverage.first();
                let value = json!({"id":id,"library_id":target.library_id,"kind":if first.is_some(){"episode"}else{"movie"},
                    "title":episode.as_ref().map(|e|&e["name"]).unwrap_or(&title["title"]),"show_title":if first.is_some(){title["title"].clone()}else{Value::Null},
                    "season_number":first.map(|e|e.season),"episode_number":first.map(|e|e.episode),"episode_coverage":coverage,
                    "duration_seconds":null,"container":null,"video_codec":null,"audio_codec":null,
                    "file_available":false,"position_seconds":0,"completed":false});
                let path = PathBuf::from(&target.root).join(".oxevu-pending").join(&id);
                sqlx::query("INSERT INTO items(id,library_id,path,data) VALUES(?,?,?,?)")
                    .bind(&id)
                    .bind(&target.library_id)
                    .bind(path.to_string_lossy().as_ref())
                    .bind(value.to_string())
                    .execute(&mut *tx)
                    .await
                    .map_err(db)?;
                let mut metadata = imports::imported_metadata(&title);
                metadata["title"] = value["title"].clone();
                if let Some(e) = &episode {
                    metadata["show_title"] = title["title"].clone();
                    metadata["overview"] = e["overview"].clone();
                }
                sqlx::query("INSERT INTO item_metadata VALUES(?,?)")
                    .bind(&id)
                    .bind(metadata.to_string())
                    .execute(&mut *tx)
                    .await
                    .map_err(db)?;
            }
            sqlx::query("INSERT OR IGNORE INTO acquisition_stream_items VALUES(?,?,?,?)")
                .bind(id)
                .bind(&download)
                .bind(&title_id)
                .bind(index)
                .execute(&mut *tx)
                .await
                .map_err(db)?;
            tx.commit().await.map_err(db)?;
        }
    }
    Ok(())
}

/// Point the pending identity at a checksum-verified publication before scanning,
/// so a concurrent catalog refresh cannot observe a second temporary item ID.
pub(super) async fn locate_import(
    a: &App,
    download: &str,
    library: &str,
    file: &import_plan::File,
) -> Result<()> {
    sqlx::query("UPDATE items SET path=? WHERE library_id=? AND COALESCE(json_extract(data,'$.file_available'),1)=0 AND id=(SELECT item_id FROM acquisition_stream_items WHERE download_id=? AND file_index=? AND title_id=?) AND NOT EXISTS(SELECT 1 FROM items other WHERE other.path=? AND other.id!=items.id)")
        .bind(&file.destination).bind(library).bind(download).bind(file.file_index as i64).bind(&file.title_id).bind(&file.destination).execute(&a.pool).await.map_err(db)?;
    Ok(())
}

/// Dynamic transfer status never marks a sparse torrent file as a local file.
pub(crate) async fn annotate(a: &App, items: &mut [Value]) -> Result<()> {
    let rows = sqlx::query("SELECT s.item_id,s.file_index,s.title_id expected_title,d.id,d.title_id,d.state,d.infohash,d.data FROM acquisition_stream_items s JOIN acquisition_downloads d ON d.id=s.download_id")
        .fetch_all(&a.pool).await.map_err(db)?;
    let engine = a.acquisition.current().await;
    let enabled = config::load(a).await?.enabled;
    for row in rows {
        let Some(item) = items
            .iter_mut()
            .find(|i| i["id"] == row.get::<String, _>("item_id"))
        else {
            continue;
        };
        if item["file_available"] != false {
            continue;
        }
        let data = decode(row.get("data"))?;
        let index = row.get::<i64, _>("file_index") as usize;
        let matching = row.get::<Option<String>, _>("title_id").as_deref()
            == Some(row.get::<String, _>("expected_title").as_str());
        let state: String = row.get("state");
        let handle = engine
            .as_ref()
            .and_then(|s| downloads::find(s, row.get::<Option<String>, _>("infohash").as_deref()));
        let ready = enabled
            && matching
            && data["input"]["selected_files"]
                .as_array()
                .is_none_or(|selected| selected.iter().any(|i| i.as_u64() == Some(index as u64)))
            && matches!(state.as_str(), "downloading" | "seeding" | "completed")
            && handle.is_some_and(|h| {
                let s = h.stats();
                (s.finished || s.state.to_string() == "live")
                    && h.only_files()
                        .is_none_or(|selected| selected.contains(&index))
            });
        item["download"] = json!({"id":row.get::<String,_>("id"),"file_index":index,"state":if matching {state.as_str()} else {"identity_changed"},
            "bytes_received":data["stats"]["file_progress"].get(index).and_then(Value::as_u64).unwrap_or(0),"size_bytes":data["files"].get(index).and_then(|f|f["size"].as_u64()),"playable":ready});
    }
    Ok(())
}
