//! Bottleneck Identification and Analysis
//!
//! Identifies performance bottlenecks in process execution.
//!
//! ## Academic Foundation
//! Based on Little's Law and queueing theory
//! - Bottleneck: Resource or activity with longest queue time or throughput constraint
//! - Severity scoring: Normalized 0-100 based on impact
//! - Resource contention: Multiple cases competing for same resource
//!
//! ## Algorithms
//! - O(n) activity timing: single pass through log
//! - O(n log n) percentile calculation: sorting
//! - O(n) resource contention: hash aggregation

use crate::log::EventLog;
use crate::observability::Tracing;
use std::collections::{HashMap, HashSet};

/// Activity performance metrics
#[derive(Debug, Clone)]
pub struct ActivityMetrics {
    /// Activity name
    pub activity: String,
    /// Number of instances
    pub frequency: usize,
    /// Minimum duration in seconds
    pub min_duration: f64,
    /// Maximum duration in seconds
    pub max_duration: f64,
    /// Average duration in seconds
    pub average_duration: f64,
    /// Median duration in seconds
    pub median_duration: f64,
    /// P25 quartile (seconds)
    pub p25: f64,
    /// P75 quartile (seconds)
    pub p95: f64,
    /// Standard deviation (seconds)
    pub std_dev: f64,
    /// Total time spent (seconds)
    pub total_duration: f64,
    /// Average waiting time before activity (seconds)
    pub average_waiting_time: f64,
}

/// Bottleneck metrics
#[derive(Debug, Clone)]
pub struct BottleneckMetrics {
    /// Activity name
    pub activity: String,
    /// Cumulative time spent (seconds) - total duration of activity
    pub cumulative_time: f64,
    /// Average waiting time (seconds)
    pub average_waiting_time: f64,
    /// Resource contention score (0.0-1.0)
    pub contention_score: f64,
    /// Throughput (activities per hour)
    pub throughput: f64,
    /// Severity score (0-100)
    pub severity_score: f64,
    /// Explanation
    pub reason: String,
}

/// SLA violation
#[derive(Debug, Clone)]
pub struct SLAViolation {
    /// Case ID
    pub case_id: String,
    /// Activity name
    pub activity: String,
    /// Actual duration (seconds)
    pub actual_duration: f64,
    /// SLA threshold (seconds)
    pub sla_threshold: f64,
    /// Violation amount (seconds over threshold)
    pub violation_amount: f64,
}

/// Resource performance
#[derive(Debug, Clone)]
pub struct ResourcePerformance {
    /// Resource name
    pub resource: String,
    /// Number of activities performed
    pub activity_count: usize,
    /// Average activity duration (seconds)
    pub average_activity_duration: f64,
    /// Total time working (seconds)
    pub total_working_time: f64,
    /// Number of different activities
    pub unique_activities: usize,
    /// Resource utilization (0.0-1.0)
    pub utilization: f64,
}

/// Path performance metrics
#[derive(Debug, Clone)]
pub struct PathMetrics {
    /// Path as sequence of activities
    pub path: Vec<String>,
    /// Number of cases following this path
    pub frequency: usize,
    /// Percentage of cases (0.0-1.0)
    pub frequency_percentage: f64,
    /// Average duration (seconds)
    pub average_duration: f64,
    /// Minimum duration (seconds)
    pub min_duration: f64,
    /// Maximum duration (seconds)
    pub max_duration: f64,
    /// Percentile ranks
    pub p50: f64,
    pub p95: f64,
}

