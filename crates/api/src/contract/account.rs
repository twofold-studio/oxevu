use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct Credentials {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct PasswordChange {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SyncplayAccess {
    CreateJoin,
    Join,
    Disabled,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct User {
    pub id: String,
    pub username: String,
    #[schema(required)]
    pub avatar: Option<String>,
    pub is_admin: bool,
    pub disabled: bool,
    pub can_play: bool,
    pub can_transcode: bool,
    pub syncplay_access: SyncplayAccess,
    pub all_libraries: bool,
    pub library_ids: Vec<String>,
    pub acquisition: crate::acquisition::Permissions,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct UserEnvelope {
    pub user: User,
}

#[derive(Serialize, ToSchema)]
pub(crate) struct AuthStatus {
    pub setup_required: bool,
    #[schema(required)]
    pub user: Option<User>,
}
