//! Torrent engine lifecycle, torrent validation and release identity.
use serde_json::{Value, json};
pub mod engine;
pub mod torrent_files;
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct Error(String);
type Result<T> = std::result::Result<T, Error>;
fn bad(message: impl ToString) -> Error {
    Error(message.to_string())
}
