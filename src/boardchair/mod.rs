//! Board Chair Analytics — Conway's Law and Little's Law analyzers.
//!
//! Pure functions with no side effects. WvdA-sound: all inputs bounded,
//! deterministic outputs. Used by HTTP endpoints to compute analysis results
//! before emitting OTEL spans.
//!
//! # Conway's Law
//! A Conway violation occurs when boundary handoff time exceeds 40% of cycle time.
//! High conway_score means team boundaries are the process bottleneck — requires
//! board-level structural decision (not operational healing).
//!
//! # Little's Law
//! L = λW: expected WIP = arrival_rate × average_cycle_time_seconds.
//! A violation occurs when actual WIP > 1.5× expected WIP (50% overload threshold).

use serde::{Deserialize, Serialize};

/// Conway's Law check request
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ConwayCheckRequest {
    /// Time spent crossing org boundaries (ms)
    pub boundary_time_ms: i64,
    /// Total end-to-end cycle time (ms)
    pub cycle_time_ms: i64,
    /// Optional process/department identifier
    pub process_id: Option<String>,
}

/// Conway's Law check result
#[derive(Debug, Serialize, Clone)]
pub struct ConwayCheckResult {
    /// Whether a Conway violation was detected (conway_score > 0.4)
    pub is_violation: bool,
    /// boundary_time_ms / cycle_time_ms — range [0.0, 1.0]
    pub conway_score: f64,
    /// Boundary handoff time in ms (echoed back)
    pub boundary_time_ms: i64,
    /// Total cycle time in ms (echoed back)
    pub cycle_time_ms: i64,
}

/// Little's Law analysis request
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LittlesLawRequest {
    /// Arrival rate: items per second (λ)
    pub arrival_rate: f64,
    /// Observed WIP count (L)
    pub wip: f64,
    /// Average cycle time in milliseconds (W, converted internally to seconds)
    pub cycle_time_ms: f64,
}

/// Little's Law analysis result
#[derive(Debug, Serialize, Clone)]
pub struct LittlesLawResult {
    /// Whether actual WIP exceeds 1.5× expected WIP
    pub is_violation: bool,
    /// actual_wip / expected_wip ratio
    pub stability_ratio: f64,
    /// Observed WIP (echoed back)
    pub actual_wip: f64,
    /// arrival_rate × (cycle_time_ms / 1000.0) — theoretical WIP from Little's Law
    pub expected_wip: f64,
}

/// Conway violation threshold: boundary time > 40% of cycle time → escalate to board
const CONWAY_VIOLATION_THRESHOLD: f64 = 0.4;

/// Little's Law overload threshold: actual WIP > 1.5× expected → operational healing
const LITTLES_LAW_OVERLOAD_FACTOR: f64 = 1.5;

/// Check for a Conway's Law violation.
///
/// conway_score = boundary_time_ms / cycle_time_ms
/// violation when conway_score > 0.4
///
/// WvdA: pure function, deterministic, bounded inputs only.
/// Returns `is_violation = false` (safe default) when cycle_time_ms == 0
/// to prevent division by zero.
pub fn check_conway(boundary_time_ms: i64, cycle_time_ms: i64) -> ConwayCheckResult {
    if cycle_time_ms <= 0 {
        return ConwayCheckResult {
            is_violation: false,
            conway_score: 0.0,
            boundary_time_ms,
            cycle_time_ms,
        };
    }

    let conway_score = boundary_time_ms as f64 / cycle_time_ms as f64;
    let is_violation = conway_score > CONWAY_VIOLATION_THRESHOLD;

    ConwayCheckResult {
        is_violation,
        conway_score,
        boundary_time_ms,
        cycle_time_ms,
    }
}

/// Analyse queue stability using Little's Law (L = λW).
///
/// expected_wip = arrival_rate × (cycle_time_ms / 1000.0)
/// violation when actual_wip > 1.5 × expected_wip
///
/// WvdA: pure function, deterministic, bounded inputs only.
/// Returns `is_violation = false` (safe default) when expected_wip == 0
/// to prevent division by zero.
pub fn analyze_littles_law(arrival_rate: f64, wip: f64, cycle_time_ms: f64) -> LittlesLawResult {
    let expected_wip = arrival_rate * (cycle_time_ms / 1000.0);

    if expected_wip <= 0.0 {
        return LittlesLawResult {
            is_violation: false,
            stability_ratio: 0.0,
            actual_wip: wip,
            expected_wip: 0.0,
        };
    }

    let stability_ratio = wip / expected_wip;
    let is_violation = wip > LITTLES_LAW_OVERLOAD_FACTOR * expected_wip;

    LittlesLawResult {
        is_violation,
        stability_ratio,
        actual_wip: wip,
        expected_wip,
    }
}

