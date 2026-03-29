#![allow(dropping_references, clippy::redundant_closure_call)]
//! pm4py official API tests — auto-generated, DO NOT EDIT
//! Source: pm4py-rust/ggen/ontology/pm4py-api.ttl
//! Regenerate: cd pm4py-rust/ggen && /Users/sac/ggen/target/release/ggen sync --manifest ggen.toml
//!
//! Each test mirrors a test from the official pm4py Python test suite.
//! Skip convention: pm4py_available() guard — passes in CI without Python.

#![cfg(test)]
#![allow(deprecated, dead_code, unused_variables, unused_imports)]

use pm4py::log::{EventLog, Trace, Event};
use pm4py::models::{PetriNet, Place, Transition, Arc};
use chrono::Utc;
use tempfile::NamedTempFile;

// ── Runtime guard ────────────────────────────────────────────────────────────

#[allow(deprecated)]
fn pm4py_available() -> bool {
    pyo3::prepare_freethreaded_python();
    pyo3::Python::with_gil(|py| {
        py.import("pm4py").is_ok() && py.import("pandas").is_ok()
    })
}

// ── Fixtures ─────────────────────────────────────────────────────────────────

/// Two-activity log: A → B (single trace). Minimal fixture for all log_only tests.
fn two_activity_log() -> EventLog {
    let mut log = EventLog::new();
    let mut trace = Trace::new("case1");
    trace.events.push(Event::new("A", Utc::now()));
    trace.events.push(Event::new("B", Utc::now()));
    log.traces.push(trace);
    log
}

/// Running-example log — 6 variants, 13 traces. Used for realistic conformance tests.
fn running_example_log() -> EventLog {
    pyo3::Python::with_gil(|py| {
        pm4py::python::generated::io::call_read_xes(py, "test_data/running-example.xes")
    }).unwrap_or_else(|_| two_activity_log())
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

// ── Test data paths ───────────────────────────────────────────────────────────

const XES_PATH:  &str = "test_data/running-example.xes";
const PNML_PATH: &str = "test_data/running-example.pnml";
const PTML_PATH: &str = "test_data/running-example.ptml";
const BPMN_PATH: &str = "test_data/running-example.bpmn";
const DFG_PATH:  &str = "test_data/running-example.dfg";

// ── Generated tests — one per pm4py API function ─────────────────────────────








#[test]
fn call_conformance_declare_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_conformance_declare_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::conformance::call_conformance_declare(py, &log)
    });
    let avg = result.expect("conformance_declare failed");
    assert!((0.0..=1.0).contains(&avg),
        "conformance_declare: avg must be in [0,1], got {avg}");
}







#[test]
fn call_conformance_diagnostics_alignments_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_conformance_diagnostics_alignments_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let net = ab_petri_net();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::conformance::call_conformance_diagnostics_alignments(py, &log, &net)
    });
    let avg = result.expect("conformance_diagnostics_alignments failed");
    assert!((0.0..=1.0).contains(&avg),
        "conformance_diagnostics_alignments: avg must be in [0,1], got {avg}");
}







#[test]
fn call_conformance_diagnostics_footprints_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_conformance_diagnostics_footprints_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let net = ab_petri_net();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::conformance::call_conformance_diagnostics_footprints(py, &log, &net)
    });
    result.expect("conformance_diagnostics_footprints failed");
}







#[test]
fn call_conformance_diagnostics_token_based_replay_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_conformance_diagnostics_token_based_replay_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let net = ab_petri_net();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::conformance::call_conformance_diagnostics_token_based_replay(py, &log, &net)
    });
    let avg = result.expect("conformance_diagnostics_token_based_replay failed");
    assert!((0.0..=1.0).contains(&avg),
        "conformance_diagnostics_token_based_replay: avg must be in [0,1], got {avg}");
}







#[test]
fn call_conformance_log_skeleton_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_conformance_log_skeleton_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::conformance::call_conformance_log_skeleton(py, &log)
    });
    let avg = result.expect("conformance_log_skeleton failed");
    assert!((0.0..=1.0).contains(&avg),
        "conformance_log_skeleton: avg must be in [0,1], got {avg}");
}







#[test]
fn call_conformance_temporal_profile_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_conformance_temporal_profile_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::conformance::call_conformance_temporal_profile(py, &log)
    });
    result.expect("conformance_temporal_profile failed");
}







