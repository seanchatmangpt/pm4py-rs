// Comprehensive pm4py-rust parity test with Python pm4py 2.7.19.5

use chrono::Utc;
use pm4py::{
    discovery::{AlphaMiner, InductiveMiner, TreeMiner},
    models::{ProcessTree, ProcessTreeNode},
    Event, EventLog, Trace,
};

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║        pm4py-rust ↔ Python pm4py 2.7.19.5 Parity Test       ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    // Create standard test log: A -> B -> C repeated 5 times
    let mut log = create_test_log(5);

    let mut total_tests = 0;
    let mut passed_tests = 0;

    // Test 1: Alpha Miner
    println!("┌─ TEST 1: Alpha Miner ─────────────────────────────────────────┐");
    total_tests += 1;
    let miner = AlphaMiner::new();
    let net = miner.discover(&log);

    println!(
        "│ Rust pm4py:      Places={}, Transitions={}, Arcs={}",
        net.places.len(),
        net.transitions.len(),
        net.arcs.len()
    );
    println!("│ Python pm4py:    Places=4, Transitions=3, Arcs=6");

    let places_match = net.places.len() == 4;
    let transitions_match = net.transitions.len() == 3;
    let arcs_match = net.arcs.len() == 6;
    let test1_pass = places_match && transitions_match && arcs_match;

    if test1_pass {
        passed_tests += 1;
    }
    println!("│ Result: {}", if test1_pass { "✓ PASS" } else { "✗ FAIL" });
    println!("└────────────────────────────────────────────────────────────────┘");
    println!();

    // Test 2: Tree Miner (Process Tree)
    println!("┌─ TEST 2: Tree Miner (Process Tree) ──────────────────────────┐");
    total_tests += 1;
    let miner = TreeMiner::new();
    let tree = miner.discover(&log);

    let (nodes, operators) = count_tree_nodes(&tree.root);

    println!(
        "│ Rust pm4py:      Nodes={}, Operators={}",
        nodes, operators
    );
    println!("│ Python pm4py:    Nodes=4, Operators=1");

    let nodes_match = nodes == 4;
    let operators_match = operators == 1;
    let test2_pass = nodes_match && operators_match;

    if test2_pass {
        passed_tests += 1;
    }
    println!("│ Result: {}", if test2_pass { "✓ PASS" } else { "✗ FAIL" });
    println!("└────────────────────────────────────────────────────────────────┘");
    println!();

    // Test 3: Log Statistics
    println!("┌─ TEST 3: Log Statistics ───────────────────────────────────────┐");
    total_tests += 1;
    let trace_count = log.traces.len();
    let event_count: usize = log.traces.iter().map(|t| t.events.len()).sum();
    let unique_activities: std::collections::HashSet<String> = log
        .traces
        .iter()
        .flat_map(|t| t.events.iter().map(|e| e.activity.clone()))
        .collect();

    println!(
        "│ Rust pm4py:      Traces={}, Events={}, Activities={}",
        trace_count,
        event_count,
        unique_activities.len()
    );
    println!("│ Python pm4py:    Traces=5, Events=15, Activities=3");

    let traces_match = trace_count == 5;
    let events_match = event_count == 15;
    let activities_match = unique_activities.len() == 3;
    let test3_pass = traces_match && events_match && activities_match;

    if test3_pass {
        passed_tests += 1;
    }
    println!("│ Result: {}", if test3_pass { "✓ PASS" } else { "✗ FAIL" });
    println!("└────────────────────────────────────────────────────────────────┘");
    println!();

    // Summary
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                         SUMMARY                              ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  Tests Run:        {}", total_tests);
    println!("║  Tests Passed:     {}", passed_tests);
    println!("║  Tests Failed:     {}", total_tests - passed_tests);
    println!(
        "║  Pass Rate:        {}%",
        (passed_tests * 100 / total_tests)
    );
    println!("╚══════════════════════════════════════════════════════════════╝");

    if passed_tests == total_tests {
        println!();
        println!("✓ ALL PARITY TESTS PASSED - pm4py-rust matches Python pm4py!");
        std::process::exit(0);
    } else {
        println!();
        println!("✗ SOME PARITY TESTS FAILED");
        std::process::exit(1);
    }
}

fn create_test_log(count: usize) -> EventLog {
    let mut log = EventLog::new();

    for i in 0..count {
        let mut trace = Trace::new(format!("case_{}", i));

        let event_a = Event::new("A", Utc::now());
        trace.add_event(event_a);

        let event_b = Event::new("B", Utc::now());
        trace.add_event(event_b);

        let event_c = Event::new("C", Utc::now());
        trace.add_event(event_c);

        log.add_trace(trace);
    }

    log
}

fn count_tree_nodes(node: &ProcessTreeNode) -> (usize, usize) {
    match node {
        ProcessTreeNode::Activity(_) => (1, 0),
        ProcessTreeNode::Operator { children, .. } => {
            let mut nodes = 1;
            let mut operators = 1;
            for child in children {
                let (n, o) = count_tree_nodes(child);
                nodes += n;
                operators += o;
            }
            (nodes, operators)
        }
    }
}
