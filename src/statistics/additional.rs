//! Additional Statistics Functions for Complete Python pm4py Parity
//!
//! Implements 15+ missing functions across 5 categories:
//! 1. Attribute Statistics (4 functions)
//! 2. Resource/Role Analytics (4 functions)
//! 3. Time-Based Analytics (4 functions)
//! 4. Advanced Process Metrics (4 functions)
//! 5. Deviation Detection (3 functions)
//!
//! Target: <1e-10 parity with Python pm4py on test datasets

use crate::log::EventLog;
use std::collections::{HashMap, HashSet};
use chrono::Datelike;

// ============================================================================
// ATTRIBUTE STATISTICS (4 functions)
// ============================================================================

/// Attribute value frequency per case
#[derive(Debug, Clone)]
pub struct AttributeCaseStats {
    pub case_id: String,
    pub attributes: HashMap<String, String>,
}

/// Attribute value frequency across all cases
#[derive(Debug, Clone)]
pub struct AttributeFrequency {
    pub attribute_key: String,
    pub attribute_value: String,
    pub frequency: usize,
    pub percentage: f64,
}

/// Attribute co-occurrence pattern
#[derive(Debug, Clone)]
pub struct AttributeCoOccurrence {
    pub attribute1: (String, String), // (key, value)
    pub attribute2: (String, String), // (key, value)
    pub co_occurrence_count: usize,
    pub co_occurrence_percentage: f64,
}

/// Get attribute values per case
/// Returns a map of attribute keys to values for each case
pub fn get_case_attributes(log: &EventLog) -> Vec<HashMap<String, String>> {
    let mut case_attributes: Vec<HashMap<String, String>> = Vec::new();

    for trace in &log.traces {
        let mut attrs: HashMap<String, String> = HashMap::new();

        // Aggregate attributes from all events in the trace
        for event in &trace.events {
            if let Some(ref attr_map) = event.attributes {
                for (key, value) in attr_map {
                    // For same key, prefer first occurrence or concatenate
                    attrs.entry(key.clone()).or_insert_with(|| value.clone());
                }
            }
        }

        case_attributes.push(attrs);
    }

    case_attributes
}

/// Get attribute value frequency distribution
/// Returns frequency of each unique (attribute_key, attribute_value) pair
pub fn get_attribute_value_frequency(log: &EventLog) -> Vec<AttributeFrequency> {
    let mut attr_freq: HashMap<(String, String), usize> = HashMap::new();

    for trace in &log.traces {
        for event in &trace.events {
            if !event.attributes.is_empty() {
                for (key, value) in &event.attributes {
                    let pair = (key.clone(), value.clone());
                    *attr_freq.entry(pair).or_insert(0) += 1;
                }
            }
        }
    }

    let total_events = log.num_events() as f64;
    let mut result: Vec<AttributeFrequency> = attr_freq
        .into_iter()
        .map(|((key, value), freq)| AttributeFrequency {
            attribute_key: key,
            attribute_value: value,
            frequency: freq,
            percentage: (freq as f64 / total_events) * 100.0,
        })
        .collect();

    // Sort by frequency descending
    result.sort_by(|a, b| b.frequency.cmp(&a.frequency));
    result
}

