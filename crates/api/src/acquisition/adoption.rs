use super::*;
use import_plan::File;
use std::os::unix::fs::MetadataExt;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Input {
    pub source_root: String,
    pub token: Option<String>,
}

async fn plan(a: &App, id: &str, input: &Input) -> Result<Vec<File>> {
    let c = config::load(a).await?;
    if !c.enabled {
        return Err(bad("enable acquisition before adopting files"));
    }
    let row = sqlx::query("SELECT data,state FROM acquisition_downloads WHERE id=?")
        .bind(id)
        .fetch_optional(&a.pool)
        .await
        .map_err(db)?
        .ok_or_else(|| bad("unknown download"))?;
    if row.get::<String, _>("state") != "paused" {
        return Err(bad("pause this transfer before adopting data"));
    }
    let data = decode(row.get("data"))?;
    let raw = data["cached_torrent"]
        .as_str()
        .or(data["input"]["torrent_base64"].as_str())
        .ok_or_else(|| {
            bad("upload the original .torrent file; magnet metadata must be resolved first")
        })?;
    let (_, files) = torrent_files::validate(
        &downloads::decode_torrent(raw)?,
        c.storage_limit_gb as u64 * 1024 * 1024 * 1024,
    )?;
    let source = tokio::fs::canonicalize(&input.source_root)
        .await
        .map_err(|_| bad("source folder unavailable"))?;
    if !source.is_absolute()
        || input.source_root.len() > 4096
        || !tokio::fs::metadata(&source)
            .await
            .map_err(|_| bad("source unavailable"))?
            .is_dir()
    {
        return Err(bad("choose an existing absolute source folder"));
    }
    // Restrict adoption to a separate download tree; never copy application state
    // or existing library media through a torrent migration action.
    storage::validate_download_root(a, &source).await?;
    let download = storage::canonical_candidate(std::path::Path::new(&c.download_dir)).await?;
    if storage::overlaps(&source, &download) {
        return Err(bad("source and managed download folders must be separate"));
    }
    tokio::fs::create_dir_all(&download)
        .await
        .map_err(|_| bad("download storage unavailable"))?;
    let device = tokio::fs::metadata(&download)
        .await
        .map_err(|_| bad("download storage unavailable"))?
        .dev();
    let mut result = Vec::new();
    for file in files.as_array().into_iter().flatten() {
        let path = file["path"]
            .as_str()
            .ok_or_else(|| bad("invalid torrent path"))?;
        let expected = file["size"]
            .as_u64()
            .ok_or_else(|| bad("invalid torrent size"))?;
        let original = source.join(path);
        let resolved = match tokio::fs::canonicalize(&original).await {
            Ok(p) => p,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(_) => return Err(bad("source file unavailable")),
        };
        if !resolved.starts_with(&source) || resolved != original {
            return Err(bad("source contains symlinks or escapes its root"));
        }
        let metadata = tokio::fs::metadata(&resolved)
            .await
            .map_err(|_| bad("source unavailable"))?;
        if !metadata.is_file() || metadata.len() != expected {
            continue;
        }
        result.push(File {
            file_index: file["index"]
                .as_u64()
                .ok_or_else(|| bad("invalid file index"))? as usize,
            source: resolved.to_string_lossy().into_owned(),
            destination: download.join(id).join(path).to_string_lossy().into_owned(),
            size: expected,
            coverage: Vec::new(),
            title_id: String::new(),
            root_device: device,
            root: download.to_string_lossy().into_owned(),
            source_stamp: import_io::stamp(&metadata),
        });
    }
    if result.is_empty() {
        return Err(bad(
            "no same-size torrent files found under this source root",
        ));
    }
    let total: u64 = result.iter().map(|f| f.size).sum();
    if storage::free_bytes(download.clone()).await?
        < total.saturating_add(c.min_free_gb as u64 * 1024 * 1024 * 1024)
        || storage::used_bytes(download).await?.saturating_add(total)
            > c.storage_limit_gb as u64 * 1024 * 1024 * 1024
    {
        return Err(bad(
            "not enough reserved download storage for adoption copies",
        ));
    }
    Ok(result)
}
pub(super) async fn preview(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<Input>,
) -> Result<Json<Value>> {
    users::admin(&a, &h).await?;
    let files = plan(&a, &id, &input).await?;
    Ok(Json(
        json!({"files":files,"token":import_plan::token(&files)?,"notice":"Copies only. Originals are retained and never hardlinked to a writable torrent. Resume explicitly to recheck pieces before seeding or downloading."}),
    ))
}
pub(super) async fn apply(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<Input>,
) -> Result<Json<Value>> {
    users::admin(&a, &h).await?;
    let _permit = a
        .acquisition
        .import_jobs
        .clone()
        .try_acquire_owned()
        .map_err(|_| bad("another import or adoption is active"))?;
    let _gate = a.acquisition.gate.lock().await;
    let files = plan(&a, &id, &input).await?;
    if input.token.as_deref() != Some(import_plan::token(&files)?.as_str()) {
        return Err(bad("preview current source and destination files first"));
    }
    if let Some(session) = a.acquisition.current().await
        && let Some(handle) = downloads::find(
            &session,
            downloads::view(&a, &id).await?["infohash"].as_str(),
        )
    {
        session
            .delete(handle.id().into(), false)
            .await
            .map_err(|_| bad("could not detach the paused engine job"))?;
    }
    let count = files.len();
    let runtime = a.acquisition.clone();
    let job = id.clone();
    tokio::task::spawn_blocking(move || {
        for f in files {
            import_io::publish_mode(
                &f,
                &format!("adopt-{job}-{}", f.file_index),
                true,
                true,
                &runtime,
            )?;
        }
        Ok::<_, std::io::Error>(())
    })
    .await
    .map_err(|_| bad("adoption interrupted"))?
    .map_err(|e| bad(format!("adoption failed: {e}")))?;
    // No old client's fast-resume counters are trusted. Initialization must hash
    // pieces again; the queue remains paused until the operator hands over ownership.
    sqlx::query("UPDATE acquisition_downloads SET engine_id=NULL,data=json_set(data,'$.stats',json('{}')) WHERE id=?").bind(&id).execute(&a.pool).await.map_err(db)?;
    event(
        &a,
        &id,
        "Existing torrent data copied; explicit resume will recheck every piece",
    )
    .await?;
    Ok(Json(json!({"copied_files":count,"state":"paused"})))
}
