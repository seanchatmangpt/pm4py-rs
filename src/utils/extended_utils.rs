//! Extended Utility Functions
//!
//! Additional utility functions for log manipulation and analysis.

use crate::log::EventLog;
use std::collections::{HashMap, HashSet};

/// Project event log on an event attribute
///
/// Creates a projection of the log where events are grouped by attribute value.
pub fn project_on_event_attribute(
    log: &EventLog,
    attribute_name: &str,
) -> HashMap<String, EventLog> {
    let mut projected: HashMap<String, EventLog> = HashMap::new();

    for trace in &log.traces {
        for event in &trace.events {
            // Get attribute value
            let key = if let Some(value) = event.get_attribute(attribute_name) {
                value.to_string()
            } else {
                "UNKNOWN".to_string()
            };

            // Add event to projected log
            let entry = projected.entry(key).or_default();
            entry.attributes = log.attributes.clone();

            // Find or create trace for this case
            let trace_entry = entry.traces.iter_mut().find(|t| t.id == trace.id);
            if let Some(t) = trace_entry {
                t.add_event(event.clone());
            } else {
                let mut new_trace = trace.clone();
                new_trace.events = vec![event.clone()];
                entry.add_trace(new_trace);
            }
        }
    }

    projected
}

/// Get all activity labels from event log
///
/// Returns a sorted list of unique activity names.
pub fn get_activity_labels(log: &EventLog) -> Vec<String> {
    let mut labels: HashSet<String> = HashSet::new();

    for trace in &log.traces {
        for event in &trace.events {
            labels.insert(event.activity.clone());
        }
    }

    let mut result: Vec<String> = labels.into_iter().collect();
    result.sort();
    result
}

/// Convert event log to time intervals
///
/// Converts events to (activity, start_time, end_time) intervals.
pub fn convert_log_to_time_intervals(
    log: &EventLog,
) -> Vec<(
    String,
    chrono::DateTime<chrono::Utc>,
    chrono::DateTime<chrono::Utc>,
)> {
    let mut intervals = Vec::new();

    for trace in &log.traces {
        for (i, event) in trace.events.iter().enumerate() {
            // Use current event timestamp as start
            let start = event.timestamp;

            // Use next event timestamp as end, or current if last event
            let end = if i + 1 < trace.events.len() {
                trace.events[i + 1].timestamp
            } else {
                event.timestamp
            };

            intervals.push((event.activity.clone(), start, end));
        }
    }

    intervals
}

/// Cluster event log traces
///
/// Groups traces into clusters based on similarity.
pub fn cluster_log(log: &EventLog, num_clusters: usize) -> Vec<(usize, EventLog)> {
    if log.is_empty() || num_clusters == 0 {
        return Vec::new();
    }

    // Simple clustering: group by trace length
    let mut by_length: HashMap<usize, Vec<&crate::log::Trace>> = HashMap::new();

    for trace in &log.traces {
        by_length.entry(trace.len()).or_default().push(trace);
    }

    // Convert lengths to cluster IDs
    let mut lengths: Vec<usize> = by_length.keys().cloned().collect();
    lengths.sort();

    // Assign cluster IDs based on length buckets
    let bucket_size = (lengths.len() as f64 / num_clusters as f64).ceil() as usize;

    let mut result = Vec::new();
    for (cluster_idx, length_chunk) in lengths.chunks(bucket_size).enumerate() {
        for &length in length_chunk {
            if let Some(traces) = by_length.get(&length) {
                let mut cluster_log = EventLog::new();
                cluster_log.attributes = log.attributes.clone();

                for trace in traces {
                    cluster_log.add_trace((*trace).clone());
                }

                result.push((cluster_idx, cluster_log));
            }
        }
    }

    result
}

