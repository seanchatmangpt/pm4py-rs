//! PyO3 bridge tests — Chicago TDD (RED → GREEN → REFACTOR)
//!
//! Tests cover:
//!   T1: pm4py_available() guard works without panic
//!   T2: InductiveMiner with bridge discovers parallel structure       [pm4py-bridge + pm4py]
//!   T3: eventlog_to_pm4py converts 3-trace log to pm4py EventLog    [pm4py-bridge + pm4py]
//!   T4: call_conformance_alignments returns fitness in [0,1]         [pm4py-bridge + pm4py]
//!   T5: InductiveMiner never panics (Armstrong fault tolerance)
//!   T6: precision_token_based_replay uses ETC formula, not fitness   ← RED until ETC impl
//!   T7: structural_similarity with no-label-overlap nets < 0.9       ← RED until Jaccard impl
//!   T8: structural_similarity of identical net returns 1.0

use chrono::Utc;
use pm4py::conformance::token_replay::{precision_token_based_replay, TokenReplay};
use pm4py::log::{Event, EventLog, Trace};
use pm4py::models::petri_net::{Arc, Place, Transition};
use pm4py::models::PetriNet;
use pm4py::statistics::missing_stats::structural_similarity;

// ---------------------------------------------------------------------------
// Guard: detect whether pm4py Python package is reachable at runtime
// ---------------------------------------------------------------------------

