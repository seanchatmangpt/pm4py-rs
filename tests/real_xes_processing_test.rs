/// Real XES Processing Integration Test
///
/// This test closes the critical gap: 30+ real XES event log files exist in the
/// repository but were NEVER used in any test. Process mining had never been
/// tested with real data until now.
///
/// Uses: pm4py-rust/test_data/running-example.xes (Fluxicon Nitro benchmark)
/// — 6 traces, 42 events, 8 unique activities (insurance claim handling process)
use pm4py::conformance::TokenReplay;
use pm4py::discovery::AlphaMiner;
use pm4py::io::XESReader;
use pm4py::models::PetriNet;
use std::collections::HashSet;
use std::path::Path;

/// Path to the real XES file (Fluxicon running-example, a classic PM benchmark)
const RUNNING_EXAMPLE_XES: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/test_data/running-example.xes");

// ---------------------------------------------------------------------------
// 1. XES Parsing — prove real data loads correctly
// ---------------------------------------------------------------------------

#[test]
fn test_real_xes_parses_with_nonzero_traces() {
    let reader = XESReader::new();
    let log = reader
        .read(Path::new(RUNNING_EXAMPLE_XES))
        .expect("running-example.xes must parse without error");

    let trace_count = log.traces.len();
    assert!(
        trace_count > 0,
        "Real XES must have > 0 traces, got {}",
        trace_count
    );

    println!("[REAL XES] Trace count: {}", trace_count);
}

#[test]
fn test_real_xes_parses_with_nonzero_events() {
    let reader = XESReader::new();
    let log = reader
        .read(Path::new(RUNNING_EXAMPLE_XES))
        .expect("running-example.xes must parse without error");

    let event_count: usize = log.traces.iter().map(|t| t.events.len()).sum();
    assert!(
        event_count > 0,
        "Real XES must have > 0 events, got {}",
        event_count
    );

    println!("[REAL XES] Event count: {}", event_count);
}

#[test]
fn test_real_xes_expected_trace_count() {
    // running-example.xes is a well-known benchmark with exactly 6 traces
    let reader = XESReader::new();
    let log = reader
        .read(Path::new(RUNNING_EXAMPLE_XES))
        .expect("running-example.xes must parse");

    assert_eq!(
        log.traces.len(),
        6,
        "running-example.xes has exactly 6 traces"
    );
}

#[test]
fn test_real_xes_unique_activities() {
    let reader = XESReader::new();
    let log = reader
        .read(Path::new(RUNNING_EXAMPLE_XES))
        .expect("running-example.xes must parse");

    let activities: HashSet<&str> = log
        .traces
        .iter()
        .flat_map(|t| t.events.iter().map(|e| e.activity.as_str()))
        .collect();

    // running-example.xes activities: register request, examine casually,
    // examine thoroughly, check ticket, decide, reinitiate request,
    // pay compensation, reject request
    assert!(
        activities.len() >= 5,
        "Expected at least 5 unique activities from running-example.xes, got {}: {:?}",
        activities.len(),
        activities
    );

    println!(
        "[REAL XES] Unique activities ({}): {:?}",
        activities.len(),
        activities
    );
}

// ---------------------------------------------------------------------------
// 2. Discovery — prove Alpha Miner produces a real Petri net from real data
// ---------------------------------------------------------------------------

#[test]
fn test_alpha_miner_discovers_petri_net_from_real_xes() {
    let reader = XESReader::new();
    let log = reader
        .read(Path::new(RUNNING_EXAMPLE_XES))
        .expect("running-example.xes must parse");

    let miner = AlphaMiner::new();
    let net: PetriNet = miner.discover(&log);

    assert!(
        !net.places.is_empty(),
        "Discovered Petri net must have > 0 places, got 0"
    );
    assert!(
        !net.transitions.is_empty(),
        "Discovered Petri net must have > 0 transitions, got 0"
    );
    assert!(
        !net.arcs.is_empty(),
        "Discovered Petri net must have > 0 arcs, got 0"
    );

    println!(
        "[REAL XES → Alpha Miner] Places: {}, Transitions: {}, Arcs: {}",
        net.places.len(),
        net.transitions.len(),
        net.arcs.len()
    );
}

