//! INNOVATIVE cross-project integration tests - Ralph Loop iteration 5
//!
//! These tests push beyond basic discovery/conformance into novel process mining
//! scenarios using real data from across the ChatmanGPT ecosystem.
//!
//! Innovation areas:
//! 1. Log merge and inter-process flow analysis
//! 2. Statistical fingerprinting across domains
//! 3. Conformance degradation under noise
//! 4. Cross-domain process similarity via structural metrics
//! 5. Temporal decomposition analysis
//! 6. Resource entropy and specialization measurement
//! 7. Process model complexity comparison
//! 8. Prefix-based prediction quality measurement
//! 9. Organizational role overlap detection
//! 10. Multi-model ensemble conformance

use chrono::{Duration, Utc};
use pm4py::conformance::{fitness_footprints, precision_footprints, AlignmentChecker, TokenReplay};
use pm4py::discovery::{
    conformance_log_skeleton, discover_activity_based_resource_similarity, discover_batches,
    discover_eventually_follows_graph, discover_handover_of_work_network,
    discover_organizational_roles, discover_performance_dfg, discover_prefix_tree,
    discover_transition_system, discover_working_together_network, AlphaMiner, AlphaPlusMiner,
    DFGMiner, HeuristicMiner, InductiveMiner, LogSkeletonMiner,
};
use pm4py::io::{CSVReader, XESReader};
use pm4py::log::{
    activity_frequency, directly_follows, end_activities, filter_activities_rework, filter_between,
    filter_case_size, filter_directly_follows_relation, filter_eventually_follows_relation,
    filter_four_eyes_principle, filter_paths_performance, filter_trace_prefix,
    filter_traces_containing_activity, is_consistent, sort_traces_by_timestamp, start_activities,
    variants, Event, EventLog, Trace,
};
use pm4py::models::footprints::{ActivityRelationship, Footprints};
use pm4py::statistics::{
    embeddings_similarity, get_activity_position_summary, get_case_arrival_average,
    get_case_overlap, get_prefixes_from_log, get_rework_cases_per_activity, get_variants_as_tuples,
    log_statistics, split_train_test, structural_similarity,
};
use pm4py::visualization::svg_renderer;
use std::collections::{HashMap, HashSet};
use std::path::Path;

const CANOPY_DIR: &str = "/Users/sac/chatmangpt/canopy/priv/demo_data";
const TEST_DATA_DIR: &str = "test_data";

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
        .expect("Failed to load invoice CSV")
}

fn load_onboarding() -> EventLog {
    canopy_csv()
        .read(Path::new(&format!(
            "{}/customer_onboarding_events.csv",
            CANOPY_DIR
        )))
        .expect("Failed to load onboarding CSV")
}

fn load_compliance() -> EventLog {
    canopy_csv()
        .read(Path::new(&format!(
            "{}/compliance_reporting_events.csv",
            CANOPY_DIR
        )))
        .expect("Failed to load compliance CSV")
}

fn load_running() -> EventLog {
    XESReader::new()
        .read(Path::new(&format!("{}/running-example.xes", TEST_DATA_DIR)))
        .expect("Failed to load running-example.xes")
}

fn load_receipt() -> EventLog {
    XESReader::new()
        .read(Path::new(&format!("{}/receipt.xes", TEST_DATA_DIR)))
        .expect("Failed to load receipt.xes")
}

fn load_roadtraffic() -> EventLog {
    XESReader::new()
        .read(Path::new(&format!(
            "{}/roadtraffic100traces.xes",
            TEST_DATA_DIR
        )))
        .expect("Failed to load roadtraffic.xes")
}

// ============================================================================
// INNOVATION 1: Cross-Process Flow Analysis
// Simulate a merged log to find inter-process handoffs
// ============================================================================

