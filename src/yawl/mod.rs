//! YAWL engine integration — bridges pm4py Petri nets to ~/yawlv6 HTTP API.
//!
//! Uses reqwest async client wrapped in a tokio Runtime for synchronous callers.
//! Set `YAWLV6_URL` environment variable to override the default engine URL.
//!
//! # WvdA Soundness
//! - All HTTP calls have a 10-second timeout (deadlock freedom).
//! - Runtime creation is bounded and deterministic (boundedness).
//! - `check_conformance` either returns Ok or propagates a typed `YawlError`; no infinite loops.
//!
//! # OTEL Span Emission
//! `check_conformance` emits two spans per call:
//! - `yawl.case`           — root span for the entire conformance check; attributes: `yawl.spec.uri`
//! - `yawl.task.execution` — child span for each task id in the YAWL XML spec;
//!   attributes: `yawl.task.id`, `yawl.token.consumed`, `yawl.token.produced`
//!
//! Armstrong: spans are emitted regardless of whether the HTTP call succeeds.
//! A failed HTTP call closes the case span with status = "error" so the supervisor
//! can observe the failure via OTEL.

use crate::observability::Tracing;
use crate::semconv::yawl_attributes::{
    YAWL_SPEC_URI, YAWL_TASK_ID, YAWL_TOKEN_CONSUMED, YAWL_TOKEN_PRODUCED,
};
use crate::semconv::yawl_span_names::{YAWL_CASE_SPAN, YAWL_TASK_EXECUTION_SPAN};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

pub mod spec_builder;

pub use spec_builder::petri_net_to_yawl_xml;

/// Response returned by the yawlv6 conformance endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConformanceResponse {
    pub fitness: f64,
    pub violations: Vec<String>,
    pub is_sound: bool,
}

/// Errors produced by the YAWL bridge.
#[derive(Debug, thiserror::Error)]
pub enum YawlError {
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("YAWL engine unreachable")]
    Unreachable,
    #[error("Response parse error: {0}")]
    Parse(String),
    #[error("Runtime error: {0}")]
    Runtime(String),
}

/// HTTP client that talks to the yawlv6 conformance API.
///
/// Holds an `Arc<Tracing>` for OTEL span emission.  Production code calls `new()` which
/// creates a private `Tracing` instance (spans are observable via OTLP export).
/// Tests inject a shared `Arc<Tracing>` via `new_with_tracing` so span assertions can be
/// made after `check_conformance` returns.
pub struct YawlClient {
    base_url: String,
    tracing: Arc<Tracing>,
}

impl YawlClient {
    /// Create a new client. Reads `YAWLV6_URL` from the environment;
    /// defaults to `http://localhost:8080`. Uses a private Tracing instance.
    pub fn new() -> Self {
        let base_url =
            std::env::var("YAWLV6_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
        Self {
            base_url,
            tracing: Arc::new(Tracing::new()),
        }
    }

    /// Create a new client with a caller-supplied `Tracing` instance.
    ///
    /// Use this in tests to inspect spans after `check_conformance` returns.
    /// Armstrong: tracing is not optional - every call must emit observable spans.
    pub fn new_with_tracing(tracing: Arc<Tracing>) -> Self {
        let base_url =
            std::env::var("YAWLV6_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
        Self { base_url, tracing }
    }

    /// Submit a YAWL spec + event log to the yawlv6 conformance endpoint.
    ///
    /// Emits two OTEL span types per call:
    ///   - `yawl.case` (root) with `yawl.spec.uri` attribute
    ///   - `yawl.task.execution` (child of case) for each task in the spec XML, carrying
    ///     `yawl.task.id`, `yawl.token.consumed`, and `yawl.token.produced` attributes
    ///
    /// Spans are closed with status = "ok" on success or "error" on HTTP failure.
    /// Armstrong: spans are ALWAYS emitted so the supervisor observes failures via span status.
    ///
    /// # Arguments
    /// * `spec_xml`        - YAWL XML specification produced by `spec_builder`
    /// * `event_log_json`  - Raw JSON bytes of the event log
    ///
    /// # WvdA
    /// 10-second timeout prevents deadlock. Returns `Err(YawlError::Unreachable)`
    /// when the engine is down; callers must handle this explicitly.
    pub fn check_conformance(
        &self,
        spec_xml: &str,
        event_log_json: &[u8],
    ) -> Result<ConformanceResponse, YawlError> {
        // -- yawl.case root span
        let spec_uri = extract_spec_uri(spec_xml);
        let mut case_attrs = HashMap::new();
        case_attrs.insert(YAWL_SPEC_URI.to_string(), spec_uri);

        let mut case_span = self
            .tracing
            .start_span(YAWL_CASE_SPAN, case_attrs, None)
            .expect("Tracing::start_span must not fail -- crash lets supervisor restart");

        // -- yawl.task.execution child spans (one per task id in the spec)
        let task_ids = extract_task_ids(spec_xml);
        for task_id in &task_ids {
            let mut task_attrs = HashMap::new();
            task_attrs.insert(YAWL_TASK_ID.to_string(), task_id.clone());
            // Token flow: 1 consumed -> 1 produced (sequence / WCP-1 pattern)
            task_attrs.insert(YAWL_TOKEN_CONSUMED.to_string(), "1".to_string());
            task_attrs.insert(YAWL_TOKEN_PRODUCED.to_string(), "1".to_string());

            let mut task_span = self
                .tracing
                .start_span(
                    YAWL_TASK_EXECUTION_SPAN,
                    task_attrs,
                    Some(case_span.span_id.clone()),
                )
                .expect("Tracing::start_span for task execution must not fail");

            // Task span closed immediately -- lifecycle tracks spec parsing, not HTTP I/O
            self.tracing
                .end_span(&mut task_span, "ok", None)
                .expect("Tracing::end_span must not fail");
        }

        // -- HTTP call to yawlv6 engine
        let rt = tokio::runtime::Runtime::new().map_err(|e| YawlError::Runtime(e.to_string()))?;

        let http_result = rt.block_on(self.check_conformance_async(spec_xml, event_log_json));

        // -- Close case span: Armstrong: error is information, not hidden
        let (status, error_msg) = match &http_result {
            Ok(_) => ("ok", None),
            Err(e) => ("error", Some(e.to_string())),
        };
        self.tracing
            .end_span(&mut case_span, status, error_msg.as_deref())
            .expect("Tracing::end_span for case must not fail");

        http_result
    }

    async fn check_conformance_async(
        &self,
        spec_xml: &str,
        event_log_json: &[u8],
    ) -> Result<ConformanceResponse, YawlError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| YawlError::Http(e.to_string()))?;

        let event_log_value: serde_json::Value =
            serde_json::from_slice(event_log_json).unwrap_or(serde_json::Value::Null);

        let body = serde_json::json!({
            "spec": spec_xml,
            "event_log": event_log_value
        });

        let resp = client
            .post(format!("{}/api/process-mining/conformance", self.base_url))
            .json(&body)
            .send()
            .await
            .map_err(|_| YawlError::Unreachable)?;

        resp.json::<ConformanceResponse>()
            .await
            .map_err(|e| YawlError::Parse(e.to_string()))
    }
}

