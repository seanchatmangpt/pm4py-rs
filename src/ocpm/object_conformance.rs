//! Object-Centric Conformance Checking
//!
//! This module implements object-centric conformance checking algorithms that validate
//! whether object lifecycles and interactions conform to expected process models

use super::object_log::ObjectCentricEventLog;
use super::ocpm_miner::ObjectCentricPetriNet;
use std::collections::BTreeMap;

/// Result of conformance checking for an object
#[derive(Debug, Clone)]
pub struct ObjectConformanceResult {
    pub object_id: String,
    pub is_conformant: bool,
    pub fitness: f64,
    pub violations: usize,
    pub violation_details: Vec<String>,
}

/// Result of object interaction conformance
#[derive(Debug, Clone)]
pub struct InteractionConformanceResult {
    pub event_id: uuid::Uuid,
    pub activity: String,
    pub is_conformant: bool,
    pub objects_involved: Vec<String>,
    pub violations: Vec<String>,
}

/// Comprehensive object-centric conformance check result
#[derive(Debug, Clone)]
pub struct ObjectCentricConformanceResult {
    pub is_conformant: bool,
    pub fitness: f64,
    pub object_results: Vec<ObjectConformanceResult>,
    pub interaction_results: Vec<InteractionConformanceResult>,
    pub total_objects: usize,
    pub conformant_objects: usize,
    pub total_interactions: usize,
    pub conformant_interactions: usize,
    /// Per-object-type mean fitness score (average of all individual object fitness values).
    /// Enables Connection 4/RAG: confidence grounding per object type.
    pub fitness_by_object_type: BTreeMap<String, f64>,
}

/// Object-centric token replay conformance checker
#[derive(Debug)]
pub struct ObjectCentricTokenReplay {
    pub fitness_threshold: f64,
}

impl ObjectCentricTokenReplay {
    pub fn new(fitness_threshold: f64) -> Self {
        Self { fitness_threshold }
    }

    pub fn check(
        &self,
        log: &ObjectCentricEventLog,
        _net: &ObjectCentricPetriNet,
    ) -> ObjectCentricConformanceResult {
        let mut object_results = Vec::new();
        let mut interaction_results = Vec::new();

        for (object_id, object) in &log.objects {
            let result = self.check_object_lifecycle(log, object_id, object);
            object_results.push(result);
        }

        for mapping in &log.event_object_mappings {
            let result = self.check_interaction(log, mapping);
            interaction_results.push(result);
        }

        let total_objects = object_results.len();
        let conformant_objects = object_results.iter().filter(|r| r.is_conformant).count();
        let total_interactions = interaction_results.len();
        let conformant_interactions = interaction_results
            .iter()
            .filter(|r| r.is_conformant)
            .count();

        let object_fitness = if total_objects > 0 {
            conformant_objects as f64 / total_objects as f64
        } else {
            1.0
        };

        let interaction_fitness = if total_interactions > 0 {
            conformant_interactions as f64 / total_interactions as f64
        } else {
            1.0
        };

        let overall_fitness = (object_fitness + interaction_fitness) / 2.0;

        // Compute per-object-type mean fitness by grouping individual object results
        let mut type_fitness_samples: BTreeMap<String, Vec<f64>> = BTreeMap::new();
        for obj_result in &object_results {
            if let Some(obj) = log.get_object(&obj_result.object_id) {
                type_fitness_samples
                    .entry(obj.object_type.name.clone())
                    .or_default()
                    .push(obj_result.fitness);
            }
        }
        let fitness_by_object_type: BTreeMap<String, f64> = type_fitness_samples
            .into_iter()
            .map(|(type_name, samples)| {
                let mean = samples.iter().sum::<f64>() / samples.len() as f64;
                (type_name, mean)
            })
            .collect();

        ObjectCentricConformanceResult {
            is_conformant: overall_fitness >= self.fitness_threshold,
            fitness: overall_fitness,
            object_results,
            interaction_results,
            total_objects,
            conformant_objects,
            total_interactions,
            conformant_interactions,
            fitness_by_object_type,
        }
    }

