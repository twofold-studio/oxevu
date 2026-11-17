//! HTTP error mapping for the independent library services.
pub(crate) use oxevu_library::{Error, catalog, identification, roots, scanner};
pub(crate) fn response(error: Error) -> crate::Error {
    match error {
        Error::Storage(error) => crate::db(error),
        Error::Invalid(message) => crate::bad(message),
    }
}
