//! Log Statistics Example
//!
//! Demonstrates how to extract comprehensive statistics from event logs.
//! This example shows:
//! - Computing trace and event metrics
//! - Analyzing activity performance
//! - Identifying process variants
//! - Detecting bottlenecks
//!
//! Run with: cargo run --example stats_example

use chrono::{Duration, Utc};
use pm4py::log::{Event, EventLog, Trace};
use std::collections::HashMap;

fn main() {
    println!("╔════════════════════════════════════════════════════════╗");
    println!("║        PM4Py Log Statistics Example                    ║");
    println!("╚════════════════════════════════════════════════════════╝\n");

    let log = create_comprehensive_log();

    println!("EVENT LOG OVERVIEW");
    println!("═════════════════════════════════════════════════════════\n");

    basic_statistics(&log);

    println!("\nACTIVITY ANALYSIS");
    println!("═════════════════════════════════════════════════════════\n");

    activity_statistics(&log);

    println!("\nTRACE LENGTH ANALYSIS");
    println!("═════════════════════════════════════════════════════════\n");

    trace_length_analysis(&log);

    println!("\nCASE DURATION ANALYSIS");
    println!("═════════════════════════════════════════════════════════\n");

    case_duration_analysis(&log);

    println!("\nVARIANT ANALYSIS");
    println!("═════════════════════════════════════════════════════════\n");

    variant_analysis(&log);

    println!("\nBOTTLENECK DETECTION");
    println!("═════════════════════════════════════════════════════════\n");

    bottleneck_detection(&log);

    println!("\nPERFORMANCE SUMMARY");
    println!("═════════════════════════════════════════════════════════\n");

    performance_summary(&log);
}

fn basic_statistics(log: &EventLog) {
    let stats = log.statistics();

    println!("Log Statistics:");
    println!("  Total traces: {}", log.traces().len());
    println!("  Total events: {}", log.num_events());
    println!("  Unique activities: {}", stats.num_activities);
    println!();
    println!("Trace Length:");
    println!(
        "  Minimum: {} events",
        log.traces()
            .iter()
            .map(|t| t.events().len())
            .min()
            .unwrap_or(0)
    );
    println!(
        "  Maximum: {} events",
        log.traces()
            .iter()
            .map(|t| t.events().len())
            .max()
            .unwrap_or(0)
    );

    let avg_length = log.num_events() as f64 / log.traces().len() as f64;
    println!("  Average: {:.2} events", avg_length);
    println!();
    println!("Time Span:");
    if let Some(first_trace) = log.traces().first() {
        if let Some(last_event) = first_trace.events().first() {
            let start = last_event.timestamp();
            if let Some(last_trace) = log.traces().last() {
                if let Some(last_ev) = last_trace.events().last() {
                    let end = last_ev.timestamp();
                    println!("  Start: {}", start.format("%Y-%m-%d %H:%M:%S"));
                    println!("  End: {}", end.format("%Y-%m-%d %H:%M:%S"));
                }
            }
        }
    }
}

fn activity_statistics(log: &EventLog) {
    let mut activities: HashMap<&str, (usize, Vec<Duration>)> = HashMap::new();

    for trace in log.traces() {
        for event in trace.events() {
            let activity = event.name().as_str();
            let entry = activities.entry(activity).or_insert((0, Vec::new()));
            entry.0 += 1;
        }
    }

    // Sort by frequency
    let mut sorted: Vec<_> = activities.into_iter().collect();
    sorted.sort_by_key(|&(_, (freq, _))| std::cmp::Reverse(freq));

    println!("Top Activities:");
    for (i, (activity, (freq, _))) in sorted.iter().take(10).enumerate() {
        let pct = (freq as f64 / log.num_events() as f64) * 100.0;
        println!(
            "  {:2}. {:20} {} times ({:.1}%)",
            i + 1,
            activity,
            freq,
            pct
        );
    }

    if sorted.len() > 10 {
        println!("  ... and {} more", sorted.len() - 10);
    }
}

fn trace_length_analysis(log: &EventLog) {
    let lengths: Vec<usize> = log.traces().iter().map(|t| t.events().len()).collect();

    if lengths.is_empty() {
        return;
    }

    let mut sorted = lengths.clone();
    sorted.sort();

    let min = *sorted.first().unwrap();
    let max = *sorted.last().unwrap();
    let mean = lengths.iter().sum::<usize>() as f64 / lengths.len() as f64;
    let median = sorted[sorted.len() / 2];

    // Calculate standard deviation
    let variance = lengths
        .iter()
        .map(|&l| ((l as f64) - mean).powi(2))
        .sum::<f64>()
        / lengths.len() as f64;
    let std_dev = variance.sqrt();

    println!("Trace Length Distribution:");
    println!("  Minimum: {} events", min);
    println!("  Maximum: {} events", max);
    println!("  Mean: {:.2} events", mean);
    println!("  Median: {} events", median);
    println!("  Std Dev: {:.2} events", std_dev);
    println!();

    // Histogram
    println!("Distribution:");
    let buckets = vec![(0, 3), (3, 6), (6, 9), (9, 12), (12, 100)];
    for (min_len, max_len) in buckets {
        let count = lengths
            .iter()
            .filter(|&&l| l >= min_len && l < max_len)
            .count();
        let pct = (count as f64 / lengths.len() as f64) * 100.0;
        let bar = "█".repeat((pct / 5.0) as usize);
        println!(
            "  {:2}-{:2} events: {} ({:.1}%)",
            min_len,
            max_len - 1,
            bar,
            pct
        );
    }
}

