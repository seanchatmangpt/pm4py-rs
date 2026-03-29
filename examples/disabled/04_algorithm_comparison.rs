//! Example 4: Compare All Discovery Algorithms
//!
//! Shows the output and characteristics of different discovery algorithms
//! on the same event log.
//!
//! Run with: cargo run --example 04_algorithm_comparison

use chrono::{Duration, Utc};
use pm4py::discovery::{AlphaMiner, HeuristicMiner, InductiveMiner, TreeMiner};
use pm4py::log::{Event, EventLog, Trace};
use std::time::Instant;

fn main() {
    println!("╔════════════════════════════════════════════════════════╗");
    println!("║        Example 4: Algorithm Comparison                 ║");
    println!("╚════════════════════════════════════════════════════════╝\n");

    let log = create_order_process_log();

    println!("Event Log: Order Processing");
    let stats = log.statistics();
    println!("  Traces: {}", stats.num_traces);
    println!("  Events: {}", stats.num_events);
    println!("  Activities: {}", stats.num_activities);
    println!("  Variants: {}\n", stats.num_variants);

    // 1. Alpha Miner
    println!("1. ALPHA MINER");
    println!("  Description: Foundational algorithm, very fast");
    let start = Instant::now();
    let alpha_model = AlphaMiner::new().discover(&log);
    let alpha_time = start.elapsed();
    println!("  Transitions: {}", alpha_model.transitions.len());
    println!("  Places: {}", alpha_model.places.len());
    println!("  Time: {:?}", alpha_time);
    println!("  Best for: Clean, simple processes");
    println!();

    // 2. Heuristic Miner
    println!("2. HEURISTIC MINER");
    println!("  Description: Frequency-based, noise-tolerant");
    let start = Instant::now();
    let heuristic_model = HeuristicMiner::new().discover(&log);
    let heuristic_time = start.elapsed();
    println!("  Transitions: {}", heuristic_model.transitions.len());
    println!("  Places: {}", heuristic_model.places.len());
    println!("  Time: {:?}", heuristic_time);
    println!("  Best for: Noisy, real-world processes");
    println!();

    // 3. Inductive Miner
    println!("3. INDUCTIVE MINER");
    println!("  Description: Recursive decomposition, hierarchical");
    let start = Instant::now();
    let inductive_model = InductiveMiner::new().discover(&log);
    let inductive_time = start.elapsed();
    println!("  Tree depth: {}", inductive_model.depth());
    println!("  Leaf count: {}", inductive_model.leaf_count());
    println!("  Time: {:?}", inductive_time);
    println!("  Best for: Complex processes with loops");
    println!();

    // 4. Tree Miner
    println!("4. TREE MINER");
    println!("  Description: Decision tree-based discovery");
    let start = Instant::now();
    let tree_model = TreeMiner::new().discover(&log);
    let tree_time = start.elapsed();
    println!("  Tree depth: {}", tree_model.depth);
    println!("  Leaf count: {}", tree_model.leaf_count);
    println!("  Time: {:?}", tree_time);
    println!("  Best for: Understanding decision logic");
    println!();

    // Summary
    println!("╔════════════════════════════════════════════════════════╗");
    println!("║                       SUMMARY                          ║");
    println!("╚════════════════════════════════════════════════════════╝\n");

    println!("Speed Ranking:");
    let mut times = vec![
        ("Alpha", alpha_time),
        ("Heuristic", heuristic_time),
        ("Inductive", inductive_time),
        ("Tree", tree_time),
    ];
    times.sort_by_key(|t| t.1);

    for (i, (name, time)) in times.iter().enumerate() {
        println!("  {}. {} - {:?}", i + 1, name, time);
    }

    println!("\n⚡ Alpha is fastest");
    println!("✓ Heuristic handles noise well");
    println!("🔄 Inductive handles loops");
    println!("🎯 Tree is most interpretable");
}

/// Create an order processing event log
fn create_order_process_log() -> EventLog {
    let mut log = EventLog::new();
    let base = Utc::now();

    // Normal flow: Order → Check Stock → Payment → Ship
    for i in 0..4 {
        let mut trace = Trace::new(format!("case_{}", i));
        let offset = base + Duration::days(i as i64);

        trace.add_event(Event::new("Order".to_string(), offset));
        trace.add_event(Event::new(
            "CheckStock".to_string(),
            offset + Duration::hours(1),
        ));
        trace.add_event(Event::new(
            "Payment".to_string(),
            offset + Duration::hours(2),
        ));
        trace.add_event(Event::new("Ship".to_string(), offset + Duration::hours(3)));

        log.add_trace(trace);
    }

    // With payment retry
    for i in 4..6 {
        let mut trace = Trace::new(format!("case_{}", i));
        let mut offset = base + Duration::days(i as i64);

        trace.add_event(Event::new("Order".to_string(), offset));
        trace.add_event(Event::new(
            "CheckStock".to_string(),
            offset + Duration::hours(1),
        ));
        offset = offset + Duration::hours(2);
        trace.add_event(Event::new("Payment".to_string(), offset));
        offset = offset + Duration::hours(1);
        trace.add_event(Event::new("PaymentFailed".to_string(), offset));
        offset = offset + Duration::hours(1);
        trace.add_event(Event::new("Payment".to_string(), offset)); // Retry
        offset = offset + Duration::hours(1);
        trace.add_event(Event::new("Ship".to_string(), offset));

        log.add_trace(trace);
    }

    // With out of stock
    for i in 6..8 {
        let mut trace = Trace::new(format!("case_{}", i));
        let offset = base + Duration::days(i as i64);

        trace.add_event(Event::new("Order".to_string(), offset));
        trace.add_event(Event::new(
            "CheckStock".to_string(),
            offset + Duration::hours(1),
        ));
        trace.add_event(Event::new(
            "OutOfStock".to_string(),
            offset + Duration::hours(2),
        ));
        trace.add_event(Event::new(
            "Cancel".to_string(),
            offset + Duration::hours(3),
        ));

        log.add_trace(trace);
    }

    log
}
