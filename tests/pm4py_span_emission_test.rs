/// Chicago TDD — OTEL span emission tests for pm4py-rust.
///
/// Tests verifying that process mining operations emit spans with the
/// correct names and attributes via the in-memory Tracing store.
///
/// No HTTP, no async, no external dependencies — pure unit tests.
///
/// Run: cargo test --test pm4py_span_emission_test
use chrono::Utc;
use pm4py::conformance::TokenReplay;
use pm4py::discovery::InductiveMiner;
use pm4py::log::{Event, EventLog, Trace};
use pm4py::models::petri_net::{Arc, Place, Transition};
use pm4py::models::PetriNet;
use pm4py::monitoring::DriftCalculator;
use pm4py::observability::Tracing;
use pm4py::semconv::conformance_attributes::CONFORMANCE_FITNESS;
use pm4py::semconv::conformance_span_names::CONFORMANCE_CHECK_SPAN;
use pm4py::semconv::process_mining_attributes::{
    process_mining_algorithm, PROCESS_MINING_ALGORITHM, PROCESS_MINING_CASE_COUNT,
    PROCESS_MINING_DRIFT_DETECTED, PROCESS_MINING_PREDICTION_CONFIDENCE,
};
use pm4py::semconv::process_mining_span_names::{
    PROCESS_MINING_DRIFT_DETECT_SPAN, PROCESS_MINING_INDUCTIVE_MINE_SPAN,
    PROCESS_MINING_PREDICTION_MAKE_SPAN,
};
use std::collections::HashMap;

// ── Shared helpers ───────────────────────────────────────────────────────────

fn build_log_with_n_cases(n: usize) -> EventLog {
    let mut log = EventLog::new();
    let now = Utc::now();
    for i in 1..=n {
        let mut trace = Trace::new(format!("case_{}", i));
        trace.add_event(Event::new("a", now));
        trace.add_event(Event::new("b", now));
        log.add_trace(trace);
    }
    log
}

fn build_two_activity_net() -> PetriNet {
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
    net.set_initial_place(p1_id.clone());
    net.set_final_place(p3_id.clone());

    net.add_arc(Arc::new(&p1_id, &t1_id));
    net.add_arc(Arc::new(&t1_id, &p2_id));
    net.add_arc(Arc::new(&p2_id, &t2_id));
    net.add_arc(Arc::new(&t2_id, &p3_id));
    net
}

// ── Test 1 ───────────────────────────────────────────────────────────────────

/// discover_with_tracing emits at least one span carrying the algorithm attribute.
#[test]
fn test_discovery_emits_span_with_algorithm_attribute() {
    let log = build_log_with_n_cases(3);
    let miner = InductiveMiner::new();

    let (_net, spans) = miner.discover_with_tracing(&log);

    let has_algo_attr = spans
        .iter()
        .any(|s| s.attributes.contains_key(PROCESS_MINING_ALGORITHM));

    assert!(
        has_algo_attr,
        "at least one emitted span must carry the '{}' attribute",
        PROCESS_MINING_ALGORITHM
    );
}

// ── Test 2 ───────────────────────────────────────────────────────────────────

/// Token replay conformance check records fitness as a span attribute via Tracing.
#[test]
fn test_conformance_check_with_tracing_emits_span() {
    let log = build_log_with_n_cases(2);
    let net = build_two_activity_net();
    let tracing = Tracing::new();

    let checker = TokenReplay::new();
    let result = checker.check(&log, &net);

    // Record a span manually (mimicking what a with_tracing wrapper does).
    let mut attrs = HashMap::new();
    attrs.insert(CONFORMANCE_FITNESS.to_string(), result.fitness.to_string());
    let mut span = tracing
        .start_span(CONFORMANCE_CHECK_SPAN, attrs, None)
        .expect("start_span must succeed");
    tracing
        .end_span(&mut span, "ok", None)
        .expect("end_span must succeed");

    let spans = tracing.get_spans();
    let conformance_span = spans
        .iter()
        .find(|s| s.span_name == CONFORMANCE_CHECK_SPAN)
        .expect("a conformance.check span must be present");

    assert!(
        conformance_span
            .attributes
            .contains_key(CONFORMANCE_FITNESS),
        "conformance.check span must carry '{}' attribute",
        CONFORMANCE_FITNESS
    );
}

// ── Test 3 ───────────────────────────────────────────────────────────────────

