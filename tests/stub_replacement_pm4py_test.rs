use chrono::{Duration as ChronoDuration, Utc};
/// Wave 1 Stub Replacement Tests — Chicago TDD (RED → GREEN → REFACTOR)
///
/// Tests cover:
///   PM4-M3: bucketed confidence interval for remaining_time prediction
///   PM4-L1: export_parquet writes real rows (non-empty file)
///   PM4-L2: read_ocel2_sqlite / write_ocel2_sqlite round-trip
///   PM4-L3: ocel_merge_duplicates removes exact duplicates, keeps distinct events
///   PM4-H1: precision BFS over full reachable state space on a linear A→B→C net
use pm4py::audit::export::{AuditExport, ExportFormat};
use pm4py::audit::hash_chain::HashChainEntry;
use pm4py::conformance::precision::Precision;
use pm4py::io::ocel2_io::{read_ocel2_sqlite, write_ocel2_sqlite};
use pm4py::log::{Event, EventLog, Trace};
use pm4py::models::petri_net::{Arc, Place, Transition};
use pm4py::models::PetriNet;
use pm4py::ocpm::ocel_utils::ocel_merge_duplicates;
use pm4py::ocpm::{Object, ObjectCentricEventLog, ObjectType};
use pm4py::predictive::remaining_time::RemainingTimePredictor;
use tempfile::NamedTempFile;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_linear_log_abc() -> EventLog {
    let mut log = EventLog::new();
    let now = Utc::now();
    let mut trace = Trace::new("case_1");
    trace.add_event(Event::new("a", now));
    trace.add_event(Event::new("b", now));
    trace.add_event(Event::new("c", now));
    log.add_trace(trace);
    log
}

fn make_linear_net_abc() -> PetriNet {
    let mut net = PetriNet::new();
    let p1 = Place::new("p1").with_initial_marking(1);
    let t1 = Transition::new("t1").with_label("a");
    let p2 = Place::new("p2");
    let t2 = Transition::new("t2").with_label("b");
    let p3 = Place::new("p3");
    let t3 = Transition::new("t3").with_label("c");
    let p4 = Place::new("p4").with_final_marking(1);

    let p1_id = p1.id.clone();
    let t1_id = t1.id.clone();
    let p2_id = p2.id.clone();
    let t2_id = t2.id.clone();
    let p3_id = p3.id.clone();
    let t3_id = t3.id.clone();
    let p4_id = p4.id.clone();

    net.add_place(p1);
    net.add_transition(t1);
    net.add_place(p2);
    net.add_transition(t2);
    net.add_place(p3);
    net.add_transition(t3);
    net.add_place(p4);

    net.add_arc(Arc::new(&p1_id, &t1_id));
    net.add_arc(Arc::new(&t1_id, &p2_id));
    net.add_arc(Arc::new(&p2_id, &t2_id));
    net.add_arc(Arc::new(&t2_id, &p3_id));
    net.add_arc(Arc::new(&p3_id, &t3_id));
    net.add_arc(Arc::new(&t3_id, &p4_id));
    net.set_initial_place(p1_id);
    net.set_final_place(p4_id);
    net
}

fn make_audit_entries() -> Vec<HashChainEntry> {
    vec![
        HashChainEntry {
            sequence_number: 1,
            event_id: Uuid::new_v4(),
            timestamp: Utc::now().to_rfc3339(),
            event_type: "model_discovered".to_string(),
            event_category: "ProcessMining".to_string(),
            previous_hash: "0".repeat(64),
            entry_hash: "a".repeat(64),
            payload: serde_json::json!({"algorithm": "alpha"}),
        },
        HashChainEntry {
            sequence_number: 2,
            event_id: Uuid::new_v4(),
            timestamp: Utc::now().to_rfc3339(),
            event_type: "conformance_checked".to_string(),
            event_category: "ProcessMining".to_string(),
            previous_hash: "a".repeat(64),
            entry_hash: "b".repeat(64),
            payload: serde_json::json!({"fitness": 0.95}),
        },
    ]
}

// ---------------------------------------------------------------------------
// PM4-M3: bucketed confidence interval
// ---------------------------------------------------------------------------