/// Get attribute co-occurrence patterns
/// Finds pairs of attributes that frequently appear together
pub fn get_attribute_co_occurrence(log: &EventLog) -> Vec<AttributeCoOccurrence> {
    let mut co_occurrence: HashMap<((String, String), (String, String)), usize> = HashMap::new();
    let mut all_attr_events: usize = 0;

    for trace in &log.traces {
        for event in &trace.events {
            if !event.attributes.is_empty() {
                let attr_pairs: Vec<_> = event.attributes.iter().collect();

                // Find all pairs within the same event
                for i in 0..attr_pairs.len() {
                    for j in (i + 1)..attr_pairs.len() {
                        let key1 = (attr_pairs[i].0.clone(), attr_pairs[i].1.clone());
                        let key2 = (attr_pairs[j].0.clone(), attr_pairs[j].1.clone());

                        // Store in canonical order
                        let (pair1, pair2) = if key1 <= key2 {
                            (key1, key2)
                        } else {
                            (key2, key1)
                        };

                        *co_occurrence.entry((pair1, pair2)).or_insert(0) += 1;
                        all_attr_events += 1;
                    }
                }
            }
        }
    }

    let total_pairs = all_attr_events as f64;
    let mut result: Vec<AttributeCoOccurrence> = co_occurrence
        .into_iter()
        .filter(|(_, count)| *count > 0)
        .map(|((attr1, attr2), count)| AttributeCoOccurrence {
            attribute1: attr1,
            attribute2: attr2,
            co_occurrence_count: count,
            co_occurrence_percentage: if total_pairs > 0.0 {
                (count as f64 / total_pairs) * 100.0
            } else {
                0.0
            },
        })
        .collect();

    // Sort by count descending
    result.sort_by(|a, b| b.co_occurrence_count.cmp(&a.co_occurrence_count));
    result
}

/// Get distinct attribute values for a specific key
pub fn get_attribute_values_by_key(log: &EventLog, key: &str) -> Vec<(String, usize)> {
    let mut value_freq: HashMap<String, usize> = HashMap::new();

    for trace in &log.traces {
        for event in &trace.events {
            if let Some(value) = event.attributes.get(key) {
                *value_freq.entry(value.clone()).or_insert(0) += 1;
            }
        }
    }

    let mut result: Vec<(String, usize)> = value_freq.into_iter().collect();

    // Sort by frequency descending
    result.sort_by(|a, b| b.1.cmp(&a.1));
    result
}

// ============================================================================
// RESOURCE/ROLE ANALYTICS (4 functions)
// ============================================================================

/// Resource workload distribution
#[derive(Debug, Clone)]
pub struct ResourceWorkload {
    pub resource: String,
    pub event_count: usize,
    pub case_count: usize,
    pub avg_events_per_case: f64,
    pub workload_percentage: f64,
}

/// Resource availability metrics
#[derive(Debug, Clone)]
pub struct ResourceAvailability {
    pub resource: String,
    pub first_activity_time: chrono::DateTime<chrono::Utc>,
    pub last_activity_time: chrono::DateTime<chrono::Utc>,
    pub active_duration_seconds: f64,
    pub num_activities: usize,
}

/// Resource collaboration pattern
#[derive(Debug, Clone)]
pub struct ResourceCollaboration {
    pub resource1: String,
    pub resource2: String,
    pub shared_cases: usize,
    pub collaboration_percentage: f64,
}

/// Resource efficiency metric
#[derive(Debug, Clone)]
pub struct ResourceEfficiency {
    pub resource: String,
    pub avg_duration_per_activity: f64,
    pub max_duration: f64,
    pub min_duration: f64,
    pub efficiency_score: f64, // 0.0-1.0, higher is better
}

/// Get resource workload distribution
pub fn get_resource_workload(log: &EventLog) -> Vec<ResourceWorkload> {
    if log.traces.is_empty() {
        return Vec::new();
    }

    let mut resource_events: HashMap<String, usize> = HashMap::new();
    let mut resource_cases: HashMap<String, HashSet<String>> = HashMap::new();

    for trace in &log.traces {
        for event in &trace.events {
            if let Some(resource) = &event.resource {
                *resource_events.entry(resource.clone()).or_insert(0) += 1;
                resource_cases
                    .entry(resource.clone())
                    .or_default()
                    .insert(trace.id.clone());
            }
        }
    }

    let total_events = resource_events.values().sum::<usize>() as f64;
    let mut result: Vec<ResourceWorkload> = resource_events
        .into_iter()
        .map(|(resource, event_count)| {
            let case_count = resource_cases.get(&resource).map(|s| s.len()).unwrap_or(0);

            ResourceWorkload {
                resource: resource.clone(),
                event_count,
                case_count,
                avg_events_per_case: if case_count > 0 {
                    event_count as f64 / case_count as f64
                } else {
                    0.0
                },
                workload_percentage: (event_count as f64 / total_events) * 100.0,
            }
        })
        .collect();

    // Sort by event count descending
    result.sort_by(|a, b| b.event_count.cmp(&a.event_count));
    result
}

