use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct ProviderStatus {
    enabled: bool,
    configured: bool,
    api_key_set: bool,
    username: String,
    password_set: bool,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct StudioImages {
    enabled: bool,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct IntegrationStatuses {
    tmdb: ProviderStatus,
    omdb: ProviderStatus,
    opensubtitles: ProviderStatus,
    studio_images: StudioImages,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct IntegrationEnvelope {
    integrations: IntegrationStatuses,
    requirements: std::collections::BTreeMap<String, String>,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct ProviderUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    clear_credentials: Option<bool>,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct IntegrationUpdates {
    #[serde(skip_serializing_if = "Option::is_none")]
    tmdb: Option<ProviderUpdate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    omdb: Option<ProviderUpdate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    opensubtitles: Option<ProviderUpdate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    studio_images: Option<StudioImages>,
}
#[derive(Deserialize, Serialize, ToSchema)]
#[serde(untagged)]
pub(crate) enum IntegrationInput {
    Wrapped { integrations: IntegrationUpdates },
    Direct(IntegrationUpdates),
}
#[derive(Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum MetadataProvider {
    Tmdb,
    Omdb,
}
#[derive(Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum MetadataKind {
    Movie,
    Tv,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct MetadataSearch {
    provider: MetadataProvider,
    query: String,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct MetadataMatch {
    provider: MetadataProvider,
    id: String,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct MetadataResult {
    provider: MetadataProvider,
    id: String,
    title: String,
    kind: MetadataKind,
    #[schema(required)]
    year: Option<super::item::ProviderId>,
    #[schema(required)]
    overview: Option<String>,
    #[schema(required)]
    poster_url: Option<String>,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct MetadataResults {
    results: Vec<MetadataResult>,
}
