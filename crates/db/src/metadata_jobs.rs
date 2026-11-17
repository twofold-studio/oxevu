//! Persistent retry bookkeeping, independent of scanner rewrites of item data.
use sqlx::{Row, SqlitePool};

pub struct Pending {
    pub id: String,
    pub path: String,
    pub data: String,
}

pub async fn next(pool: &SqlitePool) -> Result<Option<Pending>, sqlx::Error> {
    Ok(sqlx::query("SELECT i.id,i.path,i.data FROM items i JOIN item_metadata_state s ON s.id=i.id JOIN libraries l ON l.id=i.library_id LEFT JOIN library_options o ON o.library_id=l.id LEFT JOIN metadata_attempts a ON a.item_id=i.id WHERE COALESCE(o.enabled,1)=1 AND s.identified=0 AND COALESCE(json_extract(i.data,'$.file_available'),1)!=0 AND (a.item_id IS NULL OR a.attempted_at < unixepoch()-21600) ORDER BY a.attempted_at,i.id LIMIT 1")
        .fetch_optional(pool).await?.map(|r| Pending { id:r.get("id"), path:r.get("path"), data:r.get("data") }))
}

pub async fn failed(pool: &SqlitePool, id: &str, error: &str) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO metadata_attempts SELECT id,unixepoch(),? FROM items WHERE id=? ON CONFLICT(item_id) DO UPDATE SET attempted_at=excluded.attempted_at,error=excluded.error")
        .bind(error).bind(id).execute(pool).await?;
    Ok(())
}

pub async fn publish_unmatched(pool: &SqlitePool, id: &str, data: &str) -> Result<(), sqlx::Error> {
    // A manual match made while a background lookup was in flight always wins.
    sqlx::query("INSERT INTO item_metadata SELECT id,? FROM items WHERE id=? ON CONFLICT(item_id) DO UPDATE SET data=json_patch(item_metadata.data,excluded.data) WHERE item_metadata.item_id IN (SELECT id FROM item_metadata_state WHERE identified=0)")
        .bind(data)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn retry(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE metadata_attempts SET attempted_at=0")
        .execute(pool)
        .await?;
    Ok(())
}
