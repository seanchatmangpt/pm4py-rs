//! Rework Detection and Analysis
//!
//! Identifies redo activities, rework patterns, and their costs.
//!
//! ## Academic Foundation
//! Based on van der Aalst et al. "Process Mining: Data Science in Action" (2016)
//! - Rework: Activity instance caused by defect, error, or quality issue
//! - Rework detection: Identify same activity multiple times in same case
//! - Rework cost: Time/resources wasted due to rework
//!
//! ## Algorithms
//! - O(n) rework detection: single pass through log
//! - O(n log n) frequency analysis: sort occurrences
//! - O(n) cost calculation: timestamp differences

use crate::log::EventLog;
use std::collections::{HashMap, HashSet};

/// Rework instance for a specific activity in a case
#[derive(Debug, Clone, PartialEq)]
pub struct ReworkInstance {
    /// Case ID where rework occurred
    pub case_id: String,
    /// Activity name
    pub activity: String,
    /// Index of the rework occurrence (1 = first redo, 2 = second redo, etc.)
    pub redo_index: usize,
    /// Total occurrences of this activity in case
    pub total_occurrences: usize,
    /// Time between this occurrence and previous occurrence (seconds)
    pub time_since_previous: f64,
    /// Event indices in trace (0-based)
    pub event_indices: Vec<usize>,
}

/// Rework statistics for an activity
#[derive(Debug, Clone)]
pub struct ReworkStats {
    /// Activity name
    pub activity: String,
    /// Number of cases with rework for this activity
    pub cases_with_rework: usize,
    /// Total rework instances (sum of all redos across all cases)
    pub total_rework_instances: usize,
    /// Total cases where this activity appears
    pub total_cases_with_activity: usize,
    /// Rework frequency: percentage of cases that have rework
    pub rework_frequency: f64, // 0.0-1.0
    /// Average iterations per case (including first execution)
    pub average_iterations: f64,
    /// Most common rework pattern (e.g., how many times typically redone)
    pub mode_iterations: usize,
    /// Time wasted on rework (seconds)
    pub total_rework_time: f64,
    /// Average time per rework instance (seconds)
    pub average_rework_duration: f64,
    /// All rework instances for this activity
    pub instances: Vec<ReworkInstance>,
}

/// Loop detection result
#[derive(Debug, Clone)]
pub struct LoopPattern {
    /// Activities forming the loop in order
    pub activities: Vec<String>,
    /// Number of cases with this loop
    pub frequency: usize,
    /// Average loop iterations per case
    pub average_iterations: f64,
    /// Maximum nesting depth (for nested loops)
    pub max_depth: usize,
    /// Total time spent in loops (seconds)
    pub total_loop_time: f64,
}

/// Detect rework patterns: same activity occurring multiple times in a case
/// Returns list of rework instances in the order they occur
///
/// # Time Complexity
/// O(n) where n = total events in log
///
/// # Example
/// Case A: Register -> Examine -> Register (rework detected)
pub fn detect_rework(log: &EventLog) -> Vec<ReworkInstance> {
    let mut rework_instances = Vec::new();

    for trace in &log.traces {
        let case_id = trace.id.clone();
        let mut activity_indices: HashMap<String, Vec<usize>> = HashMap::new();

        // First pass: collect all indices for each activity
        for (idx, event) in trace.events.iter().enumerate() {
            activity_indices
                .entry(event.activity.clone())
                .or_default()
                .push(idx);
        }

        // Second pass: identify rework (activities with multiple occurrences)
        for (activity, indices) in activity_indices {
            if indices.len() > 1 {
                // This activity has rework
                for redo_idx in 1..indices.len() {
                    let current_event_idx = indices[redo_idx];
                    let previous_event_idx = indices[redo_idx - 1];

                    let time_diff = if let (Some(current_event), Some(previous_event)) = (
                        trace.events.get(current_event_idx),
                        trace.events.get(previous_event_idx),
                    ) {
                        let diff_ms =
                            (current_event.timestamp - previous_event.timestamp).num_milliseconds();
                        diff_ms as f64 / 1000.0
                    } else {
                        0.0
                    };

                    rework_instances.push(ReworkInstance {
                        case_id: case_id.clone(),
                        activity: activity.clone(),
                        redo_index: redo_idx,
                        total_occurrences: indices.len(),
                        time_since_previous: time_diff,
                        event_indices: indices.clone(),
                    });
                }
            }
        }
    }

    rework_instances
}

