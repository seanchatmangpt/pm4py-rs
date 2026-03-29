/// Encoding utilities for feature extraction
use crate::log::EventLog;
use std::collections::HashMap;

/// One-hot encode activities
pub fn onehot_encode(log: &EventLog) -> (Vec<Vec<usize>>, Vec<String>) {
    let activities = log.activities();
    let activity_to_idx: HashMap<_, _> = activities
        .iter()
        .enumerate()
        .map(|(i, a)| (a.clone(), i))
        .collect();

    let mut encoded = Vec::new();
    for trace in &log.traces {
        let mut trace_encoded = vec![0; activities.len()];
        for event in &trace.events {
            if let Some(&idx) = activity_to_idx.get(&event.activity) {
                trace_encoded[idx] += 1;
            }
        }
        encoded.push(trace_encoded);
    }

    (encoded, activities)
}

/// Frequency-based encoding
pub fn frequency_encode(log: &EventLog) -> Vec<HashMap<String, usize>> {
    log.traces
        .iter()
        .map(|trace| {
            let mut freq = HashMap::new();
            for event in &trace.events {
                *freq.entry(event.activity.clone()).or_insert(0) += 1;
            }
            freq
        })
        .collect()
}

/// Sequence encoding - convert trace to indices
pub fn sequence_encode(log: &EventLog) -> (Vec<Vec<usize>>, Vec<String>) {
    let activities = log.activities();
    let activity_to_idx: HashMap<_, _> = activities
        .iter()
        .enumerate()
        .map(|(i, a)| (a.clone(), i))
        .collect();

    let mut encoded = Vec::new();
    for trace in &log.traces {
        let trace_encoded: Vec<usize> = trace
            .events
            .iter()
            .filter_map(|e| activity_to_idx.get(&e.activity).copied())
            .collect();
        encoded.push(trace_encoded);
    }

    (encoded, activities)
}

/// Create numeric feature matrix
pub fn feature_matrix(log: &EventLog) -> (Vec<Vec<f64>>, Vec<String>) {
    let activities = log.activities();
    let mut features = Vec::new();

    for trace in &log.traces {
        let mut row = vec![0.0; activities.len()];

        for event in &trace.events {
            if let Some(idx) = activities.iter().position(|a| a == &event.activity) {
                row[idx] += 1.0;
            }
        }

        // Normalize
        let sum: f64 = row.iter().sum();
        if sum > 0.0 {
            row.iter_mut().for_each(|v| *v /= sum);
        }

        features.push(row);
    }

    (features, activities)
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
        trace.add_event(Event::new("b", now));
        log.add_trace(trace);

        log
    }

    #[test]
    fn test_onehot_encode() {
        let log = create_test_log();
        let (encoded, activities) = onehot_encode(&log);

        assert_eq!(activities.len(), 2);
        assert_eq!(encoded.len(), 1);
    }

    #[test]
    fn test_sequence_encode() {
        let log = create_test_log();
        let (encoded, activities) = sequence_encode(&log);

        assert_eq!(activities.len(), 2);
        assert_eq!(encoded[0].len(), 2);
    }

    #[test]
    fn test_feature_matrix() {
        let log = create_test_log();
        let (matrix, activities) = feature_matrix(&log);

        assert_eq!(activities.len(), 2);
        assert_eq!(matrix.len(), 1);
    }
}
