use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub(crate) struct Trailer {
    pub youtube_id: Option<String>,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct Genre {
    id: u32,
    name: String,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct Genres {
    genres: Vec<Genre>,
}

/// Foreign export rows are opaque here; migration performs source-specific validation.
#[derive(Deserialize, Serialize, ToSchema)]
#[serde(untagged)]
pub(crate) enum ExternalExport {
    Rows(Vec<std::collections::BTreeMap<String, serde_json::Value>>),
    Results {
        results: Vec<std::collections::BTreeMap<String, serde_json::Value>>,
    },
}

#[derive(Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum MediaKind {
    Movie,
    Tv,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct EpisodeRef {
    season: u32,
    episode: u32,
}
#[derive(Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ScopeMode {
    All,
    Existing,
    Future,
    Seasons,
    Episodes,
    Requests,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct RequestScope {
    mode: ScopeMode,
    #[serde(default)]
    #[schema(required)]
    seasons: Vec<u32>,
    #[serde(default)]
    #[schema(required)]
    episodes: Vec<EpisodeRef>,
    #[schema(required)]
    cutoff: Option<String>,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct AvailableItem {
    id: String,
    #[schema(required)]
    season: Option<u32>,
    #[schema(required)]
    episode: Option<u32>,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct Availability {
    items: Vec<AvailableItem>,
    offline: bool,
    requested: bool,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct CatalogSeason {
    season_number: u32,
    name: String,
    episode_count: u32,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct DiscoverTitle {
    id: String,
    kind: MediaKind,
    provider_id: u64,
    title: String,
    #[schema(required)]
    year: Option<String>,
    overview: Option<String>,
    poster_url: Option<String>,
    rating: Option<f64>,
    release_date: Option<String>,
    availability: Option<Availability>,
    seasons: Option<Vec<CatalogSeason>>,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct CatalogEpisode {
    id: u64,
    season_number: u32,
    episode_number: u32,
    name: String,
    #[schema(required)]
    air_date: Option<String>,
    overview: Option<String>,
    available: bool,
}
#[derive(Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum RequestStatus {
    Pending,
    Approved,
    Fulfilled,
    Declined,
    Cancelled,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct MediaRequest {
    stage: String,
    id: String,
    title: DiscoverTitle,
    user_id: String,
    username: String,
    scope: RequestScope,
    status: RequestStatus,
    note: String,
    created_at: i64,
    availability: Availability,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct WantedUnit {
    #[schema(required)]
    season: Option<u32>,
    #[schema(required)]
    episode: Option<u32>,
    name: String,
    #[schema(required)]
    air_date: Option<String>,
    wanted: bool,
    available: bool,
    offline: bool,
    #[schema(required)]
    item_id: Option<String>,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct Monitor {
    enabled: bool,
    scope: RequestScope,
    #[schema(required)]
    library_id: Option<String>,
    #[schema(required)]
    root: Option<String>,
    profile_id: String,
    automatic: bool,
    next_search: Option<i64>,
    failures: Option<u32>,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct MonitoredTitle {
    title: DiscoverTitle,
    units: Vec<WantedUnit>,
    monitor: Monitor,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct DiscoverResults {
    results: Vec<DiscoverTitle>,
    page: u32,
    total_pages: u32,
    attribution: String,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct TitleDetails {
    title: DiscoverTitle,
    episodes: Vec<CatalogEpisode>,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct TitlesEnvelope {
    titles: Vec<DiscoverTitle>,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct RequestsEnvelope {
    requests: Vec<MediaRequest>,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct RequestEnvelope {
    request: MediaRequest,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct MonitoredEnvelope {
    titles: Vec<MonitoredTitle>,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct MonitorEnvelope {
    monitor: Monitor,
}