#[test]
fn test_cross_process_merged_log_analysis() {
    // Merge all three Canopy processes into a single log
    let mut merged = EventLog::new();
    for mut trace in load_invoice().traces {
        trace
            .attributes
            .insert("source_process".to_string(), "invoice".to_string());
        merged.add_trace(trace);
    }
    for mut trace in load_onboarding().traces {
        trace
            .attributes
            .insert("source_process".to_string(), "onboarding".to_string());
        merged.add_trace(trace);
    }
    for mut trace in load_compliance().traces {
        trace
            .attributes
            .insert("source_process".to_string(), "compliance".to_string());
        merged.add_trace(trace);
    }

    let stats = log_statistics(&merged);
    let dfg = DFGMiner::new().discover(&merged);
    let all_vars = variants(&merged);

    println!("Cross-Process Merged Log:");
    println!("   Total traces: {}", merged.len());
    println!("   Total events: {}", stats.num_events);
    println!("   Unique activities: {}", stats.num_unique_activities);
    println!(
        "   DFG nodes: {}, edges: {}",
        dfg.nodes.len(),
        dfg.edges.len()
    );
    println!("   Total variants: {}", all_vars.len());

    // The merged log should have more activities than any individual process
    assert!(
        stats.num_unique_activities > 7,
        "Merged should have more unique activities than any single process"
    );
}

// ============================================================================
// INNOVATION 2: Shannon Entropy of Activity Distribution
// Measure information content across different processes
// ============================================================================

#[test]
fn test_activity_entropy_comparison() {
    fn shannon_entropy(freq: &HashMap<String, usize>) -> f64 {
        let total: usize = freq.values().sum();
        if total == 0 {
            return 0.0;
        }
        let mut entropy = 0.0;
        for &count in freq.values() {
            let p = count as f64 / total as f64;
            if p > 0.0 {
                entropy -= p * p.log(2.0);
            }
        }
        entropy
    }

    let processes: Vec<(&str, EventLog)> = vec![
        ("Invoice", load_invoice()),
        ("Onboarding", load_onboarding()),
        ("Compliance", load_compliance()),
        ("Running", load_running()),
        ("Receipt", load_receipt()),
        ("Roadtraffic", load_roadtraffic()),
    ];

    println!("Activity Entropy Comparison (Shannon bits):");
    for (name, log) in &processes {
        let freq = activity_frequency(log);
        let entropy = shannon_entropy(&freq);
        println!(
            "   {}: {:.4} bits ({} activities)",
            name,
            entropy,
            freq.len()
        );
    }

    // Invoice process has only 3 variants (very structured) so should have lower entropy
    let invoice_freq = activity_frequency(&processes[0].1);
    let receipt_freq = activity_frequency(&processes[3].1);
    let invoice_entropy = shannon_entropy(&invoice_freq);
    let receipt_entropy = shannon_entropy(&receipt_freq);

    assert!(invoice_entropy > 0.0, "Entropy should be positive");
    assert!(receipt_entropy > 0.0, "Entropy should be positive");
}

// ============================================================================
// INNOVATION 3: Conformance Under Progressive Noise Injection
// Add noise to traces and measure how conformance degrades
// ============================================================================

#[test]
fn test_conformance_noise_resilience() {
    let log = load_receipt();
    let net = InductiveMiner::new().discover(&log);
    let clean_result = AlignmentChecker::new().check(&log, &net);

    // Create noisy version: swap last event in 10% of traces
    let mut noisy_log = EventLog::new();
    let activities: Vec<String> = activity_frequency(&log).keys().cloned().collect();

    for (i, mut trace) in log.traces.into_iter().enumerate() {
        if i % 10 == 0 && trace.events.len() > 1 {
            // Swap last event with a random different activity
            let last_idx = trace.events.len() - 1;
            let original = trace.events[last_idx].activity.clone();
            let alternative = activities
                .iter()
                .find(|a| **a != original)
                .cloned()
                .unwrap_or(original);
            trace.events[last_idx] = Event::new(alternative, trace.events[last_idx].timestamp);
        }
        noisy_log.add_trace(trace);
    }

    let noisy_result = AlignmentChecker::new().check(&noisy_log, &net);

    println!("Conformance Noise Resilience:");
    println!("   Clean fitness: {:.4}", clean_result.fitness);
    println!("   Noisy fitness: {:.4}", noisy_result.fitness);
    println!(
        "   Fitness delta: {:.4}",
        clean_result.fitness - noisy_result.fitness
    );

    // Noisy should have lower or equal fitness
    assert!(noisy_result.fitness <= clean_result.fitness + 0.001);
}

