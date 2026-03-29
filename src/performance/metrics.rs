/// Performance metrics for process analysis
use crate::log::EventLog;
use chrono::Duration;
use std::collections::HashMap;

/// Case duration metrics
pub struct CaseDurationMetrics {
    pub total_cases: usize,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub avg_duration: Duration,
    pub median_duration: Duration,
}

/// Activity duration metrics
pub struct ActivityDurationMetrics {
    pub activity: String,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub avg_duration: Duration,
    pub frequency: usize,
}

/// Compute case durations
pub fn case_durations(log: &EventLog) -> Vec<Duration> {
    let mut durations = Vec::new();

    for trace in &log.traces {
        if let (Some(first), Some(last)) = (trace.events.first(), trace.events.last()) {
            let duration = last.timestamp - first.timestamp;
            durations.push(duration);
        }
    }

    durations
}

/// Compute case duration metrics
pub fn case_duration_metrics(log: &EventLog) -> Option<CaseDurationMetrics> {
    let durations = case_durations(log);

    if durations.is_empty() {
        return None;
    }

    let mut sorted = durations.clone();
    sorted.sort();

    let min = sorted.first().copied().unwrap_or_default();
    let max = sorted.last().copied().unwrap_or_default();

    let total: Duration = sorted.iter().sum();
    let avg = total / sorted.len() as i32;

    let median = if sorted.len() % 2 == 0 {
        (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2
    } else {
        sorted[sorted.len() / 2]
    };

    Some(CaseDurationMetrics {
        total_cases: log.len(),
        min_duration: min,
        max_duration: max,
        avg_duration: avg,
        median_duration: median,
    })
}

/// Get waiting time between two activities
pub fn waiting_time(log: &EventLog, activity_a: &str, activity_b: &str) -> Vec<Duration> {
    let mut times = Vec::new();

    for trace in &log.traces {
        for i in 0..trace.events.len() {
            if trace.events[i].activity == activity_a && i + 1 < trace.events.len() {
                for j in (i + 1)..trace.events.len() {
                    if trace.events[j].activity == activity_b {
                        let duration = trace.events[j].timestamp - trace.events[i].timestamp;
                        times.push(duration);
                        break;
                    }
                }
            }
        }
    }

    times
}

/// Get average processing time per activity
pub fn activity_processing_times(log: &EventLog) -> HashMap<String, Duration> {
    let mut processing_times: HashMap<String, Vec<Duration>> = HashMap::new();

    for trace in &log.traces {
        for i in 0..trace.events.len() - 1 {
            let activity = &trace.events[i].activity;
            let duration = trace.events[i + 1].timestamp - trace.events[i].timestamp;

            processing_times
                .entry(activity.clone())
                .or_default()
                .push(duration);
        }
    }

    let mut avg_times = HashMap::new();

    for (activity, times) in processing_times {
        if !times.is_empty() {
            let total: Duration = times.iter().sum();
            let avg = total / times.len() as i32;
            avg_times.insert(activity, avg);
        }
    }

    avg_times
}

/// Get throughput (cases per time unit)
pub fn throughput(log: &EventLog) -> Option<f64> {
    let durations = case_durations(log);

    if durations.is_empty() {
        return None;
    }

    let total_time: Duration = durations.iter().sum();
    if total_time.num_seconds() == 0 {
        return None;
    }

    let throughput = log.len() as f64 / total_time.num_seconds() as f64;
    Some(throughput)
}

/// Get rework cases (traces with repeated activities)
pub fn rework_cases(log: &EventLog) -> Vec<String> {
    let mut rework = Vec::new();

    for trace in &log.traces {
        let mut seen_activities = std::collections::HashSet::new();

        for event in &trace.events {
            if !seen_activities.insert(&event.activity) {
                // Activity already seen - this is rework
                rework.push(trace.id.clone());
                break;
            }
        }
    }

    rework
}

/// Get rework percentage
pub fn rework_percentage(log: &EventLog) -> f64 {
    if log.is_empty() {
        return 0.0;
    }

    let rework_count = rework_cases(log).len();
    (rework_count as f64 / log.len() as f64) * 100.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::{Event, Trace};
    use chrono::Utc;

    fn create_test_log() -> EventLog {
        let mut log = EventLog::new();
        let now = Utc::now();

        let mut trace = Trace::new("case_1");
        trace.add_event(Event::new("a", now));
        trace.add_event(Event::new("b", now + Duration::hours(1)));
        trace.add_event(Event::new("c", now + Duration::hours(2)));

        log.add_trace(trace);
        log
    }

    #[test]
    fn test_case_durations() {
        let log = create_test_log();
        let durations = case_durations(&log);

        assert_eq!(durations.len(), 1);
    }

    #[test]
    fn test_case_duration_metrics() {
        let log = create_test_log();
        let metrics = case_duration_metrics(&log);

        assert!(metrics.is_some());
        let m = metrics.unwrap();
        assert_eq!(m.total_cases, 1);
    }

    #[test]
    fn test_rework_cases() {
        let mut log = EventLog::new();
        let now = Utc::now();

        let mut trace = Trace::new("case_1");
        trace.add_event(Event::new("a", now));
        trace.add_event(Event::new("b", now));
        trace.add_event(Event::new("a", now)); // Rework

        log.add_trace(trace);

        let rework = rework_cases(&log);
        assert_eq!(rework.len(), 1);
    }
}