/// A case with the same activity prefix as training cases should produce a
/// narrower confidence interval than one computed from the overall std_dev.
///
/// Training data: two clusters
///   - prefix ["fast"] → total durations ~600 s (small spread)
///   - prefix ["slow"] → total durations ~3600 s (large spread)
///
/// For a case that has already done "fast", the prefix-bucketed CI width
/// must be strictly smaller than an interval computed from overall std_dev
/// (which mixes both clusters).
#[test]
fn test_pm4m3_bucketed_confidence_interval_narrower_for_known_prefix() {
    let mut log = EventLog::new();
    let start = Utc::now();

    // 5 "fast" traces: activities ["fast"], total ~600 s
    for i in 0..5 {
        let mut t = Trace::new(format!("fast_{}", i));
        t.add_event(Event::new("fast", start));
        t.add_event(Event::new(
            "end",
            start + ChronoDuration::seconds(600 + i * 10),
        ));
        log.add_trace(t);
    }

    // 5 "slow" traces: activities ["slow"], total ~3600 s
    for i in 0..5 {
        let mut t = Trace::new(format!("slow_{}", i));
        t.add_event(Event::new("slow", start));
        t.add_event(Event::new(
            "end",
            start + ChronoDuration::seconds(3600 + i * 100),
        ));
        log.add_trace(t);
    }

    let predictor = RemainingTimePredictor::new(&log);

    // Partial trace with prefix ["fast"]
    let mut partial = Trace::new("ongoing_fast");
    partial.add_event(Event::new("fast", start));

    let pred = predictor
        .predict_remaining_time(&partial, None)
        .expect("should produce a prediction for non-empty log");

    // The CI width when using the bucketed approach must be ≥ 0 and
    // the interval must be consistent (upper ≥ lower ≥ 0).
    assert!(
        pred.confidence_lower >= 0.0,
        "lower bound must be non-negative"
    );
    assert!(
        pred.confidence_upper >= pred.confidence_lower,
        "upper must be ≥ lower"
    );
    assert!(pred.confidence > 0.0, "confidence score must be positive");

    // Key behavioral assertion: the CI width for a case whose prefix maps to
    // the "fast" bucket must be narrower than if we used the overall std_dev
    // (which spans both fast ~600 s and slow ~3600 s clusters).
    //
    // Overall std_dev ≈ sqrt(variance of [600..640, 3600..4000]) >> 1000 s
    // Bucketed std_dev for "fast" ≈ sqrt(variance of [600..640]) << 50 s
    //
    // So prefix-bucketed CI width (upper - lower) must be < 200 s.
    // The overall-std-dev CI would be > 2000 s wide (1.96 * ~1500 s).
    let ci_width = pred.confidence_upper - pred.confidence_lower;
    assert!(
        ci_width < 200.0,
        "bucketed CI width ({:.1} s) should be < 200 s for the 'fast' prefix cluster; \
         the old code would produce a CI > 2000 s wide",
        ci_width
    );
}

// ---------------------------------------------------------------------------
// PM4-L1: export_parquet writes non-empty file
// ---------------------------------------------------------------------------

/// export_parquet must write a Parquet file that contains at least one row
/// group with the expected number of rows.  We verify by checking the byte
/// length of the output (a valid Parquet file with rows is always > the
/// minimal empty-file header, which is ~12 bytes) and by using the parquet
/// reader to confirm the row count.
#[test]
fn test_pm4l1_export_parquet_writes_real_rows() {
    let entries = make_audit_entries();

    let bytes =
        AuditExport::export(&entries, ExportFormat::Parquet).expect("export_parquet must not fail");

    // An empty Parquet file (header+footer with no rows) is typically < 200
    // bytes.  A file with two actual rows should be noticeably larger.
    assert!(
        bytes.len() > 200,
        "Parquet output ({} bytes) is too small — looks like an empty row group",
        bytes.len()
    );

    // Verify the magic bytes for a valid Parquet file: "PAR1" at start and end.
    assert_eq!(&bytes[..4], b"PAR1", "missing Parquet magic bytes at start");
    assert_eq!(
        &bytes[bytes.len() - 4..],
        b"PAR1",
        "missing Parquet magic bytes at end"
    );

    // Use parquet crate to read back the row count via a temp file.
    use parquet::file::reader::{FileReader, SerializedFileReader};
    let tmp = NamedTempFile::new().expect("tempfile creation must succeed");
    std::fs::write(tmp.path(), &bytes).expect("write parquet to temp file");
    let f = std::fs::File::open(tmp.path()).expect("open parquet temp file");
    let reader = SerializedFileReader::new(f).expect("parquet reader must open");
    let metadata = reader.metadata();
    let total_rows: i64 = (0..metadata.num_row_groups())
        .map(|i| metadata.row_group(i).num_rows())
        .sum();
    assert_eq!(
        total_rows,
        entries.len() as i64,
        "Parquet file must contain exactly {} rows (one per audit entry)",
        entries.len()
    );
}

