/// Chicago TDD — Remaining Time Prediction MCP exposure tests.
///
/// T1: predict_remaining_time_from_log returns an estimate for a partial trace
/// T2: returned confidence score is bounded in [0.0, 1.0]
use chrono::{Duration as ChronoDuration, Utc};
use pm4py::log::{Event, EventLog, Trace};
use pm4py::predictive::remaining_time::predict_remaining_time_from_log;

/// Build a training log with 3 completed cases of known duration (~60 min each).
fn make_training_log() -> EventLog {
    let mut log = EventLog::new();
    let now = Utc::now();

    for i in 0..3_u32 {
        let mut t = Trace::new(format!("case_{}", i));
        t.add_event(Event::new("Start", now));
        t.add_event(Event::new("Activity_A", now + ChronoDuration::minutes(15)));
        t.add_event(Event::new("Activity_B", now + ChronoDuration::minutes(40)));
        t.add_event(Event::new("End", now + ChronoDuration::minutes(60)));
        log.add_trace(t);
    }

    log
}

/// A partial trace with only the first two events (partial — not yet complete).
fn make_partial_trace_events() -> Vec<Event> {
    let now = Utc::now();
    vec![
        Event::new("Start", now),
        Event::new("Activity_A", now + ChronoDuration::minutes(10)),
    ]
}

// ── T1 ────────────────────────────────────────────────────────────────────────

#[test]
fn remaining_time_prediction_returns_estimate_for_partial_trace() {
    let training_log = make_training_log();
    let partial = make_partial_trace_events();

    let resp = predict_remaining_time_from_log(&training_log, &partial);

    assert!(
        resp.is_some(),
        "predict_remaining_time_from_log must return Some for a non-empty training log \
         and non-empty partial trace"
    );

    let resp = resp.unwrap();
    assert!(
        resp.predicted_remaining_seconds >= 0.0,
        "Predicted remaining seconds must be non-negative; got {}",
        resp.predicted_remaining_seconds
    );
    assert!(
        resp.similar_cases_count > 0,
        "similar_cases_count must be > 0 for a non-empty training log"
    );
}

// ── T2 ────────────────────────────────────────────────────────────────────────

#[test]
fn remaining_time_prediction_confidence_bounded_0_to_1() {
    let training_log = make_training_log();
    let partial = make_partial_trace_events();

    let resp = predict_remaining_time_from_log(&training_log, &partial)
        .expect("predict_remaining_time_from_log should return Some");

    assert!(
        resp.confidence >= 0.0 && resp.confidence <= 1.0,
        "Confidence must be in [0.0, 1.0]; got {}",
        resp.confidence
    );

    // percentile_10 ≤ predicted ≤ percentile_90 (confidence interval ordering)
    assert!(
        resp.percentile_10 <= resp.predicted_remaining_seconds + 1e-9,
        "percentile_10 ({}) must be ≤ predicted_remaining_seconds ({})",
        resp.percentile_10,
        resp.predicted_remaining_seconds
    );
}
