//! Object-Centric Performance Analysis.
//!
//! Implements Connection 3 from "No AI Without PI": classical optimization from process constraints.
//!
//! Provides:
//! - `compute_throughput_by_object_type` — end-to-end throughput stats per object type
//! - `compute_activity_waiting_times` — waiting time between consecutive activities
//! - `detect_bottlenecks` — top-N edges ranked by severity (mean_wait × frequency)

use super::object_log::ObjectCentricEventLog;
use std::collections::BTreeMap;

/// End-to-end throughput statistics for objects of a single type.
#[derive(Debug, Clone, PartialEq)]
pub struct ThroughputStats {
    /// Mean case duration (first event → last event), in seconds.
    pub mean_secs: f64,
    /// Median case duration (seconds).
    pub median_secs: f64,
    /// 95th-percentile case duration (seconds).
    pub p95_secs: f64,
    /// Minimum case duration (seconds).
    pub min_secs: f64,
    /// Maximum case duration (seconds).
    pub max_secs: f64,
    /// Number of objects included in this computation.
    pub count: usize,
}

/// Waiting-time statistics between two consecutive activities for a given object type.
#[derive(Debug, Clone, PartialEq)]
pub struct ActivityWaitStats {
    /// Mean waiting time (seconds) between `from_activity` and `to_activity`.
    pub mean_wait_secs: f64,
    /// How many times this directly-follows pair was observed.
    pub frequency: u64,
    /// Sum of all observed waiting times (seconds) — useful for aggregation.
    pub total_wait_secs: f64,
}

/// A ranked bottleneck finding.
#[derive(Debug, Clone, PartialEq)]
pub struct BottleneckReport {
    /// Object type this bottleneck belongs to.
    pub object_type: String,
    /// Source activity.
    pub from_activity: String,
    /// Target activity.
    pub to_activity: String,
    /// Mean waiting time on this edge (seconds).
    pub mean_wait_secs: f64,
    /// How many times this edge was observed.
    pub frequency: u64,
    /// Composite severity: `mean_wait_secs × ln(frequency + 1)`.
    /// Higher = more impactful bottleneck.
    pub severity_score: f64,
}

// ── Throughput ────────────────────────────────────────────────────────────────

/// Compute end-to-end throughput (first→last event duration) for each object type.
///
/// Objects with fewer than 2 lifecycle events are excluded (no duration to measure).
pub fn compute_throughput_by_object_type(
    log: &ObjectCentricEventLog,
) -> BTreeMap<String, ThroughputStats> {
    // Collect per-object durations grouped by type
    let mut durations_by_type: BTreeMap<String, Vec<f64>> = BTreeMap::new();

    for (object_id, object) in &log.objects {
        let lifecycle = log.get_lifecycle_for_object(object_id);
        if lifecycle.len() < 2 {
            continue;
        }

        let first_ts = lifecycle.first().unwrap().2;
        let last_ts = lifecycle.last().unwrap().2;
        let duration_secs = (last_ts - first_ts).num_milliseconds().max(0) as f64 / 1000.0;

        durations_by_type
            .entry(object.object_type.name.clone())
            .or_default()
            .push(duration_secs);
    }

    durations_by_type
        .into_iter()
        .filter_map(|(type_name, mut samples)| {
            if samples.is_empty() {
                return None;
            }
            let count = samples.len();
            let mean_secs = samples.iter().sum::<f64>() / count as f64;
            let min_secs = samples.iter().cloned().fold(f64::INFINITY, f64::min);
            let max_secs = samples.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

            samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let median_secs = if count % 2 == 0 {
                (samples[count / 2 - 1] + samples[count / 2]) / 2.0
            } else {
                samples[count / 2]
            };
            let p95_idx = ((count as f64 * 0.95) as usize).min(count - 1);
            let p95_secs = samples[p95_idx];

            Some((
                type_name,
                ThroughputStats {
                    mean_secs,
                    median_secs,
                    p95_secs,
                    min_secs,
                    max_secs,
                    count,
                },
            ))
        })
        .collect()
}

// ── Activity Waiting Times ────────────────────────────────────────────────────

/// Compute mean waiting time between consecutive activities, keyed by
/// `(object_type, from_activity, to_activity)`.
pub fn compute_activity_waiting_times(
    log: &ObjectCentricEventLog,
) -> BTreeMap<(String, String, String), ActivityWaitStats> {
    // Raw samples: (type, from, to) → Vec<duration_secs>
    let mut raw: BTreeMap<(String, String, String), Vec<f64>> = BTreeMap::new();

    for (object_id, object) in &log.objects {
        let type_name = object.object_type.name.clone();
        let lifecycle = log.get_lifecycle_for_object(object_id);

        for window in lifecycle.windows(2) {
            let (_, from_act, from_ts) = &window[0];
            let (_, to_act, to_ts) = &window[1];

            let duration_secs = (*to_ts - *from_ts).num_milliseconds().max(0) as f64 / 1000.0;
            raw.entry((type_name.clone(), from_act.clone(), to_act.clone()))
                .or_default()
                .push(duration_secs);
        }
    }

    raw.into_iter()
        .map(|(key, samples)| {
            let frequency = samples.len() as u64;
            let total_wait_secs: f64 = samples.iter().sum();
            let mean_wait_secs = total_wait_secs / frequency as f64;
            (
                key,
                ActivityWaitStats {
                    mean_wait_secs,
                    frequency,
                    total_wait_secs,
                },
            )
        })
        .collect()
}

