use crate::*;

pub(crate) fn router(app: App) -> Router {
    let api = Router::new()
        .merge(acquisition::routes())
        .merge(parties::routes())
        .route("/playback/capabilities", get(playback::capabilities))
        .route("/account/avatar", axum::routing::put(avatars::save))
        .route("/playback/sessions", post(playback::create))
        .route("/playback/sessions/{id}", get(playback::get))
        .route(
            "/playback/sessions/{id}/heartbeat",
            post(playback::heartbeat),
        )
        .route("/playback/sessions/{id}/stop", post(playback::stop))
        .route("/playback/sessions/{id}/hls/{file}", get(playback::hls))
        .route("/users", get(users::list).post(users::create))
        .route("/users/{id}", put(users::update))
        .route("/auth/password", put(auth::password))
        .route("/libraries/{id}", put(libraries::update))
        .route("/libraries/paths/suggest", post(library_paths::suggest))
        .route("/libraries/{id}/remove", post(libraries::remove))
        .route("/settings", get(settings::get).put(settings::put))
        .route(
            "/integrations",
            get(integrations::get).put(integrations::put),
        )
        .route("/items/{id}/metadata/search", get(integrations::search))
        .route("/items/{id}/metadata/match", post(integrations::apply))
        .route(
            "/items/{id}/metadata/artwork/{kind}",
            get(integrations::metadata_artwork),
        )
        .route(
            "/items/{id}/metadata/studios/{name}",
            get(integrations::studio),
        )
        .route(
            "/items/{id}/subtitles/search",
            get(integrations::subtitle_search),
        )
        .route(
            "/items/{id}/subtitles/download",
            post(integrations::subtitle_download),
        )
        .route("/auth/status", get(status))
        .route("/auth/setup", post(setup))
        .route("/auth/login", post(login))
        .route("/auth/logout", post(logout))
        .route("/libraries", get(libraries).post(add_library))
        .route("/libraries/{id}/scan", post(scan::scan))
        .route("/items", get(items))
        .route("/items/{id}", get(item))
        .route("/items/{id}/favorite", put(catalog::favorite))
        .route("/items/{id}/reaction", put(media_files::reaction))
        .route("/library/removals/preview", post(media_files::preview))
        .route("/library/removals", post(media_files::apply))
        .route("/items/{id}/stream", get(stream))
        .route("/items/{id}/subtitles", get(subtitles::list))
        .route("/items/{id}/subtitles/{track}", get(subtitles::serve))
        .route("/items/{id}/artwork/{kind}", get(artwork::serve))
        .route("/items/{id}/progress", put(progress));
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
