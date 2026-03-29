//! Example 1: Simple Alpha Miner
//!
//! Demonstrates basic process discovery using the Alpha Miner algorithm.
//! This is the simplest and fastest discovery method.
//!
//! Run with: cargo run --example 01_simple_alpha_miner

use chrono::{Duration, Utc};
use pm4py::discovery::AlphaMiner;
use pm4py::log::{Event, EventLog, Trace};

fn main() {
    println!("╔════════════════════════════════════════════════════════╗");
    println!("║    Example 1: Simple Alpha Miner Discovery            ║");
    println!("╚════════════════════════════════════════════════════════╝\n");

    // Step 1: Create a simple event log
    let log = create_simple_event_log();

    println!("Log Statistics:");
    let stats = log.statistics();
    println!("  Traces: {}", stats.num_traces);
    println!("  Events: {}", stats.num_events);
    println!("  Activities: {}", stats.num_activities);
    println!("  Avg trace length: {:.1}\n", stats.avg_trace_length);

    // Step 2: Discover process model
    let miner = AlphaMiner::new();
    let petri_net = miner.discover(&log);

    println!("Discovered Petri Net:");
    println!("  Places: {}", petri_net.places.len());
    println!("  Transitions: {}", petri_net.transitions.len());
    println!("  Arcs: {}", petri_net.arcs.len());

    println!("\n✓ Discovery complete!");
}

/// Create a simple, well-structured event log
fn create_simple_event_log() -> EventLog {
    let mut log = EventLog::new();
    let base_time = Utc::now();

    // Define the process: Order → Payment → Ship → Deliver
    let activities = vec!["Order", "Payment", "Ship", "Deliver"];

    // Create 5 traces, all following the same sequence
    for case_id in 0..5 {
        let mut trace = Trace::new(format!("case_{}", case_id));
        let mut current_time = base_time + Duration::days(case_id as i64);

        for activity in &activities {
            trace.add_event(Event::new(activity.to_string(), current_time));
            current_time = current_time + Duration::hours(1);
        }

        log.add_trace(trace);
    }

    log
}
