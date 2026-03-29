use chrono::Utc;
/// Chicago TDD — RED phase: conformance span emission tests.
///
/// These tests verify that token_replay, alignment, and precision conformance
/// checkers emit OTEL spans with the required attributes into the pm4py Tracing
/// in-memory store.
///
/// Run: cargo test conformance_spans
use pm4py::log::{Event, EventLog, Trace};
use pm4py::models::petri_net::{Arc, Place, Transition};
use pm4py::models::PetriNet;
use pm4py::observability::Tracing;
use pm4py::semconv::conformance_attributes::{CONFORMANCE_FITNESS, CONFORMANCE_PRECISION};
use pm4py::semconv::spans::CONFORMANCE_CHECK;

// ── Helpers ──────────────────────────────────────────────────────────────────

fn minimal_log() -> EventLog {
    let mut log = EventLog::new();
    let now = Utc::now();
    let mut trace = Trace::new("case_1");
    trace.add_event(Event::new("a", now));
    trace.add_event(Event::new("b", now));
    log.add_trace(trace);
    log
}

fn minimal_net() -> PetriNet {
    let mut net = PetriNet::new();
    let p1 = Place::new("p1").with_initial_marking(1);
    let t1 = Transition::new("t1").with_label("a");
    let p2 = Place::new("p2");
    let t2 = Transition::new("t2").with_label("b");
    let p3 = Place::new("p3").with_final_marking(1);

    let p1_id = p1.id.clone();
    let t1_id = t1.id.clone();
    let p2_id = p2.id.clone();
    let t2_id = t2.id.clone();
    let p3_id = p3.id.clone();

    net.add_place(p1);
    net.add_transition(t1);
    net.add_place(p2);
    net.add_transition(t2);
    net.add_place(p3);

    net.add_arc(Arc::new(&p1_id, &t1_id));
    net.add_arc(Arc::new(&t1_id, &p2_id));
    net.add_arc(Arc::new(&p2_id, &t2_id));
    net.add_arc(Arc::new(&t2_id, &p3_id));

    net.set_initial_place(p1_id);
    net.set_final_place(p3_id);

    net
}

// ── Token Replay span tests ───────────────────────────────────────────────────

#[test]
fn token_replay_emits_conformance_check_span() {
    let log = minimal_log();
    let net = minimal_net();
    let tracing = Tracing::new();

    pm4py::conformance::token_replay::check_with_tracing(&log, &net, &tracing);

    let spans = tracing.get_spans();
    let conformance_span = spans.iter().find(|s| s.span_name == CONFORMANCE_CHECK);
    assert!(
        conformance_span.is_some(),
        "Expected span named '{}' but got spans: {:?}",
        CONFORMANCE_CHECK,
        spans.iter().map(|s| &s.span_name).collect::<Vec<_>>()
    );
}

#[test]
fn token_replay_span_has_fitness_attribute() {
    let log = minimal_log();
    let net = minimal_net();
    let tracing = Tracing::new();

    pm4py::conformance::token_replay::check_with_tracing(&log, &net, &tracing);

    let spans = tracing.get_spans();
    let span = spans
        .iter()
        .find(|s| s.span_name == CONFORMANCE_CHECK)
        .expect("conformance.check span must exist");

    assert!(
        span.attributes.contains_key(CONFORMANCE_FITNESS),
        "Span must have attribute '{}', got: {:?}",
        CONFORMANCE_FITNESS,
        span.attributes.keys().collect::<Vec<_>>()
    );
}

#[test]
fn token_replay_span_has_trace_count_attribute() {
    let log = minimal_log();
    let net = minimal_net();
    let tracing = Tracing::new();

    pm4py::conformance::token_replay::check_with_tracing(&log, &net, &tracing);

    let spans = tracing.get_spans();
    let span = spans
        .iter()
        .find(|s| s.span_name == CONFORMANCE_CHECK)
        .expect("conformance.check span must exist");

    assert!(
        span.attributes.contains_key("conformance.trace_count"),
        "Span must have attribute 'conformance.trace_count', got: {:?}",
        span.attributes.keys().collect::<Vec<_>>()
    );
    // Verify the count is correct (we put 1 trace in minimal_log)
    assert_eq!(
        span.attributes.get("conformance.trace_count"),
        Some(&"1".to_string()),
        "trace_count must equal the number of traces in the log"
    );
}

#[test]
fn token_replay_span_records_algorithm_attribute() {
    let log = minimal_log();
    let net = minimal_net();
    let tracing = Tracing::new();

    pm4py::conformance::token_replay::check_with_tracing(&log, &net, &tracing);

    let spans = tracing.get_spans();
    let span = spans
        .iter()
        .find(|s| s.span_name == CONFORMANCE_CHECK)
        .expect("conformance.check span must exist");

    assert_eq!(
        span.attributes.get("conformance.algorithm"),
        Some(&"token_replay".to_string()),
        "Span must record which algorithm ran"
    );
}