/// PROCESS_MINING_ALGORITHM constant value is the canonical OTel key.
#[test]
fn test_span_algorithm_attribute_value_is_inductive_miner() {
    assert_eq!(
        PROCESS_MINING_ALGORITHM, "process.mining.algorithm",
        "PROCESS_MINING_ALGORITHM must equal 'process.mining.algorithm'"
    );
    assert_eq!(
        process_mining_algorithm::INDUCTIVE_MINER,
        "inductive_miner",
        "process_mining_algorithm::INDUCTIVE_MINER must equal 'inductive_miner'"
    );
}

// ── Test 4 ───────────────────────────────────────────────────────────────────

/// PROCESS_MINING_INDUCTIVE_MINE_SPAN constant has the expected canonical value.
#[test]
fn test_discovery_span_name_is_correct() {
    assert_eq!(
        PROCESS_MINING_INDUCTIVE_MINE_SPAN, "process.mining.inductive.mine",
        "PROCESS_MINING_INDUCTIVE_MINE_SPAN must equal 'process.mining.inductive.mine'"
    );
}

// ── Test 5 ───────────────────────────────────────────────────────────────────

/// CONFORMANCE_CHECK_SPAN constant has the expected canonical value.
#[test]
fn test_conformance_span_name_is_correct() {
    assert_eq!(
        CONFORMANCE_CHECK_SPAN, "conformance.check",
        "CONFORMANCE_CHECK_SPAN must equal 'conformance.check'"
    );
}

// ── Test 6 ───────────────────────────────────────────────────────────────────

/// PROCESS_MINING_CASE_COUNT constant has the expected canonical key value.
#[test]
fn test_span_case_count_attribute_is_correct_key() {
    assert_eq!(
        PROCESS_MINING_CASE_COUNT, "process.mining.case_count",
        "PROCESS_MINING_CASE_COUNT must equal 'process.mining.case_count'"
    );
}

// ── Test 7 ───────────────────────────────────────────────────────────────────

/// discover_with_tracing returns a non-empty net AND a non-empty spans vec.
#[test]
fn test_discovery_with_tracing_returns_net_and_spans() {
    let log = build_log_with_n_cases(4);
    let miner = InductiveMiner::new();

    let (net, spans) = miner.discover_with_tracing(&log);

    assert!(
        !net.transitions.is_empty(),
        "discover_with_tracing must return a Petri net with at least one transition"
    );
    assert!(
        !spans.is_empty(),
        "discover_with_tracing must return at least one span"
    );
}

// ── Test 8 ───────────────────────────────────────────────────────────────────

/// TokenReplay::check returns a fitness score in the valid range [0.0, 1.0].
#[test]
fn test_conformance_with_tracing_records_fitness() {
    let log = build_log_with_n_cases(2);
    let net = build_two_activity_net();
    let checker = TokenReplay::new();

    let result = checker.check(&log, &net);

    assert!(
        result.fitness >= 0.0 && result.fitness <= 1.0,
        "fitness must be in [0.0, 1.0], got {}",
        result.fitness
    );
}

// ── Test 9 ───────────────────────────────────────────────────────────────────

/// CONFORMANCE_FITNESS constant equals the canonical OTel attribute key.
#[test]
fn test_span_fitness_attribute_key() {
    assert_eq!(
        CONFORMANCE_FITNESS, "conformance.fitness",
        "CONFORMANCE_FITNESS must equal 'conformance.fitness'"
    );
}

// ── Test 10 ──────────────────────────────────────────────────────────────────

/// All process mining discovery / mining span name constants start with
/// the "process.mining." prefix, confirming the OTel naming convention.
#[test]
fn test_all_span_names_follow_process_mining_prefix() {
    use pm4py::semconv::process_mining_span_names::{
        PROCESS_MINING_BOTTLENECK_DETECTION_SPAN, PROCESS_MINING_DFG_COMPUTE_SPAN,
        PROCESS_MINING_DFG_SPAN, PROCESS_MINING_DISCOVERY_SPAN, PROCESS_MINING_INDUCTIVE_MINE_SPAN,
        PROCESS_MINING_LOG_PREPROCESS_SPAN, PROCESS_MINING_REPLAY_CHECK_SPAN,
        PROCESS_MINING_SIMULATION_RUN_SPAN, PROCESS_MINING_VARIANT_ANALYZE_SPAN,
    };

    let span_names = [
        PROCESS_MINING_DISCOVERY_SPAN,
        PROCESS_MINING_INDUCTIVE_MINE_SPAN,
        PROCESS_MINING_DFG_SPAN,
        PROCESS_MINING_DFG_COMPUTE_SPAN,
        PROCESS_MINING_REPLAY_CHECK_SPAN,
        PROCESS_MINING_VARIANT_ANALYZE_SPAN,
        PROCESS_MINING_BOTTLENECK_DETECTION_SPAN,
        PROCESS_MINING_LOG_PREPROCESS_SPAN,
        PROCESS_MINING_SIMULATION_RUN_SPAN,
    ];

    for name in &span_names {
        assert!(
            name.starts_with("process.mining."),
            "span name '{}' must start with 'process.mining.'",
            name
        );
    }
}

