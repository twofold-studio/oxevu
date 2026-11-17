use crate::*;

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
    ) && r.headers().get("origin").and_then(|v| v.to_str().ok()) != Some(&a.origin)
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
