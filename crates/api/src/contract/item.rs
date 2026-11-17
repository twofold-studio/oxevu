use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ItemKind {
    Movie,
    Episode,
}
#[derive(Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Reaction {
    Like,
    Dislike,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct DownloadStatus {
    id: String,
    file_index: u32,
    state: String,
    bytes_received: u64,
    #[schema(required)]
    size_bytes: Option<u64>,
    playable: bool,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct StudioImage {
    name: String,
    url: String,
}
#[derive(Deserialize, Serialize, ToSchema)]
#[serde(untagged)]
pub(crate) enum ProviderId {
    Number(u64),
    Text(String),
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct ProviderIds {
    tmdb: Option<ProviderId>,
    imdb: Option<String>,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct ExternalUrls {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    tmdb: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    imdb: Option<String>,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct Item {
    pub id: String,
    pub library_id: String,
    pub title: String,
    pub kind: ItemKind,
    #[schema(required)]
    pub show_title: Option<String>,
    #[schema(required)]
    pub season_number: Option<u32>,
    #[schema(required)]
    pub episode_number: Option<u32>,
    #[schema(required)]
    pub duration_seconds: Option<f64>,
    #[schema(required)]
    pub container: Option<String>,
    #[schema(required)]
    pub video_codec: Option<String>,
    #[schema(required)]
    pub audio_codec: Option<String>,
    pub position_seconds: f64,
    pub completed: bool,
    #[serde(default)]
    pub favorite: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bitrate: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub video_width: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub video_height: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub video_bitrate: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio_bitrate: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio_channels: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio_channel_layout: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub poster_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backdrop_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_available: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_removed_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub download: Option<DownloadStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reaction: Option<Reaction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overview: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub year: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_year: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rating: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rating_source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub season_overview: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub season_rating: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_rating: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub genres: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub studios: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub studio_images: Option<Vec<StudioImage>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub creators: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_ids: Option<ProviderIds>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_urls: Option<ExternalUrls>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_overview: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_year: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_end_year: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_content_rating: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_rating: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_rating_source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_genres: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_studios: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_creators: Option<Vec<String>>,
}
