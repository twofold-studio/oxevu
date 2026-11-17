use super::super::*;
use axum::{extract::State as HttpState, routing::get as http_get};
use librqbit::{
    AddTorrent, AddTorrentOptions, CreateTorrentOptions, ListenerMode, ListenerOptions, Session,
    SessionOptions, create_torrent, spawn_utils::BlockingSpawner,
};

#[derive(Clone)]
struct Feed {
    bytes: Arc<Vec<u8>>,
    port: u16,
    size: u64,
    magnet: String,
}
pub(super) struct Fixture {
    pub a: App,
    pub directory: tempfile::TempDir,
    pub endpoint: String,
    pub title: Value,
    pub source: PathBuf,
    pub root: PathBuf,
    pub seed: Arc<Session>,
    tracker: tokio::task::JoinHandle<()>,
}
impl Fixture {
    pub async fn new() -> Self {
        let directory = tempfile::Builder::new()
            .prefix("oxevu-acquisition-test-")
            .tempdir()
            .unwrap();
        let base = directory.path();
        let data = base.join("app");
        tokio::fs::create_dir_all(&data).await.unwrap();
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(
                sqlx::sqlite::SqliteConnectOptions::new()
                    .filename(data.join("oxevu.sqlite"))
                    .create_if_missing(true)
                    .foreign_keys(true),
            )
            .await
            .unwrap();
        oxevu_db::MIGRATOR.run(&pool).await.unwrap();
        let a = App {
            playback: playback::Runtime::new(&data, &pool).await.unwrap(),
            acquisition: Runtime::new(data.clone()),
            pool,
            token: "fixture".into(),
            origin: "http://localhost:3000".into(),
            allowed_origins: Vec::new(),
            scan: Arc::new(Semaphore::new(1)),
            artwork_jobs: Arc::new(Semaphore::new(1)),
            artwork_cache: data.join("artwork"),
            parties: Arc::new(parties::Store::default()),
        };
        let root = base.join("library");
        let source = base.join("source");
        tokio::fs::create_dir(&root).await.unwrap();
        tokio::fs::create_dir(&source).await.unwrap();
        let status = tokio::process::Command::new("ffmpeg")
            .args([
                "-v",
                "error",
                "-f",
                "lavfi",
                "-i",
                "color=c=blue:s=1280x720:r=10",
                "-t",
                "3",
                "-c:v",
                "libx264",
                "-preset",
                "ultrafast",
                "-pix_fmt",
                "yuv420p",
            ])
            .arg(source.join("Fixture Film (2026).mp4"))
            .status()
            .await
            .unwrap();
        assert!(status.success());
        let seed = Session::new_with_opts(
            source.clone(),
            SessionOptions {
                dht: None,
                bind_device_name: Some("lo".into()),
                disable_local_service_discovery: true,
                listen: Some(ListenerOptions {
                    listen_addr: ([127, 0, 0, 1], 0).into(),
                    mode: ListenerMode::TcpOnly,
                    enable_upnp_port_forwarding: false,
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint = format!("http://{}", listener.local_addr().unwrap());
        let torrent = create_torrent(
            &source,
            CreateTorrentOptions {
                name: Some("fixture-film"),
                piece_length: Some(16384),
                trackers: vec![format!("{endpoint}/announce")],
            },
            &BlockingSpawner::new(2),
        )
        .await
        .unwrap()
        .as_bytes()
        .unwrap()
        .to_vec();
        seed.add_torrent(
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
        let feed = Feed {
            magnet: format!(
                "magnet:?xt=urn:btih:{}&tr={endpoint}/announce",
                torrent_files::validate(&torrent, 1024 * 1024 * 1024)
                    .unwrap()
                    .0
            ),
            bytes: Arc::new(torrent),
            port: seed.listen_addr().unwrap().port(),
            size: tokio::fs::metadata(source.join("Fixture Film (2026).mp4"))
                .await
                .unwrap()
                .len(),
        };
        let router = Router::new()
            .route("/api", http_get(api))
            .route("/torrent", http_get(torrent_file))
            .route(
                "/magnet",
                http_get(|HttpState(feed): HttpState<Feed>| async move {
                    (StatusCode::MOVED_PERMANENTLY, [("location", feed.magnet)])
                }),
            )
            .route("/announce", http_get(announce))
            .with_state(feed);
        let tracker = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
        let title = json!({"id":"tmdb:movie:1000","kind":"movie","provider_id":1000,"title":"Fixture Film","year":"2026","release_date":"2026-01-01","overview":"Generated legal fixture"});
        sqlx::query("INSERT INTO acquisition_titles(id,kind,provider_id,data) VALUES('tmdb:movie:1000','movie',1000,?)").bind(title.to_string()).execute(&a.pool).await.unwrap();
        sqlx::query("INSERT INTO libraries VALUES('fixture-library','Fixture movies',?,'movies')")
            .bind(root.to_string_lossy().as_ref())
            .execute(&a.pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO library_roots VALUES('fixture-library',?)")
            .bind(root.to_string_lossy().as_ref())
            .execute(&a.pool)
            .await
            .unwrap();
        Self {
            a,
            directory,
            endpoint,
            title,
            source,
            root,
            seed,
            tracker,
        }
    }
    pub async fn close(self) {
        self.a.acquisition.shutdown().await;
        self.a.playback.shutdown().await;
        self.seed.stop().await;
        self.tracker.abort();
        let _ = self.tracker.await;
        self.a.pool.close().await;
        drop(self.a);
        drop(self.seed);
        self.directory.close().unwrap();
    }
}
async fn api(
    HttpState(f): HttpState<Feed>,
    Query(q): Query<std::collections::HashMap<String, String>>,
) -> String {
    if q.get("t").is_some_and(|t| t == "caps") {
        return "<caps><limits max=\"100\"/><searching><search available=\"yes\" supportedParams=\"q\"/><movie-search available=\"yes\" supportedParams=\"q\"/></searching></caps>".into();
    }
    format!(
        "<rss><channel><item><title>Fixture Film (2026) 720p WEB-DL x264</title><guid>fixture-film</guid><enclosure url=\"/torrent?passkey=fixture-secret\" length=\"{}\"/><attr name=\"seeders\" value=\"10\"/></item></channel></rss>",
        f.size
    )
}
async fn torrent_file(HttpState(f): HttpState<Feed>) -> Vec<u8> {
    f.bytes.as_ref().clone()
}
async fn announce(HttpState(f): HttpState<Feed>) -> Vec<u8> {
    let mut bytes = b"d8:intervali1e5:peers6:".to_vec();
    bytes.extend_from_slice(&[127, 0, 0, 1]);
    bytes.extend_from_slice(&f.port.to_be_bytes());
    bytes.push(b'e');
    bytes
}
