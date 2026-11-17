use super::*;

pub(super) fn router() -> OpenApiRouter<App> {
    OpenApiRouter::new()
        .routes(routes!(acquisition_settings_get, acquisition_settings_put))
        .routes(routes!(profiles_list))
        .routes(routes!(profile_put))
        .routes(routes!(indexers_list))
        .routes(routes!(indexer_put))
        .routes(routes!(indexer_test))
        .routes(routes!(release_sources))
        .routes(routes!(access_get))
        .routes(routes!(access_put))
        .routes(routes!(import_defaults))
        .routes(routes!(events_list))
        .routes(routes!(acquisition_artwork))
}

#[utoipa::path(get, path = "/acquisition/settings",
responses((status = 200, body = AcquisitionSettingsEnvelope), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody),
(status = 429, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn acquisition_settings_get(
    State(a): State<App>,
    h: HeaderMap,
) -> Result<Json<AcquisitionSettingsEnvelope>> {
    let Json(value) = super::super::config::get(State(a), h).await?;
    Ok(Json(crate::contract::decode(value)?))
}

#[utoipa::path(put, path = "/acquisition/settings", request_body = super::super::config::Config,
responses((status = 200, body = AcquisitionSettingsEnvelope), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody),
(status = 429, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn acquisition_settings_put(
    State(a): State<App>,
    h: HeaderMap,
    crate::contract::ApiJson(body): crate::contract::ApiJson<super::super::config::Config>,
) -> Result<Json<AcquisitionSettingsEnvelope>> {
    let Json(value) = super::super::config::put(State(a), h, Json(body)).await?;
    Ok(Json(crate::contract::decode(value)?))
}

#[utoipa::path(get, path = "/acquisition/profiles",
responses((status = 200, body = ProfilesEnvelope), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody),
(status = 429, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn profiles_list(State(a): State<App>, h: HeaderMap) -> Result<Json<ProfilesEnvelope>> {
    let Json(value) = super::super::profiles::list(State(a), h).await?;
    Ok(Json(crate::contract::decode(value)?))
}

#[utoipa::path(put, path = "/acquisition/profiles/{id}", params(("id" = String, Path)), request_body = super::super::profiles::Profile,
responses((status = 200, body = ProfileRecord), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody),
(status = 429, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn profile_put(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    crate::contract::ApiJson(body): crate::contract::ApiJson<super::super::profiles::Profile>,
) -> Result<Json<ProfileRecord>> {
    let Json(value) = super::super::profiles::put(State(a), h, Path(id), Json(body)).await?;
    Ok(Json(crate::contract::decode(value)?))
}

#[utoipa::path(get, path = "/acquisition/indexers",
responses((status = 200, body = IndexersEnvelope), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody),
(status = 429, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn indexers_list(State(a): State<App>, h: HeaderMap) -> Result<Json<IndexersEnvelope>> {
    let Json(value) = super::super::indexers::list(State(a), h).await?;
    Ok(Json(crate::contract::decode(value)?))
}

#[utoipa::path(put, path = "/acquisition/indexers/{id}", params(("id" = String, Path)), request_body = super::super::indexers::Indexer,
responses((status = 200, body = IndexerEnvelope), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody),
(status = 429, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn indexer_put(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    crate::contract::ApiJson(body): crate::contract::ApiJson<super::super::indexers::Indexer>,
) -> Result<Json<IndexerEnvelope>> {
    let Json(value) = super::super::indexers::put(State(a), h, Path(id), Json(body)).await?;
    Ok(Json(crate::contract::decode(value)?))
}

#[utoipa::path(post, path = "/acquisition/indexers/{id}/test", params(("id" = String, Path)),
responses((status = 200, body = IndexerEnvelope), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody),
(status = 429, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn indexer_test(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<IndexerEnvelope>> {
    let Json(value) = super::super::indexers::test(State(a), h, Path(id)).await?;
    Ok(Json(crate::contract::decode(value)?))
}

#[utoipa::path(get, path = "/acquisition/release-sources",
responses((status = 200, body = ReleaseSources), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody),
(status = 429, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn release_sources(State(a): State<App>, h: HeaderMap) -> Result<Json<ReleaseSources>> {
    let Json(value) = super::super::indexers::sources(State(a), h).await?;
    Ok(Json(crate::contract::decode(value)?))
}

#[utoipa::path(get, path = "/acquisition/access",
responses((status = 200, body = PermissionsEnvelope), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody),
(status = 429, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn access_get(State(a): State<App>, h: HeaderMap) -> Result<Json<PermissionsEnvelope>> {
    let Json(value) = super::super::access::get(State(a), h).await?;
    Ok(Json(crate::contract::decode(value)?))
}

#[utoipa::path(put, path = "/acquisition/access/{id}", params(("id" = String, Path)), request_body = super::super::access::Permissions,
responses((status = 200, body = PermissionsEnvelope), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody),
(status = 429, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn access_put(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    crate::contract::ApiJson(body): crate::contract::ApiJson<super::super::access::Permissions>,
) -> Result<Json<PermissionsEnvelope>> {
    let Json(value) = super::super::access::put(State(a), h, Path(id), Json(body)).await?;
    Ok(Json(crate::contract::decode(value)?))
}

#[utoipa::path(get, path = "/acquisition/import-defaults",
responses((status = 200, body = ImportDefaults), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody),
(status = 429, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn import_defaults(State(a): State<App>, h: HeaderMap) -> Result<Json<ImportDefaults>> {
    let Json(value) = super::super::destinations::get(State(a), h).await?;
    Ok(Json(crate::contract::decode(value)?))
}

#[utoipa::path(get, path = "/acquisition/events",
responses((status = 200, body = EventsEnvelope), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody),
(status = 429, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn events_list(State(a): State<App>, h: HeaderMap) -> Result<Json<EventsEnvelope>> {
    let Json(value) = super::super::events(State(a), h).await?;
    Ok(Json(crate::contract::decode(value)?))
}

#[utoipa::path(get, path = "/acquisition/artwork/{file}", params(("file" = String, Path)),
responses((status = 200, description = "Native artwork bytes"), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn acquisition_artwork(
    State(a): State<App>,
    h: HeaderMap,
    Path(file): Path<String>,
) -> Result<Response> {
    super::super::catalog::artwork(State(a), h, Path(file)).await
}
