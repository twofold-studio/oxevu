use super::*;
use librqbit::{ManagedTorrent, Session};

pub(crate) async fn run(a: App) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(4));
    loop {
        interval.tick().await;
        if a.acquisition.is_stopping() {
            break;
        }
        if let Err(e) = tick(&a).await {
            let message = e.1.0["error"]
                .as_str()
                .unwrap_or("acquisition worker failed")
                .to_owned();
            let _ = event(&a, "worker", &message).await;
        }
    }
}
pub(super) async fn tick(a: &App) -> Result<()> {
    let c = config::load(a).await?;
    if !c.enabled {
        return Ok(());
    }
    storage::validate_download_root(a, std::path::Path::new(&c.download_dir)).await?;
    let rows=sqlx::query("SELECT * FROM acquisition_downloads WHERE state IN ('queued','checking','downloading','seeding','error') AND retry_at<=unixepoch() ORDER BY created_at,id LIMIT 1000").fetch_all(&a.pool).await.map_err(db)?;
    if rows.is_empty() {
        return Ok(());
    }
    let session = {
        let _gate = a.acquisition.gate.lock().await;
        let latest = config::load(a).await?;
        if serde_json::to_value(&latest).map_err(bad)? != serde_json::to_value(&c).map_err(bad)? {
            return Ok(());
        }
        a.acquisition.engine(&c).await?
    };
    let low_space = storage::free_bytes(PathBuf::from(&c.download_dir)).await?
        < c.min_free_gb as u64 * 1024 * 1024 * 1024
        || storage::used_bytes(PathBuf::from(&c.download_dir)).await?
            >= c.storage_limit_gb as u64 * 1024 * 1024 * 1024;
    for r in rows {
        if a.acquisition.is_stopping() {
            break;
        }
        let id: String = r.get("id");
        let state: String = r.get("state");
        let data = decode(r.get("data"))?;
        let mut handle =
            downloads::find(&session, r.get::<Option<String>, _>("infohash").as_deref());
        if handle.is_none() {
            if active_count(&session, None) >= c.max_active_downloads
                || low_space
                || (state == "error" && r.get::<i64, _>("attempts") >= 3)
            {
                continue;
            }
            match transfer::prepare(a, &c, &session, &id, data).await {
                Ok(found) => handle = found,
                Err(e) => {
                    fail(a, &id, &e.1.0).await?;
                    continue;
                }
            }
        }
        if let Some(handle) = handle {
            match reconcile(
                a,
                &c,
                &session,
                &id,
                &handle,
                active_count(&session, Some(handle.id())) < c.max_active_downloads && !low_space,
            )
            .await
            {
                Ok(_) => {}
                Err(e) => {
                    let _ = session.pause(&handle).await;
                    let _ = session.delete(handle.id().into(), false).await;
                    fail(a, &id, &e.1.0).await?;
                }
            }
        }
    }
    Ok(())
}

fn active_count(session: &Session, exclude: Option<usize>) -> u32 {
    session.with_torrents(|torrents| {
        let mut n = 0;
        for (_, h) in torrents {
            let s = h.stats();
            if Some(h.id()) != exclude
                && !s.finished
                && matches!(s.state.to_string().as_str(), "live" | "initializing")
            {
                n += 1;
            }
        }
        n
    })
}

