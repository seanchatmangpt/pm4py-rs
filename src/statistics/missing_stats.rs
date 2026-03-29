//! Missing statistics functions for Python pm4py parity
//!
//! These functions were identified as missing through Chicago TDD testing.

use crate::log::EventLog;
use crate::models::PetriNet;
use crate::ocpm::ObjectCentricEventLog;
use std::collections::HashMap;

/// Split log into train and test sets
///
/// Randomly splits the event log into training and testing sets
/// based on the provided ratio.
pub fn split_train_test(log: &EventLog, ratio: f64) -> (EventLog, EventLog) {
    let total_traces = log.traces.len();
    let train_size = (total_traces as f64 * ratio).round() as usize;

    let mut train_log = EventLog::new();
    let mut test_log = EventLog::new();

    for (i, trace) in log.traces.iter().enumerate() {
        if i < train_size {
            train_log.traces.push(trace.clone());
        } else {
            test_log.traces.push(trace.clone());
        }
    }

    (train_log, test_log)
}

/// Convert event log to DataFrame format
///
/// Returns a representation of the log as tabular data.
pub fn convert_to_dataframe(log: &EventLog) -> Vec<HashMap<String, String>> {
    let mut rows = Vec::new();

    for trace in &log.traces {
        for event in &trace.events {
            let mut row = HashMap::new();

            // Add case-level attributes
            for (key, value) in &trace.attributes {
                row.insert(format!("case:{}", key), value.clone());
            }

            // Add event attributes
            row.insert("activity".to_string(), event.activity.clone());
            row.insert("timestamp".to_string(), event.timestamp.to_rfc3339());
            row.insert("case_id".to_string(), trace.id.clone());

            if let Some(ref resource) = event.resource {
                row.insert("resource".to_string(), resource.clone());
            }

            // Add other event attributes
            for (key, value) in &event.attributes {
                row.insert(key.clone(), value.clone());
            }

            rows.push(row);
        }
    }

    rows
}

/// Filter log by case performance (duration)
///
/// Returns only traces whose duration falls within the specified range.
pub fn filter_case_performance(log: &EventLog, min_duration: i64, max_duration: i64) -> EventLog {
    let mut filtered = EventLog::new();
    filtered.attributes = log.attributes.clone();

    for trace in &log.traces {
        if let (Some(first), Some(last)) = (trace.events.first(), trace.events.last()) {
            let duration = last
                .timestamp
                .signed_duration_since(first.timestamp)
                .num_milliseconds();
            if duration >= min_duration && duration <= max_duration {
                filtered.traces.push(trace.clone());
            }
        }
    }

    filtered
}

/// Extract ML features from event log
///
/// Returns a feature matrix suitable for machine learning.
pub fn extract_features_dataframe(log: &EventLog) -> Vec<Vec<f64>> {
    let mut features = Vec::new();

    for trace in &log.traces {
        let mut trace_features = Vec::new();

        // Basic features
        trace_features.push(trace.events.len() as f64); // trace length

        // Activity counts
        let mut activity_counts: HashMap<String, usize> = HashMap::new();
        for event in &trace.events {
            *activity_counts.entry(event.activity.clone()).or_insert(0) += 1;
        }

        // Add top 5 most frequent activities as features
        let mut counts: Vec<_> = activity_counts.iter().collect();
        counts.sort_by(|a, b| b.1.cmp(a.1));

        for i in 0..5 {
            if i < counts.len() {
                trace_features.push(*counts[i].1 as f64);
            } else {
                trace_features.push(0.0);
            }
        }

        features.push(trace_features);
    }

    features
}

