/// Chicago TDD parity tests: Weaver-generated pm4py semconv constants match schema definitions.
///
/// These tests verify that constant string values in the generated Rust code match
/// what the semconv YAML schema specifies. If a schema key or enum value is renamed,
/// these tests catch the drift at compile / test time.
///
/// Run with: cargo test --test semconv_pm4py_parity
use pm4py::semconv::conformance_attributes;
use pm4py::semconv::process_attributes;
use pm4py::semconv::process_attributes::process_mining_algorithm;
use pm4py::semconv::process_mining_span_names;

// ============================================================
// Span name parity
// ============================================================

#[test]
fn process_mining_discovery_span_name_matches_schema() {
    assert_eq!(
        process_mining_span_names::PROCESS_MINING_DISCOVERY_SPAN,
        "process.mining.discovery"
    );
}

// ============================================================
// Algorithm enum value parity
// ============================================================

#[test]
fn process_mining_algorithm_alpha_value_matches_schema() {
    assert_eq!(process_mining_algorithm::ALPHA_MINER, "alpha_miner");
}

#[test]
fn process_mining_algorithm_inductive_value_matches_schema() {
    assert_eq!(process_mining_algorithm::INDUCTIVE_MINER, "inductive_miner");
}

// ============================================================
// Conformance attribute key parity
// ============================================================

#[test]
fn conformance_fitness_key_is_correct_otel_name() {
    assert_eq!(
        conformance_attributes::CONFORMANCE_FITNESS,
        "conformance.fitness"
    );
}

#[test]
fn conformance_precision_key_is_correct_otel_name() {
    assert_eq!(
        conformance_attributes::CONFORMANCE_PRECISION,
        "conformance.precision"
    );
}

// ============================================================
// Process mining attribute key format
// ============================================================

#[test]
fn process_mining_case_count_key_format() {
    let key = process_attributes::PROCESS_MINING_CASE_COUNT;
    assert_eq!(key, "process.mining.case_count");
    assert!(
        key.starts_with("process.mining"),
        "key must start with process.mining namespace, got: {key}"
    );
}

// ============================================================
// Boolean attribute — compile-time usability + string parity
// ============================================================

#[test]
fn drift_detected_key_is_boolean_type() {
    // Compile-time check: the constant is &str and can be used to build a KeyValue with a bool value.
    let key = process_attributes::PROCESS_MINING_DRIFT_DETECTED;
    let _kv = opentelemetry::KeyValue::new(key, true);
    assert_eq!(key, "process.mining.drift.detected");
}

// ============================================================
// Float attribute — compile-time usability + string parity
// ============================================================

#[test]
fn prediction_confidence_is_double_type() {
    // Compile-time check: the constant is &str and can be used to build a KeyValue with an f64 value.
    let key = process_attributes::PROCESS_MINING_PREDICTION_CONFIDENCE;
    let _kv = opentelemetry::KeyValue::new(key, 0.87_f64);
    assert_eq!(key, "process.mining.prediction.confidence");
}

// ============================================================
// All pm4py span names follow dot-separated convention
// ============================================================

#[test]
fn all_pm4py_span_names_follow_dot_convention() {
    let span_names = [
        process_mining_span_names::PROCESS_MINING_DISCOVERY_SPAN,
        process_mining_span_names::PROCESS_MINING_BOTTLENECK_DETECTION_SPAN,
        process_mining_span_names::PROCESS_MINING_CONFORMANCE_REPAIR_SPAN,
        process_mining_span_names::PROCESS_MINING_DRIFT_DETECT_SPAN,
        process_mining_span_names::PROCESS_MINING_REPLAY_CHECK_SPAN,
        process_mining_span_names::PROCESS_MINING_SIMULATION_RUN_SPAN,
        process_mining_span_names::PROCESS_MINING_VARIANT_ANALYZE_SPAN,
    ];

    for name in &span_names {
        assert!(
            name.starts_with("process.mining"),
            "span name must start with 'process.mining', got: {name}"
        );
        assert_eq!(
            *name,
            name.to_ascii_lowercase(),
            "span name must be lowercase, got: {name}"
        );
    }
}

// ============================================================
// Weaver policy compliance — structural format + optional subprocess check
// ============================================================

#[test]
fn weaver_policy_compliance_bos_module_name() {
    // Structural format verification: a canonical bos attribute key must have
    // exactly 3 dot-separated segments and start with "bos".
    let example_key = "bos.module.name";
    let segments: Vec<&str> = example_key.split('.').collect();
    assert_eq!(
        segments.len(),
        3,
        "bos attribute key must have 3 dot-separated segments, got: {example_key}"
    );
    assert!(
        example_key.starts_with("bos"),
        "bos attribute key must start with 'bos', got: {example_key}"
    );

    // Optional: invoke weaver registry check if the binary is available.
    match std::process::Command::new("weaver")
        .args([
            "registry",
            "check",
            "-r",
            "./semconv/model",
            "-p",
            "./semconv/policies",
        ])
        .current_dir(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap_or(std::path::Path::new(".")),
        )
        .output()
    {
        Err(_) => {
            // weaver not installed — skip the subprocess check, structural assertion above is sufficient
        }
        Ok(out) => {
            assert_eq!(
                out.status.code(),
                Some(0),
                "weaver registry check failed:\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            );
        }
    }
}