#[test]
fn token_replay_span_status_is_ok() {
    let log = minimal_log();
    let net = minimal_net();
    let tracing = Tracing::new();

    pm4py::conformance::token_replay::check_with_tracing(&log, &net, &tracing);

    let spans = tracing.get_spans();
    let span = spans
        .iter()
        .find(|s| s.span_name == CONFORMANCE_CHECK)
        .expect("conformance.check span must exist");

    // Span must have been ended with status "ok"
    assert_eq!(
        span.status, "ok",
        "Span status must be 'ok' after successful conformance check"
    );
}

// ── Alignment span tests ──────────────────────────────────────────────────────

#[test]
fn alignment_checker_emits_conformance_check_span() {
    let log = minimal_log();
    let net = minimal_net();
    let tracing = Tracing::new();

    pm4py::conformance::alignment::check_with_tracing(&log, &net, &tracing);

    let spans = tracing.get_spans();
    let conformance_span = spans.iter().find(|s| s.span_name == CONFORMANCE_CHECK);
    assert!(
        conformance_span.is_some(),
        "AlignmentChecker must emit a '{}' span",
        CONFORMANCE_CHECK
    );
}

#[test]
fn alignment_span_has_fitness_attribute() {
    let log = minimal_log();
    let net = minimal_net();
    let tracing = Tracing::new();

    pm4py::conformance::alignment::check_with_tracing(&log, &net, &tracing);

    let spans = tracing.get_spans();
    let span = spans
        .iter()
        .find(|s| s.span_name == CONFORMANCE_CHECK)
        .expect("conformance.check span must exist");

    assert!(
        span.attributes.contains_key(CONFORMANCE_FITNESS),
        "Alignment span must carry conformance.fitness attribute"
    );
}

#[test]
fn alignment_span_records_algorithm_attribute() {
    let log = minimal_log();
    let net = minimal_net();
    let tracing = Tracing::new();

    pm4py::conformance::alignment::check_with_tracing(&log, &net, &tracing);

    let spans = tracing.get_spans();
    let span = spans
        .iter()
        .find(|s| s.span_name == CONFORMANCE_CHECK)
        .expect("conformance.check span must exist");

    assert_eq!(
        span.attributes.get("conformance.algorithm"),
        Some(&"alignment".to_string())
    );
}

// ── Precision span tests ──────────────────────────────────────────────────────

#[test]
fn precision_emits_conformance_metrics_span() {
    let log = minimal_log();
    let net = minimal_net();
    let tracing = Tracing::new();

    pm4py::conformance::precision::calculate_with_tracing(&log, &net, &tracing);

    let spans = tracing.get_spans();
    // Precision emits "conformance.metrics.compute" per spans.rs constants
    let precision_span = spans
        .iter()
        .find(|s| s.span_name == pm4py::semconv::spans::CONFORMANCE_METRICS_COMPUTE);
    assert!(
        precision_span.is_some(),
        "Precision::calculate must emit a '{}' span, got: {:?}",
        pm4py::semconv::spans::CONFORMANCE_METRICS_COMPUTE,
        spans.iter().map(|s| &s.span_name).collect::<Vec<_>>()
    );
}

#[test]
fn precision_span_has_precision_attribute() {
    let log = minimal_log();
    let net = minimal_net();
    let tracing = Tracing::new();

    pm4py::conformance::precision::calculate_with_tracing(&log, &net, &tracing);

    let spans = tracing.get_spans();
    let span = spans
        .iter()
        .find(|s| s.span_name == pm4py::semconv::spans::CONFORMANCE_METRICS_COMPUTE)
        .expect("conformance.metrics.compute span must exist");

    assert!(
        span.attributes.contains_key(CONFORMANCE_PRECISION),
        "Precision span must carry '{}' attribute, got: {:?}",
        CONFORMANCE_PRECISION,
        span.attributes.keys().collect::<Vec<_>>()
    );
}

#[test]
fn precision_span_status_is_ok() {
    let log = minimal_log();
    let net = minimal_net();
    let tracing = Tracing::new();

    pm4py::conformance::precision::calculate_with_tracing(&log, &net, &tracing);

    let spans = tracing.get_spans();
    let span = spans
        .iter()
        .find(|s| s.span_name == pm4py::semconv::spans::CONFORMANCE_METRICS_COMPUTE)
        .expect("span must exist");

    assert_eq!(span.status, "ok");
}

// ── Semconv constant correctness ──────────────────────────────────────────────

#[test]
fn conformance_check_span_name_constant_matches_schema() {
    // Schema: semconv/model/conformance or src/semconv/spans.rs
    assert_eq!(CONFORMANCE_CHECK, "conformance.check");
}

#[test]
fn conformance_fitness_attribute_key_matches_schema() {
    assert_eq!(CONFORMANCE_FITNESS, "conformance.fitness");
}

#[test]
fn conformance_precision_attribute_key_matches_schema() {
    assert_eq!(CONFORMANCE_PRECISION, "conformance.precision");
}
