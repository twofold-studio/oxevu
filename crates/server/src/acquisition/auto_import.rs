use super::*;
use import_plan::{Input, Mapping};
use scope::Episode;

pub(crate) async fn run(a: App) {
    let mut timer = tokio::time::interval(std::time::Duration::from_secs(10));
    loop {
        timer.tick().await;
        if a.acquisition.is_stopping() {
            break;
        }
        if let Err(e) = tick(&a).await {
            let _ = event(
                &a,
                "imports",
                e.1.0["error"].as_str().unwrap_or("import worker failed"),
            )
            .await;
        }
    }
}
pub(super) async fn tick(a: &App) -> Result<()> {
    let c = config::load(a).await?;
    if !c.import_enabled {
        return Ok(());
    }
    recover(a).await?;
    let rows=sqlx::query("SELECT d.id,d.title_id,d.data,m.library_id,m.root,m.root_device FROM acquisition_downloads d JOIN acquisition_monitors m ON m.title_id=d.title_id WHERE d.state IN ('downloading','seeding','completed','paused') AND m.enabled=1 AND m.library_id IS NOT NULL AND m.root IS NOT NULL AND json_array_length(json_extract(d.data,'$.files'))>0 ORDER BY d.created_at LIMIT 1000").fetch_all(&a.pool).await.map_err(db)?;
    for row in rows {
        if a.acquisition.is_stopping() {
            break;
        }
        let id: String = row.get("id");
        match for_download(a, &row).await {
            Ok(()) => {}
            Err(e) => {
                let message = e.1.0["error"]
                    .as_str()
                    .unwrap_or("manual import review required");
                let mut data = decode(row.get("data"))?;
                if data["import_warning"] != message {
                    data["import_warning"] = json!(message);
                    // Update one JSON field without replacing concurrently refreshed stats.
                    sqlx::query("UPDATE acquisition_downloads SET data=json_set(data,'$.import_warning',?) WHERE id=?").bind(message).bind(&id).execute(&a.pool).await.map_err(db)?;
                    event(a, &id, message).await?;
                }
            }
        }
    }
    Ok(())
}

async fn for_download(a: &App, row: &sqlx::sqlite::SqliteRow) -> Result<()> {
    use std::os::unix::fs::MetadataExt;
    let root: String = row.get("root");
    let meta = tokio::fs::metadata(&root)
        .await
        .map_err(|_| bad("library root offline; import suspended"))?;
    if Some(meta.dev() as i64) != row.get::<Option<i64>, _>("root_device") {
        return Err(bad("library mount changed; import suspended"));
    }
    let raw: String = sqlx::query_scalar("SELECT data FROM acquisition_titles WHERE id=?")
        .bind(row.get::<String, _>("title_id"))
        .fetch_one(&a.pool)
        .await
        .map_err(db)?;
    let title = decode(&raw)?;
    let data = decode(row.get("data"))?;
    let items = wanted::inventory(a, &title).await?;
    if items.iter().any(|i| i["online"] != true) {
        return Err(bad(
            "existing media offline or unverified; import suspended",
        ));
    }
    let units = wanted::units(a, &title, &items).await?;
    let mappings = mappings(&data, &title, &units)?;
    if mappings.is_empty() {
        return Ok(());
    }
    let input = Input {
        library_id: row.get("library_id"),
        root,
        mappings,
        token: None,
    };
    let plan = import_plan::build(a, row.get("id"), &input).await?;
    imports::execute(a, row.get("id"), &input.library_id, plan).await
}

pub(super) fn is_video(path: &str) -> bool {
    let path = path.to_ascii_lowercase();
    !["sample", "trailer", "featurette", "extras/", "extras\\"]
        .iter()
        .any(|s| path.contains(s))
        && std::path::Path::new(&path)
            .extension()
            .and_then(|s| s.to_str())
            .is_some_and(|s| ["mkv", "mp4", "avi", "mov", "webm", "m4v", "ts"].contains(&s))
}
pub(super) fn file_coverage(path: &str) -> Result<Vec<Episode>> {
    let facts = release_facts::facts(&path.replace('_', " "))?;
    let (Some(season), Some(first)) = (facts["season"].as_u64(), facts["episode"].as_u64()) else {
        return Ok(Vec::new());
    };
    let last = facts["episode_end"].as_u64().unwrap_or(first);
    if last < first || last - first >= 20 {
        return Err(bad("invalid multi-episode file range"));
    }
    Ok((first..=last)
        .map(|episode| Episode {
            season: season as u32,
            episode: episode as u32,
        })
        .collect())
}
pub(super) fn mappings(data: &Value, title: &Value, units: &[Value]) -> Result<Vec<Mapping>> {
    let mut mappings = Vec::new();
    for f in data["files"].as_array().into_iter().flatten() {
        let path = f["path"]
            .as_str()
            .ok_or_else(|| bad("invalid torrent file"))?;
        if !is_video(path) {
            continue;
        }
        let index = f["index"]
            .as_u64()
            .ok_or_else(|| bad("invalid torrent file index"))? as usize;
        if data["input"]["selected_files"]
            .as_array()
            .is_some_and(|selected| !selected.iter().any(|i| i.as_u64() == Some(index as u64)))
        {
            continue;
        }
        if data["stats"]["file_progress"]
            .as_array()
            .and_then(|p| p.get(index))
            .and_then(Value::as_u64)
            != f["size"].as_u64()
        {
            continue;
        }
        let episodes = if title["kind"] == "tv" {
            file_coverage(path)?
        } else {
            Vec::new()
        };
        if title["kind"] == "tv" && episodes.is_empty() {
            return Err(bad(
                "episode numbering is ambiguous; review file mappings manually",
            ));
        }
        let wanted = if title["kind"] == "movie" {
            units
                .iter()
                .any(|u| u["wanted"] == true && u["available"] != true)
        } else {
            episodes.iter().any(|e| {
                units.iter().any(|u| {
                    u["season"] == e.season
                        && u["episode"] == e.episode
                        && u["wanted"] == true
                        && u["available"] != true
                })
            })
        };
        if wanted {
            mappings.push(Mapping {
                file_index: index,
                episodes,
            });
        }
    }
    if title["kind"] == "movie" && mappings.len() > 1 {
        return Err(bad(
            "multiple movie files found; select the main film manually",
        ));
    }
    Ok(mappings)
}

async fn recover(a: &App) -> Result<()> {
    let rows=sqlx::query("SELECT id,download_id,library_id,plan FROM acquisition_imports WHERE state IN ('copying','published') LIMIT 10").fetch_all(&a.pool).await.map_err(db)?;
    for row in rows {
        let Some(library) = row.get::<Option<String>, _>("library_id") else {
            continue;
        };
        let plan: import_plan::File = serde_json::from_str(row.get("plan"))
            .map_err(|_| bad("invalid import recovery plan"))?;
        if let Err(e) = imports::execute(a, row.get("download_id"), &library, vec![plan]).await {
            sqlx::query("UPDATE acquisition_imports SET state='failed',error=? WHERE id=?")
                .bind(e.1.0["error"].as_str())
                .bind(row.get::<String, _>("id"))
                .execute(&a.pool)
                .await
                .map_err(db)?;
        }
    }
    Ok(())
}
