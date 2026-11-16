use axum::{
    Json, Router,
    extract::{Path, Request, State},
    http::{HeaderMap, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post, put},
};
use serde_json::{Value, json};
use sqlx::{Row, SqlitePool};
use std::{path::PathBuf, sync::Arc};
use tokio::sync::Semaphore;
mod acquisition;
mod artwork;
mod assets;
mod auth;
mod catalog;
mod episode_title;
mod integrations;
mod libraries;
mod library_paths;
mod media_files;
mod media_probe;
mod parties;
mod playback;
mod scan;
mod settings;
mod subtitles;
mod users;
use auth::*;
use catalog::*;
use libraries::*;
type Error = (StatusCode, Json<Value>);
type Result<T> = std::result::Result<T, Error>;
fn bad(s: impl ToString) -> Error {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({"error":s.to_string()})),
    )
}
fn db(e: sqlx::Error) -> Error {
    eprintln!("database operation failed: {e}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"error":"database operation failed"})),
    )
}
#[derive(Clone)]
struct App {
    pool: SqlitePool,
    token: String,
    origin: String,
    scan: Arc<Semaphore>,
    artwork_jobs: Arc<Semaphore>,
    artwork_cache: PathBuf,
    playback: Arc<playback::Runtime>,
    parties: Arc<parties::Store>,
    acquisition: Arc<acquisition::Runtime>,
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let dir = std::env::var_os("OXEVU_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".local/share/oxevu")
        });
    tokio::fs::create_dir_all(&dir).await?;
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(dir.join("oxevu.sqlite"))
                .create_if_missing(true)
                .foreign_keys(true),
        )
        .await?;
    sqlx::migrate!().run(&pool).await?;
    use std::os::unix::fs::PermissionsExt;
    tokio::fs::set_permissions(
        dir.join("oxevu.sqlite"),
        std::fs::Permissions::from_mode(0o600),
    )
    .await?;
    let token = uuid::Uuid::new_v4().to_string();
    use std::os::unix::fs::OpenOptionsExt;
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(dir.join("setup-token"))?;
    use std::io::Write;
    f.write_all(token.as_bytes())?;
    let bind = std::env::var("OXEVU_BIND").unwrap_or_else(|_| "127.0.0.1:3000".into());
    let address: std::net::SocketAddr = bind.parse()?;
    let users: i64 = sqlx::query_scalar("SELECT count(*) FROM users")
        .fetch_one(&pool)
        .await?;
    anyhow::ensure!(
        users > 0 || address.ip().is_loopback(),
        "initial setup requires a loopback OXEVU_BIND"
    );
    let origin = std::env::var("OXEVU_ORIGIN").unwrap_or_else(|_| "http://localhost:3000".into());
    let playback = playback::Runtime::new(&dir, &pool)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e.1.0))?;
    let app = App {
        acquisition: acquisition::Runtime::new(dir.clone()),
        pool,
        parties: Arc::new(parties::Store::default()),
        playback,
        token,
        origin,
        scan: Arc::new(Semaphore::new(1)),
        artwork_jobs: Arc::new(Semaphore::new(1)),
        artwork_cache: dir.join("artwork"),
    };
    let reaper = tokio::spawn(playback::reap(app.clone()));
    let party_reaper = tokio::spawn(parties::reap(app.clone()));
    let acquisition_worker = tokio::spawn(acquisition::run(app.clone()));
    let monitor_worker = tokio::spawn(acquisition::monitor(app.clone()));
    let import_worker = tokio::spawn(acquisition::import_worker(app.clone()));
    let shutdown_runtime = app.playback.clone();
    let acquisition_runtime = app.acquisition.clone();
    let api = Router::new()
        .merge(acquisition::routes())
        .merge(parties::routes())
        .route("/playback/capabilities", get(playback::capabilities))
        .route("/playback/sessions", post(playback::create))
        .route("/playback/sessions/{id}", get(playback::get))
        .route(
            "/playback/sessions/{id}/heartbeat",
            post(playback::heartbeat),
        )
        .route("/playback/sessions/{id}/stop", post(playback::stop))
        .route("/playback/sessions/{id}/hls/{file}", get(playback::hls))
        .route("/users", get(users::list).post(users::create))
        .route("/users/{id}", put(users::update))
        .route("/auth/password", put(auth::password))
        .route("/libraries/{id}", put(libraries::update))
        .route("/libraries/paths/suggest", post(library_paths::suggest))
        .route("/libraries/{id}/remove", post(libraries::remove))
        .route("/settings", get(settings::get).put(settings::put))
        .route(
            "/integrations",
            get(integrations::get).put(integrations::put),
        )
        .route("/items/{id}/metadata/search", get(integrations::search))
        .route("/items/{id}/metadata/match", post(integrations::apply))
        .route(
            "/items/{id}/metadata/artwork/{kind}",
            get(integrations::metadata_artwork),
        )
        .route(
            "/items/{id}/metadata/studios/{name}",
            get(integrations::studio),
        )
        .route(
            "/items/{id}/subtitles/search",
            get(integrations::subtitle_search),
        )
        .route(
            "/items/{id}/subtitles/download",
            post(integrations::subtitle_download),
        )
        .route("/auth/status", get(status))
        .route("/auth/setup", post(setup))
        .route("/auth/login", post(login))
        .route("/auth/logout", post(logout))
        .route("/libraries", get(libraries).post(add_library))
        .route("/libraries/{id}/scan", post(scan::scan))
        .route("/items", get(items))
        .route("/items/{id}", get(item))
        .route("/items/{id}/favorite", put(catalog::favorite))
        .route("/items/{id}/reaction", put(media_files::reaction))
        .route("/library/removals/preview", post(media_files::preview))
        .route("/library/removals", post(media_files::apply))
        .route("/items/{id}/stream", get(stream))
        .route("/items/{id}/subtitles", get(subtitles::list))
        .route("/items/{id}/subtitles/{track}", get(subtitles::serve))
        .route("/items/{id}/artwork/{kind}", get(artwork::serve))
        .route("/items/{id}/progress", put(progress));
    let router = Router::new()
        .nest("/api/v1", api)
        .route("/healthz", get(|| async { "ok" }))
        .fallback(assets::serve)
        .layer(middleware::from_fn_with_state(app.clone(), origin_check))
        .with_state(app);
    println!("Oxevu listening on {bind}");
    let signal_runtime = shutdown_runtime.clone();
    axum::serve(tokio::net::TcpListener::bind(bind).await?, router)
        .with_graceful_shutdown(async move {
            #[cfg(unix)]
            if let Ok(mut terminate) =
                tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            {
                tokio::select! { _=tokio::signal::ctrl_c()=>{}, _=terminate.recv()=>{} }
            } else {
                let _ = tokio::signal::ctrl_c().await;
            }
            #[cfg(not(unix))]
            let _ = tokio::signal::ctrl_c().await;
            signal_runtime.shutdown().await;
            acquisition_runtime.shutdown().await;
        })
        .await?;
    reaper.abort();
    party_reaper.abort();
    acquisition_worker.abort();
    monitor_worker.abort();
    import_worker.abort();
    let _ = import_worker.await;
    let _ = monitor_worker.await;
    let _ = acquisition_worker.await;
    shutdown_runtime.shutdown().await;
    Ok(())
}
async fn origin_check(State(a): State<App>, r: Request, n: Next) -> Response {
    if !matches!(
        *r.method(),
        axum::http::Method::GET | axum::http::Method::HEAD
    ) && r.headers().get("origin").and_then(|v| v.to_str().ok()) != Some(&a.origin)
    {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error":"same-origin Origin header required"})),
        )
            .into_response();
    }
    n.run(r).await
}

#[cfg(test)]
mod tests;