// ---------------------------------------------------------------------------
// PM4-L2: ocel2 sqlite round-trip
// ---------------------------------------------------------------------------

/// write_ocel2_sqlite followed by read_ocel2_sqlite must return the same
/// number of events with the same activity names and same objects.
#[test]
fn test_pm4l2_ocel2_sqlite_roundtrip() {
    let tmp = NamedTempFile::new().expect("tempfile creation must succeed");
    let path = tmp.path().with_extension("sqlite");

    // Build an OCEL with 3 events and 2 objects.
    let mut ocel = ObjectCentricEventLog::new();
    let order_type = ObjectType::new("Order");
    let now = Utc::now();

    let obj1 = Object::new("order_001", order_type.clone(), now);
    let obj2 = Object::new("order_002", order_type.clone(), now);
    ocel.add_object(obj1);
    ocel.add_object(obj2);

    let e1 = Uuid::new_v4();
    let e2 = Uuid::new_v4();
    let e3 = Uuid::new_v4();
    ocel.add_event(e1, "place_order", now, None);
    ocel.add_event(e2, "confirm_order", now + ChronoDuration::minutes(5), None);
    ocel.add_event(e3, "ship_order", now + ChronoDuration::minutes(30), None);

    write_ocel2_sqlite(&ocel, &path).expect("write_ocel2_sqlite must not fail");
    assert!(path.exists(), "SQLite file must be created on disk");

    let loaded = read_ocel2_sqlite(&path).expect("read_ocel2_sqlite must not fail");

    // Event count must be preserved
    assert_eq!(
        loaded.events.len(),
        3,
        "loaded OCEL must have 3 events, got {}",
        loaded.events.len()
    );

    // Object count must be preserved
    assert_eq!(
        loaded.objects.len(),
        2,
        "loaded OCEL must have 2 objects, got {}",
        loaded.objects.len()
    );

    // All three activity names must be present
    let activities: std::collections::HashSet<String> = loaded
        .events
        .values()
        .map(|(act, _, _)| act.clone())
        .collect();
    assert!(
        activities.contains("place_order"),
        "place_order must survive round-trip"
    );
    assert!(
        activities.contains("confirm_order"),
        "confirm_order must survive round-trip"
    );
    assert!(
        activities.contains("ship_order"),
        "ship_order must survive round-trip"
    );
}

// ---------------------------------------------------------------------------
// PM4-L3: ocel_merge_duplicates
// ---------------------------------------------------------------------------

/// ocel_merge_duplicates must remove exact duplicates (same activity,
/// timestamp, and sorted object_ids) and keep distinct events.
#[test]
fn test_pm4l3_merge_duplicates_removes_exact_duplicates_keeps_distinct() {
    use pm4py::ocpm::EventToObjectMapping;

    let mut ocel = ObjectCentricEventLog::new();
    let now = Utc::now();

    // Event A — will be duplicated
    let ea1 = Uuid::new_v4();
    let ea2 = Uuid::new_v4(); // exact duplicate of ea1
                              // Event B — distinct (different activity)
    let eb = Uuid::new_v4();

    ocel.add_event(ea1, "submit", now, None);
    ocel.add_event(ea2, "submit", now, None); // same activity + timestamp
    ocel.add_event(eb, "approve", now, None); // different activity

    // Give both duplicate events the same single object
    let mut m1 = EventToObjectMapping::new(ea1);
    m1.add_object("obj_1");
    ocel.add_event_object_mapping(m1);

    let mut m2 = EventToObjectMapping::new(ea2);
    m2.add_object("obj_1");
    ocel.add_event_object_mapping(m2);

    // Distinct event has a different object
    let mut m3 = EventToObjectMapping::new(eb);
    m3.add_object("obj_2");
    ocel.add_event_object_mapping(m3);

    // Before merge: 3 events
    assert_eq!(ocel.events.len(), 3);

    ocel_merge_duplicates(&mut ocel);

    // After merge: 2 events (one duplicate removed, distinct kept)
    assert_eq!(
        ocel.events.len(),
        2,
        "after merging duplicates there must be exactly 2 distinct events, got {}",
        ocel.events.len()
    );

    // The distinct activities must still be present
    let activities: std::collections::HashSet<String> = ocel
        .events
        .values()
        .map(|(act, _, _)| act.clone())
        .collect();
    assert!(activities.contains("submit"), "submit event must be kept");
    assert!(activities.contains("approve"), "approve event must be kept");
}

