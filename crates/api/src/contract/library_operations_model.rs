use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct PathInput {
    path: String,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct PathSuggestion {
    name: String,
    path: String,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct PathSuggestions {
    paths: Vec<PathSuggestion>,
    truncated: bool,
}
#[derive(Deserialize, Serialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct RemovalInput {
    item_ids: Vec<String>,
    #[serde(default = "keep_default")]
    keep_metadata: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    token: Option<String>,
    #[serde(default)]
    confirm: bool,
}
fn keep_default() -> bool {
    true
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct RemovalPreviewItem {
    id: String,
    title: String,
    #[schema(required)]
    size_bytes: Option<u64>,
    local_file: bool,
    shared_links: u64,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct RemovalPreview {
    notice: String,
    token: String,
    count: u64,
    keep_metadata: bool,
    bytes_selected: u64,
    shared_files_count: u64,
    items: Vec<RemovalPreviewItem>,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct RemovalFailure {
    id: String,
    message: String,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct RemovalResult {
    removed_ids: Vec<String>,
    deleted_files: u64,
    kept_metadata: bool,
    failures: Vec<RemovalFailure>,
    bytes_freed_estimate: u64,
    shared_files_count: u64,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct ReactionUpdate {
    #[schema(required)]
    value: Option<super::item::Reaction>,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct SubtitleTrack {
    id: String,
    label: String,
    #[schema(required)]
    language: Option<String>,
    codec: String,
    default: bool,
    forced: bool,
    supported: bool,
    #[schema(required)]
    url: Option<String>,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct SubtitleTracks {
    tracks: Vec<SubtitleTrack>,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct SubtitleSearch {
    language: String,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct SubtitleDownload {
    id: String,
    language: String,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct SubtitleResult {
    id: String,
    label: String,
    language: String,
    hearing_impaired: bool,
    download_count: u64,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct SubtitleResults {
    results: Vec<SubtitleResult>,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct SubtitleEnvelope {
    track: SubtitleTrack,
}
