use super::ErrorBody;
use crate::*;
use utoipa_axum::{router::OpenApiRouter, routes};
pub(crate) fn router() -> OpenApiRouter<App> {
    OpenApiRouter::new()
        .routes(routes!(media_stream))
        .routes(routes!(media_hls))
        .routes(routes!(media_subtitle))
        .routes(routes!(media_artwork))
        .routes(routes!(metadata_artwork))
        .routes(routes!(studio_artwork))
}

#[utoipa::path(get, path = "/items/{id}/stream", params(("id" = String, Path)),
responses((status = 200, description = "Native media bytes; use the URL with browser media APIs"),
(status = 206, description = "Partial media bytes"), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody),
(status = 416, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn media_stream(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    request: Request,
) -> Result<Response> {
    crate::catalog::stream(State(a), h, Path(id), request).await
}

#[utoipa::path(get, path = "/playback/sessions/{id}/hls/{file}", params(("id" = String, Path), ("file" = String, Path)),
responses((status = 200, description = "Native media bytes; use the URL with browser media APIs"),
(status = 206, description = "Partial media bytes"), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody),
(status = 416, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn media_hls(
    State(a): State<App>,
    h: HeaderMap,
    Path((id, file)): Path<(String, String)>,
    request: Request,
) -> Result<Response> {
    crate::playback::hls(State(a), h, Path((id, file)), request).await
}

#[utoipa::path(get, path = "/items/{id}/subtitles/{track}", params(("id" = String, Path), ("track" = String, Path)),
responses((status = 200, description = "Native media bytes; use the URL with browser media APIs"),
(status = 206, description = "Partial media bytes"), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody),
(status = 416, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn media_subtitle(
    State(a): State<App>,
    h: HeaderMap,
    Path((id, track)): Path<(String, String)>,
) -> Result<Response> {
    crate::subtitles::serve(State(a), h, Path((id, track))).await
}

#[utoipa::path(get, path = "/items/{id}/artwork/{kind}", params(("id" = String, Path), ("kind" = String, Path)),
responses((status = 200, description = "Native media bytes; use the URL with browser media APIs"),
(status = 206, description = "Partial media bytes"), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody),
(status = 416, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn media_artwork(
    State(a): State<App>,
    h: HeaderMap,
    Path((id, kind)): Path<(String, String)>,
) -> Result<Response> {
    crate::artwork::serve(State(a), h, Path((id, kind))).await
}

#[utoipa::path(get, path = "/items/{id}/metadata/artwork/{kind}", params(("id" = String, Path), ("kind" = String, Path)),
responses((status = 200, description = "Native media bytes; use the URL with browser media APIs"),
(status = 206, description = "Partial media bytes"), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody),
(status = 416, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn metadata_artwork(
    State(a): State<App>,
    h: HeaderMap,
    Path((id, kind)): Path<(String, String)>,
) -> Result<Response> {
    crate::integrations::metadata_artwork(State(a), h, Path((id, kind))).await
}

#[utoipa::path(get, path = "/items/{id}/metadata/studios/{name}", params(("id" = String, Path), ("name" = String, Path)),
responses((status = 200, description = "Native media bytes; use the URL with browser media APIs"),
(status = 206, description = "Partial media bytes"), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody),
(status = 416, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn studio_artwork(
    State(a): State<App>,
    h: HeaderMap,
    Path((id, name)): Path<(String, String)>,
) -> Result<Response> {
    crate::integrations::studio(State(a), h, Path((id, name))).await
}
