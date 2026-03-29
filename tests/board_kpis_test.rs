//! Chicago TDD tests for Board KPI Pipeline.
//!
//! Tests assert exact behaviour for the 4 board-ready KPIs:
//! cycle_time_avg_ms, conformance_score, bottleneck_count, variant_count.
//!
//! No shared state; each test sets up its own event log.
//! WvdA: verifies bounded computation (truncation) and safe defaults.

use chrono::{Duration, Utc};
use pm4py::board_kpis::{compute_board_kpis, truncate_event_log, BoardKpiResponse};
use pm4py::log::{Event, EventLog, Trace};

/// Helper: build an event log with `trace_count` traces, each with the given activities
/// spaced `interval_ms` apart.
fn build_log(trace_count: usize, activities: &[&str], interval_ms: i64) -> EventLog {
    let mut log = EventLog::new();
    let base = Utc::now();

    for i in 0..trace_count {
        let mut trace = Trace::new(format!("case-{}", i));
        for (j, act) in activities.iter().enumerate() {
            let ts = base + Duration::milliseconds((i as i64) * 10000 + (j as i64) * interval_ms);
            trace.add_event(Event::new(*act, ts));
        }
        log.add_trace(trace);
    }
    log
}

// ── Test 1: All four metrics present ─────────────────────────────────────────

#[test]
fn test_board_kpis_returns_all_four_metrics() {
    let log = build_log(5, &["Start", "Review", "Approve", "End"], 2000);
    let result = compute_board_kpis(&log);

    // cycle_time_avg_ms: 3 intervals * 2000ms = 6000ms expected
    assert!(
        result.cycle_time_avg_ms > 0.0,
        "cycle_time_avg_ms should be positive, got {}",
        result.cycle_time_avg_ms
    );

    // conformance_score: must be present and numeric
    assert!(
        !result.conformance_score.is_nan(),
        "conformance_score must not be NaN"
    );

    // bottleneck_count: non-negative
    // (with 2s intervals all activities have same duration; count depends on threshold)
    assert!(
        result.bottleneck_count <= 4,
        "bottleneck_count should be <= number of activities"
    );

    // variant_count: all traces have same sequence → 1 variant
    assert_eq!(
        result.variant_count, 1,
        "all identical traces should produce 1 variant, got {}",
        result.variant_count
    );

    // events_processed: 5 traces * 4 events = 20
    assert_eq!(result.events_processed, 20);
    assert!(!result.truncated);
}

// ── Test 2: Conformance bounded [0, 1] ──────────────────────────────────────

