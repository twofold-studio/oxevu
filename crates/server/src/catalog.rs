use super::*;
pub(crate) async fn catalog(a: &App, h: &HeaderMap) -> Result<Vec<Value>> {
    let u = user(a, h).await?;
    let favorites: Vec<String> =
        sqlx::query_scalar("SELECT item_id FROM favorites WHERE user_id=?")
            .bind(u["id"].as_str())
            .fetch_all(&a.pool)
            .await
            .map_err(db)?;
    let rows=sqlx::query("SELECT items.id,items.library_id,data,CAST(COALESCE(position,0.0) AS REAL) position,COALESCE(completed,0) completed FROM items LEFT JOIN progress ON item_id=items.id AND user_id=?").bind(u["id"].as_str()).fetch_all(&a.pool).await.map_err(db)?;
    let mut visible = Vec::new();
    for r in rows {
        if users::authorize_item(a, h, &r.get::<String, _>("id"), false)
            .await
            .is_ok()
        {
            visible.push(r)
        }
    }
    let metadata = sqlx::query("SELECT item_id,data FROM item_metadata")
        .fetch_all(&a.pool)
        .await
        .map_err(db)?;
    let metadata: std::collections::HashMap<String, Value> = metadata
        .into_iter()
        .map(|r| {
            let id = r.get::<String, _>("item_id");
            let data = serde_json::from_str::<Value>(r.get("data"))
                .map_err(|_| bad("invalid matched metadata"))?;
            Ok((id, data))
        })
        .collect::<Result<_>>()?;
    visible
        .iter()
        .map(|r| {
            let mut v: Value = serde_json::from_str(r.get("data")).map_err(bad)?;
            if let Some(enriched) = metadata
                .get(&r.get::<String, _>("id"))
                .and_then(Value::as_object)
                && let Some(object) = v.as_object_mut()
            {
                let local_poster = object
                    .get("poster_url")
                    .filter(|value| {
                        value
                            .as_str()
                            .is_some_and(|url| url.contains("/artwork/poster"))
                    })
                    .cloned();
                object.extend(enriched.clone());
                if let Some(poster) = local_poster {
                    object.insert("poster_url".into(), poster);
                }
            }
            for kind in ["poster", "thumbnail", "backdrop"] {
                v.as_object_mut()
                    .ok_or_else(|| bad("invalid catalog item"))?
                    .entry(format!("{kind}_url"))
                    .or_insert(Value::Null);
            }
            v["position_seconds"] = json!(r.try_get::<f64, _>("position").map_err(db)?);
            v["completed"] = json!(r.try_get::<bool, _>("completed").map_err(db)?);
            v["favorite"] = json!(favorites.contains(&r.get::<String, _>("id")));
            Ok(v)
        })
        .collect()
}
pub(crate) async fn items(State(a): State<App>, h: HeaderMap) -> Result<Json<Value>> {
    Ok(Json(json!({"items":catalog(&a,&h).await?})))
}
pub(crate) async fn favorite(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> Result<StatusCode> {
    let u = users::authorize_item(&a, &h, &id, false).await?;
    let enabled = body["favorite"]
        .as_bool()
        .ok_or_else(|| bad("favorite must be boolean"))?;
    if enabled {
        sqlx::query("INSERT OR IGNORE INTO favorites(user_id,item_id) VALUES(?,?)")
            .bind(u["id"].as_str())
            .bind(&id)
            .execute(&a.pool)
            .await
            .map_err(db)?;
    } else {
        sqlx::query("DELETE FROM favorites WHERE user_id=? AND item_id=?")
            .bind(u["id"].as_str())
            .bind(&id)
            .execute(&a.pool)
            .await
            .map_err(db)?;
    }
    Ok(StatusCode::NO_CONTENT)
}
pub(crate) async fn item(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    Ok(Json(
        json!({"item":catalog(&a,&h).await?.into_iter().find(|v|v["id"]==id).ok_or_else(||(StatusCode::NOT_FOUND,Json(json!({"error":"item not found"}))))?}),
    ))
}
pub(crate) async fn progress(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    Json(v): Json<Value>,
) -> Result<StatusCode> {
    let u = users::authorize_item(&a, &h, &id, true).await?;
    let mut p = v["position_seconds"]
        .as_f64()
        .filter(|p| p.is_finite() && *p >= 0.)
        .ok_or_else(|| bad("invalid position"))?;
    let mut c = v["completed"]
        .as_bool()
        .ok_or_else(|| bad("completed required"))?;
    if sqlx::query_scalar::<_, i64>("SELECT count(*) FROM items WHERE id=?")
        .bind(&id)
        .fetch_one(&a.pool)
        .await
        .map_err(db)?
        == 0
    {
        return Err(bad("unknown item"));
    }
    let settings = crate::settings::load(&a).await?;
    let data: String = sqlx::query_scalar("SELECT data FROM items WHERE id=?")
        .bind(&id)
        .fetch_one(&a.pool)
        .await
        .map_err(db)?;
    let data: Value = serde_json::from_str(&data).map_err(bad)?;
    if let Some(duration) = data["duration_seconds"].as_f64().filter(|d| *d > 0.) {
        let percent = p / duration * 100.;
        c = c || percent >= settings.resume_max_percent;
        if duration < f64::from(settings.resume_min_duration_seconds)
            || percent < settings.resume_min_percent
            || c
        {
            p = 0.;
        }
    }
    sqlx::query("INSERT INTO progress VALUES(?,?,?,?) ON CONFLICT(user_id,item_id) DO UPDATE SET position=excluded.position,completed=excluded.completed").bind(u["id"].as_str()).bind(id).bind(p).bind(c).execute(&a.pool).await.map_err(db)?;
    Ok(StatusCode::NO_CONTENT)
}
pub(crate) async fn stream(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    r: Request,
) -> Result<Response> {
    users::authorize_item(&a, &h, &id, true).await?;
    let row=sqlx::query("SELECT items.path,items.data,libraries.path root FROM items JOIN libraries ON library_id=libraries.id WHERE items.id=?").bind(&id).fetch_optional(&a.pool).await.map_err(db)?.ok_or_else(||bad("unknown item"))?;
    let path = tokio::fs::canonicalize(row.get::<String, _>("path"))
        .await
        .map_err(bad)?;
    if !libraries::contains(&a, &id, &path).await? {
        return Err(bad("file outside library"));
    }
    use tower_http::services::ServeFile;
    let data: Value = serde_json::from_str(row.get("data")).map_err(bad)?;
    let container = data["container"].as_str().unwrap_or("");
    let mime = if path.extension().is_some_and(|ext| ext == "webm") {
        "video/webm"
    } else if container.contains("matroska") {
        "video/x-matroska"
    } else if container.contains("mp4") || container.contains("mov") {
        "video/mp4"
    } else if container.contains("mpegts") {
        "video/mp2t"
    } else {
        "application/octet-stream"
    };
    Ok(ServeFile::new_with_mime(path, &mime.parse().map_err(bad)?)
        .try_call(r)
        .await
        .map_err(bad)?
        .into_response())
}
#[cfg(test)]
mod favorite_tests {
    use super::*;
    #[tokio::test]
    async fn favorites_are_authorized_and_independent() {
        let a = crate::tests::app().await;
        let first = crate::tests::account(&a, "first", true).await;
        let second = crate::tests::account(&a, "second", true).await;
        let viewer = crate::tests::account(&a, "viewer", false).await;
        sqlx::query(
            "INSERT INTO libraries(id,name,path,kind) VALUES('l','Movies','/fixture','movies')",
        )
        .execute(&a.pool)
        .await
        .unwrap();
        sqlx::query("INSERT INTO items VALUES('i','l','/fixture/movie.mp4','{\"id\":\"i\",\"library_id\":\"l\"}')").execute(&a.pool).await.unwrap();
        assert!(
            favorite(
                State(a.clone()),
                viewer,
                Path("i".into()),
                Json(json!({"favorite":true}))
            )
            .await
            .is_err()
        );
        favorite(
            State(a.clone()),
            first.clone(),
            Path("i".into()),
            Json(json!({"favorite":true})),
        )
        .await
        .unwrap();
        assert_eq!(catalog(&a, &first).await.unwrap()[0]["favorite"], true);
        assert_eq!(catalog(&a, &second).await.unwrap()[0]["favorite"], false);
    }
}
