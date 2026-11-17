use super::*;
pub(super) async fn finish(a: &App, id: &str, keep: bool) -> Result<()> {
    let raw:String=sqlx::query_scalar("SELECT json_patch(i.data,COALESCE(m.data,'{}')) FROM items i LEFT JOIN item_metadata m ON m.item_id=i.id WHERE i.id=?").bind(id).fetch_one(&a.pool).await.map_err(db)?;
    let item: Value = serde_json::from_str(&raw).map_err(bad)?;
    let provider = item["provider_ids"]["tmdb"]
        .as_u64()
        .or_else(|| item["provider_ids"]["tmdb"].as_str()?.parse().ok());
    let title = provider.map(|provider| {
        format!(
            "tmdb:{}:{provider}",
            if item["kind"] == "episode" {
                "tv"
            } else {
                "movie"
            }
        )
    });
    let mut tx = a.pool.begin().await.map_err(db)?;
    if let Some(title) = title {
        let mut units = Vec::new();
        if item["kind"] == "movie" {
            units.push((-1i64, -1i64));
        } else {
            if let (Some(season), Some(episode)) = (
                item["season_number"].as_i64(),
                item["episode_number"].as_i64(),
            ) {
                units.push((season, episode));
            }
            for unit in item["episode_coverage"].as_array().into_iter().flatten() {
                if let (Some(s), Some(e)) = (unit["season"].as_i64(), unit["episode"].as_i64()) {
                    units.push((s, e));
                }
            }
        }
        for (season, episode) in units {
            sqlx::query("INSERT OR IGNORE INTO acquisition_removed_units SELECT id,?,? FROM acquisition_titles WHERE id=?").bind(season).bind(episode).bind(&title).execute(&mut *tx).await.map_err(db)?;
        }
    }
    if !keep {
        sqlx::query("DELETE FROM progress WHERE item_id=?")
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(db)?;
        sqlx::query("DELETE FROM items WHERE id=?")
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(db)?;
    }
    tx.commit().await.map_err(db)?;
    Ok(())
}

/// Reuse a single retained identity after a new release arrives at a different path.
pub(crate) async fn restore_identity(a: &App, new_id: &str, title: &Value) -> Result<String> {
    let row = sqlx::query("SELECT library_id,path,data FROM items WHERE id=?")
        .bind(new_id)
        .fetch_one(&a.pool)
        .await
        .map_err(db)?;
    let mut data: Value = serde_json::from_str(row.get("data")).map_err(bad)?;
    let candidates=sqlx::query("SELECT i.id,i.data FROM items i JOIN item_metadata m ON m.item_id=i.id WHERE i.library_id=? AND json_extract(i.data,'$.file_available')=0 AND CAST(json_extract(m.data,'$.provider_ids.tmdb') AS INTEGER)=?")
        .bind(row.get::<String,_>("library_id")).bind(title["provider_id"].as_i64()).fetch_all(&a.pool).await.map_err(db)?;
    let mut matches = Vec::new();
    for candidate in candidates {
        let old: Value = serde_json::from_str(candidate.get("data")).map_err(bad)?;
        if old["kind"] == data["kind"]
            && (old["kind"] == "movie"
                || (old["season_number"] == data["season_number"]
                    && old["episode_number"] == data["episode_number"]))
        {
            matches.push(candidate.get::<String, _>("id"));
        }
    }
    if matches.len() != 1 {
        return Ok(new_id.to_owned());
    }
    let old_id = &matches[0];
    let mut tx = a.pool.begin().await.map_err(db)?;
    let history:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM progress WHERE item_id=? UNION ALL SELECT 1 FROM favorites WHERE item_id=? UNION ALL SELECT 1 FROM item_reactions WHERE item_id=?)").bind(new_id).bind(new_id).bind(new_id).fetch_one(&mut *tx).await.map_err(db)?;
    if history {
        return Ok(new_id.to_owned());
    }
    data["id"] = json!(old_id);
    for field in ["poster_url", "thumbnail_url", "backdrop_url"] {
        if let Some(url) = data[field]
            .as_str()
            .and_then(|url| url.strip_prefix(&format!("/api/v1/items/{new_id}/")))
        {
            data[field] = json!(format!("/api/v1/items/{old_id}/{url}"));
        }
    }
    data["file_available"] = json!(true);
    data["file_removed_at"] = Value::Null;
    sqlx::query("DELETE FROM items WHERE id=?")
        .bind(new_id)
        .execute(&mut *tx)
        .await
        .map_err(db)?;
    sqlx::query("UPDATE items SET path=?,data=? WHERE id=?")
        .bind(row.get::<String, _>("path"))
        .bind(data.to_string())
        .bind(old_id)
        .execute(&mut *tx)
        .await
        .map_err(db)?;
    tx.commit().await.map_err(db)?;
    Ok(old_id.clone())
}
