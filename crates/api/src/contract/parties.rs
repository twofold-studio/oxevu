use super::{ErrorBody, party_model::*};
use crate::*;
use utoipa_axum::{router::OpenApiRouter, routes};
pub(crate) fn router() -> OpenApiRouter<App> {
    OpenApiRouter::new()
        .routes(routes!(party_list, party_create))
        .routes(routes!(party_read))
        .routes(routes!(party_join))
        .routes(routes!(party_leave))
        .routes(routes!(party_control))
}

#[utoipa::path(get, path = "/parties",
responses((status = 200, body = PartiesEnvelope), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody),
(status = 429, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn party_list(State(a): State<App>, h: HeaderMap) -> Result<Json<PartiesEnvelope>> {
    let Json(value) = crate::parties::list(State(a), h).await?;
    Ok(Json(super::decode(value)?))
}

#[utoipa::path(post, path = "/parties", request_body = PartyCreate,
responses((status = 200, body = PartyEnvelope), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody),
(status = 429, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn party_create(
    State(a): State<App>,
    h: HeaderMap,
    super::ApiJson(body): super::ApiJson<PartyCreate>,
) -> Result<Json<PartyEnvelope>> {
    let Json(value) =
        crate::parties::create(State(a), h, Json(serde_json::to_value(body).map_err(bad)?)).await?;
    Ok(Json(super::decode(value)?))
}

#[utoipa::path(get, path = "/parties/{id}", params(("id" = String, Path)),
responses((status = 200, body = PartyEnvelope), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody),
(status = 429, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn party_read(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<PartyEnvelope>> {
    let Json(value) = crate::parties::read(State(a), h, Path(id)).await?;
    Ok(Json(super::decode(value)?))
}

#[utoipa::path(post, path = "/parties/{id}/join", params(("id" = String, Path)),
responses((status = 200, body = PartyEnvelope), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody),
(status = 429, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn party_join(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<PartyEnvelope>> {
    let Json(value) = crate::parties::join(State(a), h, Path(id)).await?;
    Ok(Json(super::decode(value)?))
}

#[utoipa::path(post, path = "/parties/{id}/leave", params(("id" = String, Path)),
responses((status = 204), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody),
(status = 429, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn party_leave(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode> {
    crate::parties::leave(State(a), h, Path(id)).await
}

#[utoipa::path(post, path = "/parties/{id}/control", params(("id" = String, Path)), request_body = PartyControl,
responses((status = 200, body = PartyEnvelope), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody),
(status = 429, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn party_control(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    super::ApiJson(body): super::ApiJson<PartyControl>,
) -> Result<Json<PartyEnvelope>> {
    let Json(value) = crate::parties::control(
        State(a),
        h,
        Path(id),
        Json(serde_json::to_value(body).map_err(bad)?),
    )
    .await?;
    Ok(Json(super::decode(value)?))
}
