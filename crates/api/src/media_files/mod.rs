use crate::*;
use serde::Deserialize;
mod plan;
mod retention;
#[cfg(test)]
mod tests;
mod unlink;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {
    pub item_ids: Vec<String>,
    #[serde(default = "keep_default")]
    pub keep_metadata: bool,
    #[serde(default)]
    pub token: Option<String>,
    #[serde(default)]
    pub confirm: bool,
}
fn keep_default() -> bool {
    true
}
pub(crate) async fn preview(
    State(a): State<App>,
    h: HeaderMap,
    Json(input): Json<Input>,
) -> Result<Json<Value>> {
    users::admin(&a, &h).await?;
    let entries = plan::build(&a, &input.item_ids).await?;
    Ok(Json(plan::summary(&entries, input.keep_metadata)?))
}
pub(crate) async fn apply(
    State(a): State<App>,
    h: HeaderMap,
    Json(input): Json<Input>,
) -> Result<Json<Value>> {
    users::admin(&a, &h).await?;
    if !input.confirm {
        return Err(bad("confirm permanent removal explicitly"));
    }
    let _scan = a
        .scan
        .clone()
        .try_acquire_owned()
        .map_err(|_| bad("wait for the current library operation"))?;
    let _imports = a
        .acquisition
        .import_jobs
        .clone()
        .try_acquire_owned()
        .map_err(|_| bad("wait for the current import"))?;
    let sessions = a.playback.sessions.lock().await;
    if sessions
        .values()
        .any(|entry| input.item_ids.contains(&entry.item))
    {
        return Err(bad(
            "stop playback of the selected media before removing it",
        ));
    }
    let entries = plan::build(&a, &input.item_ids).await?;
    if input.token.as_deref() != Some(plan::token(&entries, input.keep_metadata)?.as_str()) {
        return Err(bad("media changed; preview removal again"));
    }
    let mut removed = Vec::new();
    let mut failures = Vec::new();
    let mut estimated = 0u64;
    let mut shared = 0usize;
    let mut deleted = 0usize;
    for entry in entries {
        let mut retained = entry.data.clone();
        retained["file_available"] = json!(false);
        retained["file_removed_at"] = json!(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        );
        sqlx::query("UPDATE items SET data=? WHERE id=?")
            .bind(retained.to_string())
            .bind(&entry.id)
            .execute(&a.pool)
            .await
            .map_err(db)?;
        if retention::finish(&a, &entry.id, true).await.is_err() {
            sqlx::query("UPDATE items SET data=? WHERE id=?")
                .bind(entry.data.to_string())
                .bind(&entry.id)
                .execute(&a.pool)
                .await
                .map_err(db)?;
            failures.push(json!({"id":entry.id,"message":"Removal intent could not be saved; no file was removed."}));
            continue;
        }
        if let Some(stamp) = entry.stamp.clone() {
            let root = entry.root.clone();
            let path = entry.path.clone();
            let expected = stamp.clone();
            let result =
                tokio::task::spawn_blocking(move || unlink::remove(&root, &path, &expected))
                    .await
                    .map_err(|_| bad("media removal interrupted"))?;
            if result.is_err() {
                sqlx::query("UPDATE items SET data=? WHERE id=?")
                    .bind(entry.data.to_string())
                    .bind(&entry.id)
                    .execute(&a.pool)
                    .await
                    .map_err(db)?;
                failures.push(json!({"id":entry.id,"message":"File changed or could not be deleted; preview it again."}));
                continue;
            }
            if stamp.links > 1 {
                shared += 1;
            } else {
                estimated = estimated.saturating_add(stamp.allocated);
            }
            deleted += 1;
        }
        if !input.keep_metadata && retention::finish(&a, &entry.id, false).await.is_err() {
            failures.push(json!({"id":entry.id,"message":"The local file was removed, but its metadata could not be forgotten. Refresh before retrying."}));
        }
        removed.push(entry.id);
    }
    drop(sessions);
    Ok(Json(
        json!({"removed_ids":removed,"deleted_files":deleted,"kept_metadata":input.keep_metadata,"failures":failures,"bytes_freed_estimate":estimated,"shared_files_count":shared}),
    ))
}

pub(crate) async fn reaction(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> Result<Json<Value>> {
    let u = users::authorize_item(&a, &h, &id, false).await?;
    let value = match &body["value"] {
        Value::Null => None,
        Value::String(s) if matches!(s.as_str(), "like" | "dislike") => Some(s.as_str()),
        _ => return Err(bad("choose like, dislike or null")),
    };
    if let Some(value) = value {
        sqlx::query("INSERT INTO item_reactions VALUES(?,?,?) ON CONFLICT(user_id,item_id) DO UPDATE SET value=excluded.value").bind(u["id"].as_str()).bind(&id).bind(value).execute(&a.pool).await.map_err(db)?;
    } else {
        sqlx::query("DELETE FROM item_reactions WHERE user_id=? AND item_id=?")
            .bind(u["id"].as_str())
            .bind(&id)
            .execute(&a.pool)
            .await
            .map_err(db)?;
    }
    Ok(Json(json!({"value":value})))
}
pub(crate) use retention::restore_identity;
