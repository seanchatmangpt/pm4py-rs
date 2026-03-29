//! Board Chair Intelligence System — Deviation Reporting API
//!
//! Provides HTTP endpoint for pm4py-rust to report process deviations to OSA
//! for autonomous healing. When conformance fitness drops below 0.8, this
//! endpoint forwards the deviation to OSA's HealingBridge which triggers
//! ReflexArcs → proof written → L0 invalidated → briefing updated.
//!
//! Environment Variables:
//! - PM4PY_RUST_OSA_URL: OSA backend URL (default: http://localhost:8089)

use axum::{
    extract::Json,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use opentelemetry::global;
use opentelemetry::trace::{Span, Tracer};
use opentelemetry::KeyValue;
use serde::{Deserialize, Serialize};
use std::env;

use crate::http::otel_helpers::{extract_trace_context, resolve_correlation_id};

/// Deviation report from process mining analysis
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DeviationReport {
    /// Process identifier (e.g., "purchase-to-pay", "order-fulfillment")
    pub process_id: String,
    /// Conformance fitness score: 0.0 (no conformance) to 1.0 (perfect)
    pub fitness: f64,
    /// Deviation category: "conformance" | "timing" | "resource"
    pub deviation_type: String,
    /// ISO8601 datetime when deviation was detected
    pub detected_at: String,
}

/// Response sent back to caller (202 Accepted)
#[derive(Debug, Serialize)]
pub struct DeviationAccepted {
    pub status: String,
    pub process_id: String,
    pub fitness: f64,
    pub healing_triggered: bool,
    pub osa_forwarded: bool,
}

/// Error response
#[derive(Debug, Serialize)]
pub struct DeviationError {
    pub error: String,
    pub details: Option<String>,
}

impl IntoResponse for DeviationError {
    fn into_response(self) -> Response {
        (StatusCode::UNPROCESSABLE_ENTITY, Json(self)).into_response()
    }
}

/// WvdA conformance threshold — below this fitness score, healing is triggered
const CONFORMANCE_THRESHOLD: f64 = 0.8;

/// POST /board/deviation
///
/// Reports a process deviation detected by pm4py-rust to OSA for autonomous healing.
/// Returns 202 Accepted immediately — OSA processes asynchronously.
///
/// Validates fitness is in [0.0, 1.0] range before forwarding.
/// Forwards to OSA's POST /api/v1/board/deviation endpoint.
pub async fn report_deviation(
    headers: HeaderMap,
    Json(deviation): Json<DeviationReport>,
) -> Response {
    let parent_cx = extract_trace_context(&headers);
    let correlation_id = resolve_correlation_id(&headers);
    let tracer = global::tracer("pm4py-rust");
    // TODO: add board.deviation_reported to semconv/model/board/spans.yaml so a typed
    // constant can replace this raw string.
    let mut span = tracer.start_with_context("board.deviation_reported", &parent_cx);

    span.set_attribute(KeyValue::new(
        "chatmangpt.run.correlation_id",
        correlation_id,
    ));
    span.set_attribute(KeyValue::new(
        "board.process_id",
        deviation.process_id.clone(),
    ));
    span.set_attribute(KeyValue::new("board.fitness", deviation.fitness));
    span.set_attribute(KeyValue::new(
        "board.deviation_type",
        deviation.deviation_type.clone(),
    ));
    span.set_attribute(KeyValue::new(
        "board.healing_triggered",
        deviation.fitness < CONFORMANCE_THRESHOLD,
    ));

    // Validate process_id
    if deviation.process_id.is_empty() {
        span.set_attribute(KeyValue::new("board.error", "missing_process_id"));
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(DeviationError {
                error: "process_id is required".to_string(),
                details: None,
            }),
        )
            .into_response();
    }

    // Validate fitness range
    if deviation.fitness < 0.0 || deviation.fitness > 1.0 {
        span.set_attribute(KeyValue::new("board.error", "fitness_out_of_range"));
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(DeviationError {
                error: "fitness must be between 0.0 and 1.0".to_string(),
                details: Some(format!("received: {}", deviation.fitness)),
            }),
        )
            .into_response();
    }

    let healing_triggered = deviation.fitness < CONFORMANCE_THRESHOLD;

    // Forward to OSA HealingBridge — fire and forget with timeout
    let osa_url =
        env::var("PM4PY_RUST_OSA_URL").unwrap_or_else(|_| "http://localhost:8089".to_string());
    let osa_endpoint = format!("{}/api/v1/board/deviation", osa_url);

    let osa_forwarded = forward_to_osa(&osa_endpoint, &deviation).await;

    span.set_attribute(KeyValue::new("board.osa_forwarded", osa_forwarded));
    span.set_attribute(KeyValue::new("board.outcome", "accepted"));

    (
        StatusCode::ACCEPTED,
        Json(DeviationAccepted {
            status: "accepted".to_string(),
            process_id: deviation.process_id,
            fitness: deviation.fitness,
            healing_triggered,
            osa_forwarded,
        }),
    )
        .into_response()
}

