//! Library application services. HTTP adapters stay at the API boundary.
pub mod automatic_metadata;
pub mod catalog;
pub mod identification;
pub mod roots;
pub mod scanner;
mod sidecars;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Storage(#[from] sqlx::Error),
    #[error("{0}")]
    Invalid(String),
}
type Result<T> = std::result::Result<T, Error>;
fn invalid(message: impl ToString) -> Error {
    Error::Invalid(message.to_string())
}
