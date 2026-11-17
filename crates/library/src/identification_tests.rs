use super::*;

struct Provider;
impl MetadataSource for Provider {
    fn tmdb<'a>(&'a self, route: &'a str, _: &'a [(&'a str, String)]) -> MetadataFuture<'a> {
        Box::pin(async move {
            if route.ends_with("episode/2") {
                return Err(bad("provider unavailable"));
            }
            Ok(
                json!({"name":"Matched title","overview":"Overview","vote_average":8.0,"first_air_date":"2020-01-01"}),
            )
        })
    }
    fn omdb<'a>(&'a self, _: &'a [(&'a str, String)]) -> MetadataFuture<'a> {
        Box::pin(async {
            Ok(
                json!({"Title":"Matched film","Year":"2020","Plot":"Overview","imdbRating":"8.1","Genre":"Drama, Action"}),
            )
        })
    }
}

async fn fixture(tv: bool) -> (tempfile::TempDir, SqlitePool, Value) {
    let root = tempfile::tempdir().unwrap();
    let pool = oxevu_db::open(root.path()).await.unwrap();
    sqlx::query("INSERT INTO libraries VALUES('library','Library','/virtual',?)")
        .bind(if tv { "tv" } else { "movies" })
        .execute(&pool)
        .await
        .unwrap();
    let item = json!({"id":"one","library_id":"library","kind":if tv {"episode"} else {"movie"},"title":"Original title","show_title":"Show","season_number":1,"episode_number":1});
    sqlx::query("INSERT INTO items VALUES('one','library','/virtual/one',?)")
        .bind(item.to_string())
        .execute(&pool)
        .await
        .unwrap();
    (root, pool, item)
}

#[tokio::test]
async fn omdb_matching_publishes_normalized_metadata_without_application_state() {
    let (_root, pool, item) = fixture(false).await;
    apply(
        &pool,
        "one",
        item,
        Selection {
            provider: "omdb".into(),
            id: "tt123".into(),
        },
        &Provider,
    )
    .await
    .unwrap();
    let raw: String = sqlx::query_scalar("SELECT data FROM item_metadata WHERE item_id='one'")
        .fetch_one(&pool)
        .await
        .unwrap();
    let value: Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(value["title"], "Matched film");
    assert_eq!(value["year"], 2020);
    assert_eq!(value["genres"], json!(["Drama", "Action"]));
    assert_eq!(value["provider_ids"]["imdb"], "tt123");
}

#[tokio::test]
async fn provider_failure_never_publishes_partial_series_metadata() {
    let (_root, pool, item) = fixture(true).await;
    let mut second = item.clone();
    second["id"] = json!("two");
    second["episode_number"] = json!(2);
    sqlx::query("INSERT INTO items VALUES('two','library','/virtual/two',?)")
        .bind(second.to_string())
        .execute(&pool)
        .await
        .unwrap();
    assert!(
        apply(
            &pool,
            "one",
            item,
            Selection {
                provider: "tmdb".into(),
                id: "123".into()
            },
            &Provider
        )
        .await
        .is_err()
    );
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM item_metadata")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
}
