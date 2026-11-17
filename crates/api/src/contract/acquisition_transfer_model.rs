use super::acquisition_catalog_model::DiscoverTitle;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct ImportDestination {
    library_id: String,
    root: String,
    root_device: Option<u64>,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct ImportDefaults {
    #[schema(required)]
    movies: Option<ImportDestination>,
    #[schema(required)]
    tv: Option<ImportDestination>,
}
#[derive(Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum RecognitionState {
    Pending,
    Matched,
    NeedsReview,
    Unavailable,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct Recognition {
    state: RecognitionState,
    message: Option<String>,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct ReleaseSource {
    indexer_id: String,
    indexer_name: String,
    hits: Option<u64>,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct Release {
    id: String,
    title: String,
    size: Option<u64>,
    seeders: Option<u64>,
    height: Option<u32>,
    codec: Option<String>,
    source: Option<String>,
    season: Option<u32>,
    episode: Option<u32>,
    indexer_id: String,
    indexer_name: String,
    accepted: bool,
    reasons: Vec<String>,
    identity_match: bool,
    blocked: Option<bool>,
    infohash: Option<String>,
    also_on: Option<Vec<ReleaseSource>>,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct IndexerError {
    indexer_id: String,
    indexer_name: String,
    error: String,
    retryable: bool,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct Searched {
    indexers: u32,
    failed: u32,
    took_ms: u64,
    hits: u64,
    sources: Vec<ReleaseSource>,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct ReleaseResults {
    releases: Vec<Release>,
    errors: Vec<IndexerError>,
    searched: Searched,
    offset: u32,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct TorrentFile {
    index: u32,
    path: String,
    size: u64,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct Speed {
    mbps: f64,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct PeerSnapshot {
    peer_stats: Option<std::collections::BTreeMap<String, u64>>,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct LiveStats {
    download_speed: Speed,
    upload_speed: Speed,
    snapshot: Option<PeerSnapshot>,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct TorrentStats {
    state: Option<String>,
    progress_bytes: Option<u64>,
    total_bytes: Option<u64>,
    uploaded_bytes: Option<u64>,
    finished: Option<bool>,
    file_progress: Option<Vec<u64>>,
    live: Option<LiveStats>,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct Download {
    id: String,
    #[schema(required)]
    title_id: Option<String>,
    #[schema(required)]
    title: Option<DiscoverTitle>,
    #[schema(required)]
    infohash: Option<String>,
    state: String,
    #[schema(required)]
    error: Option<String>,
    stats: TorrentStats,
    files: Vec<TorrentFile>,
    #[schema(required)]
    selected_files: Option<Vec<u32>>,
    seed_seconds: u64,
    created_at: i64,
    import_warning: Option<String>,
    queue_reason: Option<String>,
    release_title: Option<String>,
    indexer_name: Option<String>,
    release_size: Option<u64>,
    imported: Option<bool>,
    import_blocker: Option<String>,
    download_limit_kib: Option<u32>,
    recognition: Option<Recognition>,
    import_destination: Option<ImportDestination>,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct DownloadEnvelope {
    download: Download,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct DownloadsEnvelope {
    downloads: Vec<Download>,
    import_enabled: bool,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct ReleaseSources {
    sources: Vec<ReleaseSource>,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct IndexerCapabilities {
    search: bool,
    #[serde(rename = "tv-search")]
    tv_search: bool,
    #[serde(rename = "movie-search")]
    movie_search: bool,
    max_limit: u32,
    categories: Vec<String>,
    search_params: Option<String>,
    #[serde(rename = "tv-search_params")]
    tv_search_params: Option<String>,
    #[serde(rename = "movie-search_params")]
    movie_search_params: Option<String>,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct Indexer {
    id: String,
    name: String,
    endpoint: String,
    enabled: bool,
    api_key_set: bool,
    download_hosts: Vec<String>,
    #[schema(required)]
    capabilities: Option<IndexerCapabilities>,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct IndexerEnvelope {
    indexer: Indexer,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct IndexersEnvelope {
    indexers: Vec<Indexer>,
}
