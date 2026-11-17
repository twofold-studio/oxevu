//! Failures starting or serving the application, independent of HTTP responses.

/// A startup or listener failure reported by the executable.
#[derive(Debug, thiserror::Error)]
pub enum StartupError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Storage(#[from] oxevu_db::Error),
    #[error(transparent)]
    InvalidAddress(#[from] std::net::AddrParseError),
    #[error("initial setup requires a loopback OXEVU_BIND")]
    SetupRequiresLoopback,
    #[error("{0}")]
    Playback(String),
}
