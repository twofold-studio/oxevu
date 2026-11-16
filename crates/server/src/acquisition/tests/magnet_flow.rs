use super::fixture::Fixture;
use super::*;

#[tokio::test]
async fn redirected_magnet_downloads_legal_fixture_and_snapshots_release_identity() {
    let f = Fixture::new().await;
    let a = &f.a;
    let admin = crate::tests::account(a, "admin", true).await;
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
            bind_interface: Some("lo".into()),
            dht: false,
            min_free_gb: 0,
            storage_limit_gb: 1,
            ..Default::default()
        }),
    )
    .await
    .unwrap();
    let indexer = indexers::Indexer {
        name: "Redirect fixture".into(),
        endpoint: format!("{}/api", f.endpoint),
        enabled: true,
        ..Default::default()
    };
    let _ = indexers::put(
        State(a.clone()),
        admin.clone(),
        Path("redirect".into()),
        Json(indexer.clone()),
    )
    .await
    .unwrap();
    let raw = json!([{"title":"Fixture Film (2026) 720p WEB-DL x264","guid":"redirect-fixture","download_url":"/magnet","size":1000000,"seeders":5}]);
    let rows = releases::store_results(
        a,
        "redirect",
        &indexer,
        &f.title,
        &raw,
        &profiles::Profile::default(),
    )
    .await
    .unwrap();
    let id = downloads::enqueue(
        a,
        downloads::Input {
            release_id: Some(rows[0]["id"].as_str().unwrap().into()),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    tokio::time::timeout(std::time::Duration::from_secs(30), async {
        loop {
            jobs::tick(a).await.unwrap();
            let view = downloads::view(a, &id).await.unwrap();
            assert_ne!(view["state"], "error", "{view}");
            if view["state"] == "seeding" {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    })
    .await
    .unwrap();
    let view = downloads::view(a, &id).await.unwrap();
    assert_eq!(view["release_title"], raw[0]["title"]);
    assert_eq!(view["indexer_name"], "Redirect fixture");
    let cached: Option<String> = sqlx::query_scalar(
        "SELECT json_extract(data,'$.cached_magnet') FROM acquisition_downloads WHERE id=?",
    )
    .bind(&id)
    .fetch_one(&a.pool)
    .await
    .unwrap();
    assert!(cached.unwrap().starts_with("magnet:?"));
    let listed = downloads::list(State(a.clone()), admin).await.unwrap().0;
    assert_eq!(listed["import_enabled"], false);
    assert_eq!(view["imported"], false);
    f.close().await;
}
