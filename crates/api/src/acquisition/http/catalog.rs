use super::*;

pub(super) fn router() -> OpenApiRouter<App> {
    OpenApiRouter::new()
        .routes(routes!(discover))
        .routes(routes!(genres))
        .routes(routes!(title_details))
        .routes(routes!(title_trailer))
        .routes(routes!(titles_list))
        .routes(routes!(requests_list, request_create))
        .routes(routes!(request_decide))
        .routes(routes!(monitored_list))
        .routes(routes!(monitor_put))
}

#[utoipa::path(get, path = "/acquisition/discover", params(("query" = Option<String>, Query), ("kind" = Option<MediaKind>, Query), ("page" = Option<u32>, Query), ("year" = Option<u32>, Query), ("genre" = Option<u32>, Query), ("min_rating" = Option<f64>, Query), ("language" = Option<String>, Query), ("sort" = Option<super::super::discovery::Sort>, Query)),
responses((status = 200, body = DiscoverResults), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody),
(status = 429, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn discover(
    State(a): State<App>,
    h: HeaderMap,
    crate::contract::ApiQuery(query): crate::contract::ApiQuery<super::super::catalog::Search>,
) -> Result<Json<DiscoverResults>> {
    let Json(value) = super::super::catalog::discover(State(a), h, Query(query)).await?;
    Ok(Json(crate::contract::decode(value)?))
}

#[utoipa::path(get, path = "/acquisition/genres", params(("kind" = Option<MediaKind>, Query)),
responses((status = 200, body = Genres), (status = 400, body = ErrorBody), (status = 401, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn genres(
    State(a): State<App>,
    h: HeaderMap,
    crate::contract::ApiQuery(query): crate::contract::ApiQuery<super::super::catalog::Search>,
) -> Result<Json<Genres>> {
    let Json(value) = super::super::catalog::genres(State(a), h, Query(query)).await?;
    Ok(Json(crate::contract::decode(value)?))
}

#[utoipa::path(get, path = "/acquisition/titles/{kind}/{id}", params(("kind" = MediaKind, Path), ("id" = u64, Path)),
responses((status = 200, body = TitleDetails), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody),
(status = 429, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn title_details(
    State(a): State<App>,
    h: HeaderMap,
    Path((kind, id)): Path<(String, u64)>,
) -> Result<Json<TitleDetails>> {
    let Json(value) = super::super::catalog::details(State(a), h, Path((kind, id))).await?;
    Ok(Json(crate::contract::decode(value)?))
}

#[utoipa::path(get, path = "/acquisition/titles/{kind}/{id}/trailer", params(("kind" = MediaKind, Path), ("id" = u64, Path)),
responses((status = 200, body = Trailer), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn title_trailer(
    State(a): State<App>,
    h: HeaderMap,
    Path((kind, id)): Path<(String, u64)>,
) -> Result<Json<Trailer>> {
    Ok(Json(
        super::super::trailers::lookup(&a, &h, &kind, id).await?,
    ))
}

#[utoipa::path(get, path = "/acquisition/titles",
responses((status = 200, body = TitlesEnvelope), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody),
(status = 429, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn titles_list(State(a): State<App>, h: HeaderMap) -> Result<Json<TitlesEnvelope>> {
    let Json(value) = super::super::catalog::list(State(a), h).await?;
    Ok(Json(crate::contract::decode(value)?))
}

#[utoipa::path(get, path = "/acquisition/requests",
responses((status = 200, body = RequestsEnvelope), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody),
(status = 429, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn requests_list(State(a): State<App>, h: HeaderMap) -> Result<Json<RequestsEnvelope>> {
    let Json(value) = super::super::requests::list(State(a), h).await?;
    Ok(Json(crate::contract::decode(value)?))
}

#[utoipa::path(post, path = "/acquisition/requests", request_body = super::super::requests::Input,
responses((status = 200, body = RequestEnvelope), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody),
(status = 429, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn request_create(
    State(a): State<App>,
    h: HeaderMap,
    crate::contract::ApiJson(body): crate::contract::ApiJson<super::super::requests::Input>,
) -> Result<Json<RequestEnvelope>> {
    let Json(value) = super::super::requests::create(State(a), h, Json(body)).await?;
    Ok(Json(crate::contract::decode(value)?))
}

#[utoipa::path(put, path = "/acquisition/requests/{id}", params(("id" = String, Path)), request_body = super::super::requests::Decision,
responses((status = 200, body = RequestEnvelope), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody),
(status = 429, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn request_decide(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    crate::contract::ApiJson(body): crate::contract::ApiJson<super::super::requests::Decision>,
) -> Result<Json<RequestEnvelope>> {
    let Json(value) = super::super::requests::update(State(a), h, Path(id), Json(body)).await?;
    Ok(Json(crate::contract::decode(value)?))
}

#[utoipa::path(get, path = "/acquisition/monitored",
responses((status = 200, body = MonitoredEnvelope), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody),
(status = 429, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn monitored_list(State(a): State<App>, h: HeaderMap) -> Result<Json<MonitoredEnvelope>> {
    let Json(value) = super::super::monitoring::list(State(a), h).await?;
    Ok(Json(crate::contract::decode(value)?))
}

#[utoipa::path(put, path = "/acquisition/monitored/{id}", params(("id" = String, Path)), request_body = super::super::monitoring::Input,
responses((status = 200, body = MonitorEnvelope), (status = 400, body = ErrorBody),
(status = 401, body = ErrorBody), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody),
(status = 429, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn monitor_put(
    State(a): State<App>,
    h: HeaderMap,
    Path(id): Path<String>,
    crate::contract::ApiJson(body): crate::contract::ApiJson<super::super::monitoring::Input>,
) -> Result<Json<MonitorEnvelope>> {
    let Json(value) = super::super::monitoring::put(State(a), h, Path(id), Json(body)).await?;
    Ok(Json(crate::contract::decode(value)?))
}
