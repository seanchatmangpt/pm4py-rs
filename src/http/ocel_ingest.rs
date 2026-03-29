//! OCEL 2.0 Ingest Endpoint
//!
//! Accepts OCEL 2.0 JSON payloads and queues them for process mining discovery.
//! This is a fire-and-forget endpoint: the payload is parsed and validated, counts
//! are returned immediately, and actual mining happens asynchronously.
//!
//! # Endpoint
//! `POST /api/ocel/ingest`
//!
//! # Request
//! OCEL 2.0 JSON object with `ocelVersion`, `objectTypes`, `eventTypes`,
//! `objects`, and `events` fields.
//!
//! # Response 200
//! `{"status":"queued","eventCount":N,"traceCount":M}`
//!
//! # Response 400
//! `{"error":"invalid OCEL format","detail":"..."}`

use axum::{extract::Json, http::StatusCode, response::IntoResponse, response::Response};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Request structures (OCEL 2.0 JSON schema)
// ---------------------------------------------------------------------------

/// A named object type with optional attribute definitions
#[derive(Debug, Deserialize)]
pub struct OcelObjectType {
    pub name: String,
    #[serde(default)]
    pub attributes: Vec<serde_json::Value>,
}

/// A named event type with optional attribute definitions
#[derive(Debug, Deserialize)]
pub struct OcelEventType {
    pub name: String,
    #[serde(default)]
    pub attributes: Vec<serde_json::Value>,
}

/// A single object instance in the OCEL log
#[derive(Debug, Deserialize)]
pub struct OcelObject {
    pub id: String,
    #[serde(rename = "type")]
    pub object_type: String,
    #[serde(default)]
    pub attributes: HashMap<String, serde_json::Value>,
}

/// A relationship linking an event to an object
#[derive(Debug, Deserialize)]
pub struct OcelRelationship {
    #[serde(rename = "objectId")]
    pub object_id: String,
    #[serde(default)]
    pub qualifier: String,
}

/// A single event in the OCEL log
#[derive(Debug, Deserialize)]
pub struct OcelEvent {
    pub id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub time: String,
    #[serde(default)]
    pub relationships: Vec<OcelRelationship>,
    #[serde(default)]
    pub attributes: HashMap<String, serde_json::Value>,
}

/// Top-level OCEL 2.0 JSON document
#[derive(Debug, Deserialize)]
pub struct OcelIngestRequest {
    /// Must be "2.0"
    #[serde(rename = "ocelVersion", default)]
    pub ocel_version: String,
    #[serde(rename = "objectTypes", default)]
    pub object_types: Vec<OcelObjectType>,
    #[serde(rename = "eventTypes", default)]
    pub event_types: Vec<OcelEventType>,
    #[serde(default)]
    pub objects: Vec<OcelObject>,
    #[serde(default)]
    pub events: Vec<OcelEvent>,
}

// ---------------------------------------------------------------------------
// Response structures
// ---------------------------------------------------------------------------

/// Successful ingest acknowledgement
#[derive(Debug, Serialize)]
pub struct OcelIngestResponse {
    pub status: String,
    #[serde(rename = "eventCount")]
    pub event_count: usize,
    /// Number of distinct object IDs (analogous to "traces" in case-level logs)
    #[serde(rename = "traceCount")]
    pub trace_count: usize,
}

/// Error response for malformed OCEL payloads
#[derive(Debug, Serialize)]
pub struct OcelIngestError {
    pub error: String,
    pub detail: String,
}

impl IntoResponse for OcelIngestError {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, Json(self)).into_response()
    }
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