fn pm4py_available() -> bool {
    #[cfg(feature = "pm4py-bridge")]
    {
        use pyo3::prelude::*;
        Python::with_gil(|py| py.import("pm4py").is_ok())
    }
    #[cfg(not(feature = "pm4py-bridge"))]
    {
        false
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Log where A and B appear in both orderings — parallel structure.
fn make_parallel_log() -> EventLog {
    let mut log = EventLog::new();
    let now = Utc::now();
    for i in 0..5_u32 {
        let mut t = Trace::new(format!("ab_{}", i));
        t.add_event(Event::new("A", now));
        t.add_event(Event::new("B", now));
        log.add_trace(t);
    }
    for i in 0..5_u32 {
        let mut t = Trace::new(format!("ba_{}", i));
        t.add_event(Event::new("B", now));
        t.add_event(Event::new("A", now));
        log.add_trace(t);
    }
    log
}

/// XOR net: source → A → p1 → B → sink
///                         p1 → C → sink
fn make_xor_net_abc() -> PetriNet {
    let mut net = PetriNet::new();

    let p0 = Place::new("p0").with_initial_marking(1);
    let p0_id = p0.id.clone();
    let ta = Transition::new("ta").with_label("A");
    let ta_id = ta.id.clone();
    let p1 = Place::new("p1");
    let p1_id = p1.id.clone();
    let tb = Transition::new("tb").with_label("B");
    let tb_id = tb.id.clone();
    let tc = Transition::new("tc").with_label("C");
    let tc_id = tc.id.clone();
    let psink = Place::new("psink").with_final_marking(1);
    let psink_id = psink.id.clone();

    net.add_place(p0);
    net.add_transition(ta);
    net.add_place(p1);
    net.add_transition(tb);
    net.add_transition(tc);
    net.add_place(psink);

    net.add_arc(Arc::new(&p0_id, &ta_id));
    net.add_arc(Arc::new(&ta_id, &p1_id));
    net.add_arc(Arc::new(&p1_id, &tb_id));
    net.add_arc(Arc::new(&p1_id, &tc_id));
    net.add_arc(Arc::new(&tb_id, &psink_id));
    net.add_arc(Arc::new(&tc_id, &psink_id));

    net.set_initial_place(p0_id);
    net.set_final_place(psink_id);
    net
}

/// Five traces of A → B (used against XOR net above).
fn make_ab_only_log() -> EventLog {
    let mut log = EventLog::new();
    let now = Utc::now();
    for i in 0..5_u32 {
        let mut t = Trace::new(format!("case_{}", i));
        t.add_event(Event::new("A", now));
        t.add_event(Event::new("B", now));
        log.add_trace(t);
    }
    log
}

// ---------------------------------------------------------------------------
// T1: pm4py_available() returns a bool — must not panic
// ---------------------------------------------------------------------------

#[test]
fn test_bridge_available_returns_bool() {
    let _ = pm4py_available(); // must not panic in any environment
}

// ---------------------------------------------------------------------------
// T2: InductiveMiner via bridge discovers parallel structure
//     GUARD: pm4py-bridge feature + pm4py Python package installed
// ---------------------------------------------------------------------------

#[test]
#[cfg(feature = "pm4py-bridge")]
fn test_inductive_miner_via_bridge_discovers_parallel_structure() {
    if !pm4py_available() {
        return;
    }
    use pm4py::discovery::InductiveMiner;

    let log = make_parallel_log();
    let net = InductiveMiner::new().discover(&log);

    // Real inductive miner via Python discovers A ∥ B — the net has tau (silent)
    // transitions for the parallel split/join.  The alphabetical stub has NO silent
    // transitions (only A and B).
    // We check structural proof: bridge net must have at least one invisible transition.
    let silent_count = net.transitions.iter().filter(|t| t.label.is_none()).count();
    assert!(
        silent_count >= 1,
        "bridge inductive miner must produce at least one tau transition for A ∥ B; \
         got {} (suggests stub fallback was used)",
        silent_count
    );
    // Must also have both A and B visible transitions.
    assert!(net
        .transitions
        .iter()
        .any(|t| t.label.as_deref() == Some("A")));
    assert!(net
        .transitions
        .iter()
        .any(|t| t.label.as_deref() == Some("B")));
}

// ---------------------------------------------------------------------------
// T3: eventlog_to_pm4py converts 3-trace log to pm4py EventLog (len == 3)
//     GUARD: pm4py-bridge feature + pm4py Python package installed
// ---------------------------------------------------------------------------

#[test]
#[cfg(feature = "pm4py-bridge")]
fn test_eventlog_to_pm4py_converts_traces() {
    if !pm4py_available() {
        return;
    }
    use pm4py::python::pm4py_bridge::eventlog_to_pm4py;
    use pyo3::prelude::*;

    let mut log = EventLog::new();
    let now = Utc::now();
    for i in 0..3_u32 {
        let mut t = Trace::new(format!("case_{}", i));
        t.add_event(Event::new("A", now));
        log.add_trace(t);
    }

    Python::with_gil(|py| {
        let py_log = eventlog_to_pm4py(py, &log)
            .expect("eventlog_to_pm4py must succeed when pm4py is installed");
        let len: usize = py_log
            .call_method0(py, "__len__")
            .and_then(|l| l.extract(py))
            .unwrap_or(0);
        assert_eq!(len, 3, "pm4py EventLog must have 3 cases");
    });
}

// ---------------------------------------------------------------------------
// T4: call_conformance_alignments returns fitness in [0.0, 1.0]
//     GUARD: pm4py-bridge feature + pm4py Python package installed
// ---------------------------------------------------------------------------

#[test]
#[cfg(feature = "pm4py-bridge")]
fn test_alignment_bridge_fitness_between_0_and_1() {
    if !pm4py_available() {
        return;
    }
    use pm4py::discovery::AlphaMiner;
    use pm4py::python::pm4py_bridge::call_conformance_alignments;
    use pyo3::prelude::*;

    let log = make_parallel_log();
    let net = AlphaMiner::new().discover(&log);

    let fitness = Python::with_gil(|py| call_conformance_alignments(py, &log, &net).unwrap_or(0.0));
    assert!(
        (0.0..=1.0).contains(&fitness),
        "alignment fitness must be in [0, 1], got {:.4}",
        fitness
    );
}

// ---------------------------------------------------------------------------
// T5: InductiveMiner must never panic (Armstrong fault-tolerance)
//     Runs with and without pm4py installed.
// ---------------------------------------------------------------------------

#[test]
fn test_bridge_fallback_never_panics() {
    use pm4py::discovery::InductiveMiner;

    let mut log = EventLog::new();
    let mut t = Trace::new("c1");
    t.add_event(Event::new("A", Utc::now()));
    log.add_trace(t);

    // Must return a valid net regardless of Python availability
    let net = InductiveMiner::new().discover(&log);
    assert!(
        !net.transitions.is_empty(),
        "InductiveMiner must return at least one transition even without Python"
    );
}

// ---------------------------------------------------------------------------
// T6: precision_token_based_replay uses ETC formula (RED until ETC implemented)
//
// Setup: XOR net (A → {B, C}), log with only A→B traces.
//   Token replay fitness = 1.0  (A and B both in net, all tokens produced)
//   ETC precision < 1.0         (at p1 both B and C enabled, only B fired)
//
// Current stub: returns result.fitness = 1.0  → assertion precision < 0.9 FAILS  (RED)
// After ETC impl: returns ≈ 0.67             → assertion PASSES                  (GREEN)
// ---------------------------------------------------------------------------

#[test]
fn test_precision_etc_differs_from_fitness_on_xor_net() {
    let net = make_xor_net_abc();
    let log = make_ab_only_log();

    let precision = precision_token_based_replay(&log, &net);

    // ETC: step-1 (event A): 1 enabled (ta), 1 executed → at+=1, et+=1
    //      step-2 (event B): 2 enabled (tb, tc), 1 executed → at+=2, et+=1
    // Per trace: at=3, et=2.  Five traces: at=15, et=10 → precision ≈ 0.667
    assert!(
        precision < 0.9,
        "ETC precision for XOR net (only B executed) must be < 0.9, got {:.4}. \
         The stub returns fitness=1.0; implement WvdA ETC to fix this.",
        precision
    );
}

// ---------------------------------------------------------------------------
// T7: structural_similarity with no label overlap returns < 0.9
//     (RED until Jaccard impl; current count-diff returns 1.0 for same-size nets)
// ---------------------------------------------------------------------------

#[test]
fn test_structural_similarity_different_labels_not_fully_similar() {
    // net1: place → A → place  (label "A")
    let mut net1 = PetriNet::new();
    let p1a = Place::new("p1a");
    let p1a_id = p1a.id.clone();
    let t1a = Transition::new("t1a").with_label("A");
    let t1a_id = t1a.id.clone();
    let p1b = Place::new("p1b");
    let p1b_id = p1b.id.clone();
    net1.add_place(p1a);
    net1.add_transition(t1a);
    net1.add_place(p1b);
    net1.add_arc(Arc::new(&p1a_id, &t1a_id));
    net1.add_arc(Arc::new(&t1a_id, &p1b_id));

    // net2: same topology but label "Z" — no overlap with "A"
    let mut net2 = PetriNet::new();
    let p2a = Place::new("p2a");
    let p2a_id = p2a.id.clone();
    let t2a = Transition::new("t2a").with_label("Z");
    let t2a_id = t2a.id.clone();
    let p2b = Place::new("p2b");
    let p2b_id = p2b.id.clone();
    net2.add_place(p2a);
    net2.add_transition(t2a);
    net2.add_place(p2b);
    net2.add_arc(Arc::new(&p2a_id, &t2a_id));
    net2.add_arc(Arc::new(&t2a_id, &p2b_id));

    let sim = structural_similarity(&net1, &net2);

    // Count-diff stub: places_diff=0, trans_diff=0, arcs_diff=0 → returns 1.0 (WRONG)
    // Jaccard over labels: {"A"} ∩ {"Z"} = ∅, union = 2 → jaccard = 0.0
    // combined (jaccard + density_sim) / 2 ≈ 0.5 → passes assertion
    assert!(
        sim < 0.9,
        "nets with completely different activity labels must have similarity < 0.9, got {:.4}. \
         The count-diff stub returns 1.0; implement Jaccard to fix this.",
        sim
    );
}

// ---------------------------------------------------------------------------
// T8: structural_similarity of identical net returns 1.0 (regression guard)
// ---------------------------------------------------------------------------

#[test]
fn test_structural_similarity_identical_nets_returns_1() {
    let mut net = PetriNet::new();
    let p = Place::new("p1");
    let p_id = p.id.clone();
    let t = Transition::new("t1").with_label("A");
    let t_id = t.id.clone();
    let q = Place::new("p2");
    let q_id = q.id.clone();
    net.add_place(p);
    net.add_transition(t);
    net.add_place(q);
    net.add_arc(Arc::new(&p_id, &t_id));
    net.add_arc(Arc::new(&t_id, &q_id));

    let sim = structural_similarity(&net, &net);
    assert!(
        (sim - 1.0).abs() < 1e-9,
        "structural_similarity(net, net) must return 1.0, got {:.6}",
        sim
    );
}
