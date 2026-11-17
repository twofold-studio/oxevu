//! Bounded indexer XML parsing and credential-preserving HTTP redirect policy.
use serde_json::{Value, json};
pub mod http;
pub mod torznab;
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct Error(String);
type Result<T> = std::result::Result<T, Error>;
fn bad(message: impl ToString) -> Error {
    Error(message.to_string())
}
