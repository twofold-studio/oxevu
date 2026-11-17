use super::{ErrorBody, playback_model::*};
use crate::*;
use utoipa_axum::{router::OpenApiRouter, routes};
pub(crate) fn router() -> OpenApiRouter<App> {
    OpenApiRouter::new()
        .routes(routes!(playback_capabilities))
        .routes(routes!(playback_create))
        .routes(routes!(playback_get))
        .routes(routes!(playback_heartbeat))
        .routes(routes!(playback_stop))
}

#[utoipa::path(get, path = "/playback/capabilities",
responses((status = 200, body = CapabilitiesEnvelope), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody),
(status = 429, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn playback_capabilities(
    State(a): State<App>,
    h: HeaderMap,
) -> Result<Json<CapabilitiesEnvelope>> {
    let Json(value) = crate::playback::capabilities(State(a), h).await?;
    Ok(Json(super::decode(value)?))
}

#[utoipa::path(post, path = "/playback/sessions", request_body = PlaybackSessionRequest,
responses((status = 200, body = PlaybackEnvelope), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody),
(status = 429, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn playback_create(
    State(a): State<App>,
    h: HeaderMap,
    super::ApiJson(body): super::ApiJson<PlaybackSessionRequest>,
) -> Result<Json<PlaybackEnvelope>> {
    let Json(value) =
        crate::playback::create(State(a), h, Json(serde_json::to_value(body).map_err(bad)?))
            .await?;
    Ok(Json(super::decode(value)?))
}

#[utoipa::path(get, path = "/playback/sessions/{id}", params(("id" = String, Path)),
responses((status = 200, body = PlaybackEnvelope), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody),
(status = 429, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn playback_get(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<PlaybackEnvelope>> {
    let Json(value) = crate::playback::get(State(a), h, Path(id)).await?;
    Ok(Json(super::decode(value)?))
}

#[utoipa::path(post, path = "/playback/sessions/{id}/heartbeat", params(("id" = String, Path)),
responses((status = 204), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody),
(status = 429, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn playback_heartbeat(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode> {
    crate::playback::heartbeat(State(a), h, Path(id)).await
}

#[utoipa::path(post, path = "/playback/sessions/{id}/stop", params(("id" = String, Path)),
responses((status = 204), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody),
(status = 429, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn playback_stop(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode> {
    crate::playback::stop(State(a), h, Path(id)).await
}
