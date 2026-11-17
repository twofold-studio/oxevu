use super::*;
use librqbit::{
    AddTorrent, AddTorrentOptions, CreateTorrentOptions, ListenerMode, ListenerOptions, Session,
    SessionOptions, spawn_utils::BlockingSpawner,
};

pub(super) async fn app(root: &std::path::Path) -> App {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    oxevu_db::MIGRATOR.run(&pool).await.unwrap();
    App {
        playback: playback::Runtime::new(root, &pool).await.unwrap(),
        pool,
        acquisition: acquisition::Runtime::new(root.to_owned()),
        parties: Arc::new(parties::Store::default()),
        token: "fixture".into(),
        origin: "http://localhost:3000".into(),
        allowed_origins: Vec::new(),
        scan: Arc::new(Semaphore::new(1)),
        artwork_jobs: Arc::new(Semaphore::new(1)),
        artwork_cache: root.join("artwork"),
    }
}
pub(super) async fn session(path: PathBuf) -> Arc<Session> {
    Session::new_with_opts(
        path,
        SessionOptions {
            dht: None,
            persistence: None,
            disable_trackers: true,
            disable_local_service_discovery: true,
            bind_device_name: Some("lo".into()),
            listen: Some(ListenerOptions {
                listen_addr: ([127, 0, 0, 1], 0).into(),
                mode: ListenerMode::TcpOnly,
                enable_upnp_port_forwarding: false,
                ..Default::default()
            }),
            runtime_worker_threads: Some(MAX_READERS + 2),
            ..Default::default()
        },
    )
    .await
    .unwrap()
}