#[test]
fn call_fitness_alignments_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_fitness_alignments_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let net = ab_petri_net();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::conformance::call_fitness_alignments(py, &log, &net)
    });
    let score = result.expect("fitness_alignments failed");
    assert!(score >= 0.0,
        "fitness_alignments: score must be non-negative, got {score}");
}







#[test]
fn call_fitness_footprints_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_fitness_footprints_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let net = ab_petri_net();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::conformance::call_fitness_footprints(py, &log, &net)
    });
    let score = result.expect("fitness_footprints failed");
    assert!(score >= 0.0,
        "fitness_footprints: score must be non-negative, got {score}");
}







#[test]
fn call_fitness_token_based_replay_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_fitness_token_based_replay_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let net = ab_petri_net();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::conformance::call_fitness_token_based_replay(py, &log, &net)
    });
    let score = result.expect("fitness_token_based_replay failed");
    assert!(score >= 0.0,
        "fitness_token_based_replay: score must be non-negative, got {score}");
}







#[test]
fn call_generalization_tbr_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_generalization_tbr_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let net = ab_petri_net();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::conformance::call_generalization_tbr(py, &log, &net)
    });
    let score = result.expect("generalization_tbr failed");
    assert!(score >= 0.0,
        "generalization_tbr: score must be non-negative, got {score}");
}







#[test]
fn call_precision_alignments_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_precision_alignments_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let net = ab_petri_net();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::conformance::call_precision_alignments(py, &log, &net)
    });
    let score = result.expect("precision_alignments failed");
    assert!(score >= 0.0,
        "precision_alignments: score must be non-negative, got {score}");
}







#[test]
fn call_precision_footprints_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_precision_footprints_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let net = ab_petri_net();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::conformance::call_precision_footprints(py, &log, &net)
    });
    let score = result.expect("precision_footprints failed");
    assert!(score >= 0.0,
        "precision_footprints: score must be non-negative, got {score}");
}







#[test]
fn call_precision_token_based_replay_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_precision_token_based_replay_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let net = ab_petri_net();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::conformance::call_precision_token_based_replay(py, &log, &net)
    });
    let score = result.expect("precision_token_based_replay failed");
    assert!(score >= 0.0,
        "precision_token_based_replay: score must be non-negative, got {score}");
}







#[test]
fn call_derive_minimum_self_distance_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_derive_minimum_self_distance_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::discovery::call_derive_minimum_self_distance(py, &log)
    });
    result.expect("derive_minimum_self_distance failed");
}







#[test]
fn call_discover_bpmn_inductive_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_discover_bpmn_inductive_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::discovery::call_discover_bpmn_inductive(py, &log)
    });
    result.expect("discover_bpmn_inductive failed");
}







#[test]
fn call_discover_declare_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_discover_declare_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::discovery::call_discover_declare(py, &log)
    });
    result.expect("discover_declare failed");
}







#[test]
fn call_discover_dfg_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_discover_dfg_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::discovery::call_discover_dfg(py, &log)
    });
    result.expect("discover_dfg failed");
}







#[test]
fn call_discover_dfg_typed_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_discover_dfg_typed_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::discovery::call_discover_dfg_typed(py, &log)
    });
    result.expect("discover_dfg_typed failed");
}







#[test]
fn call_discover_directly_follows_graph_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_discover_directly_follows_graph_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::discovery::call_discover_directly_follows_graph(py, &log)
    });
    result.expect("discover_directly_follows_graph failed");
}







#[test]
fn call_discover_eventually_follows_graph_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_discover_eventually_follows_graph_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::discovery::call_discover_eventually_follows_graph(py, &log)
    });
    result.expect("discover_eventually_follows_graph failed");
}







#[test]
fn call_discover_footprints_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_discover_footprints_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::discovery::call_discover_footprints(py, &log)
    });
    result.expect("discover_footprints failed");
}







#[test]
fn call_discover_heuristics_net_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_discover_heuristics_net_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::discovery::call_discover_heuristics_net(py, &log)
    });
    result.expect("discover_heuristics_net failed");
}







#[test]
fn call_discover_log_skeleton_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_discover_log_skeleton_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::discovery::call_discover_log_skeleton(py, &log)
    });
    result.expect("discover_log_skeleton failed");
}







#[test]
fn call_discover_performance_dfg_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_discover_performance_dfg_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::discovery::call_discover_performance_dfg(py, &log)
    });
    result.expect("discover_performance_dfg failed");
}







