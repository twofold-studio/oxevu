use super::*;
use base64::{Engine, engine::general_purpose::STANDARD};
use librqbit::{AddTorrent, AddTorrentOptions, AddTorrentResponse, ManagedTorrent, Session};

pub(super) async fn prepare(
    a: &App,
    c: &config::Config,
    session: &Arc<Session>,
    id: &str,
    mut data: Value,
) -> Result<Option<Arc<ManagedTorrent>>> {
    let mut input: downloads::Input = serde_json::from_value(data["input"].clone())
        .map_err(|_| bad("invalid download intent"))?;
    let source = source(a, &data, &input).await?;
    let listed = tokio::time::timeout(
        std::time::Duration::from_secs(60),
        session.add_torrent(
            source,
            Some(AddTorrentOptions {
                list_only: true,
                ..Default::default()
            }),
        ),
    )
    .await
    .map_err(|_| bad("torrent metadata discovery timed out"))?
    .map_err(|_| bad("could not read torrent metadata"))?;
    let listed = match listed {
        AddTorrentResponse::ListOnly(v) => v,
        AddTorrentResponse::AlreadyManaged(_, h) => {
            let owner: Option<String> =
                sqlx::query_scalar("SELECT id FROM acquisition_downloads WHERE infohash=?")
                    .bind(h.info_hash().as_string())
                    .fetch_optional(&a.pool)
                    .await
                    .map_err(db)?;
            if owner.as_deref() == Some(id) {
                return Ok(Some(h));
            }
            sqlx::query("UPDATE acquisition_downloads SET state='duplicate',error='Torrent is already managed by another job',data=json_set(data,'$.duplicate_of',?) WHERE id=? AND state NOT IN ('removed','paused')").bind(owner).bind(id).execute(&a.pool).await.map_err(db)?;
            return Ok(None);
        }
        _ => return Err(bad("unexpected torrent discovery response")),
    };
    let _gate = a.acquisition.gate.lock().await;
    if !config::load(a).await?.enabled
        || !a
            .acquisition
            .current()
            .await
            .is_some_and(|current| Arc::ptr_eq(&current, session))
    {
        return Ok(None);
    }
    let latest = sqlx::query("SELECT state,data FROM acquisition_downloads WHERE id=?")
        .bind(id)
        .fetch_optional(&a.pool)
        .await
        .map_err(db)?;
    let Some(latest) = latest else {
        return Ok(None);
    };
    if matches!(
        latest.get::<String, _>("state").as_str(),
        "removed" | "paused"
    ) || a.acquisition.is_stopping()
    {
        return Ok(None);
    }
    data = decode(latest.get("data"))?;
    input = serde_json::from_value(data["input"].clone())
        .map_err(|_| bad("invalid download intent"))?;
    if input.only_wanted {
        let reserved: i64 =
            sqlx::query_scalar("SELECT count(*) FROM acquisition_claims WHERE download_id=?")
                .bind(id)
                .fetch_one(&a.pool)
                .await
                .map_err(db)?;
        if reserved == 0 {
            sqlx::query("UPDATE acquisition_downloads SET state='duplicate',error='Wanted coverage has been reassigned; review before retrying' WHERE id=?").bind(id).execute(&a.pool).await.map_err(db)?;
            return Ok(None);
        }
    }
    let bytes = listed.torrent_bytes.to_vec();
    let (hash, files) =
        torrent_files::validate(&bytes, c.storage_limit_gb as u64 * 1024 * 1024 * 1024)?;
    let n = files.as_array().map_or(0, Vec::len);
    data["cached_torrent"] = json!(STANDARD.encode(&bytes));
    data["files"] = files.clone();
    sqlx::query("UPDATE acquisition_downloads SET data=json_set(data,'$.cached_torrent',?,'$.files',json(?)) WHERE id=? AND state!='removed'")
        .bind(data["cached_torrent"].as_str()).bind(files.to_string()).bind(id).execute(&a.pool).await.map_err(db)?;
    if input.only_wanted && input.selected_files.is_none() {
        let title_id = input
            .title_id
            .as_deref()
            .ok_or_else(|| bad("automatic transfer has no catalog identity"))?;
        let raw: String = sqlx::query_scalar("SELECT data FROM acquisition_titles WHERE id=?")
            .bind(title_id)
            .fetch_one(&a.pool)
            .await
            .map_err(db)?;
        let title = decode(&raw)?;
        let items = wanted::inventory(a, &title).await?;
        let mut units = wanted::units(a, &title, &items).await?;
        if let Some(coverage) = data["wanted_coverage"].as_array() {
            units.retain(|u| {
                coverage.iter().any(|pair| {
                    pair[0].as_i64() == Some(u["season"].as_i64().unwrap_or(-1))
                        && pair[1].as_i64() == Some(u["episode"].as_i64().unwrap_or(-1))
                })
            });
        }
        let selected = select_wanted(&files, &title, &units)?;
        if selected.is_empty() {
            return Err(bad(
                "torrent has no unambiguous wanted video files; select files manually",
            ));
        }
        input.selected_files = Some(selected);
        data["input"] = serde_json::to_value(&input).map_err(bad)?;
    }
    if input
        .selected_files
        .as_ref()
        .is_some_and(|v| v.iter().any(|i| *i >= n))
    {
        return Err(bad("selected file index is outside torrent"));
    }
    if input.only_wanted {
        claims::retain_files(a, id, &files, input.selected_files.as_deref()).await?;
    }
    let selected_size: u64 = files
        .as_array()
        .into_iter()
        .flatten()
        .filter(|f| {
            input.selected_files.as_ref().is_none_or(|v| {
                f["index"]
                    .as_u64()
                    .is_some_and(|i| v.contains(&(i as usize)))
            })
        })
        .filter_map(|f| f["size"].as_u64())
        .sum();
    tokio::fs::create_dir_all(&c.download_dir)
        .await
        .map_err(|_| bad("download storage unavailable"))?;
    let free = storage::free_bytes(PathBuf::from(&c.download_dir)).await?;
    let remaining =
        selected_size.saturating_sub(data["stats"]["progress_bytes"].as_u64().unwrap_or(0));
    if free < remaining.saturating_add(c.min_free_gb as u64 * 1024 * 1024 * 1024) {
        return Err(bad("not enough free download storage"));
    }
    let state: Option<String> =
        sqlx::query_scalar("SELECT state FROM acquisition_downloads WHERE id=?")
            .bind(id)
            .fetch_optional(&a.pool)
            .await
            .map_err(db)?;
    if state
        .as_deref()
        .is_none_or(|s| matches!(s, "removed" | "paused"))
        || a.acquisition.is_stopping()
    {
        return Ok(None);
    }
    let current = config::load(a).await?;
    if !current.enabled || current.download_dir != c.download_dir {
        return Ok(None);
    }
    let duplicate: Option<String> =
        sqlx::query_scalar("SELECT id FROM acquisition_downloads WHERE infohash=? AND id!=?")
            .bind(&hash)
            .bind(id)
            .fetch_optional(&a.pool)
            .await
            .map_err(db)?;
    if let Some(other) = duplicate {
        data["duplicate_of"] = json!(other);
        sqlx::query("UPDATE acquisition_downloads SET state='duplicate',data=?,error='Torrent already belongs to another queue entry' WHERE id=?").bind(data.to_string()).bind(id).execute(&a.pool).await.map_err(db)?;
        return Ok(None);
    }
    let output = PathBuf::from(&c.download_dir).join(id);
    if tokio::fs::symlink_metadata(&output)
        .await
        .is_ok_and(|m| m.file_type().is_symlink())
    {
        return Err(bad("download job folder is a symlink"));
    }
    data["cached_torrent"] = json!(STANDARD.encode(&bytes));
    data["files"] = files;
    data["uploaded_base"] = json!(data["stats"]["uploaded_bytes"].as_u64().unwrap_or(0));
    sqlx::query(
        "UPDATE acquisition_downloads SET data=?,infohash=?,state='checking',error=NULL WHERE id=?",
    )
    .bind(data.to_string())
    .bind(&hash)
    .bind(id)
    .execute(&a.pool)
    .await
    .map_err(db)?;
    let response = session
        .add_torrent(
            AddTorrent::from_bytes(bytes),
            Some(AddTorrentOptions {
                output_folder: Some(output.to_string_lossy().into_owned()),
                only_files: input.selected_files,
                paused: true,
                overwrite: true,
                initial_peers: Some(listed.seen_peers.into_iter().take(128).collect()),
                ..Default::default()
            }),
        )
        .await
        .map_err(|_| bad("could not initialize torrent storage"))?;
    let handle = response
        .into_handle()
        .ok_or_else(|| bad("torrent handle missing"))?;
    sqlx::query("UPDATE acquisition_downloads SET engine_id=? WHERE id=?")
        .bind(handle.id() as i64)
        .bind(id)
        .execute(&a.pool)
        .await
        .map_err(db)?;
    event(
        a,
        id,
        "Torrent metadata verified; rechecking existing pieces",
    )
    .await?;
    Ok(Some(handle))
}

