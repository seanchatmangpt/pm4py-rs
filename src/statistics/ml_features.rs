//! Machine Learning Feature Extraction
//!
//! Extract features from event logs for ML-based process mining tasks.

use crate::log::EventLog;
use std::collections::{HashMap, HashSet};

/// Extract features from a trace for ML purposes
#[derive(Debug, Clone)]
pub struct TraceFeatures {
    pub trace_id: String,
    pub length: usize,
    pub activities: Vec<String>,
    pub activity_counts: HashMap<String, usize>,
    pub duration_seconds: Option<i64>,
    pub start_activity: Option<String>,
    pub end_activity: Option<String>,
    pub resource_set: HashSet<String>,
}

impl TraceFeatures {
    pub fn new(trace_id: String) -> Self {
        Self {
            trace_id,
            length: 0,
            activities: Vec::new(),
            activity_counts: HashMap::new(),
            duration_seconds: None,
            start_activity: None,
            end_activity: None,
            resource_set: HashSet::new(),
        }
    }
}

/// Extract ML features from event log
pub fn extract_features(log: &EventLog) -> Vec<TraceFeatures> {
    let mut features = Vec::new();

    for trace in &log.traces {
        let mut tf = TraceFeatures::new(trace.id.clone());

        tf.length = trace.events.len();
        tf.activities = trace.events.iter().map(|e| e.activity.clone()).collect();

        // Count activities
        for event in &trace.events {
            *tf.activity_counts
                .entry(event.activity.clone())
                .or_insert(0) += 1;
        }

        // Duration
        if let (Some(first), Some(last)) = (trace.events.first(), trace.events.last()) {
            tf.duration_seconds = Some(
                last.timestamp
                    .signed_duration_since(first.timestamp)
                    .num_seconds(),
            );
        }

        // Start and end activities
        tf.start_activity = trace.events.first().map(|e| e.activity.clone());
        tf.end_activity = trace.events.last().map(|e| e.activity.clone());

        // Resources
        for event in &trace.events {
            if let Some(ref resource) = event.resource {
                tf.resource_set.insert(resource.clone());
            }
        }

        features.push(tf);
    }

    features
}

/// Get feature names for ML training
pub fn get_feature_names() -> Vec<String> {
    vec![
        "trace_length".to_string(),
        "num_unique_activities".to_string(),
        "duration_seconds".to_string(),
        "num_resources".to_string(),
        "start_activity".to_string(),
        "end_activity".to_string(),
    ]
}

/// Convert trace features to a feature vector (for ML algorithms)
pub fn features_to_vector(features: &TraceFeatures, all_activities: &[String]) -> Vec<f64> {
    // Basic features
    let mut vector = vec![
        features.length as f64,
        features.activities.len() as f64,
        features.duration_seconds.unwrap_or(0) as f64,
        features.resource_set.len() as f64,
    ];

    // Activity presence (one-hot encoding)
    for activity in all_activities {
        vector.push(if features.activities.contains(activity) {
            1.0
        } else {
            0.0
        });
    }

    // Activity counts
    for activity in all_activities {
        vector.push(*features.activity_counts.get(activity).unwrap_or(&0) as f64);
    }

    vector
}

/// Get all unique activities from the log (for feature encoding)
pub fn get_all_activities(log: &EventLog) -> Vec<String> {
    log.activities()
}

/// Get string attributes from log (for categorical features)
pub fn get_str_attributes(log: &EventLog) -> Vec<String> {
    let mut attributes = HashSet::new();

    for trace in &log.traces {
        for (key, value) in &trace.attributes {
            // Only include string attributes that have few unique values
            if value.len() < 50 {
                attributes.insert(key.clone());
            }
        }

        for event in &trace.events {
            for (key, value) in &event.attributes {
                if value.len() < 50 {
                    attributes.insert(format!("event:{}", key));
                }
            }
        }
    }

    let mut result: Vec<String> = attributes.into_iter().collect();
    result.sort();
    result
}

