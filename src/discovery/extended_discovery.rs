//! Extended Discovery Algorithms
//!
//! Additional discovery algorithms for specialized process mining scenarios.

use crate::log::EventLog;
use crate::models::DirectlyFollowsGraph;
use std::collections::HashMap;

/// Typed DFG with activity type information
#[derive(Debug, Clone)]
pub struct TypedDFG {
    /// DFG edges with counts
    pub dfg: HashMap<(String, String), usize>,
    /// Activity to type mapping
    pub activity_types: HashMap<String, String>,
    /// Start activities with counts
    pub start_activities: HashMap<String, usize>,
    /// End activities with counts
    pub end_activities: HashMap<String, usize>,
}

impl TypedDFG {
    pub fn new() -> Self {
        Self {
            dfg: HashMap::new(),
            activity_types: HashMap::new(),
            start_activities: HashMap::new(),
            end_activities: HashMap::new(),
        }
    }
}

impl Default for TypedDFG {
    fn default() -> Self {
        Self::new()
    }
}

/// Discover a typed directly-follows graph
///
/// Returns a DFG where each activity has an associated type.
pub fn discover_dfg_typed(log: &EventLog, activity_type_key: Option<&str>) -> TypedDFG {
    let mut result = TypedDFG::new();

    for trace in &log.traces {
        if let Some((first, rest)) = trace.events.split_first() {
            // Record start activity
            *result
                .start_activities
                .entry(first.activity.clone())
                .or_insert(0) += 1;

            // Set activity type if key provided
            if let Some(key) = activity_type_key {
                if let Some(type_val) = first.get_attribute(key) {
                    result
                        .activity_types
                        .insert(first.activity.clone(), type_val.to_string());
                }
            }

            let mut prev = first;
            for event in rest {
                // Record DFG edge
                let key = (prev.activity.clone(), event.activity.clone());
                *result.dfg.entry(key).or_insert(0) += 1;

                // Set activity type if key provided
                if let Some(key) = activity_type_key {
                    if let Some(type_val) = event.get_attribute(key) {
                        result
                            .activity_types
                            .insert(event.activity.clone(), type_val.to_string());
                    }
                }

                prev = event;
            }

            // Record end activity
            *result
                .end_activities
                .entry(prev.activity.clone())
                .or_insert(0) += 1;
        }
    }

    result
}

/// Eventually-follows relation with distance information
#[derive(Debug, Clone)]
pub struct EventuallyFollowsGraph {
    /// Eventually-follows relations with counts
    pub relations: HashMap<(String, String), usize>,
    /// Average distance between activities
    pub distances: HashMap<(String, String), f64>,
}

impl EventuallyFollowsGraph {
    pub fn new() -> Self {
        Self {
            relations: HashMap::new(),
            distances: HashMap::new(),
        }
    }
}

impl Default for EventuallyFollowsGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Discover eventually-follows graph
///
/// Finds activities that eventually follow each other, not necessarily directly.
pub fn discover_eventually_follows_graph(log: &EventLog) -> EventuallyFollowsGraph {
    let mut relations: HashMap<(String, String), usize> = HashMap::new();
    let mut distance_sums: HashMap<(String, String), usize> = HashMap::new();
    let mut distance_counts: HashMap<(String, String), usize> = HashMap::new();

    for trace in &log.traces {
        let events = &trace.events;

        for i in 0..events.len() {
            for j in (i + 1)..events.len() {
                let from = &events[i].activity;
                let to = &events[j].activity;

                // Record relation
                *relations.entry((from.clone(), to.clone())).or_insert(0) += 1;

                // Record distance
                let distance = j - i;
                *distance_sums.entry((from.clone(), to.clone())).or_insert(0) += distance;
                *distance_counts
                    .entry((from.clone(), to.clone()))
                    .or_insert(0) += 1;
            }
        }
    }

    // Calculate average distances
    let mut distances = HashMap::new();
    for (key, sum) in distance_sums {
        if let Some(count) = distance_counts.get(&key) {
            if *count > 0 {
                distances.insert(key, sum as f64 / *count as f64);
            }
        }
    }

    EventuallyFollowsGraph {
        relations,
        distances,
    }
}

/// Occurrence Transition Graph (OTG)
///
/// Represents activity transitions with occurrence counts.
#[derive(Debug, Clone)]
pub struct OccurrenceTransitionGraph {
    /// Transition counts
    pub transitions: HashMap<(String, String), usize>,
    /// Activity occurrence counts
    pub occurrences: HashMap<String, usize>,
}

impl OccurrenceTransitionGraph {
    pub fn new() -> Self {
        Self {
            transitions: HashMap::new(),
            occurrences: HashMap::new(),
        }
    }
}

