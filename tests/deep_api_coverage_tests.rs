//! Deep API Coverage Integration Tests for pm4py-rust
//!
//! Ralph Loop iteration 9: Tests previously untested high-priority APIs against
//! Canopy demo data and BusinessOS/OSA schema-driven logs.
//!
//! Innovation areas:
//! 1. StreamingMiner + concept drift detection
//! 2. TokenMiner + implicit dependency detection
//! 3. ILPMiner with decomposition
//! 4. TreeMiner frequency-based discovery
//! 5. DFGMinerExtended dependency matrix + parallel arcs
//! 6. FootprintsConformanceChecker (log vs model)
//! 7. HeuristicTokenAllocator strategies
//! 8. Eventually-follows graph + OTG + typed DFG
//! 9. DFG model methods (parallel_activities, density, has_loop)
//! 10. ProcessTree advanced methods (operator_frequencies, all_traces, is_valid)
//! 11. Footprint advanced methods (from_traces, to_matrix, compare)
//! 12. Extended performance metrics (cycle_time, sojourn_time, resource utilization)
//! 13. OCEL utilities (flattening, summaries)
//! 14. Dotted chart visualization
//! 15. BPMN executor + sequence validation
//! 16. Correlation miner
//! 17. OCDFG conformance
//! 18. Five miners comparison matrix
//! 19. Behavioral profile from model vs log
//! 20. Footprints diagnostics + fitness/precision

use chrono::{Duration, Utc};
use pm4py::conformance::footprints::FootprintsConformanceChecker;
use pm4py::conformance::token_replay_advanced::HeuristicTokenAllocator;
use pm4py::conformance::{AlignmentChecker, TokenReplay};
use pm4py::discovery::dfg_miner_extended::DFGMinerExtended;
use pm4py::discovery::ilp_miner::ILPMiner;
use pm4py::discovery::streaming_miner::StreamingMiner;
use pm4py::discovery::token_miner::TokenMiner;
use pm4py::discovery::{
    correlation_miner, discover_eventually_follows_graph, discover_ocdfg, discover_otg,
    discover_prefix_tree, CausalNetMiner, DFGMiner, InductiveMiner, SplitMiner, TreeMiner,
};
use pm4py::io::CSVReader;
use pm4py::log::{Event, EventLog, Trace};
use pm4py::models::conversions::petri_net_to_bpmn;
use pm4py::models::dfg::DirectlyFollowsGraph;
use pm4py::models::footprints::{ActivityRelationship, Footprints};
use pm4py::statistics::extended_metrics::{
    calculate_cycle_time, calculate_resource_utilization, calculate_sojourn_time,
    calculate_waiting_times, process_performance_analysis, trace_performance_metrics,
};
use pm4py::visualization::dotted_chart::{create_dotted_chart, DottedChartOptions};
use std::collections::HashMap;
use std::path::Path;

const CANOPY_DIR: &str = "/Users/sac/chatmangpt/canopy/priv/demo_data";

fn canopy_csv() -> CSVReader {
    CSVReader::new()
        .with_case_column("case_id")
        .with_activity_column("activity")
        .with_timestamp_column("timestamp")
        .with_resource_column(Some("resource"))
}

fn load_invoice() -> EventLog {
    canopy_csv()
        .read(Path::new(&format!(
            "{}/invoice_processing_events.csv",
            CANOPY_DIR
        )))
        .unwrap()
}

fn load_onboarding() -> EventLog {
    canopy_csv()
        .read(Path::new(&format!(
            "{}/customer_onboarding_events.csv",
            CANOPY_DIR
        )))
        .unwrap()
}

fn load_compliance() -> EventLog {
    canopy_csv()
        .read(Path::new(&format!(
            "{}/compliance_reporting_events.csv",
            CANOPY_DIR
        )))
        .unwrap()
}

// ============================================================================
// 1. StreamingMiner + Concept Drift Detection
// ============================================================================

