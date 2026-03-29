//! Example 6: Statistical Analysis of Event Logs
//!
//! Demonstrates how to extract useful statistics from event logs
//! including activity metrics, temporal analysis, and variant detection.
//!
//! Run with: cargo run --example 06_statistics_analysis

use chrono::{Duration, Utc};
use pm4py::log::{Event, EventLog, Trace};
use std::collections::HashMap;

fn main() {
    println!("╔════════════════════════════════════════════════════════╗");
    println!("║        Example 6: Statistical Analysis                 ║");
    println!("╚════════════════════════════════════════════════════════╝\n");

    let log = create_statistical_log();

    // Basic statistics
    println!("Basic Log Statistics:");
    let stats = log.statistics();
    println!("  Total traces: {}", stats.num_traces);
    println!("  Total events: {}", stats.num_events);
    println!("  Unique activities: {}", stats.num_activities);
    println!("  Process variants: {}", stats.num_variants);
    println!("  Avg trace length: {:.2}", stats.avg_trace_length);
    println!();

    // Activity frequency
    println!("Activity Frequency:");
    let activity_counts = count_activities(&log);
    let mut activity_vec: Vec<_> = activity_counts.iter().collect();
    activity_vec.sort_by_key(|a| std::cmp::Reverse(a.1));

    for (activity, count) in activity_vec.iter().take(5) {
        let percentage = (*count as f64 / stats.num_events as f64) * 100.0;
        println!("  {}: {} ({:.1}%)", activity, count, percentage);
    }
    println!();

    // Trace length distribution
    println!("Trace Length Distribution:");
    let length_dist = trace_length_distribution(&log);
    let mut lengths: Vec<_> = length_dist.iter().collect();
    lengths.sort_by_key(|l| l.0);

    for (length, count) in lengths {
        println!("  Length {}: {} traces", length, count);
    }
    println!();

    // Temporal analysis
    println!("Temporal Analysis:");
    let time_stats = analyze_temporal(&log);
    println!("  Span: {} days", time_stats.span_days);
    println!(
        "  Avg case duration: {:.2} hours",
        time_stats.avg_duration_hours
    );
    println!(
        "  Min case duration: {:.2} hours",
        time_stats.min_duration_hours
    );
    println!(
        "  Max case duration: {:.2} hours",
        time_stats.max_duration_hours
    );
    println!();

    // Most common variants
    println!("Top Process Variants:");
    let variants = extract_variants(&log);
    let mut variant_list: Vec<_> = variants.iter().collect();
    variant_list.sort_by_key(|v| std::cmp::Reverse(v.1));

    for (i, (variant, count)) in variant_list.iter().take(3).enumerate() {
        println!("  {}. ({} cases)", i + 1, count);
        println!("     {}", variant.join(" → "));
    }
}

fn count_activities(log: &EventLog) -> HashMap<String, usize> {
    let mut counts = HashMap::new();

    for trace in log.traces() {
        for event in &trace.events {
            *counts.entry(event.name.clone()).or_insert(0) += 1;
        }
    }

    counts
}

fn trace_length_distribution(log: &EventLog) -> HashMap<usize, usize> {
    let mut dist = HashMap::new();

    for trace in log.traces() {
        let length = trace.events.len();
        *dist.entry(length).or_insert(0) += 1;
    }

    dist
}

fn analyze_temporal(log: &EventLog) -> TemporalStats {
    let mut all_durations = Vec::new();
    let mut timestamps = Vec::new();

    for trace in log.traces() {
        if trace.events.is_empty() {
            continue;
        }

        let start = trace.events[0].timestamp;
        let end = trace.events[trace.events.len() - 1].timestamp;
        let duration = (end - start).num_hours();

        all_durations.push(duration);

        for event in &trace.events {
            timestamps.push(event.timestamp);
        }
    }

    timestamps.sort();
    let span = if timestamps.len() > 1 {
        (timestamps[timestamps.len() - 1] - timestamps[0]).num_days()
    } else {
        0
    };

    let avg_duration = if !all_durations.is_empty() {
        all_durations.iter().sum::<i64>() as f64 / all_durations.len() as f64
    } else {
        0.0
    };

    let min_duration = all_durations.iter().min().copied().unwrap_or(0) as f64;
    let max_duration = all_durations.iter().max().copied().unwrap_or(0) as f64;

    TemporalStats {
        span_days: span,
        avg_duration_hours: avg_duration,
        min_duration_hours: min_duration,
        max_duration_hours: max_duration,
    }
}

fn extract_variants(log: &EventLog) -> HashMap<Vec<String>, usize> {
    let mut variants = HashMap::new();

    for trace in log.traces() {
        let activities: Vec<String> = trace.events.iter().map(|e| e.name.clone()).collect();
        *variants.entry(activities).or_insert(0) += 1;
    }

    variants
}

struct TemporalStats {
    span_days: i64,
    avg_duration_hours: f64,
    min_duration_hours: f64,
    max_duration_hours: f64,
}

fn create_statistical_log() -> EventLog {
    let mut log = EventLog::new();
    let base = Utc::now();

    // Variant 1: Normal flow (60%)
    for i in 0..6 {
        let mut trace = Trace::new(format!("case_normal_{}", i));
        let offset = base + Duration::days(i as i64);

        trace.add_event(Event::new("Init".to_string(), offset));
        trace.add_event(Event::new(
            "Process".to_string(),
            offset + Duration::hours(2),
        ));
        trace.add_event(Event::new(
            "Review".to_string(),
            offset + Duration::hours(4),
        ));
        trace.add_event(Event::new(
            "Complete".to_string(),
            offset + Duration::hours(6),
        ));

        log.add_trace(trace);
    }

    // Variant 2: With rework (25%)
    for i in 6..9 {
        let mut trace = Trace::new(format!("case_rework_{}", i));
        let offset = base + Duration::days(i as i64);

        trace.add_event(Event::new("Init".to_string(), offset));
        trace.add_event(Event::new(
            "Process".to_string(),
            offset + Duration::hours(2),
        ));
        trace.add_event(Event::new(
            "Review".to_string(),
            offset + Duration::hours(4),
        ));
        trace.add_event(Event::new(
            "Rework".to_string(),
            offset + Duration::hours(6),
        ));
        trace.add_event(Event::new(
            "Review".to_string(),
            offset + Duration::hours(8),
        ));
        trace.add_event(Event::new(
            "Complete".to_string(),
            offset + Duration::hours(10),
        ));

        log.add_trace(trace);
    }

    // Variant 3: Rejected (15%)
    for i in 9..10 {
        let mut trace = Trace::new(format!("case_reject_{}", i));
        let offset = base + Duration::days(i as i64);

        trace.add_event(Event::new("Init".to_string(), offset));
        trace.add_event(Event::new(
            "Process".to_string(),
            offset + Duration::hours(2),
        ));
        trace.add_event(Event::new(
            "Review".to_string(),
            offset + Duration::hours(4),
        ));
        trace.add_event(Event::new(
            "Reject".to_string(),
            offset + Duration::hours(6),
        ));

        log.add_trace(trace);
    }

    log
}
