use super::*;
use std::os::unix::fs::symlink;

async fn fixture() -> (App, HeaderMap, tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    oxevu_db::MIGRATOR.run(&pool).await.unwrap();
    let playback = playback::Runtime::new(dir.path(), &pool).await.unwrap();
    let a = App {
        pool,
        playback,
        acquisition: acquisition::Runtime::new(dir.path().to_owned()),
        parties: Arc::new(parties::Store::default()),
        token: "test".into(),
        origin: "http://localhost:3000".into(),
        allowed_origins: Vec::new(),
        scan: Arc::new(Semaphore::new(1)),
        artwork_jobs: Arc::new(Semaphore::new(1)),
        artwork_cache: dir.path().join("artwork"),
    };
    let h = crate::tests::account(&a, "owner", true).await;
    let root = dir.path().join("media");
    tokio::fs::create_dir(&root).await.unwrap();
    let path = root.join("Film.mp4");
    tokio::fs::write(&path, b"owned media fixture")
        .await
        .unwrap();
    sqlx::query("INSERT INTO libraries VALUES('lib','Movies',?,'movies')")
        .bind(root.to_str())
        .execute(&a.pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO library_roots(library_id,path) VALUES('lib',?)")
        .bind(root.to_str())
        .execute(&a.pool)
        .await
        .unwrap();
    let data = json!({"id":"film","library_id":"lib","kind":"movie","title":"Film","file_available":true,"position_seconds":0,"completed":false});
    sqlx::query("INSERT INTO items VALUES('film','lib',?,?)")
        .bind(path.to_str())
        .bind(data.to_string())
        .execute(&a.pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO item_metadata VALUES('film','{\"provider_ids\":{\"tmdb\":1},\"overview\":\"Saved overview\",\"rating\":8.1}')").execute(&a.pool).await.unwrap();
    sqlx::query("INSERT INTO acquisition_titles(id,kind,provider_id,data) VALUES('tmdb:movie:1','movie',1,'{}')").execute(&a.pool).await.unwrap();
    let u = user(&a, &h).await.unwrap();
    sqlx::query("INSERT INTO progress VALUES(?,'film',120,1)")
        .bind(u["id"].as_str())
        .execute(&a.pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO favorites VALUES(?,'film')")
        .bind(u["id"].as_str())
        .execute(&a.pool)
        .await
        .unwrap();
    let Json(opinion) = reaction(
        State(a.clone()),
        h.clone(),
        Path("film".into()),
        Json(json!({"value":"like"})),
    )
    .await
    .unwrap();
    assert_eq!(opinion["value"], "like");
    (a, h, dir, path)
}
fn input(keep: bool, token: Option<String>) -> Input {
    Input {
        item_ids: vec!["film".into()],
        keep_metadata: keep,
        confirm: token.is_some(),
        token,
    }
}
async fn remove(a: &App, h: &HeaderMap, keep: bool) -> Value {
    let Json(preview) = preview(State(a.clone()), h.clone(), Json(input(keep, None)))
        .await
        .unwrap();
    apply(
        State(a.clone()),
        h.clone(),
        Json(input(keep, Some(preview["token"].as_str().unwrap().into()))),
    )
    .await
    .unwrap()
    .0
}

#[tokio::test]
async fn media_removal_keeps_metadata_history_and_reactions_and_reports_shared_links() {
    let (a, h, dir, path) = fixture().await;
    let seed = dir.path().join("seeding.mp4");
    tokio::fs::hard_link(&path, &seed).await.unwrap();
    let result = remove(&a, &h, true).await;
    assert!(!path.exists());
    assert!(seed.exists());
    assert_eq!(result["shared_files_count"], 1);
    assert_eq!(result["deleted_files"], 1);
    assert_eq!(result["bytes_freed_estimate"], 0);
    let Json(scan) = scan::scan_library(a.clone(), "lib".into()).await.unwrap();
    assert_eq!(scan["scanned"], 0);
    let item = catalog::catalog(&a, &h).await.unwrap().remove(0);
    assert_eq!(item["file_available"], false);
    assert_eq!(item["completed"], true);
    assert_eq!(item["favorite"], true);
    assert_eq!(item["reaction"], "like");
    assert_eq!(item["overview"], "Saved overview");
    let suppressed: i64 = sqlx::query_scalar("SELECT count(*) FROM acquisition_removed_units")
        .fetch_one(&a.pool)
        .await
        .unwrap();
    assert_eq!(suppressed, 1);
}
#[tokio::test]
async fn media_removal_can_forget_all_item_history_after_explicit_confirmation() {
    let (a, h, _dir, path) = fixture().await;
    let result = remove(&a, &h, false).await;
    assert_eq!(result["removed_ids"], json!(["film"]));
    assert!(!path.exists());
    for table in [
        "items",
        "item_metadata",
        "progress",
        "favorites",
        "item_reactions",
    ] {
        let count: i64 = sqlx::query_scalar(&format!("SELECT count(*) FROM {table}"))
            .fetch_one(&a.pool)
            .await
            .unwrap();
        assert_eq!(count, 0, "{table}");
    }
}
#[tokio::test]
async fn media_removal_requires_admin_confirmation_and_a_current_preview() {
    let (a, h, _dir, path) = fixture().await;
    let viewer = crate::tests::account(&a, "viewer", false).await;
    assert!(
        preview(State(a.clone()), viewer, Json(input(true, None)))
            .await
            .is_err()
    );
    assert!(
        apply(State(a.clone()), h.clone(), Json(input(true, None)))
            .await
            .is_err()
    );
    let Json(old) = preview(State(a.clone()), h.clone(), Json(input(true, None)))
        .await
        .unwrap();
    tokio::fs::write(&path, b"changed fixture bytes")
        .await
        .unwrap();
    assert!(
        apply(
            State(a.clone()),
            h,
            Json(input(true, Some(old["token"].as_str().unwrap().into())))
        )
        .await
        .is_err()
    );
    assert!(path.exists());
}
#[tokio::test]
async fn media_removal_refuses_symlinks_and_preserves_their_target() {
    let (a, h, dir, path) = fixture().await;
    let outside = dir.path().join("outside.mp4");
    tokio::fs::write(&outside, b"must remain").await.unwrap();
    tokio::fs::remove_file(&path).await.unwrap();
    symlink(&outside, &path).unwrap();
    assert!(preview(State(a), h, Json(input(true, None))).await.is_err());
    assert_eq!(tokio::fs::read(outside).await.unwrap(), b"must remain");
}
#[tokio::test]
async fn media_removal_restored_release_reuses_the_retained_identity_and_history() {
    let (a, h, dir, _path) = fixture().await;
    remove(&a, &h, true).await;
    let path = dir.path().join("media/Film.new.mkv");
    tokio::fs::write(&path, b"new release").await.unwrap();
    let data = json!({"id":"new-film","library_id":"lib","kind":"movie","title":"Film new","file_available":true});
    sqlx::query("INSERT INTO items VALUES('new-film','lib',?,?)")
        .bind(path.to_str())
        .bind(data.to_string())
        .execute(&a.pool)
        .await
        .unwrap();
    assert_eq!(
        restore_identity(&a, "new-film", &json!({"provider_id":1}))
            .await
            .unwrap(),
        "film"
    );
    let items = catalog::catalog(&a, &h).await.unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["id"], "film");
    assert_eq!(items[0]["completed"], true);
    assert_eq!(items[0]["reaction"], "like");
    assert_eq!(items[0]["file_available"], true);
}

#[tokio::test]
async fn media_removal_refuses_an_active_playback_session() {
    let (a, h, _dir, path) = fixture().await;
    let (cancel, _) = tokio::sync::watch::channel(false);
    let (_, done) = tokio::sync::watch::channel(true);
    a.playback.sessions.lock().await.insert(
        "playing".into(),
        playback::runtime::Entry {
            pid: None,
            owner: "owner".into(),
            item: "film".into(),
            value: json!({"status":"ready"}),
            touched: std::time::Instant::now(),
            idle: 0,
            transcode: false,
            cancel,
            done,
        },
    );
    let Json(preview) = preview(State(a.clone()), h.clone(), Json(input(true, None)))
        .await
        .unwrap();
    assert!(
        apply(
            State(a),
            h,
            Json(input(true, Some(preview["token"].as_str().unwrap().into())))
        )
        .await
        .is_err()
    );
    assert!(path.exists());
}
