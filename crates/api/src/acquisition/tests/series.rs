use super::super::*;
use super::fixture::Fixture;
use base64::{Engine, engine::general_purpose::STANDARD};
use librqbit::{
    AddTorrent, AddTorrentOptions, CreateTorrentOptions, create_torrent,
    spawn_utils::BlockingSpawner,
};

#[tokio::test]
async fn shared_pack_imports_only_wanted_episodes_and_survives_engine_restart() {
    let f = Fixture::new().await;
    let a = &f.a;
    let admin = crate::tests::account(a, "admin", true).await;
    let first = crate::tests::account(a, "first", false).await;
    let second = crate::tests::account(a, "second", false).await;
    let root = f.directory.path().join("tv");
    let source = f.directory.path().join("series-source");
    tokio::fs::create_dir_all(&root).await.unwrap();
    tokio::fs::create_dir_all(&source).await.unwrap();
    sqlx::query("INSERT INTO libraries VALUES('tv','TV',?,'tv')")
        .bind(root.to_str())
        .execute(&a.pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO library_roots VALUES('tv',?)")
        .bind(root.to_str())
        .execute(&a.pool)
        .await
        .unwrap();
    for user in ["first", "second"] {
        sqlx::query("INSERT INTO library_grants VALUES(?,'tv')")
            .bind(user)
            .execute(&a.pool)
            .await
            .unwrap();
    }
    let title = json!({"id":"tmdb:tv:2000","provider_id":2000,"kind":"tv","title":"Fixture Series","year":"2026"});
    sqlx::query("INSERT INTO acquisition_titles(id,kind,provider_id,data) VALUES('tmdb:tv:2000','tv',2000,?)").bind(title.to_string()).execute(&a.pool).await.unwrap();
    for season in 1..=2 {
        for episode in 1..=2 {
            let file = format!("Fixture.Series.S{season:02}E{episode:02}.mp4");
            tokio::fs::copy(f.source.join("Fixture Film (2026).mp4"), source.join(file))
                .await
                .unwrap();
            sqlx::query(
                "INSERT INTO acquisition_episodes VALUES('tmdb:tv:2000',?,?,?,'2026-01-01')",
            )
            .bind(season)
            .bind(episode)
            .bind(
                json!({"name":format!("Episode {episode}"),"overview":"Generated fixture"})
                    .to_string(),
            )
            .execute(&a.pool)
            .await
            .unwrap();
        }
    }
    let mut requests = Vec::new();
    for (headers, scope) in [
        (
            first,
            scope::Scope {
                mode: "seasons".into(),
                seasons: vec![1],
                ..Default::default()
            },
        ),
        (
            second,
            scope::Scope {
                mode: "episodes".into(),
                episodes: vec![scope::Episode {
                    season: 2,
                    episode: 1,
                }],
                ..Default::default()
            },
        ),
    ] {
        let id = requests::create_for(a, &user(a, &headers).await.unwrap(), &title, scope)
            .await
            .unwrap();
        let _ = requests::update(
            State(a.clone()),
            admin.clone(),
            Path(id.clone()),
            Json(requests::Decision {
                status: "approved".into(),
                note: String::new(),
            }),
        )
        .await
        .unwrap();
        requests.push(id);
    }
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
            import_enabled: true,
            ..Default::default()
        }),
    )
    .await
    .unwrap();
    let _ = monitoring::put(
        State(a.clone()),
        admin.clone(),
        Path("tmdb:tv:2000".into()),
        Json(monitoring::Input {
            enabled: true,
            scope: scope::Scope {
                mode: "requests".into(),
                ..Default::default()
            },
            library_id: Some("tv".into()),
            root: Some(root.to_string_lossy().into_owned()),
            profile_id: "default".into(),
            automatic: false,
        }),
    )
    .await
    .unwrap();
    let torrent = create_torrent(
        &source,
        CreateTorrentOptions {
            trackers: vec![format!("{}/announce", f.endpoint)],
            piece_length: Some(16384),
            ..Default::default()
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
            AddTorrent::from_bytes(torrent.clone()),
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
            title_id: Some("tmdb:tv:2000".into()),
            torrent_base64: Some(STANDARD.encode(torrent)),
            only_wanted: true,
            ..Default::default()
        },
    )
    .await
    .unwrap();
    tokio::time::timeout(std::time::Duration::from_secs(40), async {
        loop {
            jobs::tick(a).await.unwrap();
            auto_import::tick(a).await.unwrap();
            wanted::fulfill(a).await.unwrap();
            let n: i64 = sqlx::query_scalar(
                "SELECT count(*) FROM acquisition_requests WHERE status='fulfilled'",
            )
            .fetch_one(&a.pool)
            .await
            .unwrap();
            if n == 2 {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    })
    .await
    .unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM acquisition_imports WHERE state='scanned'"
        )
        .fetch_one(&a.pool)
        .await
        .unwrap(),
        3
    );
    assert_eq!(
        downloads::view(a, &id).await.unwrap()["selected_files"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    let raw: String =
        sqlx::query_scalar("SELECT data FROM acquisition_titles WHERE id='tmdb:tv:2000'")
            .fetch_one(&a.pool)
            .await
            .unwrap();
    let units = wanted::units(
        a,
        &decode(&raw).unwrap(),
        &wanted::inventory(a, &title).await.unwrap(),
    )
    .await
    .unwrap();
    assert!(
        units
            .iter()
            .any(|u| u["season"] == 2 && u["episode"] == 2 && u["available"] == false)
    );
    a.acquisition.shutdown().await;
    let restarted = App {
        acquisition: Runtime::new(a.acquisition.directory.parent().unwrap().to_path_buf()),
        ..a.clone()
    };
    let a = &restarted;
    tokio::time::timeout(std::time::Duration::from_secs(20), async {
        loop {
            jobs::tick(a).await.unwrap();
            if downloads::view(a, &id).await.unwrap()["stats"]["finished"] == true {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    })
    .await
    .unwrap();
    // Replay publication/scanning after a crash without another copy or import row.
    sqlx::query("UPDATE acquisition_imports SET state='published'")
        .execute(&a.pool)
        .await
        .unwrap();
    auto_import::tick(a).await.unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM acquisition_imports WHERE state='scanned'"
        )
        .fetch_one(&a.pool)
        .await
        .unwrap(),
        3
    );
    restarted.acquisition.shutdown().await;
    f.close().await;
}