// ── Bottleneck Detection ──────────────────────────────────────────────────────

/// Detect the top-N bottlenecks ranked by severity score.
///
/// Severity = `mean_wait_secs × ln(frequency + 1)` — balances slowness and frequency.
/// A rare 10-minute wait ranks lower than a common 2-minute wait that blocks thousands.
pub fn detect_bottlenecks(log: &ObjectCentricEventLog, top_n: usize) -> Vec<BottleneckReport> {
    let waiting_times = compute_activity_waiting_times(log);

    let mut reports: Vec<BottleneckReport> = waiting_times
        .into_iter()
        .map(|((object_type, from_activity, to_activity), stats)| {
            let severity_score = stats.mean_wait_secs * (stats.frequency as f64 + 1.0).ln();
            BottleneckReport {
                object_type,
                from_activity,
                to_activity,
                mean_wait_secs: stats.mean_wait_secs,
                frequency: stats.frequency,
                severity_score,
            }
        })
        .collect();

    // Sort descending by severity_score
    reports.sort_by(|a, b| {
        b.severity_score
            .partial_cmp(&a.severity_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    reports.truncate(top_n);
    reports
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ocpm::object_log::{
        EventToObjectMapping, Object, ObjectCentricEventLog, ObjectType,
    };
    use chrono::{Duration, Utc};
    use uuid::Uuid;

    fn make_two_event_log_with_duration(duration_secs: i64) -> ObjectCentricEventLog {
        let mut log = ObjectCentricEventLog::new();
        let t0 = Utc::now();
        let t1 = t0 + Duration::seconds(duration_secs);

        let order_type = ObjectType::new("order");
        log.add_object(Object::new("o1", order_type, t0));

        let e1 = Uuid::new_v4();
        let e2 = Uuid::new_v4();
        log.add_event(e1, "create", t0, None);
        log.add_event(e2, "approve", t1, None);

        let mut m1 = EventToObjectMapping::new(e1);
        m1.add_object("o1");
        log.add_event_object_mapping(m1);

        let mut m2 = EventToObjectMapping::new(e2);
        m2.add_object("o1");
        log.add_event_object_mapping(m2);

        log
    }

    #[test]
    fn test_compute_throughput_basic() {
        let log = make_two_event_log_with_duration(60);
        let stats = compute_throughput_by_object_type(&log);

        let order_stats = stats.get("order").expect("order stats must exist");
        assert_eq!(order_stats.count, 1);
        // Duration should be ~60 seconds
        assert!(
            order_stats.mean_secs >= 59.0 && order_stats.mean_secs <= 61.0,
            "mean should be ~60s, got {}",
            order_stats.mean_secs
        );
    }

    #[test]
    fn test_compute_throughput_empty_log() {
        let log = ObjectCentricEventLog::new();
        let stats = compute_throughput_by_object_type(&log);
        assert!(stats.is_empty());
    }

    #[test]
    fn test_detect_bottlenecks_returns_known_slow_edge() {
        // o1: create → approve in 300 seconds (the bottleneck)
        let log = make_two_event_log_with_duration(300);
        let bottlenecks = detect_bottlenecks(&log, 5);

        assert_eq!(bottlenecks.len(), 1);
        assert_eq!(bottlenecks[0].from_activity, "create");
        assert_eq!(bottlenecks[0].to_activity, "approve");
        assert!(
            bottlenecks[0].mean_wait_secs >= 299.0,
            "bottleneck duration must be ~300s, got {}",
            bottlenecks[0].mean_wait_secs
        );
    }

    #[test]
    fn test_detect_bottlenecks_top_n_limit() {
        // Only 1 edge exists, so top-5 still returns 1
        let log = make_two_event_log_with_duration(10);
        let bottlenecks = detect_bottlenecks(&log, 5);
        assert!(bottlenecks.len() <= 5);
    }

    #[test]
    fn test_compute_activity_waiting_times() {
        let log = make_two_event_log_with_duration(120);
        let waits = compute_activity_waiting_times(&log);

        let key = (
            "order".to_string(),
            "create".to_string(),
            "approve".to_string(),
        );
        let wait = waits.get(&key).expect("create→approve wait must exist");
        assert_eq!(wait.frequency, 1);
        assert!(
            wait.mean_wait_secs >= 119.0 && wait.mean_wait_secs <= 121.0,
            "wait must be ~120s, got {}",
            wait.mean_wait_secs
        );
    }
}
