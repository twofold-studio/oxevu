use crate::*;
use axum::extract::Query;
use serde::{Deserialize, Serialize};

mod access;
mod adoption;
mod auto_import;
mod catalog;
mod claims;
mod config;
mod downloads;
mod feeds;
mod import_io;
mod import_plan;
mod imports;
mod indexers;
mod jobs;
mod migration;
mod monitoring;
mod profiles;
mod release_facts;
mod release_identity;
mod releases;
mod request_status;
mod requests;
mod runtime;
mod scope;
mod storage;
mod torrent_files;
mod torznab;
mod transfer;
mod transfer_source;
mod wanted;
pub(crate) use access::permissions;
pub(crate) use auto_import::run as import_worker;
pub(crate) use jobs::run;
pub(crate) use monitoring::run as monitor;
pub(crate) use runtime::Runtime;

pub(crate) fn routes() -> Router<App> {
    Router::new()
        .route("/acquisition/release-sources", get(indexers::sources))
        .route(
            "/acquisition/downloads/{id}",
            get(downloads::read).put(downloads::control),
        )
        .route("/acquisition/access", get(access::get))
        .route("/acquisition/migration/preview", post(migration::preview))
        .route("/acquisition/migration", post(migration::apply))
        .route("/acquisition/events", get(events))
        .route("/acquisition/settings", get(config::get).put(config::put))
        .route("/acquisition/profiles", get(profiles::list))
        .route("/acquisition/profiles/{id}", put(profiles::put))
        .route("/acquisition/indexers", get(indexers::list))
        .route(
            "/acquisition/downloads",
            get(downloads::list).post(downloads::create).layer(
                axum::extract::DefaultBodyLimit::max(3 * 1024 * 1024 + 65536),
            ),
        )
        .route(
            "/acquisition/downloads/{id}/adoption/preview",
            post(adoption::preview),
        )
        .route(
            "/acquisition/downloads/{id}/adoption",
            post(adoption::apply),
        )
        .route(
            "/acquisition/downloads/{id}/imports/preview",
            post(imports::preview),
        )
        .route("/acquisition/downloads/{id}/imports", post(imports::apply))
        .route("/acquisition/indexers/{id}", put(indexers::put))
        .route("/acquisition/indexers/{id}/test", post(indexers::test))
        .route("/acquisition/titles/{id}/releases", post(releases::search))
        .route("/acquisition/access/{id}", put(access::put))
        .route("/acquisition/discover", get(catalog::discover))
        .route("/acquisition/artwork/{file}", get(catalog::artwork))
        .route("/acquisition/titles", get(catalog::list))
        .route("/acquisition/monitored", get(monitoring::list))
        .route("/acquisition/monitored/{id}", put(monitoring::put))
        .route("/acquisition/titles/{kind}/{id}", get(catalog::details))
        .route(
            "/acquisition/requests",
            get(requests::list).post(requests::create),
        )
        .route("/acquisition/requests/{id}", put(requests::update))
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