/// Compute rework statistics by activity
///
/// # Time Complexity
/// O(n log n) due to sorting by activity
pub fn rework_statistics(log: &EventLog) -> Vec<ReworkStats> {
    let rework_instances = detect_rework(log);

    // Group by activity
    let mut activity_groups: HashMap<String, Vec<ReworkInstance>> = HashMap::new();
    for instance in rework_instances {
        activity_groups
            .entry(instance.activity.clone())
            .or_default()
            .push(instance);
    }

    // Count cases with activity
    let mut activity_case_count: HashMap<String, HashSet<String>> = HashMap::new();
    for trace in &log.traces {
        for event in &trace.events {
            activity_case_count
                .entry(event.activity.clone())
                .or_default()
                .insert(trace.id.clone());
        }
    }

    // Compute statistics for each activity with rework
    let mut stats = Vec::new();

    for (activity, instances) in activity_groups {
        let cases_with_rework = instances
            .iter()
            .map(|i| i.case_id.clone())
            .collect::<HashSet<_>>()
            .len();

        let total_cases = activity_case_count
            .get(&activity)
            .map(|s| s.len())
            .unwrap_or(0);

        let total_instances = instances.len();
        let total_rework_time: f64 = instances.iter().map(|i| i.time_since_previous).sum();

        // Count iterations: sum up (total_occurrences) across all cases
        let total_iterations: usize = instances.iter().map(|i| i.total_occurrences).sum();
        let unique_cases_with_rework = instances
            .iter()
            .map(|i| i.case_id.clone())
            .collect::<HashSet<_>>()
            .len();

        let average_iterations = if unique_cases_with_rework > 0 {
            total_iterations as f64 / unique_cases_with_rework as f64
        } else {
            0.0
        };

        // Mode: most common iteration count
        let mut iteration_counts: HashMap<usize, usize> = HashMap::new();
        for instance in &instances {
            *iteration_counts
                .entry(instance.total_occurrences)
                .or_insert(0) += 1;
        }
        let mode_iterations = iteration_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(iter, _)| iter)
            .unwrap_or(2);

        stats.push(ReworkStats {
            activity: activity.clone(),
            cases_with_rework,
            total_rework_instances: total_instances,
            total_cases_with_activity: total_cases,
            rework_frequency: if total_cases > 0 {
                cases_with_rework as f64 / total_cases as f64
            } else {
                0.0
            },
            average_iterations,
            mode_iterations,
            total_rework_time,
            average_rework_duration: if total_instances > 0 {
                total_rework_time / total_instances as f64
            } else {
                0.0
            },
            instances,
        });
    }

    // Sort by rework frequency (descending)
    stats.sort_by(|a, b| {
        b.rework_frequency
            .partial_cmp(&a.rework_frequency)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    stats
}

/// Detect cyclic activity patterns in a trace
/// Returns all loops found, with their depth and frequency
///
/// Algorithm:
/// 1. Build directed activity graph from trace
/// 2. Detect cycles using DFS
/// 3. Group cycles by pattern
///
/// # Time Complexity
/// O(n²) in worst case (dense activity graph)
pub fn detect_loops(log: &EventLog) -> Vec<LoopPattern> {
    let mut loop_patterns: HashMap<Vec<String>, usize> = HashMap::new();
    let mut loop_times: HashMap<Vec<String>, f64> = HashMap::new();

    for trace in &log.traces {
        let activities: Vec<String> = trace.events.iter().map(|e| e.activity.clone()).collect();

        // Detect all loops in this trace
        for start_idx in 0..activities.len() {
            for end_idx in (start_idx + 1)..activities.len() {
                // Check if activities[start_idx] appears again at end_idx
                if activities[start_idx] == activities[end_idx] {
                    // Found a loop pattern from start_idx to end_idx
                    let loop_activities: Vec<String> = activities[start_idx..=end_idx].to_vec();

                    // Calculate time spent in loop
                    let loop_time = if let (Some(start_event), Some(end_event)) =
                        (trace.events.get(start_idx), trace.events.get(end_idx))
                    {
                        let diff_ms =
                            (end_event.timestamp - start_event.timestamp).num_milliseconds();
                        diff_ms as f64 / 1000.0
                    } else {
                        0.0
                    };

                    *loop_patterns.entry(loop_activities.clone()).or_insert(0) += 1;
                    *loop_times.entry(loop_activities).or_insert(0.0) += loop_time;
                }
            }
        }
    }

    // Convert to LoopPattern structs
    let mut patterns: Vec<LoopPattern> = loop_patterns
        .into_iter()
        .map(|(activities, frequency)| {
            let total_time = loop_times.get(&activities).copied().unwrap_or(0.0);
            let loop_depth = activities.len();

            LoopPattern {
                activities: activities.clone(),
                frequency,
                average_iterations: frequency as f64,
                max_depth: loop_depth,
                total_loop_time: total_time,
            }
        })
        .collect();

    // Sort by frequency (descending)
    patterns.sort_by(|a, b| b.frequency.cmp(&a.frequency));

    patterns
}

/// Detect if a loop could be infinite (recurring pattern appears many times)
/// Returns true if loop pattern repeats beyond expected bounds
///
/// Uses heuristic: if same pattern repeats > 2x median occurrences, flag as potential infinite loop
pub fn detect_potential_infinite_loops(log: &EventLog) -> Vec<String> {
    let loops = detect_loops(log);

    if loops.is_empty() {
        return Vec::new();
    }

    let frequencies: Vec<usize> = loops.iter().map(|l| l.frequency).collect();
    let median = frequencies[frequencies.len() / 2];

    loops
        .into_iter()
        .filter(|loop_pattern| {
            // Flag as potentially infinite if:
            // 1. Very high frequency (> 10 cases) AND
            // 2. Loop depth > 3 (complex pattern) AND
            // 3. Frequency significantly above median
            loop_pattern.frequency > 10
                && loop_pattern.max_depth > 3
                && loop_pattern.frequency > median * 2
        })
        .map(|l| {
            format!(
                "{} (freq: {}, depth: {})",
                l.activities.join(" -> "),
                l.frequency,
                l.max_depth
            )
        })
        .collect()
}

/// Get rework cases for a specific activity
/// Returns case IDs where the activity was redone
pub fn get_rework_cases_for_activity(log: &EventLog, activity: &str) -> Vec<String> {
    let rework_instances = detect_rework(log);

    rework_instances
        .into_iter()
        .filter(|instance| instance.activity == activity)
        .map(|instance| instance.case_id)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::Trace;
    use chrono::Utc;

    fn create_test_trace(case_id: &str, activities: Vec<&str>) -> Trace {
        let mut trace = Trace::new(case_id);
        let mut timestamp = Utc::now();

        for activity in activities {
            trace.add_event(crate::log::Event::new(activity, timestamp));
            timestamp = timestamp + chrono::Duration::hours(1);
        }

        trace
    }

    #[test]
    fn test_detect_rework_single_activity() {
        let mut log = EventLog::new();
        log.add_trace(create_test_trace(
            "case_1",
            vec!["Register", "Examine", "Register"],
        ));

        let rework = detect_rework(&log);
        assert_eq!(rework.len(), 1);
        assert_eq!(rework[0].activity, "Register");
        assert_eq!(rework[0].redo_index, 1);
        assert_eq!(rework[0].total_occurrences, 2);
    }

    #[test]
    fn test_detect_rework_multiple_activities() {
        let mut log = EventLog::new();
        log.add_trace(create_test_trace(
            "case_1",
            vec![
                "Register", "Examine", "Register", "Decide", "Examine", "Approve",
            ],
        ));

        let rework = detect_rework(&log);
        assert_eq!(rework.len(), 2); // Register + Examine both have rework
        assert!(rework.iter().any(|r| r.activity == "Register"));
        assert!(rework.iter().any(|r| r.activity == "Examine"));
    }

    #[test]
    fn test_rework_statistics() {
        let mut log = EventLog::new();
        log.add_trace(create_test_trace(
            "case_1",
            vec!["Register", "Examine", "Register"],
        ));
        log.add_trace(create_test_trace(
            "case_2",
            vec!["Register", "Examine", "Register"],
        ));
        log.add_trace(create_test_trace(
            "case_3",
            vec!["Register", "Examine", "Approve"],
        ));

        let stats = rework_statistics(&log);
        let register_stats = stats
            .iter()
            .find(|s| s.activity == "Register")
            .expect("Register activity must exist in rework statistics");

        assert_eq!(register_stats.cases_with_rework, 2);
        assert_eq!(register_stats.total_rework_instances, 2);
        assert_eq!(register_stats.total_cases_with_activity, 3);
        assert!(register_stats.rework_frequency > 0.66 && register_stats.rework_frequency < 0.67);
    }

    #[test]
    fn test_detect_loops() {
        let mut log = EventLog::new();
        log.add_trace(create_test_trace(
            "case_1",
            vec!["A", "B", "C", "A"], // Loop: A -> ... -> A
        ));

        let loops = detect_loops(&log);
        assert!(!loops.is_empty());
        assert!(loops[0].activities.contains(&"A".to_string()));
    }

    #[test]
    fn test_no_rework() {
        let mut log = EventLog::new();
        log.add_trace(create_test_trace(
            "case_1",
            vec!["Register", "Examine", "Approve"],
        ));

        let rework = detect_rework(&log);
        assert_eq!(rework.len(), 0);
    }

    #[test]
    fn test_no_loops() {
        let mut log = EventLog::new();
        log.add_trace(create_test_trace(
            "case_1",
            vec!["Register", "Examine", "Approve"],
        ));

        let loops = detect_loops(&log);
        assert_eq!(loops.len(), 0);
    }
}
