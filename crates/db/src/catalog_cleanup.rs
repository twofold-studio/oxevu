//! Explicit catalog-only merges. Physical media is never removed here.
use sqlx::{Row, SqlitePool};

pub async fn merge(
    pool: &SqlitePool,
    keep: &str,
    duplicate: &str,
    expected_path: &str,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    let rows = sqlx::query("SELECT id,library_id,path FROM items WHERE id IN (?,?)")
        .bind(keep)
        .bind(duplicate)
        .fetch_all(&mut *tx)
        .await?;
    if rows.len() != 2
        || rows[0].get::<String, _>("library_id") != rows[1].get::<String, _>("library_id")
        || !rows.iter().any(|r| {
            r.get::<String, _>("id") == duplicate && r.get::<String, _>("path") == expected_path
        })
    {
        return Err(sqlx::Error::Protocol(
            "duplicate catalog changed; inspect again".into(),
        ));
    }
    let active: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM acquisition_stream_items WHERE item_id IN (?,?))",
    )
    .bind(keep)
    .bind(duplicate)
    .fetch_one(&mut *tx)
    .await?;
    if active {
        return Err(sqlx::Error::Protocol(
            "finish active transfers before merging".into(),
        ));
    }
    sqlx::query("INSERT INTO progress SELECT user_id,?,position,completed FROM progress WHERE item_id=? ON CONFLICT(user_id,item_id) DO UPDATE SET position=max(progress.position,excluded.position),completed=max(progress.completed,excluded.completed)")
        .bind(keep).bind(duplicate).execute(&mut *tx).await?;
    sqlx::query("INSERT OR IGNORE INTO favorites SELECT user_id,? FROM favorites WHERE item_id=?")
        .bind(keep)
        .bind(duplicate)
        .execute(&mut *tx)
        .await?;
    sqlx::query("INSERT OR IGNORE INTO item_reactions SELECT user_id,?,value FROM item_reactions WHERE item_id=?").bind(keep).bind(duplicate).execute(&mut *tx).await?;
    sqlx::query("UPDATE item_subtitles SET item_id=? WHERE item_id=?")
        .bind(keep)
        .bind(duplicate)
        .execute(&mut *tx)
        .await?;
    let old_prefix = format!("/items/{duplicate}/");
    let new_prefix = format!("/items/{keep}/");
    sqlx::query("INSERT INTO item_metadata SELECT ?,replace(data,?,?) FROM item_metadata WHERE item_id=? ON CONFLICT(item_id) DO UPDATE SET data=CASE WHEN json_extract(item_metadata.data,'$.metadata_source')='manual' THEN item_metadata.data ELSE json_patch(item_metadata.data,excluded.data) END")
        .bind(keep).bind(old_prefix).bind(new_prefix).bind(duplicate).execute(&mut *tx).await?;
    sqlx::query("INSERT OR REPLACE INTO scan_exclusions VALUES(?,'verified identical copy')")
        .bind(expected_path)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM progress WHERE item_id=?")
        .bind(duplicate)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM items WHERE id=?")
        .bind(duplicate)
        .execute(&mut *tx)
        .await?;
    tx.commit().await
}

pub async fn exclude_sample(
    pool: &SqlitePool,
    id: &str,
    expected_path: &str,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM items WHERE id=? AND path=?)")
            .bind(id)
            .bind(expected_path)
            .fetch_one(&mut *tx)
            .await?;
    if !exists {
        return Err(sqlx::Error::RowNotFound);
    }
    let active: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM acquisition_stream_items WHERE item_id=?)")
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
    if active {
        return Err(sqlx::Error::Protocol(
            "finish active transfers before excluding a sample".into(),
        ));
    }
    sqlx::query("INSERT OR REPLACE INTO scan_exclusions VALUES(?,'release sample')")
        .bind(expected_path)
        .execute(&mut *tx)
        .await?;
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
