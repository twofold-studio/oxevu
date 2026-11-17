//! Catalog persistence. Authorization and metadata projection belong to services.
use sqlx::{Row, SqlitePool};
use std::collections::{HashMap, HashSet};

pub struct Item {
    pub id: String,
    pub data: String,
    pub position: f64,
    pub completed: bool,
}
pub struct Snapshot {
    pub items: Vec<Item>,
    pub favorites: HashSet<String>,
    pub reactions: HashMap<String, String>,
    pub metadata: HashMap<String, String>,
}
pub async fn snapshot(pool: &SqlitePool, user: &str) -> Result<Snapshot, sqlx::Error> {
    let favorites = sqlx::query_scalar("SELECT item_id FROM favorites WHERE user_id=?")
        .bind(user)
        .fetch_all(pool)
        .await?
        .into_iter()
        .collect();
    let reactions = sqlx::query("SELECT item_id,value FROM item_reactions WHERE user_id=?")
        .bind(user)
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|r| (r.get("item_id"), r.get("value")))
        .collect();
    let items = sqlx::query("SELECT items.id,data,CAST(COALESCE(position,0.0) AS REAL) position,COALESCE(completed,0) completed FROM items LEFT JOIN progress ON item_id=items.id AND user_id=?")
        .bind(user).fetch_all(pool).await?.into_iter().map(|r| Item { id:r.get("id"),data:r.get("data"),position:r.get("position"),completed:r.get("completed") }).collect();
    let metadata = sqlx::query("SELECT item_id,data FROM item_metadata")
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|r| (r.get("item_id"), r.get("data")))
        .collect();
    Ok(Snapshot {
        items,
        favorites,
        reactions,
        metadata,
    })
}
pub async fn favorite(
    pool: &SqlitePool,
    user: &str,
    item: &str,
    enabled: bool,
) -> Result<(), sqlx::Error> {
    let query = if enabled {
        "INSERT OR IGNORE INTO favorites(user_id,item_id) VALUES(?,?)"
    } else {
        "DELETE FROM favorites WHERE user_id=? AND item_id=?"
    };
    sqlx::query(query)
        .bind(user)
        .bind(item)
        .execute(pool)
        .await?;
    Ok(())
}
pub async fn item_data(pool: &SqlitePool, item: &str) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar("SELECT data FROM items WHERE id=?")
        .bind(item)
        .fetch_optional(pool)
        .await
}
pub async fn progress(
    pool: &SqlitePool,
    user: &str,
    item: &str,
    position: f64,
    completed: bool,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO progress SELECT ?,id,?,? FROM items WHERE id=? ON CONFLICT(user_id,item_id) DO UPDATE SET position=excluded.position,completed=excluded.completed")
        .bind(user).bind(position).bind(completed).bind(item).execute(pool).await?;
    Ok(())
}
pub struct Source {
    pub path: String,
    pub data: String,
}
pub async fn source(pool: &SqlitePool, item: &str) -> Result<Option<Source>, sqlx::Error> {
    Ok(sqlx::query("SELECT items.path,items.data FROM items JOIN libraries ON library_id=libraries.id WHERE items.id=?")
        .bind(item).fetch_optional(pool).await?.map(|r| Source { path:r.get("path"),data:r.get("data") }))
}