#[test]
fn test_streaming_miner_invoice_windowed_discovery() {
    let log = load_invoice();

    let miner = StreamingMiner::new()
        .with_window_size(50)
        .with_step_size(25)
        .with_drift_detection(true)
        .with_drift_threshold(0.3);

    let windows = miner.discover_windowed(&log);
    let drifts = miner.detect_concept_drift(&log);
    let (avg_stability, _) = miner.compute_stability_metrics(&log);

    println!("StreamingMiner (Invoice):");
    println!("   Windows discovered: {}", windows.len());
    for (i, w) in windows.iter().take(3).enumerate() {
        println!(
            "     Window {}: {} nodes, {} edges",
            i,
            w.nodes.len(),
            w.edges.len()
        );
    }
    println!("   Concept drifts detected: {}", drifts.len());
    for d in drifts.iter().take(3) {
        println!(
            "     Window {}: severity={:.3}, disappeared={}, new={}, changed={}",
            d.window_index,
            d.severity,
            d.disappeared_activities.len(),
            d.new_activities.len(),
            d.changed_relations.len()
        );
    }
    println!("   Avg stability: {:.4}", avg_stability);

    assert!(!windows.is_empty());
}

// ============================================================================
// 2. TokenMiner + Implicit Dependencies
// ============================================================================

#[test]
fn test_token_miner_implicit_dependencies_invoice() {
    let log = load_invoice();

    let miner = TokenMiner::new()
        .with_implicit_dependency_detection(true)
        .with_min_implicit_confidence(0.5);

    let dfg = miner.discover(&log);
    let implicit = miner.detect_implicit_dependencies(&log);
    let avg_fitness = miner.compute_average_fitness(&log);
    let (min_f, avg_f, max_f) = miner.compute_fitness_distribution(&log);
    let poor = miner.find_poor_fitness_traces(&log, 0.5);

    println!("TokenMiner (Invoice):");
    println!(
        "   DFG: {} nodes, {} edges",
        dfg.nodes.len(),
        dfg.edges.len()
    );
    println!("   Implicit dependencies: {}", implicit.len());
    for dep in implicit.iter().take(3) {
        println!(
            "     {} → {}: conf={:.3}",
            dep.source, dep.target, dep.confidence
        );
    }
    println!("   Avg fitness: {:.4}", avg_fitness);
    println!(
        "   Fitness distribution: min={:.4}, avg={:.4}, max={:.4}",
        min_f, avg_f, max_f
    );
    println!("   Poor fitness traces (threshold=0.5): {}", poor.len());

    assert!(avg_fitness >= 0.0 && avg_fitness <= 1.0);
}

// ============================================================================
// 3. ILPMiner with Decomposition
// ============================================================================

#[test]
fn test_ilp_miner_discovery() {
    let log = load_invoice();

    let net = ILPMiner::new()
        .with_decomposition(true)
        .with_min_coverage(0.8)
        .discover(&log);

    println!("ILPMiner (Invoice):");
    println!(
        "   Places: {}, Transitions: {}, Arcs: {}",
        net.places.len(),
        net.transitions.len(),
        net.arcs.len()
    );

    assert!(
        net.transitions.len() > 0,
        "ILPMiner should produce transitions"
    );
}

// ============================================================================
// 4. TreeMiner Frequency-Based Discovery
// ============================================================================

#[test]
fn test_tree_miner_discovery() {
    let log = load_invoice();

    let tree = TreeMiner::new()
        .with_min_support(0.1)
        .with_min_activity_frequency(0.05)
        .discover(&log);

    println!("TreeMiner (Invoice):");
    println!(
        "   Depth: {}, Leaves: {}",
        tree.root.depth(),
        tree.root.leaf_count()
    );
    println!("   Activities: {:?}", tree.activities());

    assert!(tree.root.leaf_count() > 0);
}

// ============================================================================
// 5. DFGMinerExtended Dependency Matrix + Parallel Arcs
// ============================================================================