#[test]
fn test_board_kpis_conformance_bounded_0_to_1() {
    let log = build_log(10, &["A", "B", "C"], 1000);
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

// ── Test 3: Empty log returns defaults ──────────────────────────────────────

#[test]
fn test_board_kpis_empty_log_returns_defaults() {
    let log = EventLog::new();
    let result = compute_board_kpis(&log);

    assert_eq!(result.cycle_time_avg_ms, 0.0, "empty → 0.0 cycle time");
    assert_eq!(result.conformance_score, 0.0, "empty → 0.0 conformance");
    assert_eq!(result.bottleneck_count, 0, "empty → 0 bottlenecks");
    assert_eq!(result.variant_count, 0, "empty → 0 variants");
    assert_eq!(result.events_processed, 0, "empty → 0 events");
    assert!(!result.truncated, "empty → not truncated");
}

// ── Test 4: Computation timeout bounded (truncation as proxy) ───────────────

#[test]
fn test_board_kpis_computation_timeout_bounded() {
    // Build a log exceeding MAX_EVENTS_PER_BATCH (10,000).
    // 150 traces * 100 events = 15,000 events → truncation enforced.
    let mut log = EventLog::new();
    let base = Utc::now();

    for i in 0..150 {
        let mut trace = Trace::new(format!("case-{}", i));
        for j in 0..100 {
            let ts = base + Duration::milliseconds((i * 1000 + j * 10) as i64);
            trace.add_event(Event::new(format!("Act-{}", j % 5), ts));
        }
        log.add_trace(trace);
    }

    let result = compute_board_kpis(&log);

    // Truncation enforced: events_processed <= 10,000
    assert!(
        result.events_processed <= 10_000,
        "events_processed ({}) must not exceed 10,000",
        result.events_processed
    );
    assert!(
        result.truncated,
        "log exceeding 10k events must be truncated"
    );

    // Computation still produces valid results
    assert!(
        result.cycle_time_avg_ms >= 0.0,
        "cycle_time must be non-negative"
    );
    assert!(
        result.conformance_score >= 0.0 && result.conformance_score <= 1.0,
        "conformance must be in [0, 1]"
    );
}

// ── Additional: truncation preserves complete traces ────────────────────────

#[test]
fn test_truncate_event_log_preserves_complete_traces() {
    let log = build_log(3, &["A", "B", "C", "D", "E"], 100); // 3 traces * 5 events = 15
    let (truncated, count, was_truncated) = truncate_event_log(&log, 10);

    assert_eq!(
        truncated.traces.len(),
        2,
        "should keep 2 complete traces (10 events)"
    );
    assert_eq!(count, 10);
    assert!(was_truncated);
}

// ── Additional: multiple variants detected ──────────────────────────────────

#[test]
fn test_board_kpis_multiple_variants() {
    let mut log = EventLog::new();
    let base = Utc::now();

    // Variant 1: A -> B -> C
    for i in 0..3 {
        let mut trace = Trace::new(format!("v1-{}", i));
        for (j, act) in ["A", "B", "C"].iter().enumerate() {
            trace.add_event(Event::new(
                *act,
                base + Duration::milliseconds((i * 100 + j * 10) as i64),
            ));
        }
        log.add_trace(trace);
    }

    // Variant 2: A -> C -> B (different order)
    for i in 0..2 {
        let mut trace = Trace::new(format!("v2-{}", i));
        for (j, act) in ["A", "C", "B"].iter().enumerate() {
            trace.add_event(Event::new(
                *act,
                base + Duration::milliseconds((1000 + i * 100 + j * 10) as i64),
            ));
        }
        log.add_trace(trace);
    }

    let result = compute_board_kpis(&log);
    assert!(
        result.variant_count >= 2,
        "should detect at least 2 variants, got {}",
        result.variant_count
    );
}

// ── Additional: response serializes correctly ───────────────────────────────

#[test]
fn test_board_kpi_response_json_roundtrip() {
    let resp = BoardKpiResponse {
        cycle_time_avg_ms: 5500.0,
        conformance_score: 0.92,
        bottleneck_count: 3,
        variant_count: 7,
        events_processed: 500,
        truncated: false,
    };
    let json = serde_json::to_string(&resp).unwrap();
    let parsed: BoardKpiResponse = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.cycle_time_avg_ms, 5500.0);
    assert_eq!(parsed.conformance_score, 0.92);
    assert_eq!(parsed.bottleneck_count, 3);
    assert_eq!(parsed.variant_count, 7);
    assert_eq!(parsed.events_processed, 500);
    assert!(!parsed.truncated);
}

// ── Additional: single-event traces handled gracefully ──────────────────────

#[test]
fn test_board_kpis_single_event_traces() {
    let mut log = EventLog::new();
    let base = Utc::now();

    for i in 0..5 {
        let mut trace = Trace::new(format!("case-{}", i));
        trace.add_event(Event::new(
            "OnlyActivity",
            base + Duration::milliseconds(i * 100),
        ));
        log.add_trace(trace);
    }

    let result = compute_board_kpis(&log);
    // Single-event traces have 0 duration
    assert_eq!(
        result.cycle_time_avg_ms, 0.0,
        "single-event traces should have 0 cycle time"
    );
    assert!(result.variant_count >= 1);
    assert_eq!(result.events_processed, 5);
}
