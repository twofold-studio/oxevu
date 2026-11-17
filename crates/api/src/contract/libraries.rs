use super::ErrorBody;
use crate::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

#[derive(Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
enum LibraryKind {
    Movies,
    Tv,
}

#[derive(Deserialize, Serialize, ToSchema)]
struct Library {
    id: String,
    name: String,
    kind: LibraryKind,
    path: String,
    paths: Vec<String>,
    enabled: bool,
}

#[derive(Deserialize, Serialize, ToSchema)]
struct LibraryInput {
    name: String,
    kind: LibraryKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    paths: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    enabled: Option<bool>,
}

#[derive(Deserialize, Serialize, ToSchema)]
struct LibraryEnvelope {
    library: Library,
}
#[derive(Deserialize, Serialize, ToSchema)]
struct LibrariesEnvelope {
    libraries: Vec<Library>,
}
#[derive(Deserialize, Serialize, ToSchema)]
struct ScanResult {
    scanned: u64,
    errors: Vec<String>,
}

pub(crate) fn router() -> OpenApiRouter<App> {
    OpenApiRouter::new()
        .routes(routes!(list, create))
        .routes(routes!(update))
        .routes(routes!(remove))
        .routes(routes!(scan))
}

#[utoipa::path(get, path = "/libraries", responses((status = 200, body = LibrariesEnvelope),
    (status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn list(State(a): State<App>, h: HeaderMap) -> Result<Json<LibrariesEnvelope>> {
    let Json(value) = crate::libraries::libraries(State(a), h).await?;
    Ok(Json(super::decode(value)?))
}

#[utoipa::path(post, path = "/libraries", request_body = LibraryInput,
    responses((status = 200, body = LibraryEnvelope), (status = 400, body = ErrorBody),
        (status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn create(
    State(a): State<App>,
    h: HeaderMap,
    super::ApiJson(body): super::ApiJson<LibraryInput>,
) -> Result<Json<LibraryEnvelope>> {
    let Json(value) =
        crate::libraries::add_library(State(a), h, Json(serde_json::to_value(body).map_err(bad)?))
            .await?;
    Ok(Json(super::decode(value)?))
}

#[utoipa::path(put, path = "/libraries/{id}", params(("id" = String, Path)), request_body = LibraryInput,
    responses((status = 200, body = LibraryEnvelope), (status = 400, body = ErrorBody),
        (status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn update(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    super::ApiJson(body): super::ApiJson<LibraryInput>,
) -> Result<Json<LibraryEnvelope>> {
    let Json(value) = crate::libraries::update(
        State(a),
        h,
        Path(id),
        Json(serde_json::to_value(body).map_err(bad)?),
    )
    .await?;
    Ok(Json(super::decode(value)?))
}

#[utoipa::path(post, path = "/libraries/{id}/remove", params(("id" = String, Path)),
    responses((status = 204), (status = 400, body = ErrorBody), (status = 401, body = ErrorBody),
        (status = 403, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn remove(State(a): State<App>, h: HeaderMap, Path(id): Path<String>) -> Result<StatusCode> {
    crate::libraries::remove(State(a), h, Path(id)).await
}

#[utoipa::path(post, path = "/libraries/{id}/scan", params(("id" = String, Path)),
    responses((status = 200, body = ScanResult), (status = 400, body = ErrorBody),
        (status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn scan(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<ScanResult>> {
    let Json(value) = crate::scan::scan(State(a), h, Path(id)).await?;
    Ok(Json(super::decode(value)?))
}
