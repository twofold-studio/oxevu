//! Library records and atomic root/catalog updates. This module never touches media.
use sqlx::{Row, SqlitePool};
use std::path::PathBuf;

pub struct Library {
    pub id: String,
    pub name: String,
    pub path: String,
    pub kind: String,
    pub enabled: bool,
}
fn decode(row: sqlx::sqlite::SqliteRow) -> Library {
    Library {
        id: row.get("id"),
        name: row.get("name"),
        path: row.get("path"),
        kind: row.get("kind"),
        enabled: row.get("enabled"),
    }
}
pub async fn list(pool: &SqlitePool) -> Result<Vec<Library>, sqlx::Error> {
    Ok(sqlx::query("SELECT libraries.*,COALESCE(enabled,1) enabled FROM libraries LEFT JOIN library_options ON library_id=libraries.id")
        .fetch_all(pool).await?.into_iter().map(decode).collect())
}
pub async fn get(pool: &SqlitePool, id: &str) -> Result<Library, sqlx::Error> {
    Ok(decode(sqlx::query("SELECT libraries.*,COALESCE(enabled,1) enabled FROM libraries LEFT JOIN library_options ON library_id=libraries.id WHERE libraries.id=?")
        .bind(id).fetch_one(pool).await?))
}
pub async fn roots(pool: &SqlitePool, id: &str) -> Result<Vec<String>, sqlx::Error> {
    let roots =
        sqlx::query_scalar("SELECT path FROM library_roots WHERE library_id=? ORDER BY rowid")
            .bind(id)
            .fetch_all(pool)
            .await?;
    if !roots.is_empty() {
        return Ok(roots);
    }
    sqlx::query_scalar("SELECT path FROM libraries WHERE id=?")
        .bind(id)
        .fetch_all(pool)
        .await
}
pub async fn other_roots(pool: &SqlitePool, id: &str) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar("SELECT path FROM library_roots WHERE library_id!=? UNION SELECT path FROM libraries WHERE id!=?")
        .bind(id).bind(id).fetch_all(pool).await
}
pub async fn item_library(pool: &SqlitePool, item: &str) -> Result<String, sqlx::Error> {
    sqlx::query_scalar("SELECT library_id FROM items WHERE id=?")
        .bind(item)
        .fetch_one(pool)
        .await
}
pub struct Save<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub kind: &'a str,
    pub enabled: bool,
    pub paths: &'a [PathBuf],
    pub existing: bool,
}
/// Return false for an unknown update; all changes are committed together.
pub async fn save(pool: &SqlitePool, input: Save<'_>) -> Result<bool, sqlx::Error> {
    let first = input
        .paths
        .first()
        .ok_or_else(|| sqlx::Error::Protocol("library requires a root".into()))?;
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    if input.existing {
        let updated = sqlx::query("UPDATE libraries SET name=?,path=?,kind=? WHERE id=?")
            .bind(input.name)
            .bind(first.to_string_lossy().as_ref())
            .bind(input.kind)
            .bind(input.id)
            .execute(&mut *tx)
            .await?;
        if updated.rows_affected() != 1 {
            return Ok(false);
        }
    } else {
        sqlx::query("INSERT INTO libraries VALUES(?,?,?,?)")
            .bind(input.id)
            .bind(input.name)
            .bind(first.to_string_lossy().as_ref())
            .bind(input.kind)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("DELETE FROM library_roots WHERE library_id=?")
        .bind(input.id)
        .execute(&mut *tx)
        .await?;
    for path in input.paths {
        sqlx::query("INSERT INTO library_roots VALUES(?,?)")
            .bind(input.id)
            .bind(path.to_string_lossy().as_ref())
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("INSERT INTO library_options VALUES(?,?) ON CONFLICT(library_id) DO UPDATE SET enabled=excluded.enabled")
        .bind(input.id).bind(input.enabled).execute(&mut *tx).await?;
    for item in sqlx::query("SELECT id,path FROM items WHERE library_id=?")
        .bind(input.id)
        .fetch_all(&mut *tx)
        .await?
    {
        if !input
            .paths
            .iter()
            .any(|p| PathBuf::from(item.get::<String, _>("path")).starts_with(p))
        {
            let id: String = item.get("id");
            sqlx::query("DELETE FROM progress WHERE item_id=?")
                .bind(&id)
                .execute(&mut *tx)
                .await?;
            sqlx::query("DELETE FROM items WHERE id=?")
                .bind(id)
                .execute(&mut *tx)
                .await?;
        }
    }
    tx.commit().await?;
    Ok(true)
}
pub async fn remove(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    for query in [
        "DELETE FROM progress WHERE item_id IN(SELECT id FROM items WHERE library_id=?)",
        "DELETE FROM items WHERE library_id=?",
        "DELETE FROM library_grants WHERE library_id=?",
        "DELETE FROM library_roots WHERE library_id=?",
        "DELETE FROM library_options WHERE library_id=?",
        "DELETE FROM libraries WHERE id=?",
    ] {
        sqlx::query(query).bind(id).execute(&mut *tx).await?;
    }
    tx.commit().await
}
