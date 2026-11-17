use super::super::*;
use super::fixture::Fixture;
use base64::{Engine, engine::general_purpose::STANDARD};
use librqbit::{
    AddTorrent, AddTorrentOptions, CreateTorrentOptions, create_torrent,
    spawn_utils::BlockingSpawner,
};

#[tokio::test]
async fn entry_limits_require_permission_validate_and_survive_reloading() {
    let a = crate::tests::app().await;
    let admin = crate::tests::account(&a, "admin", true).await;
    let viewer = crate::tests::account(&a, "viewer", false).await;
    sqlx::query("INSERT INTO acquisition_downloads(id,data,state) VALUES('limited','{\"input\":{},\"files\":[],\"stats\":{}}','paused')")
        .execute(&a.pool).await.unwrap();
    assert_eq!(
        set(&a, &viewer, "limited", 128).await.unwrap_err().0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        set(&a, &HeaderMap::new(), "limited", 128)
            .await
            .unwrap_err()
            .0,
        StatusCode::UNAUTHORIZED
    );
    for bad in [1, 15, 1_000_001] {
        assert!(set(&a, &admin, "limited", bad).await.is_err());
    }
    for limit in [16, 128, 0] {
        let response = set(&a, &admin, "limited", limit).await.unwrap().0;
        assert_eq!(response["download"]["download_limit_kib"], limit);
        assert_eq!(response["download"]["state"], "paused");
        let raw: String =
            sqlx::query_scalar("SELECT data FROM acquisition_downloads WHERE id='limited'")
                .fetch_one(&a.pool)
                .await
                .unwrap();
        assert_eq!(
            download_limits::engine_limits(&decode(&raw).unwrap())
                .download_bps
                .map(|n| n.get())
                .unwrap_or(0),
            limit * 1024
        );
    }
    assert!(set(&a, &admin, "missing", 128).await.is_err());
}

async fn set(a: &App, h: &HeaderMap, id: &str, limit: u32) -> Result<Json<Value>> {
    download_limits::put(
        State(a.clone()),
        h.clone(),
        Path(id.into()),
        Json(download_limits::Input {
            download_limit_kib: limit,
        }),
    )
    .await
}

#[tokio::test]
async fn real_transfers_obey_entry_and_combined_caps_and_reconnect_without_losing_files() {
    let f = Fixture::new().await;
    let a = &f.a;
    let admin = crate::tests::account(a, "admin", true).await;
    let config = config::Config {
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
        download_limit_kib: 64,
        ..Default::default()
    };
    let _ = config::put(State(a.clone()), admin.clone(), Json(config))
        .await
        .unwrap();
    let mut ids = Vec::new();
    for n in 0..2u8 {
        let source = f.directory.path().join(format!("rate-source-{n}"));
        tokio::fs::create_dir(&source).await.unwrap();
        tokio::fs::write(source.join("legal-fixture.bin"), vec![n; 512 * 1024])
            .await
            .unwrap();
        let bytes = create_torrent(
            &source,
            CreateTorrentOptions {
                name: Some(if n == 0 { "speed-one" } else { "speed-two" }),
                piece_length: Some(16384),
                trackers: vec![format!("{}/announce", f.endpoint)],
            },
            &BlockingSpawner::new(2),
        )
        .await
        .unwrap()
        .as_bytes()
        .unwrap()
        .to_vec();
        f.seed
            .add_torrent(
                AddTorrent::from_bytes(bytes.clone()),
                Some(AddTorrentOptions {
                    output_folder: Some(source.to_string_lossy().into_owned()),
                    overwrite: true,
                    ..Default::default()
                }),
            )
            .await
            .unwrap()
            .into_handle()
            .unwrap()
            .wait_until_completed()
            .await
            .unwrap();
        let id = downloads::enqueue(
            a,
            downloads::Input {
                torrent_base64: Some(STANDARD.encode(bytes)),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        let _ = set(a, &admin, &id, if n == 0 { 16 } else { 128 })
            .await
            .unwrap();
        ids.push(id);
    }
    // Start both torrents, then sample actual verified network bytes. Initial
    // token-bucket bursts are permitted, so these bounds allow one extra second.
    jobs::tick(a).await.unwrap();
    let start = std::time::Instant::now();
    while start.elapsed().as_secs() < 5 {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        jobs::tick(a).await.unwrap();
    }
    let session = a.acquisition.current().await.unwrap();
    let mut counts = Vec::new();
    for id in &ids {
        let view = downloads::view(a, id).await.unwrap();
        let handle = downloads::find(&session, view["infohash"].as_str()).unwrap();
        counts.push(handle.stats().progress_bytes);
    }
    assert!(
        counts[0] > 0 && counts[1] > 0,
        "both peers transferred: {counts:?}"
    );
    assert!(counts[0] <= 128 * 1024, "entry cap: {counts:?}");
    assert!(
        counts.iter().sum::<u64>() <= 448 * 1024,
        "combined cap: {counts:?}"
    );
    let old = downloads::view(a, &ids[0]).await.unwrap();
    let _ = set(a, &admin, &ids[0], 256).await.unwrap();
    assert_eq!(
        downloads::view(a, &ids[0]).await.unwrap()["files"],
        old["files"]
    );
    tokio::time::timeout(std::time::Duration::from_secs(35), async {
        loop {
            jobs::tick(a).await.unwrap();
            if downloads::view(a, &ids[0]).await.unwrap()["stats"]["finished"] == true
                && downloads::view(a, &ids[1]).await.unwrap()["stats"]["finished"] == true
            {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    })
    .await
    .unwrap();
    let view = downloads::view(a, &ids[0]).await.unwrap();
    assert_eq!(view["download_limit_kib"], 256);
    assert_eq!(
        view["import_blocker"],
        "Recognizing this torrent before automatic import."
    );
    for (n, id) in ids.iter().enumerate() {
        assert_eq!(
            tokio::fs::read(
                f.directory
                    .path()
                    .join("downloads")
                    .join(id)
                    .join("legal-fixture.bin")
            )
            .await
            .unwrap(),
            vec![n as u8; 512 * 1024]
        );
    }
    f.close().await;
}
