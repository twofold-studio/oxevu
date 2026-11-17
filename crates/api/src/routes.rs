use crate::*;

pub(crate) fn parse_origins(raw: &str) -> std::result::Result<Vec<String>, StartupError> {
    if raw.len() > 65536 {
        return Err(StartupError::InvalidOrigins);
    }
    let values: Vec<String> =
        serde_json::from_str(raw).map_err(|_| StartupError::InvalidOrigins)?;
    if values.len() > 32 {
        return Err(StartupError::InvalidOrigins);
    }
    values
        .into_iter()
        .map(|value| {
            let url = reqwest::Url::parse(&value).map_err(|_| StartupError::InvalidOrigins)?;
            if !matches!(url.scheme(), "http" | "https")
                || url.host_str().is_none_or(|h| h.contains('*'))
                || !url.username().is_empty()
                || url.password().is_some()
                || url.path() != "/"
                || url.query().is_some()
                || url.fragment().is_some()
            {
                return Err(StartupError::InvalidOrigins);
            }
            Ok(url.origin().ascii_serialization())
        })
        .collect()
}

pub(crate) fn router(app: App) -> Router {
    let api = contract::router().split_for_parts().0;
    Router::new()
        .nest("/api/v1", api)
        .route("/healthz", get(|| async { "ok" }))
        .fallback(assets::serve)
        .layer(middleware::from_fn_with_state(app.clone(), origin_check))
        .with_state(app)
}

pub(crate) async fn origin_check(State(a): State<App>, r: Request, n: Next) -> Response {
    if !matches!(
        *r.method(),
        axum::http::Method::GET | axum::http::Method::HEAD
    ) && !r
        .headers()
        .get("origin")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|origin| {
            origin == a.origin || a.allowed_origins.iter().any(|trusted| trusted == origin)
        })
    {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error":"same-origin Origin header required"})),
        )
            .into_response();
    }
    n.run(r).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use tower::ServiceExt;

    #[test]
    fn trusted_origins_are_explicit_and_canonical() {
        assert_eq!(
            parse_origins(r#"["http://192.168.0.2:8096/","http://vd.netbird.cloud:8096"]"#)
                .unwrap(),
            vec!["http://192.168.0.2:8096", "http://vd.netbird.cloud:8096"]
        );
        for invalid in [
            r#"["*"]"#,
            r#"["http://user:password@example.com"]"#,
            r#"["https://example.com/path"]"#,
            r#"["https://example.com?query=1"]"#,
        ] {
            assert!(parse_origins(invalid).is_err());
        }
    }

    #[tokio::test]
    async fn network_origin_can_login_but_other_origins_remain_forbidden() {
        let mut a = crate::tests::app().await;
        a.allowed_origins = vec!["http://vd.netbird.cloud:8096".into()];
        let app = router(a);
        for (origin, allowed) in [
            ("http://vd.netbird.cloud:8096", true),
            ("https://evil.example", false),
        ] {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/api/v1/auth/logout")
                        .header("origin", origin)
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status() != StatusCode::FORBIDDEN, allowed);
        }
    }

    #[tokio::test]
    async fn composed_router_preserves_health_authentication_and_origin_checks() {
        let app = router(crate::tests::app().await);
        for (method, uri, expected) in [
            ("GET", "/healthz", StatusCode::OK),
            ("GET", "/api/v1/libraries", StatusCode::UNAUTHORIZED),
            ("POST", "/api/v1/auth/logout", StatusCode::FORBIDDEN),
        ] {
            let request = Request::builder()
                .method(method)
                .uri(uri)
                .body(Body::empty())
                .unwrap();
            assert_eq!(
                app.clone().oneshot(request).await.unwrap().status(),
                expected
            );
        }
    }
}
