use axum::{
    Json, Router,
    extract::{Path, Request, State},
    http::{HeaderMap, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
};
use serde_json::{Value, json};
use sqlx::{Row, SqlitePool};
use std::{path::PathBuf, sync::Arc};
use tokio::sync::Semaphore;
mod acquisition;
mod artwork;
use oxevu_web as assets;
mod auth;
mod avatars;
mod catalog;
pub mod contract;
mod integrations;
mod libraries;
mod library;
mod library_paths;
mod media_files;
mod media_probe;
mod parties;
mod playback;
mod routes;
mod scan;
mod settings;
mod startup_error;
mod subtitles;
mod users;
use auth::*;
#[cfg(test)]
use catalog::*;
#[cfg(test)]
use libraries::{add_library, libraries};
#[cfg(test)]
use routes::origin_check;
pub use startup_error::StartupError;
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
/// Run the authenticated HTTP service and its supervised background workers.
pub async fn run() -> std::result::Result<(), StartupError> {
    let dir = std::env::var_os("OXEVU_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".local/share/oxevu")
        });
    tokio::fs::create_dir_all(&dir).await?;
    let pool = oxevu_db::open(&dir).await?;
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
    if users == 0 && !address.ip().is_loopback() {
        return Err(StartupError::SetupRequiresLoopback);
    }
    let origin = std::env::var("OXEVU_ORIGIN").unwrap_or_else(|_| "http://localhost:3000".into());
    let playback = playback::Runtime::new(&dir, &pool)
        .await
        .map_err(|e| StartupError::Playback(e.1.0.to_string()))?;
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
    let router = routes::router(app.clone());
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

#[cfg(test)]
mod tests;
