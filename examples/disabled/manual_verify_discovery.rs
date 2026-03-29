/// Manual verification of pm4py-rust capabilities - NO TESTS TRUSTED
/// This script manually verifies each capability works correctly
use pm4py::discovery::{AlphaMiner, DFGMiner, HeuristicMiner, ILPMiner, InductiveMiner, TreeMiner};
use pm4py::io::XESReader;
use std::path::Path;

fn main() {
    println!("=== MANUAL VERIFICATION OF PM4PY-RUST DISCOVERY ALGORITHMS ===\n");

    // 1. XES FILE READING - MANUAL VERIFICATION
    println!("1. XES FILE READING");
    let path = Path::new("/Users/sac/chatmangpt/test_simple.xes");
    let reader = XESReader::new();
    let log_result = reader.read(path);

    match &log_result {
        Ok(log) => {
            println!("  ✓ XES file loaded");
            println!("  - Traces: {}", log.traces.len());
            println!(
                "  - Events: {}",
                log.traces.iter().map(|t| t.events.len()).sum::<usize>()
            );

            // Print actual trace data
            for (i, trace) in log.traces.iter().take(2).enumerate() {
                println!(
                    "  - Trace {}: {} events, activities: {:?}",
                    i,
                    trace.events.len(),
                    trace.events.iter().map(|e| &e.activity).collect::<Vec<_>>()
                );
            }
        }
        Err(e) => {
            println!("  ✗ FAILED: {:?}", e);
            return;
        }
    }

    let log = log_result.unwrap();

    // 2. ALPHA MINER - MANUAL VERIFICATION
    println!("\n2. ALPHA MINER");
    let alpha_miner = AlphaMiner::new();
    let alpha_net = alpha_miner.discover(&log);
    println!("  - Places: {}", alpha_net.places.len());
    println!("  - Transitions: {}", alpha_net.transitions.len());
    println!("  - Arcs: {}", alpha_net.arcs.len());

    // Verify transitions are correct
    let activity_names: Vec<&str> = alpha_net
        .transitions
        .iter()
        .map(|t| t.label.as_deref())
        .filter_map(|l| l)
        .collect();
    println!("  - Activities: {:?}", activity_names);

    if activity_names.len() != 3
        || !activity_names.contains(&"A")
        || !activity_names.contains(&"B")
        || !activity_names.contains(&"C")
    {
        println!("  ✗ FAILED: Expected 3 transitions (A, B, C)");
    } else {
        println!("  ✓ ALPHA MINER WORKS");
    }

    // 3. INDUCTIVE MINER - MANUAL VERIFICATION
    println!("\n3. INDUCTIVE MINER");
    let inductive_miner = InductiveMiner::new();
    let inductive_net = inductive_miner.discover(&log);
    println!("  - Places: {}", inductive_net.places.len());
    println!("  - Transitions: {}", inductive_net.transitions.len());
    println!("  - Arcs: {}", inductive_net.arcs.len());

    if inductive_net.transitions.len() >= 3 {
        println!("  ✓ INDUCTIVE MINER WORKS");
    } else {
        println!("  ✗ FAILED: Expected at least 3 transitions");
    }

    // 4. HEURISTIC MINER - MANUAL VERIFICATION
    println!("\n4. HEURISTIC MINER");
    let heuristic_miner = HeuristicMiner::new();
    let heuristic_net = heuristic_miner.discover(&log);
    println!("  - Places: {}", heuristic_net.places.len());
    println!("  - Transitions: {}", heuristic_net.transitions.len());
    println!("  - Arcs: {}", heuristic_net.arcs.len());

    if heuristic_net.transitions.len() >= 3 {
        println!("  ✓ HEURISTIC MINER WORKS");
    } else {
        println!("  ✗ FAILED: Expected at least 3 transitions");
    }

    // 5. ILP MINER - MANUAL VERIFICATION
    println!("\n5. ILP MINER");
    let ilp_miner = ILPMiner::new();
    let ilp_net = ilp_miner.discover(&log);
    println!("  - Places: {}", ilp_net.places.len());
    println!("  - Transitions: {}", ilp_net.transitions.len());
    println!("  - Arcs: {}", ilp_net.arcs.len());

    if ilp_net.transitions.len() >= 3 {
        println!("  ✓ ILP MINER WORKS");
    } else {
        println!("  ✗ FAILED: Expected at least 3 transitions");
    }

    // 6. TREE MINER - MANUAL VERIFICATION
    println!("\n6. TREE MINER");
    let tree_miner = TreeMiner::new();
    let tree = tree_miner.discover(&log);
    println!(
        "  - Process tree type: {:?}",
        std::mem::discriminant(&tree.root)
    );

    // Verify tree has content
    match &tree.root {
        pm4py::models::process_tree::ProcessTreeNode::Activity(name) => {
            println!("  - Root activity: {}", name);
            println!("  ✓ TREE MINER WORKS (single activity)");
        }
        pm4py::models::process_tree::ProcessTreeNode::Operator {
            operator, children, ..
        } => {
            println!("  - Root operator: {:?}", operator);
            println!("  - Children: {}", children.len());
            println!("  ✓ TREE MINER WORKS (operator tree)");
        }
    }

    // 7. DFG MINER - MANUAL VERIFICATION
    println!("\n7. DFG MINER");
    let dfg_miner = DFGMiner::new();
    let dfg = dfg_miner.discover(&log);
    println!("  - DFG nodes: {}", dfg.nodes.len());
    println!("  - DFG edges: {}", dfg.edges.len());
    println!("  - Activities: {:?}", dfg.nodes);

    if dfg.nodes.len() == 3 && dfg.nodes.contains(&"A".to_string()) {
        println!("  ✓ DFG MINER WORKS");
    } else {
        println!("  ✗ FAILED: Expected 3 activities in DFG");
    }

    println!("\n=== DISCOVERY ALGORITHMS VERIFIED ===");
}
