use axum::{
    http::{StatusCode, Uri},
    response::{IntoResponse, Response},
};
include!(concat!(env!("OUT_DIR"), "/assets.rs"));
pub async fn serve(uri: Uri) -> Response {
    let key = uri.path().trim_start_matches('/');
    let asset = ASSETS
        .iter()
        .find(|(p, _)| *p == key)
        .or_else(|| ASSETS.iter().find(|(p, _)| *p == "index.html"));
    match asset {
        Some((p, bytes)) => {
            let mime = if p.ends_with(".js") {
                "text/javascript"
            } else if p.ends_with(".css") {
                "text/css"
            } else if p.ends_with(".svg") {
                "image/svg+xml"
            } else if p.ends_with(".woff2") {
                "font/woff2"
            } else if p.ends_with(".png") {
                "image/png"
            } else {
                "text/html; charset=utf-8"
            };
            ([(axum::http::header::CONTENT_TYPE, mime)], *bytes).into_response()
        }
        None => (
            StatusCode::SERVICE_UNAVAILABLE,
            "Build apps/web/dist and rebuild server to enable the web UI.",
        )
            .into_response(),
    }
}
