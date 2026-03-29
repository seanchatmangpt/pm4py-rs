//! BFS Reachability Tests for `can_follow_in_petri`
//!
//! Chicago TDD: RED → GREEN → REFACTOR
//!
//! Tests `FootprintsConformanceChecker::footprints_from_petri_net` which calls
//! `can_follow_in_petri` internally. We test the observable outcome (footprint
//! relationships) so there is no need to expose the private function.
//!
//! # Test nets
//!
//! ## Net 1 — Disconnected branches (core RED test)
//!
//! ```
//! source_A → [A] → sink_A       (branch 1)
//! source_B → [B] → sink_B       (branch 2)
//! ```
//!
//! A and B share no places.  After A fires its output is `sink_A`.
//! BFS from `{sink_A:1}` will never enable B (which needs `source_B`).
//! Expected: can_follow(A,B) == false  →  relationship for (A,B) is Choice.
//!
//! With the old stub (`return true`): both A→B and B→A are true → Parallel.
//! With BFS: both are false → Choice.  RED assertion: `== Choice`.
//!
//! ## Net 2 — Linear sequence A → B
//!
//! ```
//! src → [A] → mid → [B] → end
//! ```
//!
//! A fires → token on `mid` → enables B.  B fires → token on `end` → A not enabled.
//! Expected: (A,B) = Causal, (B,A) = Choice.
//!
//! ## Net 3 — True parallel split (AND gateway)
//!
//! ```
//! src → [split] → p1 AND p2
//!                  p1 → [A] → q1
//!                  p2 → [B] → q2
//!                  q1 AND q2 → [join] → end
//! ```
//!
//! After split fires, p1 and p2 each get a token.
//! A is enabled (p1 has token). After A fires, p1 gone, p2 still present → B enabled.
//! After B fires, p2 gone, A not enabled.
//! BFS from {p1:1}: A enabled immediately → fires → q1:1, p2 not in start state → B not enabled.
//! So can_follow(split,A) and can_follow(split,B) depend on split being a tau.
//!
//! For the visible transitions A and B:
//! Start state after A fires (from output of A = q1): only q1 has token.
//! B needs p2, which is absent → can_follow(A,B) = false.
//! Start state after B fires (from output of B = q2): only q2 has token.
//! A needs p1, absent → can_follow(B,A) = false.
//! Expected: (A,B) and (B,A) both Choice.
//!
//! NOTE: The AND-parallel result may be surprising but is correct for this BFS approach
//! which starts from *only* the output places of the firing transition. Real parallel
//! detection requires a richer initial marking. We test only what BFS guarantees.

use pm4py::conformance::footprints::FootprintsConformanceChecker;
use pm4py::models::footprints::ActivityRelationship;
use pm4py::models::petri_net::{Arc, PetriNet, Place, Transition};

// ---------------------------------------------------------------------------
// Helper: build the disconnected two-branch net
// ---------------------------------------------------------------------------

/// Returns (net, a_id, b_id)
fn build_disconnected_net() -> (PetriNet, String, String) {
    let mut net = PetriNet::new();

    // Branch A: source_A → [A] → sink_A
    let source_a = Place::new("source_A");
    let sink_a = Place::new("sink_A");
    let t_a = Transition::new("A").with_label("A");

    let source_a_id = source_a.id.clone();
    let sink_a_id = sink_a.id.clone();
    let a_id = t_a.id.clone();

    net.add_place(source_a);
    net.add_place(sink_a);
    net.add_transition(t_a);
    net.add_arc(Arc::new(&source_a_id, &a_id));
    net.add_arc(Arc::new(&a_id, &sink_a_id));

    // Branch B: source_B → [B] → sink_B
    let source_b = Place::new("source_B");
    let sink_b = Place::new("sink_B");
    let t_b = Transition::new("B").with_label("B");

    let source_b_id = source_b.id.clone();
    let sink_b_id = sink_b.id.clone();
    let b_id = t_b.id.clone();

    net.add_place(source_b);
    net.add_place(sink_b);
    net.add_transition(t_b);
    net.add_arc(Arc::new(&source_b_id, &b_id));
    net.add_arc(Arc::new(&b_id, &sink_b_id));

    (net, a_id, b_id)
}

// ---------------------------------------------------------------------------
// Helper: build a linear sequence net A → B → C
// ---------------------------------------------------------------------------