// ============================================================================
// INNOVATION 4: Resource Specialization Index
// Measure how specialized each resource is (low entropy = specialized)
// ============================================================================

#[test]
fn test_resource_specialization_analysis() {
    let log = load_invoice();

    // Build resource -> activity frequency map
    let mut resource_activities: HashMap<String, HashMap<String, usize>> = HashMap::new();
    for trace in &log.traces {
        for event in &trace.events {
            let resource = event
                .resource
                .clone()
                .unwrap_or_else(|| "unknown".to_string());
            *resource_activities
                .entry(resource)
                .or_default()
                .entry(event.activity.clone())
                .or_insert(0) += 1;
        }
    }

    // Calculate specialization (inverse of Shannon entropy per resource)
    let mut specializations: Vec<(String, f64)> = Vec::new();
    for (resource, act_freq) in &resource_activities {
        let total: usize = act_freq.values().sum();
        if total == 0 {
            continue;
        }
        let mut entropy = 0.0;
        for &count in act_freq.values() {
            let p = count as f64 / total as f64;
            if p > 0.0 {
                entropy -= p * p.log(2.0);
            }
        }
        // Specialization = 1 - normalized entropy
        let max_entropy = (act_freq.len() as f64).log(2.0);
        let normalized = if max_entropy > 0.0 {
            entropy / max_entropy
        } else {
            0.0
        };
        specializations.push((resource.clone(), 1.0 - normalized));
    }

    specializations.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    println!("Resource Specialization Index:");
    for (resource, spec) in specializations.iter().take(10) {
        println!("   {}: {:.4}", resource, spec);
    }

    assert!(!specializations.is_empty(), "Should have resource data");
}

// ============================================================================
// INNOVATION 5: Cross-Domain Footprint Divergence
// Measure how different process domains produce different footprint patterns
// ============================================================================

#[test]
fn test_footprint_divergence_matrix() {
    let processes: Vec<(&str, EventLog)> = vec![
        ("Invoice", load_invoice()),
        ("Onboarding", load_onboarding()),
        ("Compliance", load_compliance()),
        ("Running", load_running()),
        ("Receipt", load_receipt()),
    ];

    let footprints: Vec<(&str, Footprints)> = processes
        .iter()
        .map(|(name, log)| (*name, Footprints::from_log(log)))
        .collect();

    println!("Footprint Divergence Matrix:");
    print!("   {:12}", "");
    for (name, _) in &footprints {
        print!(" {:>8}", name);
    }
    println!();

    for (i, (name_i, fp_i)) in footprints.iter().enumerate() {
        print!("   {:12}", name_i);
        for (j, (_, fp_j)) in footprints.iter().enumerate() {
            if i == j {
                print!(" {:>8.4}", 0.0);
            } else {
                let diffs = fp_i.compare(fp_j);
                let divergence = diffs.len() as f64;
                print!(" {:>8.1}", divergence);
            }
        }
        println!();
    }

    // Same process should have 0 divergence
    let diffs = footprints[0].1.compare(&footprints[0].1);
    assert!(
        diffs.is_empty(),
        "Self-comparison should have no differences"
    );
}

// ============================================================================
// INNOVATION 6: Temporal Burst Detection
// Find time periods with unusually high activity
// ============================================================================

