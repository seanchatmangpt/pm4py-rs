//! Chicago TDD — OTEL span emission tests for predictive analytics and statistics.
//!
//! RED phase: these tests assert that the Tracing struct captures spans with
//! the correct names and attributes after calling the prediction/bottleneck functions.
//!
//! Run: cargo test predictive_spans

use chrono::{Duration as ChronoDuration, Utc};
use pm4py::log::{Event, EventLog, Trace};
use pm4py::observability::Tracing;
use pm4py::predictive::{NextActivityPredictor, OutcomePredictor, RemainingTimePredictor};
use pm4py::statistics::bottleneck::identify_bottlenecks_with_tracing;

// ─── shared test fixture ────────────────────────────────────────────────────

fn sample_log() -> EventLog {
    let mut log = EventLog::new();
    let now = Utc::now();

    let mut t1 = Trace::new("case_1");
    t1.add_event(Event::new("Start", now));
    t1.add_event(Event::new("Review", now + ChronoDuration::seconds(60)));
    t1.add_event(Event::new("End", now + ChronoDuration::seconds(120)));
    log.add_trace(t1);

    let mut t2 = Trace::new("case_2");
    t2.add_event(Event::new("Start", now));
    t2.add_event(Event::new("Approve", now + ChronoDuration::seconds(90)));
    t2.add_event(Event::new("End", now + ChronoDuration::seconds(180)));
    log.add_trace(t2);

    log
}

// ─── Test 1: NextActivityPredictor emits prediction span ────────────────────

#[test]
fn test_next_activity_emits_prediction_span() {
    let log = sample_log();
    let tracing = Tracing::new();
    let predictor = NextActivityPredictor::new(&log);

    let partial = {
        let mut t = Trace::new("ongoing");
        t.add_event(Event::new("Start", Utc::now()));
        t
    };

    predictor.predict_next_activity_traced(&partial, 3, &tracing);

    let spans = tracing.get_spans();
    assert!(
        !spans.is_empty(),
        "expected at least one span after predict_next_activity_traced"
    );

    let prediction_span = spans
        .iter()
        .find(|s| s.span_name == "process.mining.prediction.make");
    assert!(
        prediction_span.is_some(),
        "expected span named 'process.mining.prediction.make', got: {:?}",
        spans.iter().map(|s| &s.span_name).collect::<Vec<_>>()
    );

    let span = prediction_span.unwrap();
    assert_eq!(span.status, "ok", "span status must be 'ok'");
    assert!(
        span.attributes
            .contains_key("process.mining.prediction.model_type"),
        "span must carry process.mining.prediction.model_type attribute"
    );
}

// ─── Test 2: OutcomePredictor emits prediction span with model_type ──────────

#[test]
fn test_outcome_prediction_emits_span() {
    let log = sample_log();
    let tracing = Tracing::new();
    let predictor =
        OutcomePredictor::new(&log, |_trace| pm4py::predictive::CaseOutcome::Successful);

    let partial = {
        let mut t = Trace::new("ongoing");
        t.add_event(Event::new("Start", Utc::now()));
        t
    };

    predictor.assess_risk_traced(&partial, &tracing);

    let spans = tracing.get_spans();
    let prediction_span = spans
        .iter()
        .find(|s| s.span_name == "process.mining.prediction.make");
    assert!(
        prediction_span.is_some(),
        "expected span 'process.mining.prediction.make' from assess_risk_traced"
    );

    let span = prediction_span.unwrap();
    assert_eq!(span.status, "ok");
    assert!(
        span.attributes
            .contains_key("process.mining.prediction.model_type"),
        "must carry model_type attribute"
    );
    // outcome predictor records risk score
    assert!(
        span.attributes
            .contains_key("process.mining.prediction.confidence"),
        "must carry confidence attribute"
    );
}

// ─── Test 3: RemainingTimePredictor emits span with remaining_time ───────────

#[test]
fn test_remaining_time_emits_span() {
    let log = sample_log();
    let tracing = Tracing::new();
    let predictor = RemainingTimePredictor::new(&log);

    let now = Utc::now();
    let partial = {
        let mut t = Trace::new("ongoing");
        t.add_event(Event::new("Start", now));
        t.add_event(Event::new("Review", now + ChronoDuration::seconds(30)));
        t
    };

    predictor.predict_remaining_time_traced(&partial, None, &tracing);

    let spans = tracing.get_spans();
    let prediction_span = spans
        .iter()
        .find(|s| s.span_name == "process.mining.prediction.make");
    assert!(
        prediction_span.is_some(),
        "expected span 'process.mining.prediction.make' from predict_remaining_time_traced"
    );

    let span = prediction_span.unwrap();
    assert_eq!(span.status, "ok");
    assert!(
        span.attributes
            .contains_key("process.mining.prediction.model_type"),
        "must carry model_type attribute"
    );
}

// ─── Test 4: identify_bottlenecks emits bottleneck_detection span ────────────

#[test]
fn test_bottleneck_detection_emits_span() {
    let log = sample_log();
    let tracing = Tracing::new();

    identify_bottlenecks_with_tracing(&log, &tracing);

    let spans = tracing.get_spans();
    assert!(
        !spans.is_empty(),
        "expected spans after identify_bottlenecks_with_tracing"
    );

    let bottleneck_span = spans
        .iter()
        .find(|s| s.span_name == "process.mining.bottleneck_detection");
    assert!(
        bottleneck_span.is_some(),
        "expected span 'process.mining.bottleneck_detection', got: {:?}",
        spans.iter().map(|s| &s.span_name).collect::<Vec<_>>()
    );

    let span = bottleneck_span.unwrap();
    assert_eq!(span.status, "ok");
    assert!(
        span.attributes
            .contains_key("process.mining.bottleneck.activity"),
        "must carry bottleneck.activity attribute"
    );
    assert!(
        span.attributes
            .contains_key("process.mining.bottleneck.score"),
        "must carry bottleneck.score attribute"
    );
}
