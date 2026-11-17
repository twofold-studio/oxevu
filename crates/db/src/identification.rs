//! Candidate selection and atomic publication of matched metadata.
use sqlx::{Row, SqlitePool};

pub struct Candidate {
    pub id: String,
    pub data: String,
}

pub async fn candidates(
    pool: &SqlitePool,
    id: &str,
    series: Option<(Option<&str>, Option<&str>)>,
) -> Result<Vec<Candidate>, sqlx::Error> {
    let rows = match series {
        Some((library, title)) => sqlx::query("SELECT id,data FROM items WHERE library_id=? AND json_extract(data,'$.show_title')=? ORDER BY id LIMIT 101")
            .bind(library).bind(title).fetch_all(pool).await?,
        None => sqlx::query("SELECT id,data FROM items WHERE id=?").bind(id).fetch_all(pool).await?,
    };
    Ok(rows
        .into_iter()
        .map(|row| Candidate {
            id: row.get("id"),
            data: row.get("data"),
        })
        .collect())
}

pub async fn publish(pool: &SqlitePool, updates: &[(String, String)]) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    for (id, data) in updates {
        sqlx::query("INSERT INTO item_metadata VALUES(?,?) ON CONFLICT(item_id) DO UPDATE SET data=excluded.data")
            .bind(id).bind(data).execute(&mut *tx).await?;
    }
    tx.commit().await
}
