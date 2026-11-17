use super::*;
use serde_json::json;

async fn fixture() -> (tempfile::TempDir, SqlitePool) {
    let root = tempfile::tempdir().unwrap();
    let pool = oxevu_db::open(root.path()).await.unwrap();
    sqlx::query("INSERT INTO libraries VALUES('library','Movies',?,'movies')")
        .bind(root.path().to_string_lossy().as_ref())
        .execute(&pool)
        .await
        .unwrap();
    for id in ["one", "two"] {
        let path = root.path().join(format!("{id}.mkv"));
        tokio::fs::write(&path, b"same legal fixture")
            .await
            .unwrap();
        oxevu_db::scanning::upsert(
            &pool,
            id,
            "library",
            path.to_string_lossy().as_ref(),
            &json!({"id":id,"library_id":"library","kind":"movie","title":"Film"}).to_string(),
        )
        .await
        .unwrap();
    }
    (root, pool)
}

#[tokio::test]
async fn merging_retains_files_progress_favorites_and_metadata_and_survives_rescan() {
    let (root, pool) = fixture().await;
    sqlx::query("INSERT INTO users VALUES('viewer','Viewer','test-hash')")
        .execute(&pool)
        .await
        .unwrap();
    for (id, position) in [("one", 10.0), ("two", 37.0)] {
        oxevu_db::catalog::progress(&pool, "viewer", id, position, false)
            .await
            .unwrap();
    }
    oxevu_db::catalog::favorite(&pool, "viewer", "two", true)
        .await
        .unwrap();
    oxevu_db::identification::publish(
        &pool,
        &[(
            "two".into(),
            json!({"overview":"Overview","poster_url":"/api/v1/items/two/metadata/artwork/poster"})
                .to_string(),
        )],
    )
    .await
    .unwrap();
    merge_identical(&pool, "one", "two").await.unwrap();
    let position: f64 = sqlx::query_scalar("SELECT position FROM progress WHERE item_id='one'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(position, 37.0);
    let favorite: String = sqlx::query_scalar("SELECT item_id FROM favorites")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(favorite, "one");
    let metadata: String = sqlx::query_scalar("SELECT data FROM item_metadata WHERE item_id='one'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&metadata).unwrap()["poster_url"],
        "/api/v1/items/one/metadata/artwork/poster"
    );
    for id in ["one", "two"] {
        assert!(root.path().join(format!("{id}.mkv")).exists());
    }
    let duplicate = root.path().join("two.mkv");
    oxevu_db::scanning::upsert(
        &pool,
        "two",
        "library",
        duplicate.to_string_lossy().as_ref(),
        "{}",
    )
    .await
    .unwrap();
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM items")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 1);
}

#[tokio::test]
async fn different_content_cannot_be_merged() {
    let (root, pool) = fixture().await;
    tokio::fs::write(root.path().join("two.mkv"), b"different fixture")
        .await
        .unwrap();
    assert!(merge_identical(&pool, "one", "two").await.is_err());
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM items")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 2);
}
