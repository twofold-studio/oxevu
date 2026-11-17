use super::{Error, Result, invalid as bad};
use oxevu_db::libraries as repository;
use serde_json::{Value, json};
use sqlx::SqlitePool;
use std::path::{Path, PathBuf};

pub async fn roots(pool: &SqlitePool, id: &str) -> Result<Vec<String>> {
    repository::roots(pool, id).await.map_err(Error::Storage)
}
pub async fn item_root(pool: &SqlitePool, id: &str, path: &Path) -> Result<PathBuf> {
    let library = repository::item_library(pool, id)
        .await
        .map_err(Error::Storage)?;
    let path = tokio::fs::canonicalize(path).await.map_err(bad)?;
    for root in roots(pool, &library).await? {
        if let Ok(root) = tokio::fs::canonicalize(root).await
            && path.starts_with(&root)
        {
            return Ok(root);
        }
    }
    Err(bad("file outside library"))
}
async fn representation(pool: &SqlitePool, id: &str) -> Result<Value> {
    let row = repository::get(pool, id).await.map_err(Error::Storage)?;
    Ok(
        json!({"id":id,"name":row.name,"kind":row.kind,"path":row.path,"paths":roots(pool,id).await?,"enabled":row.enabled}),
    )
}
pub async fn list(pool: &SqlitePool) -> Result<Vec<Value>> {
    let mut result = Vec::new();
    for row in repository::list(pool).await.map_err(Error::Storage)? {
        result.push(representation(pool, &row.id).await?);
    }
    Ok(result)
}
pub async fn save(
    pool: &SqlitePool,
    id: Option<String>,
    v: Value,
    protected: &[PathBuf],
) -> Result<Value> {
    let name = v["name"]
        .as_str()
        .filter(|s| !s.trim().is_empty() && s.len() <= 200)
        .ok_or_else(|| bad("invalid name"))?;
    let kind = v["kind"]
        .as_str()
        .filter(|s| matches!(*s, "movies" | "tv"))
        .ok_or_else(|| bad("invalid kind"))?;
    let enabled = v["enabled"].as_bool().unwrap_or(true);
    let input: Vec<String> = if v.get("paths").is_some() {
        serde_json::from_value(v["paths"].clone()).map_err(bad)?
    } else {
        vec![
            v["path"]
                .as_str()
                .ok_or_else(|| bad("paths required"))?
                .into(),
        ]
    };
    if input.is_empty() || input.len() > 32 {
        return Err(bad("provide 1–32 roots"));
    }
    let existing = id.is_some();
    let id = id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let other = repository::other_roots(pool, &id)
        .await
        .map_err(Error::Storage)?;
    let mut paths: Vec<PathBuf> = Vec::new();
    for input in input {
        let path = tokio::fs::canonicalize(input)
            .await
            .map_err(|_| bad("root must be an accessible directory"))?;
        if !tokio::fs::metadata(&path).await.map_err(bad)?.is_dir() {
            return Err(bad("root must be directory"));
        }
        for other in paths
            .iter()
            .cloned()
            .chain(other.iter().map(PathBuf::from))
            .chain(protected.iter().cloned())
        {
            let other = tokio::fs::canonicalize(&other).await.unwrap_or(other);
            if path.starts_with(&other) || other.starts_with(&path) {
                return Err(bad("overlapping or protected root"));
            }
        }
        paths.push(path);
    }

    let saved = repository::save(
        pool,
        repository::Save {
            id: &id,
            name,
            kind,
            enabled,
            paths: &paths,
            existing,
        },
    )
    .await
    .map_err(Error::Storage)?;
    if !saved {
        return Err(bad("unknown library"));
    }
    representation(pool, &id).await
}
pub async fn remove(pool: &SqlitePool, id: &str) -> Result<()> {
    repository::remove(pool, id).await.map_err(Error::Storage)
}
