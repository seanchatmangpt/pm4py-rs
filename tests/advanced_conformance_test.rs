use chrono::Utc;
/// Advanced Conformance Tests — Chicago TDD (RED → GREEN → REFACTOR)
///
/// Tests cover:
///   estimate_simplicity: arc-degree formula  1 / (1 + arcs/transitions)
///   estimate_generalization_from_net: transition coverage fraction
///
/// RED phase: these tests fail against the old diversity-based stubs.
/// GREEN phase: they pass after the implementations below are merged.
use pm4py::conformance::ExtendedFitnessCalculator;
use pm4py::log::{Event, EventLog, Trace};
use pm4py::models::petri_net::{Arc, Place, Transition};
use pm4py::models::PetriNet;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a net with exactly `n_transitions` labeled transitions and `n_arcs` arcs.
/// Places are added minimally (n_transitions + 1) to form a chain, but we also
/// add extra arcs to reach the desired arc count.
fn make_net_with_counts(n_transitions: usize, n_arcs: usize) -> PetriNet {
    let mut net = PetriNet::new();

    // Add places
    let mut place_ids = Vec::new();
    for i in 0..=(n_transitions) {
        let p = Place::new(format!("p{i}")).with_initial_marking(if i == 0 { 1 } else { 0 });
        place_ids.push(p.id.clone());
        net.add_place(p);
    }

    // Add transitions
    let mut transition_ids = Vec::new();
    for i in 0..n_transitions {
        let t = Transition::new(format!("t{i}")).with_label(format!("activity_{i}"));
        transition_ids.push(t.id.clone());
        net.add_transition(t);
    }

    // Add exactly n_arcs arcs: first place→transition arcs, then transition→place arcs
    let mut arcs_added = 0;
    let mut arc_index = 0;

    // Sequence of arcs: p0→t0, t0→p1, p1→t1, t1→p2, ...
    let arc_pattern: Vec<(String, String)> = (0..n_transitions)
        .flat_map(|i| {
            vec![
                (place_ids[i].clone(), transition_ids[i].clone()),
                (transition_ids[i].clone(), place_ids[i + 1].clone()),
            ]
        })
        .collect();

    while arcs_added < n_arcs && arc_index < arc_pattern.len() {
        let (from, to) = &arc_pattern[arc_index];
        net.add_arc(Arc::new(from.clone(), to.clone()));
        arcs_added += 1;
        arc_index += 1;
    }

    net
}

/// Build a log that executes exactly the given activities.
fn make_log_with_activities(activities: &[&str]) -> EventLog {
    let mut log = EventLog::new();
    let now = Utc::now();
    let mut trace = Trace::new("case_1");
    for &act in activities {
        trace.add_event(Event::new(act, now));
    }
    log.add_trace(trace);
    log
}

// ---------------------------------------------------------------------------
// estimate_simplicity — arc-degree formula
// ---------------------------------------------------------------------------

/// For a net with 2 transitions and 4 arcs:
///   arc-degree simplicity = 1 / (1 + 4/2) = 1 / 3 ≈ 0.333
///
/// The OLD formula: complexity = transitions + places + arcs (includes places)
/// For 2 transitions, 3 places, 4 arcs: complexity = 9 → old simplicity = 1/(1+0.9) ≈ 0.526
/// So this assertion FAILS under the old formula (0.526 > 0.4) and
/// PASSES under the new arc-degree formula (0.333 < 0.4).
#[test]
fn test_simplicity_arc_degree_2_transitions_4_arcs() {
    let net = make_net_with_counts(2, 4);

    // Sanity-check the net we built
    assert_eq!(net.transitions.len(), 2, "expected 2 transitions");
    assert_eq!(net.arcs.len(), 4, "expected 4 arcs");

    let simplicity = ExtendedFitnessCalculator::estimate_simplicity(&net);

    // arc-degree formula: 1/(1+4/2) = 1/3 ≈ 0.333
    let expected = 1.0_f64 / 3.0;

    // Assert close to arc-degree result (within 1e-9)
    assert!(
        (simplicity - expected).abs() < 1e-9,
        "estimate_simplicity({}, {}) = {:.6}, expected arc-degree {:.6}",
        net.transitions.len(),
        net.arcs.len(),
        simplicity,
        expected
    );
}

/// Edge case: zero arcs → simplicity should be 1.0 (no complexity)
#[test]
fn test_simplicity_zero_arcs_returns_one() {
    let net = make_net_with_counts(3, 0);

    let simplicity = ExtendedFitnessCalculator::estimate_simplicity(&net);

    // 1/(1+0/3) = 1/1 = 1.0
    assert!(
        (simplicity - 1.0).abs() < 1e-9,
        "simplicity with 0 arcs should be 1.0, got {simplicity:.6}"
    );
}

