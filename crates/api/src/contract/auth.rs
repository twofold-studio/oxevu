use super::{AuthStatus, Credentials, ErrorBody, PasswordChange, UserEnvelope};
use crate::*;
use utoipa_axum::{router::OpenApiRouter, routes};

pub(crate) fn router() -> OpenApiRouter<App> {
    OpenApiRouter::new()
        .routes(routes!(status))
        .routes(routes!(setup))
        .routes(routes!(login))
        .routes(routes!(logout))
        .routes(routes!(password))
}

#[utoipa::path(get, path = "/auth/status", responses(
    (status = 200, body = AuthStatus), (status = 500, body = ErrorBody)
))]
async fn status(State(a): State<App>, h: HeaderMap) -> Result<Json<AuthStatus>> {
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM users")
        .fetch_one(&a.pool)
        .await
        .map_err(db)?;
    let user = match auth::user(&a, &h).await.ok() {
        Some(value) => Some(super::decode(value)?),
        None => None,
    };
    Ok(Json(AuthStatus {
        setup_required: count == 0,
        user,
    }))
}

#[utoipa::path(post, path = "/auth/setup", request_body = Credentials,
    params(("X-Oxevu-Setup-Token" = String, Header, description = "Local one-time bootstrap token")),
    responses((status = 200, body = UserEnvelope), (status = 400, body = ErrorBody),
        (status = 403, body = ErrorBody), (status = 429, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn setup(
    State(a): State<App>,
    h: HeaderMap,
    super::ApiJson(body): super::ApiJson<Credentials>,
) -> Result<Response> {
    auth::setup(State(a), h, Json(serde_json::to_value(body).map_err(bad)?)).await
}

#[utoipa::path(post, path = "/auth/login", request_body = Credentials,
    responses((status = 200, body = UserEnvelope), (status = 400, body = ErrorBody),
        (status = 429, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn login(
    State(a): State<App>,
    super::ApiJson(body): super::ApiJson<Credentials>,
) -> Result<Response> {
    auth::login(State(a), Json(serde_json::to_value(body).map_err(bad)?)).await
}

#[utoipa::path(post, path = "/auth/logout", responses(
    (status = 204), (status = 401, body = ErrorBody), (status = 403, body = ErrorBody),
    (status = 500, body = ErrorBody)
))]
async fn logout(State(a): State<App>, h: HeaderMap) -> Result<Response> {
    auth::logout(State(a), h).await
}

#[utoipa::path(put, path = "/auth/password", request_body = PasswordChange,
    responses((status = 204), (status = 400, body = ErrorBody), (status = 401, body = ErrorBody),
        (status = 403, body = ErrorBody), (status = 429, body = ErrorBody), (status = 500, body = ErrorBody)))]
async fn password(
    State(a): State<App>,
    h: HeaderMap,
    super::ApiJson(body): super::ApiJson<PasswordChange>,
) -> Result<StatusCode> {
    auth::password(State(a), h, Json(serde_json::to_value(body).map_err(bad)?)).await
}