/// Returns (net, a_id, b_id, c_id)
fn build_linear_net() -> (PetriNet, String, String, String) {
    let mut net = PetriNet::new();

    let src = Place::new("src");
    let mid1 = Place::new("mid1");
    let mid2 = Place::new("mid2");
    let end = Place::new("end");

    let t_a = Transition::new("A").with_label("A");
    let t_b = Transition::new("B").with_label("B");
    let t_c = Transition::new("C").with_label("C");

    let src_id = src.id.clone();
    let mid1_id = mid1.id.clone();
    let mid2_id = mid2.id.clone();
    let end_id = end.id.clone();
    let a_id = t_a.id.clone();
    let b_id = t_b.id.clone();
    let c_id = t_c.id.clone();

    net.add_place(src);
    net.add_place(mid1);
    net.add_place(mid2);
    net.add_place(end);
    net.add_transition(t_a);
    net.add_transition(t_b);
    net.add_transition(t_c);

    // src → A → mid1 → B → mid2 → C → end
    net.add_arc(Arc::new(&src_id, &a_id));
    net.add_arc(Arc::new(&a_id, &mid1_id));
    net.add_arc(Arc::new(&mid1_id, &b_id));
    net.add_arc(Arc::new(&b_id, &mid2_id));
    net.add_arc(Arc::new(&mid2_id, &c_id));
    net.add_arc(Arc::new(&c_id, &end_id));

    (net, a_id, b_id, c_id)
}

// ---------------------------------------------------------------------------
// CORE RED TEST (Step 1 of Chicago TDD)
// ---------------------------------------------------------------------------

/// RED: disconnected branches — A cannot follow B and B cannot follow A.
///
/// With the stub (`return true`), both directions are `true` → Parallel.
/// With BFS, both directions are `false` → Choice.
/// This test will be RED until BFS is implemented.
#[test]
fn bfs_disconnected_branches_are_choice_not_parallel() {
    let (net, _a_id, _b_id) = build_disconnected_net();

    let footprints = FootprintsConformanceChecker::footprints_from_petri_net(&net);

    // A cannot reach B's enabling place — expect Choice in both directions
    let a_to_b = footprints.get_relationship("A", "B");
    let b_to_a = footprints.get_relationship("B", "A");

    assert_eq!(
        a_to_b,
        Some(ActivityRelationship::Choice),
        "Expected A→B = Choice (no path from A's output to B's input), got {:?}",
        a_to_b
    );
    assert_eq!(
        b_to_a,
        Some(ActivityRelationship::Choice),
        "Expected B→A = Choice (no path from B's output to A's input), got {:?}",
        b_to_a
    );
}

// ---------------------------------------------------------------------------
// Additional tests (will also be RED with stub, GREEN after BFS)
// ---------------------------------------------------------------------------

/// In a linear net A → B → C:
/// - A can reach B (direct successor) → Causal
/// - B cannot reach A (A needs src, which B doesn't produce) → Choice
#[test]
fn bfs_linear_sequence_ab_is_causal() {
    let (net, _a_id, _b_id, _c_id) = build_linear_net();

    let footprints = FootprintsConformanceChecker::footprints_from_petri_net(&net);

    let a_to_b = footprints.get_relationship("A", "B");
    let b_to_a = footprints.get_relationship("B", "A");

    assert_eq!(
        a_to_b,
        Some(ActivityRelationship::Causal),
        "Expected A→B = Causal in linear net, got {:?}",
        a_to_b
    );
    assert_eq!(
        b_to_a,
        Some(ActivityRelationship::Choice),
        "Expected B→A = Choice in linear net (B output does not lead back to A), got {:?}",
        b_to_a
    );
}

/// In a linear net A → B → C:
/// - B can reach C (direct successor) → Causal
/// - C cannot reach B → Choice
#[test]
fn bfs_linear_sequence_bc_is_causal() {
    let (net, _a_id, _b_id, _c_id) = build_linear_net();

    let footprints = FootprintsConformanceChecker::footprints_from_petri_net(&net);

    let b_to_c = footprints.get_relationship("B", "C");
    let c_to_b = footprints.get_relationship("C", "B");

    assert_eq!(
        b_to_c,
        Some(ActivityRelationship::Causal),
        "Expected B→C = Causal in linear net, got {:?}",
        b_to_c
    );
    assert_eq!(
        c_to_b,
        Some(ActivityRelationship::Choice),
        "Expected C→B = Choice in linear net, got {:?}",
        c_to_b
    );
}

/// BFS state cap: net with a long chain should not hang.
/// Verify the function terminates for a 10-transition chain.
#[test]
fn bfs_terminates_on_long_chain() {
    let mut net = PetriNet::new();
    let n = 10usize;

    let mut place_ids = Vec::with_capacity(n + 1);
    let mut trans_ids = Vec::with_capacity(n);

    for i in 0..=n {
        let p = Place::new(format!("p{}", i));
        place_ids.push(p.id.clone());
        net.add_place(p);
    }
    for i in 0..n {
        let label = format!("T{}", i);
        let t = Transition::new(&label).with_label(&label);
        trans_ids.push(t.id.clone());
        net.add_transition(t);
        net.add_arc(Arc::new(&place_ids[i], &trans_ids[i]));
        net.add_arc(Arc::new(&trans_ids[i], &place_ids[i + 1]));
    }

    // Must finish in <5 seconds (no panic, no hang)
    let footprints = FootprintsConformanceChecker::footprints_from_petri_net(&net);

    // T0 → T1 must be Causal
    let t0_t1 = footprints.get_relationship("T0", "T1");
    assert_eq!(
        t0_t1,
        Some(ActivityRelationship::Causal),
        "Expected T0→T1 = Causal in long chain, got {:?}",
        t0_t1
    );
}
