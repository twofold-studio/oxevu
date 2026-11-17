//! Normalize responses from supported release indexers.
use serde_json::{Value, json};
pub mod native;
pub mod torznab;
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct Error(String);
type Result<T> = std::result::Result<T, Error>;
fn bad(message: impl ToString) -> Error {
    Error(message.to_string())
}
