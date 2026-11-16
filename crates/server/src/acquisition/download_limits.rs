use super::*;
use librqbit::limits::LimitsConfig;
use std::num::NonZeroU32;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Input {
    pub download_limit_kib: u32,
}

pub(super) fn engine_limits(data: &Value) -> LimitsConfig {
    LimitsConfig {
        download_bps: data["download_limit_kib"]
            .as_u64()
            .and_then(|v| u32::try_from(v).ok())
            .and_then(|v| v.checked_mul(1024))
            .and_then(NonZeroU32::new),
        upload_bps: None,
    }
}

pub(super) async fn put(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<Input>,
) -> Result<Json<Value>> {
    access::require(&a, &h, "downloads").await?;
    let limit = input.download_limit_kib;
    if limit > 1_000_000 || (limit > 0 && limit < 16) {
        return Err(bad("download limit must be zero or 16–1000000 KiB/s"));
    }
    let _gate = a.acquisition.gate.lock().await;
    let row = sqlx::query(
        "SELECT infohash,data,state FROM acquisition_downloads WHERE id=? AND state!='removed'",
    )
    .bind(&id)
    .fetch_optional(&a.pool)
    .await
    .map_err(db)?
    .ok_or_else(|| bad("unknown download"))?;
    let mut data = decode(row.get("data"))?;
    if data["download_limit_kib"].as_u64().unwrap_or(0) == u64::from(limit) {
        return Ok(Json(json!({"download":downloads::view(&a,&id).await?})));
    }
    let mut state: String = row.get("state");
    if let Some(session) = a.acquisition.current().await
        && let Some(handle) = downloads::find(
            &session,
            row.get::<Option<String>, _>("infohash").as_deref(),
        )
    {
        // rqbit 9 exposes per-torrent limits only at creation. Keep files and
        // durable intent, then re-add/recheck with the new limit on the next tick.
        let _ = session.pause(&handle).await;
        let stats = handle.stats();
        let uploaded = stats
            .uploaded_bytes
            .saturating_add(data["uploaded_base"].as_u64().unwrap_or(0));
        data["stats"] = serde_json::to_value(stats).map_err(bad)?;
        data["stats"]["uploaded_bytes"] = json!(uploaded);
        session
            .delete(handle.id().into(), false)
            .await
            .map_err(|_| bad("could not reconnect transfer with the new limit"))?;
        if matches!(state.as_str(), "checking" | "downloading" | "seeding") {
            state = "queued".into();
        }
    }
    sqlx::query(
        "UPDATE acquisition_downloads SET data=json_set(data,'$.download_limit_kib',?,'$.stats',json(?)),state=?,updated_at=unixepoch() WHERE id=?",
    )
    .bind(limit)
    .bind(data["stats"].to_string())
    .bind(state)
    .bind(&id)
    .execute(&a.pool)
    .await
    .map_err(db)?;
    event(
        &a,
        &id,
        if limit == 0 {
            "Entry download limit removed; combined limit still applies"
        } else {
            "Entry download limit updated"
        },
    )
    .await?;
    Ok(Json(json!({"download":downloads::view(&a,&id).await?})))
}