async fn reconcile(
    a: &App,
    c: &config::Config,
    session: &Arc<Session>,
    id: &str,
    handle: &Arc<ManagedTorrent>,
    can_start: bool,
) -> Result<bool> {
    let _gate = a.acquisition.gate.lock().await;
    if !config::load(a).await?.enabled
        || a.acquisition.is_stopping()
        || !a
            .acquisition
            .current()
            .await
            .is_some_and(|current| Arc::ptr_eq(&current, session))
    {
        return Ok(false);
    }
    let row = sqlx::query(
        "SELECT state,data,updated_at,seed_seconds FROM acquisition_downloads WHERE id=?",
    )
    .bind(id)
    .fetch_one(&a.pool)
    .await
    .map_err(db)?;
    let intent: String = row.get("state");
    if intent == "removed" {
        session
            .delete(handle.id().into(), false)
            .await
            .map_err(|_| bad("torrent cleanup failed"))?;
        return Ok(false);
    }
    if intent == "paused" {
        let _ = session.pause(handle).await;
        return Ok(false);
    }
    let mut data = decode(row.get("data"))?;
    let stats = handle.stats();
    if stats.error.is_some() {
        return Err(bad(
            "torrent engine reported a transfer error; retry or recheck storage",
        ));
    }
    let raw_state = stats.state.to_string();
    let mut state = if stats.finished {
        "seeding"
    } else if raw_state == "initializing" {
        "checking"
    } else {
        "downloading"
    };
    let mut started = false;
    if raw_state == "paused" && !stats.finished && !can_start {
        state = "queued";
    }
    if raw_state == "paused" && (stats.finished || can_start) {
        session
            .unpause(handle)
            .await
            .map_err(|_| bad("could not resume torrent"))?;
        started = !stats.finished;
    } else if !stats.finished && !can_start && raw_state == "live" {
        session
            .pause(handle)
            .await
            .map_err(|_| bad("could not enforce queue limit"))?;
        state = "queued";
    }
    let now: i64 = sqlx::query_scalar("SELECT unixepoch()")
        .fetch_one(&a.pool)
        .await
        .map_err(db)?;
    let mut seed_seconds: i64 = row.get("seed_seconds");
    if stats.finished && raw_state == "live" {
        seed_seconds += (now - row.get::<i64, _>("updated_at")).clamp(0, 30);
    }
    let uploaded = stats
        .uploaded_bytes
        .saturating_add(data["uploaded_base"].as_u64().unwrap_or(0));
    let ratio = uploaded as f64 / stats.total_bytes.max(1) as f64;
    let ratio_done = c.seed_ratio <= 0.0 || ratio >= c.seed_ratio;
    let time_done = c.seed_hours == 0 || seed_seconds >= c.seed_hours as i64 * 3600;
    if stats.finished && (c.seed_ratio > 0.0 || c.seed_hours > 0) && ratio_done && time_done {
        let _ = session.pause(handle).await;
        state = "completed";
    }
    data["stats"] = serde_json::to_value(&stats).map_err(bad)?;
    data["stats"]["uploaded_bytes"] = json!(uploaded);
    data["queue_reason"] = if state == "queued" {
        json!("Waiting for queue capacity or free storage")
    } else {
        Value::Null
    };
    sqlx::query("UPDATE acquisition_downloads SET state=?,data=json_set(data,'$.stats',json(?),'$.queue_reason',json(?)),error=NULL,seed_seconds=?,updated_at=unixepoch() WHERE id=? AND state NOT IN ('removed','paused')")
        .bind(state).bind(data["stats"].to_string()).bind(data["queue_reason"].to_string()).bind(seed_seconds).bind(id).execute(&a.pool).await.map_err(db)?;
    Ok(started)
}

async fn fail(a: &App, id: &str, error: &Value) -> Result<()> {
    let message = error["error"].as_str().unwrap_or("transfer failed");
    sqlx::query("UPDATE acquisition_downloads SET state='error',error=?,attempts=attempts+1,retry_at=unixepoch()+MIN(3600,60*(attempts+1)*(attempts+1)),updated_at=unixepoch() WHERE id=? AND state NOT IN ('paused','removed')")
        .bind(message).bind(id).execute(&a.pool).await.map_err(db)?;
    sqlx::query("UPDATE acquisition_releases SET blocked_until=unixepoch()+86400 WHERE id=(SELECT release_id FROM acquisition_downloads WHERE id=? AND attempts>=3)").bind(id).execute(&a.pool).await.map_err(db)?;
    event(a, id, message).await
}