/// ocel_merge_duplicates on a log with no duplicates must leave event count
/// unchanged.
#[test]
fn test_pm4l3_merge_duplicates_leaves_distinct_events_unchanged() {
    let mut ocel = ObjectCentricEventLog::new();
    let now = Utc::now();

    ocel.add_event(Uuid::new_v4(), "a", now, None);
    ocel.add_event(Uuid::new_v4(), "b", now + ChronoDuration::seconds(1), None);
    ocel.add_event(Uuid::new_v4(), "c", now + ChronoDuration::seconds(2), None);

    ocel_merge_duplicates(&mut ocel);

    assert_eq!(
        ocel.events.len(),
        3,
        "no-duplicate log must keep all 3 events"
    );
}

// ---------------------------------------------------------------------------
// PM4-H1: precision BFS over full reachable state space
// ---------------------------------------------------------------------------

/// For a simple linear A→B→C net, the BFS must discover all three
/// directly-follows pairs (a→b, b→c) in the reachable state space.
/// Precision of the matching log must be > 0.5.
///
/// The old one-step extraction only fired transitions from the initial
/// marking, so it missed b→c (reachable only after firing a) and produced 0.
#[test]
fn test_pm4h1_precision_bfs_discovers_all_relations_in_linear_net() {
    let net = make_linear_net_abc();
    let relations = Precision::extract_model_relations(&net);

    // The BFS must find both a→b and b→c
    assert!(
        relations.contains(&("a".to_string(), "b".to_string())),
        "BFS must discover a→b relation; found: {:?}",
        relations
    );
    assert!(
        relations.contains(&("b".to_string(), "c".to_string())),
        "BFS must discover b→c relation; found: {:?}",
        relations
    );
    assert_eq!(
        relations.len(),
        2,
        "linear A→B→C net has exactly 2 directly-follows pairs, got {}",
        relations.len()
    );
}

/// Precision of a log that exactly matches the linear A→B→C net must be > 0.5.
#[test]
fn test_pm4h1_precision_linear_net_score_above_threshold() {
    let log = make_linear_log_abc();
    let net = make_linear_net_abc();

    let score = Precision::calculate(&log, &net);

    assert!(
        score > 0.5,
        "precision for perfectly matching linear net must be > 0.5, got {:.4}",
        score
    );
}

// ---------------------------------------------------------------------------
// PyO3 bridge tests (require Python env with pm4py installed)
// ---------------------------------------------------------------------------

/// conformance_alignments must return a non-empty AlignmentResult with
/// average_fitness in [0.0, 1.0].
///
/// For a perfectly conformant log (A→B→C exactly matches the net) the fitness
/// returned by pm4py token-based replay must be exactly 1.0.
#[test]
fn test_bridge_conformance_alignments_returns_nonzero_fitness() {
    use pm4py::conformance::alignments::conformance_alignments;

    let log = make_linear_log_abc();
    let net = make_linear_net_abc();

    let result = conformance_alignments(&log, &net);

    assert!(
        !result.alignments.is_empty(),
        "pm4py bridge must return at least one alignment entry"
    );
    assert!(
        result.average_fitness >= 0.0 && result.average_fitness <= 1.0,
        "average_fitness must be in [0.0, 1.0], got {}",
        result.average_fitness
    );
    // A perfectly conformant log (A→B→C log on A→B→C net) must score 1.0
    assert_eq!(
        result.average_fitness, 1.0,
        "perfectly conformant log must have fitness 1.0 from pm4py, got {}",
        result.average_fitness
    );
}

