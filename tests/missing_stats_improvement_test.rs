//! Chicago TDD: RED → GREEN tests for improved missing_stats functions.
//!
//! Test 1: embeddings_similarity uses cosine over frequency vectors
//!   - Jaccard returns 1.0 for logs with identical activity SETS but different frequencies
//!   - Cosine correctly returns < 1.0 (≈ 0.198) for the skewed-frequency case
//!
//! Test 2: extract_ocel_features returns per-object-type vectors with 4 features each

use chrono::Utc;
use pm4py::log::{Event, EventLog, Trace};
use pm4py::ocpm::object_log::{Object, ObjectCentricEventLog, ObjectType};
use pm4py::statistics::{embeddings_similarity, extract_ocel_features};

// ─── helpers ────────────────────────────────────────────────────────────────

fn make_event(activity: &str) -> Event {
    Event::new(activity, Utc::now())
}

fn make_trace(activities: &[&str]) -> Trace {
    let mut t = Trace::new(uuid::Uuid::new_v4().to_string());
    for a in activities {
        t.events.push(make_event(a));
    }
    t
}

// ─── Test 1: cosine similarity distinguishes frequency differences ───────────

/// RED assertion: Jaccard returns 1.0 for these logs (both have activities {A, B}).
/// After the fix, cosine must return a value < 0.9.
///
/// Log1: 10× A, 1× B  →  freq vector [10, 1]
/// Log2:  1× A, 10× B →  freq vector [ 1, 10]
/// Cosine = dot / (|v1| * |v2|)
///        = (10·1 + 1·10) / (√101 · √101)
///        = 20 / 101
///        ≈ 0.198
#[test]
fn test_cosine_similarity_skewed_frequency_logs_differ() {
    // Build Log1: 10× "A", 1× "B"
    let mut log1 = EventLog::new();
    for _ in 0..10 {
        log1.traces.push(make_trace(&["A"]));
    }
    log1.traces.push(make_trace(&["B"]));

    // Build Log2: 1× "A", 10× "B"
    let mut log2 = EventLog::new();
    log2.traces.push(make_trace(&["A"]));
    for _ in 0..10 {
        log2.traces.push(make_trace(&["B"]));
    }

    let sim = embeddings_similarity(&log1, &log2);

    // Jaccard would return 1.0 here because both logs have the same activity set {A, B}.
    // Cosine must return < 0.9 because the frequency distribution is very skewed.
    assert!(
        sim < 0.9,
        "cosine similarity of frequency-skewed logs should be < 0.9, got {sim:.4}"
    );

    // Also verify the value is in a reasonable range (not negative, not > 1)
    assert!(sim >= 0.0, "similarity must be ≥ 0.0, got {sim:.4}");
    assert!(sim <= 1.0, "similarity must be ≤ 1.0, got {sim:.4}");
}

/// Identical logs must still return 1.0 with cosine (regression guard).
#[test]
fn test_cosine_similarity_identical_logs_is_one() {
    let mut log = EventLog::new();
    log.traces.push(make_trace(&["A", "B", "C"]));
    log.traces.push(make_trace(&["A", "C"]));

    let sim = embeddings_similarity(&log, &log);
    assert!(
        (sim - 1.0).abs() < 1e-9,
        "identical logs must have similarity 1.0, got {sim:.9}"
    );
}

/// Empty logs should return 0.0 (both vectors are zero → cosine undefined → 0.0).
#[test]
fn test_cosine_similarity_empty_logs_returns_zero() {
    let log1 = EventLog::new();
    let log2 = EventLog::new();
    let sim = embeddings_similarity(&log1, &log2);
    assert_eq!(sim, 0.0, "two empty logs should return 0.0");
}

// ─── Test 2: per-object-type 4-feature vectors ──────────────────────────────

/// RED assertion: old stub returns vec![vec![objects, events, types]] — a single
/// 3-element vector regardless of object type count.
/// After the fix, we must get one vector per object type, each with 4 features.
#[test]
fn test_extract_ocel_features_per_object_type_with_4_features() {
    let now = Utc::now();

    let order_type = ObjectType::new("order");
    let item_type = ObjectType::new("item");

    let mut ocel = ObjectCentricEventLog::new();

    // Register object types
    ocel.object_types.insert(order_type.clone());
    ocel.object_types.insert(item_type.clone());

    // Add 2 order objects
    ocel.objects.insert(
        "order-1".to_string(),
        Object::new("order-1", order_type.clone(), now),
    );
    ocel.objects.insert(
        "order-2".to_string(),
        Object::new("order-2", order_type.clone(), now),
    );

    // Add 3 item objects
    ocel.objects.insert(
        "item-1".to_string(),
        Object::new("item-1", item_type.clone(), now),
    );
    ocel.objects.insert(
        "item-2".to_string(),
        Object::new("item-2", item_type.clone(), now),
    );
    ocel.objects.insert(
        "item-3".to_string(),
        Object::new("item-3", item_type.clone(), now),
    );

    // Add some events
    let ev1 = uuid::Uuid::new_v4();
    let ev2 = uuid::Uuid::new_v4();
    ocel.events
        .insert(ev1, ("place_order".to_string(), now, None));
    ocel.events
        .insert(ev2, ("ship_item".to_string(), now, None));

    let features = extract_ocel_features(&ocel);

    // Must return one vector per object type (2 types → 2 vectors)
    assert_eq!(
        features.len(),
        2,
        "should return one feature vector per object type, got {} vectors",
        features.len()
    );

    // Each vector must have exactly 4 features
    for (i, vec) in features.iter().enumerate() {
        assert_eq!(
            vec.len(),
            4,
            "feature vector {i} must have 4 features, got {}",
            vec.len()
        );
    }
}

/// Empty OCEL (no object types) should return a single fallback vector with 4 features.
#[test]
fn test_extract_ocel_features_empty_ocel_returns_fallback_with_4_features() {
    let ocel = ObjectCentricEventLog::new();
    let features = extract_ocel_features(&ocel);

    assert_eq!(
        features.len(),
        1,
        "empty OCEL should return 1 fallback vector"
    );
    assert_eq!(
        features[0].len(),
        4,
        "fallback vector must have 4 features, got {}",
        features[0].len()
    );
}