    fn check_object_lifecycle(
        &self,
        log: &ObjectCentricEventLog,
        object_id: &str,
        object: &crate::ocpm::object_log::Object,
    ) -> ObjectConformanceResult {
        let mut violations = Vec::new();
        let mut violation_count = 0;

        let lifecycle = log.get_lifecycle_for_object(object_id);

        if !lifecycle.is_empty() && lifecycle[0].2 < object.creation_time {
            violations.push(format!(
                "Object {} has events before creation time",
                object_id
            ));
            violation_count += 1;
        }

        if let Some(end_time) = object.end_time {
            for (_, _, event_time) in &lifecycle {
                if *event_time > end_time {
                    violations.push(format!("Object {} has events after end time", object_id));
                    violation_count += 1;
                    break;
                }
            }
        }

        if let Some(state) = &object.state {
            if state != "initial" && lifecycle.is_empty() {
                violations.push(format!(
                    "Object {} has non-initial state but no lifecycle events",
                    object_id
                ));
                violation_count += 1;
            }
        }

        let fitness = if lifecycle.is_empty() {
            1.0
        } else {
            let valid_events = lifecycle.len() - violation_count.min(lifecycle.len());
            valid_events as f64 / lifecycle.len() as f64
        };

        ObjectConformanceResult {
            object_id: object_id.to_string(),
            is_conformant: violation_count == 0,
            fitness,
            violations: violation_count,
            violation_details: violations,
        }
    }

    fn check_interaction(
        &self,
        log: &ObjectCentricEventLog,
        mapping: &crate::ocpm::object_log::EventToObjectMapping,
    ) -> InteractionConformanceResult {
        let mut violations = Vec::new();

        let (activity, _timestamp, _resource) = log
            .events
            .get(&mapping.event_id)
            .map(|e| (e.0.clone(), e.1, e.2.clone()))
            .unwrap_or_else(|| ("unknown".to_string(), chrono::Utc::now(), None));

        for obj_id in &mapping.object_ids {
            if !log.objects.contains_key(obj_id) {
                violations.push(format!(
                    "Object {} referenced in event but not found in log",
                    obj_id
                ));
            }
        }

        for (obj_id, role_str) in mapping.object_roles.iter() {
            if role_str.is_empty() {
                violations.push(format!("Object {} has empty role in event", obj_id));
            }
        }

        InteractionConformanceResult {
            event_id: mapping.event_id,
            activity,
            is_conformant: violations.is_empty(),
            objects_involved: mapping.object_ids.iter().cloned().collect(),
            violations,
        }
    }
}

impl Default for ObjectCentricTokenReplay {
    fn default() -> Self {
        Self::new(0.8)
    }
}

/// Validates lifecycle consistency for object relationships
#[derive(Debug)]
pub struct ObjectRelationshipValidator {
    pub enforce_strict_ordering: bool,
}

impl ObjectRelationshipValidator {
    pub fn new(enforce_strict_ordering: bool) -> Self {
        Self {
            enforce_strict_ordering,
        }
    }

    pub fn validate_relationships(&self, log: &ObjectCentricEventLog) -> BTreeMap<String, usize> {
        let mut relationship_violations: BTreeMap<String, usize> = BTreeMap::new();

        for mapping in &log.event_object_mappings {
            let objects: Vec<_> = mapping.object_ids.iter().collect();

            for i in 0..objects.len() {
                for j in (i + 1)..objects.len() {
                    let obj_id_1 = objects[i];
                    let obj_id_2 = objects[j];

                    if let (Some(obj1), Some(obj2)) =
                        (log.get_object(obj_id_1), log.get_object(obj_id_2))
                    {
                        if self.enforce_strict_ordering && obj2.creation_time < obj1.creation_time {
                            let key = format!("{} before {}", obj_id_2, obj_id_1);
                            *relationship_violations.entry(key).or_insert(0) += 1;
                        }
                    }
                }
            }
        }

        relationship_violations
    }
}