/// InductiveMiner::discover on a log with a branch (A→B and A→C in the same
/// log) must return a Petri net that has more than 1 transition — the real
/// inductive miner discovers parallel / exclusive splits.
#[test]
fn test_bridge_inductive_miner_discovers_branching_net() {
    use pm4py::discovery::InductiveMiner;

    // Log: trace 1 = A→B, trace 2 = A→C (exclusive choice)
    let mut log = EventLog::new();
    let now = Utc::now();

    let mut t1 = Trace::new("case_1");
    t1.add_event(Event::new("A", now));
    t1.add_event(Event::new("B", now + ChronoDuration::seconds(10)));
    log.add_trace(t1);

    let mut t2 = Trace::new("case_2");
    t2.add_event(Event::new("A", now));
    t2.add_event(Event::new("C", now + ChronoDuration::seconds(10)));
    log.add_trace(t2);

    let miner = InductiveMiner::new();
    let net = miner.discover(&log);

    assert!(
        net.transitions.len() > 1,
        "pm4py inductive miner must produce > 1 transition for a branching log, got {}",
        net.transitions.len()
    );
    assert!(
        !net.places.is_empty(),
        "pm4py inductive miner must produce at least one place"
    );
}

// ---------------------------------------------------------------------------
// PM4-H4: discover_tree delegates to real pm4py inductive miner
// ---------------------------------------------------------------------------

/// discover_tree on an XOR log (A→B, A→C) must produce a tree containing
/// all three activities A, B, and C.
#[test]
fn test_pm4h4_discover_tree_exclusive_choice_includes_both_branches() {
    use pm4py::discovery::InductiveMiner;

    let mut log = EventLog::new();
    let now = Utc::now();
    let mut t1 = Trace::new("case_1");
    t1.add_event(Event::new("A", now));
    t1.add_event(Event::new("B", now + ChronoDuration::seconds(10)));
    log.add_trace(t1);
    let mut t2 = Trace::new("case_2");
    t2.add_event(Event::new("A", now));
    t2.add_event(Event::new("C", now + ChronoDuration::seconds(10)));
    log.add_trace(t2);

    let miner = InductiveMiner::new();
    let tree = miner.discover_tree(&log);
    let activities = tree.activities();

    assert!(
        activities.iter().any(|a| a == "A"),
        "tree must contain A; got {:?}",
        activities
    );
    assert!(
        activities.iter().any(|a| a == "B"),
        "tree must contain B; got {:?}",
        activities
    );
    assert!(
        activities.iter().any(|a| a == "C"),
        "tree must contain C; got {:?}",
        activities
    );
}

/// discover_tree result must be a structurally valid ProcessTree with
/// at least 3 leaf nodes for a 3-activity linear log.
#[test]
fn test_pm4h4_discover_tree_result_is_valid_process_tree() {
    use pm4py::discovery::InductiveMiner;

    let mut log = EventLog::new();
    let now = Utc::now();
    let mut t1 = Trace::new("case_1");
    t1.add_event(Event::new("A", now));
    t1.add_event(Event::new("B", now + ChronoDuration::seconds(5)));
    t1.add_event(Event::new("C", now + ChronoDuration::seconds(10)));
    log.add_trace(t1);

    let miner = InductiveMiner::new();
    let tree = miner.discover_tree(&log);

    assert!(
        tree.is_valid(),
        "discover_tree must return a valid process tree"
    );
    assert!(
        tree.leaf_count() >= 3,
        "tree for 3-activity log must have ≥ 3 leaves, got {}",
        tree.leaf_count()
    );
}

/// discover_tree for a single-activity log must have exactly one leaf.
#[test]
fn test_pm4h4_discover_tree_single_activity_log_has_one_leaf() {
    use pm4py::discovery::InductiveMiner;

    let mut log = EventLog::new();
    let now = Utc::now();
    let mut t1 = Trace::new("case_1");
    t1.add_event(Event::new("A", now));
    log.add_trace(t1);

    let miner = InductiveMiner::new();
    let tree = miner.discover_tree(&log);

    assert_eq!(
        tree.leaf_count(),
        1,
        "single-activity log must produce a tree with exactly 1 leaf"
    );
}

// ---------------------------------------------------------------------------
// PM4-M5: check_soundness delegates to PetriNetAnalyzer (pure Rust, no bridge)
// ---------------------------------------------------------------------------