/// Behavioral similarity between two traces
///
/// Calculates similarity based on shared activities and sequences.
pub fn behavioral_similarity(trace1: &[crate::log::Event], trace2: &[crate::log::Event]) -> f64 {
    if trace1.is_empty() && trace2.is_empty() {
        return 1.0;
    }

    if trace1.is_empty() || trace2.is_empty() {
        return 0.0;
    }

    // Get activity sets
    let acts1: HashSet<&str> = trace1.iter().map(|e| e.activity.as_str()).collect();
    let acts2: HashSet<&str> = trace2.iter().map(|e| e.activity.as_str()).collect();

    // Jaccard similarity of activity sets
    let intersection: HashSet<_> = acts1.intersection(&acts2).collect();
    let union: HashSet<_> = acts1.union(&acts2).collect();

    if union.is_empty() {
        return 0.0;
    }

    let activity_similarity = intersection.len() as f64 / union.len() as f64;

    // Sequence similarity (simplified)
    let mut shared_pairs = 0;
    let mut total_pairs = 0;

    for window in trace1.windows(2) {
        total_pairs += 1;
        let pair = (&window[0].activity, &window[1].activity);

        for window2 in trace2.windows(2) {
            if (&window2[0].activity, &window2[1].activity) == pair {
                shared_pairs += 1;
                break;
            }
        }
    }

    for _window in trace2.windows(2) {
        total_pairs += 1;
    }

    let sequence_similarity = if total_pairs > 0 {
        shared_pairs as f64 / total_pairs as f64
    } else {
        1.0
    };

    // Combined similarity
    (activity_similarity + sequence_similarity) / 2.0
}

/// Calculate behavioral similarity matrix for event log
///
/// Returns a matrix of pairwise similarities between traces.
#[allow(clippy::needless_range_loop)]
pub fn behavioral_similarity_matrix(log: &EventLog) -> Vec<Vec<f64>> {
    let n = log.traces.len();
    let mut matrix = vec![vec![0.0; n]; n];

    for i in 0..n {
        for j in i..n {
            let similarity = behavioral_similarity(&log.traces[i].events, &log.traces[j].events);
            matrix[i][j] = similarity;
            matrix[j][i] = similarity;
        }
    }

    matrix
}

/// Embedding similarity (simplified)
///
/// Calculates similarity based on simple event frequency embeddings.
pub fn embeddings_similarity(log1: &EventLog, log2: &EventLog) -> f64 {
    // Create simple frequency-based embeddings
    let embed1 = create_frequency_embedding(log1);
    let embed2 = create_frequency_embedding(log2);

    // Cosine similarity
    let mut dot_product = 0.0;
    let mut norm1 = 0.0;
    let mut norm2 = 0.0;

    for (act, &count1) in &embed1 {
        let count2 = embed2.get(act).copied().unwrap_or(0.0);
        dot_product += count1 * count2;
        norm1 += count1 * count1;
    }

    for &count2 in embed2.values() {
        norm2 += count2 * count2;
    }

    let denominator = norm1.sqrt() * norm2.sqrt();
    if denominator > 0.0 {
        dot_product / denominator
    } else {
        0.0
    }
}

/// Create frequency-based embedding for event log
fn create_frequency_embedding(log: &EventLog) -> HashMap<String, f64> {
    let mut embedding = HashMap::new();
    let mut total_events = 0.0;

    for trace in &log.traces {
        for event in &trace.events {
            *embedding.entry(event.activity.clone()).or_insert(0.0) += 1.0;
            total_events += 1.0;
        }
    }

    // Normalize
    if total_events > 0.0 {
        for value in embedding.values_mut() {
            *value /= total_events;
        }
    }

    embedding
}

/// Generalization through token-based replay
///
/// Measures how well a model generalizes to unseen data.
pub fn generalization_tbr(log: &EventLog, dfg: &HashMap<(String, String), usize>) -> f64 {
    if dfg.is_empty() {
        return 1.0;
    }

    // Count edges in log
    let mut log_edges: HashSet<(String, String)> = HashSet::new();
    let mut total_edges = 0;

    for trace in &log.traces {
        for window in trace.events.windows(2) {
            let edge = (window[0].activity.clone(), window[1].activity.clone());
            log_edges.insert(edge);
            total_edges += 1;
        }
    }

    if total_edges == 0 {
        return 1.0;
    }

    // Calculate generalization
    let model_edges = dfg.len();
    let edge_coverage = log_edges.len() as f64 / model_edges.max(1) as f64;

    // Penalize over-specified models
    let simplicity = if model_edges > total_edges {
        total_edges as f64 / model_edges as f64
    } else {
        1.0
    };

    (edge_coverage + simplicity) / 2.0
}
