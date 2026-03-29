//! DFG-based filtering operations
//!
//! Filter event logs based on Directly-Follows Graph properties.

use crate::log::EventLog;
use std::collections::{HashMap, HashSet};

/// Filter log keeping only activities that cover a certain percentage of DFG activities
pub fn filter_dfg_activities_percentage(log: &EventLog, percentage: f64) -> EventLog {
    let mut filtered = EventLog::new();
    filtered.attributes = log.attributes.clone();

    // Get activity frequencies
    let mut activity_counts: HashMap<String, usize> = HashMap::new();
    for trace in &log.traces {
        for event in &trace.events {
            *activity_counts.entry(event.activity.clone()).or_insert(0) += 1;
        }
    }

    let total_events: usize = activity_counts.values().sum();
    let threshold = (total_events as f64 * percentage / 100.0) as usize;

    // Find activities that meet the threshold
    let keep_activities: HashSet<String> = activity_counts
        .into_iter()
        .filter(|(_, count)| *count >= threshold)
        .map(|(act, _)| act)
        .collect();

    // Filter traces to only keep events with kept activities
    for trace in &log.traces {
        let mut filtered_trace = trace.clone();
        filtered_trace
            .events
            .retain(|e| keep_activities.contains(&e.activity));
        if !filtered_trace.events.is_empty() {
            filtered.traces.push(filtered_trace);
        }
    }

    filtered
}

/// Filter log keeping only paths that cover a certain percentage of DFG paths
pub fn filter_dfg_paths_percentage(log: &EventLog, percentage: f64) -> EventLog {
    let mut filtered = EventLog::new();
    filtered.attributes = log.attributes.clone();

    // Get all path counts
    let mut path_counts: HashMap<(String, String), usize> = HashMap::new();
    let mut total_paths = 0usize;

    for trace in &log.traces {
        for window in trace.events.windows(2) {
            let path = (window[0].activity.clone(), window[1].activity.clone());
            *path_counts.entry(path).or_insert(0) += 1;
            total_paths += 1;
        }
    }

    let threshold = (total_paths as f64 * percentage / 100.0) as usize;

    // Find high-frequency paths
    let keep_paths: HashSet<(String, String)> = path_counts
        .into_iter()
        .filter(|(_, count)| *count >= threshold)
        .map(|(path, _)| path)
        .collect();

    // Keep traces that have high-frequency paths
    for trace in &log.traces {
        let has_high_freq_path = trace.events.windows(2).any(|window| {
            let path = (window[0].activity.clone(), window[1].activity.clone());
            keep_paths.contains(&path)
        });

        if has_high_freq_path {
            filtered.traces.push(trace.clone());
        }
    }

    filtered
}

/// Filter log by path performance (duration between activities)
pub fn filter_paths_performance(
    log: &EventLog,
    activity_a: &str,
    activity_b: &str,
    min_duration_seconds: f64,
    max_duration_seconds: f64,
) -> EventLog {
    let mut filtered = EventLog::new();
    filtered.attributes = log.attributes.clone();

    for trace in &log.traces {
        let mut keep_trace = false;

        for (i, event) in trace.events.iter().enumerate() {
            if event.activity == activity_a {
                // Find next occurrence of activity_b
                for later_event in trace.events.iter().skip(i + 1) {
                    if later_event.activity == activity_b {
                        let duration = (later_event.timestamp - event.timestamp).num_seconds();
                        if duration >= min_duration_seconds as i64
                            && duration <= max_duration_seconds as i64
                        {
                            keep_trace = true;
                        }
                        break;
                    }
                }
            }
        }

        if keep_trace {
            filtered.traces.push(trace.clone());
        }
    }

    filtered
}

/// Filter log by four-eyes principle (separation of duties)
pub fn filter_four_eyes_principle(log: &EventLog, activity_a: &str, activity_b: &str) -> EventLog {
    let mut filtered = EventLog::new();
    filtered.attributes = log.attributes.clone();

    for trace in &log.traces {
        let resource_a = trace
            .events
            .iter()
            .find(|e| e.activity == activity_a)
            .and_then(|e| e.resource.clone());

        let resource_b = trace
            .events
            .iter()
            .find(|e| e.activity == activity_b)
            .and_then(|e| e.resource.clone());

        // Four-eyes principle: different resources must perform the activities
        match (&resource_a, &resource_b) {
            (Some(ra), Some(rb)) if ra != rb => {
                filtered.traces.push(trace.clone());
            }
            (Some(_), None) | (None, Some(_)) => {
                // One activity not present, can't verify
                filtered.traces.push(trace.clone());
            }
            _ => {}
        }
    }

    filtered
}

