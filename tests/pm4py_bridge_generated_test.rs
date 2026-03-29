//! Chicago TDD — generated pm4py bridge tests
//!
//! RED: compile without Python, functions are callable
//! GREEN: when pm4py is available at runtime, bridge returns correct values
//!
//! Skip convention: tests that call Python guard with `pm4py_available()`.
//! Tests PASS in CI even without Python — they skip gracefully.

#![cfg(test)]
#![allow(deprecated)]

use chrono::Utc;
use pm4py::log::{Event, EventLog, Trace};
use pm4py::models::{Arc, PetriNet, Place, Transition};

// ── Runtime guard ────────────────────────────────────────────────────────────

#[allow(deprecated)]
fn pm4py_available() -> bool {
    pyo3::prepare_freethreaded_python();
    pyo3::Python::with_gil(|py| py.import("pm4py").is_ok() && py.import("pandas").is_ok())
}

// ── Fixtures ─────────────────────────────────────────────────────────────────

/// Minimal 2-activity event log A → B.
fn two_activity_log() -> EventLog {
    let mut log = EventLog::new();
    let mut trace = Trace::new("case1");
    trace.events.push(Event::new("A", Utc::now()));
    trace.events.push(Event::new("B", Utc::now()));
    log.traces.push(trace);
    log
}

/// Simple A→B Petri net with one token in start place.
fn ab_petri_net() -> PetriNet {
    let mut net = PetriNet::new();

    let mut p_start = Place::new("start");
    p_start.id = "start".to_string();
    p_start.initial_marking = 1;
    let mut p_mid = Place::new("mid");
    p_mid.id = "mid".to_string();
    let mut p_end = Place::new("end");
    p_end.id = "end".to_string();

    net.add_place(p_start);
    net.add_place(p_mid);
    net.add_place(p_end);
    net.set_initial_place("start".to_string());
    net.set_final_place("end".to_string());

    let mut t_a = Transition::new("t_A");
    t_a.id = "t_A".to_string();
    t_a = t_a.with_label("A".to_string());
    let mut t_b = Transition::new("t_B");
    t_b.id = "t_B".to_string();
    t_b = t_b.with_label("B".to_string());
    net.add_transition(t_a);
    net.add_transition(t_b);

    net.add_arc(Arc::new("start", "t_A").with_weight(1));
    net.add_arc(Arc::new("t_A", "mid").with_weight(1));
    net.add_arc(Arc::new("mid", "t_B").with_weight(1));
    net.add_arc(Arc::new("t_B", "end").with_weight(1));
    net
}

// ── T-GEN-01: discover_petri_net_inductive ───────────────────────────────────

#[test]
fn t_gen_01_discover_inductive_returns_nonempty_petri_net() {
    if !pm4py_available() {
        eprintln!("SKIP t_gen_01: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let net = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::discovery::call_discover_petri_net_inductive(py, &log)
    })
    .expect("T-GEN-01: discover_petri_net_inductive failed");

    assert!(
        !net.places.is_empty(),
        "T-GEN-01: PetriNet must have places"
    );
    assert!(
        !net.transitions.is_empty(),
        "T-GEN-01: PetriNet must have transitions"
    );
}

// ── T-GEN-02: discover_petri_net_alpha ───────────────────────────────────────

#[test]
fn t_gen_02_discover_alpha_returns_nonempty_petri_net() {
    if !pm4py_available() {
        eprintln!("SKIP t_gen_02: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let net = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::discovery::call_discover_petri_net_alpha(py, &log)
    })
    .expect("T-GEN-02: discover_petri_net_alpha failed");

    assert!(
        !net.places.is_empty(),
        "T-GEN-02: PetriNet must have places"
    );
}

// ── T-GEN-03: discover_petri_net_heuristics ──────────────────────────────────

#[test]
fn t_gen_03_discover_heuristics_returns_nonempty_petri_net() {
    if !pm4py_available() {
        eprintln!("SKIP t_gen_03: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let net = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::discovery::call_discover_petri_net_heuristics(py, &log)
    })
    .expect("T-GEN-03: discover_petri_net_heuristics failed");

    assert!(
        !net.places.is_empty(),
        "T-GEN-03: PetriNet must have places"
    );
}

// ── T-GEN-04: discover_petri_net_ilp ─────────────────────────────────────────

#[test]
fn t_gen_04_discover_ilp_returns_nonempty_petri_net() {
    if !pm4py_available() {
        eprintln!("SKIP t_gen_04: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let net = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::discovery::call_discover_petri_net_ilp(py, &log)
    })
    .expect("T-GEN-04: discover_petri_net_ilp failed");

    assert!(
        !net.places.is_empty(),
        "T-GEN-04: ILP PetriNet must have places"
    );
}

// ── T-GEN-05: fitness_token_based_replay ─────────────────────────────────────

