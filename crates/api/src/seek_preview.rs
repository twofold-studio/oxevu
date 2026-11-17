use crate::*;
use axum::http::header;

static JOBS: Semaphore = Semaphore::const_new(2);

pub(crate) async fn serve(
    a: App,
    headers: HeaderMap,
    id: String,
    seconds: u32,
) -> Result<Response> {
    users::authorize_item(&a, &headers, &id, true).await?;
    let row = oxevu_db::catalog::source(&a.pool, &id)
        .await
        .map_err(db)?
        .ok_or_else(|| bad("unknown item"))?;
    let item: Value = serde_json::from_str(&row.data).map_err(bad)?;
    if item["file_available"] == false
        || item["duration_seconds"]
            .as_f64()
            .is_none_or(|duration| f64::from(seconds) >= duration)
    {
        return Err(bad("preview unavailable for this position"));
    }
    let path = tokio::fs::canonicalize(&row.path)
        .await
        .map_err(|_| bad("preview source unavailable"))?;
    libraries::item_root(&a, &id, &path).await?;
    let _permit = JOBS.try_acquire().map_err(|_| {
        (
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({"error":"seek previews are busy"})),
        )
    })?;
    let frame = oxevu_playback::seek_frame::jpeg(&a.playback.ffmpeg, &path, seconds)
        .await
        .map_err(bad)?;
    Ok((
        [
            (header::CONTENT_TYPE, "image/jpeg"),
            (header::CACHE_CONTROL, "no-store"),
        ],
        frame,
    )
        .into_response())
}
