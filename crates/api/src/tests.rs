use super::*;
pub(crate) async fn app() -> App {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    oxevu_db::MIGRATOR.run(&pool).await.unwrap();
    let playback = playback::Runtime::new(
        &std::env::temp_dir().join(uuid::Uuid::new_v4().to_string()),
        &pool,
    )
    .await
    .unwrap();
    App {
        acquisition: acquisition::Runtime::new(
            std::env::temp_dir().join(uuid::Uuid::new_v4().to_string()),
        ),
        parties: Arc::new(parties::Store::default()),
        playback,
        pool,
        token: "test-token".into(),
        origin: "http://localhost:3000".into(),
        scan: Arc::new(Semaphore::new(1)),
        artwork_jobs: Arc::new(Semaphore::new(1)),
        artwork_cache: std::env::temp_dir()
            .join(uuid::Uuid::new_v4().to_string())
            .join("artwork"),
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

pub(crate) async fn account(a: &App, id: &str, admin: bool) -> HeaderMap {
    sqlx::query("INSERT INTO users VALUES(?,?, 'unused')")
        .bind(id)
        .bind(id)
        .execute(&a.pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO user_access(user_id,is_admin,all_libraries) VALUES(?,?,0)")
        .bind(id)
        .bind(admin)
        .execute(&a.pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO sessions VALUES(?,?,unixepoch()+600)")
        .bind(id)
        .bind(id)
        .execute(&a.pool)
        .await
        .unwrap();
    let mut h = HeaderMap::new();
    h.insert("cookie", format!("oxevu_session={id}").parse().unwrap());
    h
}
fn access_body(admin: bool) -> Value {
    json!({"username":"owner","is_admin":admin,"disabled":false,"can_play":true,"can_transcode":true,"syncplay_access":"create_join","all_libraries":true,"library_ids":[]})
}
#[tokio::test]
async fn migration_preserves_legacy_accounts_roots_and_progress() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::raw_sql(include_str!("../../db/migrations/0001_initial.sql"))
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO users VALUES('old','old','hash')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO libraries VALUES('l','movies','/media','movies')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO items VALUES('i','l','/media/a','{}')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO progress VALUES('old','i',42,0)")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::raw_sql(include_str!("../../db/migrations/0003_access.sql"))
        .execute(&pool)
        .await
        .unwrap();
    assert!(
        sqlx::query_scalar::<_, bool>("SELECT is_admin FROM user_access")
            .fetch_one(&pool)
            .await
            .unwrap()
    );
    assert_eq!(
        sqlx::query_scalar::<_, String>("SELECT path FROM library_roots")
            .fetch_one(&pool)
            .await
            .unwrap(),
        "/media"
    );
    assert_eq!(
        sqlx::query_scalar::<_, f64>("SELECT position FROM progress")
            .fetch_one(&pool)
            .await
            .unwrap(),
        42.
    );
}
#[tokio::test]
async fn permissions_guard_catalog_stream_artwork_and_progress() {
    let a = app().await;
    let h = account(&a, "guest", false).await;
    sqlx::query("INSERT INTO libraries VALUES('l','movies','/media','movies')")
        .execute(&a.pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO items VALUES('i','l','/media/a',?)")
        .bind(json!({"id":"i","library_id":"l"}).to_string())
        .execute(&a.pool)
        .await
        .unwrap();
    assert!(catalog(&a, &h).await.unwrap().is_empty());
    assert!(
        stream(
            State(a.clone()),
            h.clone(),
            Path("i".into()),
            Request::new(axum::body::Body::empty())
        )
        .await
        .is_err()
    );
    assert!(
        artwork::serve(
            State(a.clone()),
            h.clone(),
            Path(("i".into(), "poster".into()))
        )
        .await
        .is_err()
    );
    assert!(
        progress(
            State(a.clone()),
            h.clone(),
            Path("i".into()),
            Json(json!({"position_seconds":10,"completed":false}))
        )
        .await
        .is_err()
    );
    assert!(settings::get(State(a.clone()), h.clone()).await.is_err());
    assert!(libraries(State(a.clone()), h.clone()).await.is_err());
    sqlx::query("INSERT INTO library_grants VALUES('guest','l')")
        .execute(&a.pool)
        .await
        .unwrap();
    assert_eq!(catalog(&a, &h).await.unwrap().len(), 1);
    sqlx::query("UPDATE user_access SET can_play=0 WHERE user_id='guest'")
        .execute(&a.pool)
        .await
        .unwrap();
    assert!(users::authorize_item(&a, &h, "i", true).await.is_err());
    assert!(users::authorize_item(&a, &h, "i", false).await.is_ok());
    sqlx::query("UPDATE user_access SET disabled=1 WHERE user_id='guest'")
        .execute(&a.pool)
        .await
        .unwrap();
    assert!(user(&a, &h).await.is_err());
}
#[tokio::test]
async fn last_admin_is_protected_and_access_changes_revoke_sessions() {
    let a = app().await;
    let h = account(&a, "owner", true).await;
    assert!(
        users::update(
            State(a.clone()),
            h.clone(),
            Path("owner".into()),
            Json(access_body(false))
        )
        .await
        .is_err()
    );
    assert!(user(&a, &h).await.is_ok());
    account(&a, "second", true).await;
    let _ = users::update(
        State(a.clone()),
        h.clone(),
        Path("owner".into()),
        Json(access_body(false)),
    )
    .await
    .unwrap();
    assert!(user(&a, &h).await.is_err());
}
#[tokio::test]
async fn failed_login_limit_is_enforced() {
    let a = app().await;
    for _ in 0..5 {
        assert!(
            login(
                State(a.clone()),
                Json(json!({"username":"missing","password":"long enough password"}))
            )
            .await
            .is_err()
        );
    }
    let err = login(
        State(a.clone()),
        Json(json!({"username":"missing","password":"long enough password"})),
    )
    .await
    .unwrap_err();
    assert_eq!(err.0, StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn multiple_roots_edit_remove_preserves_media_and_remaining_progress() {
    let a = app().await;
    let h = account(&a, "owner", true).await;
    let base = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
    let first = base.join("first");
    let second = base.join("second");
    tokio::fs::create_dir_all(&first).await.unwrap();
    tokio::fs::create_dir_all(&second).await.unwrap();
    let Json(created) = add_library(
        State(a.clone()),
        h.clone(),
        Json(json!({"name":"Movies","kind":"movies","paths":[first,second],"enabled":true})),
    )
    .await
    .unwrap();
    let id = created["library"]["id"].as_str().unwrap().to_owned();
    assert_eq!(created["library"]["paths"].as_array().unwrap().len(), 2);
    let file = first.join("fixture.mp4");
    tokio::fs::write(&file, b"read only fixture").await.unwrap();
    sqlx::query("INSERT INTO items VALUES('i',?,?,?)")
        .bind(&id)
        .bind(file.to_string_lossy().as_ref())
        .bind(json!({"id":"i","library_id":id,"duration_seconds":1000}).to_string())
        .execute(&a.pool)
        .await
        .unwrap();
    progress(
        State(a.clone()),
        h.clone(),
        Path("i".into()),
        Json(json!({"position_seconds":200,"completed":false})),
    )
    .await
    .unwrap();
    let _ = libraries::update(
        State(a.clone()),
        h.clone(),
        Path(id.clone()),
        Json(json!({"name":"Renamed","kind":"movies","paths":[first],"enabled":true})),
    )
    .await
    .unwrap();
    assert_eq!(catalog(&a, &h).await.unwrap()[0]["position_seconds"], 200.);
    libraries::remove(State(a.clone()), h, Path(id))
        .await
        .unwrap();
    assert_eq!(tokio::fs::read(&file).await.unwrap(), b"read only fixture");
    tokio::fs::remove_dir_all(base).await.unwrap();
}
#[tokio::test]
async fn progress_is_independent_and_resume_settings_apply() {
    let a = app().await;
    let owner = account(&a, "owner", true).await;
    let other = account(&a, "other", true).await;
    sqlx::query("INSERT INTO libraries VALUES('l','Movies','/media','movies')")
        .execute(&a.pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO items VALUES('i','l','/media/file',?)")
        .bind(json!({"id":"i","library_id":"l","duration_seconds":1000}).to_string())
        .execute(&a.pool)
        .await
        .unwrap();
    progress(
        State(a.clone()),
        owner.clone(),
        Path("i".into()),
        Json(json!({"position_seconds":200,"completed":false})),
    )
    .await
    .unwrap();
    assert_eq!(
        catalog(&a, &other).await.unwrap()[0]["position_seconds"],
        0.
    );
    progress(
        State(a.clone()),
        other.clone(),
        Path("i".into()),
        Json(json!({"position_seconds":10,"completed":false})),
    )
    .await
    .unwrap();
    assert_eq!(
        catalog(&a, &other).await.unwrap()[0]["position_seconds"],
        0.
    );
    progress(
        State(a.clone()),
        other.clone(),
        Path("i".into()),
        Json(json!({"position_seconds":950,"completed":false})),
    )
    .await
    .unwrap();
    assert_eq!(catalog(&a, &other).await.unwrap()[0]["completed"], true);
    assert_eq!(
        catalog(&a, &owner).await.unwrap()[0]["position_seconds"],
        200.
    );
}
#[tokio::test]
async fn password_change_verifies_current_and_revokes_sessions() {
    let a = app().await;
    let h = account(&a, "owner", true).await;
    sqlx::query("UPDATE users SET password=?")
        .bind(hash("old secure password".into()).await.unwrap())
        .execute(&a.pool)
        .await
        .unwrap();
    assert!(
        auth::password(
            State(a.clone()),
            h.clone(),
            Json(json!({"current_password":"wrong","new_password":"new secure password"}))
        )
        .await
        .is_err()
    );
    auth::password(
        State(a.clone()),
        h.clone(),
        Json(
            json!({"current_password":"old secure password","new_password":"new secure password"}),
        ),
    )
    .await
    .unwrap();
    assert!(user(&a, &h).await.is_err());
    assert!(
        login(
            State(a),
            Json(json!({"username":"owner","password":"new secure password"}))
        )
        .await
        .is_ok()
    );
}

#[tokio::test]
async fn scan_reads_multiple_roots_and_preserves_offline_catalog() {
    let a = app().await;
    let h = account(&a, "owner", true).await;
    let base = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
    let first = base.join("first");
    let second = base.join("second");
    tokio::fs::create_dir_all(&first).await.unwrap();
    tokio::fs::create_dir_all(&second).await.unwrap();
    let status = tokio::process::Command::new("ffmpeg")
        .args([
            "-nostdin",
            "-v",
            "error",
            "-f",
            "lavfi",
            "-i",
            "color=size=32x32:rate=1",
            "-t",
            "1",
            "-c:v",
            "libx264",
            "-threads",
            "1",
        ])
        .arg(first.join("One.mp4"))
        .status()
        .await
        .unwrap();
    assert!(status.success());
    tokio::fs::copy(first.join("One.mp4"), second.join("Two.mp4"))
        .await
        .unwrap();
    let Json(created) = add_library(
        State(a.clone()),
        h.clone(),
        Json(json!({"name":"Movies","kind":"movies","paths":[first,second],"enabled":true})),
    )
    .await
    .unwrap();
    let id = created["library"]["id"].as_str().unwrap().to_owned();
    let Json(result) = scan::scan(State(a.clone()), h.clone(), Path(id.clone()))
        .await
        .unwrap();
    assert_eq!(result["scanned"], 2);
    let before = catalog(&a, &h).await.unwrap();
    assert_eq!(before.len(), 2);
    tokio::fs::rename(&second, base.join("offline"))
        .await
        .unwrap();
    let _ = scan::scan(State(a.clone()), h.clone(), Path(id))
        .await
        .unwrap();
    let after = catalog(&a, &h).await.unwrap();
    assert_eq!(after.len(), 2);
    for item in &before {
        assert!(after.iter().any(|new| new["id"] == item["id"]));
    }
    tokio::fs::remove_dir_all(base).await.unwrap();
    let _ = tokio::fs::remove_dir_all(a.artwork_cache.parent().unwrap()).await;
}

#[tokio::test]
async fn settings_persist_and_unknown_library_grants_are_rejected() {
    let a = app().await;
    let h = account(&a, "owner", true).await;
    let changed = settings::Settings {
        resume_min_percent: 12.,
        ..Default::default()
    };
    let _ = settings::put(State(a.clone()), h.clone(), Json(changed))
        .await
        .unwrap();
    let Json(read) = settings::get(State(a.clone()), h.clone()).await.unwrap();
    assert_eq!(read["settings"]["resume_min_percent"], 12.);
    let mut body = access_body(true);
    body["library_ids"] = json!(["nonexistent"]);
    assert!(
        users::update(
            State(a.clone()),
            h.clone(),
            Path("owner".into()),
            Json(body)
        )
        .await
        .is_err()
    );
    assert!(user(&a, &h).await.is_ok());
}
