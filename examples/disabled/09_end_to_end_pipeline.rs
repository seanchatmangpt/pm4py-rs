//! Example 9: Complete End-to-End Pipeline
//!
//! Shows a realistic workflow: load log → filter → discover → check conformance
//!
//! Run with: cargo run --example 09_end_to_end_pipeline

use chrono::{Duration, Utc};
use pm4py::conformance::TokenReplay;
use pm4py::discovery::AlphaMiner;
use pm4py::log::{Event, EventLog, Trace};

fn main() {
    println!("╔════════════════════════════════════════════════════════╗");
    println!("║         Example 9: End-to-End Pipeline                 ║");
    println!("╚════════════════════════════════════════════════════════╝\n");

    let start = std::time::Instant::now();

    // Step 1: Load log
    println!("Step 1: Loading event log...");
    let mut log = load_event_log();
    println!(
        "  ✓ Loaded {} traces, {} events\n",
        log.traces().len(),
        log.num_events()
    );

    // Step 2: Clean and filter
    println!("Step 2: Cleaning and filtering...");
    let original_activities = log.statistics().num_activities;
    log = log.filter_activities_by_threshold(0.90);
    let filtered_activities = log.statistics().num_activities;
    println!(
        "  ✓ Reduced activities: {} → {}\n",
        original_activities, filtered_activities
    );

    // Step 3: Discover process model
    println!("Step 3: Discovering process model...");
    let miner = AlphaMiner::new();
    let model = miner.discover(&log);
    println!(
        "  ✓ Discovered {} transitions, {} places\n",
        model.transitions.len(),
        model.places.len()
    );

    // Step 4: Conformance check
    println!("Step 4: Checking conformance...");
    let checker = TokenReplay::new();
    let results = checker.replay(&log, &model);

    let avg_fitness: f64 = results.iter().map(|r| r.fitness).sum::<f64>() / results.len() as f64;

    let conforming = results.iter().filter(|r| r.fitness >= 1.0).count();
    let mostly_conforming = results
        .iter()
        .filter(|r| r.fitness >= 0.8 && r.fitness < 1.0)
        .count();
    let deviating = results.len() - conforming - mostly_conforming;

    println!("  ✓ Analyzed {} traces", results.len());
    println!(
        "    - Perfectly conforming: {} ({:.0}%)",
        conforming,
        (conforming as f64 / results.len() as f64) * 100.0
    );
    println!(
        "    - Mostly conforming: {} ({:.0}%)",
        mostly_conforming,
        (mostly_conforming as f64 / results.len() as f64) * 100.0
    );
    println!(
        "    - Deviating: {} ({:.0}%)",
        deviating,
        (deviating as f64 / results.len() as f64) * 100.0
    );
    println!("    - Average fitness: {:.2}\n", avg_fitness);

    // Summary
    let elapsed = start.elapsed();
    println!("╔════════════════════════════════════════════════════════╗");
    println!("║                    SUMMARY                             ║");
    println!("╚════════════════════════════════════════════════════════╝\n");

    println!("Pipeline completed in {:?}", elapsed);
    println!("\nProcessed:");
    println!("  • {} event log traces", log.traces().len());
    println!("  • {} total events", log.num_events());
    println!("  • {} process activities", filtered_activities);
    println!(
        "  • Discovered model with {} transitions",
        model.transitions.len()
    );
    println!("\nQuality Metrics:");
    println!("  • Overall fitness: {:.2}", avg_fitness);
    println!(
        "  • Conformance rate: {:.0}%",
        ((conforming as f64 / results.len() as f64) * 100.0)
    );

    if avg_fitness >= 0.95 {
        println!("\n✓ EXCELLENT: Model captures process accurately");
    } else if avg_fitness >= 0.80 {
        println!("\n✓ GOOD: Model fits most behavior");
    } else {
        println!("\n⚠ FAIR: Model may need improvement");
    }
}

fn load_event_log() -> EventLog {
    let mut log = EventLog::new();
    let base = Utc::now();

    // Normal cases (70%)
    for i in 0..7 {
        let mut trace = Trace::new(format!("norm_{}", i));
        let offset = base + Duration::days(i as i64);

        trace.add_event(Event::new("Receive".to_string(), offset));
        trace.add_event(Event::new(
            "Review".to_string(),
            offset + Duration::hours(2),
        ));
        trace.add_event(Event::new(
            "Approve".to_string(),
            offset + Duration::hours(4),
        ));
        trace.add_event(Event::new(
            "Execute".to_string(),
            offset + Duration::hours(6),
        ));
        trace.add_event(Event::new(
            "Archive".to_string(),
            offset + Duration::hours(8),
        ));

        log.add_trace(trace);
    }

    // With rework (20%)
    for i in 7..9 {
        let mut trace = Trace::new(format!("rework_{}", i));
        let offset = base + Duration::days(i as i64);

        trace.add_event(Event::new("Receive".to_string(), offset));
        trace.add_event(Event::new(
            "Review".to_string(),
            offset + Duration::hours(2),
        ));
        trace.add_event(Event::new(
            "Rework".to_string(),
            offset + Duration::hours(4),
        ));
        trace.add_event(Event::new(
            "Review".to_string(),
            offset + Duration::hours(6),
        ));
        trace.add_event(Event::new(
            "Approve".to_string(),
            offset + Duration::hours(8),
        ));
        trace.add_event(Event::new(
            "Execute".to_string(),
            offset + Duration::hours(10),
        ));
        trace.add_event(Event::new(
            "Archive".to_string(),
            offset + Duration::hours(12),
        ));

        log.add_trace(trace);
    }

    // Rejected (10%)
    for i in 9..10 {
        let mut trace = Trace::new(format!("reject_{}", i));
        let offset = base + Duration::days(i as i64);

        trace.add_event(Event::new("Receive".to_string(), offset));
        trace.add_event(Event::new(
            "Review".to_string(),
            offset + Duration::hours(2),
        ));
        trace.add_event(Event::new(
            "Reject".to_string(),
            offset + Duration::hours(4),
        ));
        trace.add_event(Event::new(
            "Archive".to_string(),
            offset + Duration::hours(6),
        ));

        log.add_trace(trace);
    }

    log
}
