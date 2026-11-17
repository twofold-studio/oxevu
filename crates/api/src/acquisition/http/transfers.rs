use super::*;

pub(super) fn router() -> OpenApiRouter<App> {
    OpenApiRouter::new()
        .merge(
            OpenApiRouter::new()
                .routes(routes!(downloads_list, download_create))
                .layer(axum::extract::DefaultBodyLimit::max(
                    3 * 1024 * 1024 + 65536,
                )),
        )
        .routes(routes!(download_get, download_control))
        .routes(routes!(download_limit))
        .routes(routes!(download_identity))
        .routes(routes!(releases_search))
        .routes(routes!(import_preview))
        .routes(routes!(import_apply))
        .routes(routes!(adoption_preview))
        .routes(routes!(adoption_apply))
}

#[utoipa::path(get, path = "/acquisition/downloads",
responses((status = 200, body = DownloadsEnvelope), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody),
(status = 429, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn downloads_list(State(a): State<App>, h: HeaderMap) -> Result<Json<DownloadsEnvelope>> {
    let Json(value) = super::super::downloads::list(State(a), h).await?;
    Ok(Json(crate::contract::decode(value)?))
}

#[utoipa::path(post, path = "/acquisition/downloads", request_body = super::super::downloads::Input,
responses((status = 200, body = DownloadEnvelope), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody),
(status = 429, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn download_create(
    State(a): State<App>,
    h: HeaderMap,
    crate::contract::ApiJson(body): crate::contract::ApiJson<super::super::downloads::Input>,
) -> Result<Json<DownloadEnvelope>> {
    let Json(value) = super::super::downloads::create(State(a), h, Json(body)).await?;
    Ok(Json(crate::contract::decode(value)?))
}

#[utoipa::path(get, path = "/acquisition/downloads/{id}", params(("id" = String, Path)),
responses((status = 200, body = DownloadEnvelope), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody),
(status = 429, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn download_get(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<DownloadEnvelope>> {
    let Json(value) = super::super::downloads::read(State(a), h, Path(id)).await?;
    Ok(Json(crate::contract::decode(value)?))
}

#[utoipa::path(put, path = "/acquisition/downloads/{id}", params(("id" = String, Path)), request_body = super::super::downloads::Action,
responses((status = 200, body = DownloadEnvelope), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody),
(status = 429, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn download_control(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    crate::contract::ApiJson(body): crate::contract::ApiJson<super::super::downloads::Action>,
) -> Result<Json<DownloadEnvelope>> {
    let Json(value) = super::super::downloads::control(State(a), h, Path(id), Json(body)).await?;
    Ok(Json(crate::contract::decode(value)?))
}

#[utoipa::path(put, path = "/acquisition/downloads/{id}/limits", params(("id" = String, Path)), request_body = super::super::download_limits::Input,
responses((status = 200, body = DownloadEnvelope), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody),
(status = 429, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn download_limit(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    crate::contract::ApiJson(body): crate::contract::ApiJson<super::super::download_limits::Input>,
) -> Result<Json<DownloadEnvelope>> {
    let Json(value) = super::super::download_limits::put(State(a), h, Path(id), Json(body)).await?;
    Ok(Json(crate::contract::decode(value)?))
}

#[utoipa::path(put, path = "/acquisition/downloads/{id}/identity", params(("id" = String, Path)), request_body = super::super::recognition::Identity,
responses((status = 200, body = DownloadEnvelope), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody),
(status = 429, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn download_identity(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    crate::contract::ApiJson(body): crate::contract::ApiJson<super::super::recognition::Identity>,
) -> Result<Json<DownloadEnvelope>> {
    let Json(value) =
        super::super::recognition::associate(State(a), h, Path(id), Json(body)).await?;
    Ok(Json(crate::contract::decode(value)?))
}

#[utoipa::path(post, path = "/acquisition/titles/{id}/releases", params(("id" = String, Path)), request_body = super::super::releases::Search,
responses((status = 200, body = ReleaseResults), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody),
(status = 429, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn releases_search(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    crate::contract::ApiJson(body): crate::contract::ApiJson<super::super::releases::Search>,
) -> Result<Json<ReleaseResults>> {
    let Json(value) = super::super::releases::search(State(a), h, Path(id), Json(body)).await?;
    Ok(Json(crate::contract::decode(value)?))
}

#[utoipa::path(post, path = "/acquisition/downloads/{id}/imports/preview", params(("id" = String, Path)), request_body = super::super::import_plan::Input,
responses((status = 200, body = ImportPreview), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody),
(status = 429, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn import_preview(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    crate::contract::ApiJson(body): crate::contract::ApiJson<super::super::import_plan::Input>,
) -> Result<Json<ImportPreview>> {
    let Json(value) = super::super::imports::preview(State(a), h, Path(id), Json(body)).await?;
    Ok(Json(crate::contract::decode(value)?))
}

#[utoipa::path(post, path = "/acquisition/downloads/{id}/imports", params(("id" = String, Path)), request_body = super::super::import_plan::Input,
responses((status = 200, body = ImportsEnvelope), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody),
(status = 429, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn import_apply(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    crate::contract::ApiJson(body): crate::contract::ApiJson<super::super::import_plan::Input>,
) -> Result<Json<ImportsEnvelope>> {
    let Json(value) = super::super::imports::apply(State(a), h, Path(id), Json(body)).await?;
    Ok(Json(crate::contract::decode(value)?))
}

#[utoipa::path(post, path = "/acquisition/downloads/{id}/adoption/preview", params(("id" = String, Path)), request_body = super::super::adoption::Input,
responses((status = 200, body = AdoptionPreview), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody),
(status = 429, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn adoption_preview(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    crate::contract::ApiJson(body): crate::contract::ApiJson<super::super::adoption::Input>,
) -> Result<Json<AdoptionPreview>> {
    let Json(value) = super::super::adoption::preview(State(a), h, Path(id), Json(body)).await?;
    Ok(Json(crate::contract::decode(value)?))
}

#[utoipa::path(post, path = "/acquisition/downloads/{id}/adoption", params(("id" = String, Path)), request_body = super::super::adoption::Input,
responses((status = 200, body = AdoptionResult), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody),
(status = 429, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn adoption_apply(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    crate::contract::ApiJson(body): crate::contract::ApiJson<super::super::adoption::Input>,
) -> Result<Json<AdoptionResult>> {
    let Json(value) = super::super::adoption::apply(State(a), h, Path(id), Json(body)).await?;
    Ok(Json(crate::contract::decode(value)?))
}
