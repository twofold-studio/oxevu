use super::{ErrorBody, item::Item};
use crate::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

#[derive(Deserialize, Serialize, ToSchema)]
struct ItemsEnvelope {
    items: Vec<Item>,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct ItemEnvelope {
    pub item: Item,
}
#[derive(Deserialize, Serialize, ToSchema)]
struct FavoriteUpdate {
    favorite: bool,
}
#[derive(Deserialize, Serialize, ToSchema)]
struct ProgressUpdate {
    position_seconds: f64,
    completed: bool,
}

pub(crate) fn router() -> OpenApiRouter<App> {
    OpenApiRouter::new()
        .routes(routes!(items))
        .routes(routes!(item))
        .routes(routes!(favorite))
        .routes(routes!(progress))
}

#[utoipa::path(get, path = "/items", responses((status = 200, body = ItemsEnvelope),
    (status = 401, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn items(State(a): State<App>, h: HeaderMap) -> Result<Json<ItemsEnvelope>> {
    let Json(value) = crate::catalog::items(State(a), h).await?;
    Ok(Json(super::decode(value)?))
}

#[utoipa::path(get, path = "/items/{id}", params(("id" = String, Path)),
    responses((status = 200, body = ItemEnvelope), (status = 401, body = ErrorBody),
        (status = 404, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn item(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<ItemEnvelope>> {
    let Json(value) = crate::catalog::item(State(a), h, Path(id)).await?;
    Ok(Json(super::decode(value)?))
}

#[utoipa::path(put, path = "/items/{id}/favorite", params(("id" = String, Path)), request_body = FavoriteUpdate,
    responses((status = 204), (status = 400, body = ErrorBody), (status = 401, body = ErrorBody),
        (status = 403, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn favorite(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    super::ApiJson(body): super::ApiJson<FavoriteUpdate>,
) -> Result<StatusCode> {
    crate::catalog::favorite(
        State(a),
        h,
        Path(id),
        Json(serde_json::to_value(body).map_err(bad)?),
    )
    .await
}

#[utoipa::path(put, path = "/items/{id}/progress", params(("id" = String, Path)), request_body = ProgressUpdate,
    responses((status = 204), (status = 400, body = ErrorBody), (status = 401, body = ErrorBody),
        (status = 403, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn progress(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    super::ApiJson(body): super::ApiJson<ProgressUpdate>,
) -> Result<StatusCode> {
    crate::catalog::progress(
        State(a),
        h,
        Path(id),
        Json(serde_json::to_value(body).map_err(bad)?),
    )
    .await
}
