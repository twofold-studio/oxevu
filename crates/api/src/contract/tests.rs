use super::*;
use crate::*;
use axum::body::{Body, to_bytes};
use tower::ServiceExt;

#[test]
fn committed_contract_matches_the_registered_handlers() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../apps/web/src/lib/api/openapi.json");
    let committed: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(
        serde_json::to_value(openapi()).unwrap(),
        committed,
        "run just api-generate after changing API contracts"
    );
}

#[tokio::test]
async fn authentication_status_matches_real_response() {
    let app = crate::routes::router(crate::tests::app().await);
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let value: Value =
        serde_json::from_slice(&to_bytes(response.into_body(), 65536).await.unwrap()).unwrap();
    assert!(value["setup_required"].as_bool().is_some());
    assert!(value.get("user").is_some_and(Value::is_null));
}

#[tokio::test]
async fn catalog_contract_accepts_stored_records_and_rejects_invalid_progress() {
    let a = crate::tests::app().await;
    let headers = crate::tests::account(&a, "owner", true).await;
    sqlx::query("INSERT INTO libraries VALUES('movies','Movies','/virtual','movies')")
        .execute(&a.pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO items VALUES('film','movies','/virtual/film.mkv',?)")
        .bind(json!({"id":"film","library_id":"movies","title":"Film","kind":"movie"}).to_string())
        .execute(&a.pool)
        .await
        .unwrap();
    let origin = a.origin.clone();
    let app = crate::routes::router(a);
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/items/film")
                .header("cookie", &headers["cookie"])
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let value: Value =
        serde_json::from_slice(&to_bytes(response.into_body(), 65536).await.unwrap()).unwrap();
    let _: super::catalog::ItemEnvelope = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(value["item"]["title"], "Film");
    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/v1/items/film/progress")
                .header("cookie", &headers["cookie"])
                .header("origin", origin)
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"position_seconds":"wrong","completed":false}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let value: Value =
        serde_json::from_slice(&to_bytes(response.into_body(), 65536).await.unwrap()).unwrap();
    assert!(value["error"].is_string());
}

#[test]
fn generated_operations_have_concrete_json_schemas() {
    let value = serde_json::to_value(openapi()).unwrap();
    let mut operation_ids = std::collections::HashSet::new();
    let binary_paths = [
        "/items/{id}/stream",
        "/playback/sessions/{id}/hls/{file}",
        "/items/{id}/subtitles/{track}",
        "/items/{id}/artwork/{kind}",
        "/items/{id}/metadata/artwork/{kind}",
        "/items/{id}/metadata/studios/{name}",
        "/acquisition/artwork/{file}",
    ];
    for (path, item) in value["paths"].as_object().unwrap() {
        for (_, operation) in item.as_object().unwrap() {
            if operation["responses"].get("200").is_some() && !binary_paths.contains(&path.as_str())
            {
                assert!(
                    operation
                        .pointer("/responses/200/content/application~1json/schema")
                        .is_some(),
                    "missing JSON response schema: {path}"
                );
            }
            assert!(
                operation_ids.insert(operation["operationId"].as_str().unwrap()),
                "duplicate operation ID: {path}"
            );
            for response in operation["responses"].as_object().unwrap().values() {
                if let Some(schema) = response.pointer("/content/application~1json/schema") {
                    assert!(
                        schema.get("$ref").is_some() || schema.get("type").is_some(),
                        "untyped response: {path}"
                    );
                }
            }
        }
    }
    for (name, schema) in value["components"]["schemas"].as_object().unwrap() {
        concrete_schema(schema, name);
    }
}

#[test]
fn all_local_schema_references_resolve() {
    fn check(node: &serde_json::Value, document: &serde_json::Value) {
        match node {
            serde_json::Value::Object(fields) => {
                if let Some(reference) = fields
                    .get("$ref")
                    .and_then(|v| v.as_str())
                    .and_then(|v| v.strip_prefix('#'))
                {
                    assert!(
                        document.pointer(reference).is_some(),
                        "unresolved schema reference: {reference}"
                    );
                }
                for value in fields.values() {
                    check(value, document);
                }
            }
            serde_json::Value::Array(values) => {
                for value in values {
                    check(value, document);
                }
            }
            _ => {}
        }
    }
    let document = serde_json::to_value(openapi()).unwrap();
    check(&document, &document);
}

fn concrete_schema(schema: &Value, location: &str) {
    let object = schema.as_object().expect("schema must be an object");
    assert!(!object.is_empty(), "untyped schema at {location}");
    if let Some(properties) = schema["properties"].as_object() {
        for (name, property) in properties {
            concrete_schema(property, &format!("{location}.{name}"));
        }
    }
    if let Some(items) = schema.get("items") {
        concrete_schema(items, location);
    }
    for combination in ["allOf", "oneOf", "anyOf"] {
        if let Some(options) = schema[combination].as_array() {
            for option in options {
                concrete_schema(option, location);
            }
        }
    }
}
