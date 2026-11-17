use super::tests::{app, session};
use super::*;
use librqbit::{AddTorrent, AddTorrentOptions, CreateTorrentOptions, spawn_utils::BlockingSpawner};

#[tokio::test]
async fn progressive_session_converts_verified_media_and_releases_private_reader() {
    let dir = tempfile::tempdir().unwrap();
    let a = app(dir.path()).await;
    let headers = crate::tests::account(&a, "owner", true).await;
    let root = dir.path().join("seed");
    tokio::fs::create_dir(&root).await.unwrap();
    let path = root.join("Legal.Movie.2026.mp4");
    assert!(
        tokio::process::Command::new("ffmpeg")
            .args([
                "-nostdin",
                "-v",
                "error",
                "-f",
                "lavfi",
                "-i",
                "color=size=32x32:rate=4",
                "-t",
                "2",
                "-c:v",
                "libx264",
                "-threads",
                "1"
            ])
            .arg(&path)
            .status()
            .await
            .unwrap()
            .success()
    );
    let size = tokio::fs::metadata(&path).await.unwrap().len();
    let torrent = librqbit::create_torrent(
        &root,
        CreateTorrentOptions {
            name: Some("legal-film"),
            piece_length: Some(16 * 1024),
            trackers: vec![],
        },
        &BlockingSpawner::new(2),
    )
    .await
    .unwrap()
    .as_bytes()
    .unwrap()
    .to_vec();
    let engine = session(root.clone()).await;
    let handle = engine
        .add_torrent(
            AddTorrent::from_bytes(torrent),
            Some(AddTorrentOptions {
                output_folder: Some(root.to_string_lossy().into_owned()),
                overwrite: true,
                paused: true,
                ..Default::default()
            }),
        )
        .await
        .unwrap()
        .into_handle()
        .unwrap();
    handle.wait_until_initialized().await.unwrap();
    a.acquisition.use_test_engine(engine.clone()).await;
    sqlx::query("INSERT INTO libraries VALUES('movies','Movies',?,'movies')")
        .bind(dir.path().join("library").to_string_lossy().as_ref())
        .execute(&a.pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO library_roots(library_id,path) SELECT id,path FROM libraries")
        .execute(&a.pool)
        .await
        .unwrap();
    let title = json!({"id":"tmdb:movie:1","kind":"movie","provider_id":1,"title":"Legal Movie","year":"2026"});
    sqlx::query("INSERT INTO acquisition_titles(id,kind,provider_id,data) VALUES('tmdb:movie:1','movie',1,?)").bind(title.to_string()).execute(&a.pool).await.unwrap();
    sqlx::query("INSERT INTO acquisition_downloads(id,title_id,infohash,data,state) VALUES('download','tmdb:movie:1',?,?,'completed')").bind(handle.info_hash().as_string()).bind(json!({"input":{},"files":[{"index":0,"path":"Legal.Movie.2026.mp4","size":size}],"stats":{}}).to_string()).execute(&a.pool).await.unwrap();
    sqlx::query("INSERT INTO acquisition_config VALUES(1,?)")
        .bind(json!({"enabled":true,"auto_identify":false}).to_string())
        .execute(&a.pool)
        .await
        .unwrap();
    super::super::downloading::sync(&a).await.unwrap();
    let id: String = sqlx::query_scalar("SELECT item_id FROM acquisition_stream_items")
        .fetch_one(&a.pool)
        .await
        .unwrap();
    let Json(created)=playback::create(State(a.clone()),headers.clone(),Json(json!({"item_id":id,"mode":"transcode","direct_play_supported":false,"start_seconds":0}))).await.unwrap();
    let session = created["session"]["id"].as_str().unwrap();
    assert_eq!(created["session"]["progressive"], true);
    let deadline = tokio::time::Instant::now() + Duration::from_secs(15);
    loop {
        let Json(current) =
            playback::get(State(a.clone()), headers.clone(), Path(session.to_owned()))
                .await
                .unwrap();
        assert_ne!(current["session"]["status"], "failed", "{current}");
        if current["session"]["status"] == "ready" {
            assert_eq!(current["session"]["duration_seconds"], 2.0);
            break;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "HLS did not become ready"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let response = playback::hls(
        State(a.clone()),
        headers.clone(),
        Path((session.to_owned(), "index.m3u8".into())),
        Request::new(Body::empty()),
    )
    .await
    .unwrap();
    let playlist = axum::body::to_bytes(response.into_body(), 65536)
        .await
        .unwrap();
    assert!(String::from_utf8_lossy(&playlist).contains("segment000000.ts"));
    playback::stop(State(a.clone()), headers, Path(session.to_owned()))
        .await
        .unwrap();
    assert_eq!(a.acquisition.streams.available_permits(), 8);
    a.acquisition.shutdown().await;
    a.playback.shutdown().await;
}