impl Default for OccurrenceTransitionGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Discover occurrence transition graph
///
/// Creates a graph showing transition frequencies between activities.
pub fn discover_otg(log: &EventLog) -> OccurrenceTransitionGraph {
    let mut transitions: HashMap<(String, String), usize> = HashMap::new();
    let mut occurrences: HashMap<String, usize> = HashMap::new();

    for trace in &log.traces {
        for event in &trace.events {
            // Count occurrence
            *occurrences.entry(event.activity.clone()).or_insert(0) += 1;
        }

        // Count transitions
        for window in trace.events.windows(2) {
            let from = &window[0].activity;
            let to = &window[1].activity;
            *transitions.entry((from.clone(), to.clone())).or_insert(0) += 1;
        }
    }

    OccurrenceTransitionGraph {
        transitions,
        occurrences,
    }
}

/// Batch detection result
#[derive(Debug, Clone)]
pub struct BatchInfo {
    /// Activity that is performed in batches
    pub activity: String,
    /// Resource performing the batched activity
    pub resource: String,
    /// Number of batches detected
    pub batch_count: usize,
    /// Average batch size
    pub avg_batch_size: f64,
    /// List of batch cases
    pub cases: Vec<String>,
}

/// Discover batches in event log
///
/// Detects activities performed in batches by the same resource.
pub fn discover_batches(log: &EventLog, min_batch_size: usize) -> Vec<BatchInfo> {
    let mut batches: Vec<BatchInfo> = Vec::new();
    let mut batch_key: HashMap<(String, String), Vec<&str>> = HashMap::new();

    // Group events by (activity, resource)
    for trace in &log.traces {
        let trace_id = trace.id.as_str();

        for event in &trace.events {
            if let Some(ref resource) = event.resource {
                let key = (event.activity.clone(), resource.clone());
                batch_key.entry(key).or_default().push(trace_id);
            }
        }
    }

    // Detect batches
    for ((activity, resource), case_ids) in batch_key {
        // Count occurrences per case
        let mut case_counts: HashMap<&str, usize> = HashMap::new();
        for case_id in &case_ids {
            *case_counts.entry(case_id).or_insert(0) += 1;
        }

        // Find cases with multiple occurrences (potential batches)
        let batch_cases: Vec<(&str, usize)> = case_counts
            .iter()
            .filter(|(_, &count)| count >= min_batch_size)
            .map(|(&id, &count)| (id, count))
            .collect();

        if !batch_cases.is_empty() {
            let total_count: usize = batch_cases.iter().map(|(_, count)| *count).sum();
            let avg_size = total_count as f64 / batch_cases.len() as f64;

            batches.push(BatchInfo {
                activity: activity.clone(),
                resource: resource.clone(),
                batch_count: batch_cases.len(),
                avg_batch_size: avg_size,
                cases: batch_cases.iter().map(|(id, _)| id.to_string()).collect(),
            });
        }
    }

    batches.sort_by(|a, b| b.batch_count.cmp(&a.batch_count));
    batches
}

/// Correlation miner result
#[derive(Debug, Clone)]
pub struct CorrelationMinerResult {
    /// Activity pairs with correlation values
    pub correlations: HashMap<(String, String), f64>,
    /// Highly correlated pairs (correlation > threshold)
    pub high_correlation: Vec<(String, String, f64)>,
}

/// Discover activity correlations
///
/// Finds activities that are correlated in their occurrence patterns.
pub fn correlation_miner(log: &EventLog, correlation_threshold: f64) -> CorrelationMinerResult {
    let mut correlations: HashMap<(String, String), f64> = HashMap::new();
    let activities = log.activities();

    // Count activity occurrences per case
    let mut case_vectors: Vec<HashMap<String, usize>> = Vec::new();
    for trace in &log.traces {
        let mut vector: HashMap<String, usize> = HashMap::new();
        for event in &trace.events {
            *vector.entry(event.activity.clone()).or_insert(0) += 1;
        }
        case_vectors.push(vector);
    }

    // Calculate pairwise correlations
    for i in 0..activities.len() {
        for j in (i + 1)..activities.len() {
            let act1 = &activities[i];
            let act2 = &activities[j];

            let correlation = calculate_correlation(&case_vectors, act1, act2);
            correlations.insert((act1.clone(), act2.clone()), correlation);
        }
    }

    // Find high correlations
    let mut high_correlation: Vec<(String, String, f64)> = correlations
        .iter()
        .filter(|(_, &corr)| corr >= correlation_threshold)
        .map(|((a1, a2), corr)| (a1.clone(), a2.clone(), *corr))
        .collect();

    high_correlation.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

    CorrelationMinerResult {
        correlations,
        high_correlation,
    }
}

