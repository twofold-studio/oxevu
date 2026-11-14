use super::*;
pub(crate) async fn catalog(a: &App, h: &HeaderMap) -> Result<Vec<Value>> {
    let u = user(a, h).await?;
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
    visible
        .iter()
        .map(|r| {
            let mut v: Value = serde_json::from_str(r.get("data")).map_err(bad)?;
            for kind in ["poster", "thumbnail", "backdrop"] {
                v.as_object_mut()
                    .ok_or_else(|| bad("invalid catalog item"))?
                    .entry(format!("{kind}_url"))
                    .or_insert(Value::Null);
            }
            v["position_seconds"] = json!(r.try_get::<f64, _>("position").map_err(db)?);
            v["completed"] = json!(r.try_get::<bool, _>("completed").map_err(db)?);
            Ok(v)
        })
        .collect()
}
pub(crate) async fn items(State(a): State<App>, h: HeaderMap) -> Result<Json<Value>> {
    Ok(Json(json!({"items":catalog(&a,&h).await?})))
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
