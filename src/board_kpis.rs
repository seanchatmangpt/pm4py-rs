//! Board KPI Pipeline — board-ready KPIs computed from process mining data.
//!
//! Connects pm4py-rust to the Board Intelligence system by computing 4 key
//! metrics from event logs:
//!
//! - `cycle_time_avg_ms`: Average case cycle time in milliseconds
//! - `conformance_score`: 0.0-1.0 conformance against a reference Petri net
//! - `bottleneck_count`: Number of detected bottleneck activities
//! - `variant_count`: Number of unique process variants
//!
//! WvdA Soundness:
//! - Bounded computation: max 10,000 events per batch
//! - Timeout: 5s tokio::time::timeout wraps all computation
//! - Safe defaults: empty/missing data returns zeroed response (no crash)

use serde::{Deserialize, Serialize};

use crate::conformance::TokenReplay;
use crate::discovery::AlphaMiner;
use crate::log::EventLog;
use crate::statistics::advanced::{
    case_duration_distribution, get_bottleneck_activities, get_variant_frequency,
};

/// Maximum number of events processed per batch (WvdA boundedness).
const MAX_EVENTS_PER_BATCH: usize = 10_000;

/// Computation timeout in milliseconds (WvdA deadlock freedom).
const COMPUTATION_TIMEOUT_MS: u64 = 5_000;

/// Minimum average duration (ms) for an activity to be classified as a bottleneck.
/// Activities slower than this threshold are counted in `bottleneck_count`.
const BOTTLENECK_THRESHOLD_MS: f64 = 1000.0;

/// Board KPI request — optional scoping to a specific event log.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BoardKpiRequest {
    /// Optional event log identifier to scope the computation.
    /// When absent, uses the most recent / default log.
    pub event_log_id: Option<String>,
}

/// Board KPI response — the 4 key metrics for the board briefing.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BoardKpiResponse {
    /// Average case cycle time in milliseconds.
    pub cycle_time_avg_ms: f64,
    /// Conformance score: 0.0 (no conformance) to 1.0 (perfect).
    pub conformance_score: f64,
    /// Number of detected bottleneck activities.
    pub bottleneck_count: usize,
    /// Number of unique process variants.
    pub variant_count: usize,
    /// Total number of events processed (capped at MAX_EVENTS_PER_BATCH).
    pub events_processed: usize,
    /// Whether the event log was truncated to respect the batch limit.
    pub truncated: bool,
}

impl Default for BoardKpiResponse {
    fn default() -> Self {
        Self {
            cycle_time_avg_ms: 0.0,
            conformance_score: 0.0,
            bottleneck_count: 0,
            variant_count: 0,
            events_processed: 0,
            truncated: false,
        }
    }
}

/// Truncate an EventLog to at most `max_events` total events.
///
/// Preserves complete traces (does not split a trace mid-way).
/// If adding the next trace would exceed the limit, stop.
/// Returns (truncated_log, total_events_kept, was_truncated).
pub fn truncate_event_log(log: &EventLog, max_events: usize) -> (EventLog, usize, bool) {
    let mut truncated = EventLog::new();
    let mut total_events: usize = 0;
    let original_total: usize = log.traces.iter().map(|t| t.events.len()).sum();

    for trace in &log.traces {
        if total_events + trace.events.len() > max_events {
            break;
        }
        truncated.add_trace(trace.clone());
        total_events += trace.events.len();
    }

    let was_truncated = total_events < original_total;
    (truncated, total_events, was_truncated)
}

