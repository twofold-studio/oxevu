use super::{ErrorBody, library_operations_model::*};
use crate::*;
use axum::extract::Query;
use utoipa_axum::{router::OpenApiRouter, routes};
pub(crate) fn router() -> OpenApiRouter<App> {
    OpenApiRouter::new()
        .routes(routes!(paths_suggest))
        .routes(routes!(removal_preview))
        .routes(routes!(removal_apply))
        .routes(routes!(reaction_update))
        .routes(routes!(subtitles_list))
        .routes(routes!(subtitles_search))
        .routes(routes!(subtitles_download))
}

#[utoipa::path(post, path = "/libraries/paths/suggest", request_body = PathInput,
responses((status = 200, body = PathSuggestions), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn paths_suggest(
    State(a): State<App>,
    h: HeaderMap,
    super::ApiJson(body): super::ApiJson<PathInput>,
) -> Result<Json<PathSuggestions>> {
    let Json(value) = crate::library_paths::suggest(
        State(a),
        h,
        Json(serde_json::from_value(serde_json::to_value(body).map_err(bad)?).map_err(bad)?),
    )
    .await?;
    Ok(Json(super::decode(value)?))
}

#[utoipa::path(post, path = "/library/removals/preview", request_body = RemovalInput,
responses((status = 200, body = RemovalPreview), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn removal_preview(
    State(a): State<App>,
    h: HeaderMap,
    super::ApiJson(body): super::ApiJson<RemovalInput>,
) -> Result<Json<RemovalPreview>> {
    let Json(value) = crate::media_files::preview(
        State(a),
        h,
        Json(serde_json::from_value(serde_json::to_value(body).map_err(bad)?).map_err(bad)?),
    )
    .await?;
    Ok(Json(super::decode(value)?))
}

#[utoipa::path(post, path = "/library/removals", request_body = RemovalInput,
responses((status = 200, body = RemovalResult), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn removal_apply(
    State(a): State<App>,
    h: HeaderMap,
    super::ApiJson(body): super::ApiJson<RemovalInput>,
) -> Result<Json<RemovalResult>> {
    let Json(value) = crate::media_files::apply(
        State(a),
        h,
        Json(serde_json::from_value(serde_json::to_value(body).map_err(bad)?).map_err(bad)?),
    )
    .await?;
    Ok(Json(super::decode(value)?))
}

#[utoipa::path(put, path = "/items/{id}/reaction", params(("id" = String, Path)), request_body = ReactionUpdate,
responses((status = 200, body = ReactionUpdate), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn reaction_update(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    super::ApiJson(body): super::ApiJson<ReactionUpdate>,
) -> Result<Json<ReactionUpdate>> {
    let Json(value) = crate::media_files::reaction(
        State(a),
        h,
        Path(id),
        Json(serde_json::to_value(body).map_err(bad)?),
    )
    .await?;
    Ok(Json(super::decode(value)?))
}

#[utoipa::path(get, path = "/items/{id}/subtitles", params(("id" = String, Path)),
responses((status = 200, body = SubtitleTracks), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn subtitles_list(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<SubtitleTracks>> {
    let Json(value) = crate::subtitles::list(State(a), h, Path(id)).await?;
    Ok(Json(super::decode(value)?))
}

#[utoipa::path(get, path = "/items/{id}/subtitles/search", params(("id" = String, Path), ("language" = String, Query)),
responses((status = 200, body = SubtitleResults), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn subtitles_search(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    super::ApiQuery(body): super::ApiQuery<SubtitleSearch>,
) -> Result<Json<SubtitleResults>> {
    let Json(value) = crate::integrations::subtitle_search(
        State(a),
        h,
        Path(id),
        Query(serde_json::from_value(serde_json::to_value(body).map_err(bad)?).map_err(bad)?),
    )
    .await?;
    Ok(Json(super::decode(value)?))
}

#[utoipa::path(post, path = "/items/{id}/subtitles/download", params(("id" = String, Path)), request_body = SubtitleDownload,
responses((status = 200, body = SubtitleEnvelope), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn subtitles_download(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    super::ApiJson(body): super::ApiJson<SubtitleDownload>,
) -> Result<Json<SubtitleEnvelope>> {
    let Json(value) = crate::integrations::subtitle_download(
        State(a),
        h,
        Path(id),
        Json(serde_json::from_value(serde_json::to_value(body).map_err(bad)?).map_err(bad)?),
    )
    .await?;
    Ok(Json(super::decode(value)?))
}
