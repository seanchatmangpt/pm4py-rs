//! "No AI Without PI" — van der Aalst et al. — Unit Tests
//!
//! Five Chicago TDD tests, one per paper connection between
//! Object-Centric Process Mining (OCPM) and AI.
//!
//! Paper: "No AI Without PI: Process Intelligence as a Key Enabler of Responsible AI"
//!        Wil van der Aalst et al., 2024.
//!
//! Connection map:
//!   1. OCED as training data for predictive AI
//!      → OcelTypedValue enum covers all OCEL 2.0 attribute types
//!   2. Process models feeding prescriptive AI
//!      → discover_ocdfg() produces non-empty edge graph from event log
//!   3. Classical optimization from process constraints
//!      → detect_bottlenecks() ranks edges by severity_score
//!   4. GenAI/RAG grounded in OCEL context
//!      → abstract_ocel() produces structured "=== PROCESS CONTEXT ===" block
//!   5. GenAI for data preparation
//!      → GroqRequestConfig::for_ocel_query() has larger token budget than Default

use chrono::{Duration, Utc};
use pm4py::llm::abstract_ocel::abstract_ocel;
use pm4py::llm::groq_client::GroqRequestConfig;
use pm4py::ocpm::object_log::{
    EventToObjectMapping, Object, ObjectCentricEventLog, ObjectType, OcelTypedValue,
};
use pm4py::ocpm::oc_dfg::discover_ocdfg;
use pm4py::ocpm::oc_performance::detect_bottlenecks;
use uuid::Uuid;

// ── Test fixture ──────────────────────────────────────────────────────────────

/// Build a minimal ObjectCentricEventLog suitable for all 5 tests.
///
/// Contains one "order" object with two consecutive events:
///   - "create_order"  at T+0s
///   - "approve_order" at T+300s (5-minute gap — the bottleneck)
///
/// This is enough to trigger a directly-follows edge and a bottleneck report.
fn build_minimal_log() -> ObjectCentricEventLog {
    let mut log = ObjectCentricEventLog::new();
    let t0 = Utc::now();
    let t1 = t0 + Duration::seconds(300);

    let order_type = ObjectType::new("order");
    log.add_object(Object::new("o1", order_type, t0));

    let e1 = Uuid::new_v4();
    let e2 = Uuid::new_v4();
    log.add_event(e1, "create_order", t0, None);
    log.add_event(e2, "approve_order", t1, None);

    let mut m1 = EventToObjectMapping::new(e1);
    m1.add_object("o1");
    log.add_event_object_mapping(m1);

    let mut m2 = EventToObjectMapping::new(e2);
    m2.add_object("o1");
    log.add_event_object_mapping(m2);

    log
}

// ── Connection 1: OCED as training data for predictive AI ────────────────────

