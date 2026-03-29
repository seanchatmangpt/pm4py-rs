use serde::{Deserialize, Serialize};
/// Causal Net representation and operations
///
/// A Causal Net (C-Net) is an alternative process model to Petri nets.
/// It explicitly models causal relations between activities, including:
/// - → (direct causality)
/// - || (concurrency/parallelism)
/// - # (conflict/choice)
use std::collections::{HashMap, HashSet};

/// Causal relation type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CausalRelation {
    /// Direct causality: a → b
    Causality,
    /// Concurrency: a || b
    Parallel,
    /// Conflict/Choice: a # b
    Conflict,
}

/// Input/output set for an activity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IOSet {
    /// Input set: set of activities that precede this activity
    pub input: Vec<HashSet<String>>,
    /// Output set: set of activities that follow this activity
    pub output: Vec<HashSet<String>>,
}

impl IOSet {
    pub fn new() -> Self {
        Self {
            input: Vec::new(),
            output: Vec::new(),
        }
    }

    pub fn with_input(mut self, input_set: HashSet<String>) -> Self {
        self.input.push(input_set);
        self
    }

    pub fn with_output(mut self, output_set: HashSet<String>) -> Self {
        self.output.push(output_set);
        self
    }
}

impl Default for IOSet {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a Causal Net
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalNet {
    /// All activities in the net
    pub activities: HashSet<String>,
    /// Start activities
    pub start_activities: HashSet<String>,
    /// End activities
    pub end_activities: HashSet<String>,
    /// Input/output sets for each activity
    pub io_sets: HashMap<String, IOSet>,
    /// Causal relations between activities
    pub relations: HashMap<(String, String), CausalRelation>,
}

impl CausalNet {
    pub fn new() -> Self {
        Self {
            activities: HashSet::new(),
            start_activities: HashSet::new(),
            end_activities: HashSet::new(),
            io_sets: HashMap::new(),
            relations: HashMap::new(),
        }
    }

    /// Add an activity
    pub fn add_activity(&mut self, activity: String) {
        self.activities.insert(activity.clone());
        self.io_sets.entry(activity).or_default();
    }

    /// Set as start activity
    pub fn add_start_activity(&mut self, activity: String) {
        self.add_activity(activity.clone());
        self.start_activities.insert(activity);
    }

    /// Set as end activity
    pub fn add_end_activity(&mut self, activity: String) {
        self.add_activity(activity.clone());
        self.end_activities.insert(activity);
    }

    /// Add a causal relation
    pub fn add_relation(&mut self, from: String, to: String, relation: CausalRelation) {
        self.add_activity(from.clone());
        self.add_activity(to.clone());
        self.relations.insert((from, to), relation);
    }

    /// Get all relations of a specific type
    pub fn get_relations_by_type(
        &self,
        relation_type: CausalRelation,
    ) -> Vec<((String, String), CausalRelation)> {
        self.relations
            .iter()
            .filter(|(_, rel)| **rel == relation_type)
            .map(|(k, v)| (k.clone(), *v))
            .collect()
    }

    /// Get activities directly caused by an activity
    pub fn get_directly_caused_by(&self, activity: &str) -> Vec<String> {
        self.relations
            .iter()
            .filter(|((from, _), rel)| from == activity && **rel == CausalRelation::Causality)
            .map(|((_, to), _)| to.clone())
            .collect()
    }

    /// Get activities that directly cause an activity
    pub fn get_causes(&self, activity: &str) -> Vec<String> {
        self.relations
            .iter()
            .filter(|((_, to), rel)| to == activity && **rel == CausalRelation::Causality)
            .map(|((from, _), _)| from.clone())
            .collect()
    }

    /// Check if activity is a start activity
    pub fn is_start(&self, activity: &str) -> bool {
        self.start_activities.contains(activity)
    }

    /// Check if activity is an end activity
    pub fn is_end(&self, activity: &str) -> bool {
        self.end_activities.contains(activity)
    }