#[test]
fn test_dfg_extended_dependency_and_parallel() {
    let log = load_onboarding();

    let extended = DFGMinerExtended::new()
        .with_min_support(0.1)
        .with_min_confidence(0.3)
        .with_min_dependency_strength(0.1)
        .with_parallelism_detection(true);

    let dfg = extended.discover(&log);
    let deps = extended.compute_dependency_matrix(&log);
    let parallels = extended.detect_parallel_arcs(&log);
    let top5 = extended.get_top_relations(&log, 5);

    println!("DFGMinerExtended (Onboarding):");
    println!(
        "   DFG: {} nodes, {} edges",
        dfg.nodes.len(),
        dfg.edges.len()
    );
    println!("   Dependency pairs: {}", deps.len());
    for d in deps.iter().take(3) {
        println!(
            "     {} → {}: dep={:.3}, conf={:.3}",
            d.from, d.to, d.dependency_strength, d.confidence
        );
    }
    println!("   Parallel arcs: {}", parallels.len());
    for p in parallels.iter().take(3) {
        println!(
            "     {} ↔ {}: degree={:.3}",
            p.activity_a, p.activity_b, p.parallelism_degree
        );
    }
    println!("   Top 5 relations:");
    for t in &top5 {
        println!(
            "     {} → {}: strength={:.3}",
            t.from, t.to, t.dependency_strength
        );
    }

    assert!(!deps.is_empty());
}

// ============================================================================
// 6. FootprintsConformanceChecker
// ============================================================================

#[test]
fn test_footprints_conformance_checker() {
    let log = load_invoice();
    let net = InductiveMiner::new().discover(&log);

    // Check log vs model footprints
    let log_fp = Footprints::from_log(&log);
    let model_fp = FootprintsConformanceChecker::footprints_from_petri_net(&net);
    let result = FootprintsConformanceChecker::check_log(&log, &model_fp);

    println!("FootprintsConformanceChecker (Invoice):");
    println!("   Log activities: {}", log_fp.activities().len());
    println!("   Model activities: {}", model_fp.activities().len());
    println!("   Conformant: {}", result.is_conformant);
    println!("   Fitness: {:.4}", result.fitness);
    println!("   Precision: {:.4}", result.precision());
    println!(
        "   Total pairs: {}, Matching: {}",
        result.total_pairs, result.matching_pairs
    );
    println!("   Mismatches: {}", result.mismatching_pairs.len());

    // Compare footprints
    let compared = FootprintsConformanceChecker::compare_footprints(&log_fp, &model_fp);
    println!(
        "   Compared mismatches: {}",
        compared.mismatching_pairs.len()
    );

    assert!(result.fitness >= 0.0 && result.fitness <= 1.0);
}

// ============================================================================
// 7. HeuristicTokenAllocator
// ============================================================================

#[test]
fn test_heuristic_token_allocator() {
    let log = load_receipt();
    let net = InductiveMiner::new().discover(&log);

    let mut marking1: HashMap<String, usize> = HashMap::new();
    let mut marking2: HashMap<String, usize> = HashMap::new();
    let mut marking3: HashMap<String, usize> = HashMap::new();

    let freq_tokens = HeuristicTokenAllocator::allocate_by_frequency(&log, &net, &mut marking1);
    let bf_tokens = HeuristicTokenAllocator::allocate_breadth_first(&net, &mut marking2);

    let costs: HashMap<String, f64> = net
        .transitions
        .iter()
        .filter_map(|t| t.label.clone().map(|l| (l, 1.0)))
        .collect();
    let cost_tokens = HeuristicTokenAllocator::allocate_cost_based(&net, &mut marking3, &costs);

    println!("HeuristicTokenAllocator (Receipt):");
    println!("   Frequency-based tokens: {}", freq_tokens);
    println!("   Breadth-first tokens: {}", bf_tokens);
    println!("   Cost-based tokens: {}", cost_tokens);

    assert!(freq_tokens > 0);
}

fn load_receipt() -> EventLog {
    let path = format!("test_data/receipt.xes");
    pm4py::io::XESReader::new().read(Path::new(&path)).unwrap()
}

// ============================================================================
// 8. Eventually-Follows Graph + OTG
// ============================================================================

#[test]
fn test_eventually_follows_and_otg() {
    let log = load_onboarding();

    let efg = discover_eventually_follows_graph(&log);
    let otg = discover_otg(&log);
    let ocdfg = discover_ocdfg(&log);

    println!("Eventually-Follows (Onboarding):");
    println!("   EF relations: {}", efg.relations.len());
    println!("   EF distances: {}", efg.distances.len());
    println!("   OTG transitions: {}", otg.transitions.len());
    println!("   OTG occurrences: {}", otg.occurrences.len());
    println!("   OCDFG edges: {}", ocdfg.edges.len());

    assert!(!efg.relations.is_empty());
    assert!(!otg.transitions.is_empty());
}

