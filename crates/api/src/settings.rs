use crate::{App, Result, bad, db};
use axum::{Json, extract::State, http::HeaderMap};
pub use oxevu_config::Settings;
use serde_json::{Value, json};

pub async fn load(a: &App) -> Result<Settings> {
    let stored: Option<String> = sqlx::query_scalar("SELECT data FROM server_settings WHERE id=1")
        .fetch_optional(&a.pool)
        .await
        .map_err(db)?;
    stored.map_or_else(
        || Ok(Settings::default()),
        |s| serde_json::from_str(&s).map_err(bad),
    )
}
pub async fn get(State(a): State<App>, h: HeaderMap) -> Result<Json<Value>> {
    crate::users::admin(&a, &h).await?;
    Ok(Json(
        json!({"settings":load(&a).await?,"capabilities":crate::playback::runtime::capabilities(&a,true).await}),
    ))
}
pub async fn put(
    State(a): State<App>,
    h: HeaderMap,
    Json(settings): Json<Settings>,
) -> Result<Json<Value>> {
    crate::users::admin(&a, &h).await?;
    settings.validate().map_err(bad)?;
    sqlx::query("INSERT INTO server_settings(id,data) VALUES(1,?) ON CONFLICT(id) DO UPDATE SET data=excluded.data")
        .bind(serde_json::to_string(&settings).map_err(bad)?)
        .execute(&a.pool).await.map_err(db)?;
    Ok(Json(
        json!({"settings":settings,"capabilities":crate::playback::runtime::capabilities(&a,true).await}),
    ))
}
