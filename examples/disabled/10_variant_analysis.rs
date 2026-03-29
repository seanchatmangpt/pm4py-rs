//! Example 10: Process Variant Analysis
//!
//! Analyzes process variants and identifies the most common execution patterns
//!
//! Run with: cargo run --example 10_variant_analysis

use chrono::{Duration, Utc};
use pm4py::log::{Event, EventLog, Trace};
use std::collections::HashMap;

fn main() {
    println!("╔════════════════════════════════════════════════════════╗");
    println!("║         Example 10: Variant Analysis                   ║");
    println!("╚════════════════════════════════════════════════════════╝\n");

    let log = create_variant_log();

    println!("Log Overview:");
    println!("  Total traces: {}", log.traces().len());
    println!("  Total events: {}", log.num_events());
    println!("  Process variants: {}\n", log.statistics().num_variants);

    // Extract and analyze variants
    let mut variants = HashMap::new();

    for trace in log.traces() {
        let sequence: Vec<String> = trace.events.iter().map(|e| e.name.clone()).collect();
        *variants.entry(sequence).or_insert(0) += 1;
    }

    // Sort by frequency
    let mut variant_list: Vec<_> = variants.into_iter().collect();
    variant_list.sort_by_key(|v| std::cmp::Reverse(v.1));

    println!("╔════════════════════════════════════════════════════════╗");
    println!("║               PROCESS VARIANTS                         ║");
    println!("╚════════════════════════════════════════════════════════╝\n");

    for (idx, (sequence, count)) in variant_list.iter().enumerate() {
        let percentage = (*count as f64 / log.traces().len() as f64) * 100.0;

        println!("Variant {}:", idx + 1);
        println!("  Frequency: {} traces ({:.1}%)", count, percentage);
        println!("  Sequence: {}", sequence.join(" → "));

        // Characterize variant
        if sequence.contains(&"Reject".to_string()) {
            println!("  Type: ⊗ Rejection path");
        } else if sequence.contains(&"Rework".to_string()) {
            println!("  Type: ↻ Rework/Loop path");
        } else {
            println!("  Type: ✓ Happy path");
        }
        println!();
    }

    // Statistics
    println!("╔════════════════════════════════════════════════════════╗");
    println!("║             VARIANT STATISTICS                         ║");
    println!("╚════════════════════════════════════════════════════════╝\n");

    let most_common = variant_list.first().map(|v| v.1).unwrap_or(&0);
    let total = log.traces().len();
    println!(
        "Top variant frequency: {}/{} ({:.1}%)",
        most_common,
        total,
        (*most_common as f64 / total as f64) * 100.0
    );

    let top_3_coverage: usize = variant_list.iter().take(3).map(|v| v.1).sum();
    println!(
        "Top 3 variants coverage: {}/{} ({:.1}%)",
        top_3_coverage,
        total,
        (top_3_coverage as f64 / total as f64) * 100.0
    );

    let variant_count = variant_list.len();
    println!("Total unique variants: {}", variant_count);
    println!(
        "Variant diversity: {:.2} (more = more diverse)",
        variant_count as f64 / total as f64
    );
}

fn create_variant_log() -> EventLog {
    let mut log = EventLog::new();
    let base = Utc::now();

    // Variant 1: Happy path (50%)
    for i in 0..5 {
        let mut trace = Trace::new(format!("happy_{}", i));
        let offset = base + Duration::days(i as i64);

        trace.add_event(Event::new("Register".to_string(), offset));
        trace.add_event(Event::new(
            "Validate".to_string(),
            offset + Duration::hours(1),
        ));
        trace.add_event(Event::new(
            "Approve".to_string(),
            offset + Duration::hours(2),
        ));
        trace.add_event(Event::new(
            "Activate".to_string(),
            offset + Duration::hours(3),
        ));

        log.add_trace(trace);
    }

    // Variant 2: Rework path (30%)
    for i in 5..8 {
        let mut trace = Trace::new(format!("rework_{}", i));
        let offset = base + Duration::days(i as i64);

        trace.add_event(Event::new("Register".to_string(), offset));
        trace.add_event(Event::new(
            "Validate".to_string(),
            offset + Duration::hours(1),
        ));
        trace.add_event(Event::new(
            "Rework".to_string(),
            offset + Duration::hours(2),
        ));
        trace.add_event(Event::new(
            "Validate".to_string(),
            offset + Duration::hours(3),
        ));
        trace.add_event(Event::new(
            "Approve".to_string(),
            offset + Duration::hours(4),
        ));
        trace.add_event(Event::new(
            "Activate".to_string(),
            offset + Duration::hours(5),
        ));

        log.add_trace(trace);
    }

    // Variant 3: Rejection (15%)
    for i in 8..9 {
        let mut trace = Trace::new(format!("reject_{}", i));
        let offset = base + Duration::days(i as i64);

        trace.add_event(Event::new("Register".to_string(), offset));
        trace.add_event(Event::new(
            "Validate".to_string(),
            offset + Duration::hours(1),
        ));
        trace.add_event(Event::new(
            "Reject".to_string(),
            offset + Duration::hours(2),
        ));

        log.add_trace(trace);
    }

    // Variant 4: Double rework (5%)
    for i in 9..10 {
        let mut trace = Trace::new(format!("double_rework_{}", i));
        let offset = base + Duration::days(i as i64);

        trace.add_event(Event::new("Register".to_string(), offset));
        trace.add_event(Event::new(
            "Validate".to_string(),
            offset + Duration::hours(1),
        ));
        trace.add_event(Event::new(
            "Rework".to_string(),
            offset + Duration::hours(2),
        ));
        trace.add_event(Event::new(
            "Validate".to_string(),
            offset + Duration::hours(3),
        ));
        trace.add_event(Event::new(
            "Rework".to_string(),
            offset + Duration::hours(4),
        ));
        trace.add_event(Event::new(
            "Validate".to_string(),
            offset + Duration::hours(5),
        ));
        trace.add_event(Event::new(
            "Approve".to_string(),
            offset + Duration::hours(6),
        ));
        trace.add_event(Event::new(
            "Activate".to_string(),
            offset + Duration::hours(7),
        ));

        log.add_trace(trace);
    }

    log
}