/// Connection 1: The OCEL 2.0 standard requires typed event attributes so that
/// the event log can serve as a richly typed training dataset for predictive AI
/// (e.g., next-activity prediction, remaining time estimation).
///
/// Proof: OcelTypedValue covers all five OCEL 2.0 primitive types.
/// A missing variant would prevent typed attribute round-trips and break
/// the feature-engineering pipeline that feeds predictive models.
#[test]
fn connection_1_ocel_as_training_data_typed_attributes() {
    // All five OCEL 2.0 primitive attribute types must be representable.
    let string_val = OcelTypedValue::String("process-mining".to_string());
    let int_val = OcelTypedValue::Integer(42_i64);
    let float_val = OcelTypedValue::Float(3.14_f64);
    let bool_val = OcelTypedValue::Boolean(true);
    let ts_val = OcelTypedValue::Timestamp(Utc::now());

    // Each variant must round-trip through its enum arm (no data loss).
    assert!(matches!(string_val, OcelTypedValue::String(_)));
    assert!(matches!(int_val, OcelTypedValue::Integer(_)));
    assert!(matches!(float_val, OcelTypedValue::Float(_)));
    assert!(matches!(bool_val, OcelTypedValue::Boolean(_)));
    assert!(matches!(ts_val, OcelTypedValue::Timestamp(_)));

    // as_str() must return Some only for String variant — type safety for feature extractors.
    assert!(
        string_val.as_str().is_some(),
        "String variant must expose as_str()"
    );
    assert!(
        int_val.as_str().is_none(),
        "Integer variant must NOT expose as_str()"
    );
    assert!(
        float_val.as_str().is_none(),
        "Float variant must NOT expose as_str()"
    );

    // Objects can store typed attributes — the training data schema is enforced at compile time.
    let t = Utc::now();
    let order_type = ObjectType::new("order");
    let obj = Object::new("o1", order_type, t)
        .with_attribute("priority", OcelTypedValue::Integer(1))
        .with_attribute("region", OcelTypedValue::String("EMEA".to_string()))
        .with_attribute("amount", OcelTypedValue::Float(1250.50));

    // Typed attribute storage must preserve values without casting.
    assert!(
        matches!(
            obj.attributes.get("priority"),
            Some(OcelTypedValue::Integer(1))
        ),
        "Integer attribute must be stored as Integer variant"
    );
    assert!(
        obj.attributes.get("region").and_then(|v| v.as_str()) == Some("EMEA"),
        "String attribute must be retrievable via as_str()"
    );
}

// ── Connection 2: Process models feeding prescriptive AI ─────────────────────

/// Connection 2: discover_ocdfg() produces an Object-Centric DFG (one node per
/// object-type/activity pair, one edge per directly-follows pair).  The DFG is
/// the process model that feeds prescriptive AI — it tells the AI *what paths
/// exist*, enabling recommendations like "what should happen next?".
///
/// Proof: a two-event log with one object produces exactly one edge.
#[test]
fn connection_2_process_models_discover_ocdfg() {
    let log = build_minimal_log();

    let dfg = discover_ocdfg(&log);

    // Must have exactly one directly-follows edge: create_order → approve_order.
    assert_eq!(
        dfg.total_edge_count(),
        1,
        "Two-event log must produce exactly one OCDFG edge (create_order → approve_order)"
    );

    // The edge must be attributed to the correct object type.
    let order_edges = dfg
        .edges
        .get("order")
        .expect("OCDFG must have an edge set for object type 'order'");

    assert!(
        order_edges.contains_key(&("create_order".to_string(), "approve_order".to_string())),
        "Edge create_order → approve_order must be present for object type 'order'"
    );

    // The DFG must report the object type it contains.
    assert!(
        dfg.object_types().contains(&"order"),
        "object_types() must list 'order'"
    );
}

// ── Connection 3: Classical optimization from process constraints ─────────────

/// Connection 3: detect_bottlenecks() applies classical operations-research
/// optimization — ranking edges by severity (mean_wait × ln(frequency+1)) —
/// to identify where the process is slowest relative to its volume.
///
/// Proof: a 300-second gap between create_order and approve_order must
/// rank as the top (and only) bottleneck with severity_score > 0.
#[test]
fn connection_3_optimization_detect_bottlenecks() {
    let log = build_minimal_log();

    let bottlenecks = detect_bottlenecks(&log, 3);

    assert_eq!(
        bottlenecks.len(),
        1,
        "Single-edge log must produce exactly one bottleneck report"
    );

    let top = &bottlenecks[0];

    assert_eq!(
        top.object_type, "order",
        "Bottleneck must belong to object type 'order'"
    );
    assert_eq!(
        top.from_activity, "create_order",
        "Bottleneck source must be 'create_order'"
    );
    assert_eq!(
        top.to_activity, "approve_order",
        "Bottleneck target must be 'approve_order'"
    );

    // Mean wait must be approximately 300 seconds (the fixture gap).
    assert!(
        top.mean_wait_secs >= 299.0 && top.mean_wait_secs <= 301.0,
        "Mean wait must be ~300s, got {}",
        top.mean_wait_secs
    );

    // Severity score must be positive — severity = mean_wait × ln(freq+1).
    assert!(
        top.severity_score > 0.0,
        "Severity score must be positive, got {}",
        top.severity_score
    );

    // Verify top_n limit is respected.
    let many = detect_bottlenecks(&log, 100);
    assert!(
        many.len() <= 100,
        "detect_bottlenecks must respect top_n limit"
    );
}