fn case_duration_analysis(log: &EventLog) {
    let durations: Vec<Duration> = log
        .traces()
        .iter()
        .filter_map(|t| {
            let events = t.events();
            if events.len() >= 2 {
                let start = events.first()?.timestamp();
                let end = events.last()?.timestamp();
                Some(end.signed_duration_since(start))
            } else {
                None
            }
        })
        .collect();

    if durations.is_empty() {
        println!("No case duration data available");
        return;
    }

    let mean_secs = durations.iter().map(|d| d.num_seconds()).sum::<i64>() / durations.len() as i64;
    let mean = Duration::seconds(mean_secs);

    // Calculate percentiles
    let mut sorted = durations.clone();
    sorted.sort();

    let p50_idx = (sorted.len() as f64 * 0.50) as usize;
    let p95_idx = (sorted.len() as f64 * 0.95) as usize;
    let p99_idx = (sorted.len() as f64 * 0.99) as usize;

    println!("Case Duration Distribution:");
    println!(
        "  Mean: {} days {}h {}m",
        mean.num_days(),
        mean.num_hours() % 24,
        (mean.num_minutes() % 60)
    );
    println!(
        "  Median: {} days {}h {}m",
        sorted[p50_idx].num_days(),
        sorted[p50_idx].num_hours() % 24,
        (sorted[p50_idx].num_minutes() % 60)
    );
    println!(
        "  P95: {} days {}h {}m",
        sorted[p95_idx].num_days(),
        sorted[p95_idx].num_hours() % 24,
        (sorted[p95_idx].num_minutes() % 60)
    );
    println!(
        "  P99: {} days {}h {}m",
        sorted[p99_idx].num_days(),
        sorted[p99_idx].num_hours() % 24,
        (sorted[p99_idx].num_minutes() % 60)
    );
    println!(
        "  Min: {} days {}h {}m",
        sorted[0].num_days(),
        sorted[0].num_hours() % 24,
        (sorted[0].num_minutes() % 60)
    );
    println!(
        "  Max: {} days {}h {}m",
        sorted[sorted.len() - 1].num_days(),
        sorted[sorted.len() - 1].num_hours() % 24,
        (sorted[sorted.len() - 1].num_minutes() % 60)
    );
}

fn variant_analysis(log: &EventLog) {
    let mut variants: HashMap<Vec<String>, usize> = HashMap::new();

    for trace in log.traces() {
        let pattern: Vec<String> = trace.events().iter().map(|e| e.name().clone()).collect();
        *variants.entry(pattern).or_insert(0) += 1;
    }

    // Sort by frequency
    let mut sorted: Vec<_> = variants.into_iter().collect();
    sorted.sort_by_key(|&(_, count)| std::cmp::Reverse(count));

    println!("Top 10 Trace Variants:");
    for (i, (pattern, count)) in sorted.iter().take(10).enumerate() {
        let pct = (count as f64 / log.traces().len() as f64) * 100.0;

        // Format pattern
        let pattern_str = pattern
            .iter()
            .map(|s| s.chars().take(8).collect::<String>())
            .collect::<Vec<_>>()
            .join(" → ");

        println!("  {:2}. {} cases ({:.1}%)", i + 1, count, pct);
        println!("       {}", pattern_str);
    }

    if sorted.len() > 10 {
        println!("\n  ... and {} more variants", sorted.len() - 10);
    }

    // Variant concentration
    let top5_count: usize = sorted.iter().take(5).map(|(_, c)| c).sum();
    let top10_count: usize = sorted.iter().take(10).map(|(_, c)| c).sum();
    let pct_top5 = (top5_count as f64 / log.traces().len() as f64) * 100.0;
    let pct_top10 = (top10_count as f64 / log.traces().len() as f64) * 100.0;

    println!("\nVariant Concentration:");
    println!("  Top 5 variants: {:.1}% of cases", pct_top5);
    println!("  Top 10 variants: {:.1}% of cases", pct_top10);
    println!("  Total unique variants: {}", sorted.len());
}

