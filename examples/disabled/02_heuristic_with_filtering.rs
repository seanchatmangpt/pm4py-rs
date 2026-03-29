//! Example 2: Heuristic Miner with Filtering
//!
//! Shows how to use the Heuristic Miner with parameter tuning
//! to handle noisy logs with infrequent variants.
//!
//! Run with: cargo run --example 02_heuristic_with_filtering

use chrono::{Duration, Utc};
use pm4py::discovery::HeuristicMiner;
use pm4py::log::{Event, EventLog, Trace};

fn main() {
    println!("╔════════════════════════════════════════════════════════╗");
    println!("║   Example 2: Heuristic Miner with Threshold           ║");
    println!("╚════════════════════════════════════════════════════════╝\n");

    // Create a log with noise
    let log = create_noisy_event_log();

    println!("Original Log Statistics:");
    let stats = log.statistics();
    println!("  Traces: {}", stats.num_traces);
    println!("  Events: {}", stats.num_events);
    println!("  Activities: {}", stats.num_activities);
    println!("  Variants: {}\n", stats.num_variants);

    // Try different thresholds
    let thresholds = vec![0.1, 0.5, 0.9];

    for threshold in thresholds {
        println!("Heuristic Miner with threshold = {}", threshold);

        let miner = HeuristicMiner::with_threshold(threshold);
        let model = miner.discover(&log);

        println!("  Transitions: {}", model.transitions.len());
        println!("  Places: {}", model.places.len());

        match threshold {
            t if t < 0.3 => println!("  → Very permissive (includes rare variants)"),
            t if t < 0.7 => println!("  → Balanced (normal behavior)"),
            _ => println!("  → Very strict (only frequent patterns)"),
        }
        println!();
    }

    println!("✓ Comparison complete!");
    println!("\nKey insight:");
    println!("  Higher threshold → smaller model (less noise)");
    println!("  Lower threshold → larger model (includes variants)");
}

/// Create a log with noise and infrequent variants
fn create_noisy_event_log() -> EventLog {
    let mut log = EventLog::new();
    let base_time = Utc::now();

    // Main process: A → B → C → D (80%)
    for case_id in 0..8 {
        let mut trace = Trace::new(format!("case_{}", case_id));
        let offset = base_time + Duration::days(case_id as i64);

        trace.add_event(Event::new("A".to_string(), offset));
        trace.add_event(Event::new("B".to_string(), offset + Duration::hours(1)));
        trace.add_event(Event::new("C".to_string(), offset + Duration::hours(2)));
        trace.add_event(Event::new("D".to_string(), offset + Duration::hours(3)));

        log.add_trace(trace);
    }

    // Variant 1: Skip B (10%)
    for case_id in 8..9 {
        let mut trace = Trace::new(format!("case_{}", case_id));
        let offset = base_time + Duration::days(case_id as i64);

        trace.add_event(Event::new("A".to_string(), offset));
        trace.add_event(Event::new("C".to_string(), offset + Duration::hours(1)));
        trace.add_event(Event::new("D".to_string(), offset + Duration::hours(2)));

        log.add_trace(trace);
    }

    // Variant 2: Extra rework E (5%)
    for case_id in 9..10 {
        let mut trace = Trace::new(format!("case_{}", case_id));
        let offset = base_time + Duration::days(case_id as i64);

        trace.add_event(Event::new("A".to_string(), offset));
        trace.add_event(Event::new("B".to_string(), offset + Duration::hours(1)));
        trace.add_event(Event::new("E".to_string(), offset + Duration::hours(2))); // Rework
        trace.add_event(Event::new("B".to_string(), offset + Duration::hours(3))); // Redo
        trace.add_event(Event::new("C".to_string(), offset + Duration::hours(4)));
        trace.add_event(Event::new("D".to_string(), offset + Duration::hours(5)));

        log.add_trace(trace);
    }

    // Variant 3: Wrong order F → A (5%)
    for case_id in 10..11 {
        let mut trace = Trace::new(format!("case_{}", case_id));
        let offset = base_time + Duration::days(case_id as i64);

        trace.add_event(Event::new("F".to_string(), offset)); // Rare pre-activity
        trace.add_event(Event::new("A".to_string(), offset + Duration::hours(1)));
        trace.add_event(Event::new("B".to_string(), offset + Duration::hours(2)));
        trace.add_event(Event::new("C".to_string(), offset + Duration::hours(3)));
        trace.add_event(Event::new("D".to_string(), offset + Duration::hours(4)));

        log.add_trace(trace);
    }

    log
}
