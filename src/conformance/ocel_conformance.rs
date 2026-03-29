//! OCEL (Object-Centric) Conformance Checking
//!
//! Conformance checking for object-centric event logs.

use crate::ocpm::ObjectCentricEventLog;
use std::collections::{HashMap, HashSet};

/// OCEL conformance result
#[derive(Debug, Clone)]
pub struct OCELConformanceResult {
    pub num_deviations: usize,
    pub deviation_details: Vec<String>,
    pub fitness: f64,
}

impl OCELConformanceResult {
    pub fn new() -> Self {
        Self {
            num_deviations: 0,
            deviation_details: Vec::new(),
            fitness: 1.0,
        }
    }

    pub fn calculate_fitness(&mut self, total_checks: usize) {
        if total_checks == 0 {
            self.fitness = 1.0;
        } else {
            self.fitness = 1.0 - (self.num_deviations as f64 / total_checks as f64);
        }
    }
}

impl Default for OCELConformanceResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Check OCEL lifecycle conformance
///
/// Verifies that objects follow expected lifecycle patterns.
pub fn check_ocel_lifecycle_conformance(
    ocel: &ObjectCentricEventLog,
    lifecycle_constraints: &HashMap<String, Vec<String>>,
) -> OCELConformanceResult {
    let mut result = OCELConformanceResult::new();
    let mut total_checks = 0;

    for object in ocel.objects.values() {
        let object_type = &object.object_type.name;
        total_checks += 1;

        if let Some(expected_stages) = lifecycle_constraints.get(object_type) {
            // Check if object follows expected lifecycle
            let events = ocel.get_lifecycle_for_object(&object.id);
            let actual_stages: Vec<String> = events.iter().map(|e| e.1.clone()).collect();

            // Verify stages match expected sequence
            if !actual_stages.is_empty() && !expected_stages.is_empty() {
                let first_stage = &actual_stages[0];
                if !expected_stages.contains(first_stage) {
                    result.num_deviations += 1;
                    result.deviation_details.push(format!(
                        "Object {} has invalid starting stage: {}",
                        object.id, first_stage
                    ));
                }
            }
        }
    }

    result.calculate_fitness(total_checks);
    result
}

/// Check OCEL relationship constraints
///
/// Verifies that relationships between objects satisfy constraints.
pub fn check_ocel_relationships(
    ocel: &ObjectCentricEventLog,
    relationship_constraints: &HashMap<(String, String), HashSet<String>>,
) -> OCELConformanceResult {
    let mut result = OCELConformanceResult::new();
    let mut total_checks = 0;

    for mapping in &ocel.event_object_mappings {
        let object_ids: Vec<&String> = mapping.object_ids.iter().collect();

        for i in 0..object_ids.len() {
            for j in (i + 1)..object_ids.len() {
                total_checks += 1;

                if let Some(obj1) = ocel.get_object(object_ids[i]) {
                    if let Some(obj2) = ocel.get_object(object_ids[j]) {
                        let type_pair =
                            (obj1.object_type.name.clone(), obj2.object_type.name.clone());

                        if let Some(allowed_relations) = relationship_constraints.get(&type_pair) {
                            // Check if this relationship is allowed
                            let relation_key = format!("{}->{}", obj1.id, obj2.id);
                            if !allowed_relations.contains(&relation_key) {
                                result.num_deviations += 1;
                                result.deviation_details.push(format!(
                                    "Invalid relationship: {} -> {}",
                                    obj1.id, obj2.id
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    result.calculate_fitness(total_checks);
    result
}

/// OCEL cardinality constraint
#[derive(Debug, Clone)]
pub struct CardinalityConstraint {
    pub object_type: String,
    pub relation_type: String,
    pub min: usize,
    pub max: usize,
}

/// Check OCEL cardinality constraints
pub fn check_ocel_cardinality(
    ocel: &ObjectCentricEventLog,
    constraints: &[CardinalityConstraint],
) -> OCELConformanceResult {
    let mut result = OCELConformanceResult::new();
    let mut total_checks = 0;

    for constraint in constraints {
        // Count objects of this type
        let count = ocel.get_objects_by_type(&constraint.object_type).len();
        total_checks += 1;

        if count < constraint.min || count > constraint.max {
            result.num_deviations += 1;
            result.deviation_details.push(format!(
                "Cardinality violation for type {}: {} (expected {}-{})",
                constraint.object_type, count, constraint.min, constraint.max
            ));
        }
    }

    result.calculate_fitness(total_checks);
    result
}

/// Get OCEL object lifecycle statistics
pub fn get_ocel_lifecycle_stats(
    ocel: &ObjectCentricEventLog,
) -> HashMap<String, HashMap<String, usize>> {
    let mut stats: HashMap<String, HashMap<String, usize>> = HashMap::new();

    for object in ocel.objects.values() {
        let type_name = &object.object_type.name;
        let lifecycle = object
            .lifecycle_stage
            .clone()
            .unwrap_or_else(|| "unknown".to_string());

        stats
            .entry(type_name.clone())
            .or_default()
            .entry(lifecycle)
            .or_insert(0);
    }

    stats
}

/// Validate OCEL event ordering
///
/// Ensures events are properly ordered by timestamp.
pub fn validate_ocel_event_ordering(ocel: &ObjectCentricEventLog) -> OCELConformanceResult {
    let mut result = OCELConformanceResult::new();
    let mut total_checks = 0;

    // Check each object's events are ordered
    for object in ocel.objects.values() {
        let events = ocel.get_lifecycle_for_object(&object.id);
        total_checks += 1;

        for window in events.windows(2) {
            if window[0].2 > window[1].2 {
                result.num_deviations += 1;
                result.deviation_details.push(format!(
                    "Event ordering violation for object {}: {:?} after {:?}",
                    object.id, window[0], window[1]
                ));
            }
        }
    }

    result.calculate_fitness(total_checks);
    result
}

/// OCEL temporal constraint
#[derive(Debug, Clone)]
pub struct OCELTemporalConstraint {
    pub from_activity: String,
    pub to_activity: String,
    pub min_duration_seconds: Option<i64>,
    pub max_duration_seconds: Option<i64>,
}

/// Check OCEL temporal constraints
pub fn check_ocel_temporal_constraints(
    ocel: &ObjectCentricEventLog,
    constraints: &[OCELTemporalConstraint],
) -> OCELConformanceResult {
    let mut result = OCELConformanceResult::new();
    let mut total_checks = 0;

    for constraint in constraints {
        for (activity, timestamp, _resource) in ocel.events.values() {
            if *activity == constraint.from_activity {
                // Find corresponding to_activity events
                for (other_activity, other_timestamp, _) in ocel.events.values() {
                    if *other_activity == constraint.to_activity {
                        total_checks += 1;
                        let duration = other_timestamp
                            .signed_duration_since(*timestamp)
                            .num_seconds();

                        if let Some(min_dur) = constraint.min_duration_seconds {
                            if duration < min_dur {
                                result.num_deviations += 1;
                                result.deviation_details.push(format!(
                                    "Duration too short: {} -> {}: {}s (min {}s)",
                                    constraint.from_activity,
                                    constraint.to_activity,
                                    duration,
                                    min_dur
                                ));
                            }
                        }

                        if let Some(max_dur) = constraint.max_duration_seconds {
                            if duration > max_dur {
                                result.num_deviations += 1;
                                result.deviation_details.push(format!(
                                    "Duration too long: {} -> {}: {}s (max {}s)",
                                    constraint.from_activity,
                                    constraint.to_activity,
                                    duration,
                                    max_dur
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    result.calculate_fitness(total_checks);
    result
}