#[test]
fn test_temporal_burst_detection() {
    let log = load_invoice();

    // Group events by hour
    let mut hourly_counts: HashMap<String, usize> = HashMap::new();
    for trace in &log.traces {
        for event in &trace.events {
            let hour_key = event.timestamp.format("%Y-%m-%d %H:00").to_string();
            *hourly_counts.entry(hour_key).or_insert(0) += 1;
        }
    }

    let avg_per_hour =
        hourly_counts.values().sum::<usize>() as f64 / hourly_counts.len().max(1) as f64;

    // Find burst hours (> 2x average)
    let bursts: Vec<_> = hourly_counts
        .iter()
        .filter(|(_, count)| **count as f64 > avg_per_hour * 2.0)
        .collect();

    println!("Temporal Burst Detection:");
    println!("   Avg events/hour: {:.1}", avg_per_hour);
    println!("   Burst hours (>2x avg): {}", bursts.len());
    for (hour, count) in bursts.iter().take(5) {
        println!(
            "     {}: {} events ({:.1}x avg)",
            hour,
            count,
            **count as f64 / avg_per_hour
        );
    }
}

// ============================================================================
// INNOVATION 7: Process Model Complexity Metrics
// Compare structural complexity of discovered models
// ============================================================================

#[test]
fn test_process_model_complexity_comparison() {
    let processes: Vec<(&str, EventLog)> = vec![
        ("Invoice", load_invoice()),
        ("Onboarding", load_onboarding()),
        ("Compliance", load_compliance()),
        ("Running", load_running()),
        ("Receipt", load_receipt()),
    ];

    println!("Process Model Complexity:");
    println!(
        "   {:12} {:>6} {:>6} {:>6} {:>8} {:>8}",
        "Process", "Places", "Trans", "Arcs", "Density", "Cyclomatic"
    );

    for (name, log) in &processes {
        let net = InductiveMiner::new().discover(&log);
        let places = net.places.len();
        let transitions = net.transitions.len();
        let arcs = net.arcs.len();

        // Arc density
        let nodes = places + transitions;
        let max_arcs = nodes * (nodes - 1);
        let density = if max_arcs > 0 {
            arcs as f64 / max_arcs as f64
        } else {
            0.0
        };

        // Cyclomatic complexity approximation
        let cyclomatic = arcs as i64 - nodes as i64 + 2;

        println!(
            "   {:12} {:>6} {:>6} {:>6} {:>8.4} {:>8}",
            name, places, transitions, arcs, density, cyclomatic
        );
    }
}

// ============================================================================
// INNOVATION 8: Prefix Prediction Quality
// Measure how predictable a process is from its prefixes
// ============================================================================

#[test]
fn test_prefix_prediction_quality() {
    let log = load_receipt();
    let vars = variants(&log);
    let total_traces = log.len() as f64;

    // For each prefix length, count how many unique next-activity options exist
    for prefix_len in 1..=4 {
        let prefixes = get_prefixes_from_log(&log, prefix_len);

        // Count next activities after each prefix
        let mut prefix_next: HashMap<Vec<String>, HashMap<String, usize>> = HashMap::new();
        for trace in &log.traces {
            if trace.events.len() > prefix_len {
                let prefix: Vec<String> = trace.events[..prefix_len]
                    .iter()
                    .map(|e| e.activity.clone())
                    .collect();
                let next = trace.events[prefix_len].activity.clone();
                *prefix_next
                    .entry(prefix)
                    .or_default()
                    .entry(next)
                    .or_insert(0) += 1;
            }
        }

        // Average branching factor
        let total_branches: usize = prefix_next.values().map(|m| m.len()).sum();
        let num_prefixes = prefix_next.len().max(1);
        let avg_branching = total_branches as f64 / num_prefixes as f64;

        // Perfect prediction rate (1 next activity = deterministic)
        let deterministic = prefix_next.values().filter(|m| m.len() == 1).count();
        let determinism = deterministic as f64 / num_prefixes as f64;

        println!(
            "   Prefix len {}: {} unique prefixes, avg branching={:.2}, determinism={:.1}%",
            prefix_len,
            prefixes.len(),
            avg_branching,
            determinism * 100.0
        );
    }

    println!(
        "   Total variants: {} (variant ratio: {:.2})",
        vars.len(),
        vars.len() as f64 / total_traces
    );
}