/// Calculate correlation between two activities
fn calculate_correlation(case_vectors: &[HashMap<String, usize>], act1: &str, act2: &str) -> f64 {
    let n = case_vectors.len() as f64;

    // Get counts for each activity
    let counts1: Vec<f64> = case_vectors
        .iter()
        .map(|v| *v.get(act1).unwrap_or(&0) as f64)
        .collect();
    let counts2: Vec<f64> = case_vectors
        .iter()
        .map(|v| *v.get(act2).unwrap_or(&0) as f64)
        .collect();

    // Calculate means
    let mean1 = counts1.iter().sum::<f64>() / n;
    let mean2 = counts2.iter().sum::<f64>() / n;

    // Calculate covariance and variances
    let mut covariance = 0.0;
    let mut var1 = 0.0;
    let mut var2 = 0.0;

    for i in 0..case_vectors.len() {
        let diff1 = counts1[i] - mean1;
        let diff2 = counts2[i] - mean2;
        covariance += diff1 * diff2;
        var1 += diff1 * diff1;
        var2 += diff2 * diff2;
    }

    // Calculate correlation
    let denominator = (var1 * var2).sqrt();
    if denominator > 0.0 {
        covariance / denominator
    } else {
        0.0
    }
}

// ========================================================================
// MISSING WRAPPER FUNCTIONS FOR PYTHON PM4PY PARITY
// ========================================================================

/// Wrapper: Discover DECLARE constraints from event log
///
/// This is a convenience wrapper for DeclareMiner.
pub fn discover_declare(log: &EventLog) -> crate::discovery::declare_miner::DeclareModel {
    crate::discovery::declare_miner::DeclareMiner::new().discover(log)
}

/// Wrapper: Discover OCDFG (Object-Centric Directly-Follows Graph)
///
/// For standard event logs, this returns the same as DFG.
pub fn discover_ocdfg(log: &EventLog) -> DirectlyFollowsGraph {
    DirectlyFollowsGraph::from_log(log)
}

/// Discover a POWL (Partially Ordered Workflow Language) model from an event log.
///
/// Returns a [`crate::models::POWLModel`] whose `activities` list contains every
/// unique activity seen in the log, `partial_order` lists the DFG edges as
/// index pairs, `choice_groups` lists pairs that never co-occur (XOR exclusive),
/// and `parallel_groups` lists pairs observed in both orderings (concurrent).
pub fn discover_powl(log: &EventLog) -> crate::models::POWLModel {
    use crate::models::POWLModel;
    use std::collections::{HashMap, HashSet};

    // Collect all unique activities (sorted for determinism)
    let mut activity_set: HashSet<String> = HashSet::new();
    for trace in &log.traces {
        for event in &trace.events {
            activity_set.insert(event.activity.clone());
        }
    }
    let mut activities: Vec<String> = activity_set.into_iter().collect();
    activities.sort();

    // Index map: activity name → index in `activities`
    let act_index: HashMap<String, usize> = activities
        .iter()
        .enumerate()
        .map(|(i, a)| (a.clone(), i))
        .collect();

    // Build partial order from DFG edges (deduplicated)
    let mut edge_set: HashSet<(usize, usize)> = HashSet::new();
    for trace in &log.traces {
        for window in trace.events.windows(2) {
            if let (Some(&fi), Some(&ti)) = (
                act_index.get(&window[0].activity),
                act_index.get(&window[1].activity),
            ) {
                edge_set.insert((fi, ti));
            }
        }
    }
    let mut partial_order: Vec<(usize, usize)> = edge_set.into_iter().collect();
    partial_order.sort();

    // --- Choice groups: pairs that NEVER appear together in any trace ---
    let mut co_occur: HashSet<(usize, usize)> = HashSet::new();
    for trace in &log.traces {
        let trace_acts: Vec<usize> = trace
            .events
            .iter()
            .filter_map(|e| act_index.get(&e.activity).copied())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        for i in 0..trace_acts.len() {
            for j in (i + 1)..trace_acts.len() {
                let pair = (
                    trace_acts[i].min(trace_acts[j]),
                    trace_acts[i].max(trace_acts[j]),
                );
                co_occur.insert(pair);
            }
        }
    }
    let n = activities.len();
    let mut choice_groups: Vec<Vec<usize>> = Vec::new();
    for i in 0..n {
        for j in (i + 1)..n {
            if !co_occur.contains(&(i, j)) {
                choice_groups.push(vec![i, j]);
            }
        }
    }

    // --- Parallel groups: pairs seen in both orderings A→B and B→A ---
    let mut order_forward: HashSet<(usize, usize)> = HashSet::new();
    let mut order_backward: HashSet<(usize, usize)> = HashSet::new();
    for trace in &log.traces {
        for window in trace.events.windows(2) {
            if let (Some(&fi), Some(&ti)) = (
                act_index.get(&window[0].activity),
                act_index.get(&window[1].activity),
            ) {
                order_forward.insert((fi, ti));
                order_backward.insert((ti, fi));
            }
        }
    }
    let mut parallel_groups: Vec<Vec<usize>> = order_forward
        .iter()
        .filter(|(a, b)| order_backward.contains(&(*a, *b)))
        .map(|&(a, b)| vec![a.min(b), a.max(b)])
        .collect::<HashSet<Vec<usize>>>()
        .into_iter()
        .collect();
    parallel_groups.sort();

    POWLModel {
        activities,
        partial_order,
        choice_groups,
        parallel_groups,
    }
}
