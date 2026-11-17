use super::*;
pub(crate) async fn catalog(a: &App, h: &HeaderMap) -> Result<Vec<Value>> {
    let actor = user(a, h).await?;
    let snapshot =
        oxevu_db::catalog::snapshot(&a.pool, actor["id"].as_str().ok_or_else(users::forbidden)?)
            .await
            .map_err(db)?;
    let mut visible = std::collections::HashSet::new();
    for item in &snapshot.items {
        if users::authorize_item(a, h, &item.id, false).await.is_ok() {
            visible.insert(item.id.clone());
        }
    }
    let mut items =
        crate::library::catalog::project(snapshot, &visible).map_err(crate::library::response)?;
    acquisition::annotate_downloads(a, &mut items).await?;
    Ok(items)
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
    crate::library::catalog::favorite(
        &a.pool,
        u["id"].as_str().ok_or_else(users::forbidden)?,
        &id,
        &body,
    )
    .await
    .map_err(crate::library::response)?;
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
    let settings = crate::settings::load(&a).await?;
    crate::library::catalog::progress(
        &a.pool,
        u["id"].as_str().ok_or_else(users::forbidden)?,
        &id,
        &v,
        &settings,
    )
    .await
    .map_err(crate::library::response)?;
    Ok(StatusCode::NO_CONTENT)
}
pub(crate) async fn stream(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    r: Request,
) -> Result<Response> {
    users::authorize_item(&a, &h, &id, true).await?;
    if let Some(source) = acquisition::streaming::for_item(&a, &id).await? {
        return source.response(a, r).await;
    }
    let row = oxevu_db::catalog::source(&a.pool, &id)
        .await
        .map_err(db)?
        .ok_or_else(|| bad("unknown item"))?;
    let path = tokio::fs::canonicalize(&row.path).await.map_err(bad)?;
    if !libraries::contains(&a, &id, &path).await? {
        return Err(bad("file outside library"));
    }
    use tower_http::services::ServeFile;
    let data: Value = serde_json::from_str(&row.data).map_err(bad)?;
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
