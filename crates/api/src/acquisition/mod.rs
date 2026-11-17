use crate::*;
use axum::extract::Query;
use serde::{Deserialize, Serialize};

mod access;
mod adoption;
mod auto_import;
#[cfg(test)]
mod automatic_import_tests;
mod catalog;
mod claims;
mod config;
mod destinations;
mod discovery;
mod download_limits;
mod downloading;
mod downloads;
mod feeds;
pub(crate) mod http;
mod import_io;
mod import_plan;
mod imports;
mod indexers;
mod jobs;
mod migration;
mod monitoring;
mod profiles;
mod recognition;
mod release_facts;
use oxevu_acquisition::release_identity;
mod releases;
mod request_status;
mod requests;
mod runtime;
use oxevu_acquisition::scope;
mod storage;
pub(crate) mod streaming;
mod torrent_files;
mod torznab;
mod trailers;
mod transfer;
mod transfer_source;
mod wanted;
pub(crate) use access::{Permissions, permissions};
pub(crate) use auto_import::run as import_worker;
pub(crate) use discovery::Sort as DiscoverSort;
pub(crate) use downloading::annotate as annotate_downloads;
pub(crate) use jobs::run;
pub(crate) use monitoring::run as monitor;
pub(crate) use runtime::Runtime;

#[cfg(test)]
pub(crate) fn routes() -> Router<App> {
    http::router().split_for_parts().0
}

async fn events(State(a): State<App>, h: HeaderMap) -> Result<Json<Value>> {
    access::require(&a, &h, "downloads").await?;
    let rows = sqlx::query("SELECT * FROM acquisition_events ORDER BY id DESC LIMIT 100")
        .fetch_all(&a.pool)
        .await
        .map_err(db)?;
    Ok(Json(
        json!({"events":rows.iter().map(|r|json!({"id":r.get::<i64,_>("id"),"entity_id":r.get::<String,_>("entity_id"),"message":r.get::<String,_>("message"),"created_at":r.get::<i64,_>("created_at")})).collect::<Vec<_>>()}),
    ))
}

pub(super) async fn event(a: &App, entity: &str, message: &str) -> Result<()> {
    sqlx::query("INSERT INTO acquisition_events(entity_id,message) VALUES(?,?)")
        .bind(entity)
        .bind(message)
        .execute(&a.pool)
        .await
        .map_err(db)?;
    sqlx::query("DELETE FROM acquisition_events WHERE id < (SELECT COALESCE(MAX(id),0)-5000 FROM acquisition_events)")
        .execute(&a.pool).await.map_err(db)?;
    Ok(())
}

pub(super) fn decode(raw: &str) -> Result<Value> {
    serde_json::from_str(raw).map_err(|_| bad("invalid acquisition data"))
}

pub(super) fn title_key(kind: &str, id: u64) -> Result<String> {
    if !matches!(kind, "movie" | "tv") || id == 0 || id > i64::MAX as u64 {
        return Err(bad("invalid catalog identity"));
    }
    Ok(format!("tmdb:{kind}:{id}"))
}

#[cfg(test)]
mod tests;
