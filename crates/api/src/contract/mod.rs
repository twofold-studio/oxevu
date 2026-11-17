//! REST wire contracts and offline OpenAPI export.
mod account;
pub(crate) mod acquisition_catalog_model;
pub(crate) mod acquisition_transfer_model;
mod admin;
mod admin_model;
pub(crate) mod auth;
mod catalog;
mod integration_model;
mod integrations;
mod item;
mod json;
mod library_operations;
mod library_operations_model;
mod media;
pub(crate) use json::{ApiJson, ApiQuery};
pub(crate) mod libraries;
pub(crate) mod parties;
mod party_model;
mod playback;
mod playback_model;
#[cfg(test)]
mod tests;

pub(crate) use account::{AuthStatus, Credentials, PasswordChange, UserEnvelope};

#[derive(serde::Serialize, utoipa::ToSchema)]
pub(crate) struct ErrorBody {
    pub error: String,
}

/// Export the contract without opening a database or starting the service.
pub fn openapi() -> utoipa::openapi::OpenApi {
    use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityRequirement, SecurityScheme};
    let mut document = router().split_for_parts().1;
    document.servers = Some(vec![utoipa::openapi::Server::new("/api/v1")]);
    document
        .components
        .get_or_insert_with(Default::default)
        .add_security_scheme(
            "session",
            SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new("oxevu_session"))),
        );
    document.security = Some(vec![SecurityRequirement::new(
        "session",
        Vec::<String>::new(),
    )]);
    for (path, post) in [
        ("/auth/status", false),
        ("/auth/login", true),
        ("/auth/setup", true),
    ] {
        if let Some(item) = document.paths.paths.get_mut(path) {
            let operation = if post {
                item.post.as_mut()
            } else {
                item.get.as_mut()
            };
            if let Some(operation) = operation {
                operation.security = Some(Vec::new());
            }
        }
    }
    document
}

pub(crate) fn router() -> utoipa_axum::router::OpenApiRouter<crate::App> {
    auth::router()
        .merge(libraries::router())
        .merge(catalog::router())
        .merge(playback::router())
        .merge(parties::router())
        .merge(admin::router())
        .merge(integrations::router())
        .merge(library_operations::router())
        .merge(crate::acquisition::http::router())
        .merge(media::router())
}

pub(crate) fn decode<T: serde::de::DeserializeOwned>(value: serde_json::Value) -> crate::Result<T> {
    serde_json::from_value(value).map_err(|error| {
        eprintln!("API response violates its contract: {error}");
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({"error":"invalid server response"})),
        )
    })
}
