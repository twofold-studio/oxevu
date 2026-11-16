use super::*;
use axum::{response::IntoResponse, routing::get};

#[tokio::test]
async fn magnet_and_one_same_origin_redirect_work_without_following_external_locations() {
    let magnet = format!("magnet:?xt=urn:btih:{}&dn=fixture", "a".repeat(40));
    let location = magnet.clone();
    let router =
        Router::new()
            .route(
                "/magnet",
                get(move || {
                    let location = location.clone();
                    async move {
                        (StatusCode::MOVED_PERMANENTLY, [("location", location)]).into_response()
                    }
                }),
            )
            .route(
                "/relative",
                get(|| async { (StatusCode::FOUND, [("location", "/torrent")]).into_response() }),
            )
            .route("/torrent", get(|| async { "legal torrent fixture" }))
            .route(
                "/external",
                get(|| async {
                    (
                        StatusCode::TEMPORARY_REDIRECT,
                        [("location", "http://169.254.169.254/latest/meta-data")],
                    )
                        .into_response()
                }),
            )
            .route(
                "/loop",
                get(|| async {
                    (StatusCode::PERMANENT_REDIRECT, [("location", "/loop")]).into_response()
                }),
            )
            .route("/rate", get(|| async { StatusCode::TOO_MANY_REQUESTS }));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let endpoint = format!("http://{}/api", listener.local_addr().unwrap());
    let task = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    let indexer = indexers::Indexer {
        endpoint,
        ..Default::default()
    };
    match indexers::fetch_torrent(&indexer, "/magnet").await.unwrap() {
        indexers::TorrentSource::Magnet(value) => assert_eq!(value, magnet),
        _ => panic!("expected magnet"),
    }
    match indexers::fetch_torrent(&indexer, "/relative")
        .await
        .unwrap()
    {
        indexers::TorrentSource::Bytes(value) => assert_eq!(value, b"legal torrent fixture"),
        _ => panic!("expected bytes"),
    }
    for path in ["/external", "/loop"] {
        assert!(indexers::fetch_torrent(&indexer, path).await.is_err());
    }
    let error = indexers::fetch_torrent(&indexer, "/rate")
        .await
        .err()
        .unwrap();
    assert!(error.1.0["error"].as_str().unwrap().contains("429"));
    assert_eq!(error.1.0["retryable"], true);
    task.abort();
    let _ = task.await;
}
