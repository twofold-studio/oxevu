use super::super::*;
use super::fixture::Fixture;

#[tokio::test]
async fn approved_movie_reaches_playable_library_without_moving_seed_source() {
    let f = Fixture::new().await;
    let a = &f.a;
    let admin = crate::tests::account(a, "admin", true).await;
    let viewer = crate::tests::account(a, "viewer", false).await;
    sqlx::query("INSERT INTO library_grants VALUES('viewer','fixture-library')")
        .execute(&a.pool)
        .await
        .unwrap();
    let request = requests::create_for(
        a,
        &user(a, &viewer).await.unwrap(),
        &f.title,
        scope::Scope::default(),
    )
    .await
    .unwrap();
    let _ = requests::update(
        State(a.clone()),
        admin.clone(),
        Path(request.clone()),
        Json(requests::Decision {
            status: "approved".into(),
            note: String::new(),
        }),
    )
    .await
    .unwrap();
    let c = config::Config {
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
        import_enabled: true,
        ..Default::default()
    };
    let _ = config::put(State(a.clone()), admin.clone(), Json(c))
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
            automatic: false,
        }),
    )
    .await
    .unwrap();
    let _ = indexers::put(
        State(a.clone()),
        admin.clone(),
        Path("fixture".into()),
        Json(indexers::Indexer {
            name: "Fixture tracker".into(),
            endpoint: format!("{}/api", f.endpoint),
            api_key: "fixture-secret".into(),
            enabled: true,
            ..Default::default()
        }),
    )
    .await
    .unwrap();
    let found = releases::search_for(a, "tmdb:movie:1000", &releases::Search::default())
        .await
        .unwrap();
    assert!(!found.to_string().contains("fixture-secret"));
    assert_eq!(found["releases"][0]["accepted"], true);
    assert_eq!(found["releases"][0]["identity_match"], true);
    let release = found["releases"][0]["id"].as_str().unwrap().to_owned();
    let input = downloads::Input {
        release_id: Some(release),
        only_wanted: true,
        ..Default::default()
    };
    let id = downloads::enqueue(a, input.clone()).await.unwrap();
    assert_eq!(downloads::enqueue(a, input).await.unwrap(), id);
    tokio::time::timeout(std::time::Duration::from_secs(30), async {
        let mut last = String::new();
        loop {
            jobs::tick(a).await.unwrap();
            auto_import::tick(a).await.unwrap();
            wanted::fulfill(a).await.unwrap();
            let view = downloads::view(a, &id).await.unwrap();
            let state = format!(
                "{}:{}:{}",
                view["state"], view["error"], view["import_warning"]
            );
            if state != last {
                eprintln!("download: {view}");
                last = state;
            }
            if requests::view(a, &request, &admin).await.unwrap()["status"] == "fulfilled" {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    })
    .await
    .unwrap();
    let row=sqlx::query("SELECT destination,item_id FROM acquisition_imports WHERE download_id=? AND state='scanned'").bind(&id).fetch_one(&a.pool).await.unwrap();
    assert_eq!(
        requests::create_for(
            a,
            &user(a, &viewer).await.unwrap(),
            &f.title,
            scope::Scope::default()
        )
        .await
        .unwrap(),
        request
    );
    let other = crate::tests::account(a, "other", false).await;
    sqlx::query("INSERT INTO library_grants VALUES('other','fixture-library')")
        .execute(&a.pool)
        .await
        .unwrap();
    assert!(
        requests::create_for(
            a,
            &user(a, &other).await.unwrap(),
            &f.title,
            scope::Scope::default()
        )
        .await
        .unwrap_err()
        .1
        .0["error"]
            .as_str()
            .unwrap()
            .contains("already available")
    );
    let target: String = row.get("destination");
    let item: String = row.get("item_id");
    assert_eq!(
        tokio::fs::read(&target).await.unwrap(),
        tokio::fs::read(f.source.join("Fixture Film (2026).mp4"))
            .await
            .unwrap()
    );
    let mut headers = viewer.clone();
    headers.insert("range", "bytes=0-9".parse().unwrap());
    let req = Request::builder()
        .header("range", "bytes=0-9")
        .body(axum::body::Body::empty())
        .unwrap();
    let response = crate::catalog::stream(State(a.clone()), headers, Path(item), req)
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);
    assert!(
        !downloads::view(a, &id)
            .await
            .unwrap()
            .to_string()
            .contains("fixture-secret")
    );
    let _ = downloads::control(
        State(a.clone()),
        admin,
        Path(id),
        Json(downloads::Action {
            action: "remove".into(),
            selected_files: None,
        }),
    )
    .await
    .unwrap();
    assert!(tokio::fs::try_exists(target).await.unwrap());
    assert!(
        tokio::fs::try_exists(f.source.join("Fixture Film (2026).mp4"))
            .await
            .unwrap()
    );
    f.close().await;
}
