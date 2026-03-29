/// Extended performance metrics for detailed process analysis
use crate::log::{EventLog, Trace};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TracePerformanceMetrics {
    pub trace_id: String,
    pub cycle_time_seconds: f64,
    pub sojourn_times: HashMap<String, Vec<f64>>,
    pub avg_sojourn_times: HashMap<String, f64>,
    pub waiting_times: Vec<f64>,
    pub avg_waiting_time: f64,
    pub total_waiting_time: f64,
}

#[derive(Debug, Clone)]
pub struct ProcessPerformanceAnalysis {
    pub avg_cycle_time: f64,
    pub min_cycle_time: f64,
    pub max_cycle_time: f64,
    pub cycle_time_std_dev: f64,
    pub avg_sojourn_by_activity: HashMap<String, f64>,
    pub percentile_95_cycle_time: f64,
    pub avg_process_waiting_time: f64,
}

#[derive(Debug, Clone)]
pub struct ResourceUtilization {
    pub resource: String,
    pub active_time: f64,
    pub num_activities: usize,
    pub avg_time_per_activity: f64,
    pub activity_frequency: HashMap<String, usize>,
}

pub fn calculate_cycle_time(trace: &Trace) -> f64 {
    if trace.len() < 2 {
        return 0.0;
    }
    let sorted = trace.events_sorted();
    (sorted[sorted.len() - 1].timestamp - sorted[0].timestamp).num_seconds() as f64
}

pub fn calculate_sojourn_time(trace: &Trace, activity: &str) -> f64 {
    let sorted = trace.events_sorted();
    let mut total_time = 0.0;
    let mut count = 0;
    for i in 0..sorted.len() {
        if sorted[i].activity == activity && i + 1 < sorted.len() {
            total_time += (sorted[i + 1].timestamp - sorted[i].timestamp).num_seconds() as f64;
            count += 1;
        }
    }
    if count > 0 {
        total_time / count as f64
    } else {
        0.0
    }
}

pub fn calculate_waiting_times(trace: &Trace) -> Vec<f64> {
    if trace.len() < 2 {
        return Vec::new();
    }
    let sorted = trace.events_sorted();
    let mut waiting_times = Vec::new();
    for i in 0..sorted.len() - 1 {
        waiting_times.push((sorted[i + 1].timestamp - sorted[i].timestamp).num_seconds() as f64);
    }
    waiting_times
}

pub fn trace_performance_metrics(trace: &Trace) -> TracePerformanceMetrics {
    let mut sojourn_times: HashMap<String, Vec<f64>> = HashMap::new();
    let waiting_times = calculate_waiting_times(trace);
    let mut unique_activities = std::collections::HashSet::new();
    for event in &trace.events {
        unique_activities.insert(event.activity.clone());
    }
    for activity in unique_activities {
        let sojourn = calculate_sojourn_time(trace, &activity);
        if sojourn > 0.0 {
            sojourn_times.insert(activity, vec![sojourn]);
        }
    }
    let avg_sojourn_times = sojourn_times
        .iter()
        .map(|(a, t)| (a.clone(), t.iter().sum::<f64>() / t.len() as f64))
        .collect();
    let avg_waiting_time = if !waiting_times.is_empty() {
        waiting_times.iter().sum::<f64>() / waiting_times.len() as f64
    } else {
        0.0
    };
    TracePerformanceMetrics {
        trace_id: trace.id.clone(),
        cycle_time_seconds: calculate_cycle_time(trace),
        sojourn_times,
        avg_sojourn_times,
        waiting_times,
        avg_waiting_time,
        total_waiting_time: avg_waiting_time,
    }
}

pub fn process_performance_analysis(log: &EventLog) -> ProcessPerformanceAnalysis {
    if log.is_empty() {
        return ProcessPerformanceAnalysis {
            avg_cycle_time: 0.0,
            min_cycle_time: 0.0,
            max_cycle_time: 0.0,
            cycle_time_std_dev: 0.0,
            avg_sojourn_by_activity: HashMap::new(),
            percentile_95_cycle_time: 0.0,
            avg_process_waiting_time: 0.0,
        };
    }
    let mut cycle_times = Vec::new();
    for trace in &log.traces {
        cycle_times.push(calculate_cycle_time(trace));
    }
    let avg_cycle_time = cycle_times.iter().sum::<f64>() / cycle_times.len() as f64;
    let min_cycle_time = cycle_times.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_cycle_time = cycle_times
        .iter()
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max);
    let mut sorted_cycle = cycle_times.clone();
    sorted_cycle.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let percentile_95_index =
        ((sorted_cycle.len() as f64 * 0.95) as usize).min(sorted_cycle.len() - 1);
    ProcessPerformanceAnalysis {
        avg_cycle_time,
        min_cycle_time,
        max_cycle_time,
        cycle_time_std_dev: 0.0,
        avg_sojourn_by_activity: HashMap::new(),
        percentile_95_cycle_time: sorted_cycle[percentile_95_index],
        avg_process_waiting_time: 0.0,
    }
}

