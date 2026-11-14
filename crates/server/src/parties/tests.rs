use super::*;
use tower::ServiceExt;
async fn request(
    a: &App,
    h: &HeaderMap,
    method: &str,
    path: &str,
    body: Value,
) -> (StatusCode, Value) {
    let router = Router::new()
        .nest("/api/v1", routes())
        .layer(middleware::from_fn_with_state(a.clone(), origin_check))
        .with_state(a.clone());
    let mut req = Request::builder()
        .method(method)
        .uri(format!("/api/v1{path}"))
        .header("origin", &a.origin)
        .header("content-type", "application/json");
    if let Some(cookie) = h.get("cookie") {
        req = req.header("cookie", cookie)
    }
    let response = router
        .oneshot(req.body(axum::body::Body::from(body.to_string())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), 1_000_000)
        .await
        .unwrap();
    let value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    };
    (status, value)
}
async fn fixture() -> (App, HeaderMap, HeaderMap) {
    let a = crate::tests::app().await;
    let host = crate::tests::account(&a, "host", true).await;
    let guest = crate::tests::account(&a, "guest", false).await;
    sqlx::query("UPDATE user_access SET syncplay_access='join' WHERE user_id='guest'")
        .execute(&a.pool)
        .await
        .unwrap();
    for (library, item) in [("shared", "one"), ("private", "two")] {
        sqlx::query("INSERT INTO libraries VALUES(?,?,?,'movies')")
            .bind(library)
            .bind(library)
            .bind(format!("/fixture/{library}"))
            .execute(&a.pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO items VALUES(?,?,?,?)")
            .bind(item)
            .bind(library)
            .bind(format!("/fixture/{library}/{item}.mkv"))
            .bind(json!({"id":item,"library_id":library,"duration_seconds":1000}).to_string())
            .execute(&a.pool)
            .await
            .unwrap();
    }
    sqlx::query("INSERT INTO library_grants VALUES('guest','shared')")
        .execute(&a.pool)
        .await
        .unwrap();
    (a, host, guest)
}
async fn create_room(a: &App, h: &HeaderMap, item: &str) -> Value {
    let (status, v) = request(
        a,
        h,
        "POST",
        "/parties",
        json!({"name":"Movie night","item_id":item}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{v}");
    v["party"].clone()
}

mod lifecycle;
mod limits;
mod permissions;