    /// Check if a trace is accepted by this causal net
    /// A trace is accepted if all causal relations are respected
    pub fn accepts_trace(&self, trace: &[String]) -> bool {
        if trace.is_empty() {
            return true;
        }

        // Check if first activity is a start activity
        if !self.is_start(&trace[0]) {
            return false;
        }

        // Check if last activity is an end activity
        if !self.is_end(&trace[trace.len() - 1]) {
            return false;
        }

        // Check if all activities are in the net
        for activity in trace {
            if !self.activities.contains(activity) {
                return false;
            }
        }

        // Check directly-follows relations
        for i in 0..trace.len() - 1 {
            let current = &trace[i];
            let next = &trace[i + 1];

            // Check if there's a causal relation or parallel relation
            let has_causal = self
                .relations
                .get(&(current.clone(), next.clone()))
                .map(|r| *r == CausalRelation::Causality)
                .unwrap_or(false);

            let has_parallel = self
                .relations
                .get(&(current.clone(), next.clone()))
                .map(|r| *r == CausalRelation::Parallel)
                .unwrap_or(false);

            // In a basic trace acceptance, we accept if there's any direct relation
            if !has_causal && !has_parallel {
                // For conflict relations, we allow them to be traversed
                let has_conflict = self
                    .relations
                    .get(&(current.clone(), next.clone()))
                    .map(|r| *r == CausalRelation::Conflict)
                    .unwrap_or(false);

                if !has_conflict {
                    return false;
                }
            }
        }

        true
    }

    /// Count number of activities
    pub fn num_activities(&self) -> usize {
        self.activities.len()
    }

    /// Count number of relations
    pub fn num_relations(&self) -> usize {
        self.relations.len()
    }
}

impl Default for CausalNet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_causal_net_creation() {
        let mut net = CausalNet::new();
        net.add_activity("A".to_string());
        net.add_activity("B".to_string());
        net.add_activity("C".to_string());

        assert_eq!(net.num_activities(), 3);
    }

    #[test]
    fn test_causal_net_relations() {
        let mut net = CausalNet::new();
        net.add_activity("A".to_string());
        net.add_activity("B".to_string());
        net.add_activity("C".to_string());

        net.add_relation("A".to_string(), "B".to_string(), CausalRelation::Causality);
        net.add_relation("B".to_string(), "C".to_string(), CausalRelation::Causality);

        assert_eq!(net.num_relations(), 2);
        assert_eq!(net.get_directly_caused_by("A"), vec!["B".to_string()]);
    }

    #[test]
    fn test_causal_net_trace_acceptance() {
        let mut net = CausalNet::new();
        net.add_start_activity("A".to_string());
        net.add_activity("B".to_string());
        net.add_end_activity("C".to_string());

        net.add_relation("A".to_string(), "B".to_string(), CausalRelation::Causality);
        net.add_relation("B".to_string(), "C".to_string(), CausalRelation::Causality);

        let trace = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        assert!(net.accepts_trace(&trace));
    }

    #[test]
    fn test_causal_net_trace_rejection() {
        let mut net = CausalNet::new();
        net.add_start_activity("A".to_string());
        net.add_activity("B".to_string());
        net.add_end_activity("C".to_string());

        net.add_relation("A".to_string(), "B".to_string(), CausalRelation::Causality);
        net.add_relation("B".to_string(), "C".to_string(), CausalRelation::Causality);

        // Invalid trace: missing activity B
        let trace = vec!["A".to_string(), "C".to_string()];
        assert!(!net.accepts_trace(&trace));
    }

    #[test]
    fn test_start_and_end_activities() {
        let mut net = CausalNet::new();
        net.add_start_activity("A".to_string());
        net.add_activity("B".to_string());
        net.add_end_activity("C".to_string());

        assert!(net.is_start("A"));
        assert!(!net.is_start("B"));
        assert!(net.is_end("C"));
    }

    #[test]
    fn test_parallel_relations() {
        let mut net = CausalNet::new();
        net.add_activity("A".to_string());
        net.add_activity("B".to_string());
        net.add_activity("C".to_string());

        net.add_relation("A".to_string(), "B".to_string(), CausalRelation::Parallel);
        net.add_relation("A".to_string(), "C".to_string(), CausalRelation::Parallel);

        let parallel_rels = net.get_relations_by_type(CausalRelation::Parallel);
        assert_eq!(parallel_rels.len(), 2);
    }
}
