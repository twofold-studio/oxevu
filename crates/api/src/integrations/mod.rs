use crate::*;
mod http;
use serde::{Deserialize, Serialize};
mod metadata;
mod metadata_worker;
pub(crate) use metadata::tmdb;
pub(crate) use metadata_worker::run as metadata_worker;
mod opensubtitles;
mod studios;
pub(crate) use metadata::{Match, Search};
pub(crate) use metadata::{apply, artwork as metadata_artwork, search};
pub(crate) use opensubtitles::{subtitle_download, subtitle_search};
pub(crate) use studios::studio;

#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub(super) struct Provider {
    pub enabled: bool,
    pub api_key: String,
    pub username: String,
    pub password: String,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub(super) struct Config {
    pub tmdb: Provider,
    pub omdb: Provider,
    pub opensubtitles: Provider,
    pub studio_images: Value,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            tmdb: Provider::default(),
            omdb: Provider::default(),
            opensubtitles: Provider::default(),
            studio_images: json!({"enabled":true}),
        }
    }
}
fn path(a: &App) -> Result<PathBuf> {
    Ok(a.artwork_cache
        .parent()
        .ok_or_else(|| bad("data directory unavailable"))?
        .join("integrations.json"))
}
pub(crate) async fn load(a: &App) -> Result<Config> {
    match tokio::fs::read(path(a)?).await {
        Ok(bytes) => {
            serde_json::from_slice(&bytes).map_err(|_| bad("invalid integration configuration"))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Config::default()),
        Err(_) => Err(bad("cannot read integration configuration")),
    }
}
fn response(config: &Config) -> Value {
    let provider = |p: &Provider, account: bool| json!({"enabled":p.enabled,"configured":!p.api_key.is_empty() && (!account || (!p.username.is_empty()&&!p.password.is_empty())),"api_key_set":!p.api_key.is_empty(),"username":p.username,"password_set":!p.password.is_empty()});
    json!({"integrations":{"tmdb":provider(&config.tmdb,false),"omdb":provider(&config.omdb,false),"opensubtitles":provider(&config.opensubtitles,true),"studio_images":config.studio_images},"requirements":{"tmdb":"Application API key required; attribution required","omdb":"API key required","opensubtitles":"Application API key, username and password required","studio_images":"Public studio artwork, CC-BY-SA 4.0; no key required"}})
}
pub(crate) async fn get(State(a): State<App>, h: HeaderMap) -> Result<Json<Value>> {
    users::admin(&a, &h).await?;
    Ok(Json(response(&load(&a).await?)))
}
pub(crate) async fn put(
    State(a): State<App>,
    h: HeaderMap,
    Json(v): Json<Value>,
) -> Result<Json<Value>> {
    users::admin(&a, &h).await?;
    // Share the bounded single-writer job gate so concurrent partial edits cannot lose credentials.
    let _permit = a
        .artwork_jobs
        .clone()
        .acquire_owned()
        .await
        .map_err(|_| bad("server shutting down"))?;
    let mut c = load(&a).await?;
    let v = v.get("integrations").unwrap_or(&v);
    for (name, p) in [
        ("tmdb", &mut c.tmdb),
        ("omdb", &mut c.omdb),
        ("opensubtitles", &mut c.opensubtitles),
    ] {
        if let Some(value) = v.get(name) {
            if value["clear_credentials"] == true {
                p.api_key.clear();
                p.password.clear();
                p.username.clear();
            }
            if let Some(enabled) = value.get("enabled") {
                p.enabled = enabled
                    .as_bool()
                    .ok_or_else(|| bad("invalid integration enabled flag"))?;
            }
            for (field, target) in [
                ("api_key", &mut p.api_key),
                ("username", &mut p.username),
                ("password", &mut p.password),
            ] {
                if let Some(value) = value.get(field) {
                    let value = value
                        .as_str()
                        .filter(|s| s.len() <= 4096)
                        .ok_or_else(|| bad("invalid integration credential"))?;
                    if !value.is_empty() {
                        *target = value.to_owned();
                    }
                }
            }
        }
    }
    if let Some(studios) = v.get("studio_images") {
        c.studio_images = json!({"enabled":studios["enabled"].as_bool().ok_or_else(||bad("invalid studio images flag"))?});
    }
    let target = path(&a)?;
    if let Some(parent) = target.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|_| bad("cannot create data directory"))?;
    }
    let temp = target.with_extension(format!("{}.tmp", uuid::Uuid::new_v4()));
    use tokio::io::AsyncWriteExt;
    let mut file = tokio::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(&temp)
        .await
        .map_err(|_| bad("cannot save integration configuration"))?;
    let result = async {
        file.write_all(
            &serde_json::to_vec(&c).map_err(|_| bad("cannot encode integration configuration"))?,
        )
        .await
        .map_err(|_| bad("cannot save integration configuration"))?;
        file.sync_all()
            .await
            .map_err(|_| bad("cannot save integration configuration"))?;
        tokio::fs::rename(&temp, &target)
            .await
            .map_err(|_| bad("cannot save integration configuration"))
    }
    .await;
    if result.is_err() {
        let _ = tokio::fs::remove_file(temp).await;
    }
    result?;
    oxevu_db::metadata_jobs::retry(&a.pool).await.map_err(db)?;
    Ok(Json(response(&c)))
}
pub(crate) fn client() -> Result<reqwest::Client> {
    http::client().map_err(|_| bad("HTTP client unavailable"))
}
pub(crate) async fn bytes(mut response: reqwest::Response, limit: usize) -> Result<Vec<u8>> {
    if !response.status().is_success() {
        return Err(bad(format!(
            "provider returned HTTP {}",
            response.status().as_u16()
        )));
    }
    if response.content_length().is_some_and(|n| n > limit as u64) {
        return Err(bad("provider response too large"));
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| bad("provider response failed"))?
    {
        if bytes.len() + chunk.len() > limit {
            return Err(bad("provider response too large"));
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}
pub(super) async fn json_response(response: reqwest::Response) -> Result<Value> {
    serde_json::from_slice(&bytes(response, 2 * 1024 * 1024).await?)
        .map_err(|_| bad("invalid provider response"))
}
pub(super) async fn item_data(a: &App, h: &HeaderMap, id: &str) -> Result<Value> {
    users::authorize_item(a, h, id, false).await?;
    let data: String = sqlx::query_scalar("SELECT data FROM items WHERE id=?")
        .bind(id)
        .fetch_one(&a.pool)
        .await
        .map_err(db)?;
    serde_json::from_str(&data).map_err(|_| bad("invalid item metadata"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn credentials_are_admin_only_redacted_and_private() {
        let a = crate::tests::app().await;
        let admin = crate::tests::account(&a, "admin", true).await;
        let viewer = crate::tests::account(&a, "viewer", false).await;
        assert!(get(State(a.clone()), viewer).await.is_err());
        let result=put(State(a.clone()),admin.clone(),Json(json!({"opensubtitles":{"enabled":true,"api_key":"private-key","username":"test-account","password":"private-password"}}))).await.unwrap().0;
        let serialized = result.to_string();
        assert!(!serialized.contains("private-key"));
        assert!(!serialized.contains("private-password"));
        assert_eq!(result["integrations"]["opensubtitles"]["configured"], true);
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            tokio::fs::metadata(path(&a).unwrap())
                .await
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        let _ = put(
            State(a.clone()),
            admin.clone(),
            Json(json!({"opensubtitles":{"password":"","enabled":false}})),
        )
        .await
        .unwrap();
        assert_eq!(
            load(&a).await.unwrap().opensubtitles.password,
            "private-password"
        );
        let _ = put(
            State(a.clone()),
            admin,
            Json(json!({"opensubtitles":{"clear_credentials":true}})),
        )
        .await
        .unwrap();
        assert!(load(&a).await.unwrap().opensubtitles.password.is_empty());
        let _ = tokio::fs::remove_file(path(&a).unwrap()).await;
    }
}
