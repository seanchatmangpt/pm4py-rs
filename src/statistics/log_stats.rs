/// Log-level statistics
use crate::log::{operations, EventLog};
use std::collections::{HashMap, HashSet};

/// Log statistics summary
#[derive(Debug, Clone)]
pub struct LogStats {
    pub num_traces: usize,
    pub num_events: usize,
    pub num_unique_activities: usize,
    pub num_variants: usize,
    pub avg_trace_length: f64,
    pub min_trace_length: usize,
    pub max_trace_length: usize,
}

/// Calculate comprehensive log statistics
pub fn log_statistics(log: &EventLog) -> LogStats {
    let num_traces = log.len();
    let num_events = log.num_events();
    let activities = log.activities();
    let num_unique_activities = activities.len();

    let variants = operations::variants(log);
    let num_variants = variants.len();

    let lengths: Vec<usize> = log.traces.iter().map(|t| t.len()).collect();
    let min_trace_length = *lengths.iter().min().unwrap_or(&0);
    let max_trace_length = *lengths.iter().max().unwrap_or(&0);

    let avg_trace_length = if num_traces > 0 {
        num_events as f64 / num_traces as f64
    } else {
        0.0
    };

    LogStats {
        num_traces,
        num_events,
        num_unique_activities,
        num_variants,
        avg_trace_length,
        min_trace_length,
        max_trace_length,
    }
}

/// Get activity occurrence matrix
pub fn activity_occurrence_matrix(log: &EventLog) -> HashMap<String, usize> {
    operations::activity_frequency(log)
}

/// Get directly-follows matrix
pub fn directly_follows_matrix(log: &EventLog) -> HashMap<(String, String), usize> {
    operations::directly_follows(log)
}

/// Filter traces by attributes
pub fn filter_traces_by_attribute(log: &EventLog, key: &str, value: &str) -> EventLog {
    let mut filtered = EventLog::new();
    filtered.attributes = log.attributes.clone();

    for trace in &log.traces {
        if let Some(attr_value) = trace.get_attribute(key) {
            if attr_value == value {
                filtered.add_trace(trace.clone());
            }
        }
    }

    filtered
}

/// Sample traces from log
pub fn sample_traces(log: &EventLog, sample_size: usize) -> EventLog {
    let mut filtered = EventLog::new();
    filtered.attributes = log.attributes.clone();

    let step = if !log.is_empty() {
        log.len() / sample_size.min(log.len())
    } else {
        1
    };

    for (i, trace) in log.traces.iter().enumerate() {
        if i % step == 0 && filtered.len() < sample_size {
            filtered.add_trace(trace.clone());
        }
    }

    filtered
}

/// Get start activities from log
pub fn get_start_activities(log: &EventLog) -> HashMap<String, usize> {
    let mut start_activities: HashMap<String, usize> = HashMap::new();

    for trace in &log.traces {
        if let Some(first_event) = trace.events.first() {
            *start_activities
                .entry(first_event.activity.clone())
                .or_insert(0) += 1;
        }
    }

    start_activities
}

/// Get end activities from log
pub fn get_end_activities(log: &EventLog) -> HashMap<String, usize> {
    let mut end_activities: HashMap<String, usize> = HashMap::new();

    for trace in &log.traces {
        if let Some(last_event) = trace.events.last() {
            *end_activities
                .entry(last_event.activity.clone())
                .or_insert(0) += 1;
        }
    }

    end_activities
}

/// Filter log by start activities
pub fn filter_start_activities(log: &EventLog, activities: &[String]) -> EventLog {
    let activity_set: HashSet<String> = activities.iter().cloned().collect();
    let mut filtered = EventLog::new();
    filtered.attributes = log.attributes.clone();

    for trace in &log.traces {
        if let Some(first_event) = trace.events.first() {
            if activity_set.contains(&first_event.activity) {
                filtered.add_trace(trace.clone());
            }
        }
    }

    filtered
}

/// Filter log by end activities
pub fn filter_end_activities(log: &EventLog, activities: &[String]) -> EventLog {
    let activity_set: HashSet<String> = activities.iter().cloned().collect();
    let mut filtered = EventLog::new();
    filtered.attributes = log.attributes.clone();

    for trace in &log.traces {
        if let Some(last_event) = trace.events.last() {
            if activity_set.contains(&last_event.activity) {
                filtered.add_trace(trace.clone());
            }
        }
    }

    filtered
}

/// Get case duration (time from first to last event in each trace)
pub fn get_case_duration(log: &EventLog) -> Vec<(String, chrono::Duration)> {
    let mut durations = Vec::new();

    for trace in &log.traces {
        if let (Some(first), Some(last)) = (trace.events.first(), trace.events.last()) {
            let duration = last.timestamp.signed_duration_since(first.timestamp);
            durations.push((trace.id.clone(), duration));
        }
    }

    durations
}

/// Get trace length (number of events) for each trace
pub fn get_trace_length(log: &EventLog) -> Vec<(String, usize)> {
    log.traces
        .iter()
        .map(|trace| (trace.id.clone(), trace.len()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::{Event, Trace};
    use chrono::Utc;

    fn create_test_log() -> EventLog {
        let mut log = EventLog::new();
        let now = Utc::now();

        for i in 1..=3 {
            let mut trace = Trace::new(format!("case_{}", i));
            trace.add_event(Event::new("a", now));
            trace.add_event(Event::new("b", now));
            log.add_trace(trace);
        }

        log
    }

    #[test]
    fn test_log_statistics() {
        let log = create_test_log();
        let stats = log_statistics(&log);

        assert_eq!(stats.num_traces, 3);
        assert_eq!(stats.num_events, 6);
        assert_eq!(stats.num_unique_activities, 2);
    }

    #[test]
    fn test_activity_occurrence_matrix() {
        let log = create_test_log();
        let matrix = activity_occurrence_matrix(&log);

        assert_eq!(matrix.get("a"), Some(&3));
        assert_eq!(matrix.get("b"), Some(&3));
    }

    #[test]
    fn test_sample_traces() {
        let log = create_test_log();
        let sampled = sample_traces(&log, 2);

        assert!(sampled.len() <= 2);
    }
}