/// Compute board KPIs from an EventLog.
///
/// Pure function (no I/O, no side effects). All inputs bounded.
/// Returns safe defaults for empty logs.
///
/// The event log is truncated to MAX_EVENTS_PER_BATCH before processing.
pub fn compute_board_kpis(log: &EventLog) -> BoardKpiResponse {
    if log.traces.is_empty() {
        return BoardKpiResponse::default();
    }

    // WvdA boundedness: truncate to max batch size
    let (bounded_log, events_processed, truncated) = truncate_event_log(log, MAX_EVENTS_PER_BATCH);

    if bounded_log.traces.is_empty() {
        return BoardKpiResponse {
            events_processed: 0,
            truncated,
            ..Default::default()
        };
    }

    // 1. Cycle time: average case duration in milliseconds
    let duration_stats = case_duration_distribution(&bounded_log);
    // case_duration_distribution returns seconds; convert to ms
    let cycle_time_avg_ms = duration_stats.mean_duration * 1000.0;

    // 2. Conformance score: discover a reference model then replay
    let conformance_score = compute_conformance(&bounded_log);

    // 3. Bottleneck count: activities with avg duration above threshold
    let bottlenecks = get_bottleneck_activities(&bounded_log, 100);
    let bottleneck_count = bottlenecks
        .iter()
        .filter(|(_, avg_duration_s)| {
            // get_bottleneck_activities returns durations in seconds; compare in ms
            (*avg_duration_s * 1000.0) > BOTTLENECK_THRESHOLD_MS
        })
        .count();

    // 4. Variant count: unique process execution paths
    let variants = get_variant_frequency(&bounded_log);
    let variant_count = variants.len();

    BoardKpiResponse {
        cycle_time_avg_ms,
        conformance_score,
        bottleneck_count,
        variant_count,
        events_processed,
        truncated,
    }
}

/// Compute conformance score by discovering an Alpha model and running token replay.
///
/// Returns 0.0 on any failure (safe default). Bounded by the already-truncated log.
fn compute_conformance(log: &EventLog) -> f64 {
    // Discover a reference Petri net using Alpha miner
    let miner = AlphaMiner::new();
    let net = miner.discover(log);

    // Run token replay for conformance fitness
    let checker = TokenReplay;
    let result = checker.check(log, &net);

    // Clamp to [0.0, 1.0] — defensive against any floating-point edge case
    result.fitness.clamp(0.0, 1.0)
}

/// HTTP handlers for board KPI endpoints.
pub mod handlers {
    use super::*;
    use axum::{
        extract::Json,
        http::StatusCode,
        response::{IntoResponse, Response},
    };
    use opentelemetry::global;
    use opentelemetry::trace::{Span, Tracer};
    use opentelemetry::KeyValue;
    use std::time::Duration;
    use tokio::time::timeout;

    use crate::semconv::board_attributes::{
        BOARD_KPI_BOTTLENECK_COUNT, BOARD_KPI_CONFORMANCE_SCORE, BOARD_KPI_CYCLE_TIME_AVG_MS,
        BOARD_KPI_ERROR, BOARD_KPI_VARIANT_COUNT, BOARD_PROCESS_ID,
    };
    use crate::semconv::board_span_names::BOARD_KPI_COMPUTE_SPAN;

    /// GET /api/board/kpis — compute board KPIs from a sample event log.
    ///
    /// Returns sensible defaults when no data is available.
    /// WvdA: 5s timeout + max 10k events.
    pub async fn board_kpis_get() -> Response {
        let tracer = global::tracer("pm4py-rust");
        let mut span = tracer.start(BOARD_KPI_COMPUTE_SPAN);
        span.set_attribute(KeyValue::new(BOARD_PROCESS_ID, "default"));

        // Build a sample log or return defaults
        let log = EventLog::new();

        let result = match timeout(
            Duration::from_millis(super::COMPUTATION_TIMEOUT_MS),
            tokio::task::spawn_blocking(move || compute_board_kpis(&log)),
        )
        .await
        {
            Ok(Ok(kpis)) => kpis,
            Ok(Err(_join_err)) => {
                span.set_attribute(KeyValue::new(BOARD_KPI_ERROR, "computation_panic"));
                BoardKpiResponse::default()
            }
            Err(_elapsed) => {
                span.set_attribute(KeyValue::new(BOARD_KPI_ERROR, "timeout"));
                BoardKpiResponse::default()
            }
        };

        span.set_attribute(KeyValue::new(
            BOARD_KPI_CYCLE_TIME_AVG_MS,
            result.cycle_time_avg_ms,
        ));
        span.set_attribute(KeyValue::new(
            BOARD_KPI_CONFORMANCE_SCORE,
            result.conformance_score,
        ));
        span.set_attribute(KeyValue::new(
            BOARD_KPI_BOTTLENECK_COUNT,
            result.bottleneck_count as i64,
        ));
        span.set_attribute(KeyValue::new(
            BOARD_KPI_VARIANT_COUNT,
            result.variant_count as i64,
        ));

        drop(span);

        (StatusCode::OK, Json(result)).into_response()
    }

