use super::account::{SyncplayAccess, User};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct UsersEnvelope {
    users: Vec<User>,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct UserInput {
    username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_admin: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    disabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    can_play: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    can_transcode: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    all_libraries: Option<bool>,
    syncplay_access: SyncplayAccess,
    library_ids: Vec<String>,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct AvatarInput {
    image: String,
}

#[derive(Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HardwareAcceleration {
    None,
    Nvenc,
}
#[derive(Deserialize, Serialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct Settings {
    transcoding_enabled: bool,
    hardware_acceleration: HardwareAcceleration,
    hardware_encoding: bool,
    hardware_decoding: bool,
    gpu_index: u32,
    allow_cpu_fallback: bool,
    max_concurrent_transcodes: u32,
    max_video_bitrate_mbps: f64,
    max_height: u32,
    cpu_crf: u32,
    scratch_limit_gb: f64,
    #[serde(default = "default_cache_enabled")]
    #[schema(required)]
    cache_enabled: bool,
    #[serde(default = "default_cache_limit")]
    #[schema(required)]
    cache_limit_gb: f64,
    session_idle_seconds: u32,
    resume_min_percent: f64,
    resume_max_percent: f64,
    resume_min_duration_seconds: u32,
}
fn default_cache_enabled() -> bool {
    true
}
fn default_cache_limit() -> f64 {
    5.
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct SettingsEnvelope {
    settings: Settings,
    capabilities: super::playback_model::Capabilities,
}