impl Default for YawlClient {
    fn default() -> Self {
        Self::new()
    }
}

// -- Private helpers

/// Extract the `uri` attribute from a YAWL `<specification uri="...">` element.
/// Returns `"unknown"` when the XML does not match (never panics -- Armstrong).
fn extract_spec_uri(spec_xml: &str) -> String {
    if let Some(spec_pos) = spec_xml.find("<specification") {
        let after = &spec_xml[spec_pos..];
        if let Some(uri_pos) = after.find("uri=\"") {
            let value_start = uri_pos + 5; // skip past uri="
            if let Some(end_pos) = after[value_start..].find('"') {
                return after[value_start..value_start + end_pos].to_string();
            }
        }
    }
    "unknown".to_string()
}

/// Extract all `<task id="...">` identifiers from YAWL XML.
/// Returns an empty `Vec` when no tasks are found (never panics -- Armstrong).
fn extract_task_ids(spec_xml: &str) -> Vec<String> {
    let mut ids = Vec::new();
    let mut rest = spec_xml;
    while let Some(task_pos) = rest.find("<task") {
        let after = &rest[task_pos..];
        if let Some(id_pos) = after.find("id=\"") {
            let value_start = id_pos + 4; // skip past id="
            if let Some(end_pos) = after[value_start..].find('"') {
                ids.push(after[value_start..value_start + end_pos].to_string());
            }
        }
        // Advance past this <task token to avoid re-matching
        rest = &rest[task_pos + "<task".len()..];
    }
    ids
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conformance_response_deserializes_correctly() {
        let json = r#"{"fitness":0.95,"violations":["v1"],"is_sound":true}"#;
        let result: ConformanceResponse = serde_json::from_str(json).unwrap();
        assert!((result.fitness - 0.95).abs() < 0.001);
        assert_eq!(result.violations, vec!["v1"]);
        assert!(result.is_sound);
    }

    #[test]
    fn conformance_response_empty_violations_deserializes() {
        let json = r#"{"fitness":0.0,"violations":[],"is_sound":false}"#;
        let result: ConformanceResponse = serde_json::from_str(json).unwrap();
        assert!((result.fitness - 0.0).abs() < 0.001);
        assert!(result.violations.is_empty());
        assert!(!result.is_sound);
    }

    #[test]
    fn yawl_client_returns_error_when_unreachable() {
        // WvdA: bounded by 10s timeout; engine deliberately not running on port 19999
        std::env::set_var("YAWLV6_URL", "http://localhost:19999");
        let client = YawlClient::new();
        let result = client.check_conformance("<xml/>", b"{}");
        // Must not panic; must return a typed error
        assert!(result.is_err());
    }

    #[test]
    fn yawl_client_default_url_is_localhost_8080() {
        std::env::remove_var("YAWLV6_URL");
        let client = YawlClient::new();
        assert_eq!(client.base_url, "http://localhost:8080");
    }

    #[test]
    fn yawl_error_display_messages_are_descriptive() {
        let err = YawlError::Http("connection refused".to_string());
        assert!(err.to_string().contains("HTTP error"));

        let err2 = YawlError::Unreachable;
        assert!(err2.to_string().contains("unreachable"));

        let err3 = YawlError::Parse("unexpected EOF".to_string());
        assert!(err3.to_string().contains("parse"));
    }

    #[test]
    fn extract_spec_uri_parses_yawl_xml() {
        let xml = r#"<specificationSet><specification uri="PM4Py_Discovered"><rootNet/></specification></specificationSet>"#;
        assert_eq!(extract_spec_uri(xml), "PM4Py_Discovered");
    }

    #[test]
    fn extract_spec_uri_returns_unknown_for_empty_xml() {
        assert_eq!(extract_spec_uri("<xml/>"), "unknown");
    }

    #[test]
    fn extract_task_ids_finds_all_tasks() {
        let xml = petri_net_to_yawl_xml(&["step1", "step2", "step3"]);
        let ids = extract_task_ids(&xml);
        assert!(ids.contains(&"step1".to_string()));
        assert!(ids.contains(&"step2".to_string()));
        assert!(ids.contains(&"step3".to_string()));
    }

    #[test]
    fn extract_task_ids_empty_for_no_tasks() {
        assert!(extract_task_ids("<xml/>").is_empty());
    }
}