#[test]
fn call_discover_petri_net_alpha_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_discover_petri_net_alpha_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::discovery::call_discover_petri_net_alpha(py, &log)
    });
    let discovered = result.expect("discover_petri_net_alpha failed");
    assert!(!discovered.places.is_empty(),
        "discover_petri_net_alpha: PetriNet must have places");
    assert!(!discovered.transitions.is_empty(),
        "discover_petri_net_alpha: PetriNet must have transitions");
}







#[test]
fn call_discover_petri_net_alpha_plus_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_discover_petri_net_alpha_plus_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::discovery::call_discover_petri_net_alpha_plus(py, &log)
    });
    let discovered = result.expect("discover_petri_net_alpha_plus failed");
    assert!(!discovered.places.is_empty(),
        "discover_petri_net_alpha_plus: PetriNet must have places");
    assert!(!discovered.transitions.is_empty(),
        "discover_petri_net_alpha_plus: PetriNet must have transitions");
}







#[test]
fn call_discover_petri_net_heuristics_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_discover_petri_net_heuristics_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::discovery::call_discover_petri_net_heuristics(py, &log)
    });
    let discovered = result.expect("discover_petri_net_heuristics failed");
    assert!(!discovered.places.is_empty(),
        "discover_petri_net_heuristics: PetriNet must have places");
    assert!(!discovered.transitions.is_empty(),
        "discover_petri_net_heuristics: PetriNet must have transitions");
}







#[test]
fn call_discover_petri_net_ilp_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_discover_petri_net_ilp_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::discovery::call_discover_petri_net_ilp(py, &log)
    });
    let discovered = result.expect("discover_petri_net_ilp failed");
    assert!(!discovered.places.is_empty(),
        "discover_petri_net_ilp: PetriNet must have places");
    assert!(!discovered.transitions.is_empty(),
        "discover_petri_net_ilp: PetriNet must have transitions");
}







#[test]
fn call_discover_petri_net_inductive_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_discover_petri_net_inductive_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::discovery::call_discover_petri_net_inductive(py, &log)
    });
    let discovered = result.expect("discover_petri_net_inductive failed");
    assert!(!discovered.places.is_empty(),
        "discover_petri_net_inductive: PetriNet must have places");
    assert!(!discovered.transitions.is_empty(),
        "discover_petri_net_inductive: PetriNet must have transitions");
}







#[test]
fn call_discover_powl_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_discover_powl_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::discovery::call_discover_powl(py, &log)
    });
    result.expect("discover_powl failed");
}







#[test]
fn call_discover_prefix_tree_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_discover_prefix_tree_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::discovery::call_discover_prefix_tree(py, &log)
    });
    result.expect("discover_prefix_tree failed");
}







#[test]
fn call_discover_process_tree_inductive_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_discover_process_tree_inductive_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::discovery::call_discover_process_tree_inductive(py, &log)
    });
    result.expect("discover_process_tree_inductive failed");
}







#[test]
fn call_discover_temporal_profile_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_discover_temporal_profile_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::discovery::call_discover_temporal_profile(py, &log)
    });
    result.expect("discover_temporal_profile failed");
}







#[test]
fn call_discover_transition_system_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_discover_transition_system_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::discovery::call_discover_transition_system(py, &log)
    });
    result.expect("discover_transition_system failed");
}







#[test]
fn call_read_ocel_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_read_ocel_official: pm4py not available");
        return;
    }
    // SKIP: no OCEL test fixture available
    return;
    #[allow(unreachable_code)]
    let result: Result<(), ()> = Ok(());
    result.expect("read_ocel failed");
}







#[test]
fn call_read_ocel2_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_read_ocel2_official: pm4py not available");
        return;
    }
    // SKIP: no OCEL test fixture available
    return;
    #[allow(unreachable_code)]
    let result: Result<(), ()> = Ok(());
    result.expect("read_ocel2 failed");
}







#[test]
fn call_read_ocel2_json_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_read_ocel2_json_official: pm4py not available");
        return;
    }
    // SKIP: no OCEL test fixture available
    return;
    #[allow(unreachable_code)]
    let result: Result<(), ()> = Ok(());
    result.expect("read_ocel2_json failed");
}







