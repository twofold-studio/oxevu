use super::super::*;
use super::fixture::Fixture;

#[tokio::test]
async fn feeds_and_missing_searches_coalesce_and_suspend_on_offline_roots() {
    let f = Fixture::new().await;
    let a = &f.a;
    let admin = crate::tests::account(a, "admin", true).await;
    requests::create_for(
        a,
        &user(a, &admin).await.unwrap(),
        &f.title,
        scope::Scope::default(),
    )
    .await
    .unwrap();
    let _ = config::put(
        State(a.clone()),
        admin.clone(),
        Json(config::Config {
            enabled: true,
            download_dir: f
                .directory
                .path()
                .join("downloads")
                .to_string_lossy()
                .into_owned(),
            listen_port: 0,
            dht: false,
            min_free_gb: 0,
            storage_limit_gb: 1,
            ..Default::default()
        }),
    )
    .await
    .unwrap();
    let _ = monitoring::put(
        State(a.clone()),
        admin.clone(),
        Path("tmdb:movie:1000".into()),
        Json(monitoring::Input {
            enabled: true,
            scope: scope::Scope {
                mode: "requests".into(),
                ..Default::default()
            },
            library_id: Some("fixture-library".into()),
            root: Some(f.root.to_string_lossy().into_owned()),
            profile_id: "default".into(),
            automatic: true,
        }),
    )
    .await
    .unwrap();
    let _ = indexers::put(
        State(a.clone()),
        admin.clone(),
        Path("fixture".into()),
        Json(indexers::Indexer {
            name: "Fixture".into(),
            endpoint: format!("{}/api", f.endpoint),
            enabled: true,
            ..Default::default()
        }),
    )
    .await
    .unwrap();
    feeds::tick(a).await.unwrap();
    monitoring::tick(a).await.unwrap();
    feeds::tick(a).await.unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM acquisition_downloads")
            .fetch_one(&a.pool)
            .await
            .unwrap(),
        1
    );
    sqlx::query("UPDATE acquisition_downloads SET state='removed'")
        .execute(&a.pool)
        .await
        .unwrap();
    tokio::fs::rename(&f.root, f.directory.path().join("offline-library"))
        .await
        .unwrap();
    sqlx::query("UPDATE acquisition_feeds SET next_poll=0")
        .execute(&a.pool)
        .await
        .unwrap();
    sqlx::query("UPDATE acquisition_monitors SET next_search=0")
        .execute(&a.pool)
        .await
        .unwrap();
    monitoring::tick(a).await.unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM acquisition_downloads")
            .fetch_one(&a.pool)
            .await
            .unwrap(),
        1
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT failures FROM acquisition_monitors")
            .fetch_one(&a.pool)
            .await
            .unwrap(),
        1
    );
    f.close().await;
}

#[test]
fn xml_profiles_and_release_identity_do_not_promote_unknown_facts() {
    assert!(
        torznab::parse(
            b"<!DOCTYPE x SYSTEM 'http://localhost/private'><rss/>",
            false
        )
        .is_err()
    );
    assert!(
        torznab::parse(b"<error code='100' description='api-key-secret'/>", false)
            .unwrap_err()
            .1
            .0
            .to_string()
            .find("api-key-secret")
            .is_none()
    );
    let mut r = release_facts::facts("Fixture Film (2026) 720p WEB-DL x264").unwrap();
    r["title"] = json!("Fixture Film (2026) 720p WEB-DL x264");
    r["size"] = json!(1024);
    r["seeders"] = json!(10);
    assert!(profiles::evaluate(&profiles::Profile::default(), &r).is_empty());
    assert!(
        release_facts::identity_matches(
            &json!({"title":"Fixture Film","kind":"movie","year":"2026","provider_id":1}),
            &r
        )
        .unwrap()
    );
    assert!(
        !release_facts::identity_matches(
            &json!({"title":"Fixture Film Two","kind":"movie","year":"2026","provider_id":2}),
            &r
        )
        .unwrap()
    );
    r["height"] = Value::Null;
    assert!(!profiles::evaluate(&profiles::Profile::default(), &r).is_empty());
    let safe = import_plan::safe_name("../../A/B:* Film");
    assert_eq!(std::path::Path::new(&safe).components().count(), 1);
    assert!(!matches!(safe.as_str(), "." | ".."));
}

#[tokio::test]
async fn competing_release_claims_deduplicate_without_blocking_later_episodes() {
    let a = crate::tests::app().await;
    let t = super::title(&a, "tv", 1234).await;
    sqlx::query("INSERT INTO acquisition_config VALUES(1,?)")
        .bind(json!({"enabled":true}).to_string())
        .execute(&a.pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO acquisition_monitors(title_id,scope) VALUES(?,?)")
        .bind(t["id"].as_str())
        .bind(json!({"mode":"all"}).to_string())
        .execute(&a.pool)
        .await
        .unwrap();
    for (id, episode) in [("first", 1), ("duplicate", 1), ("next", 2)] {
        sqlx::query("INSERT INTO acquisition_releases(id,title_id,data) VALUES(?,?,?)").bind(id).bind(t["id"].as_str()).bind(json!({"title":"Generated fixture S01E01 1080p x264","season":1,"episode":episode,"height":1080,"codec":"h264","size":1000,"seeders":10}).to_string()).execute(&a.pool).await.unwrap();
    }
    let input = |id: &str| downloads::Input {
        release_id: Some(id.into()),
        only_wanted: true,
        ..Default::default()
    };
    let first = downloads::enqueue(&a, input("first")).await.unwrap();
    sqlx::query("UPDATE acquisition_downloads SET state='seeding' WHERE id=?")
        .bind(&first)
        .execute(&a.pool)
        .await
        .unwrap();
    assert_eq!(
        downloads::enqueue(&a, input("duplicate")).await.unwrap(),
        first
    );
    assert_ne!(downloads::enqueue(&a, input("next")).await.unwrap(), first);
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM acquisition_downloads")
            .fetch_one(&a.pool)
            .await
            .unwrap(),
        2
    );
}
