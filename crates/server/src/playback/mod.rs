use crate::*;
pub(crate) mod runtime;
mod worker;
pub(crate) use runtime::Runtime;
use tokio::sync::watch;

pub(crate) async fn capabilities(State(a): State<App>, h: HeaderMap) -> Result<Json<Value>> {
    let u = user(&a, &h).await?;
    Ok(Json(
        json!({"capabilities":runtime::capabilities(&a,u["is_admin"]==true).await}),
    ))
}
pub(crate) async fn create(
    State(a): State<App>,
    h: HeaderMap,
    Json(v): Json<Value>,
) -> Result<Json<Value>> {
    let item = v["item_id"]
        .as_str()
        .ok_or_else(|| bad("item_id required"))?
        .to_owned();
    let u = users::authorize_item(&a, &h, &item, true).await?;
    let start = v["start_seconds"]
        .as_f64()
        .filter(|v| v.is_finite() && *v >= 0.)
        .ok_or_else(|| bad("invalid start_seconds"))?;
    let mode = v["mode"]
        .as_str()
        .filter(|m| matches!(*m, "auto" | "direct" | "transcode"))
        .ok_or_else(|| bad("invalid mode"))?;
    let supported = v["direct_play_supported"]
        .as_bool()
        .ok_or_else(|| bad("direct_play_supported required"))?;
    let row = sqlx::query("SELECT path,data FROM items WHERE id=?")
        .bind(&item)
        .fetch_one(&a.pool)
        .await
        .map_err(db)?;
    let path = tokio::fs::canonicalize(row.get::<String, _>("path"))
        .await
        .map_err(|_| bad("source unavailable"))?;
    libraries::item_root(&a, &item, &path).await?;
    let data: Value = serde_json::from_str(row.get("data")).map_err(bad)?;
    let duration = data["duration_seconds"].as_f64();
    if duration.is_some_and(|d| start >= d) {
        return Err(bad("start offset exceeds duration"));
    }
    let settings = settings::load(&a).await?;
    let direct = mode == "direct" || (mode == "auto" && supported);
    let method = if direct {
        "direct"
    } else {
        if u["can_transcode"] != true {
            return Err(users::forbidden());
        }
        if !settings.transcoding_enabled {
            return Err(bad("transcoding disabled"));
        }
        if mode == "auto" && data["video_codec"] == "h264" {
            "remux"
        } else if settings.hardware_acceleration == "nvenc" && settings.hardware_encoding {
            "nvenc"
        } else {
            "cpu"
        }
    };
    let id = uuid::Uuid::new_v4().to_string();
    let (cancel, rx) = watch::channel(false);
    let (done, done_rx) = watch::channel(direct);
    let value = json!({"id":id,"item_id":item,"method":method,"status":if direct{"ready"}else{"starting"},"stream_url":if direct{format!("/api/v1/items/{item}/stream")}else{format!("/api/v1/playback/sessions/{id}/hls/index.m3u8")},"start_seconds":start,"duration_seconds":duration,"reason":if direct{"Native playback requested"}else{"H264/AAC HLS delivery"},"error":null});
    let mut sessions = a.playback.sessions.lock().await;
    if a.playback
        .shutting_down
        .load(std::sync::atomic::Ordering::SeqCst)
    {
        return Err(bad("server shutting down"));
    }
    if sessions.len() >= 128
        || (!direct
            && sessions
                .values()
                .filter(|e| e.transcode && !*e.done.borrow())
                .count()
                >= settings.max_concurrent_transcodes as usize)
    {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({"error":"playback session limit reached"})),
        ));
    }
    sessions.insert(
        id.clone(),
        runtime::Entry {
            pid: None,
            owner: u["id"].as_str().unwrap_or_default().into(),
            item,
            value: value.clone(),
            touched: std::time::Instant::now(),
            idle: settings.session_idle_seconds,
            transcode: !direct,
            cancel,
            done: done_rx,
        },
    );
    drop(sessions);
    if !direct {
        let a = a.clone();
        let id = id.clone();
        let method = method.to_owned();
        tokio::spawn(async move {
            worker::run(a, id, path, start, method, settings, rx, done).await;
        });
    }
    Ok(Json(json!({"session":value})))
}
async fn authorize(a: &App, h: &HeaderMap, id: &str, inspect: bool) -> Result<Value> {
    let u = user(a, h).await?;
    let sessions = a.playback.sessions.lock().await;
    let e = sessions
        .get(id)
        .ok_or_else(|| bad("unknown playback session"))?;
    if e.owner != u["id"].as_str().unwrap_or_default() && !(inspect && u["is_admin"] == true) {
        return Err(users::forbidden());
    }
    let item = e.item.clone();
    let value = e.value.clone();
    drop(sessions);
    users::authorize_item(a, h, &item, true).await?;
    if value["method"] != "direct" && u["can_transcode"] != true {
        return Err(users::forbidden());
    }
    Ok(value)
}
pub(crate) async fn get(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    Ok(Json(json!({"session":authorize(&a,&h,&id,true).await?})))
}
pub(crate) async fn heartbeat(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode> {
    authorize(&a, &h, &id, false).await?;
    if let Some(e) = a.playback.sessions.lock().await.get_mut(&id) {
        e.touched = std::time::Instant::now()
    }
    Ok(StatusCode::NO_CONTENT)
}
pub(crate) async fn stop(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode> {
    authorize(&a, &h, &id, false).await?;
    stop_id(&a, &id).await;
    Ok(StatusCode::NO_CONTENT)
}
async fn stop_id(a: &App, id: &str) {
    let entry = a.playback.sessions.lock().await.remove(id);
    if let Some(mut e) = entry {
        let _ = e.cancel.send(true);
        while !*e.done.borrow() {
            if e.done.changed().await.is_err() {
                break;
            }
        }
    }
    let _ = tokio::fs::remove_dir_all(a.playback.scratch.join(id)).await;
}
pub(crate) async fn hls(
    State(a): State<App>,
    h: HeaderMap,
    Path((id, file)): Path<(String, String)>,
    r: Request,
) -> Result<Response> {
    authorize(&a, &h, &id, false).await?;
    if file != "index.m3u8"
        && !(file.starts_with("segment")
            && file.ends_with(".ts")
            && file[7..file.len() - 3].bytes().all(|b| b.is_ascii_digit()))
    {
        return Err(bad("invalid segment name"));
    }
    let directory = a.playback.scratch.join(&id);
    let path = tokio::fs::canonicalize(directory.join(&file))
        .await
        .map_err(|_| bad("segment unavailable"))?;
    if !path.starts_with(&directory) {
        return Err(users::forbidden());
    }
    let mime = if file.ends_with("m3u8") {
        "application/vnd.apple.mpegurl"
    } else {
        "video/mp2t"
    };
    Ok(
        tower_http::services::ServeFile::new_with_mime(path, &mime.parse().map_err(bad)?)
            .try_call(r)
            .await
            .map_err(bad)?
            .into_response(),
    )
}
pub(crate) async fn reap(a: App) {
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        let ids: Vec<String> = a
            .playback
            .sessions
            .lock()
            .await
            .iter()
            .filter(|(_, e)| e.touched.elapsed().as_secs() > u64::from(e.idle))
            .map(|(id, _)| id.clone())
            .collect();
        for id in ids {
            stop_id(&a, &id).await;
        }
    }
}

#[cfg(test)]
mod tests;