pub fn calculate_resource_utilization(log: &EventLog) -> Vec<ResourceUtilization> {
    let mut resource_metrics: HashMap<String, (f64, usize, HashMap<String, usize>)> =
        HashMap::new();
    for trace in &log.traces {
        for event in &trace.events {
            if let Some(resource) = &event.resource {
                let entry =
                    resource_metrics
                        .entry(resource.clone())
                        .or_insert((0.0, 0, HashMap::new()));
                entry.1 += 1;
                *entry.2.entry(event.activity.clone()).or_insert(0) += 1;
            }
        }
    }
    resource_metrics
        .into_iter()
        .map(|(r, (t, n, f))| ResourceUtilization {
            resource: r,
            active_time: t,
            num_activities: n,
            avg_time_per_activity: if n > 0 { t / n as f64 } else { 0.0 },
            activity_frequency: f,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::{Event, Trace};
    use chrono::Utc;

    fn create_test_trace() -> Trace {
        let base_time = Utc::now();
        let mut trace = Trace::new("case_1");
        trace.add_event(Event::new("a", base_time).with_resource("res1"));
        trace.add_event(Event::new("b", base_time + chrono::Duration::seconds(10)));
        trace
    }

    #[test]
    fn test_calculate_cycle_time() {
        let trace = create_test_trace();
        assert!(calculate_cycle_time(&trace) > 0.0);
    }

    #[test]
    fn test_cycle_time_empty() {
        let trace = Trace::new("empty");
        assert_eq!(calculate_cycle_time(&trace), 0.0);
    }

    #[test]
    fn test_calculate_sojourn_time() {
        let trace = create_test_trace();
        let sojourn = calculate_sojourn_time(&trace, "a");
        assert!(sojourn >= 0.0);
    }

    #[test]
    fn test_waiting_times() {
        let trace = create_test_trace();
        let times = calculate_waiting_times(&trace);
        assert!(!times.is_empty());
    }

    #[test]
    fn test_trace_metrics() {
        let trace = create_test_trace();
        let metrics = trace_performance_metrics(&trace);
        assert_eq!(metrics.trace_id, "case_1");
        assert!(metrics.cycle_time_seconds > 0.0);
    }

    #[test]
    fn test_process_analysis() {
        let mut log = EventLog::new();
        let mut trace = Trace::new("case_1");
        trace.add_event(Event::new("a", Utc::now()));
        log.add_trace(trace);
        let analysis = process_performance_analysis(&log);
        assert!(analysis.avg_cycle_time >= 0.0);
    }

    #[test]
    fn test_resource_util() {
        let mut log = EventLog::new();
        let mut trace = Trace::new("case_1");
        trace.add_event(Event::new("a", Utc::now()).with_resource("res1"));
        log.add_trace(trace);
        let util = calculate_resource_utilization(&log);
        assert!(!util.is_empty());
    }

    #[test]
    fn test_percentile() {
        let mut log = EventLog::new();
        let base_time = Utc::now();
        let mut trace = Trace::new("case_1");
        trace.add_event(Event::new("a", base_time));
        trace.add_event(Event::new("b", base_time + chrono::Duration::seconds(10)));
        log.add_trace(trace);
        let analysis = process_performance_analysis(&log);
        assert!(analysis.percentile_95_cycle_time >= 0.0);
    }

    #[test]
    fn test_resource_frequency() {
        let mut log = EventLog::new();
        let mut trace = Trace::new("case_1");
        trace.add_event(Event::new("a", Utc::now()).with_resource("res1"));
        log.add_trace(trace);
        let util = calculate_resource_utilization(&log);
        assert!(util.iter().all(|u| !u.activity_frequency.is_empty()));
    }
}
