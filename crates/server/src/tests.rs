use super::*;
async fn app() -> App {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!().run(&pool).await.unwrap();
    App {
        pool,
        token: "test-token".into(),
        origin: "http://localhost:3000".into(),
        scan: Arc::new(Semaphore::new(1)),
        artwork_jobs: Arc::new(Semaphore::new(1)),
        artwork_cache: std::env::temp_dir().join(uuid::Uuid::new_v4().to_string()),
    }
}
#[tokio::test]
async fn bootstrap_requires_local_token_and_cannot_replace_admin() {
    let a = app().await;
    let body = json!({"username":"admin","password":"a secure password"});
    assert!(
        setup(State(a.clone()), HeaderMap::new(), Json(body.clone()))
            .await
            .is_err()
    );
    let mut h = HeaderMap::new();
    h.insert("x-oxevu-setup-token", "test-token".parse().unwrap());
    let response = setup(State(a.clone()), h.clone(), Json(body.clone()))
        .await
        .unwrap();
    assert!(
        response.headers()["set-cookie"]
            .to_str()
            .unwrap()
            .contains("HttpOnly")
    );
    assert!(setup(State(a.clone()), h, Json(body)).await.is_err());
    let stored: String = sqlx::query_scalar("SELECT password FROM users")
        .fetch_one(&a.pool)
        .await
        .unwrap();
    assert!(stored.starts_with("$argon2id$"));
}
#[tokio::test]
async fn catalog_requires_authentication() {
    assert_eq!(
        catalog(&app().await, &HeaderMap::new())
            .await
            .unwrap_err()
            .0,
        StatusCode::UNAUTHORIZED
    );
}
#[tokio::test]
async fn password_hashes_are_salted() {
    assert_ne!(
        hash("password".into()).await.unwrap(),
        hash("password".into()).await.unwrap()
    );
}
#[tokio::test]
async fn original_ranges_and_progress_are_authenticated_and_persistent() {
    let a = app().await;
    let dir = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
    tokio::fs::create_dir(&dir).await.unwrap();
    let file = dir.join("fixture.mp4");
    tokio::fs::write(&file, b"0123456789").await.unwrap();
    sqlx::query("INSERT INTO users VALUES('u','admin','unused')")
        .execute(&a.pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO sessions VALUES('token','u',unixepoch()+60)")
        .execute(&a.pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO libraries VALUES('l','test',?,'movies')")
        .bind(dir.to_str())
        .execute(&a.pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO items VALUES('i','l',?,?)")
        .bind(file.to_str())
        .bind(r#"{"id":"i","container":"mp4"}"#)
        .execute(&a.pool)
        .await
        .unwrap();
    let mut h = HeaderMap::new();
    h.insert("cookie", "oxevu_session=token".parse().unwrap());
    assert_eq!(catalog(&a, &h).await.unwrap()[0]["position_seconds"], 0.0);
    for (range, status) in [
        ("bytes=2-4", StatusCode::PARTIAL_CONTENT),
        ("bytes=100-", StatusCode::RANGE_NOT_SATISFIABLE),
    ] {
        let request = Request::builder()
            .uri("/")
            .header("range", range)
            .body(axum::body::Body::empty())
            .unwrap();
        let response = stream(State(a.clone()), h.clone(), Path("i".into()), request)
            .await
            .unwrap();
        assert_eq!(response.status(), status);
    }
    let request = Request::builder()
        .method("HEAD")
        .uri("/")
        .body(axum::body::Body::empty())
        .unwrap();
    let response = stream(State(a.clone()), h.clone(), Path("i".into()), request)
        .await
        .unwrap();
    assert_eq!(response.headers()["content-length"], "10");
    progress(
        State(a.clone()),
        h.clone(),
        Path("i".into()),
        Json(json!({"position_seconds":3.5,"completed":false})),
    )
    .await
    .unwrap();
    assert_eq!(catalog(&a, &h).await.unwrap()[0]["position_seconds"], 3.5);
    tokio::fs::remove_dir_all(dir).await.unwrap();
}

#[tokio::test]
async fn artwork_requires_authentication() {
    let error = artwork::serve(
        State(app().await),
        HeaderMap::new(),
        Path(("unknown".to_owned(), "poster".to_owned())),
    )
    .await
    .unwrap_err();
    assert_eq!(error.0, StatusCode::UNAUTHORIZED);
}