#[test]
fn t_gen_05_fitness_token_replay_score_in_unit_interval() {
    if !pm4py_available() {
        eprintln!("SKIP t_gen_05: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let net = ab_petri_net();
    let score = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::conformance::call_fitness_token_based_replay(py, &log, &net)
    })
    .expect("T-GEN-05: fitness_token_based_replay failed");

    assert!(
        (0.0..=1.0).contains(&score),
        "T-GEN-05: fitness score must be in [0,1], got {score}"
    );
}

// ── T-GEN-06: conformance_diagnostics_token_based_replay ─────────────────────

#[test]
fn t_gen_06_token_replay_diagnostics_returns_average_in_unit_interval() {
    if !pm4py_available() {
        eprintln!("SKIP t_gen_06: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let net = ab_petri_net();
    let avg = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::conformance::call_conformance_diagnostics_token_based_replay(
            py, &log, &net,
        )
    })
    .expect("T-GEN-06: conformance_diagnostics_token_based_replay failed");

    assert!(
        (0.0..=1.0).contains(&avg),
        "T-GEN-06: avg fitness must be in [0,1], got {avg}"
    );
}

// ── T-GEN-07: get_variants compiles and runs ─────────────────────────────────

#[test]
fn t_gen_07_get_variants_runs_without_error() {
    if !pm4py_available() {
        eprintln!("SKIP t_gen_07: pm4py not available");
        return;
    }
    let log = two_activity_log();
    pyo3::Python::with_gil(|py| pm4py::python::generated::statistics::call_get_variants(py, &log))
        .expect("T-GEN-07: get_variants failed");
}

// ── T-GEN-08: eventlog_to_pm4py round-trip ───────────────────────────────────

#[test]
fn t_gen_08_eventlog_to_pm4py_converts_without_exception() {
    if !pm4py_available() {
        eprintln!("SKIP t_gen_08: pm4py not available");
        return;
    }
    let log = two_activity_log();
    pyo3::Python::with_gil(|py| pm4py::python::generated::helpers::eventlog_to_pm4py(py, &log))
        .expect("T-GEN-08: eventlog_to_pm4py raised Python exception");
}

// ── T-GEN-09: pm4py_petri_to_rust preserves structure ────────────────────────

#[test]
fn t_gen_09_discovery_round_trip_preserves_initial_and_final_places() {
    if !pm4py_available() {
        eprintln!("SKIP t_gen_09: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let net = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::discovery::call_discover_petri_net_inductive(py, &log)
    })
    .expect("T-GEN-09: discovery failed");

    assert!(
        !net.places.is_empty(),
        "T-GEN-09: round-tripped net must have places"
    );
    assert!(
        net.initial_place.is_some(),
        "T-GEN-09: initial_place must be set"
    );
    assert!(
        net.final_place.is_some(),
        "T-GEN-09: final_place must be set"
    );
}

// ── T-GEN-10: regression — alpha miner callable via generated module ──────────

#[test]
fn t_gen_10_regression_alpha_via_generated_module() {
    if !pm4py_available() {
        eprintln!("SKIP t_gen_10: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::discovery::call_discover_petri_net_alpha(py, &log)
    });
    assert!(
        result.is_ok(),
        "T-GEN-10: generated alpha regression failed: {:?}",
        result.err()
    );
}

// ── T-GEN-11: precision_token_based_replay ───────────────────────────────────

#[test]
fn t_gen_11_precision_token_replay_score_in_unit_interval() {
    if !pm4py_available() {
        eprintln!("SKIP t_gen_11: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let net = ab_petri_net();
    let score = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::conformance::call_precision_token_based_replay(py, &log, &net)
    })
    .expect("T-GEN-11: precision_token_based_replay failed");

    assert!(
        (0.0..=1.0).contains(&score),
        "T-GEN-11: precision must be in [0,1], got {score}"
    );
}

// ── T-GEN-12: fitness_alignments ─────────────────────────────────────────────

#[test]
fn t_gen_12_fitness_alignments_score_in_unit_interval() {
    if !pm4py_available() {
        eprintln!("SKIP t_gen_12: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let net = ab_petri_net();
    let score = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::conformance::call_fitness_alignments(py, &log, &net)
    })
    .expect("T-GEN-12: fitness_alignments failed");

    assert!(
        (0.0..=1.0).contains(&score),
        "T-GEN-12: alignments fitness must be in [0,1], got {score}"
    );
}

// ── T-GEN-13: get_start_activities ───────────────────────────────────────────

#[test]
fn t_gen_13_get_start_activities_runs_without_error() {
    if !pm4py_available() {
        eprintln!("SKIP t_gen_13: pm4py not available");
        return;
    }
    let log = two_activity_log();
    pyo3::Python::with_gil(|py| {
        pm4py::python::generated::statistics::call_get_start_activities(py, &log)
    })
    .expect("T-GEN-13: get_start_activities failed");
}