/// Get resource availability metrics (active time window)
pub fn get_resource_availability(log: &EventLog) -> Vec<ResourceAvailability> {
    let mut resource_times: HashMap<
        String,
        (chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>),
    > = HashMap::new();
    let mut resource_activity_count: HashMap<String, usize> = HashMap::new();

    for trace in &log.traces {
        for event in &trace.events {
            if let Some(resource) = &event.resource {
                *resource_activity_count.entry(resource.clone()).or_insert(0) += 1;

                let entry = resource_times
                    .entry(resource.clone())
                    .or_insert((event.timestamp, event.timestamp));

                if event.timestamp < entry.0 {
                    entry.0 = event.timestamp;
                }
                if event.timestamp > entry.1 {
                    entry.1 = event.timestamp;
                }
            }
        }
    }

    let mut result: Vec<ResourceAvailability> = resource_times
        .into_iter()
        .map(|(resource, (first, last))| {
            let duration_seconds = (last - first).num_seconds() as f64;
            let num_activities = resource_activity_count.get(&resource).copied().unwrap_or(0);

            ResourceAvailability {
                resource,
                first_activity_time: first,
                last_activity_time: last,
                active_duration_seconds: duration_seconds,
                num_activities,
            }
        })
        .collect();

    // Sort by number of activities descending
    result.sort_by(|a, b| b.num_activities.cmp(&a.num_activities));
    result
}

/// Get resource collaboration patterns
pub fn get_resource_collaboration(log: &EventLog) -> Vec<ResourceCollaboration> {
    let mut case_resources: HashMap<String, HashSet<String>> = HashMap::new();

    for trace in &log.traces {
        let mut resources: HashSet<String> = HashSet::new();
        for event in &trace.events {
            if let Some(resource) = &event.resource {
                resources.insert(resource.clone());
            }
        }
        case_resources.insert(trace.id.clone(), resources);
    }

    let mut collaboration: HashMap<(String, String), usize> = HashMap::new();
    let total_cases = log.traces.len() as f64;

    for resources in case_resources.values() {
        let resource_vec: Vec<_> = resources.iter().collect();
        for i in 0..resource_vec.len() {
            for j in (i + 1)..resource_vec.len() {
                let r1 = resource_vec[i].clone();
                let r2 = resource_vec[j].clone();

                // Store in canonical order
                let (res1, res2) = if r1 <= r2 { (r1, r2) } else { (r2, r1) };

                *collaboration.entry((res1, res2)).or_insert(0) += 1;
            }
        }
    }

    let mut result: Vec<ResourceCollaboration> = collaboration
        .into_iter()
        .map(|((r1, r2), count)| ResourceCollaboration {
            resource1: r1,
            resource2: r2,
            shared_cases: count,
            collaboration_percentage: (count as f64 / total_cases) * 100.0,
        })
        .collect();

    // Sort by shared cases descending
    result.sort_by(|a, b| b.shared_cases.cmp(&a.shared_cases));
    result
}

