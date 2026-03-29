use chrono::Utc;
/// Statistical Analysis Example
///
/// This example demonstrates basic statistics on an event log.
///
/// Run with: cargo run --example analysis
use pm4py::log::{Event, EventLog, Trace};

fn main() {
    println!("═══════════════════════════════════════════════════════");
    println!("     Statistical Analysis Example");
    println!("═══════════════════════════════════════════════════════\n");

    let log = create_sample_log();

    println!("Basic Log Statistics:");
    println!("  Total traces: {}", log.traces.len());
    println!(
        "  Total events: {}",
        log.traces.iter().map(|t| t.events.len()).sum::<usize>()
    );

    // Analyze trace lengths
    let lengths: Vec<usize> = log.traces.iter().map(|t| t.events.len()).collect();
    if !lengths.is_empty() {
        let avg: f64 = lengths.iter().sum::<usize>() as f64 / lengths.len() as f64;
        let min = *lengths.iter().min().unwrap_or(&0);
        let max = *lengths.iter().max().unwrap_or(&0);

        println!("  Avg trace length: {:.1}", avg);
        println!("  Min trace length: {}", min);
        println!("  Max trace length: {}", max);
    }

    // Activity analysis
    let mut activities: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for trace in &log.traces {
        for event in &trace.events {
            *activities.entry(event.activity.clone()).or_insert(0) += 1;
        }
    }

    println!("\nActivity Frequency:");
    let mut sorted: Vec<_> = activities.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));

    for (activity, count) in sorted.iter().take(5) {
        println!("  {} → {} occurrences", activity, count);
    }

    println!("\n✓ Analysis complete");
}

fn create_sample_log() -> EventLog {
    let mut log = EventLog::new();
    let base = Utc::now();

    for i in 0..10 {
        let mut trace = Trace::new(format!("case_{}", i));
        let offset = base + chrono::Duration::hours(i as i64);

        trace.add_event(Event::new("Start", offset));
        trace.add_event(Event::new(
            "Process A",
            offset + chrono::Duration::minutes(10),
        ));
        trace.add_event(Event::new(
            "Process B",
            offset + chrono::Duration::minutes(20),
        ));
        trace.add_event(Event::new(
            "Process C",
            offset + chrono::Duration::minutes(30),
        ));
        trace.add_event(Event::new("End", offset + chrono::Duration::minutes(40)));

        log.add_trace(trace);
    }

    log
}
