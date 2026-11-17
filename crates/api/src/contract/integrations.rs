use super::{ErrorBody, catalog::ItemEnvelope, integration_model::*};
use crate::*;
use axum::extract::Query;
use utoipa_axum::{router::OpenApiRouter, routes};

pub(crate) fn router() -> OpenApiRouter<App> {
    OpenApiRouter::new()
        .routes(routes!(integrations_get, integrations_put))
        .routes(routes!(metadata_search))
        .routes(routes!(metadata_match))
}

#[utoipa::path(get, path = "/integrations", responses((status = 200, body = IntegrationEnvelope),
    (status = 400, body = ErrorBody), (status = 401, body = ErrorBody),
    (status = 403, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn integrations_get(State(a): State<App>, h: HeaderMap) -> Result<Json<IntegrationEnvelope>> {
    let Json(value) = crate::integrations::get(State(a), h).await?;
    Ok(Json(super::decode(value)?))
}

#[utoipa::path(put, path = "/integrations", request_body = IntegrationInput,
    responses((status = 200, body = IntegrationEnvelope), (status = 400, body = ErrorBody),
        (status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn integrations_put(
    State(a): State<App>,
    h: HeaderMap,
    super::ApiJson(body): super::ApiJson<IntegrationInput>,
) -> Result<Json<IntegrationEnvelope>> {
    let Json(value) =
        crate::integrations::put(State(a), h, Json(serde_json::to_value(body).map_err(bad)?))
            .await?;
    Ok(Json(super::decode(value)?))
}

#[utoipa::path(get, path = "/items/{id}/metadata/search",
    params(("id" = String, Path), ("provider" = MetadataProvider, Query), ("query" = String, Query)),
    responses((status = 200, body = MetadataResults), (status = 400, body = ErrorBody),
        (status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn metadata_search(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    super::ApiQuery(query): super::ApiQuery<MetadataSearch>,
) -> Result<Json<MetadataResults>> {
    let query: crate::integrations::Search =
        serde_json::from_value(serde_json::to_value(query).map_err(bad)?).map_err(bad)?;
    let Json(value) = crate::integrations::search(State(a), h, Path(id), Query(query)).await?;
    Ok(Json(super::decode(value)?))
}

#[utoipa::path(post, path = "/items/{id}/metadata/match", params(("id" = String, Path)), request_body = MetadataMatch,
    responses((status = 200, body = ItemEnvelope), (status = 400, body = ErrorBody),
        (status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn metadata_match(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    super::ApiJson(body): super::ApiJson<MetadataMatch>,
) -> Result<Json<ItemEnvelope>> {
    let body: crate::integrations::Match =
        serde_json::from_value(serde_json::to_value(body).map_err(bad)?).map_err(bad)?;
    let Json(value) = crate::integrations::apply(State(a), h, Path(id), Json(body)).await?;
    Ok(Json(super::decode(value)?))
}