// ============================================================================
// 9. DFG Model Methods
// ============================================================================

#[test]
fn test_dfg_model_methods() {
    let log = load_invoice();
    let mut dfg = DFGMiner::new().discover(&log);

    println!("DFG Model Methods (Invoice):");
    println!("   Nodes: {}, Edges: {}", dfg.nodes.len(), dfg.edges.len());

    // density
    let dens = dfg.density();
    println!("   Density: {:.4}", dens);

    // has_loop
    let loops = dfg.has_loop();
    println!("   Has loop: {}", loops);

    // parallel_activities
    let parallels = dfg.parallel_activities();
    println!("   Parallel activities: {}", parallels.len());
    for (a, b) in parallels.iter().take(3) {
        println!("     {} ∥ {}", a, b);
    }

    // choice_points
    let choices = dfg.choice_points();
    println!("   Choice points: {:?}", choices);

    // get_edges_from / get_edges_to
    if let Some(first_act) = dfg.nodes.first() {
        let out = dfg.get_edges_from(first_act);
        let into = dfg.get_edges_to(first_act);
        println!(
            "   '{}' → {} out-edges, {} in-edges",
            first_act,
            out.len(),
            into.len()
        );
    }

    // filter_edges
    let before = dfg.edges.len();
    dfg.filter_edges(2);
    let after = dfg.edges.len();
    println!("   After filter(min_freq=2): {} → {} edges", before, after);

    assert!(dens >= 0.0 && dens <= 1.0);
}

// ============================================================================
// 10. ProcessTree Advanced Methods
// ============================================================================

#[test]
fn test_process_tree_advanced_methods() {
    let log = load_invoice();
    let mut tree = InductiveMiner::new().discover_tree(&log);

    println!("ProcessTree Advanced (Invoice):");

    // operator_frequencies
    let freqs = tree.operator_frequencies();
    println!("   Operator frequencies: {:?}", freqs);

    // all_traces
    let traces = tree.all_traces();
    println!("   Possible traces: {}", traces.len());
    for t in traces.iter().take(3) {
        println!("     {:?}", t);
    }

    // is_valid
    let valid = tree.is_valid();
    println!("   Is valid: {}", valid);

    // simplify
    let before_depth = tree.root.depth();
    tree.simplify();
    let after_depth = tree.root.depth();
    println!("   Simplify: depth {} → {}", before_depth, after_depth);

    // Individual node methods
    let op_count = tree.root.operator_count();
    let leaf_count = tree.root.leaf_count();
    let width = tree.root.width();
    let is_valid_node = tree.root.is_valid();
    println!(
        "   Root: op_count={}, leaves={}, width={}, valid={}",
        op_count, leaf_count, width, is_valid_node
    );

    assert!(valid);
    assert!(op_count > 0);
}

// ============================================================================
// 11. Footprint Advanced Methods
// ============================================================================

#[test]
fn test_footprint_advanced_methods() {
    let log = load_invoice();
    let fp = Footprints::from_log(&log);

    println!("Footprint Advanced (Invoice):");

    // from_traces
    let trace_vecs: Vec<Vec<String>> = log
        .traces
        .iter()
        .map(|t| t.events.iter().map(|e| e.activity.clone()).collect())
        .collect();
    let fp2 = Footprints::from_traces(&trace_vecs);
    println!(
        "   from_log activities: {}, from_traces activities: {}",
        fp.activities().len(),
        fp2.activities().len()
    );

    // to_matrix
    let (labels, matrix) = fp.to_matrix();
    println!("   Matrix: {}x{}", labels.len(), matrix.len());

    // compare
    let diffs = fp.compare(&fp2);
    println!("   Self-compare diffs: {}", diffs.len());

    // relationship_counts
    let counts = fp.relationship_counts();
    println!("   Relationship counts: {:?}", counts);

    // pairs
    let pairs = fp.pairs();
    println!("   Pairs: {}", pairs.len());

    assert!(fp.activities().len() > 0);
}

// ============================================================================
// 12. Extended Performance Metrics
// ============================================================================