#[tokio::test]
async fn verified_stream_waits_for_pieces_and_seeks_before_completion() {
    let dir = tempfile::tempdir().unwrap();
    let a = app(dir.path()).await;
    let headers = crate::tests::account(&a, "owner", true).await;
    let source_dir = dir.path().join("seed");
    tokio::fs::create_dir(&source_dir).await.unwrap();
    // Generated legal data: a sparse hole must never appear in the HTTP response.
    let payload: Vec<u8> = (0..8 * 1024 * 1024)
        .map(|i| ((i * 31 + i / 4096) % 251) as u8)
        .collect();
    tokio::fs::write(source_dir.join("Legal.Movie.2026.mp4"), &payload)
        .await
        .unwrap();
    let torrent = librqbit::create_torrent(
        &source_dir,
        CreateTorrentOptions {
            name: Some("legal-fixture"),
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
    let seed = session(source_dir.clone()).await;
    let seeder = seed
        .add_torrent(
            AddTorrent::from_bytes(torrent.clone()),
            Some(AddTorrentOptions {
                output_folder: Some(source_dir.to_string_lossy().into_owned()),
                overwrite: true,
                paused: true,
                ..Default::default()
            }),
        )
        .await
        .unwrap()
        .into_handle()
        .unwrap();
    seeder.wait_until_initialized().await.unwrap();
    let leecher = session(dir.path().join("downloads")).await;
    let handle = leecher
        .add_torrent(
            AddTorrent::from_bytes(torrent),
            Some(AddTorrentOptions {
                initial_peers: Some(vec![seed.listen_addr().unwrap()]),
                ratelimits: librqbit::limits::LimitsConfig {
                    download_bps: std::num::NonZeroU32::new(128 * 1024),
                    upload_bps: None,
                },
                ..Default::default()
            }),
        )
        .await
        .unwrap()
        .into_handle()
        .unwrap();
    handle.wait_until_initialized().await.unwrap();
    a.acquisition.use_test_engine(leecher.clone()).await;
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
    let data = json!({"input":{},"files":[{"index":0,"path":"Legal.Movie.2026.mp4","size":payload.len()}],"stats":{}});
    sqlx::query("INSERT INTO acquisition_downloads(id,title_id,infohash,data,state) VALUES('download','tmdb:movie:1',?,?,'downloading')").bind(handle.info_hash().as_string()).bind(data.to_string()).execute(&a.pool).await.unwrap();
    sqlx::query("INSERT INTO acquisition_config VALUES(1,?)").bind(json!({"enabled":true,"download_dir":dir.path().join("downloads"),"auto_identify":false}).to_string()).execute(&a.pool).await.unwrap();
    super::super::downloading::sync(&a).await.unwrap();
    let items = crate::catalog::catalog(&a, &headers).await.unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["download"]["playable"], true);
    assert_eq!(items[0]["file_available"], false);
    let id = items[0]["id"].as_str().unwrap();
    let source = for_item(&a, id).await.unwrap().unwrap();
    let head = source
        .response(
            a.clone(),
            Request::builder()
                .method("HEAD")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        head.headers()[header::CONTENT_LENGTH],
        payload.len().to_string()
    );
    assert_eq!(a.acquisition.streams.available_permits(), 8);
    let response = crate::catalog::stream(
        State(a.clone()),
        headers.clone(),
        Path(id.to_owned()),
        Request::builder()
            .header(header::RANGE, "bytes=-32768")
            .body(Body::empty())
            .unwrap(),
    )
    .await
    .unwrap();
    assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);
    let body = axum::body::to_bytes(response.into_body(), 32768);
    tokio::pin!(body);
    assert!(
        tokio::time::timeout(Duration::from_millis(150), &mut body)
            .await
            .is_err(),
        "Unverified sparse bytes must wait"
    );
    seed.unpause(&seeder).await.unwrap();
    let received = tokio::time::timeout(Duration::from_secs(30), body)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(received.as_ref(), &payload[payload.len() - 32768..]);
    assert!(
        !handle.stats().finished,
        "Seeking should not wait for the entire torrent"
    );
    let mut readers = Vec::new();
    for _ in 0..MAX_READERS {
        readers.push(
            source
                .response(a.clone(), Request::new(Body::empty()))
                .await
                .unwrap(),
        );
    }
    let rejected = source
        .response(a.clone(), Request::new(Body::empty()))
        .await
        .unwrap_err();
    assert_eq!(rejected.0, StatusCode::TOO_MANY_REQUESTS);
    let second = librqbit::create_torrent(
        &source_dir,
        CreateTorrentOptions {
            name: Some("second-legal-fixture"),
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
    let addition = tokio::time::timeout(
        Duration::from_secs(5),
        leecher.add_torrent(
            AddTorrent::from_bytes(second),
            Some(AddTorrentOptions {
                paused: true,
                output_folder: Some(dir.path().join("second").to_string_lossy().into_owned()),
                ..Default::default()
            }),
        ),
    )
    .await
    .unwrap()
    .unwrap()
    .into_handle()
    .unwrap();
    tokio::time::timeout(Duration::from_secs(5), addition.wait_until_initialized())
        .await
        .unwrap()
        .unwrap();
    drop(readers);
    assert_eq!(a.acquisition.streams.available_permits(), MAX_READERS);
    sqlx::query("UPDATE acquisition_downloads SET state='paused' WHERE id='download'")
        .execute(&a.pool)
        .await
        .unwrap();
    assert!(
        for_item(&a, id).await.is_err(),
        "Play must never resume paused intent"
    );
    let outsider = crate::tests::account(&a, "viewer", false).await;
    sqlx::query("UPDATE user_access SET all_libraries=0 WHERE user_id=(SELECT id FROM users WHERE username='viewer')").execute(&a.pool).await.unwrap();
    assert!(
        crate::catalog::stream(
            State(a.clone()),
            outsider,
            Path(id.to_owned()),
            Request::new(Body::empty())
        )
        .await
        .is_err()
    );
    a.acquisition.shutdown().await;
    seed.stop().await;
    a.playback.shutdown().await;
}

#[test]
fn ranges_support_seeking_suffixes_and_clamping() {
    for (raw, expected) in [
        (None, (0, 99, false)),
        (Some("bytes=10-19"), (10, 19, true)),
        (Some("bytes=90-"), (90, 99, true)),
        (Some("bytes=-10"), (90, 99, true)),
        (Some("bytes=-999"), (0, 99, true)),
        (Some("bytes=90-999"), (90, 99, true)),
    ] {
        assert_eq!(byte_range(raw, 100).unwrap(), expected);
    }
}

#[test]
fn ranges_reject_invalid_and_multiple_requests() {
    for raw in [
        "bytes=100-",
        "bytes=9-2",
        "bytes=-0",
        "bytes=0-1,3-4",
        "bytes=",
        "bytes=+1-2",
        "items=0-1",
    ] {
        assert!(byte_range(Some(raw), 100).is_err(), "{raw}");
    }
    assert!(byte_range(None, 0).is_err());
}

#[tokio::test]
async fn cancelling_a_waiting_reader_releases_its_permit() {
    let dir = tempfile::tempdir().unwrap();
    let a = app(dir.path()).await;
    let (reader, _writer) = tokio::io::duplex(64);
    let (cancel, rx) = tokio::sync::watch::channel(false);
    let permit = a.acquisition.streams.clone().try_acquire_owned().unwrap();
    let state = ReadState {
        reader: Box::new(reader),
        remaining: 1,
        a: a.clone(),
        source: Source {
            download: "fixture".into(),
            index: 0,
            size: 1,
            mime: "video/mp4",
            title: "fixture".into(),
            hash: "fixture".into(),
        },
        _permit: permit,
        cancel: Some(rx),
        checked: std::time::Instant::now(),
    };
    let task = tokio::spawn(async move {
        let mut state = state;
        state.read(&mut [0u8; 1]).await
    });
    tokio::task::yield_now().await;
    cancel.send(true).unwrap();
    let result = tokio::time::timeout(Duration::from_secs(1), task)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(result.unwrap_err().kind(), ErrorKind::Interrupted);
    assert_eq!(a.acquisition.streams.available_permits(), MAX_READERS);
    a.playback.shutdown().await;
}