#[test]
fn test_alpha_miner_transitions_match_activities() {
    let reader = XESReader::new();
    let log = reader
        .read(Path::new(RUNNING_EXAMPLE_XES))
        .expect("running-example.xes must parse");

    let activities = log.activities();
    let miner = AlphaMiner::new();
    let net = miner.discover(&log);

    // Every activity should appear as a labelled transition
    let transition_labels: HashSet<String> = net
        .transitions
        .iter()
        .filter_map(|t| t.label.clone())
        .collect();

    for activity in &activities {
        assert!(
            transition_labels.contains(activity),
            "Activity '{}' missing from discovered Petri net transitions. Found: {:?}",
            activity,
            transition_labels
        );
    }
}

#[test]
fn test_alpha_miner_net_has_initial_and_final_place() {
    let reader = XESReader::new();
    let log = reader
        .read(Path::new(RUNNING_EXAMPLE_XES))
        .expect("running-example.xes must parse");

    let miner = AlphaMiner::new();
    let net = miner.discover(&log);

    assert!(
        net.initial_place.is_some(),
        "Discovered net must have an initial place"
    );
    assert!(
        net.final_place.is_some(),
        "Discovered net must have a final place"
    );
}

// ---------------------------------------------------------------------------
// 3. Conformance — prove token replay works on real-data-discovered model
// ---------------------------------------------------------------------------

#[test]
fn test_token_replay_conformance_on_real_xes() {
    let reader = XESReader::new();
    let log = reader
        .read(Path::new(RUNNING_EXAMPLE_XES))
        .expect("running-example.xes must parse");

    let miner = AlphaMiner::new();
    let net = miner.discover(&log);

    let replay = TokenReplay::new();
    let result = replay.check(&log, &net);

    assert!(
        result.fitness > 0.0,
        "Conformance fitness must be > 0.0 for a model discovered from the same log, got {}",
        result.fitness
    );

    println!(
        "[REAL XES → Conformance] Fitness: {:.4}, Precision: {:.4}, Generalization: {:.4}, Conformant: {}",
        result.fitness, result.precision, result.generalization, result.is_conformant
    );
}

#[test]
fn test_token_replay_fitness_reasonable_range() {
    // A model discovered from the same log should have reasonable fitness (> 0.3)
    let reader = XESReader::new();
    let log = reader
        .read(Path::new(RUNNING_EXAMPLE_XES))
        .expect("running-example.xes must parse");

    let miner = AlphaMiner::new();
    let net = miner.discover(&log);

    let replay = TokenReplay::new();
    let result = replay.check(&log, &net);

    assert!(
        result.fitness >= 0.3,
        "Fitness of model discovered from same log should be >= 0.3, got {:.4}. \
         This indicates a serious issue in the Alpha Miner or Token Replay.",
        result.fitness
    );
    assert!(
        result.fitness <= 1.0,
        "Fitness must be <= 1.0, got {:.4}",
        result.fitness
    );
}

// ---------------------------------------------------------------------------
// 4. Full pipeline statistics printout (single combined test)
// ---------------------------------------------------------------------------

