use super::{ErrorBody, UserEnvelope, admin_model::*};
use crate::*;
use utoipa_axum::{router::OpenApiRouter, routes};
pub(crate) fn router() -> OpenApiRouter<App> {
    OpenApiRouter::new()
        .routes(routes!(users_list, users_create))
        .routes(routes!(users_update))
        .routes(routes!(avatar_save))
        .routes(routes!(settings_get, settings_put))
}

#[utoipa::path(get, path = "/users",
responses((status = 200, body = UsersEnvelope), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn users_list(State(a): State<App>, h: HeaderMap) -> Result<Json<UsersEnvelope>> {
    let Json(value) = crate::users::list(State(a), h).await?;
    Ok(Json(super::decode(value)?))
}

#[utoipa::path(post, path = "/users", request_body = UserInput,
responses((status = 200, body = UserEnvelope), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn users_create(
    State(a): State<App>,
    h: HeaderMap,
    super::ApiJson(body): super::ApiJson<UserInput>,
) -> Result<Json<UserEnvelope>> {
    let Json(value) =
        crate::users::create(State(a), h, Json(serde_json::to_value(body).map_err(bad)?)).await?;
    Ok(Json(super::decode(value)?))
}

#[utoipa::path(put, path = "/users/{id}", params(("id" = String, Path)), request_body = UserInput,
responses((status = 200, body = UserEnvelope), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn users_update(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    super::ApiJson(body): super::ApiJson<UserInput>,
) -> Result<Json<UserEnvelope>> {
    let Json(value) = crate::users::update(
        State(a),
        h,
        Path(id),
        Json(serde_json::to_value(body).map_err(bad)?),
    )
    .await?;
    Ok(Json(super::decode(value)?))
}

#[utoipa::path(put, path = "/account/avatar", request_body = AvatarInput,
responses((status = 200, body = UserEnvelope), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn avatar_save(
    State(a): State<App>,
    h: HeaderMap,
    super::ApiJson(body): super::ApiJson<AvatarInput>,
) -> Result<Json<UserEnvelope>> {
    let Json(value) =
        crate::avatars::save(State(a), h, Json(serde_json::to_value(body).map_err(bad)?)).await?;
    Ok(Json(super::decode(value)?))
}

#[utoipa::path(get, path = "/settings",
responses((status = 200, body = SettingsEnvelope), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn settings_get(State(a): State<App>, h: HeaderMap) -> Result<Json<SettingsEnvelope>> {
    let Json(value) = crate::settings::get(State(a), h).await?;
    Ok(Json(super::decode(value)?))
}

#[utoipa::path(put, path = "/settings", request_body = Settings,
responses((status = 200, body = SettingsEnvelope), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn settings_put(
    State(a): State<App>,
    h: HeaderMap,
    super::ApiJson(body): super::ApiJson<Settings>,
) -> Result<Json<SettingsEnvelope>> {
    let Json(value) = crate::settings::put(
        State(a),
        h,
        Json(serde_json::from_value(serde_json::to_value(body).map_err(bad)?).map_err(bad)?),
    )
    .await?;
    Ok(Json(super::decode(value)?))
}
