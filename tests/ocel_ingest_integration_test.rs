//! Integration tests for the OCEL 2.0 ingest endpoint.
//!
//! Uses Axum's `tower::ServiceExt::oneshot` to drive the handler directly without
//! binding a TCP socket — same pattern used by the inline unit tests in `ocel_ingest.rs`.
//!
//! These tests verify:
//! - Valid OCEL 2.0 JSON with 3 events and 2 objects → 200 queued
//! - Invalid JSON body → 400 with `error` field
//! - Empty `events` array → 200 queued with eventCount = 0 / traceCount = 0

use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::post,
    Router,
};
use pm4py::http::ocel_ingest::ocel_ingest;
use tower::ServiceExt;

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn build_router() -> Router {
    Router::new().route("/api/ocel/ingest", post(ocel_ingest))
}

async fn post_json(body: serde_json::Value) -> (StatusCode, serde_json::Value) {
    let router = build_router();

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/ocel/ingest")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    (status, json)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Valid OCEL 2.0 payload with 3 events and 2 distinct objects.
/// Expected: 200 OK with status = "queued", eventCount = 3, traceCount = 2.
#[tokio::test]
async fn test_valid_ocel_three_events_two_objects_returns_queued() {
    let payload = serde_json::json!({
        "ocelVersion": "2.0",
        "objectTypes": [
            {"name": "order", "attributes": []},
            {"name": "item",  "attributes": []}
        ],
        "eventTypes": [
            {"name": "Create Order",    "attributes": []},
            {"name": "Pack Item",       "attributes": []},
            {"name": "Ship Order",      "attributes": []}
        ],
        "objects": [
            {"id": "order-1", "type": "order", "attributes": {}},
            {"id": "item-1",  "type": "item",  "attributes": {}}
        ],
        "events": [
            {
                "id": "e1",
                "type": "Create Order",
                "time": "2024-01-01T09:00:00Z",
                "relationships": [{"objectId": "order-1", "qualifier": ""}]
            },
            {
                "id": "e2",
                "type": "Pack Item",
                "time": "2024-01-01T10:00:00Z",
                "relationships": [{"objectId": "item-1", "qualifier": ""}]
            },
            {
                "id": "e3",
                "type": "Ship Order",
                "time": "2024-01-01T11:00:00Z",
                "relationships": [
                    {"objectId": "order-1", "qualifier": ""},
                    {"objectId": "item-1",  "qualifier": "contains"}
                ]
            }
        ]
    });

    let (status, body) = post_json(payload).await;

    assert_eq!(status, StatusCode::OK, "Expected HTTP 200, got {}", status);
    assert_eq!(body["status"], "queued", "status field should be 'queued'");
    assert_eq!(body["eventCount"], 3, "eventCount should be 3");
    assert_eq!(
        body["traceCount"], 2,
        "traceCount (object count) should be 2"
    );
}

/// Invalid JSON body (malformed syntax) → 400 with `error` field set.
#[tokio::test]
async fn test_invalid_json_returns_400_with_error_field() {
    let router = build_router();

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/ocel/ingest")
                .header("content-type", "application/json")
                .body(Body::from("{this is not valid JSON{{"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "Expected HTTP 400 for malformed JSON"
    );

    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

    assert!(
        json.get("error").is_some(),
        "Response should have an 'error' field, got: {:?}",
        json
    );
    assert_eq!(
        json["error"], "invalid OCEL format",
        "error field should be 'invalid OCEL format'"
    );
}

/// Empty events array with no version field.
/// Expected: 200 OK, eventCount = 0, traceCount = 0.
#[tokio::test]
async fn test_empty_events_returns_queued_with_zero_counts() {
    let payload = serde_json::json!({
        "objects": [],
        "events": []
    });

    let (status, body) = post_json(payload).await;

    assert_eq!(status, StatusCode::OK, "Expected HTTP 200, got {}", status);
    assert_eq!(body["status"], "queued");
    assert_eq!(
        body["eventCount"], 0,
        "eventCount should be 0 for empty events"
    );
    assert_eq!(
        body["traceCount"], 0,
        "traceCount should be 0 for empty objects"
    );
}

/// Empty events array with explicit ocelVersion = "2.0".
/// Expected: 200 OK, eventCount = 0, traceCount = 0.
#[tokio::test]
async fn test_empty_events_with_version_returns_queued() {
    let payload = serde_json::json!({
        "ocelVersion": "2.0",
        "objectTypes": [],
        "eventTypes": [],
        "objects": [],
        "events": []
    });

    let (status, body) = post_json(payload).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "queued");
    assert_eq!(body["eventCount"], 0);
    assert_eq!(body["traceCount"], 0);
}

/// Unsupported ocelVersion → 400 with error field containing version info.
#[tokio::test]
async fn test_unsupported_ocel_version_returns_400() {
    let payload = serde_json::json!({
        "ocelVersion": "1.0",
        "objects": [],
        "events": []
    });

    let (status, body) = post_json(payload).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "invalid OCEL format");

    let detail = body["detail"].as_str().unwrap_or("");
    assert!(
        detail.contains("unsupported ocelVersion"),
        "detail should mention unsupported version, got: {}",
        detail
    );
}

/// Single-object, single-event payload — basic smoke test for the happy path.
#[tokio::test]
async fn test_single_event_single_object() {
    let payload = serde_json::json!({
        "ocelVersion": "2.0",
        "objectTypes": [{"name": "case"}],
        "eventTypes":  [{"name": "Start"}],
        "objects": [{"id": "c1", "type": "case", "attributes": {}}],
        "events": [
            {
                "id": "ev1",
                "type": "Start",
                "time": "2024-06-01T08:00:00Z",
                "relationships": [{"objectId": "c1", "qualifier": ""}]
            }
        ]
    });

    let (status, body) = post_json(payload).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "queued");
    assert_eq!(body["eventCount"], 1);
    assert_eq!(body["traceCount"], 1);
}