#[test]
fn test_full_real_xes_pipeline_statistics() {
    let reader = XESReader::new();
    let log = reader
        .read(Path::new(RUNNING_EXAMPLE_XES))
        .expect("running-example.xes must parse");

    let trace_count = log.traces.len();
    let event_count: usize = log.traces.iter().map(|t| t.events.len()).sum();
    let activities: HashSet<&str> = log
        .traces
        .iter()
        .flat_map(|t| t.events.iter().map(|e| e.activity.as_str()))
        .collect();

    let miner = AlphaMiner::new();
    let net = miner.discover(&log);

    let replay = TokenReplay::new();
    let result = replay.check(&log, &net);

    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║  REAL XES PROCESSING PIPELINE — COMPLETE STATISTICS     ║");
    println!("╠══════════════════════════════════════════════════════════╣");
    println!("║  XES File: running-example.xes (Fluxicon Nitro)        ║");
    println!(
        "║  Traces:          {:>6}                                ║",
        trace_count
    );
    println!(
        "║  Events:          {:>6}                                ║",
        event_count
    );
    println!(
        "║  Unique Activities: {:>4}                                ║",
        activities.len()
    );
    println!("║  Activities: {:?}", activities);
    println!("║  ──────────────────────────────────────────────────────  ║");
    println!("║  Discovery (Alpha Miner):                               ║");
    println!(
        "║    Places:        {:>6}                                ║",
        net.places.len()
    );
    println!(
        "║    Transitions:   {:>6}                                ║",
        net.transitions.len()
    );
    println!(
        "║    Arcs:          {:>6}                                ║",
        net.arcs.len()
    );
    println!("║  ──────────────────────────────────────────────────────  ║");
    println!("║  Conformance (Token Replay):                            ║");
    println!(
        "║    Fitness:       {:>8.4}                              ║",
        result.fitness
    );
    println!(
        "║    Precision:     {:>8.4}                              ║",
        result.precision
    );
    println!(
        "║    Generalization:{:>8.4}                              ║",
        result.generalization
    );
    println!(
        "║    Conformant:    {:>8}                              ║",
        result.is_conformant
    );
    println!("╚══════════════════════════════════════════════════════════╝");

    // Final assertions covering the entire pipeline
    assert!(trace_count > 0);
    assert!(event_count > 0);
    assert!(!activities.is_empty());
    assert!(!net.places.is_empty());
    assert!(!net.transitions.is_empty());
    assert!(!net.arcs.is_empty());
    assert!(result.fitness > 0.0);
}

// ---------------------------------------------------------------------------
// 5. Cross-file test — load a BusinessOS XES file from a different directory
// ---------------------------------------------------------------------------

#[test]
fn test_businessos_xes_files_parse_correctly() {
    // Test that BusinessOS-generated XES files also parse correctly
    let bos_xes_path = Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../BusinessOS/bos/tests/data/implicit_choice_ap1.xes"
    ));

    if !bos_xes_path.exists() {
        println!(
            "[SKIP] BusinessOS XES file not found at {:?} (expected in monorepo layout)",
            bos_xes_path
        );
        return;
    }

    let reader = XESReader::new();
    let log = reader
        .read(bos_xes_path)
        .expect("BusinessOS XES file must parse");

    let trace_count = log.traces.len();
    let event_count: usize = log.traces.iter().map(|t| t.events.len()).sum();

    assert!(trace_count > 0, "BusinessOS XES file must have > 0 traces");
    assert!(event_count > 0, "BusinessOS XES file must have > 0 events");

    println!(
        "[BusinessOS XES] Parsed implicit_choice_ap1.xes: {} traces, {} events",
        trace_count, event_count
    );
}

#[test]
fn test_yawlv6_repair_example_xes_parses() {
    // Test the YAWL v6 RepairExample.xes — a different real-world process
    let yawl_xes_path = Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../yawlv6/yawl-core/src/test/resources/processmining/xes/RepairExample.xes"
    ));

    if !yawl_xes_path.exists() {
        println!(
            "[SKIP] YAWL RepairExample.xes not found at {:?}",
            yawl_xes_path
        );
        return;
    }

    let reader = XESReader::new();
    let log = reader
        .read(yawl_xes_path)
        .expect("YAWL RepairExample.xes must parse");

    let trace_count = log.traces.len();
    let event_count: usize = log.traces.iter().map(|t| t.events.len()).sum();

    assert!(trace_count > 0, "RepairExample must have > 0 traces");
    assert!(event_count > 0, "RepairExample must have > 0 events");

    // Also run discovery + conformance on this different dataset
    if trace_count > 0 {
        let miner = AlphaMiner::new();
        let net = miner.discover(&log);

        assert!(
            !net.places.is_empty(),
            "Alpha Miner must discover places from RepairExample"
        );
        assert!(
            !net.transitions.is_empty(),
            "Alpha Miner must discover transitions from RepairExample"
        );

        let replay = TokenReplay::new();
        let result = replay.check(&log, &net);

        println!(
            "[YAWL RepairExample] {} traces, {} events → {} places, {} transitions, fitness={:.4}",
            trace_count,
            event_count,
            net.places.len(),
            net.transitions.len(),
            result.fitness
        );

        assert!(result.fitness > 0.0, "Fitness must be > 0.0");
    }
}
