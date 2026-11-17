use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct PartyMember {
    user_id: String,
    username: String,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct Party {
    id: String,
    name: String,
    host_user_id: String,
    #[schema(required)]
    item_id: Option<String>,
    paused: bool,
    position_seconds: f64,
    updated_at_ms: u64,
    server_now_ms: u64,
    revision: u64,
    playback_revision: u64,
    members: Vec<PartyMember>,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct PartyEnvelope {
    party: Party,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct PartiesEnvelope {
    parties: Vec<Party>,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct PartyCreate {
    name: String,
    item_id: Option<String>,
}
#[derive(Deserialize, Serialize, ToSchema)]
#[serde(tag = "action", rename_all = "snake_case")]
pub(crate) enum PartyControl {
    Play {
        #[serde(skip_serializing_if = "Option::is_none")]
        position_seconds: Option<f64>,
    },
    Pause {
        #[serde(skip_serializing_if = "Option::is_none")]
        position_seconds: Option<f64>,
    },
    Seek {
        position_seconds: f64,
    },
    Item {
        item_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        position_seconds: Option<f64>,
    },
}
