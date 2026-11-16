use super::*;
use base64::{Engine, engine::general_purpose::STANDARD};
use librqbit::{ManagedTorrent, Session};

#[derive(Clone, Deserialize, Serialize, Default)]
#[serde(default, deny_unknown_fields)]
pub(super) struct Input {
    pub release_id: Option<String>,
    pub title_id: Option<String>,
    pub magnet: Option<String>,
    pub torrent_base64: Option<String>,
    pub selected_files: Option<Vec<usize>>,
    pub override_profile: bool,
    pub profile_id: Option<String>,
    pub only_wanted: bool,
    pub paused: bool,
}

pub(super) async fn create(
    State(a): State<App>,
    h: HeaderMap,
    Json(v): Json<Input>,
) -> Result<Json<Value>> {
    access::require(&a, &h, "downloads").await?;
    let id = enqueue(&a, v).await?;
    Ok(Json(json!({"download":view(&a,&id).await?})))
}
pub(super) async fn enqueue(a: &App, mut v: Input) -> Result<String> {
    let c = config::load(a).await?;
    if !c.enabled {
        return Err(bad("enable downloads in acquisition settings first"));
    }
    storage::validate_download_root(a, std::path::Path::new(&c.download_dir)).await?;
    if v.selected_files
        .as_ref()
        .is_some_and(|files| files.is_empty() || files.len() > 10000)
    {
        return Err(bad("invalid file selection"));
    }
    if let Some(files) = v.selected_files.as_mut() {
        files.sort_unstable();
        files.dedup();
    }
    let sources = usize::from(v.release_id.is_some())
        + usize::from(v.magnet.is_some())
        + usize::from(v.torrent_base64.is_some());
    if sources != 1 {
        return Err(bad("provide one release, magnet or torrent file"));
    }
    let mut hash = None;
    let mut metadata_files = json!([]);
    let mut release_snapshot = Value::Null;
    if let Some(id) = v.release_id.as_deref() {
        let r=sqlx::query("SELECT title_id,data,blocked_until>unixepoch() blocked FROM acquisition_releases WHERE id=?").bind(id).fetch_optional(&a.pool).await.map_err(db)?.ok_or_else(||bad("unknown release"))?;
        let release = decode(r.get("data"))?;
        release_snapshot = json!({"title":release["title"],"indexer_name":release["indexer_name"],"size":release["size"]});
        let profile = profiles::load(a, v.profile_id.as_deref().unwrap_or("default")).await?;
        if (r.get::<bool, _>("blocked") || !profiles::evaluate(&profile, &release).is_empty())
            && !v.override_profile
        {
            return Err(bad(
                "release is blocked or does not meet the profile; explicit override required",
            ));
        }
        v.title_id = Some(r.get("title_id"));
    }
    if let Some(raw) = v.magnet.as_deref() {
        if raw.len() > 16384 || !raw.starts_with("magnet:?") {
            return Err(bad("invalid magnet link"));
        }
        let magnet = librqbit::Magnet::parse(raw).map_err(|_| bad("invalid magnet link"))?;
        hash = Some(
            magnet
                .as_id20()
                .ok_or_else(|| bad("pure v2 torrents are not supported"))?
                .as_string(),
        );
    }
    if let Some(raw) = v.torrent_base64.as_deref() {
        let bytes = decode_torrent(raw)?;
        let (infohash, files) =
            super::torrent_files::validate(&bytes, c.storage_limit_gb as u64 * 1024 * 1024 * 1024)?;
        hash = Some(infohash);
        metadata_files = files;
    }
    if let Some(title) = v.title_id.as_deref() {
        let count: i64 = sqlx::query_scalar("SELECT count(*) FROM acquisition_titles WHERE id=?")
            .bind(title)
            .fetch_one(&a.pool)
            .await
            .map_err(db)?;
        if count != 1 {
            return Err(bad("unknown catalog title"));
        }
    }
    let candidates = claims::candidates(a, &v).await?;
    let mut tx = a.pool.begin_with("BEGIN IMMEDIATE").await.map_err(db)?;
    let existing:Option<String>=sqlx::query_scalar("SELECT id FROM acquisition_downloads WHERE (infohash=? OR release_id=?) AND state!='removed' LIMIT 1")
        .bind(&hash).bind(&v.release_id).fetch_optional(&mut *tx).await.map_err(db)?;
    if let Some(id) = existing {
        return Ok(id);
    }
    let mut coverage = Vec::new();
    if v.only_wanted {
        let title = v
            .title_id
            .as_deref()
            .ok_or_else(|| bad("automatic transfer requires a title"))?;
        let (free, existing) = claims::reserve(&mut tx, title, &candidates).await?;
        if free.is_empty() {
            return existing
                .ok_or_else(|| bad("release has no unclaimed wanted episodes or movie"));
        }
        coverage = free;
    }
    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM acquisition_downloads WHERE state!='removed'")
            .fetch_one(&mut *tx)
            .await
            .map_err(db)?;
    if count >= 1000 {
        return Err(bad("download queue limit reached"));
    }
    let removed: Option<String> = sqlx::query_scalar(
        "SELECT id FROM acquisition_downloads WHERE infohash=? AND state='removed'",
    )
    .bind(&hash)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db)?;
    let id = removed.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    sqlx::query("INSERT INTO acquisition_downloads(id,title_id,release_id,infohash,data,state) VALUES(?,?,?,?,?,?) ON CONFLICT(id) DO UPDATE SET title_id=excluded.title_id,release_id=excluded.release_id,state=excluded.state,data=excluded.data,error=NULL,attempts=0,retry_at=0,updated_at=unixepoch()")
        .bind(&id).bind(&v.title_id).bind(&v.release_id).bind(hash).bind(json!({"input":v,"release":release_snapshot,"stats":{},"files":metadata_files,"uploaded_base":0,"wanted_coverage":coverage}).to_string()).bind(if v.paused {"paused"} else {"queued"}).execute(&mut *tx).await.map_err(db)?;
    for (season, episode) in coverage {
        sqlx::query("INSERT INTO acquisition_claims VALUES(?,?,?,?)")
            .bind(&v.title_id)
            .bind(season)
            .bind(episode)
            .bind(&id)
            .execute(&mut *tx)
            .await
            .map_err(db)?;
    }
    tx.commit().await.map_err(db)?;
    event(a, &id, "Download queued").await?;
    Ok(id)
}