// ── Connection 4: GenAI/RAG grounded in OCEL context ─────────────────────────

/// Connection 4: abstract_ocel() converts an ObjectCentricEventLog into a
/// structured plain-text block that is injected as the RAG system message.
/// This grounds the GenAI answer in *real process data*, not hallucination.
///
/// Proof: the output must contain the canonical "=== PROCESS CONTEXT ===" header,
/// the object type name, the total event count, and the activity names — all the
/// information a RAG-grounded LLM needs to answer process questions correctly.
#[test]
fn connection_4_rag_abstract_ocel_produces_context() {
    let log = build_minimal_log();

    let context = abstract_ocel(&log);

    // Must contain the canonical context header used by query_with_ocel_llm().
    assert!(
        context.contains("=== PROCESS CONTEXT ==="),
        "abstract_ocel() must start with '=== PROCESS CONTEXT ==='"
    );
    assert!(
        context.contains("=== END CONTEXT ==="),
        "abstract_ocel() must end with '=== END CONTEXT ==='"
    );

    // Must expose the object type so the LLM can reason about it.
    assert!(
        context.contains("order"),
        "context must mention object type 'order'"
    );

    // Must expose actual event count for factual grounding.
    assert!(
        context.contains("Total events: 2"),
        "context must report 'Total events: 2', got:\n{}",
        context
    );

    // Must include activity names — essential for next-step recommendations.
    assert!(
        context.contains("create_order"),
        "context must mention activity 'create_order'"
    );
    assert!(
        context.contains("approve_order"),
        "context must mention activity 'approve_order'"
    );

    // Context must be non-trivial (at least 50 characters).
    assert!(
        context.len() >= 50,
        "abstract_ocel() context is too short ({} chars) to be useful",
        context.len()
    );
}

// ── Connection 5: GenAI for data preparation ─────────────────────────────────

/// Connection 5: GroqRequestConfig::for_ocel_query() is the specialized config
/// used when the LLM must process a full OCEL context (potentially thousands of
/// tokens).  It must have a larger token budget than Default so that the GenAI
/// model can prepare data summaries without being cut off mid-answer.
///
/// Proof: for_ocel_query().max_completion_tokens >= 2048 > Default's 512.
/// A lower value would truncate OCEL context windows and break data preparation.
#[test]
fn connection_5_data_preparation_groq_config_for_ocel_query() {
    let default_config = GroqRequestConfig::default();
    let ocel_config = GroqRequestConfig::for_ocel_query();

    // for_ocel_query must have strictly more tokens than the default.
    assert!(
        ocel_config.max_completion_tokens > default_config.max_completion_tokens,
        "for_ocel_query() max_completion_tokens ({}) must exceed Default ({})",
        ocel_config.max_completion_tokens,
        default_config.max_completion_tokens
    );

    // The paper requires OCEL context to fit in a single completion — 2048 is the
    // minimum meaningful budget for a full OCEL summary + answer.
    assert!(
        ocel_config.max_completion_tokens >= 2048,
        "for_ocel_query() must allow at least 2048 completion tokens for OCEL context, \
         got {}",
        ocel_config.max_completion_tokens
    );

    // Temperature must be lower for factual grounding (Connection 4 prerequisite).
    assert!(
        ocel_config.temperature < default_config.temperature,
        "for_ocel_query() temperature ({}) must be lower than Default ({}) for factual grounding",
        ocel_config.temperature,
        default_config.temperature
    );

    // Timeout must be longer to accommodate larger payloads.
    assert!(
        ocel_config.timeout_secs > default_config.timeout_secs,
        "for_ocel_query() timeout_secs ({}) must exceed Default ({})",
        ocel_config.timeout_secs,
        default_config.timeout_secs
    );
}