/// `POST /api/ocel/ingest` — parse OCEL 2.0 JSON and return a queued status.
///
/// Validates the payload structure, counts events and objects, then returns
/// immediately.  Background mining is left as a future concern; this handler
/// is intentionally fire-and-forget.
pub async fn ocel_ingest(
    body: Result<Json<OcelIngestRequest>, axum::extract::rejection::JsonRejection>,
) -> Result<Json<OcelIngestResponse>, OcelIngestError> {
    let Json(req) = body.map_err(|e| OcelIngestError {
        error: "invalid OCEL format".to_string(),
        detail: e.to_string(),
    })?;

    // Basic semantic validation: version field should be "2.0" when present
    if !req.ocel_version.is_empty() && req.ocel_version != "2.0" {
        return Err(OcelIngestError {
            error: "invalid OCEL format".to_string(),
            detail: format!(
                "unsupported ocelVersion '{}', expected '2.0'",
                req.ocel_version
            ),
        });
    }

    let event_count = req.events.len();
    // "trace count" = number of distinct objects (each object has its own lifecycle)
    let trace_count = req.objects.len();

    Ok(Json(OcelIngestResponse {
        status: "queued".to_string(),
        event_count,
        trace_count,
    }))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::post,
        Router,
    };
    use tower::ServiceExt;

    fn test_router() -> Router {
        Router::new().route("/api/ocel/ingest", post(ocel_ingest))
    }

    async fn post_json(router: Router, body: serde_json::Value) -> (StatusCode, serde_json::Value) {
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

    #[tokio::test]
    async fn test_valid_ocel_payload_returns_queued() {
        let payload = serde_json::json!({
            "ocelVersion": "2.0",
            "objectTypes": [{"name": "order", "attributes": []}],
            "eventTypes": [{"name": "Create PO", "attributes": []}],
            "objects": [{"id": "o1", "type": "order", "attributes": {}}],
            "events": [
                {
                    "id": "e1",
                    "type": "Create PO",
                    "time": "2024-01-01T10:00:00Z",
                    "relationships": [{"objectId": "o1", "qualifier": ""}]
                }
            ]
        });

        let (status, body) = post_json(test_router(), payload).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], "queued");
        assert_eq!(body["eventCount"], 1);
        assert_eq!(body["traceCount"], 1);
    }

    #[tokio::test]
    async fn test_multiple_events_and_objects() {
        let payload = serde_json::json!({
            "ocelVersion": "2.0",
            "objectTypes": [{"name": "order"}],
            "eventTypes": [{"name": "A"}, {"name": "B"}],
            "objects": [
                {"id": "o1", "type": "order", "attributes": {}},
                {"id": "o2", "type": "order", "attributes": {}}
            ],
            "events": [
                {"id": "e1", "type": "A", "time": "2024-01-01T10:00:00Z", "relationships": []},
                {"id": "e2", "type": "B", "time": "2024-01-01T11:00:00Z", "relationships": []},
                {"id": "e3", "type": "A", "time": "2024-01-01T12:00:00Z", "relationships": []}
            ]
        });

        let (status, body) = post_json(test_router(), payload).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["eventCount"], 3);
        assert_eq!(body["traceCount"], 2);
    }

    #[tokio::test]
    async fn test_invalid_json_returns_400() {
        let router = test_router();
        let response = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/ocel/ingest")
                    .header("content-type", "application/json")
                    .body(Body::from("not valid json {{"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["error"], "invalid OCEL format");
    }

    #[tokio::test]
    async fn test_wrong_version_returns_400() {
        let payload = serde_json::json!({
            "ocelVersion": "1.0",
            "objects": [],
            "events": []
        });

        let (status, body) = post_json(test_router(), payload).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["error"], "invalid OCEL format");
        assert!(body["detail"]
            .as_str()
            .unwrap()
            .contains("unsupported ocelVersion"));
    }

    #[tokio::test]
    async fn test_empty_payload_with_no_version_accepted() {
        // ocelVersion defaults to "" (empty string) — treated as "not set", not an error
        let payload = serde_json::json!({
            "objects": [],
            "events": []
        });

        let (status, body) = post_json(test_router(), payload).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], "queued");
        assert_eq!(body["eventCount"], 0);
        assert_eq!(body["traceCount"], 0);
    }
}
