//! Media inspection and filename parsing, independent of HTTP and storage.
pub mod episode_title;
mod probe;
pub use probe::{ProbeError, attach_facts, read};