pub(super) fn decode_torrent(raw: &str) -> Result<Vec<u8>> {
    if raw.len() > 3 * 1024 * 1024 {
        return Err(bad("torrent file exceeds 2 MiB"));
    }
    let bytes = STANDARD
        .decode(raw)
        .map_err(|_| bad("invalid torrent encoding"))?;
    if bytes.len() > 2 * 1024 * 1024 {
        return Err(bad("torrent file exceeds 2 MiB"));
    }
    Ok(bytes)
}
pub(super) fn find(session: &Session, hash: Option<&str>) -> Option<Arc<ManagedTorrent>> {
    session.with_torrents(|torrents| {
        for (_, handle) in torrents {
            if Some(handle.info_hash().as_string().as_str()) == hash {
                return Some(handle.clone());
            }
        }
        None
    })
}
pub(super) async fn list(State(a): State<App>, h: HeaderMap) -> Result<Json<Value>> {
    access::require(&a, &h, "downloads").await?;
    let ids:Vec<String>=sqlx::query_scalar("SELECT id FROM acquisition_downloads WHERE state!='removed' ORDER BY created_at DESC,id LIMIT 1000").fetch_all(&a.pool).await.map_err(db)?;
    let mut jobs = Vec::new();
    for id in ids {
        jobs.push(view(&a, &id).await?);
    }
    Ok(Json(
        json!({"downloads":jobs,"import_enabled":config::load(&a).await?.import_enabled}),
    ))
}
pub(super) async fn view(a: &App, id: &str) -> Result<Value> {
    let r=sqlx::query("SELECT d.*,t.data title,releases.data cached_release,m.enabled monitor_enabled,m.library_id import_library,m.root import_root FROM acquisition_downloads d LEFT JOIN acquisition_titles t ON t.id=d.title_id LEFT JOIN acquisition_releases releases ON releases.id=d.release_id LEFT JOIN acquisition_monitors m ON m.title_id=d.title_id WHERE d.id=?").bind(id).fetch_optional(&a.pool).await.map_err(db)?.ok_or_else(||bad("unknown download"))?;
    let data = decode(r.get("data"))?;
    let import_blocker = if r.get::<Option<String>, _>("title_id").is_none() {
        Some("Associate this download with a catalog title before importing.")
    } else if r.get::<Option<String>, _>("import_library").is_none()
        || r.get::<Option<String>, _>("import_root").is_none()
    {
        Some("Choose this title's destination library and folder in Wanted.")
    } else if r.get::<Option<bool>, _>("monitor_enabled") != Some(true) {
        Some("Enable monitoring for this title in Wanted to import automatically.")
    } else {
        None
    };
    let release = if data["release"].is_object() {
        data["release"].clone()
    } else {
        r.get::<Option<String>, _>("cached_release")
            .map(|raw| decode(&raw))
            .transpose()?
            .unwrap_or(Value::Null)
    };
    Ok(
        json!({"id":id,"title_id":r.get::<Option<String>,_>("title_id"),"title":r.get::<Option<String>,_>("title").map(|s|decode(&s)).transpose()?,
        "infohash":r.get::<Option<String>,_>("infohash"),"state":r.get::<String,_>("state"),"error":r.get::<Option<String>,_>("error"),
        "release_title":release["title"],"indexer_name":release["indexer_name"],"release_size":release["size"],
        "imported":sqlx::query_scalar::<_,bool>("SELECT EXISTS(SELECT 1 FROM acquisition_imports WHERE download_id=? AND state='scanned')").bind(id).fetch_one(&a.pool).await.map_err(db)?,
        "download_limit_kib":data["download_limit_kib"].as_u64().unwrap_or(0),"import_blocker":import_blocker,
        "stats":data["stats"],"files":data["files"],"import_warning":data["import_warning"],"queue_reason":data["queue_reason"],"selected_files":data["input"]["selected_files"],"seed_seconds":r.get::<i64,_>("seed_seconds"),"created_at":r.get::<i64,_>("created_at")}),
    )
}
pub(super) async fn read(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    access::require(&a, &h, "downloads").await?;
    Ok(Json(json!({"download":view(&a,&id).await?})))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Action {
    pub action: String,
    pub selected_files: Option<Vec<usize>>,
}
pub(super) async fn control(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    Json(v): Json<Action>,
) -> Result<Json<Value>> {
    access::require(&a, &h, "downloads").await?;
    let _gate = a.acquisition.gate.lock().await;
    let r = sqlx::query(
        "SELECT infohash,data,state FROM acquisition_downloads WHERE id=? AND state!='removed'",
    )
    .bind(&id)
    .fetch_optional(&a.pool)
    .await
    .map_err(db)?
    .ok_or_else(|| bad("unknown download"))?;
    let mut data = decode(r.get("data"))?;
    let state = match v.action.as_str() {
        "pause" => "paused",
        "resume" | "retry" => "queued",
        "remove" => "removed",
        "select" => "paused",
        _ => return Err(bad("unknown download action")),
    };
    if v.action == "select" {
        let files = v
            .selected_files
            .filter(|v| !v.is_empty() && v.len() <= 10000)
            .ok_or_else(|| bad("select at least one file"))?;
        let n = data["files"].as_array().map_or(0, Vec::len);
        if files.iter().any(|i| *i >= n) {
            return Err(bad("file index outside torrent"));
        }
        data["input"]["selected_files"] = json!(files);
        data["input"]["only_wanted"] = json!(false);
    }
    if let Some(session) = a.acquisition.current().await
        && let Some(handle) = find(&session, r.get::<Option<String>, _>("infohash").as_deref())
    {
        if v.action == "remove" || v.action == "retry" {
            session
                .delete(handle.id().into(), false)
                .await
                .map_err(|_| bad("could not remove torrent"))?;
        } else {
            let _ = session.pause(&handle).await;
            if v.action == "select" {
                let files: Vec<usize> =
                    serde_json::from_value(data["input"]["selected_files"].clone()).map_err(bad)?;
                session
                    .update_only_files(&handle, &files.into_iter().collect())
                    .await
                    .map_err(|_| bad("could not select files"))?;
            }
        }
    }
    sqlx::query("UPDATE acquisition_downloads SET state=?,data=?,error=NULL,retry_at=0,attempts=CASE WHEN ?='retry' THEN 0 ELSE attempts END,updated_at=unixepoch() WHERE id=?")
        .bind(state).bind(data.to_string()).bind(&v.action).bind(&id).execute(&a.pool).await.map_err(db)?;
    event(&a, &id, &format!("Download {}", v.action)).await?;
    Ok(Json(json!({"download":view(&a,&id).await?})))
}