/// Edge case: empty net (0 transitions, 0 arcs) → simplicity should be 1.0
#[test]
fn test_simplicity_empty_net_returns_one() {
    let net = PetriNet::new();

    let simplicity = ExtendedFitnessCalculator::estimate_simplicity(&net);

    assert!(
        (simplicity - 1.0).abs() < 1e-9,
        "simplicity of empty net should be 1.0, got {simplicity:.6}"
    );
}

/// For a net with 1 transition and 2 arcs:
///   arc-degree simplicity = 1/(1+2/1) = 1/3 ≈ 0.333
#[test]
fn test_simplicity_one_transition_two_arcs() {
    let net = make_net_with_counts(1, 2);

    assert_eq!(net.transitions.len(), 1);
    assert_eq!(net.arcs.len(), 2);

    let simplicity = ExtendedFitnessCalculator::estimate_simplicity(&net);
    let expected = 1.0 / 3.0;

    assert!(
        (simplicity - expected).abs() < 1e-9,
        "expected {expected:.6}, got {simplicity:.6}"
    );
}

// ---------------------------------------------------------------------------
// estimate_generalization_from_net — transition coverage
// ---------------------------------------------------------------------------

/// Net has labeled transitions A, B, C.
/// Log executes only A and B.
/// Expected generalization = 2/3 ≈ 0.667
///
/// The OLD estimate_generalization(log) ignores the net entirely and uses
/// trace diversity, giving ≥ 0.5 (for a 1-trace log: diversity=1.0 → 1.0*0.5+0.5=1.0).
/// The NEW estimate_generalization_from_net gives 2/3 ≈ 0.667.
#[test]
fn test_generalization_from_net_partial_coverage() {
    let mut net = PetriNet::new();
    let p0 = Place::new("p0").with_initial_marking(1);
    let ta = Transition::new("tA").with_label("A");
    let tb = Transition::new("tB").with_label("B");
    let tc = Transition::new("tC").with_label("C");
    let p1 = Place::new("p1");

    let p0_id = p0.id.clone();
    let ta_id = ta.id.clone();
    let p1_id = p1.id.clone();
    let tb_id = tb.id.clone();
    let tc_id = tc.id.clone();

    net.add_place(p0);
    net.add_transition(ta);
    net.add_transition(tb);
    net.add_transition(tc);
    net.add_place(p1);

    net.add_arc(Arc::new(p0_id, ta_id));
    net.add_arc(Arc::new(tb_id.clone(), p1_id.clone()));
    net.add_arc(Arc::new(tc_id.clone(), p1_id));

    let log = make_log_with_activities(&["A", "B"]);

    let gen = ExtendedFitnessCalculator::estimate_generalization_from_net(&log, &net);
    let expected = 2.0 / 3.0;

    assert!(
        (gen - expected).abs() < 1e-9,
        "expected {expected:.6}, got {gen:.6}"
    );
}

/// Net with all transitions covered by log → generalization = 1.0
#[test]
fn test_generalization_from_net_full_coverage() {
    let mut net = PetriNet::new();
    for label in ["A", "B", "C"] {
        net.add_transition(Transition::new(label).with_label(label));
    }
    net.add_place(Place::new("p"));

    let log = make_log_with_activities(&["A", "B", "C"]);

    let gen = ExtendedFitnessCalculator::estimate_generalization_from_net(&log, &net);

    assert!(
        (gen - 1.0).abs() < 1e-9,
        "full coverage should give 1.0, got {gen:.6}"
    );
}

/// Net with no labeled transitions (only invisible/tau) → generalization = 1.0
#[test]
fn test_generalization_from_net_no_labeled_transitions() {
    let mut net = PetriNet::new();
    // Invisible (tau) transitions have no label
    net.add_transition(Transition::new("tau1")); // label is None
    net.add_transition(Transition::new("tau2"));

    let log = make_log_with_activities(&["A"]);

    let gen = ExtendedFitnessCalculator::estimate_generalization_from_net(&log, &net);

    assert!(
        (gen - 1.0).abs() < 1e-9,
        "no labeled transitions should give 1.0, got {gen:.6}"
    );
}

/// Net with 4 labeled transitions, log covers 1 → generalization = 0.25
#[test]
fn test_generalization_from_net_one_of_four_covered() {
    let mut net = PetriNet::new();
    for label in ["A", "B", "C", "D"] {
        net.add_transition(Transition::new(label).with_label(label));
    }
    net.add_place(Place::new("p"));

    let log = make_log_with_activities(&["A"]);

    let gen = ExtendedFitnessCalculator::estimate_generalization_from_net(&log, &net);
    let expected = 0.25;

    assert!(
        (gen - expected).abs() < 1e-9,
        "expected {expected:.6}, got {gen:.6}"
    );
}