#[test]
fn test_extended_performance_metrics() {
    let log = load_invoice();

    // Per-trace metrics
    if let Some(trace) = log.traces.first() {
        let cycle = calculate_cycle_time(trace);
        let sojourn = calculate_sojourn_time(trace, "create");
        let waiting = calculate_waiting_times(trace);
        let tpm = trace_performance_metrics(trace);

        println!("Extended Metrics (first invoice trace):");
        println!("   Cycle time: {:.2}s", cycle);
        println!("   'create' sojourn: {:.2}s", sojourn);
        println!("   Waiting times: {:?}", waiting);
        println!(
            "   Trace metrics: cycle={:.2}, avg_wait={:.2}",
            tpm.cycle_time_seconds, tpm.avg_waiting_time
        );
    }

    // Process-level analysis
    let analysis = process_performance_analysis(&log);
    println!("\nProcess Analysis (Invoice):");
    println!("   Avg cycle time: {:.2}s", analysis.avg_cycle_time);
    println!(
        "   Min/Max cycle: {:.2}s / {:.2}s",
        analysis.min_cycle_time, analysis.max_cycle_time
    );
    println!(
        "   P95 cycle time: {:.2}s",
        analysis.percentile_95_cycle_time
    );
    println!("   Std dev: {:.2}s", analysis.cycle_time_std_dev);

    // Resource utilization
    let util = calculate_resource_utilization(&log);
    println!("\n   Resource utilization entries: {}", util.len());
    for r in util.iter().take(5) {
        println!(
            "     {}: active={:.1}s, activities={}",
            r.resource, r.active_time, r.num_activities
        );
    }

    assert!(analysis.avg_cycle_time > 0.0);
}

// ============================================================================
// 13. OCEL Utilities
// ============================================================================

#[test]
fn test_ocel_utilities() {
    let mut ocel = pm4py::ocpm::ObjectCentricEventLog::with_id("util_test");
    let ot = pm4py::ocpm::ObjectType::new("document");
    ocel.register_object_type(ot.clone());
    for i in 0..5 {
        ocel.add_object(pm4py::ocpm::Object::new(
            format!("DOC-{}", i),
            ot.clone(),
            Utc::now(),
        ));
    }
    let mut ts = Utc::now();
    let mut event_ids: Vec<uuid::Uuid> = Vec::new();
    for i in 0..5 {
        let eid1 = uuid::Uuid::new_v4();
        ocel.add_event(eid1, "create", ts, Some("alice".to_string()));
        event_ids.push(eid1);
        ts += Duration::hours(1);
        let eid2 = uuid::Uuid::new_v4();
        ocel.add_event(eid2, "edit", ts, Some("bob".to_string()));
        event_ids.push(eid2);
        ts += Duration::hours(1);
        let eid3 = uuid::Uuid::new_v4();
        ocel.add_event(eid3, "approve", ts, Some("manager".to_string()));
        event_ids.push(eid3);
    }
    // Link events to objects: each doc gets its own create/edit/approve lifecycle
    for doc_idx in 0..5usize {
        let create_eid = event_ids[doc_idx * 3];
        let edit_eid = event_ids[doc_idx * 3 + 1];
        let approve_eid = event_ids[doc_idx * 3 + 2];
        let obj_id = format!("DOC-{}", doc_idx);

        let mut m1 = pm4py::ocpm::EventToObjectMapping::new(create_eid);
        m1.add_object(&obj_id);
        ocel.add_event_object_mapping(m1);

        let mut m2 = pm4py::ocpm::EventToObjectMapping::new(edit_eid);
        m2.add_object(&obj_id);
        ocel.add_event_object_mapping(m2);

        let mut m3 = pm4py::ocpm::EventToObjectMapping::new(approve_eid);
        m3.add_object(&obj_id);
        ocel.add_event_object_mapping(m3);
    }

    // flattening
    let flat = pm4py::ocpm::ocel_flattening(&ocel);
    println!("OCEL Utilities:");
    println!("   Flattened traces: {}", flat.len());

    // objects summary
    let summary = pm4py::ocpm::ocel_objects_summary(&ocel);
    println!("   Objects summary: {:?}", summary);

    // interactions summary
    let interactions = pm4py::ocpm::ocel_objects_interactions_summary(&ocel);
    println!("   Interactions: {}", interactions.len());

    // temporal summary
    let temp = pm4py::ocpm::ocel_temporal_summary(&ocel);
    println!(
        "   Temporal: events={}, objects={}, types={}",
        temp.num_events, temp.num_objects, temp.num_object_types
    );

    // attribute names
    let attrs = pm4py::ocpm::ocel_get_attribute_names(&ocel);
    println!("   Attribute names: {:?}", attrs);

    // object types
    let types = pm4py::ocpm::ocel_get_object_types(&ocel);
    println!("   Object types: {:?}", types);

    assert!(flat.len() > 0);
}