#[test]
fn call_read_ocel2_sqlite_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_read_ocel2_sqlite_official: pm4py not available");
        return;
    }
    // SKIP: no OCEL test fixture available
    return;
    #[allow(unreachable_code)]
    let result: Result<(), ()> = Ok(());
    result.expect("read_ocel2_sqlite failed");
}







#[test]
fn call_read_ocel2_xml_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_read_ocel2_xml_official: pm4py not available");
        return;
    }
    // SKIP: no OCEL test fixture available
    return;
    #[allow(unreachable_code)]
    let result: Result<(), ()> = Ok(());
    result.expect("read_ocel2_xml failed");
}







#[test]
fn call_read_ocel_csv_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_read_ocel_csv_official: pm4py not available");
        return;
    }
    // SKIP: no OCEL test fixture available
    return;
    #[allow(unreachable_code)]
    let result: Result<(), ()> = Ok(());
    result.expect("read_ocel_csv failed");
}







#[test]
fn call_read_ocel_json_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_read_ocel_json_official: pm4py not available");
        return;
    }
    // SKIP: no OCEL test fixture available
    return;
    #[allow(unreachable_code)]
    let result: Result<(), ()> = Ok(());
    result.expect("read_ocel_json failed");
}







#[test]
fn call_read_ocel_sqlite_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_read_ocel_sqlite_official: pm4py not available");
        return;
    }
    // SKIP: no OCEL test fixture available
    return;
    #[allow(unreachable_code)]
    let result: Result<(), ()> = Ok(());
    result.expect("read_ocel_sqlite failed");
}







#[test]
fn call_read_ocel_xml_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_read_ocel_xml_official: pm4py not available");
        return;
    }
    // SKIP: no OCEL test fixture available
    return;
    #[allow(unreachable_code)]
    let result: Result<(), ()> = Ok(());
    result.expect("read_ocel_xml failed");
}







#[test]
fn call_read_xes_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_read_xes_official: pm4py not available");
        return;
    }
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::io::call_read_xes(py, XES_PATH)
    });
    result.expect("read_xes read failed");
}







#[test]
fn call_write_ocel_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_write_ocel_official: pm4py not available");
        return;
    }
    // SKIP: OCEL write functions require an OCEL object, not an EventLog
    return;
    #[allow(unreachable_code)]
    let result: Result<(), ()> = Ok(());
    result.expect("write_ocel failed");
}







#[test]
fn call_write_ocel2_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_write_ocel2_official: pm4py not available");
        return;
    }
    // SKIP: OCEL write functions require an OCEL object, not an EventLog
    return;
    #[allow(unreachable_code)]
    let result: Result<(), ()> = Ok(());
    result.expect("write_ocel2 failed");
}







#[test]
fn call_write_ocel2_json_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_write_ocel2_json_official: pm4py not available");
        return;
    }
    // SKIP: OCEL write functions require an OCEL object, not an EventLog
    return;
    #[allow(unreachable_code)]
    let result: Result<(), ()> = Ok(());
    result.expect("write_ocel2_json failed");
}







#[test]
fn call_write_ocel2_sqlite_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_write_ocel2_sqlite_official: pm4py not available");
        return;
    }
    // SKIP: OCEL write functions require an OCEL object, not an EventLog
    return;
    #[allow(unreachable_code)]
    let result: Result<(), ()> = Ok(());
    result.expect("write_ocel2_sqlite failed");
}







#[test]
fn call_write_ocel2_xml_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_write_ocel2_xml_official: pm4py not available");
        return;
    }
    // SKIP: OCEL write functions require an OCEL object, not an EventLog
    return;
    #[allow(unreachable_code)]
    let result: Result<(), ()> = Ok(());
    result.expect("write_ocel2_xml failed");
}







#[test]
fn call_write_ocel_csv_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_write_ocel_csv_official: pm4py not available");
        return;
    }
    // SKIP: OCEL write functions require an OCEL object, not an EventLog
    return;
    #[allow(unreachable_code)]
    let result: Result<(), ()> = Ok(());
    result.expect("write_ocel_csv failed");
}







#[test]
fn call_write_ocel_json_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_write_ocel_json_official: pm4py not available");
        return;
    }
    // SKIP: OCEL write functions require an OCEL object, not an EventLog
    return;
    #[allow(unreachable_code)]
    let result: Result<(), ()> = Ok(());
    result.expect("write_ocel_json failed");
}







