use super::super::*;
use super::fixture::Fixture;
use base64::{Engine, engine::general_purpose::STANDARD};

#[tokio::test]
async fn adoption_copies_rechecks_and_never_shares_writable_source_inodes() {
    let f = Fixture::new().await;
    let a = &f.a;
    let admin = crate::tests::account(a, "admin", true).await;
    let viewer = crate::tests::account(a, "viewer", false).await;
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
    let torrent = reqwest::get(format!("{}/torrent", f.endpoint))
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();
    let id = downloads::enqueue(
        a,
        downloads::Input {
            torrent_base64: Some(STANDARD.encode(torrent)),
            paused: true,
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let input = || adoption::Input {
        source_root: f.source.to_string_lossy().into_owned(),
        token: None,
    };
    assert_eq!(
        downloads::view(a, &id).await.unwrap()["files"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        adoption::preview(State(a.clone()), viewer, Path(id.clone()), Json(input()))
            .await
            .unwrap_err()
            .0,
        StatusCode::FORBIDDEN
    );
    let preview = adoption::preview(
        State(a.clone()),
        admin.clone(),
        Path(id.clone()),
        Json(input()),
    )
    .await
    .unwrap()
    .0;
    let mut reviewed = input();
    reviewed.token = preview["token"].as_str().map(str::to_owned);
    let _ = adoption::apply(
        State(a.clone()),
        admin.clone(),
        Path(id.clone()),
        Json(reviewed),
    )
    .await
    .unwrap();
    assert_eq!(downloads::view(a, &id).await.unwrap()["state"], "paused");
    let target = preview["files"][0]["destination"].as_str().unwrap();
    let original = f.source.join("Fixture Film (2026).mp4");
    use std::os::unix::fs::MetadataExt;
    assert_ne!(
        tokio::fs::metadata(target).await.unwrap().ino(),
        tokio::fs::metadata(&original).await.unwrap().ino()
    );
    assert_eq!(
        tokio::fs::read(target).await.unwrap(),
        tokio::fs::read(&original).await.unwrap()
    );
    let _ = downloads::control(
        State(a.clone()),
        admin,
        Path(id.clone()),
        Json(downloads::Action {
            action: "resume".into(),
            selected_files: None,
        }),
    )
    .await
    .unwrap();
    tokio::time::timeout(std::time::Duration::from_secs(10), async {
        loop {
            jobs::tick(a).await.unwrap();
            if downloads::view(a, &id).await.unwrap()["stats"]["finished"] == true {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
    })
    .await
    .unwrap();
    f.close().await;
}

#[tokio::test]
async fn http_routes_enforce_auth_roles_origin_and_storage_separation() {
    use tower::ServiceExt;
    let f = Fixture::new().await;
    let a = &f.a;
    let admin = crate::tests::account(a, "admin", true).await;
    let viewer = crate::tests::account(a, "viewer", false).await;
    let router = routes()
        .layer(middleware::from_fn_with_state(a.clone(), origin_check))
        .with_state(a.clone());
    for (path, headers, status) in [
        (
            "/acquisition/downloads",
            HeaderMap::new(),
            StatusCode::UNAUTHORIZED,
        ),
        (
            "/acquisition/settings",
            viewer.clone(),
            StatusCode::FORBIDDEN,
        ),
        ("/acquisition/settings", admin.clone(), StatusCode::OK),
    ] {
        let mut request = Request::builder()
            .uri(path)
            .body(axum::body::Body::empty())
            .unwrap();
        *request.headers_mut() = headers;
        assert_eq!(
            router.clone().oneshot(request).await.unwrap().status(),
            status
        );
    }
    let mut request = Request::builder()
        .method("PUT")
        .uri("/acquisition/settings")
        .header("content-type", "application/json")
        .body(axum::body::Body::from("{}"))
        .unwrap();
    request.headers_mut().extend(admin.clone());
    assert_eq!(
        router.oneshot(request).await.unwrap().status(),
        StatusCode::FORBIDDEN
    );
    for path in [&f.root, &a.artwork_cache, &f.directory.path().join("app")] {
        assert!(storage::validate_download_root(a, path).await.is_err());
    }
    assert!(
        config::put(
            State(a.clone()),
            admin,
            Json(config::Config {
                download_dir: f.root.to_string_lossy().into_owned(),
                ..Default::default()
            })
        )
        .await
        .is_err()
    );
    f.close().await;
}
