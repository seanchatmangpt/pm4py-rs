//! Example 7: Event Log Filtering Techniques
//!
//! Demonstrates various filtering methods to clean and prepare logs
//! for discovery: activity filtering, trace filtering, sampling, etc.
//!
//! Run with: cargo run --example 07_filtering_techniques

use chrono::{Duration, Utc};
use pm4py::discovery::AlphaMiner;
use pm4py::log::{Event, EventLog, Trace};

fn main() {
    println!("╔════════════════════════════════════════════════════════╗");
    println!("║       Example 7: Log Filtering Techniques              ║");
    println!("╚════════════════════════════════════════════════════════╝\n");

    let original_log = create_noisy_log();

    println!("Original Log:");
    println!("  Traces: {}", original_log.traces().len());
    println!("  Events: {}", original_log.num_events());
    println!("  Activities: {}", original_log.statistics().num_activities);
    println!("  Variants: {}\n", original_log.statistics().num_variants);

    // Filter 1: Activity frequency threshold
    println!("╔ Filter 1: Activity Frequency Threshold ╗");
    let filtered_90 = original_log.filter_activities_by_threshold(0.9);
    println!(
        "Activities (90% threshold): {} → {}",
        original_log.statistics().num_activities,
        filtered_90.statistics().num_activities
    );

    let model = AlphaMiner::new().discover(&filtered_90);
    println!("Model size: {} transitions\n", model.transitions.len());

    // Filter 2: By specific activity
    println!("╔ Filter 2: Remove Specific Activity ╗");
    let without_inspect = filter_remove_activity(&original_log, "Inspect");
    println!(
        "Traces after removing 'Inspect': {}",
        without_inspect.traces().len()
    );
    println!(
        "Events after removing 'Inspect': {}\n",
        without_inspect.num_events()
    );

    // Filter 3: Trace length
    println!("╔ Filter 3: By Trace Length ╗");
    let len_3_to_5 = filter_by_trace_length(&original_log, 3, 5);
    println!("Traces with length 3-5: {}", len_3_to_5.traces().len());
    println!(
        "Total events in filtered log: {}\n",
        len_3_to_5.num_events()
    );

    // Filter 4: Sample
    println!("╔ Filter 4: Sampling ╗");
    let sample_5 = original_log.sample(5);
    println!(
        "Original: {} traces → Sample: {} traces",
        original_log.traces().len(),
        sample_5.traces().len()
    );
    println!(
        "Original: {} events → Sample: {} events\n",
        original_log.num_events(),
        sample_5.num_events()
    );

    // Combination: Multiple filters
    println!("╔ Combined Filters ╗");
    let mut combined = original_log.clone();
    combined = combined.filter_activities_by_threshold(0.85);
    println!(
        "After activity filter (85%): {} activities",
        combined.statistics().num_activities
    );

    combined = filter_by_trace_length(&combined, 3, 6);
    println!(
        "After length filter (3-6): {} traces",
        combined.traces().len()
    );

    combined = combined.sample(5);
    println!("After sampling (5): {} traces", combined.traces().len());

    let model = AlphaMiner::new().discover(&combined);
    println!("Final model: {} transitions", model.transitions.len());
}

fn filter_remove_activity(log: &EventLog, activity: &str) -> EventLog {
    let mut filtered = EventLog::new();

    for trace in log.traces() {
        let mut new_trace = Trace::new(trace.id.clone());

        for event in &trace.events {
            if event.name != activity {
                new_trace.add_event(event.clone());
            }
        }

        if !new_trace.events.is_empty() {
            filtered.add_trace(new_trace);
        }
    }

    filtered
}

fn filter_by_trace_length(log: &EventLog, min: usize, max: usize) -> EventLog {
    let mut filtered = EventLog::new();

    for trace in log.traces() {
        if trace.events.len() >= min && trace.events.len() <= max {
            filtered.add_trace(trace.clone());
        }
    }

    filtered
}

fn create_noisy_log() -> EventLog {
    let mut log = EventLog::new();
    let base = Utc::now();

    // Main variant (70%)
    for i in 0..7 {
        let mut trace = Trace::new(format!("case_main_{}", i));
        let offset = base + Duration::days(i as i64);

        trace.add_event(Event::new("Start".to_string(), offset));
        trace.add_event(Event::new(
            "Process".to_string(),
            offset + Duration::hours(1),
        ));
        trace.add_event(Event::new("End".to_string(), offset + Duration::hours(2)));

        log.add_trace(trace);
    }

    // With inspection (20%)
    for i in 7..9 {
        let mut trace = Trace::new(format!("case_inspect_{}", i));
        let offset = base + Duration::days(i as i64);

        trace.add_event(Event::new("Start".to_string(), offset));
        trace.add_event(Event::new(
            "Process".to_string(),
            offset + Duration::hours(1),
        ));
        trace.add_event(Event::new(
            "Inspect".to_string(),
            offset + Duration::hours(2),
        ));
        trace.add_event(Event::new("End".to_string(), offset + Duration::hours(3)));

        log.add_trace(trace);
    }

    // With rework (10%)
    for i in 9..10 {
        let mut trace = Trace::new(format!("case_rework_{}", i));
        let offset = base + Duration::days(i as i64);

        trace.add_event(Event::new("Start".to_string(), offset));
        trace.add_event(Event::new(
            "Process".to_string(),
            offset + Duration::hours(1),
        ));
        trace.add_event(Event::new(
            "Rework".to_string(),
            offset + Duration::hours(2),
        ));
        trace.add_event(Event::new(
            "Process".to_string(),
            offset + Duration::hours(3),
        ));
        trace.add_event(Event::new("End".to_string(), offset + Duration::hours(4)));

        log.add_trace(trace);
    }

    log
}