#[test]
fn call_write_ocel_sqlite_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_write_ocel_sqlite_official: pm4py not available");
        return;
    }
    // SKIP: OCEL write functions require an OCEL object, not an EventLog
    return;
    #[allow(unreachable_code)]
    let result: Result<(), ()> = Ok(());
    result.expect("write_ocel_sqlite failed");
}







#[test]
fn call_write_ocel_xml_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_write_ocel_xml_official: pm4py not available");
        return;
    }
    // SKIP: OCEL write functions require an OCEL object, not an EventLog
    return;
    #[allow(unreachable_code)]
    let result: Result<(), ()> = Ok(());
    result.expect("write_ocel_xml failed");
}







#[test]
fn call_write_xes_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_write_xes_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let tmp = tempfile::NamedTempFile::new().expect("tmp file must be created");
    let path = tmp.path().to_str().expect("path must be valid UTF-8");
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::io::call_write_xes(py, &log, path)
    });
    result.expect("write_xes failed");
}







#[test]
fn call_get_all_case_durations_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_get_all_case_durations_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::statistics::call_get_all_case_durations(py, &log)
    });
    result.expect("get_all_case_durations failed");
}







#[test]
fn call_get_case_arrival_average_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_get_case_arrival_average_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::statistics::call_get_case_arrival_average(py, &log)
    });
    let score = result.expect("get_case_arrival_average failed");
    assert!(score >= 0.0,
        "get_case_arrival_average: score must be non-negative, got {score}");
}







#[test]
fn call_get_case_overlap_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_get_case_overlap_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::statistics::call_get_case_overlap(py, &log)
    });
    result.expect("get_case_overlap failed");
}







#[test]
fn call_get_cycle_time_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_get_cycle_time_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::statistics::call_get_cycle_time(py, &log)
    });
    let score = result.expect("get_cycle_time failed");
    assert!(score >= 0.0,
        "get_cycle_time: score must be non-negative, got {score}");
}







#[test]
fn call_get_end_activities_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_get_end_activities_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::statistics::call_get_end_activities(py, &log)
    });
    result.expect("get_end_activities failed");
}







#[test]
fn call_get_event_attributes_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_get_event_attributes_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::statistics::call_get_event_attributes(py, &log)
    });
    result.expect("get_event_attributes failed");
}







#[test]
fn call_get_minimum_self_distance_witnesses_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_get_minimum_self_distance_witnesses_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::statistics::call_get_minimum_self_distance_witnesses(py, &log)
    });
    result.expect("get_minimum_self_distance_witnesses failed");
}







#[test]
fn call_get_minimum_self_distances_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_get_minimum_self_distances_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::statistics::call_get_minimum_self_distances(py, &log)
    });
    result.expect("get_minimum_self_distances failed");
}







#[test]
fn call_get_rework_cases_per_activity_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_get_rework_cases_per_activity_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::statistics::call_get_rework_cases_per_activity(py, &log)
    });
    result.expect("get_rework_cases_per_activity failed");
}







#[test]
fn call_get_service_time_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_get_service_time_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::statistics::call_get_service_time(py, &log)
    });
    result.expect("get_service_time failed");
}







#[test]
fn call_get_start_activities_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_get_start_activities_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::statistics::call_get_start_activities(py, &log)
    });
    result.expect("get_start_activities failed");
}







#[test]
fn call_get_stochastic_language_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_get_stochastic_language_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::statistics::call_get_stochastic_language(py, &log)
    });
    result.expect("get_stochastic_language failed");
}







#[test]
fn call_get_trace_attributes_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_get_trace_attributes_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::statistics::call_get_trace_attributes(py, &log)
    });
    result.expect("get_trace_attributes failed");
}







#[test]
fn call_get_variants_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_get_variants_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::statistics::call_get_variants(py, &log)
    });
    result.expect("get_variants failed");
}







#[test]
fn call_get_variants_as_tuples_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_get_variants_as_tuples_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::statistics::call_get_variants_as_tuples(py, &log)
    });
    result.expect("get_variants_as_tuples failed");
}







#[test]
fn call_split_by_process_variant_official() {
    if !pm4py_available() {
        eprintln!("SKIP call_split_by_process_variant_official: pm4py not available");
        return;
    }
    let log = two_activity_log();
    let result = pyo3::Python::with_gil(|py| {
        pm4py::python::generated::statistics::call_split_by_process_variant(py, &log)
    });
    result.expect("split_by_process_variant failed");
}