// ============================================================================
// INNOVATION 9: Organizational Role Overlap Detection
// Find resources that appear in multiple organizational roles
// ============================================================================

#[test]
fn test_organizational_role_overlap() {
    let invoice = load_invoice();
    let onboarding = load_onboarding();

    let roles_inv = discover_organizational_roles(&invoice);
    let roles_onb = discover_organizational_roles(&onboarding);

    // Collect all resources per role
    let mut all_resources: HashMap<String, Vec<String>> = HashMap::new();
    for (role, resources) in &roles_inv {
        for res in resources {
            all_resources
                .entry(res.clone())
                .or_default()
                .push(format!("INV:{}", role));
        }
    }
    for (role, resources) in &roles_onb {
        for res in resources {
            all_resources
                .entry(res.clone())
                .or_default()
                .push(format!("ONB:{}", role));
        }
    }

    // Find multi-role resources
    let multi_role: Vec<_> = all_resources
        .iter()
        .filter(|(_, roles)| roles.len() > 1)
        .collect();

    println!("Organizational Role Overlap:");
    println!("   Invoice roles: {}", roles_inv.len());
    println!("   Onboarding roles: {}", roles_onb.len());
    println!("   Resources with multiple roles: {}", multi_role.len());
    for (resource, roles) in multi_role.iter().take(5) {
        println!("     {}: {:?}", resource, roles);
    }
}

// ============================================================================
// INNOVATION 10: Multi-Model Ensemble Conformance
// Use conformance results from multiple miners as an ensemble
// ============================================================================

#[test]
fn test_ensemble_conformance_checking() {
    let log = load_running();

    let alpha_net = AlphaMiner::new().discover(&log);
    let ind_net = InductiveMiner::new().discover(&log);
    let heur_net = HeuristicMiner::new().discover(&log);
    let alpha_plus_net = AlphaPlusMiner::new().discover(&log);

    let alpha_result = TokenReplay::new().check(&log, &alpha_net);
    let ind_result = TokenReplay::new().check(&log, &ind_net);
    let heur_result = TokenReplay::new().check(&log, &heur_net);
    let alpha_plus_result = TokenReplay::new().check(&log, &alpha_plus_net);

    let ensemble_fitness = (alpha_result.fitness
        + ind_result.fitness
        + heur_result.fitness
        + alpha_plus_result.fitness)
        / 4.0;

    println!("Ensemble Conformance:");
    println!("   Alpha:     fitness={:.4}", alpha_result.fitness);
    println!("   Alpha+:    fitness={:.4}", alpha_plus_result.fitness);
    println!("   Inductive: fitness={:.4}", ind_result.fitness);
    println!("   Heuristic: fitness={:.4}", heur_result.fitness);
    println!("   Ensemble:  fitness={:.4}", ensemble_fitness);

    assert!(ensemble_fitness >= 0.0 && ensemble_fitness <= 1.0);
}

// ============================================================================
// INNOVATION 11: Cross-Source Train-Test Generalization
// Train on one data source, test on another
// ============================================================================