impl Default for ObjectRelationshipValidator {
    fn default() -> Self {
        Self::new(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ocpm::object_log::{
        EventToObjectMapping, Object, ObjectCentricEventLog, ObjectType,
    };
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_object_conformance_result_creation() {
        let result = ObjectConformanceResult {
            object_id: "obj_1".to_string(),
            is_conformant: true,
            fitness: 1.0,
            violations: 0,
            violation_details: vec![],
        };

        assert_eq!(result.object_id, "obj_1");
        assert!(result.is_conformant);
        assert_eq!(result.fitness, 1.0);
    }

    #[test]
    fn test_interaction_conformance_result() {
        let event_id = Uuid::new_v4();
        let result = InteractionConformanceResult {
            event_id,
            activity: "process".to_string(),
            is_conformant: true,
            objects_involved: vec!["obj_1".to_string()],
            violations: vec![],
        };

        assert_eq!(result.activity, "process");
        assert!(result.is_conformant);
        assert_eq!(result.objects_involved.len(), 1);
    }

    #[test]
    fn test_token_replay_creation() {
        let checker = ObjectCentricTokenReplay::new(0.8);
        assert_eq!(checker.fitness_threshold, 0.8);
    }

    #[test]
    fn test_token_replay_empty_log() {
        let log = ObjectCentricEventLog::new();
        let net = ObjectCentricPetriNet::new();
        let checker = ObjectCentricTokenReplay::new(0.8);

        let result = checker.check(&log, &net);
        assert!(result.is_conformant);
        assert_eq!(result.fitness, 1.0);
    }

    #[test]
    fn test_token_replay_simple_object() {
        let mut log = ObjectCentricEventLog::new();
        let now = Utc::now();
        let order_type = ObjectType::new("order");

        let order = Object::new("order_1", order_type, now);
        log.add_object(order);

        let net = ObjectCentricPetriNet::new();
        let checker = ObjectCentricTokenReplay::new(0.8);

        let result = checker.check(&log, &net);
        assert_eq!(result.total_objects, 1);
        assert_eq!(result.conformant_objects, 1);
    }

    #[test]
    fn test_token_replay_with_lifecycle_violation() {
        let mut log = ObjectCentricEventLog::new();
        let now = Utc::now();
        let later = now + chrono::Duration::hours(1);
        let order_type = ObjectType::new("order");

        let order = Object::new("order_1", order_type, now).with_end_time(later);
        log.add_object(order);

        let event_id = Uuid::new_v4();
        log.add_event(
            event_id,
            "process",
            later + chrono::Duration::minutes(10),
            None,
        );

        let mut mapping = EventToObjectMapping::new(event_id);
        mapping.add_object("order_1");
        log.add_event_object_mapping(mapping);

        let net = ObjectCentricPetriNet::new();
        let checker = ObjectCentricTokenReplay::new(0.8);

        let result = checker.check(&log, &net);
        assert!(!result.object_results[0].is_conformant);
    }

    #[test]
    fn test_relationship_validator_creation() {
        let validator = ObjectRelationshipValidator::new(true);
        assert!(validator.enforce_strict_ordering);
    }

    #[test]
    fn test_relationship_validator_no_violations() {
        let mut log = ObjectCentricEventLog::new();
        let now = Utc::now();
        let order_type = ObjectType::new("order");
        let item_type = ObjectType::new("item");

        let order = Object::new("order_1", order_type, now);
        let item = Object::new("item_1", item_type, now);

        log.add_object(order);
        log.add_object(item);

        let validator = ObjectRelationshipValidator::new(true);
        let violations = validator.validate_relationships(&log);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_comprehensive_conformance_result() {
        let result = ObjectCentricConformanceResult {
            is_conformant: true,
            fitness: 0.95,
            object_results: vec![],
            interaction_results: vec![],
            total_objects: 10,
            conformant_objects: 9,
            total_interactions: 20,
            conformant_interactions: 19,
            fitness_by_object_type: BTreeMap::new(),
        };

        assert!(result.is_conformant);
        assert_eq!(result.fitness, 0.95);
        assert_eq!(result.total_objects, 10);
        assert_eq!(result.conformant_objects, 9);
    }

    #[test]
    fn test_fitness_by_object_type_populated() {
        let mut log = ObjectCentricEventLog::new();
        let now = Utc::now();
        let order_type = ObjectType::new("order");

        log.add_object(Object::new("order_1", order_type.clone(), now));
        log.add_object(Object::new("order_2", order_type, now));

        let net = ObjectCentricPetriNet::new();
        let checker = ObjectCentricTokenReplay::new(0.8);
        let result = checker.check(&log, &net);

        assert!(
            result.fitness_by_object_type.contains_key("order"),
            "fitness_by_object_type must contain 'order'"
        );
        let order_fitness = result.fitness_by_object_type["order"];
        assert!(
            order_fitness >= 0.0 && order_fitness <= 1.0,
            "fitness must be in [0, 1], got {}",
            order_fitness
        );
    }

    #[test]
    fn test_token_replay_multiple_objects() {
        let mut log = ObjectCentricEventLog::new();
        let now = Utc::now();
        let order_type = ObjectType::new("order");
        let item_type = ObjectType::new("item");

        log.add_object(Object::new("order_1", order_type, now));
        log.add_object(Object::new("item_1", item_type.clone(), now));
        log.add_object(Object::new("item_2", item_type, now));

        let net = ObjectCentricPetriNet::new();
        let checker = ObjectCentricTokenReplay::new(0.8);

        let result = checker.check(&log, &net);
        assert_eq!(result.total_objects, 3);
        assert_eq!(result.conformant_objects, 3);
        assert!(result.is_conformant);
    }
}
