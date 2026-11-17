pub(super) fn client() -> reqwest::Result<reqwest::Client> {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .redirect(reqwest::redirect::Policy::custom(|attempt| {
            // Providers canonicalize search query casing/order with redirects.
            // Keep application credentials on the original scheme/host/port.
            let same_origin = attempt
                .previous()
                .first()
                .is_some_and(|first| first.origin() == attempt.url().origin());
            if same_origin && attempt.previous().len() <= 3 {
                attempt.follow()
            } else {
                attempt.stop()
            }
        }))
        .user_agent("Oxevu/0.1.0")
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Router, http::HeaderMap, response::Redirect, routing::get};

    #[tokio::test]
    async fn canonical_redirects_preserve_keys_but_external_redirects_and_loops_stop() {
        let router = Router::new()
            .route(
                "/search",
                get(|| async { Redirect::permanent("/canonical?query=arcane") }),
            )
            .route(
                "/canonical",
                get(|headers: HeaderMap| async move {
                    assert_eq!(headers.get("api-key").unwrap(), "test-application-key");
                    "metadata result"
                }),
            )
            .route(
                "/external",
                get(|| async { Redirect::temporary("http://127.0.0.1:9/blocked") }),
            )
            .route("/loop", get(|| async { Redirect::temporary("/loop") }));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let task = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
        let client = client().unwrap();
        let response = client
            .get(format!("{base}/search"))
            .header("Api-Key", "test-application-key")
            .send()
            .await
            .unwrap();
        assert_eq!(response.text().await.unwrap(), "metadata result");
        for path in ["external", "loop"] {
            let response = client.get(format!("{base}/{path}")).send().await.unwrap();
            assert_eq!(response.status(), reqwest::StatusCode::TEMPORARY_REDIRECT);
        }
        task.abort();
        let _ = task.await;
    }
}
