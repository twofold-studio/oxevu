use super::*;

async fn fixture() -> (App, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!().run(&pool).await.unwrap();
    let playback = playback::Runtime::new(dir.path(), &pool).await.unwrap();
    let a = App {
        pool,
        playback,
        acquisition: Runtime::new(dir.path().to_owned()),
        parties: Arc::new(parties::Store::default()),
        token: "test".into(),
        origin: "http://localhost:3000".into(),
        scan: Arc::new(Semaphore::new(1)),
        artwork_jobs: Arc::new(Semaphore::new(1)),
        artwork_cache: dir.path().join("artwork"),
    };
    (a, dir)
}

async fn library(a: &App, id: &str, kind: &str, path: &std::path::Path) {
    tokio::fs::create_dir_all(path).await.unwrap();
    sqlx::query("INSERT INTO libraries VALUES(?,?,?,?)")
        .bind(id)
        .bind(id)
        .bind(path.to_str())
        .bind(kind)
        .execute(&a.pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO library_roots(library_id,path) VALUES(?,?)")
        .bind(id)
        .bind(path.to_str())
        .execute(&a.pool)
        .await
        .unwrap();
}

#[tokio::test]
async fn automatic_import_default_is_one_enabled_root_per_kind_and_supports_override() {
    let (a, dir) = fixture().await;
    library(
        &a,
        "movies-first",
        "movies",
        &dir.path().join("movies-first"),
    )
    .await;
    library(
        &a,
        "movies-second",
        "movies",
        &dir.path().join("movies-second"),
    )
    .await;
    library(&a, "series", "tv", &dir.path().join("series")).await;
    assert_eq!(
        destinations::resolve(&a, "movies")
            .await
            .unwrap()
            .unwrap()
            .library_id,
        "movies-first"
    );
    assert_eq!(
        destinations::resolve(&a, "tv")
            .await
            .unwrap()
            .unwrap()
            .library_id,
        "series"
    );
    let mut c = config::Config::default();
    c.movie_destination = Some(destinations::Destination {
        library_id: "movies-second".into(),
        root: dir
            .path()
            .join("movies-second")
            .to_string_lossy()
            .into_owned(),
        root_device: None,
    });
    destinations::validate(&a, "movies", c.movie_destination.as_mut().unwrap())
        .await
        .unwrap();
    assert!(
        destinations::validate(&a, "tv", c.movie_destination.as_mut().unwrap())
            .await
            .is_err()
    );
    sqlx::query("INSERT INTO acquisition_config VALUES(1,?)")
        .bind(serde_json::to_string(&c).unwrap())
        .execute(&a.pool)
        .await
        .unwrap();
    assert_eq!(
        destinations::resolve(&a, "movies")
            .await
            .unwrap()
            .unwrap()
            .library_id,
        "movies-second"
    );
    sqlx::query("INSERT INTO library_options(library_id,enabled) VALUES('movies-second',0)")
        .execute(&a.pool)
        .await
        .unwrap();
    assert!(destinations::resolve(&a, "movies").await.is_err());
}

#[tokio::test]
async fn automatic_import_missing_identity_returns_a_useful_error() {
    let (a, _dir) = fixture().await;
    sqlx::query(
        "INSERT INTO acquisition_downloads(id,data,state) VALUES('unmatched','{}','seeding')",
    )
    .execute(&a.pool)
    .await
    .unwrap();
    let input = import_plan::Input {
        library_id: "movies".into(),
        root: "/movies".into(),
        mappings: vec![import_plan::Mapping {
            file_index: 0,
            episodes: vec![],
        }],
        token: None,
    };
    let error = import_plan::build(&a, "unmatched", &input)
        .await
        .err()
        .unwrap();
    assert!(
        error.1.0["error"]
            .as_str()
            .unwrap()
            .contains("not been recognized")
    );
}

#[tokio::test]
async fn automatic_import_rejects_a_reidentified_download_before_opening_files() {
    let (a, _dir) = fixture().await;
    let c = config::Config {
        import_enabled: true,
        ..Default::default()
    };
    sqlx::query("INSERT INTO acquisition_config VALUES(1,?)")
        .bind(serde_json::to_string(&c).unwrap())
        .execute(&a.pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO acquisition_titles(id,kind,provider_id,data) VALUES('new-title','movie',1,'{}')").execute(&a.pool).await.unwrap();
    sqlx::query(
        "INSERT INTO acquisition_downloads(id,title_id,data) VALUES('download','new-title','{}')",
    )
    .execute(&a.pool)
    .await
    .unwrap();
    let plan = import_plan::File {
        file_index: 0,
        source: "/does-not-exist".into(),
        destination: "/does-not-exist".into(),
        size: 1,
        coverage: vec![],
        title_id: "old-title".into(),
        root_device: 0,
        root: "/does-not-exist".into(),
        source_stamp: String::new(),
    };
    let error = imports::execute(&a, "download", "movies", vec![plan])
        .await
        .err()
        .unwrap();
    assert!(
        error.1.0["error"]
            .as_str()
            .unwrap()
            .contains("title changed")
    );
}

#[tokio::test]
async fn automatic_import_completed_movie_uses_default_without_monitor_and_keeps_source() {
    let (a, dir) = fixture().await;
    let movies = dir.path().join("movies");
    library(&a, "movies", "movies", &movies).await;
    let download = dir.path().join("downloads").join("download");
    tokio::fs::create_dir_all(&download).await.unwrap();
    let source = download.join("Legal.Test.Movie.2026.mp4");
    let status = tokio::process::Command::new("ffmpeg")
        .args([
            "-nostdin",
            "-v",
            "error",
            "-f",
            "lavfi",
            "-i",
            "color=size=32x32:rate=1",
            "-t",
            "1",
            "-c:v",
            "libx264",
            "-threads",
            "1",
        ])
        .arg(&source)
        .status()
        .await
        .unwrap();
    assert!(status.success());
    let size = tokio::fs::metadata(&source).await.unwrap().len();
    let title = json!({"id":"tmdb:movie:42","kind":"movie","provider_id":42,"title":"Legal Test Movie","year":"2026","rating":7.0});
    sqlx::query("INSERT INTO acquisition_titles(id,kind,provider_id,data) VALUES('tmdb:movie:42','movie',42,?)").bind(title.to_string()).execute(&a.pool).await.unwrap();
    let data = json!({"input":{},"files":[{"index":0,"path":"Legal.Test.Movie.2026.mp4","size":size},{"index":1,"path":"sample.mkv","size":3}],"stats":{"finished":true,"file_progress":[size,3]}});
    sqlx::query("INSERT INTO acquisition_downloads(id,title_id,data,state) VALUES('download','tmdb:movie:42',?,'seeding')").bind(data.to_string()).execute(&a.pool).await.unwrap();
    let c = config::Config {
        enabled: true,
        import_enabled: true,
        auto_identify: false,
        download_dir: dir.path().join("downloads").to_string_lossy().into_owned(),
        min_free_gb: 0,
        ..Default::default()
    };
    sqlx::query("INSERT INTO acquisition_config VALUES(1,?)")
        .bind(serde_json::to_string(&c).unwrap())
        .execute(&a.pool)
        .await
        .unwrap();
    downloading::sync(&a).await.unwrap();
    let pending: String = sqlx::query_scalar(
        "SELECT item_id FROM acquisition_stream_items WHERE download_id='download'",
    )
    .fetch_one(&a.pool)
    .await
    .unwrap();
    {
        let h = crate::tests::account(&a, "early-viewer", true).await;
        let u = user(&a, &h).await.unwrap();
        sqlx::query("INSERT INTO progress VALUES(?,?,0.25,0)")
            .bind(u["id"].as_str())
            .bind(&pending)
            .execute(&a.pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO item_reactions VALUES(?,?,'like')")
            .bind(u["id"].as_str())
            .bind(&pending)
            .execute(&a.pool)
            .await
            .unwrap();
    }
    auto_import::tick(&a).await.unwrap();
    let state: Option<String> =
        sqlx::query_scalar("SELECT state FROM acquisition_imports WHERE download_id='download'")
            .fetch_optional(&a.pool)
            .await
            .unwrap();
    assert_eq!(state.as_deref(), Some("scanned"));
    let destination = movies.join("Legal Test Movie (2026)/Legal Test Movie (2026).mp4");
    assert_eq!(
        tokio::fs::read(&source).await.unwrap(),
        tokio::fs::read(destination).await.unwrap()
    );
    auto_import::tick(&a).await.unwrap();
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM acquisition_imports")
        .fetch_one(&a.pool)
        .await
        .unwrap();
    assert_eq!(count, 1);
    let item: String =
        sqlx::query_scalar("SELECT item_id FROM acquisition_imports WHERE download_id='download'")
            .fetch_one(&a.pool)
            .await
            .unwrap();
    let headers = crate::tests::account(&a, "removal-owner", true).await;
    assert_eq!(
        item, pending,
        "Full import must preserve the downloading identity"
    );
    let position: f64 =
        sqlx::query_scalar("SELECT CAST(position AS REAL) FROM progress WHERE item_id=?")
            .bind(&item)
            .fetch_one(&a.pool)
            .await
            .unwrap();
    assert_eq!(position, 0.25);
    let opinion: String = sqlx::query_scalar("SELECT value FROM item_reactions WHERE item_id=?")
        .bind(&item)
        .fetch_one(&a.pool)
        .await
        .unwrap();
    assert_eq!(opinion, "like");
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM acquisition_stream_items")
            .fetch_one(&a.pool)
            .await
            .unwrap(),
        0
    );
    let input = crate::media_files::Input {
        item_ids: vec![item.clone()],
        keep_metadata: true,
        token: None,
        confirm: false,
    };
    let Json(preview) = crate::media_files::preview(State(a.clone()), headers.clone(), Json(input))
        .await
        .unwrap();
    let input = crate::media_files::Input {
        item_ids: vec![item.clone()],
        keep_metadata: true,
        token: Some(preview["token"].as_str().unwrap().into()),
        confirm: true,
    };
    let Json(removed) = crate::media_files::apply(State(a.clone()), headers, Json(input))
        .await
        .unwrap();
    assert!(removed["failures"].as_array().unwrap().is_empty());
    let destination = movies.join("Legal Test Movie (2026)/Legal Test Movie (2026).mp4");
    assert!(!destination.exists());
    auto_import::tick(&a).await.unwrap();
    assert!(
        !destination.exists(),
        "a completed torrent must not undo an intentional removal"
    );
    sqlx::query("UPDATE acquisition_downloads SET data=json_set(data,'$.reimport_requested',json('true')) WHERE id='download'").execute(&a.pool).await.unwrap();
    auto_import::tick(&a).await.unwrap();
    assert!(destination.exists());
    let restored: String =
        sqlx::query_scalar("SELECT item_id FROM acquisition_imports WHERE download_id='download'")
            .fetch_one(&a.pool)
            .await
            .unwrap();
    assert_eq!(restored, item);
}

#[tokio::test]
async fn automatic_import_series_uses_series_default_and_episode_directories() {
    let (a, dir) = fixture().await;
    let series = dir.path().join("series");
    library(&a, "movies", "movies", &dir.path().join("movies")).await;
    library(&a, "series", "tv", &series).await;
    let download = dir.path().join("downloads/series-download");
    tokio::fs::create_dir_all(&download).await.unwrap();
    let first = download.join("Legal.Series.S01E01.mp4");
    let status = tokio::process::Command::new("ffmpeg")
        .args([
            "-nostdin",
            "-v",
            "error",
            "-f",
            "lavfi",
            "-i",
            "color=size=32x32:rate=1",
            "-t",
            "1",
            "-c:v",
            "libx264",
            "-threads",
            "1",
        ])
        .arg(&first)
        .status()
        .await
        .unwrap();
    assert!(status.success());
    tokio::fs::copy(&first, download.join("Legal.Series.S01E02.mp4"))
        .await
        .unwrap();
    let size = tokio::fs::metadata(&first).await.unwrap().len();
    let title = json!({"id":"tmdb:tv:44","kind":"tv","provider_id":44,"title":"Legal Series","year":"2026"});
    sqlx::query(
        "INSERT INTO acquisition_titles(id,kind,provider_id,data) VALUES('tmdb:tv:44','tv',44,?)",
    )
    .bind(title.to_string())
    .execute(&a.pool)
    .await
    .unwrap();
    for episode in 1..=2 {
        sqlx::query("INSERT INTO acquisition_episodes(title_id,season,episode,data) VALUES('tmdb:tv:44',1,?,?)").bind(episode).bind(json!({"name":format!("Episode {episode}"),"vote_average":7.0}).to_string()).execute(&a.pool).await.unwrap();
    }
    let data = json!({"input":{},"files":[{"index":0,"path":"Legal.Series.S01E01.mp4","size":size},{"index":1,"path":"Legal.Series.S01E02.mp4","size":size}],"stats":{"finished":true,"file_progress":[size,size]}});
    sqlx::query("INSERT INTO acquisition_downloads(id,title_id,data,state) VALUES('series-download','tmdb:tv:44',?,'seeding')").bind(data.to_string()).execute(&a.pool).await.unwrap();
    let c = config::Config {
        enabled: true,
        import_enabled: true,
        auto_identify: false,
        download_dir: dir.path().join("downloads").to_string_lossy().into_owned(),
        min_free_gb: 0,
        ..Default::default()
    };
    sqlx::query("INSERT INTO acquisition_config VALUES(1,?)")
        .bind(serde_json::to_string(&c).unwrap())
        .execute(&a.pool)
        .await
        .unwrap();
    auto_import::tick(&a).await.unwrap();
    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM acquisition_imports WHERE state='scanned'")
            .fetch_one(&a.pool)
            .await
            .unwrap();
    assert_eq!(count, 2);
    for episode in 1..=2 {
        assert!(
            series
                .join(format!(
                    "Legal Series (2026)/Season 01/Legal Series - S01E{episode:02}.mp4"
                ))
                .is_file()
        );
        assert!(
            download
                .join(format!("Legal.Series.S01E{episode:02}.mp4"))
                .is_file()
        );
    }
}