/// Filter traces between two activities
pub fn filter_between(log: &EventLog, start_activity: &str, end_activity: &str) -> EventLog {
    let mut filtered = EventLog::new();
    filtered.attributes = log.attributes.clone();

    for trace in &log.traces {
        let has_start = trace.events.iter().any(|e| e.activity == start_activity);
        let has_end = trace.events.iter().any(|e| e.activity == end_activity);

        if has_start && has_end {
            filtered.traces.push(trace.clone());
        }
    }

    filtered
}

/// Filter by eventually-follows relation (transitive closure of directly-follows)
pub fn filter_eventually_follows_relation(
    log: &EventLog,
    activity_a: &str,
    activity_b: &str,
) -> EventLog {
    let mut filtered = EventLog::new();
    filtered.attributes = log.attributes.clone();

    for trace in &log.traces {
        let mut found_a = false;
        let mut found_b_after_a = false;

        for event in &trace.events {
            if event.activity == activity_a {
                found_a = true;
            } else if found_a && event.activity == activity_b {
                found_b_after_a = true;
                break;
            }
        }

        if found_b_after_a {
            filtered.traces.push(trace.clone());
        }
    }

    filtered
}

/// Filter variants by coverage percentage
pub fn filter_variants_by_coverage_percentage(
    log: &EventLog,
    coverage_percentage: f64,
) -> EventLog {
    let mut variant_counts: HashMap<Vec<String>, usize> = HashMap::new();
    let total_traces = log.traces.len();

    for trace in &log.traces {
        let variant: Vec<String> = trace.events.iter().map(|e| e.activity.clone()).collect();
        *variant_counts.entry(variant).or_insert(0) += 1;
    }

    let threshold = (total_traces as f64 * coverage_percentage / 100.0) as usize;

    let mut filtered = EventLog::new();
    filtered.attributes = log.attributes.clone();

    for trace in &log.traces {
        let variant: Vec<String> = trace.events.iter().map(|e| e.activity.clone()).collect();

        if let Some(count) = variant_counts.get(&variant) {
            if *count >= threshold {
                filtered.traces.push(trace.clone());
            }
        }
    }

    filtered
}

/// Filter by relative occurrence of an event attribute value
pub fn filter_log_relative_occurrence_event_attribute(
    log: &EventLog,
    attribute_name: &str,
    attribute_value: &str,
    min_occurrence: f64,
) -> EventLog {
    let mut total_occurrences = 0usize;

    for trace in &log.traces {
        for event in &trace.events {
            if let Some(val) = event.get_attribute(attribute_name) {
                if val == attribute_value {
                    total_occurrences += 1;
                }
            }
        }
    }

    let total_events: usize = log.traces.iter().map(|t| t.events.len()).sum();

    let relative_occurrence = if total_events > 0 {
        total_occurrences as f64 / total_events as f64
    } else {
        0.0
    };

    if relative_occurrence >= min_occurrence {
        // Return all traces (filter passes at log level)
        return log.clone();
    }

    // Otherwise, filter to only traces with the value
    let mut filtered = EventLog::new();
    filtered.attributes = log.attributes.clone();

    for trace in &log.traces {
        let has_value = trace.events.iter().any(|e| {
            e.get_attribute(attribute_name)
                .map(|v| v == attribute_value)
                .unwrap_or(false)
        });

        if has_value {
            filtered.traces.push(trace.clone());
        }
    }

    filtered
}

/// Filter log keeping only traces with a specific directly-follows relation
///
/// Returns traces where activity_a is directly followed by activity_b.
pub fn filter_directly_follows_relation(
    log: &EventLog,
    activity_a: &str,
    activity_b: &str,
) -> EventLog {
    let mut filtered = EventLog::new();
    filtered.attributes = log.attributes.clone();

    for trace in &log.traces {
        let has_relation = trace
            .events
            .windows(2)
            .any(|window| window[0].activity == activity_a && window[1].activity == activity_b);

        if has_relation {
            filtered.traces.push(trace.clone());
        }
    }

    filtered
}