/// Get resource efficiency metrics
pub fn get_resource_efficiency(log: &EventLog) -> Vec<ResourceEfficiency> {
    let mut resource_durations: HashMap<String, Vec<f64>> = HashMap::new();

    for trace in &log.traces {
        let sorted = trace.events_sorted();
        for window in sorted.windows(2) {
            if let [curr, next] = window {
                if let Some(resource) = &curr.resource {
                    let duration = (next.timestamp - curr.timestamp).num_seconds() as f64;
                    resource_durations
                        .entry(resource.clone())
                        .or_default()
                        .push(duration);
                }
            }
        }
    }

    let mut result: Vec<ResourceEfficiency> = resource_durations
        .into_iter()
        .filter(|(_, durations)| !durations.is_empty())
        .map(|(resource, durations)| {
            let avg = durations.iter().sum::<f64>() / durations.len() as f64;
            let max = durations.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let min = durations.iter().cloned().fold(f64::INFINITY, f64::min);

            // Efficiency: higher if min/max is close to 1 (consistent), lower if highly variable
            let efficiency_score = if max > 0.0 {
                1.0 / (1.0 + (max - min) / max)
            } else {
                0.0
            };

            ResourceEfficiency {
                resource,
                avg_duration_per_activity: avg,
                max_duration: max,
                min_duration: min,
                efficiency_score,
            }
        })
        .collect();

    // Sort by efficiency score descending
    result.sort_by(|a, b| {
        b.efficiency_score
            .partial_cmp(&a.efficiency_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    result
}

// ============================================================================
// TIME-BASED ANALYTICS (4 functions)
// ============================================================================

/// Time between consecutive activities
#[derive(Debug, Clone)]
pub struct ActivityTransitionTime {
    pub from_activity: String,
    pub to_activity: String,
    pub avg_time_seconds: f64,
    pub min_time_seconds: f64,
    pub max_time_seconds: f64,
    pub count: usize,
}

/// Service level compliance result
#[derive(Debug, Clone)]
pub struct SLACompliance {
    pub case_id: String,
    pub total_duration_seconds: f64,
    pub sla_threshold_seconds: f64,
    pub is_compliant: bool,
    pub variance_seconds: f64,
}

/// Peak hours analysis
#[derive(Debug, Clone)]
pub struct PeakHourAnalysis {
    pub hour_of_day: usize,
    pub event_count: usize,
    pub percentage: f64,
}

/// Workload distribution result
#[derive(Debug, Clone)]
pub struct WorkloadDistribution {
    pub day_of_week: usize, // 0=Monday, 6=Sunday
    pub event_count: usize,
    pub case_count: usize,
    pub percentage: f64,
}

/// Calculate time between activities (cycle time)
pub fn calculate_activity_transition_times(log: &EventLog) -> Vec<ActivityTransitionTime> {
    let mut transition_times: HashMap<(String, String), Vec<f64>> = HashMap::new();

    for trace in &log.traces {
        let sorted = trace.events_sorted();
        for window in sorted.windows(2) {
            if let [curr, next] = window {
                let time_diff = (next.timestamp - curr.timestamp).num_seconds() as f64;
                let key = (curr.activity.clone(), next.activity.clone());
                transition_times
                    .entry(key)
                    .or_default()
                    .push(time_diff);
            }
        }
    }

    let mut result: Vec<ActivityTransitionTime> = transition_times
        .into_iter()
        .filter(|(_, times)| !times.is_empty())
        .map(|((from_act, to_act), times)| {
            let avg = times.iter().sum::<f64>() / times.len() as f64;
            let min = times.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = times.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

            ActivityTransitionTime {
                from_activity: from_act,
                to_activity: to_act,
                avg_time_seconds: avg,
                min_time_seconds: min,
                max_time_seconds: max,
                count: times.len(),
            }
        })
        .collect();

    // Sort by avg time descending
    result.sort_by(|a, b| {
        b.avg_time_seconds
            .partial_cmp(&a.avg_time_seconds)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    result
}

/// Check SLA compliance (default: 24 hours = 86400 seconds)
pub fn check_sla_compliance(log: &EventLog, sla_seconds: Option<f64>) -> Vec<SLACompliance> {
    let sla_threshold = sla_seconds.unwrap_or(86400.0); // Default 24 hours
    let mut result: Vec<SLACompliance> = Vec::new();

    for trace in &log.traces {
        if let (Some(first), Some(last)) = (trace.events.first(), trace.events.last()) {
            let total_duration = (last.timestamp - first.timestamp).num_seconds() as f64;
            let is_compliant = total_duration <= sla_threshold;
            let variance = total_duration - sla_threshold;

            result.push(SLACompliance {
                case_id: trace.id.clone(),
                total_duration_seconds: total_duration,
                sla_threshold_seconds: sla_threshold,
                is_compliant,
                variance_seconds: variance,
            });
        }
    }

    result
}

/// Analyze peak hours distribution
pub fn get_peak_hours_analysis(log: &EventLog) -> Vec<PeakHourAnalysis> {
    let mut hourly_count: HashMap<usize, usize> = HashMap::new();

    for trace in &log.traces {
        for event in &trace.events {
            let hour = event.timestamp.hour() as usize;
            *hourly_count.entry(hour).or_insert(0) += 1;
        }
    }

    let total_events = log.num_events() as f64;
    let mut result: Vec<PeakHourAnalysis> = (0..24)
        .map(|hour| {
            let count = hourly_count.get(&hour).copied().unwrap_or(0);
            PeakHourAnalysis {
                hour_of_day: hour,
                event_count: count,
                percentage: if total_events > 0.0 {
                    (count as f64 / total_events) * 100.0
                } else {
                    0.0
                },
            }
        })
        .collect();

    // Sort by event count descending
    result.sort_by(|a, b| b.event_count.cmp(&a.event_count));
    result
}

/// Analyze workload distribution by day of week
pub fn get_workload_distribution_by_day(log: &EventLog) -> Vec<WorkloadDistribution> {
    let mut daily_events: HashMap<usize, usize> = HashMap::new();
    let mut daily_cases: HashMap<usize, HashSet<String>> = HashMap::new();

    for trace in &log.traces {
        for event in &trace.events {
            let day_of_week = event.timestamp.weekday().number_from_monday() as usize - 1; // 0=Monday
            *daily_events.entry(day_of_week).or_insert(0) += 1;
            daily_cases
                .entry(day_of_week)
                .or_default()
                .insert(trace.id.clone());
        }
    }

    let total_events = log.num_events() as f64;
    let mut result: Vec<WorkloadDistribution> = (0..7)
        .map(|day| {
            let event_count = daily_events.get(&day).copied().unwrap_or(0);
            let case_count = daily_cases.get(&day).map(|s| s.len()).unwrap_or(0);

            WorkloadDistribution {
                day_of_week: day,
                event_count,
                case_count,
                percentage: if total_events > 0.0 {
                    (event_count as f64 / total_events) * 100.0
                } else {
                    0.0
                },
            }
        })
        .collect();

    // Sort by event count descending
    result.sort_by(|a, b| b.event_count.cmp(&a.event_count));
    result
}

// ============================================================================
// ADVANCED PROCESS METRICS (4 functions)
// ============================================================================

/// Process efficiency metrics
#[derive(Debug, Clone)]
pub struct ProcessEfficiencyMetrics {
    pub case_id: String,
    pub actual_time_seconds: f64,
    pub ideal_time_seconds: f64,
    pub efficiency_ratio: f64, // actual / ideal
    pub waste_percentage: f64, // ((actual - ideal) / actual) * 100
}

/// Wait vs processing time analysis
#[derive(Debug, Clone)]
pub struct WaitProcessingAnalysis {
    pub from_activity: String,
    pub to_activity: String,
    pub avg_wait_time_seconds: f64,
    pub avg_processing_time_seconds: f64,
    pub wait_processing_ratio: f64,
}

/// Case complexity score
#[derive(Debug, Clone)]
pub struct CaseComplexity {
    pub case_id: String,
    pub num_events: usize,
    pub num_unique_activities: usize,
    pub num_rework_instances: usize,
    pub complexity_score: f64, // Weighted combination
}

/// Resource efficiency score
#[derive(Debug, Clone)]
pub struct CaseResourceEfficiency {
    pub case_id: String,
    pub num_resources_involved: usize,
    pub resource_variance_score: f64,
    pub multi_resource_percentage: f64,
}

/// Calculate process efficiency (actual vs ideal time)
pub fn calculate_process_efficiency(
    log: &EventLog,
    ideal_time_seconds: f64,
) -> Vec<ProcessEfficiencyMetrics> {
    let mut result: Vec<ProcessEfficiencyMetrics> = Vec::new();

    for trace in &log.traces {
        if let (Some(first), Some(last)) = (trace.events.first(), trace.events.last()) {
            let actual_time = (last.timestamp - first.timestamp).num_seconds() as f64;
            let efficiency_ratio = actual_time / ideal_time_seconds;
            let waste_percentage = if actual_time > 0.0 {
                ((actual_time - ideal_time_seconds) / actual_time) * 100.0
            } else {
                0.0
            };

            result.push(ProcessEfficiencyMetrics {
                case_id: trace.id.clone(),
                actual_time_seconds: actual_time,
                ideal_time_seconds,
                efficiency_ratio,
                waste_percentage,
            });
        }
    }

    result
}

/// Calculate wait time vs processing time ratio
pub fn calculate_wait_processing_ratio(log: &EventLog) -> Vec<WaitProcessingAnalysis> {
    let mut transition_analysis: HashMap<(String, String), (Vec<f64>, Vec<f64>)> = HashMap::new();

    for trace in &log.traces {
        let sorted = trace.events_sorted();
        for window in sorted.windows(2) {
            if let [curr, next] = window {
                let wait_time = (next.timestamp - curr.timestamp).num_seconds() as f64;
                let key = (curr.activity.clone(), next.activity.clone());

                let entry = transition_analysis
                    .entry(key)
                    .or_insert_with(|| (Vec::new(), Vec::new()));

                // For simplicity, we treat time between activities as wait time
                // Processing time would be calculated from additional data
                entry.0.push(wait_time);
                entry.1.push(0.0); // Would need more data for actual processing time
            }
        }
    }

    let mut result: Vec<WaitProcessingAnalysis> = transition_analysis
        .into_iter()
        .filter(|(_, (wait_times, _))| !wait_times.is_empty())
        .map(|((from_act, to_act), (wait_times, _processing_times))| {
            let avg_wait = wait_times.iter().sum::<f64>() / wait_times.len() as f64;
            // In real scenario, would need actual processing time data
            let avg_processing = 0.0;

            WaitProcessingAnalysis {
                from_activity: from_act,
                to_activity: to_act,
                avg_wait_time_seconds: avg_wait,
                avg_processing_time_seconds: avg_processing,
                wait_processing_ratio: if avg_processing > 0.0 {
                    avg_wait / avg_processing
                } else {
                    f64::INFINITY
                },
            }
        })
        .collect();

    result
}

/// Calculate case complexity score
pub fn calculate_case_complexity(log: &EventLog) -> Vec<CaseComplexity> {
    let mut result: Vec<CaseComplexity> = Vec::new();

    for trace in &log.traces {
        let num_events = trace.events.len();

        let unique_activities: HashSet<String> =
            trace.events.iter().map(|e| e.activity.clone()).collect();
        let num_unique_activities = unique_activities.len();

        // Count rework (activities appearing more than once)
        let mut activity_count: HashMap<String, usize> = HashMap::new();
        for event in &trace.events {
            *activity_count.entry(event.activity.clone()).or_insert(0) += 1;
        }

        let num_rework_instances = activity_count
            .values()
            .filter(|count| **count > 1)
            .map(|count| count - 1)
            .sum::<usize>();

        // Complexity score: combination of factors
        // Events weight: higher = more complex
        // Unique activities weight: higher = more branching
        // Rework weight: higher = more rework
        let complexity_score = (num_events as f64 * 0.4)
            + (num_unique_activities as f64 * 0.3)
            + (num_rework_instances as f64 * 0.3);

        result.push(CaseComplexity {
            case_id: trace.id.clone(),
            num_events,
            num_unique_activities,
            num_rework_instances,
            complexity_score,
        });
    }

    // Sort by complexity score descending
    result.sort_by(|a, b| {
        b.complexity_score
            .partial_cmp(&a.complexity_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    result
}

/// Calculate case resource efficiency
pub fn calculate_case_resource_efficiency(log: &EventLog) -> Vec<CaseResourceEfficiency> {
    let mut result: Vec<CaseResourceEfficiency> = Vec::new();

    for trace in &log.traces {
        let resources: HashSet<String> = trace
            .events
            .iter()
            .filter_map(|e| e.resource.clone())
            .collect();

        let num_resources = resources.len();

        // Resource variance: higher if resources are evenly distributed
        let mut resource_count: HashMap<String, usize> = HashMap::new();
        for event in &trace.events {
            if let Some(resource) = &event.resource {
                *resource_count.entry(resource.clone()).or_insert(0) += 1;
            }
        }

        let num_events_with_resource = resource_count.values().sum::<usize>();
        let avg_events_per_resource = if num_resources > 0 {
            num_events_with_resource as f64 / num_resources as f64
        } else {
            0.0
        };

        // Variance calculation
        let variance = if num_resources > 0 {
            resource_count
                .values()
                .map(|count| {
                    let diff = *count as f64 - avg_events_per_resource;
                    diff * diff
                })
                .sum::<f64>()
                / num_resources as f64
        } else {
            0.0
        };

        let resource_variance_score = 1.0 / (1.0 + variance.sqrt());

        let multi_resource_percentage = if trace.events.len() > 0 {
            (num_events_with_resource as f64 / trace.events.len() as f64) * 100.0
        } else {
            0.0
        };

        result.push(CaseResourceEfficiency {
            case_id: trace.id.clone(),
            num_resources_involved: num_resources,
            resource_variance_score,
            multi_resource_percentage,
        });
    }

    result
}

// ============================================================================
// DEVIATION DETECTION (3 functions)
// ============================================================================

/// Outlier case detection
#[derive(Debug, Clone)]
pub struct OutlierCase {
    pub case_id: String,
    pub metric_name: String,
    pub value: f64,
    pub mean: f64,
    pub stddev: f64,
    pub z_score: f64,
    pub is_outlier: bool,
}

/// Anomalous path in process
#[derive(Debug, Clone)]
pub struct AnomalousPath {
    pub path: Vec<String>,
    pub expected_frequency: usize,
    pub actual_frequency: usize,
    pub anomaly_score: f64, // 0.0-1.0
}

/// Frequency anomaly detection
#[derive(Debug, Clone)]
pub struct FrequencyAnomaly {
    pub activity: String,
    pub expected_frequency: f64,
    pub actual_frequency: usize,
    pub anomaly_score: f64, // 0.0-1.0
}

/// Detect outlier cases by duration (z-score > 2.0)
pub fn detect_outlier_cases_by_duration(
    log: &EventLog,
    z_threshold: Option<f64>,
) -> Vec<OutlierCase> {
    let threshold = z_threshold.unwrap_or(2.0);
    let mut durations: Vec<(String, f64)> = Vec::new();

    for trace in &log.traces {
        if let (Some(first), Some(last)) = (trace.events.first(), trace.events.last()) {
            let duration = (last.timestamp - first.timestamp).num_seconds() as f64;
            durations.push((trace.id.clone(), duration));
        }
    }

    if durations.len() < 2 {
        return Vec::new();
    }

    let mean = durations.iter().map(|(_, d)| d).sum::<f64>() / durations.len() as f64;
    let variance = durations
        .iter()
        .map(|(_, d)| (d - mean).powi(2))
        .sum::<f64>()
        / durations.len() as f64;
    let stddev = variance.sqrt();

    let mut result: Vec<OutlierCase> = durations
        .into_iter()
        .map(|(case_id, value)| {
            let z_score = if stddev > 0.0 {
                (value - mean) / stddev
            } else {
                0.0
            };
            let is_outlier = z_score.abs() > threshold;

            OutlierCase {
                case_id,
                metric_name: "case_duration".to_string(),
                value,
                mean,
                stddev,
                z_score,
                is_outlier,
            }
        })
        .collect();

    // Filter to only outliers
    result.into_iter().filter(|o| o.is_outlier).collect()
}

/// Detect anomalous activity paths
pub fn detect_anomalous_paths(log: &EventLog, threshold: Option<f64>) -> Vec<AnomalousPath> {
    let anomaly_threshold = threshold.unwrap_or(0.7);
    let mut path_frequency: HashMap<Vec<String>, usize> = HashMap::new();

    for trace in &log.traces {
        let path: Vec<String> = trace.events.iter().map(|e| e.activity.clone()).collect();
        *path_frequency.entry(path).or_insert(0) += 1;
    }

    let total_cases = log.traces.len() as f64;
    let paths_count = path_frequency.len() as f64;

    let mut result: Vec<AnomalousPath> = path_frequency
        .into_iter()
        .map(|(path, frequency)| {
            let expected = (total_cases / paths_count).ceil() as usize;
            let anomaly_score = if expected > 0 {
                let diff = (frequency as i32 - expected as i32).abs() as f64;
                (diff / expected as f64).min(1.0)
            } else {
                0.0
            };

            AnomalousPath {
                path,
                expected_frequency: expected,
                actual_frequency: frequency,
                anomaly_score,
            }
        })
        .filter(|a| a.anomaly_score >= anomaly_threshold)
        .collect();

    // Sort by anomaly score descending
    result.sort_by(|a, b| {
        b.anomaly_score
            .partial_cmp(&a.anomaly_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    result
}

/// Detect frequency anomalies in activities
pub fn detect_frequency_anomalies(
    log: &EventLog,
    z_threshold: Option<f64>,
) -> Vec<FrequencyAnomaly> {
    let threshold = z_threshold.unwrap_or(2.0);
    let mut activity_counts: HashMap<String, usize> = HashMap::new();

    for trace in &log.traces {
        for event in &trace.events {
            *activity_counts.entry(event.activity.clone()).or_insert(0) += 1;
        }
    }

    if activity_counts.is_empty() {
        return Vec::new();
    }

    let counts: Vec<usize> = activity_counts.values().cloned().collect();
    let mean = counts.iter().map(|c| *c as f64).sum::<f64>() / counts.len() as f64;
    let variance = counts
        .iter()
        .map(|c| (*c as f64 - mean).powi(2))
        .sum::<f64>()
        / counts.len() as f64;
    let stddev = variance.sqrt();

    let mut result: Vec<FrequencyAnomaly> = activity_counts
        .into_iter()
        .map(|(activity, frequency)| {
            let z_score = if stddev > 0.0 {
                ((frequency as f64 - mean) / stddev).abs()
            } else {
                0.0
            };

            FrequencyAnomaly {
                activity,
                expected_frequency: mean,
                actual_frequency: frequency,
                anomaly_score: if z_score > threshold {
                    (z_score - threshold) / z_score
                } else {
                    0.0
                },
            }
        })
        .filter(|f| f.anomaly_score > 0.0)
        .collect();

    // Sort by anomaly score descending
    result.sort_by(|a, b| {
        b.anomaly_score
            .partial_cmp(&a.anomaly_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    result
}