/// Calculate activity performance metrics
///
/// # Time Complexity
/// O(n log n) for sorting durations
pub fn calculate_activity_metrics(log: &EventLog) -> Vec<ActivityMetrics> {
    let mut activity_data: HashMap<String, Vec<f64>> = HashMap::new();
    let mut activity_waiting: HashMap<String, Vec<f64>> = HashMap::new();

    for trace in &log.traces {
        for (idx, event) in trace.events.iter().enumerate() {
            // Duration: time until next event (or 0 if last)
            let duration = if idx < trace.events.len() - 1 {
                if let Some(next_event) = trace.events.get(idx + 1) {
                    let diff_ms = (next_event.timestamp - event.timestamp).num_milliseconds();
                    (diff_ms as f64) / 1000.0
                } else {
                    0.0
                }
            } else {
                0.0
            };

            activity_data
                .entry(event.activity.clone())
                .or_default()
                .push(duration);

            // Waiting time: time since previous event
            let waiting_time = if idx > 0 {
                if let Some(prev_event) = trace.events.get(idx - 1) {
                    let diff_ms = (event.timestamp - prev_event.timestamp).num_milliseconds();
                    (diff_ms as f64) / 1000.0
                } else {
                    0.0
                }
            } else {
                0.0
            };

            activity_waiting
                .entry(event.activity.clone())
                .or_default()
                .push(waiting_time);
        }
    }

    let mut metrics = Vec::new();

    for (activity, durations) in activity_data {
        if durations.is_empty() {
            continue;
        }

        let mut sorted = durations.clone();
        sorted.sort_by(|a, b| a.total_cmp(b));

        let min = sorted[0];
        let max = sorted[sorted.len() - 1];
        let avg = sorted.iter().sum::<f64>() / sorted.len() as f64;
        let total = sorted.iter().sum::<f64>();

        let median = sorted[sorted.len() / 2];
        let p25_idx = sorted.len() / 4;
        let p25 = sorted[p25_idx];
        let p95_idx = (sorted.len() as f64 * 0.95) as usize;
        let p95 = sorted[std::cmp::min(p95_idx, sorted.len() - 1)];

        // Standard deviation
        let variance = sorted.iter().map(|d| (d - avg).powi(2)).sum::<f64>() / sorted.len() as f64;
        let std_dev = variance.sqrt();

        // Average waiting time
        let avg_waiting = if let Some(waiting_times) = activity_waiting.get(&activity) {
            if !waiting_times.is_empty() {
                waiting_times.iter().sum::<f64>() / waiting_times.len() as f64
            } else {
                0.0
            }
        } else {
            0.0
        };

        metrics.push(ActivityMetrics {
            activity,
            frequency: durations.len(),
            min_duration: min,
            max_duration: max,
            average_duration: avg,
            median_duration: median,
            p25,
            p95,
            std_dev,
            total_duration: total,
            average_waiting_time: avg_waiting,
        });
    }

    // Sort by cumulative time (descending)
    metrics.sort_by(|a, b| {
        b.total_duration
            .partial_cmp(&a.total_duration)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    metrics
}

/// Identify bottlenecks based on cumulative time and waiting time
///
/// # Bottleneck Definition
/// Activity is bottleneck if:
/// 1. High cumulative time (consumes >20% of total process time) OR
/// 2. High average waiting time (cases wait long before activity) OR
/// 3. High resource contention (many concurrent requests)
///
/// # Severity Scoring
/// score = (normalized_cumulative + normalized_waiting + contention) / 3 * 100
pub fn identify_bottlenecks(log: &EventLog) -> Vec<BottleneckMetrics> {
    let activity_metrics = calculate_activity_metrics(log);

    if activity_metrics.is_empty() {
        return Vec::new();
    }

    // Calculate total process time
    let total_time: f64 = activity_metrics.iter().map(|m| m.total_duration).sum();

    // Normalize metrics
    let max_cumulative = activity_metrics
        .iter()
        .map(|m| m.total_duration)
        .fold(0.0_f64, f64::max);
    let max_waiting = activity_metrics
        .iter()
        .map(|m| m.average_waiting_time)
        .fold(0.0_f64, f64::max);

    // Calculate resource contention (normalized by frequency)
    let mut bottlenecks = Vec::new();

    for metric in activity_metrics {
        let cumulative_norm = if max_cumulative > 0.0 {
            metric.total_duration / max_cumulative
        } else {
            0.0
        };

        let waiting_norm = if max_waiting > 0.0 {
            metric.average_waiting_time / max_waiting
        } else {
            0.0
        };

        // Resource contention: if activity frequency is high and total time is high
        let contention_score =
            (metric.frequency as f64 / log.traces.len() as f64) * cumulative_norm;

        // Severity: weighted combination
        let severity =
            ((cumulative_norm * 0.5) + (waiting_norm * 0.3) + (contention_score * 0.2)) * 100.0;

        let throughput = metric.frequency as f64 / (metric.total_duration / 3600.0);

        let mut reason_parts = Vec::new();
        if cumulative_norm > 0.3 {
            reason_parts.push(format!(
                "high cumulative time ({:.1}% of total)",
                (metric.total_duration / total_time) * 100.0
            ));
        }
        if waiting_norm > 0.5 {
            reason_parts.push(format!(
                "high waiting time ({:.1}s avg)",
                metric.average_waiting_time
            ));
        }
        if metric.frequency > (log.traces.len() / 2) {
            reason_parts.push(format!("high frequency ({}x)", metric.frequency));
        }

        let reason = if reason_parts.is_empty() {
            "General resource constraint".to_string()
        } else {
            reason_parts.join(", ")
        };

        bottlenecks.push(BottleneckMetrics {
            activity: metric.activity,
            cumulative_time: metric.total_duration,
            average_waiting_time: metric.average_waiting_time,
            contention_score: contention_score.min(1.0),
            throughput,
            severity_score: severity.min(100.0),
            reason,
        });
    }

    // Sort by severity (descending)
    bottlenecks.sort_by(|a, b| {
        b.severity_score
            .partial_cmp(&a.severity_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    bottlenecks
}

/// Identify bottlenecks and emit an OTEL span for each detected bottleneck activity.
///
/// Span name: `process.mining.bottleneck_detection`
/// Attributes on the top-level span:
/// - `process.mining.bottleneck.activity` = name of the highest-severity bottleneck (if any)
/// - `process.mining.bottleneck.score` = severity score of that bottleneck [0-100]
/// - `process.mining.case_count` = number of traces in the log
///
/// Returns the same `Vec<BottleneckMetrics>` as [`identify_bottlenecks`].
pub fn identify_bottlenecks_with_tracing(
    log: &EventLog,
    tracing: &Tracing,
) -> Vec<BottleneckMetrics> {
    let mut attrs = HashMap::new();
    attrs.insert(
        "process.mining.case_count".to_string(),
        log.traces.len().to_string(),
    );

    let mut span = tracing
        .start_span(
            crate::semconv::process_mining_span_names::PROCESS_MINING_BOTTLENECK_DETECTION_SPAN,
            attrs,
            None,
        )
        .expect("tracing start_span must not fail");

    let bottlenecks = identify_bottlenecks(log);

    // Record top bottleneck attributes on span
    if let Some(top) = bottlenecks.first() {
        span.attributes.insert(
            "process.mining.bottleneck.activity".to_string(),
            top.activity.clone(),
        );
        span.attributes.insert(
            "process.mining.bottleneck.score".to_string(),
            format!("{:.2}", top.severity_score),
        );
        span.attributes.insert(
            "process.mining.bottleneck.wait_ms".to_string(),
            format!("{:.2}", top.average_waiting_time * 1000.0),
        );
        span.attributes.insert(
            "process.mining.bottleneck.rank".to_string(),
            "1".to_string(),
        );
    }

    span.attributes.insert(
        "process.mining.bottleneck.count".to_string(),
        bottlenecks.len().to_string(),
    );

    tracing
        .end_span(&mut span, "ok", None)
        .expect("tracing end_span must not fail");

    bottlenecks
}

/// Check for SLA violations on activity duration
pub fn check_sla_violations(log: &EventLog, slas: &HashMap<String, f64>) -> Vec<SLAViolation> {
    let mut violations = Vec::new();

    for trace in &log.traces {
        for (idx, event) in trace.events.iter().enumerate() {
            if let Some(&sla_threshold) = slas.get(&event.activity) {
                // Duration: time until next event
                let duration = if idx < trace.events.len() - 1 {
                    if let Some(next_event) = trace.events.get(idx + 1) {
                        let diff_ms = (next_event.timestamp - event.timestamp).num_milliseconds();
                        (diff_ms as f64) / 1000.0
                    } else {
                        continue; // Skip last event
                    }
                } else {
                    continue; // Skip last event
                };

                if duration > sla_threshold {
                    violations.push(SLAViolation {
                        case_id: trace.id.clone(),
                        activity: event.activity.clone(),
                        actual_duration: duration,
                        sla_threshold,
                        violation_amount: duration - sla_threshold,
                    });
                }
            }
        }
    }

    violations.sort_by(|a, b| {
        b.violation_amount
            .partial_cmp(&a.violation_amount)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    violations
}

/// Analyze resource performance
pub fn analyze_resource_performance(log: &EventLog) -> Vec<ResourcePerformance> {
    let mut resource_data: HashMap<String, Vec<f64>> = HashMap::new();
    let mut resource_activities: HashMap<String, HashSet<String>> = HashMap::new();
    let mut resource_total_time: HashMap<String, f64> = HashMap::new();

    for trace in &log.traces {
        for (idx, event) in trace.events.iter().enumerate() {
            if let Some(resource) = &event.resource {
                // Duration: time until next event
                let duration = if idx < trace.events.len() - 1 {
                    if let Some(next_event) = trace.events.get(idx + 1) {
                        let diff_ms = (next_event.timestamp - event.timestamp).num_milliseconds();
                        (diff_ms as f64) / 1000.0
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };

                resource_data
                    .entry(resource.clone())
                    .or_default()
                    .push(duration);

                resource_activities
                    .entry(resource.clone())
                    .or_default()
                    .insert(event.activity.clone());

                *resource_total_time.entry(resource.clone()).or_insert(0.0) += duration;
            }
        }
    }

    let mut performance = Vec::new();

    for (resource, durations) in resource_data {
        let avg_duration = durations.iter().sum::<f64>() / durations.len() as f64;
        let total_time = resource_total_time.get(&resource).copied().unwrap_or(0.0);
        let unique_activities = resource_activities
            .get(&resource)
            .map(|s| s.len())
            .unwrap_or(0);

        // Utilization: ratio of working time to total log time
        let total_log_duration = if let (Some(first), Some(last)) = (
            log.traces.first().and_then(|t| t.events.first()),
            log.traces.last().and_then(|t| t.events.last()),
        ) {
            let diff_ms = (last.timestamp - first.timestamp).num_milliseconds();
            (diff_ms as f64) / 1000.0
        } else {
            1.0
        };

        let utilization = (total_time / total_log_duration).min(1.0);

        performance.push(ResourcePerformance {
            resource,
            activity_count: durations.len(),
            average_activity_duration: avg_duration,
            total_working_time: total_time,
            unique_activities,
            utilization,
        });
    }

    // Sort by total working time (descending)
    performance.sort_by(|a, b| {
        b.total_working_time
            .partial_cmp(&a.total_working_time)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    performance
}

/// Analyze path performance
pub fn analyze_path_performance(log: &EventLog) -> Vec<PathMetrics> {
    let mut path_durations: HashMap<Vec<String>, Vec<f64>> = HashMap::new();

    for trace in &log.traces {
        let path: Vec<String> = trace.events.iter().map(|e| e.activity.clone()).collect();

        if let (Some(first), Some(last)) = (trace.events.first(), trace.events.last()) {
            let duration_ms = (last.timestamp - first.timestamp).num_milliseconds();
            let duration = (duration_ms as f64) / 1000.0;

            path_durations.entry(path).or_default().push(duration);
        }
    }

    let total_cases = log.traces.len() as f64;
    let mut paths = Vec::new();

    for (path, durations) in path_durations {
        let mut sorted = durations.clone();
        sorted.sort_by(|a, b| a.total_cmp(b));

        let avg = sorted.iter().sum::<f64>() / sorted.len() as f64;
        let min = sorted[0];
        let max = sorted[sorted.len() - 1];
        let p50 = sorted[sorted.len() / 2];
        let p95_idx = (sorted.len() as f64 * 0.95) as usize;
        let p95 = sorted[std::cmp::min(p95_idx, sorted.len() - 1)];

        paths.push(PathMetrics {
            path,
            frequency: durations.len(),
            frequency_percentage: durations.len() as f64 / total_cases,
            average_duration: avg,
            min_duration: min,
            max_duration: max,
            p50,
            p95,
        });
    }

    // Sort by frequency (descending)
    paths.sort_by(|a, b| b.frequency.cmp(&a.frequency));

    paths
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::Trace;
    use chrono::Utc;

    fn create_test_log() -> EventLog {
        let mut log = EventLog::new();
        let now = Utc::now();

        let mut trace1 = Trace::new("case_1");
        trace1.add_event(crate::log::Event::new("A", now).with_resource("resource_1"));
        trace1.add_event(
            crate::log::Event::new("B", now + chrono::Duration::seconds(100))
                .with_resource("resource_2"),
        );
        trace1.add_event(
            crate::log::Event::new("C", now + chrono::Duration::seconds(200))
                .with_resource("resource_1"),
        );

        let mut trace2 = Trace::new("case_2");
        trace2.add_event(crate::log::Event::new("A", now).with_resource("resource_1"));
        trace2.add_event(
            crate::log::Event::new("B", now + chrono::Duration::seconds(150))
                .with_resource("resource_2"),
        );
        trace2.add_event(
            crate::log::Event::new("C", now + chrono::Duration::seconds(300))
                .with_resource("resource_1"),
        );

        log.add_trace(trace1);
        log.add_trace(trace2);
        log
    }

    #[test]
    fn test_calculate_activity_metrics() {
        let log = create_test_log();
        let metrics = calculate_activity_metrics(&log);

        assert!(!metrics.is_empty());
        assert!(metrics.iter().any(|m| m.activity == "A"));
        assert!(metrics.iter().any(|m| m.activity == "B"));
        assert!(metrics.iter().any(|m| m.activity == "C"));
    }

    #[test]
    fn test_identify_bottlenecks() {
        let log = create_test_log();
        let bottlenecks = identify_bottlenecks(&log);

        assert!(!bottlenecks.is_empty());
        assert!(bottlenecks
            .iter()
            .all(|b| b.severity_score >= 0.0 && b.severity_score <= 100.0));
    }

    #[test]
    fn test_check_sla_violations() {
        let log = create_test_log();
        let mut slas = HashMap::new();
        slas.insert("A".to_string(), 50.0); // 50 second SLA
        slas.insert("B".to_string(), 10.0); // 10 second SLA

        let violations = check_sla_violations(&log, &slas);
        assert!(!violations.is_empty()); // Should have violations
    }

    #[test]
    fn test_analyze_resource_performance() {
        let log = create_test_log();
        let performance = analyze_resource_performance(&log);

        assert!(!performance.is_empty());
        assert!(performance.iter().any(|p| p.resource == "resource_1"));
        assert!(performance.iter().any(|p| p.resource == "resource_2"));
    }

    #[test]
    fn test_analyze_path_performance() {
        let log = create_test_log();
        let paths = analyze_path_performance(&log);

        assert!(!paths.is_empty());
        // Both traces have same path: A -> B -> C
        assert!(paths.iter().any(|p| p.frequency == 2));
    }

    #[test]
    fn test_empty_log() {
        let log = EventLog::new();
        let metrics = calculate_activity_metrics(&log);
        assert_eq!(metrics.len(), 0);

        let bottlenecks = identify_bottlenecks(&log);
        assert_eq!(bottlenecks.len(), 0);
    }
}