/// Extract features from OCEL.
///
/// Returns one feature vector per object type:
/// `[num_objects, num_events_involving_type, num_unique_activities, avg_duration_secs]`
pub fn extract_ocel_features(ocel: &ObjectCentricEventLog) -> Vec<Vec<f64>> {
    use std::collections::HashSet;

    let mut features: Vec<Vec<f64>> = Vec::new();

    for obj_type in &ocel.object_types {
        // Objects of this type
        let type_obj_ids: HashSet<&str> = ocel
            .objects
            .values()
            .filter(|o| &o.object_type == obj_type)
            .map(|o| o.id.as_str())
            .collect();
        let num_objects = type_obj_ids.len() as f64;

        // Events involving any object of this type
        let mut event_count = 0usize;
        let mut unique_activities: HashSet<&str> = HashSet::new();
        for mapping in &ocel.event_object_mappings {
            if mapping
                .object_ids
                .iter()
                .any(|id| type_obj_ids.contains(id.as_str()))
            {
                event_count += 1;
                if let Some((activity, _, _)) = ocel.events.get(&mapping.event_id) {
                    unique_activities.insert(activity.as_str());
                }
            }
        }

        // Average lifecycle duration for objects with a known end_time
        let objs_of_type: Vec<_> = ocel
            .objects
            .values()
            .filter(|o| &o.object_type == obj_type)
            .collect();
        let with_end: Vec<_> = objs_of_type
            .iter()
            .filter_map(|o| o.end_time.map(|end| (end - o.creation_time).num_seconds()))
            .collect();
        let avg_duration_secs = if with_end.is_empty() {
            0.0
        } else {
            with_end.iter().sum::<i64>() as f64 / with_end.len() as f64
        };

        features.push(vec![
            num_objects,
            event_count as f64,
            unique_activities.len() as f64,
            avg_duration_secs,
        ]);
    }

    if features.is_empty() {
        // Fallback when no object types are defined
        features.push(vec![
            ocel.objects.len() as f64,
            ocel.events.len() as f64,
            ocel.object_types.len() as f64,
            0.0,
        ]);
    }

    features
}

/// Calculate cosine similarity between activity-frequency embeddings of two logs.
///
/// Each log is represented as a normalised activity-frequency vector (count /
/// total events). Returns 1.0 for identical logs and 0.0 for completely
/// disjoint activity sets.
pub fn embeddings_similarity(log1: &EventLog, log2: &EventLog) -> f64 {
    let total1 = log1
        .traces
        .iter()
        .map(|t| t.events.len())
        .sum::<usize>()
        .max(1) as f64;
    let total2 = log2
        .traces
        .iter()
        .map(|t| t.events.len())
        .sum::<usize>()
        .max(1) as f64;

    let mut freq1: HashMap<String, f64> = HashMap::new();
    let mut freq2: HashMap<String, f64> = HashMap::new();

    for trace in &log1.traces {
        for event in &trace.events {
            *freq1.entry(event.activity.clone()).or_insert(0.0) += 1.0 / total1;
        }
    }
    for trace in &log2.traces {
        for event in &trace.events {
            *freq2.entry(event.activity.clone()).or_insert(0.0) += 1.0 / total2;
        }
    }

    let dot: f64 = freq1
        .iter()
        .map(|(k, v)| v * freq2.get(k).copied().unwrap_or(0.0))
        .sum();
    let mag1: f64 = freq1.values().map(|v| v * v).sum::<f64>().sqrt();
    let mag2: f64 = freq2.values().map(|v| v * v).sum::<f64>().sqrt();

    if mag1 == 0.0 || mag2 == 0.0 {
        0.0
    } else {
        (dot / (mag1 * mag2)).min(1.0)
    }
}

/// Calculate structural similarity between two Petri nets.
///
/// Returns the average of:
/// - Jaccard similarity over labeled transition sets
/// - Arc-density similarity: `1 - |d1 - d2| / (d1 + d2 + 1)`
///
/// Returns 1.0 for structurally identical nets.
pub fn structural_similarity(net1: &PetriNet, net2: &PetriNet) -> f64 {
    use std::collections::HashSet;

    let labels1: HashSet<&str> = net1
        .transitions
        .iter()
        .filter_map(|t| t.label.as_deref())
        .collect();
    let labels2: HashSet<&str> = net2
        .transitions
        .iter()
        .filter_map(|t| t.label.as_deref())
        .collect();

    let intersection = labels1.intersection(&labels2).count() as f64;
    let union = labels1.union(&labels2).count() as f64;
    let jaccard = if union == 0.0 {
        1.0
    } else {
        intersection / union
    };

    let density1 =
        net1.arcs.len() as f64 / (net1.transitions.len() + net1.places.len()).max(1) as f64;
    let density2 =
        net2.arcs.len() as f64 / (net2.transitions.len() + net2.places.len()).max(1) as f64;
    let density_sim = 1.0 - (density1 - density2).abs() / (density1 + density2 + 1.0);

    (jaccard + density_sim) / 2.0
}