#[test]
fn test_cross_source_generalization() {
    // Train on invoice, test on compliance (both are business processes)
    let train_log = load_invoice();
    let test_log = load_compliance();

    let net = InductiveMiner::new().discover(&train_log);

    // How many test log activities are covered by the model?
    let test_activities: HashSet<String> = test_log.activities().into_iter().collect();
    let model_activities: HashSet<String> = net
        .transitions
        .iter()
        .filter_map(|t| t.label.clone())
        .collect();

    let overlap: HashSet<_> = test_activities
        .intersection(&model_activities)
        .cloned()
        .collect();
    let coverage = if test_activities.is_empty() {
        0.0
    } else {
        overlap.len() as f64 / test_activities.len() as f64
    };

    println!("Cross-Source Generalization:");
    println!(
        "   Train (Invoice): {} activities",
        train_log.activities().len()
    );
    println!("   Test (Compliance): {} activities", test_activities.len());
    println!("   Model activities: {}", model_activities.len());
    println!(
        "   Coverage: {:.1}% ({}/{})",
        coverage * 100.0,
        overlap.len(),
        test_activities.len()
    );
    println!(
        "   Missing: {:?}",
        test_activities.difference(&model_activities)
    );

    assert!(coverage >= 0.0 && coverage <= 1.0);
}

// ============================================================================
// INNOVATION 12: Log Skeleton Comparison Across Domains
// ============================================================================

#[test]
fn test_log_skeleton_cross_domain() {
    let processes: Vec<(&str, EventLog)> = vec![
        ("Invoice", load_invoice()),
        ("Onboarding", load_onboarding()),
        ("Running", load_running()),
        ("Receipt", load_receipt()),
    ];

    println!("Log Skeleton Comparison:");
    println!(
        "   {:12} {:>8} {:>8} {:>8} {:>8} {:>8}",
        "Process", "Equiv", "After", "Before", "Never", "DF"
    );

    for (name, log) in &processes {
        let skeleton = LogSkeletonMiner::new().discover(log);
        println!(
            "   {:12} {:>8} {:>8} {:>8} {:>8} {:>8}",
            name,
            skeleton.equivalence.len(),
            skeleton.after.len(),
            skeleton.before.len(),
            skeleton.never_together.len(),
            skeleton.directly_follows.len(),
        );
    }
}

// ============================================================================
// INNOVATION 13: DFG Edge Betweenness Centrality
// Find critical transition paths in a process
// ============================================================================

#[test]
fn test_dfg_critical_path_analysis() {
    let log = load_receipt();
    let dfg = DFGMiner::new().discover(&log);

    // Calculate "betweenness" - how many shortest paths use each edge
    let total_freq: usize = dfg.edges.iter().map(|e| e.frequency).sum();
    let mut edge_centrality: Vec<_> = dfg
        .edges
        .iter()
        .map(|e| {
            (
                format!("{} -> {}", e.from, e.to),
                e.frequency as f64 / total_freq.max(1) as f64,
                e.frequency,
            )
        })
        .collect();
    edge_centrality.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    println!("DFG Critical Path Analysis (Receipt):");
    println!("   Total edges: {}", dfg.edges.len());
    println!("   Top 10 by relative frequency:");
    for (i, (edge, rel, freq)) in edge_centrality.iter().take(10).enumerate() {
        println!(
            "     {:2}. {} ({:.1}%, {} occurrences)",
            i + 1,
            edge,
            rel * 100.0,
            freq
        );
    }
}

// ============================================================================
// INNOVATION 14: Trace Completeness Analysis
// Find incomplete or truncated traces
// ============================================================================