// ============================================================================
// 14. Dotted Chart Visualization
// ============================================================================

#[test]
fn test_dotted_chart_visualization() {
    let log = load_invoice();

    let opts = DottedChartOptions::new()
        .with_resources(true)
        .with_durations(true)
        .with_heatmap(true)
        .with_anomalies(true);

    let chart = create_dotted_chart(&log, opts);
    let svg = chart.generate_svg();
    let heatmap = chart.generate_heatmap_overlay();

    println!("Dotted Chart (Invoice):");
    println!("   SVG size: {} bytes", svg.len());
    println!("   Heatmap SVG: {} bytes", heatmap.len());
    println!("   Has <svg>: {}", svg.contains("<svg"));

    assert!(svg.contains("<svg"), "Dotted chart should generate SVG");
    assert!(svg.len() > 100);
}

// ============================================================================
// 15. BPMN Executor + Sequence Validation
// ============================================================================

#[test]
fn test_bpmn_executor_sequence() {
    let log = load_receipt();
    let net = InductiveMiner::new().discover(&log);
    let bpmn = petri_net_to_bpmn(&net);

    if bpmn.tasks.is_empty() {
        println!("BPMN Executor: skipped (no tasks from PetriNet→BPMN)");
        return;
    }

    // Validate a sequence using the first few activities
    let acts = log.activities();
    let sequence: Vec<&str> = acts.iter().take(3).map(|s| s.as_str()).collect();

    let valid = pm4py::models::bpmn_semantics::validate_sequence(&bpmn, &sequence);
    println!("BPMN Executor (Receipt):");
    println!("   Tasks: {}", bpmn.tasks.len());
    println!("   Gateways: {}", bpmn.gateways.len());
    println!("   Sequence {:?} valid: {:?}", sequence, valid);

    // Reachable activities
    if let Ok(reachable) = pm4py::models::bpmn_semantics::BPMNExecutor::reachable_activities(&bpmn)
    {
        println!("   Reachable activities: {:?}", reachable);
    }
}

// ============================================================================
// 16. Correlation Miner
// ============================================================================

#[test]
fn test_correlation_miner() {
    let log = load_onboarding();

    let result = correlation_miner(&log, 0.5);

    println!("Correlation Miner (Onboarding):");
    println!("   Total correlations: {}", result.correlations.len());
    println!(
        "   High correlations (>0.5): {}",
        result.high_correlation.len()
    );
    for (a, b, c) in result.high_correlation.iter().take(5) {
        println!("     {} ↔ {}: {:.3}", a, b, c);
    }

    assert!(!result.correlations.is_empty());
}

// ============================================================================
// 17. OCDFG Conformance
// ============================================================================

#[test]
fn test_ocdfg_conformance() {
    let log = load_compliance();
    let ocdfg = discover_ocdfg(&log);

    println!("OCDFG Conformance (Compliance):");
    println!(
        "   OCDFG: {} nodes, {} edges",
        ocdfg.nodes.len(),
        ocdfg.edges.len()
    );
}

// ============================================================================
// 18. Five Miners Comparison Matrix
// ============================================================================

