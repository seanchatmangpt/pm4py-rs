//! Operations on event logs

use super::{EventLog, Trace};
use chrono::{Duration, Utc};
use std::collections::HashMap;

/// Sort traces by length
pub fn sort_traces_by_length(log: &mut EventLog) {
    log.traces.sort_by_key(|t| std::cmp::Reverse(t.len()));
}

/// Sort traces by start timestamp
pub fn sort_traces_by_timestamp(log: &mut EventLog) {
    log.traces
        .sort_by_key(|t| t.events.first().map(|e| e.timestamp).unwrap_or(Utc::now()));
}

/// Start activities - get all activities that start a trace
pub fn start_activities(log: &EventLog) -> HashMap<String, usize> {
    let mut starts = HashMap::new();

    for trace in &log.traces {
        if let Some(event) = trace.events.first() {
            *starts.entry(event.activity.clone()).or_insert(0) += 1;
        }
    }

    starts
}

/// End activities - get all activities that end a trace
pub fn end_activities(log: &EventLog) -> HashMap<String, usize> {
    let mut ends = HashMap::new();

    for trace in &log.traces {
        if let Some(event) = trace.events.last() {
            *ends.entry(event.activity.clone()).or_insert(0) += 1;
        }
    }

    ends
}

/// Get activity frequency (optimized with pre-allocation and entry API)
pub fn activity_frequency(log: &EventLog) -> HashMap<String, usize> {
    let mut freq = HashMap::with_capacity(log.traces.len());

    for trace in &log.traces {
        for event in &trace.events {
            freq.entry(event.activity.clone())
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }
    }

    freq
}

/// Get directly follows relation (optimized to reduce allocations)
pub fn directly_follows(log: &EventLog) -> HashMap<(String, String), usize> {
    let mut follows = HashMap::with_capacity(log.traces.len() * 10); // Pre-allocate

    for trace in &log.traces {
        if trace.events.len() < 2 {
            continue;
        }

        for i in 0..trace.events.len() - 1 {
            let from = &trace.events[i].activity;
            let to = &trace.events[i + 1].activity;
            // Use entry API to avoid double lookup
            follows
                .entry((from.clone(), to.clone()))
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }
    }

    follows
}

/// Get activity resource mapping (optimized with pre-allocation and deduplication using HashSet)
pub fn activity_resources(log: &EventLog) -> HashMap<String, Vec<String>> {
    let mut mapping: HashMap<String, std::collections::HashSet<String>> =
        HashMap::with_capacity(log.traces.len());

    for trace in &log.traces {
        for event in &trace.events {
            if let Some(resource) = &event.resource {
                mapping
                    .entry(event.activity.clone())
                    .or_default()
                    .insert(resource.clone());
            }
        }
    }

    // Convert HashSets to sorted Vecs
    mapping
        .into_iter()
        .map(|(activity, resources)| {
            let mut sorted: Vec<_> = resources.into_iter().collect();
            sorted.sort();
            (activity, sorted)
        })
        .collect()
}

/// Remove duplicate events in a trace (keep first occurrence)
pub fn remove_duplicate_events(trace: &mut Trace) {
    let mut seen_activities = std::collections::HashSet::new();
    trace
        .events
        .retain(|e| seen_activities.insert(e.activity.clone()));
}

/// Remove duplicate events from all traces
pub fn remove_duplicates(log: &mut EventLog) {
    for trace in &mut log.traces {
        remove_duplicate_events(trace);
    }
}

/// Keep only top N activities by frequency
pub fn keep_top_activities(log: &mut EventLog, n: usize) {
    let freq = activity_frequency(log);
    let mut activities: Vec<_> = freq.into_iter().collect();
    activities.sort_by(|a, b| b.1.cmp(&a.1));

    let top_activities: std::collections::HashSet<_> = activities
        .into_iter()
        .take(n)
        .map(|(activity, _)| activity)
        .collect();

    for trace in &mut log.traces {
        trace
            .events
            .retain(|e| top_activities.contains(&e.activity));
    }

    // Remove empty traces
    log.traces.retain(|t| !t.is_empty());
}

/// Check if log is consistent (all traces contain same set of activities)
pub fn is_consistent(log: &EventLog) -> bool {
    if log.is_empty() {
        return true;
    }

    let first_trace_activities: std::collections::HashSet<_> =
        log.traces[0].events.iter().map(|e| &e.activity).collect();

    log.traces.iter().all(|trace| {
        let trace_activities: std::collections::HashSet<_> =
            trace.events.iter().map(|e| &e.activity).collect();
        trace_activities == first_trace_activities
    })
}

/// Get time between two consecutive activities in a trace
pub fn time_between_activities(
    log: &EventLog,
    activity_a: &str,
    activity_b: &str,
) -> Vec<Duration> {
    let mut durations = Vec::new();

    for trace in &log.traces {
        let events = &trace.events;
        for i in 0..events.len() - 1 {
            if events[i].activity == activity_a && events[i + 1].activity == activity_b {
                let duration = events[i + 1].timestamp - events[i].timestamp;
                durations.push(duration);
            }
        }
    }

    durations
}

/// Sequence encoding - convert trace to sequence representation
pub fn sequence_encoding(trace: &Trace) -> Vec<String> {
    trace.events.iter().map(|e| e.activity.clone()).collect()
}

/// Get variant of a trace (unique sequence of activities)
pub fn get_variant(trace: &Trace) -> String {
    sequence_encoding(trace).join(">")
}

/// Get variants with their frequency
pub fn variants(log: &EventLog) -> HashMap<String, usize> {
    let mut vars = HashMap::new();

    for trace in &log.traces {
        let variant = get_variant(trace);
        *vars.entry(variant).or_insert(0) += 1;
    }

    vars
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::{Event, Trace};
    use chrono::{Duration, Utc};

    fn create_test_log() -> EventLog {
        let mut log = EventLog::new();
        let mut trace1 = Trace::new("case_1");
        let now = Utc::now();

        trace1.add_event(Event::new("start", now));
        trace1.add_event(Event::new("middle", now + Duration::seconds(10)));
        trace1.add_event(Event::new("end", now + Duration::seconds(20)));

        log.add_trace(trace1);
        log
    }

    #[test]
    fn test_activity_frequency() {
        let log = create_test_log();
        let freq = activity_frequency(&log);

        assert_eq!(freq.get("start"), Some(&1));
        assert_eq!(freq.get("middle"), Some(&1));
        assert_eq!(freq.get("end"), Some(&1));
    }

    #[test]
    fn test_start_end_activities() {
        let log = create_test_log();
        let starts = start_activities(&log);
        let ends = end_activities(&log);

        assert_eq!(starts.get("start"), Some(&1));
        assert_eq!(ends.get("end"), Some(&1));
    }

    #[test]
    fn test_directly_follows() {
        let log = create_test_log();
        let follows = directly_follows(&log);

        assert_eq!(
            follows.get(&("start".to_string(), "middle".to_string())),
            Some(&1)
        );
        assert_eq!(
            follows.get(&("middle".to_string(), "end".to_string())),
            Some(&1)
        );
    }

    #[test]
    fn test_variants() {
        let log = create_test_log();
        let vars = variants(&log);

        assert_eq!(vars.get("start>middle>end"), Some(&1));
    }
}