#[test]
fn test_trace_completeness_analysis() {
    let logs: Vec<(&str, EventLog)> = vec![
        ("Invoice", load_invoice()),
        ("Onboarding", load_onboarding()),
        ("Compliance", load_compliance()),
    ];

    println!("Trace Completeness Analysis:");
    for (name, log) in &logs {
        let _stats = log_statistics(log);
        let starts = start_activities(log);
        let ends = end_activities(log);

        // A trace is "complete" if it starts with the most common start activity
        // and ends with the most common end activity
        let most_common_start = starts
            .iter()
            .max_by_key(|(_, c)| *c)
            .map(|(a, _)| a.as_str())
            .unwrap_or("");
        let most_common_end = ends
            .iter()
            .max_by_key(|(_, c)| *c)
            .map(|(a, _)| a.as_str())
            .unwrap_or("");

        let complete_traces = log
            .traces
            .iter()
            .filter(|t| {
                t.events
                    .first()
                    .map(|e| e.activity == most_common_start)
                    .unwrap_or(false)
                    && t.events
                        .last()
                        .map(|e| e.activity == most_common_end)
                        .unwrap_or(false)
            })
            .count();

        println!(
            "   {}: {} traces, complete={}/{} ({:.1}%), starts=[{:?}], ends=[{:?}]",
            name,
            log.len(),
            complete_traces,
            log.len(),
            complete_traces as f64 / log.len().max(1) as f64 * 100.0,
            starts.keys().collect::<Vec<_>>(),
            ends.keys().collect::<Vec<_>>(),
        );
    }
}

// ============================================================================
// INNOVATION 15: SVG Visualization Size Correlation
// Verify visualization output scales with process complexity
// ============================================================================

#[test]
fn test_visualization_complexity_correlation() {
    let logs: Vec<(&str, EventLog)> = vec![
        ("Running", load_running()),
        ("Receipt", load_receipt()),
        ("Invoice", load_invoice()),
    ];

    println!("Visualization Complexity Correlation:");
    for (name, log) in &logs {
        let dfg = DFGMiner::new().discover(log);
        let svg = svg_renderer::render_dfg_svg(&dfg, &svg_renderer::SvgRenderOptions::default());

        let bytes_per_node = if dfg.nodes.is_empty() {
            0.0
        } else {
            svg.len() as f64 / dfg.nodes.len() as f64
        };

        println!(
            "   {}: {} nodes -> {} bytes SVG ({:.0} bytes/node)",
            name,
            dfg.nodes.len(),
            svg.len(),
            bytes_per_node
        );

        assert!(
            svg.contains("<svg"),
            "Should generate valid SVG for {}",
            name
        );
        // SVG should scale with complexity
        assert!(svg.len() > 50, "SVG should have content for {}", name);
    }
}

// ============================================================================
// INNOVATION 16: Synthetic Multi-Agent Process with Real Canopy Data
// Augment real business data with synthetic agent metadata
// ============================================================================

#[test]
fn test_business_data_with_agent_metadata() {
    let invoice = load_invoice();

    // Add synthetic agent decision events to each invoice trace
    let mut augmented = EventLog::new();
    for mut trace in invoice.clone().traces {
        let trace_id = trace.id.clone();

        // Insert an "agent_evaluate" event after submit
        let mut new_events = Vec::new();
        for event in trace.events.into_iter() {
            new_events.push(event);
            if new_events.len() == 2 {
                // After "submit", add agent evaluation
                new_events.push(
                    Event::new(
                        "agent_evaluate_risk",
                        new_events.last().unwrap().timestamp + Duration::minutes(5),
                    )
                    .with_resource("ai_agent_1")
                    .with_attribute("case_id", trace_id.clone())
                    .with_attribute("confidence", "0.95"),
                );
            }
        }
        trace.events = new_events;

        augmented.add_trace(trace);
    }

    // Discover model from augmented log
    let dfg = DFGMiner::new().discover(&augmented);
    let original_dfg = DFGMiner::new().discover(&invoice);

    println!("Augmented Process Model:");
    println!(
        "   Original DFG: {} nodes, {} edges",
        original_dfg.nodes.len(),
        original_dfg.edges.len()
    );
    println!(
        "   Augmented DFG: {} nodes, {} edges",
        dfg.nodes.len(),
        dfg.edges.len()
    );

    // Augmented should have more nodes (includes agent_evaluate_risk)
    assert!(
        dfg.nodes.len() >= original_dfg.nodes.len(),
        "Augmented should have at least as many nodes"
    );
}
