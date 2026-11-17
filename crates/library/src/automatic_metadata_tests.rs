use super::*;
use serde_json::json;

struct Offline;
impl MetadataSource for Offline {
    fn tmdb<'a>(
        &'a self,
        _: &'a str,
        _: &'a [(&'a str, String)],
    ) -> identification::MetadataFuture<'a> {
        Box::pin(async { Err(bad("provider offline")) })
    }
    fn omdb<'a>(&'a self, _: &'a [(&'a str, String)]) -> identification::MetadataFuture<'a> {
        Box::pin(async { Err(bad("provider offline")) })
    }
}

#[test]
fn release_names_become_conservative_title_and_year_queries() {
    for (raw, expected, year) in [
        (
            "A Knights Tale 2001 1080p BluRay",
            "A Knights Tale",
            Some("2001"),
        ),
        (
            "Avatar Fire and Ash 2025 1080p WEB-DL",
            "Avatar Fire and Ash",
            Some("2025"),
        ),
        ("Fallout", "Fallout", None),
        (
            "Blade Runner 2049 2017 1080p BluRay",
            "Blade Runner 2049",
            Some("2017"),
        ),
        (
            "2001 A Space Odyssey 1968 1080p",
            "2001 A Space Odyssey",
            Some("1968"),
        ),
    ] {
        let (name, found) = title(
            &json!({"title":raw,"kind":"movie"}),
            Path::new("/movies/file.mkv"),
        );
        assert_eq!((name.as_str(), found.as_deref()), (expected, year));
    }
}

#[tokio::test]
async fn local_nfo_enriches_import_and_matching_state_survives_rescan() {
    let temp = tempfile::tempdir().unwrap();
    let pool = oxevu_db::open(temp.path()).await.unwrap();
    let media = temp.path().join("movie.mkv");
    tokio::fs::write(&media, b"fixture").await.unwrap();
    tokio::fs::write(
        media.with_extension("nfo"),
        b"<movie><title>Title</title><plot>Overview</plot><year>2001</year></movie>",
    )
    .await
    .unwrap();
    sqlx::query("INSERT INTO libraries VALUES('library','Movies',?,'movies')")
        .bind(temp.path().to_string_lossy().as_ref())
        .execute(&pool)
        .await
        .unwrap();
    let data = json!({"id":"one","library_id":"library","title":"Release name","kind":"movie"})
        .to_string();
    oxevu_db::scanning::upsert(
        &pool,
        "one",
        "library",
        media.to_string_lossy().as_ref(),
        &data,
    )
    .await
    .unwrap();
    let job = repository::next(&pool).await.unwrap().unwrap();
    enrich(&pool, &job, &Offline, false, false).await.unwrap();
    assert!(repository::next(&pool).await.unwrap().is_none());
    let matched: String = sqlx::query_scalar("SELECT data FROM item_metadata WHERE item_id='one'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        serde_json::from_str::<Value>(&matched).unwrap()["title"],
        "Title"
    );
    repository::failed(&pool, "one", "test failure")
        .await
        .unwrap();
    oxevu_db::scanning::upsert(
        &pool,
        "one",
        "library",
        media.to_string_lossy().as_ref(),
        &data,
    )
    .await
    .unwrap();
    let snapshot = oxevu_db::catalog::snapshot(&pool, "viewer").await.unwrap();
    let visible = ["one".to_owned()].into_iter().collect();
    assert_eq!(
        super::super::catalog::project(snapshot, &visible).unwrap()[0]["metadata_status"],
        "identified"
    );
    let second_dir = temp.path().join("unknown");
    tokio::fs::create_dir(&second_dir).await.unwrap();
    let second = second_dir.join("unmatched.mkv");
    tokio::fs::write(&second, b"fixture").await.unwrap();
    let second_data =
        json!({"id":"two","library_id":"library","title":"Unknown","kind":"movie"}).to_string();
    oxevu_db::scanning::upsert(
        &pool,
        "two",
        "library",
        second.to_string_lossy().as_ref(),
        &second_data,
    )
    .await
    .unwrap();
    let job = repository::next(&pool).await.unwrap().unwrap();
    let error = enrich(&pool, &job, &Offline, false, false)
        .await
        .unwrap_err();
    repository::failed(&pool, &job.id, &error.to_string())
        .await
        .unwrap();
    oxevu_db::scanning::upsert(
        &pool,
        "two",
        "library",
        second.to_string_lossy().as_ref(),
        &second_data,
    )
    .await
    .unwrap();
    assert!(repository::next(&pool).await.unwrap().is_none());
    let snapshot = oxevu_db::catalog::snapshot(&pool, "viewer").await.unwrap();
    let visible = ["two".to_owned()].into_iter().collect();
    assert_eq!(
        super::super::catalog::project(snapshot, &visible).unwrap()[0]["metadata_status"],
        "needs_identification"
    );
}

#[tokio::test]
async fn automatic_publication_fills_partial_import_but_preserves_manual_matches() {
    let temp = tempfile::tempdir().unwrap();
    let pool = oxevu_db::open(temp.path()).await.unwrap();
    sqlx::query("INSERT INTO libraries VALUES('library','Movies','/virtual','movies')")
        .execute(&pool)
        .await
        .unwrap();
    oxevu_db::scanning::upsert(&pool, "one", "library", "/virtual/file", "{}")
        .await
        .unwrap();
    oxevu_db::identification::publish(
        &pool,
        &[(
            "one".into(),
            json!({"poster_url":"local-poster"}).to_string(),
        )],
    )
    .await
    .unwrap();
    repository::publish_unmatched(
        &pool,
        "one",
        &json!({"title":"Auto title","overview":"Auto overview"}).to_string(),
    )
    .await
    .unwrap();
    let get = || async {
        let raw: String = sqlx::query_scalar("SELECT data FROM item_metadata WHERE item_id='one'")
            .fetch_one(&pool)
            .await
            .unwrap();
        serde_json::from_str::<Value>(&raw).unwrap()
    };
    assert_eq!(get().await["poster_url"], "local-poster");
    assert_eq!(get().await["title"], "Auto title");
    oxevu_db::identification::publish(
        &pool,
        &[(
            "one".into(),
            json!({"title":"Manual title","metadata_source":"manual"}).to_string(),
        )],
    )
    .await
    .unwrap();
    repository::publish_unmatched(
        &pool,
        "one",
        &json!({"title":"Wrong replacement","overview":"Overview"}).to_string(),
    )
    .await
    .unwrap();
    assert_eq!(get().await["title"], "Manual title");
}