/// Get numeric attributes from log (for numerical features)
pub fn get_numeric_attributes(log: &EventLog) -> Vec<String> {
    let mut attributes = HashSet::new();

    for trace in &log.traces {
        for (key, value) in &trace.attributes {
            // Try to parse as numeric
            if value.parse::<f64>().is_ok() {
                attributes.insert(key.clone());
            }
        }

        for event in &trace.events {
            for (key, value) in &event.attributes {
                if value.parse::<f64>().is_ok() {
                    attributes.insert(format!("event:{}", key));
                }
            }
        }
    }

    let mut result: Vec<String> = attributes.into_iter().collect();
    result.sort();
    result
}

/// Extract numeric attribute values for a specific attribute
pub fn get_numeric_attribute_values(log: &EventLog, attribute_name: &str) -> Vec<f64> {
    let mut values = Vec::new();

    // Check if it's an event attribute
    let is_event = attribute_name.starts_with("event:");
    let actual_name = if is_event {
        attribute_name
            .strip_prefix("event:")
            .unwrap_or(attribute_name)
    } else {
        attribute_name
    };

    for trace in &log.traces {
        if is_event {
            for event in &trace.events {
                if let Some(value) = event.attributes.get(actual_name) {
                    if let Ok(num) = value.parse::<f64>() {
                        values.push(num);
                    }
                }
            }
        } else {
            if let Some(value) = trace.attributes.get(actual_name) {
                if let Ok(num) = value.parse::<f64>() {
                    values.push(num);
                }
            }
        }
    }

    values
}

/// Extract string attribute values for a specific attribute
pub fn get_str_attribute_values(log: &EventLog, attribute_name: &str) -> Vec<String> {
    let mut values = HashSet::new();

    // Check if it's an event attribute
    let is_event = attribute_name.starts_with("event:");
    let actual_name = if is_event {
        attribute_name
            .strip_prefix("event:")
            .unwrap_or(attribute_name)
    } else {
        attribute_name
    };

    for trace in &log.traces {
        if is_event {
            for event in &trace.events {
                if let Some(value) = event.attributes.get(actual_name) {
                    values.insert(value.clone());
                }
            }
        } else {
            if let Some(value) = trace.attributes.get(actual_name) {
                values.insert(value.clone());
            }
        }
    }

    let mut result: Vec<String> = values.into_iter().collect();
    result.sort();
    result
}

/// Encode categorical features as one-hot vectors
pub fn one_hot_encode(value: &str, all_values: &[String]) -> Vec<f64> {
    all_values
        .iter()
        .map(|v| if v == value { 1.0 } else { 0.0 })
        .collect()
}

/// Create a feature matrix for ML training
pub fn create_feature_matrix(log: &EventLog) -> (Vec<Vec<f64>>, Vec<String>) {
    let features_list = extract_features(log);
    let all_activities = get_all_activities(log);
    let feature_names = get_feature_names();

    let mut matrix = Vec::new();

    for features in &features_list {
        let vector = features_to_vector(features, &all_activities);
        matrix.push(vector);
    }

    (matrix, feature_names)
}

/// Split data into training and test sets
pub fn train_test_split<T>(data: &[T], train_ratio: f64) -> (Vec<T>, Vec<T>)
where
    T: Clone,
{
    let split_idx = (data.len() as f64 * train_ratio) as usize;
    let train: Vec<T> = data[..split_idx].to_vec();
    let test: Vec<T> = data[split_idx..].to_vec();
    (train, test)
}

/// Normalize numeric features to [0, 1] range
pub fn normalize_features(features: &mut [Vec<f64>]) {
    if features.is_empty() || features[0].is_empty() {
        return;
    }

    let num_features = features[0].len();

    for feature_idx in 0..num_features {
        // Find min and max for this feature
        let min_val = features
            .iter()
            .map(|row| row[feature_idx])
            .fold(f64::INFINITY, f64::min);
        let max_val = features
            .iter()
            .map(|row| row[feature_idx])
            .fold(f64::NEG_INFINITY, f64::max);

        let range = max_val - min_val;

        if range > 0.0 {
            for row in features.iter_mut() {
                row[feature_idx] = (row[feature_idx] - min_val) / range;
            }
        }
    }
}
