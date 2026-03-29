//! Object-Centric Directly-Follows Graph (OCDFG) discovery.
//!
//! Implements Connection 2 from "No AI Without PI": process models feeding prescriptive AI.
//!
//! The OCDFG is discovered per-object-type, capturing frequency and performance
//! statistics for each directly-follows edge. This enables:
//! - Per-type bottleneck identification
//! - Variant analysis without the convergence/divergence artifacts of flattened logs
//! - Input to classical optimization (Connection 3)

use super::object_log::ObjectCentricEventLog;
use std::collections::BTreeMap;

/// Performance statistics for a directly-follows edge (source_activity → target_activity).
#[derive(Debug, Clone, PartialEq)]
pub struct EdgePerformance {
    /// Number of times this edge was observed.
    pub frequency: u64,
    /// Mean duration between source and target activity (seconds).
    pub mean_duration_secs: f64,
    /// Median duration between source and target activity (seconds).
    pub median_duration_secs: f64,
    /// Minimum observed duration (seconds).
    pub min_duration_secs: f64,
    /// Maximum observed duration (seconds).
    pub max_duration_secs: f64,
}

/// Frequency + type information for a single activity node.
#[derive(Debug, Clone, PartialEq)]
pub struct ActivityStats {
    /// How many times this activity occurred for this object type.
    pub frequency: u64,
    /// The object type this node belongs to.
    pub object_type: String,
}

/// Object-Centric Directly-Follows Graph.
///
/// `edges[object_type][(from_activity, to_activity)] = EdgePerformance`
/// `activity_stats[object_type][activity_name] = ActivityStats`
#[derive(Debug, Clone)]
pub struct ObjectCentricDFG {
    /// Per-object-type directly-follows edges with performance stats.
    pub edges: BTreeMap<String, BTreeMap<(String, String), EdgePerformance>>,
    /// Per-object-type activity frequency counts.
    pub activity_stats: BTreeMap<String, BTreeMap<String, ActivityStats>>,
}

impl ObjectCentricDFG {
    fn new() -> Self {
        Self {
            edges: BTreeMap::new(),
            activity_stats: BTreeMap::new(),
        }
    }

    /// Total number of directly-follows edges across all object types.
    pub fn total_edge_count(&self) -> usize {
        self.edges.values().map(|m| m.len()).sum()
    }

    /// Returns all object types present in this DFG.
    pub fn object_types(&self) -> Vec<&str> {
        self.edges.keys().map(|s| s.as_str()).collect()
    }
}

/// Discover an Object-Centric DFG from an `ObjectCentricEventLog`.
///
/// Algorithm:
/// 1. For each object in the log, retrieve its sorted lifecycle (via `get_lifecycle_for_object`).
/// 2. Slide a window of 2 consecutive events to form directly-follows pairs.
/// 3. Accumulate frequency counts and duration samples per object type.
/// 4. Compute mean/median/min/max from the collected duration samples.
pub fn discover_ocdfg(log: &ObjectCentricEventLog) -> ObjectCentricDFG {
    let mut dfg = ObjectCentricDFG::new();

    // Collect raw duration samples: (object_type, from_act, to_act) → Vec<f64 seconds>
    let mut duration_samples: BTreeMap<(String, String, String), Vec<f64>> = BTreeMap::new();

    for (object_id, object) in &log.objects {
        let type_name = object.object_type.name.clone();
        let lifecycle = log.get_lifecycle_for_object(object_id);

        // Activity frequency
        for (_, activity, _) in &lifecycle {
            let type_stats = dfg.activity_stats.entry(type_name.clone()).or_default();
            let stats = type_stats
                .entry(activity.clone())
                .or_insert_with(|| ActivityStats {
                    frequency: 0,
                    object_type: type_name.clone(),
                });
            stats.frequency += 1;
        }

        // Directly-follows edges — windows of 2
        for window in lifecycle.windows(2) {
            let (_, from_act, from_ts) = &window[0];
            let (_, to_act, to_ts) = &window[1];

            let duration_secs = (*to_ts - *from_ts).num_milliseconds().max(0) as f64 / 1000.0;

            let key = (type_name.clone(), from_act.clone(), to_act.clone());
            duration_samples.entry(key).or_default().push(duration_secs);
        }
    }

    // Build EdgePerformance from samples
    for ((type_name, from_act, to_act), mut samples) in duration_samples {
        let frequency = samples.len() as u64;
        let mean_duration_secs = samples.iter().sum::<f64>() / frequency as f64;
        let min_duration_secs = samples.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_duration_secs = samples.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let median_duration_secs = if samples.len() % 2 == 0 {
            (samples[samples.len() / 2 - 1] + samples[samples.len() / 2]) / 2.0
        } else {
            samples[samples.len() / 2]
        };

        let type_edges = dfg.edges.entry(type_name).or_default();
        type_edges.insert(
            (from_act, to_act),
            EdgePerformance {
                frequency,
                mean_duration_secs,
                median_duration_secs,
                min_duration_secs,
                max_duration_secs,
            },
        );
    }

    dfg
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ocpm::object_log::{
        EventToObjectMapping, Object, ObjectCentricEventLog, ObjectType,
    };
    use chrono::{Duration, Utc};
    use uuid::Uuid;

    fn make_two_event_log() -> ObjectCentricEventLog {
        let mut log = ObjectCentricEventLog::new();
        let t0 = Utc::now();
        let t1 = t0 + Duration::seconds(30);

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
    fn test_discover_ocdfg_produces_edge_for_two_event_log() {
        let log = make_two_event_log();
        let dfg = discover_ocdfg(&log);

        // Should have exactly one edge: create → approve for "order" type
        assert_eq!(dfg.total_edge_count(), 1);
        let order_edges = dfg.edges.get("order").expect("order edges must exist");
        assert!(
            order_edges.contains_key(&("create".to_string(), "approve".to_string())),
            "edge create→approve must exist"
        );
    }

    #[test]
    fn test_discover_ocdfg_edge_frequency_and_duration() {
        let log = make_two_event_log();
        let dfg = discover_ocdfg(&log);

        let edge = dfg
            .edges
            .get("order")
            .unwrap()
            .get(&("create".to_string(), "approve".to_string()))
            .unwrap();

        assert_eq!(edge.frequency, 1);
        // Duration should be ~30 seconds
        assert!(
            edge.mean_duration_secs >= 29.0 && edge.mean_duration_secs <= 31.0,
            "duration must be ~30s, got {}",
            edge.mean_duration_secs
        );
    }

    #[test]
    fn test_discover_ocdfg_activity_stats() {
        let log = make_two_event_log();
        let dfg = discover_ocdfg(&log);

        let type_stats = dfg.activity_stats.get("order").unwrap();
        assert_eq!(type_stats.get("create").unwrap().frequency, 1);
        assert_eq!(type_stats.get("approve").unwrap().frequency, 1);
    }

    #[test]
    fn test_discover_ocdfg_empty_log() {
        let log = ObjectCentricEventLog::new();
        let dfg = discover_ocdfg(&log);
        assert_eq!(dfg.total_edge_count(), 0);
    }

    #[test]
    fn test_object_types_returns_all_types() {
        let log = make_two_event_log();
        let dfg = discover_ocdfg(&log);
        let types = dfg.object_types();
        assert!(types.contains(&"order"));
    }
}