fn select_wanted(files: &Value, title: &Value, units: &[Value]) -> Result<Vec<usize>> {
    let mut selected = Vec::new();
    for file in files.as_array().into_iter().flatten() {
        let path = file["path"]
            .as_str()
            .ok_or_else(|| bad("invalid torrent path"))?;
        if !auto_import::is_video(path) {
            continue;
        }
        let wanted = if title["kind"] == "movie" {
            true
        } else {
            let coverage = auto_import::file_coverage(path)?;
            coverage.iter().any(|e| {
                units.iter().any(|u| {
                    u["wanted"] == true
                        && u["available"] != true
                        && u["season"] == e.season
                        && u["episode"] == e.episode
                })
            })
        };
        if wanted {
            selected.push(
                file["index"]
                    .as_u64()
                    .ok_or_else(|| bad("invalid file index"))? as usize,
            );
        }
    }
    if title["kind"] == "movie" && selected.len() != 1 {
        return Err(bad("movie torrent requires manual main-file selection"));
    }
    Ok(selected)
}

async fn source(a: &App, data: &Value, input: &downloads::Input) -> Result<AddTorrent<'static>> {
    if let Some(raw) = data["cached_torrent"]
        .as_str()
        .or(input.torrent_base64.as_deref())
    {
        return Ok(AddTorrent::from_bytes(downloads::decode_torrent(raw)?));
    }
    if let Some(magnet) = input.magnet.as_ref() {
        return Ok(AddTorrent::from_url(magnet.clone()));
    }
    let id = input
        .release_id
        .as_deref()
        .ok_or_else(|| bad("download source missing"))?;
    let row = sqlx::query("SELECT indexer_id,data FROM acquisition_releases WHERE id=?")
        .bind(id)
        .fetch_one(&a.pool)
        .await
        .map_err(db)?;
    let release = decode(row.get("data"))?;
    if let Some(uri) = release["magneturl"].as_str().or_else(|| {
        release["download_url"]
            .as_str()
            .filter(|s| s.starts_with("magnet:?"))
    }) {
        return Ok(AddTorrent::from_url(uri.to_owned()));
    }
    let indexer = indexers::load(a, row.get("indexer_id")).await?;
    let uri = release["download_url"]
        .as_str()
        .ok_or_else(|| bad("release has no torrent URL"))?;
    Ok(AddTorrent::from_bytes(
        indexers::fetch_torrent(&indexer, uri).await?,
    ))
}