/// A net with a dead transition (input arc from a place that never receives
/// tokens) must be detected as unsound.
#[test]
fn test_pm4m5_check_soundness_detects_dead_transition() {
    use pm4py::models::petri_net_analysis::PetriNetAnalyzer;
    use pm4py::statistics::check_soundness;

    let mut net = make_linear_net_abc();
    // Add p_x (no initial tokens) and t_dead with input arc from p_x
    let px = Place::new("p_x");
    let px_id = px.id.clone();
    net.add_place(px);
    let t_dead = Transition::new("t_dead").with_label("dead");
    let td_id = t_dead.id.clone();
    net.add_transition(t_dead);
    net.add_arc(Arc::new(&px_id, &td_id));

    let analysis = PetriNetAnalyzer::check_soundness(&net);
    assert!(
        !analysis.no_dead_transitions,
        "dead transition must be detected"
    );
    assert!(
        !analysis.is_sound,
        "net with dead transition must not be sound"
    );

    assert!(
        !check_soundness(&net),
        "check_soundness must return false for unsound net"
    );
}

/// A linear A→B→C workflow net must be sound.
#[test]
fn test_pm4m5_check_soundness_sound_net_returns_true() {
    use pm4py::statistics::check_soundness;

    let net = make_linear_net_abc();
    assert!(check_soundness(&net), "linear A→B→C net must be sound");
}

// ---------------------------------------------------------------------------
// PM4-H2: AlignmentChecker delegates to pm4py bridge
// ---------------------------------------------------------------------------

/// AlignmentChecker::check on a log containing an activity (d) not in the
/// A→B→C net must return fitness < 1.0.
#[test]
fn test_pm4h2_alignment_checker_deviant_log_has_fitness_below_one() {
    use pm4py::conformance::AlignmentChecker;

    let net = make_linear_net_abc();

    let mut log = EventLog::new();
    let now = Utc::now();
    let mut trace = Trace::new("deviant_case");
    trace.add_event(Event::new("a", now));
    trace.add_event(Event::new("b", now + ChronoDuration::seconds(5)));
    trace.add_event(Event::new("c", now + ChronoDuration::seconds(10)));
    trace.add_event(Event::new("d", now + ChronoDuration::seconds(15))); // not in net
    log.add_trace(trace);

    let checker = AlignmentChecker::new();
    let result = checker.check(&log, &net);

    assert!(
        result.fitness < 1.0,
        "log with deviant activity must have fitness < 1.0, got {}",
        result.fitness
    );
    assert!(
        result.fitness >= 0.0,
        "fitness must be non-negative, got {}",
        result.fitness
    );
}

// ---------------------------------------------------------------------------
// PM4-H3: diagnostics_token_based_replay populates precision from pm4py
// ---------------------------------------------------------------------------

/// For a perfectly conformant log, precision must be > 0.0 (not the previous
/// hardcoded 0.0 stub value).
#[test]
fn test_pm4h3_diagnostics_token_replay_precision_nonzero_for_conformant_log() {
    use pm4py::conformance::diagnostics_token_based_replay;

    let log = make_linear_log_abc();
    let net = make_linear_net_abc();

    let result = diagnostics_token_based_replay(&log, &net);

    assert!(
        result.precision > 0.0,
        "precision must be > 0.0 for conformant log (was hardcoded 0.0 in stub), got {}",
        result.precision
    );
    assert!(
        result.precision <= 1.0,
        "precision must be ≤ 1.0, got {}",
        result.precision
    );
}

// ---------------------------------------------------------------------------
// PM4-M1: CostBasedAligner delegates to pm4py bridge
// ---------------------------------------------------------------------------

/// A trace A→C→B on a strictly ordered A→B→C net must have aggregate
/// fitness < 1.0 because the order does not conform.
#[test]
fn test_pm4m1_cost_aligner_out_of_order_trace_not_full_sync() {
    use pm4py::conformance::{AlignmentCostModel, CostBasedAligner};

    let net = make_linear_net_abc();

    let mut log = EventLog::new();
    let now = Utc::now();
    let mut trace = Trace::new("out_of_order");
    trace.add_event(Event::new("a", now));
    trace.add_event(Event::new("c", now + ChronoDuration::seconds(5)));
    trace.add_event(Event::new("b", now + ChronoDuration::seconds(10)));
    log.add_trace(trace);

    let aligner = CostBasedAligner::new(AlignmentCostModel::default());
    let alignments = aligner.compute_alignments(&log, &net);

    assert!(!alignments.is_empty(), "must return at least one alignment");
    let fitness = alignments[0].fitness;
    assert!(
        fitness < 1.0,
        "out-of-order trace must have fitness < 1.0, got {}",
        fitness
    );
    assert!(
        fitness >= 0.0,
        "fitness must be non-negative, got {}",
        fitness
    );
}
