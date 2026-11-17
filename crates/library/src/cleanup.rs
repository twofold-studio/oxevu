//! Operator-requested cleanup of verified duplicates and release samples.
use super::{Result, invalid as bad};
use sqlx::SqlitePool;
use std::path::Path;
#[cfg(test)]
#[path = "cleanup_tests.rs"]
mod tests;

pub async fn merge_identical(pool: &SqlitePool, keep: &str, duplicate: &str) -> Result<()> {
    let left = oxevu_db::catalog::source(pool, keep)
        .await?
        .ok_or_else(|| bad("unknown retained item"))?;
    let right = oxevu_db::catalog::source(pool, duplicate)
        .await?
        .ok_or_else(|| bad("unknown duplicate item"))?;
    super::roots::item_root(pool, keep, Path::new(&left.path)).await?;
    super::roots::item_root(pool, duplicate, Path::new(&right.path)).await?;
    let left_path = left.path;
    let right_path = right.path.clone();
    let identical = tokio::task::spawn_blocking(move || -> std::io::Result<bool> {
        let a = std::fs::metadata(&left_path)?;
        let b = std::fs::metadata(&right_path)?;
        Ok(a.is_file()
            && b.is_file()
            && a.len() == b.len()
            && oxevu_filesystem::hash(Path::new(&left_path), &|| false)?
                == oxevu_filesystem::hash(Path::new(&right_path), &|| false)?)
    })
    .await
    .map_err(bad)?
    .map_err(bad)?;
    if !identical {
        return Err(bad("files differ; refusing a duplicate merge"));
    }
    oxevu_db::catalog_cleanup::merge(pool, keep, duplicate, &right.path).await?;
    Ok(())
}

pub async fn exclude_sample(pool: &SqlitePool, id: &str) -> Result<()> {
    let item = oxevu_db::catalog::source(pool, id)
        .await?
        .ok_or_else(|| bad("unknown sample"))?;
    if !super::scan_policy::sample(Path::new(&item.path)) {
        return Err(bad("file is not explicitly named as a sample"));
    }
    oxevu_db::catalog_cleanup::exclude_sample(pool, id, &item.path).await?;
    Ok(())
}