    /// POST /api/board/kpis — compute board KPIs from a provided event log.
    ///
    /// Accepts an inline event log JSON or an event_log_id for scoping.
    /// WvdA: 5s timeout + max 10k events.
    pub async fn board_kpis_post(Json(body): Json<serde_json::Value>) -> Response {
        let tracer = global::tracer("pm4py-rust");
        let mut span = tracer.start(BOARD_KPI_COMPUTE_SPAN);

        // Try to extract event_log_id for span attribution
        let log_id = body
            .get("event_log_id")
            .and_then(|v| v.as_str())
            .unwrap_or("inline")
            .to_string();
        span.set_attribute(KeyValue::new(BOARD_PROCESS_ID, log_id.clone()));

        // Try to parse an inline event log from the body
        let log: EventLog = if let Some(event_log_val) = body.get("event_log") {
            match serde_json::from_value::<EventLog>(event_log_val.clone()) {
                Ok(l) => l,
                Err(e) => {
                    span.set_attribute(KeyValue::new(
                        BOARD_KPI_ERROR,
                        format!("parse_error: {}", e),
                    ));
                    drop(span);
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({
                            "error": "Failed to parse event_log",
                            "details": e.to_string()
                        })),
                    )
                        .into_response();
                }
            }
        } else {
            // No inline log provided — return defaults
            EventLog::new()
        };

        let result = match timeout(
            Duration::from_millis(super::COMPUTATION_TIMEOUT_MS),
            tokio::task::spawn_blocking(move || compute_board_kpis(&log)),
        )
        .await
        {
            Ok(Ok(kpis)) => kpis,
            Ok(Err(_join_err)) => {
                span.set_attribute(KeyValue::new(BOARD_KPI_ERROR, "computation_panic"));
                BoardKpiResponse::default()
            }
            Err(_elapsed) => {
                span.set_attribute(KeyValue::new(BOARD_KPI_ERROR, "timeout"));
                BoardKpiResponse::default()
            }
        };

        span.set_attribute(KeyValue::new(
            BOARD_KPI_CYCLE_TIME_AVG_MS,
            result.cycle_time_avg_ms,
        ));
        span.set_attribute(KeyValue::new(
            BOARD_KPI_CONFORMANCE_SCORE,
            result.conformance_score,
        ));
        span.set_attribute(KeyValue::new(
            BOARD_KPI_BOTTLENECK_COUNT,
            result.bottleneck_count as i64,
        ));
        span.set_attribute(KeyValue::new(
            BOARD_KPI_VARIANT_COUNT,
            result.variant_count as i64,
        ));

        drop(span);

        (StatusCode::OK, Json(result)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::{Event, Trace};
    use chrono::{Duration, Utc};

    /// Helper: build a small event log with N traces, each having a sequence of activities.
    fn build_sample_log(trace_count: usize) -> EventLog {
        let mut log = EventLog::new();
        let base = Utc::now();

        for i in 0..trace_count {
            let mut trace = Trace::new(format!("case-{}", i));
            let activities = vec!["Start", "Review", "Approve", "End"];
            for (j, act) in activities.iter().enumerate() {
                let ts = base + Duration::milliseconds((i * 10000 + j * 2000) as i64);
                trace.add_event(Event::new(*act, ts));
            }
            log.add_trace(trace);
        }
        log
    }

    #[test]
    fn test_board_kpis_returns_all_four_metrics() {
        let log = build_sample_log(5);
        let result = compute_board_kpis(&log);

        // All four metrics should be present and non-default for a real log
        assert!(
            result.cycle_time_avg_ms > 0.0,
            "cycle_time_avg_ms should be positive, got {}",
            result.cycle_time_avg_ms
        );
        // conformance_score is computed via Alpha+TokenReplay — just verify it's in range
        assert!(
            result.conformance_score >= 0.0 && result.conformance_score <= 1.0,
            "conformance_score should be in [0.0, 1.0], got {}",
            result.conformance_score
        );
        // variant_count should be >= 1 (all traces have same activities → 1 variant)
        assert!(
            result.variant_count >= 1,
            "variant_count should be >= 1, got {}",
            result.variant_count
        );
        // events_processed should match what we put in
        assert_eq!(result.events_processed, 20); // 5 traces * 4 events
        assert!(!result.truncated);
    }

    #[test]
    fn test_board_kpis_conformance_bounded_0_to_1() {
        let log = build_sample_log(3);
        let result = compute_board_kpis(&log);
        assert!(
            result.conformance_score >= 0.0,
            "conformance_score must be >= 0.0, got {}",
            result.conformance_score
        );
        assert!(
            result.conformance_score <= 1.0,
            "conformance_score must be <= 1.0, got {}",
            result.conformance_score
        );
    }

    #[test]
    fn test_board_kpis_empty_log_returns_defaults() {
        let log = EventLog::new();
        let result = compute_board_kpis(&log);

        assert_eq!(
            result.cycle_time_avg_ms, 0.0,
            "empty log should have 0.0 cycle_time_avg_ms"
        );
        assert_eq!(
            result.conformance_score, 0.0,
            "empty log should have 0.0 conformance_score"
        );
        assert_eq!(
            result.bottleneck_count, 0,
            "empty log should have 0 bottleneck_count"
        );
        assert_eq!(
            result.variant_count, 0,
            "empty log should have 0 variant_count"
        );
        assert_eq!(result.events_processed, 0);
        assert!(!result.truncated);
    }

    #[test]
    fn test_board_kpis_truncation_enforced() {
        // Build a log that exceeds MAX_EVENTS_PER_BATCH
        let mut log = EventLog::new();
        let base = Utc::now();

        // Each trace has 100 events; 200 traces = 20,000 events > 10,000 limit
        for i in 0..200 {
            let mut trace = Trace::new(format!("case-{}", i));
            for j in 0..100 {
                let ts = base + Duration::milliseconds((i * 1000 + j * 10) as i64);
                trace.add_event(Event::new(format!("Act-{}", j % 5), ts));
            }
            log.add_trace(trace);
        }

        let result = compute_board_kpis(&log);

        assert!(result.truncated, "log should be marked as truncated");
        assert!(
            result.events_processed <= MAX_EVENTS_PER_BATCH,
            "events_processed ({}) should be <= MAX_EVENTS_PER_BATCH ({})",
            result.events_processed,
            MAX_EVENTS_PER_BATCH
        );
    }

    #[test]
    fn test_truncate_event_log_preserves_complete_traces() {
        let mut log = EventLog::new();
        let base = Utc::now();

        // 3 traces with 5, 5, 5 events = 15 total; limit to 10
        for i in 0..3 {
            let mut trace = Trace::new(format!("case-{}", i));
            for j in 0..5 {
                let ts = base + Duration::milliseconds((i * 100 + j * 10) as i64);
                trace.add_event(Event::new("Act", ts));
            }
            log.add_trace(trace);
        }

        let (truncated, count, was_truncated) = truncate_event_log(&log, 10);
        assert_eq!(truncated.traces.len(), 2, "should keep 2 complete traces");
        assert_eq!(count, 10, "should have exactly 10 events");
        assert!(was_truncated, "should be marked truncated");
    }

    #[test]
    fn test_board_kpis_response_serializes_to_json() {
        let resp = BoardKpiResponse {
            cycle_time_avg_ms: 6000.0,
            conformance_score: 0.85,
            bottleneck_count: 2,
            variant_count: 4,
            events_processed: 100,
            truncated: false,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("cycle_time_avg_ms"));
        assert!(json.contains("conformance_score"));
        assert!(json.contains("bottleneck_count"));
        assert!(json.contains("variant_count"));
    }

    #[test]
    fn test_board_kpi_request_deserializes() {
        let json = r#"{"event_log_id": "test-log-123"}"#;
        let req: BoardKpiRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.event_log_id, Some("test-log-123".to_string()));
    }

    #[test]
    fn test_board_kpi_request_optional_log_id() {
        let json = r#"{}"#;
        let req: BoardKpiRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.event_log_id, None);
    }
}
