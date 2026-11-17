use super::*;
use crate::library::{response, roots as service};

pub(crate) async fn roots(a: &App, id: &str) -> Result<Vec<String>> {
    service::roots(&a.pool, id).await.map_err(response)
}
pub(crate) async fn item_root(a: &App, id: &str, path: &std::path::Path) -> Result<PathBuf> {
    service::item_root(&a.pool, id, path)
        .await
        .map_err(response)
}
pub(crate) async fn contains(a: &App, id: &str, path: &std::path::Path) -> Result<bool> {
    Ok(item_root(a, id, path).await.is_ok())
}
pub(crate) async fn libraries(State(a): State<App>, h: HeaderMap) -> Result<Json<Value>> {
    users::admin(&a, &h).await?;
    Ok(Json(
        json!({"libraries":service::list(&a.pool).await.map_err(response)?}),
    ))
}
pub(crate) async fn add_library(
    State(a): State<App>,
    h: HeaderMap,
    Json(v): Json<Value>,
) -> Result<Json<Value>> {
    users::admin(&a, &h).await?;
    save(&a, None, v).await
}
pub(crate) async fn update(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    Json(v): Json<Value>,
) -> Result<Json<Value>> {
    users::admin(&a, &h).await?;
    save(&a, Some(id), v).await
}
async fn save(a: &App, id: Option<String>, v: Value) -> Result<Json<Value>> {
    let _permit = a
        .scan
        .clone()
        .try_acquire_owned()
        .map_err(|_| bad("scan already running"))?;
    let protected = protected_roots(a).await?;
    Ok(Json(
        json!({"library":service::save(&a.pool,id,v,&protected).await.map_err(response)?}),
    ))
}
async fn protected_roots(a: &App) -> Result<Vec<PathBuf>> {
    let mut protected = vec![
        tokio::fs::canonicalize(std::env::current_dir().map_err(bad)?)
            .await
            .map_err(bad)?,
    ];
    if let Some(source) = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
    {
        protected.push(source.to_path_buf());
    }
    if let Some(data) = a.artwork_cache.parent() {
        protected.push(data.to_path_buf());
    }
    if let Some(scratch) = std::env::var_os("OXEVU_TRANSCODE_DIR") {
        protected.push(scratch.into());
    }
    protected.push(a.playback.scratch.clone());
    Ok(protected)
}
pub(crate) async fn remove(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode> {
    users::admin(&a, &h).await?;
    let _permit = a
        .scan
        .clone()
        .try_acquire_owned()
        .map_err(|_| bad("scan already running"))?;
    service::remove(&a.pool, &id).await.map_err(response)?;
    Ok(StatusCode::NO_CONTENT)
}
