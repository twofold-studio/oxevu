//! Request scope and release recognition rules, without HTTP, storage or jobs.
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
pub mod release_facts;
pub mod release_identity;
pub mod scope;
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct Error(String);
type Result<T> = std::result::Result<T, Error>;
fn bad(message: impl ToString) -> Error {
    Error(message.to_string())
}
