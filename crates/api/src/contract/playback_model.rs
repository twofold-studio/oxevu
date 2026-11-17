use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PlaybackMode {
    Auto,
    Direct,
    Transcode,
}
#[derive(Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PlaybackMethod {
    Direct,
    Remux,
    Cpu,
    Nvenc,
}
#[derive(Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PlaybackStatus {
    Starting,
    Ready,
    Failed,
}
#[derive(Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Decoder {
    Cuda,
    Cpu,
    Copy,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct PlaybackSessionRequest {
    item_id: String,
    start_seconds: f64,
    mode: PlaybackMode,
    direct_play_supported: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    video_bitrate_mbps: Option<f64>,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct CachedRange {
    start: f64,
    end: f64,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct PlaybackSession {
    id: String,
    item_id: String,
    method: PlaybackMethod,
    status: PlaybackStatus,
    #[schema(required)]
    stream_url: Option<String>,
    start_seconds: f64,
    #[schema(required)]
    duration_seconds: Option<f64>,
    #[schema(required)]
    reason: Option<String>,
    #[schema(required)]
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    video_bitrate_mbps: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    decoder: Option<Decoder>,
    #[serde(skip_serializing_if = "Option::is_none")]
    width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cache_end_seconds: Option<f64>,
    #[serde(default)]
    cache_hit: bool,
    #[serde(default)]
    progressive: bool,
    #[serde(default)]
    cached_ranges: Vec<CachedRange>,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct PlaybackEnvelope {
    session: PlaybackSession,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct Capabilities {
    ffmpeg_available: bool,
    nvenc_available: bool,
    hardware_decode_available: bool,
    #[schema(required)]
    ffmpeg_path: Option<String>,
    #[schema(required)]
    scratch_path: Option<String>,
    notes: Vec<String>,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct CapabilitiesEnvelope {
    capabilities: Capabilities,
}