fn bottleneck_detection(log: &EventLog) {
    let mut activity_times: HashMap<&str, Vec<Duration>> = HashMap::new();

    for trace in log.traces() {
        let events = trace.events();
        for i in 0..events.len() - 1 {
            if i + 1 < events.len() {
                let activity = events[i].name().as_str();
                let duration = events[i + 1]
                    .timestamp()
                    .signed_duration_since(events[i].timestamp());

                activity_times
                    .entry(activity)
                    .or_insert_with(Vec::new)
                    .push(duration);
            }
        }
    }

    // Calculate average duration per activity
    let mut avg_times: Vec<_> = activity_times
        .iter()
        .map(|(&activity, durations)| {
            let avg_secs =
                durations.iter().map(|d| d.num_seconds()).sum::<i64>() / durations.len() as i64;
            (activity, Duration::seconds(avg_secs))
        })
        .collect();

    avg_times.sort_by_key(|&(_, d)| std::cmp::Reverse(d.num_seconds()));

    println!("Activity Duration Analysis (potential bottlenecks):");
    println!();
    for (i, (activity, duration)) in avg_times.iter().take(5).enumerate() {
        let hours = duration.num_hours();
        let minutes = (duration.num_minutes() % 60);

        println!(
            "  {}. {} - {} avg",
            i + 1,
            activity,
            format_duration(*duration)
        );

        if hours > 4 {
            println!("     ⚠️  HIGH - Consider optimization");
        } else if hours > 1 {
            println!("     ⚠️  MEDIUM - Monitor");
        } else {
            println!("     ✓ Normal");
        }
    }
}

fn performance_summary(log: &EventLog) {
    let stats = log.statistics();

    println!("Process Health Indicators:");
    println!();

    // Volume indicator
    println!("1. Volume:");
    println!("   {} cases processed", log.traces().len());
    println!("   {} total events recorded", log.num_events());
    println!();

    // Consistency indicator
    let avg_length = log.num_events() as f64 / log.traces().len() as f64;
    println!("2. Consistency:");
    println!("   Average trace length: {:.2} events", avg_length);
    if avg_length > 5.0 {
        println!("   ⚠️  High variation - check for optional steps");
    } else {
        println!("   ✓ Relatively consistent");
    }
    println!();

    // Complexity indicator
    println!("3. Complexity:");
    println!("   {} unique activities", stats.num_activities);
    if stats.num_activities > 20 {
        println!("   ⚠️  High complexity - consider process simplification");
    } else {
        println!("   ✓ Manageable complexity");
    }
}

fn create_comprehensive_log() -> EventLog {
    let mut log = EventLog::new();
    let base = Utc::now();

    // Create 100 traces with variations
    for i in 0..100 {
        let mut trace = Trace::new(format!("case_{:04}", i));
        let offset = base + Duration::hours((i % 24) as i64);

        // Standard path (70% of cases)
        if i % 10 < 7 {
            trace.add_event(Event::new("start", offset));
            trace.add_event(Event::new("validate", offset + Duration::minutes(5)));
            trace.add_event(Event::new("process", offset + Duration::minutes(35)));
            trace.add_event(Event::new("review", offset + Duration::minutes(65)));
            trace.add_event(Event::new("approve", offset + Duration::hours(2)));
            trace.add_event(Event::new("complete", offset + Duration::hours(3)));
        }
        // With rework path (20%)
        else if i % 10 < 9 {
            trace.add_event(Event::new("start", offset));
            trace.add_event(Event::new("validate", offset + Duration::minutes(5)));
            trace.add_event(Event::new("process", offset + Duration::minutes(35)));
            trace.add_event(Event::new("review", offset + Duration::minutes(65)));
            trace.add_event(Event::new("reject", offset + Duration::hours(2)));
            trace.add_event(Event::new("rework", offset + Duration::hours(3)));
            trace.add_event(Event::new("review", offset + Duration::hours(4)));
            trace.add_event(Event::new("approve", offset + Duration::hours(5)));
            trace.add_event(Event::new("complete", offset + Duration::hours(6)));
        }
        // Escalated path (10%)
        else {
            trace.add_event(Event::new("start", offset));
            trace.add_event(Event::new("validate", offset + Duration::minutes(5)));
            trace.add_event(Event::new("escalate", offset + Duration::minutes(30)));
            trace.add_event(Event::new("senior_review", offset + Duration::hours(2)));
            trace.add_event(Event::new("approve", offset + Duration::hours(5)));
            trace.add_event(Event::new("complete", offset + Duration::hours(6)));
        }

        log.add_trace(trace);
    }

    log
}

fn format_duration(d: Duration) -> String {
    let days = d.num_days();
    let hours = d.num_hours() % 24;
    let minutes = d.num_minutes() % 60;

    if days > 0 {
        format!("{}d {}h {}m", days, hours, minutes)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}