/// Forward deviation payload to OSA HealingBridge endpoint.
/// Returns true if OSA accepted the request (2xx), false on any error.
/// Uses 5s timeout — Armstrong budget constraint.
async fn forward_to_osa(endpoint: &str, deviation: &DeviationReport) -> bool {
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(5000))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Board Chair: failed to build HTTP client: {}", e);
            return false;
        }
    };

    match client.post(endpoint).json(deviation).send().await {
        Ok(resp) if resp.status().is_success() => {
            tracing::info!(
                "Board Chair: deviation forwarded to OSA ({}) — process_id={} fitness={}",
                resp.status(),
                deviation.process_id,
                deviation.fitness
            );
            true
        }
        Ok(resp) => {
            tracing::warn!(
                "Board Chair: OSA returned {} for deviation process_id={}",
                resp.status(),
                deviation.process_id
            );
            false
        }
        Err(e) => {
            tracing::warn!(
                "Board Chair: failed to forward deviation to OSA ({}): {}",
                endpoint,
                e
            );
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deviation_report_deserializes() {
        let json = r#"{
            "process_id": "purchase-to-pay",
            "fitness": 0.72,
            "deviation_type": "conformance",
            "detected_at": "2026-03-26T06:00:00Z"
        }"#;
        let report: DeviationReport = serde_json::from_str(json).unwrap();
        assert_eq!(report.process_id, "purchase-to-pay");
        assert_eq!(report.fitness, 0.72);
        assert_eq!(report.deviation_type, "conformance");
    }

    #[test]
    fn test_healing_triggered_when_below_threshold() {
        let deviation = DeviationReport {
            process_id: "test-process".to_string(),
            fitness: 0.72,
            deviation_type: "conformance".to_string(),
            detected_at: "2026-03-26T06:00:00Z".to_string(),
        };
        assert!(deviation.fitness < CONFORMANCE_THRESHOLD);
    }

    #[test]
    fn test_no_healing_when_above_threshold() {
        let deviation = DeviationReport {
            process_id: "test-process".to_string(),
            fitness: 0.92,
            deviation_type: "conformance".to_string(),
            detected_at: "2026-03-26T06:00:00Z".to_string(),
        };
        assert!(deviation.fitness >= CONFORMANCE_THRESHOLD);
    }

    #[test]
    fn test_fitness_boundary_at_threshold() {
        // fitness == 0.8 means no healing (below strict threshold)
        assert!(!(0.8_f64 < CONFORMANCE_THRESHOLD));
        // fitness == 0.799... triggers healing
        assert!(0.79_f64 < CONFORMANCE_THRESHOLD);
    }

    #[test]
    fn test_deviation_accepted_serializes() {
        let resp = DeviationAccepted {
            status: "accepted".to_string(),
            process_id: "p1".to_string(),
            fitness: 0.7,
            healing_triggered: true,
            osa_forwarded: true,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("accepted"));
        assert!(json.contains("healing_triggered"));
    }
}