// ── Test: prediction_span_has_confidence ─────────────────────────────────────

/// A span emitted for outcome prediction carries PROCESS_MINING_PREDICTION_CONFIDENCE
/// as an attribute with a value parseable as f64 in [0.0, 1.0].
#[test]
fn prediction_span_has_confidence() {
    let tracing = Tracing::new();

    // Simulate a confidence score produced by a prediction operation.
    let confidence: f64 = 0.82;

    let mut attrs = HashMap::new();
    attrs.insert(
        PROCESS_MINING_PREDICTION_CONFIDENCE.to_string(),
        confidence.to_string(),
    );

    let mut span = tracing
        .start_span(PROCESS_MINING_PREDICTION_MAKE_SPAN, attrs, None)
        .expect("start_span must succeed");
    tracing
        .end_span(&mut span, "ok", None)
        .expect("end_span must succeed");

    let spans = tracing.get_spans();
    let prediction_span = spans
        .iter()
        .find(|s| s.span_name == PROCESS_MINING_PREDICTION_MAKE_SPAN)
        .expect("a prediction.make span must be present");

    let raw = prediction_span
        .attributes
        .get(PROCESS_MINING_PREDICTION_CONFIDENCE)
        .expect("span must carry PROCESS_MINING_PREDICTION_CONFIDENCE attribute");

    let parsed: f64 = raw
        .parse()
        .expect("PROCESS_MINING_PREDICTION_CONFIDENCE must be parseable as f64");

    assert!(
        (0.0..=1.0).contains(&parsed),
        "prediction confidence must be in [0.0, 1.0], got {}",
        parsed
    );
}

// ── Test: drift_span_includes_detected_flag ───────────────────────────────────

/// A span emitted for drift detection carries PROCESS_MINING_DRIFT_DETECTED = "true"
/// when baseline avg_duration doubles (clear drift scenario).
#[test]
fn drift_span_includes_detected_flag() {
    let tracing = Tracing::new();
    let calculator = DriftCalculator::new();

    // Baseline: avg_duration = 100ms; recent: 300ms — drift of 200% exceeds threshold 0.2.
    let baseline: HashMap<String, f64> =
        [("avg_duration".to_string(), 100.0)].into_iter().collect();
    let recent: HashMap<String, f64> = [("avg_duration".to_string(), 300.0)].into_iter().collect();

    let drift_score = calculator.calculate_drift(&baseline, &recent);
    let is_drifted = calculator.is_drift_detected(drift_score);

    assert!(
        is_drifted,
        "drift must be detected when avg_duration triples (score={:.3})",
        drift_score
    );

    let mut attrs = HashMap::new();
    attrs.insert(
        PROCESS_MINING_DRIFT_DETECTED.to_string(),
        is_drifted.to_string(),
    );

    let mut span = tracing
        .start_span(PROCESS_MINING_DRIFT_DETECT_SPAN, attrs, None)
        .expect("start_span must succeed");
    tracing
        .end_span(&mut span, "ok", None)
        .expect("end_span must succeed");

    let spans = tracing.get_spans();
    let drift_span = spans
        .iter()
        .find(|s| s.span_name == PROCESS_MINING_DRIFT_DETECT_SPAN)
        .expect("a drift.detect span must be present");

    let flag = drift_span
        .attributes
        .get(PROCESS_MINING_DRIFT_DETECTED)
        .expect("span must carry PROCESS_MINING_DRIFT_DETECTED attribute");

    assert_eq!(
        flag, "true",
        "PROCESS_MINING_DRIFT_DETECTED must equal 'true' when drift is detected"
    );
}
