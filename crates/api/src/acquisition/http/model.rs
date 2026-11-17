use super::super::{access::Permissions, config::Config, import_plan::File, profiles::Profile};
use crate::contract::acquisition_catalog_model::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, ToSchema)]
pub(super) struct AcquisitionSettingsEnvelope {
    settings: Config,
    engine: String,
    network_policy: String,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(super) struct PermissionsEnvelope {
    permissions: Permissions,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(super) struct ProfileRecord {
    id: String,
    profile: Profile,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(super) struct ProfilesEnvelope {
    profiles: Vec<ProfileRecord>,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(super) struct ImportPreview {
    files: Vec<File>,
    token: String,
    copy_fallback: bool,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(super) struct AdoptionPreview {
    files: Vec<File>,
    token: String,
    notice: String,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(super) struct AdoptionResult {
    copied_files: u32,
    state: String,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(super) struct ImportRecord {
    id: String,
    file_index: u32,
    destination: String,
    state: String,
    #[schema(required)]
    error: Option<String>,
    #[schema(required)]
    item_id: Option<String>,
    episodes: Vec<EpisodeRef>,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(super) struct ImportsEnvelope {
    imports: Vec<ImportRecord>,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(super) struct AcquisitionEvent {
    id: i64,
    entity_id: String,
    message: String,
    created_at: i64,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(super) struct EventsEnvelope {
    events: Vec<AcquisitionEvent>,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(super) struct MigrationRecord {
    key: String,
    kind: MediaKind,
    provider_id: u64,
    title: String,
    #[schema(required)]
    owner: Option<String>,
    status: String,
    scope: RequestScope,
    enabled: bool,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(super) struct MigrationPreview {
    token: String,
    count: u32,
    records: Vec<MigrationRecord>,
    notice: String,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(super) struct MigrationResult {
    key: String,
    imported: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    error: Option<String>,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(super) struct MigrationResults {
    results: Vec<MigrationResult>,
}
