//! Event Log Abstraction to Plain English
//!
//! Converts event logs into concise descriptions of process execution patterns,
//! including case statistics, variant analysis, and resource utilization.

use crate::log::EventLog;
use std::collections::HashMap;

/// Variant information (execution trace)
#[derive(Debug, Clone, PartialEq)]
struct Variant {
    trace: String,
    occurrence_count: usize,
    percentage: f64,
}

/// Resource utilization information
#[derive(Debug, Clone)]
struct ResourceUtilization {
    resource_name: String,
    activity_count: usize,
    percentage: f64,
}

/// Analyze event log and return plain English abstraction
///
/// Output: "1,847 cases, 23 variants. Median duration: 5.2 days.
/// Top variant: Review→Approve→Close (456 cases, 24.7%).
/// Busiest resource: John_Smith (234 activities)."
pub fn abstract_event_log(log: &EventLog) -> String {
    let mut output = Vec::new();

    let case_count = log.len();
    output.push(format!("{} cases", case_count));

    // Analyze variants
    let variants = analyze_variants(log);
    output.push(format!("{} variants", variants.len()));

    // Compute median duration
    let durations = compute_case_durations(log);
    if !durations.is_empty() {
        let sorted_durations = sorted_durations(&durations);
        let median = median_value(&sorted_durations);
        output.push(format!("Median duration: {:.1} days", median));
    }

    let summary = output.join(", ");

    let mut result = vec![format!("{}.", summary)];

    // Top 3 variants by occurrence
    if !variants.is_empty() {
        let top_variants = &variants[0..std::cmp::min(3, variants.len())];
        let variant_str = top_variants
            .iter()
            .map(|v| {
                format!(
                    "{} ({} cases, {:.1}%)",
                    v.trace, v.occurrence_count, v.percentage
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        result.push(format!("Top variant(s): {}.", variant_str));
    }

    // Busiest resources
    let resources = analyze_resources(log);
    if !resources.is_empty() {
        let top_resource = &resources[0];
        result.push(format!(
            "Busiest resource: {} ({} activities, {:.1}%).",
            top_resource.resource_name, top_resource.activity_count, top_resource.percentage
        ));
    }

    result.join(" ")
}

/// Extract unique execution traces (variants)
fn analyze_variants(log: &EventLog) -> Vec<Variant> {
    let mut variant_map: HashMap<String, usize> = HashMap::new();

    for trace in &log.traces {
        let variant_key = trace
            .events
            .iter()
            .map(|e| e.activity.clone())
            .collect::<Vec<_>>()
            .join("→");

        *variant_map.entry(variant_key).or_insert(0) += 1;
    }

    let total = log.len();
    let mut variants: Vec<Variant> = variant_map
        .into_iter()
        .map(|(trace, count)| Variant {
            trace,
            occurrence_count: count,
            percentage: (count as f64 / total as f64) * 100.0,
        })
        .collect();

    variants.sort_by(|a, b| b.occurrence_count.cmp(&a.occurrence_count));
    variants
}

/// Compute case durations in days
fn compute_case_durations(log: &EventLog) -> Vec<f64> {
    let mut durations = Vec::new();

    for trace in &log.traces {
        if trace.events.is_empty() {
            continue;
        }

        let first_timestamp = trace.events.first().map(|e| e.timestamp);
        let last_timestamp = trace.events.last().map(|e| e.timestamp);

        if let (Some(start), Some(end)) = (first_timestamp, last_timestamp) {
            let duration_secs = (end - start).num_seconds() as f64;
            let duration_days = duration_secs / 86400.0;
            durations.push(duration_days);
        }
    }

    durations
}

/// Sort durations for median calculation
fn sorted_durations(durations: &[f64]) -> Vec<f64> {
    let mut sorted = durations.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    sorted
}

/// Calculate median value
fn median_value(sorted: &[f64]) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let mid = sorted.len() / 2;
    if sorted.len() % 2 == 0 {
        (sorted[mid - 1] + sorted[mid]) / 2.0
    } else {
        sorted[mid]
    }
}

/// Analyze resource utilization
fn analyze_resources(log: &EventLog) -> Vec<ResourceUtilization> {
    let mut resource_map: HashMap<String, usize> = HashMap::new();

    for trace in &log.traces {
        for event in &trace.events {
            if let Some(resource) = &event.resource {
                *resource_map.entry(resource.clone()).or_insert(0) += 1;
            }
        }
    }

    let total_events = log.num_events();
    let mut resources: Vec<ResourceUtilization> = resource_map
        .into_iter()
        .map(|(name, count)| ResourceUtilization {
            resource_name: name,
            activity_count: count,
            percentage: (count as f64 / total_events as f64) * 100.0,
        })
        .collect();

    resources.sort_by(|a, b| b.activity_count.cmp(&a.activity_count));
    resources
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::{Event, Trace};
    use chrono::Utc;

    #[test]
    fn test_abstract_event_log_simple() {
        let mut log = EventLog::new();

        let mut trace1 = Trace::new("case_1");
        let now = Utc::now();
        trace1.add_event(Event::new("Review", now).with_resource("Alice"));
        trace1.add_event(Event::new("Approve", now));
        log.add_trace(trace1);

        let mut trace2 = Trace::new("case_2");
        trace2.add_event(Event::new("Review", now).with_resource("Bob"));
        trace2.add_event(Event::new("Approve", now));
        log.add_trace(trace2);

        let abstract_str = abstract_event_log(&log);
        assert!(abstract_str.contains("2 cases"));
        assert!(abstract_str.contains("variant"));
    }

    #[test]
    fn test_variant_analysis() {
        let mut log = EventLog::new();

        let now = Utc::now();
        for _ in 0..5 {
            let mut trace = Trace::new("case");
            trace.add_event(Event::new("A", now));
            trace.add_event(Event::new("B", now));
            log.add_trace(trace);
        }

        let variants = analyze_variants(&log);
        assert!(!variants.is_empty());
        assert_eq!(variants[0].occurrence_count, 5);
    }

    #[test]
    fn test_median_calculation() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let median = median_value(&values);
        assert_eq!(median, 3.0);

        let values_even = vec![1.0, 2.0, 3.0, 4.0];
        let median_even = median_value(&values_even);
        assert_eq!(median_even, 2.5);
    }
}
