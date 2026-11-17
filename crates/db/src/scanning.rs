//! Persistence used by read-only library scans.
use sqlx::{Row, SqlitePool};

pub struct File {
    pub id: String,
    pub path: String,
}
pub async fn existing_id(pool: &SqlitePool, path: &str) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar("SELECT id FROM items WHERE path=?")
        .bind(path)
        .fetch_optional(pool)
        .await
}
pub async fn upsert(
    pool: &SqlitePool,
    id: &str,
    library: &str,
    path: &str,
    data: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO items VALUES(?,?,?,?) ON CONFLICT(path) DO UPDATE SET data=excluded.data",
    )
    .bind(id)
    .bind(library)
    .bind(path)
    .bind(data)
    .execute(pool)
    .await?;
    Ok(())
}
pub async fn present(pool: &SqlitePool, library: &str) -> Result<Vec<File>, sqlx::Error> {
    Ok(sqlx::query("SELECT id,path FROM items WHERE library_id=? AND COALESCE(json_extract(data,'$.file_available'),1)!=0")
        .bind(library).fetch_all(pool).await?.into_iter().map(|r| File {id:r.get("id"),path:r.get("path")}).collect())
}
pub async fn remove_missing(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM progress WHERE item_id=?")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM items WHERE id=?")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await
}