/// HTTP handlers for boardchair endpoints.
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

    use crate::semconv::board_attributes::{
        BOARD_CONWAY_SCORE, BOARD_IS_VIOLATION, BOARD_PROCESS_ID,
    };
    use crate::semconv::board_span_names::BOARD_CONWAY_CHECK_SPAN;

    /// POST /api/boardchair/conway-check
    ///
    /// Computes Conway's Law violation check and emits a `board.conway_check` span.
    /// Returns 200 OK with ConwayCheckResult JSON.
    pub async fn conway_check(Json(req): Json<ConwayCheckRequest>) -> Response {
        let tracer = global::tracer("pm4py-rust");
        let mut span = tracer.start(BOARD_CONWAY_CHECK_SPAN);

        // Set required span attributes (per spans.yaml)
        let process_id = req
            .process_id
            .clone()
            .unwrap_or_else(|| "unknown".to_string());
        span.set_attribute(KeyValue::new(BOARD_PROCESS_ID, process_id));

        let result = check_conway(req.boundary_time_ms, req.cycle_time_ms);

        span.set_attribute(KeyValue::new(BOARD_IS_VIOLATION, result.is_violation));
        span.set_attribute(KeyValue::new(BOARD_CONWAY_SCORE, result.conway_score));

        drop(span); // end span

        (StatusCode::OK, Json(result)).into_response()
    }

    /// POST /api/boardchair/littles-law
    ///
    /// Computes Little's Law stability check and emits a `board.conway_check_summary` span.
    /// Returns 200 OK with LittlesLawResult JSON.
    pub async fn littles_law(Json(req): Json<LittlesLawRequest>) -> Response {
        let tracer = global::tracer("pm4py-rust");
        let mut span = tracer.start(BOARD_CONWAY_CHECK_SPAN);

        let result = analyze_littles_law(req.arrival_rate, req.wip, req.cycle_time_ms);

        span.set_attribute(KeyValue::new(BOARD_IS_VIOLATION, result.is_violation));
        span.set_attribute(KeyValue::new(
            "board.littles_law_stability_ratio",
            result.stability_ratio,
        ));

        drop(span); // end span

        (StatusCode::OK, Json(result)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Conway's Law unit tests ---

    #[test]
    fn test_conway_violation_when_boundary_exceeds_threshold() {
        // boundary = 50ms, cycle = 100ms → score = 0.5 > 0.4 → violation
        let result = check_conway(50, 100);
        assert!(result.is_violation);
        assert!((result.conway_score - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_conway_no_violation_within_normal_range() {
        // boundary = 30ms, cycle = 100ms → score = 0.3 < 0.4 → no violation
        let result = check_conway(30, 100);
        assert!(!result.is_violation);
        assert!((result.conway_score - 0.3).abs() < 0.001);
    }

    #[test]
    fn test_conway_exactly_at_threshold_is_no_violation() {
        // score == 0.4 is NOT a violation (violation is strictly > 0.4)
        let result = check_conway(40, 100);
        assert!(!result.is_violation);
        assert!((result.conway_score - 0.4).abs() < 0.001);
    }

    #[test]
    fn test_conway_zero_cycle_time_returns_safe_default() {
        let result = check_conway(50, 0);
        assert!(!result.is_violation);
        assert_eq!(result.conway_score, 0.0);
    }

    // --- Little's Law unit tests ---

    #[test]
    fn test_littles_law_violation_when_wip_exceeds_1_5x() {
        // arrival_rate=10/s, cycle=100ms=0.1s → expected_wip=1.0; actual=2.0 > 1.5 → violation
        let result = analyze_littles_law(10.0, 2.0, 100.0);
        assert!(result.is_violation);
        assert!((result.expected_wip - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_littles_law_stable_when_wip_within_bounds() {
        // arrival_rate=10/s, cycle=100ms → expected_wip=1.0; actual=1.2 < 1.5 → no violation
        let result = analyze_littles_law(10.0, 1.2, 100.0);
        assert!(!result.is_violation);
        assert!((result.expected_wip - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_littles_law_zero_expected_wip_returns_safe_default() {
        // arrival_rate=0 → expected_wip=0 → safe default
        let result = analyze_littles_law(0.0, 5.0, 100.0);
        assert!(!result.is_violation);
        assert_eq!(result.expected_wip, 0.0);
    }
}