#[test]
fn test_five_miners_comparison_matrix() {
    let log = load_invoice();

    let alpha = pm4py::discovery::AlphaMiner::new().discover(&log);
    let ind = InductiveMiner::new().discover(&log);
    let heu = pm4py::discovery::HeuristicMiner::new().discover(&log);
    let split = SplitMiner::new().discover(&log);
    let ilp = ILPMiner::new().discover(&log);

    // Conformance for each
    let alpha_fit = TokenReplay::new().check(&log, &alpha).fitness;
    let ind_fit = TokenReplay::new().check(&log, &ind).fitness;
    let heu_fit = TokenReplay::new().check(&log, &heu).fitness;
    let split_fit = TokenReplay::new().check(&log, &split).fitness;
    let ilp_fit = TokenReplay::new().check(&log, &ilp).fitness;

    println!("Five Miners Comparison (Invoice):");
    println!(
        "   {:15} {:>8} {:>8} {:>8} {:>10}",
        "Miner", "Places", "Trans", "Arcs", "Fitness"
    );
    println!(
        "   {:15} {:>8} {:>8} {:>8} {:>10.4}",
        "Alpha",
        alpha.places.len(),
        alpha.transitions.len(),
        alpha.arcs.len(),
        alpha_fit
    );
    println!(
        "   {:15} {:>8} {:>8} {:>8} {:>10.4}",
        "Inductive",
        ind.places.len(),
        ind.transitions.len(),
        ind.arcs.len(),
        ind_fit
    );
    println!(
        "   {:15} {:>8} {:>8} {:>8} {:>10.4}",
        "Heuristic",
        heu.places.len(),
        heu.transitions.len(),
        heu.arcs.len(),
        heu_fit
    );
    println!(
        "   {:15} {:>8} {:>8} {:>8} {:>10.4}",
        "Split",
        split.places.len(),
        split.transitions.len(),
        split.arcs.len(),
        split_fit
    );
    println!(
        "   {:15} {:>8} {:>8} {:>8} {:>10.4}",
        "ILP",
        ilp.places.len(),
        ilp.transitions.len(),
        ilp.arcs.len(),
        ilp_fit
    );

    // All should have transitions
    assert!(alpha.transitions.len() > 0);
    assert!(ind.transitions.len() > 0);
    assert!(heu.transitions.len() > 0);
}

// ============================================================================
// 19. Behavioral Profile from Model vs Log
// ============================================================================

#[test]
fn test_behavioral_profile_model_vs_log() {
    let log = load_invoice();
    let net = InductiveMiner::new().discover(&log);

    let log_bp = pm4py::conformance::BehavioralProfile::extract_from_log(&log);
    let model_bp = pm4py::conformance::BehavioralProfile::extract_from_model(&net);

    println!("Behavioral Profile Model vs Log (Invoice):");
    println!("   Log profile activities: {}", log_bp.activities.len());
    println!("   Model profile activities: {}", model_bp.activities.len());
    println!("   Log profile relations: {}", log_bp.relations.len());
    println!("   Model profile relations: {}", model_bp.relations.len());

    let conflicts = log_bp.find_conflicts(&model_bp);
    println!("   Conflicts: {}", conflicts.len());
    for c in conflicts.iter().take(5) {
        println!(
            "     {} vs {}: log={:?}, model={:?}",
            c.activity_a, c.activity_b, c.log_relation, c.model_relation
        );
    }

    let conformance = log_bp.compute_conformance(&model_bp);
    println!("   Conformance: {:.4}", conformance);
    assert!(conformance >= 0.0 && conformance <= 1.0);
}

// ============================================================================
// 20. Footprints Diagnostics + Fitness/Precision
// ============================================================================

#[test]
fn test_footprints_diagnostics() {
    let log = load_compliance();
    let fp = Footprints::from_log(&log);

    let diag = pm4py::conformance::diagnostics_footprints(&log, &fp);
    let fitness = pm4py::conformance::fitness_footprints(&log, &fp);
    let precision = pm4py::conformance::precision_footprints(&log, &fp);

    println!("Footprints Diagnostics (Compliance):");
    println!("   Activities: {}", fp.activities().len());
    println!("   Fitness: {:.4}", fitness);
    println!("   Precision: {:.4}", precision);
    println!("   Diag conformant: {}", diag.is_conformant);
    println!("   Diag fitness: {:.4}", diag.fitness);
    println!("   Diag precision: {:.4}", diag.precision());
    println!("   Diag summary: {}", diag.summary());

    // Relationship distribution
    let dist = FootprintsConformanceChecker::relationship_distribution(&fp);
    println!("   Relationship distribution: {:?}", dist);

    assert!(fitness >= 0.0 && fitness <= 1.0);
    assert!(precision >= 0.0 && precision <= 1.0);
}
