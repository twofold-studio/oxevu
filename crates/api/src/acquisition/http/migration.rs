use super::*;

pub(super) fn router() -> OpenApiRouter<App> {
    OpenApiRouter::new()
        .routes(routes!(migration_preview))
        .routes(routes!(migration_apply))
}

#[utoipa::path(post, path = "/acquisition/migration/preview", request_body = super::super::migration::Input,
responses((status = 200, body = MigrationPreview), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody),
(status = 429, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn migration_preview(
    State(a): State<App>,
    h: HeaderMap,
    crate::contract::ApiJson(body): crate::contract::ApiJson<super::super::migration::Input>,
) -> Result<Json<MigrationPreview>> {
    let Json(value) = super::super::migration::preview(State(a), h, Json(body)).await?;
    Ok(Json(crate::contract::decode(value)?))
}

#[utoipa::path(post, path = "/acquisition/migration", request_body = super::super::migration::Input,
responses((status = 200, body = MigrationResults), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody),
(status = 429, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn migration_apply(
    State(a): State<App>,
    h: HeaderMap,
    crate::contract::ApiJson(body): crate::contract::ApiJson<super::super::migration::Input>,
) -> Result<Json<MigrationResults>> {
    let Json(value) = super::super::migration::apply(State(a), h, Json(body)).await?;
    Ok(Json(crate::contract::decode(value)?))
}
