/// Footprints-based Conformance Checking
///
/// Fast conformance checking using footprints (activity pair relationships).
/// This is an efficient alternative to alignment-based techniques.
use crate::log::EventLog;
use crate::models::footprints::{ActivityRelationship, Footprints};
use crate::models::PetriNet;
use std::collections::HashMap;

/// Detailed result of footprints conformance check
#[derive(Debug, Clone)]
pub struct FootprintsConformanceResult {
    pub is_conformant: bool,
    pub fitness: f64,
    pub total_pairs: usize,
    pub matching_pairs: usize,
    pub mismatching_pairs: Vec<(String, String, ActivityRelationship, ActivityRelationship)>,
    pub missing_relationships: Vec<(String, String, ActivityRelationship)>,
}

impl FootprintsConformanceResult {
    /// Get the proportion of matching pairs
    pub fn precision(&self) -> f64 {
        if self.total_pairs == 0 {
            return 1.0;
        }
        self.matching_pairs as f64 / self.total_pairs as f64
    }

    /// Get detailed description of conformance
    pub fn summary(&self) -> String {
        format!(
            "Footprints Conformance: {}/{} pairs match ({}%)\n\
             Fitness: {:.2}%\n\
             Mismatches: {} | Missing: {}",
            self.matching_pairs,
            self.total_pairs,
            (self.precision() * 100.0) as u32,
            self.fitness * 100.0,
            self.mismatching_pairs.len(),
            self.missing_relationships.len()
        )
    }
}

/// Footprints-based conformance checker
pub struct FootprintsConformanceChecker;

impl FootprintsConformanceChecker {
    /// Check conformance of a log against a model using footprints
    ///
    /// # Arguments
    /// * `log` - Event log to check
    /// * `model_footprints` - Footprints extracted from the model
    ///
    /// # Returns
    /// Detailed conformance result
    pub fn check_log(log: &EventLog, model_footprints: &Footprints) -> FootprintsConformanceResult {
        // Extract traces as activity sequences
        let traces: Vec<Vec<String>> = log
            .traces
            .iter()
            .map(|trace| trace.events.iter().map(|e| e.activity.clone()).collect())
            .collect();

        // Create footprints from log
        let log_traces: Vec<Vec<&str>> = traces
            .iter()
            .map(|t| t.iter().map(|s| s.as_str()).collect())
            .collect();

        let log_footprints = Footprints::from_traces(&log_traces);

        Self::compare_footprints(&log_footprints, model_footprints)
    }

    /// Check conformance against a Petri net
    ///
    /// # Algorithm
    /// 1. Extract footprints from Petri net
    /// 2. Extract footprints from log
    /// 3. Compare relationship matrices
    pub fn check_petri_net(log: &EventLog, petri_net: &PetriNet) -> FootprintsConformanceResult {
        // Extract model footprints from Petri net
        let model_footprints = Self::footprints_from_petri_net(petri_net);

        // Extract log footprints
        let traces: Vec<Vec<String>> = log
            .traces
            .iter()
            .map(|trace| trace.events.iter().map(|e| e.activity.clone()).collect())
            .collect();

        let log_traces: Vec<Vec<&str>> = traces
            .iter()
            .map(|t| t.iter().map(|s| s.as_str()).collect())
            .collect();

        let log_footprints = Footprints::from_traces(&log_traces);

        Self::compare_footprints(&log_footprints, &model_footprints)
    }

    /// Compare two footprints matrices
    pub fn compare_footprints(
        log_footprints: &Footprints,
        model_footprints: &Footprints,
    ) -> FootprintsConformanceResult {
        let mut matching_pairs = 0;
        let mut mismatching_pairs = Vec::new();
        let mut missing_relationships = Vec::new();

        let total_pairs = model_footprints.relationships().len();

        // Check each model relationship
        for ((a, b), model_rel) in model_footprints.relationships() {
            match log_footprints.get_relationship(a, b) {
                Some(log_rel) => {
                    if log_rel == *model_rel {
                        matching_pairs += 1;
                    } else {
                        mismatching_pairs.push((a.clone(), b.clone(), log_rel, *model_rel));
                    }
                }
                None => {
                    missing_relationships.push((a.clone(), b.clone(), *model_rel));
                }
            }
        }

        let fitness = if total_pairs == 0 {
            1.0
        } else {
            matching_pairs as f64 / total_pairs as f64
        };

        let is_conformant = missing_relationships.is_empty() && mismatching_pairs.is_empty();

        FootprintsConformanceResult {
            is_conformant,
            fitness,
            total_pairs,
            matching_pairs,
            mismatching_pairs,
            missing_relationships,
        }
    }

    /// Extract footprints from a Petri net
    ///
    /// Analyzes all possible execution paths to determine activity relationships
    pub fn footprints_from_petri_net(petri_net: &PetriNet) -> Footprints {
        let mut footprints = Footprints::new();

        // Get all visible transitions (activities)
        let visible_transitions: Vec<_> = petri_net
            .transitions
            .iter()
            .filter(|t| !t.is_invisible())
            .collect();

        // For each pair of activities, determine their relationship
        for i in 0..visible_transitions.len() {
            for j in 0..visible_transitions.len() {
                if i == j {
                    continue;
                }

                let a_label = visible_transitions[i]
                    .label
                    .as_ref()
                    .unwrap_or(&visible_transitions[i].name);
                let b_label = visible_transitions[j]
                    .label
                    .as_ref()
                    .unwrap_or(&visible_transitions[j].name);

                // Check reachability relationships
                let a_to_b = Self::can_follow_in_petri(
                    petri_net,
                    visible_transitions[i].id.as_str(),
                    visible_transitions[j].id.as_str(),
                );
                let b_to_a = Self::can_follow_in_petri(
                    petri_net,
                    visible_transitions[j].id.as_str(),
                    visible_transitions[i].id.as_str(),
                );

                let relationship = if a_to_b && b_to_a {
                    ActivityRelationship::Parallel
                } else if a_to_b {
                    ActivityRelationship::Causal
                } else {
                    ActivityRelationship::Choice
                };

                footprints.set_relationship(a_label, b_label, relationship);
            }
        }

        footprints
    }

    /// Check if b can follow a in the Petri net.
    ///
    /// BFS over the bipartite place-transition graph starting from a's output
    /// places; returns true if any of b's input places is reachable.
    /// State cap: 50 000 visited places to bound runtime on large nets.
    fn can_follow_in_petri(petri_net: &PetriNet, a_id: &str, b_id: &str) -> bool {
        use std::collections::VecDeque;

        // Output places produced when transition a fires
        let a_outputs: Vec<String> = petri_net
            .get_arcs_from(a_id)
            .into_iter()
            .map(|arc| arc.to.clone())
            .collect();
        if a_outputs.is_empty() {
            return false;
        }

        // Input places that transition b requires
        let b_inputs: std::collections::HashSet<String> = petri_net
            .get_arcs_to(b_id)
            .into_iter()
            .map(|arc| arc.from.clone())
            .collect();
        if b_inputs.is_empty() {
            return true; // b has no input constraints — always fireable after a
        }

        // BFS: place → transition → output-place
        let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut queue: VecDeque<String> = VecDeque::new();

        for place in a_outputs {
            if visited.insert(place.clone()) {
                queue.push_back(place);
            }
        }

        const MAX_VISITED: usize = 50_000;

        while let Some(place_id) = queue.pop_front() {
            if b_inputs.contains(&place_id) {
                return true;
            }
            if visited.len() >= MAX_VISITED {
                break;
            }
            // Arcs leaving a place go to transitions (place→transition edge).
            // Follow each such transition to its output places.
            for arc in petri_net.arcs.iter().filter(|a| a.from == place_id) {
                for out_arc in petri_net.get_arcs_from(&arc.to) {
                    let out_place = out_arc.to.clone();
                    if visited.insert(out_place.clone()) {
                        queue.push_back(out_place);
                    }
                }
            }
        }

        false
    }

    /// Get activity pair statistics from log
    pub fn analyze_activity_pairs(log: &EventLog) -> HashMap<(String, String), usize> {
        let mut pair_counts = HashMap::new();

        for trace in &log.traces {
            for window in trace.events.windows(2) {
                let a = &window[0].activity;
                let b = &window[1].activity;
                *pair_counts.entry((a.clone(), b.clone())).or_insert(0) += 1;
            }
        }

        pair_counts
    }

    /// Get relationship frequency distribution
    pub fn relationship_distribution(
        footprints: &Footprints,
    ) -> HashMap<ActivityRelationship, usize> {
        footprints.relationship_counts()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::{Event, Trace};
    use chrono::Utc;

    fn create_test_log() -> EventLog {
        let mut log = EventLog::new();

        // Trace 1: A -> B -> C
        let mut trace1 = Trace::new("trace_1");
        trace1.add_event(Event::new("A", Utc::now()));
        trace1.add_event(Event::new("B", Utc::now()));
        trace1.add_event(Event::new("C", Utc::now()));
        log.add_trace(trace1);

        // Trace 2: A -> B -> C
        let mut trace2 = Trace::new("trace_2");
        trace2.add_event(Event::new("A", Utc::now()));
        trace2.add_event(Event::new("B", Utc::now()));
        trace2.add_event(Event::new("C", Utc::now()));
        log.add_trace(trace2);

        log
    }

    #[test]
    fn test_conformance_creation() {
        let result = FootprintsConformanceResult {
            is_conformant: true,
            fitness: 1.0,
            total_pairs: 2,
            matching_pairs: 2,
            mismatching_pairs: Vec::new(),
            missing_relationships: Vec::new(),
        };

        assert!(result.is_conformant);
        assert_eq!(result.precision(), 1.0);
    }

    #[test]
    fn test_footprints_from_log() {
        let log = create_test_log();
        let traces: Vec<Vec<String>> = log
            .traces
            .iter()
            .map(|trace| trace.events.iter().map(|e| e.activity.clone()).collect())
            .collect();

        let traces_refs: Vec<Vec<&str>> = traces
            .iter()
            .map(|t| t.iter().map(|s| s.as_str()).collect())
            .collect();

        let footprints = Footprints::from_traces(&traces_refs);

        assert!(footprints.activities().contains("A"));
        assert!(footprints.activities().contains("B"));
        assert!(footprints.activities().contains("C"));
    }

    #[test]
    fn test_compare_matching_footprints() {
        let mut fp1 = Footprints::new();
        fp1.set_relationship("A", "B", ActivityRelationship::Causal);

        let mut fp2 = Footprints::new();
        fp2.set_relationship("A", "B", ActivityRelationship::Causal);

        let result = FootprintsConformanceChecker::compare_footprints(&fp1, &fp2);

        assert!(result.is_conformant);
        assert_eq!(result.matching_pairs, 1);
    }

    #[test]
    fn test_compare_mismatching_footprints() {
        let mut fp1 = Footprints::new();
        fp1.set_relationship("A", "B", ActivityRelationship::Causal);

        let mut fp2 = Footprints::new();
        fp2.set_relationship("A", "B", ActivityRelationship::Parallel);

        let result = FootprintsConformanceChecker::compare_footprints(&fp1, &fp2);

        assert!(!result.is_conformant);
        assert_eq!(result.mismatching_pairs.len(), 1);
    }

    #[test]
    fn test_analyze_activity_pairs() {
        let log = create_test_log();
        let pairs = FootprintsConformanceChecker::analyze_activity_pairs(&log);

        // A->B should appear twice, B->C should appear twice
        assert_eq!(pairs.get(&("A".to_string(), "B".to_string())), Some(&2));
        assert_eq!(pairs.get(&("B".to_string(), "C".to_string())), Some(&2));
    }

    #[test]
    fn test_conformance_summary() {
        let result = FootprintsConformanceResult {
            is_conformant: true,
            fitness: 0.8,
            total_pairs: 5,
            matching_pairs: 4,
            mismatching_pairs: vec![],
            missing_relationships: vec![],
        };

        let summary = result.summary();
        assert!(summary.contains("4/5"));
        assert!(summary.contains("80"));
    }

    #[test]
    fn test_relationship_distribution() {
        let mut fp = Footprints::new();
        fp.set_relationship("A", "B", ActivityRelationship::Causal);
        fp.set_relationship("B", "C", ActivityRelationship::Causal);
        fp.set_relationship("A", "C", ActivityRelationship::Choice);

        let dist = FootprintsConformanceChecker::relationship_distribution(&fp);

        assert_eq!(dist.get(&ActivityRelationship::Causal), Some(&2));
        assert_eq!(dist.get(&ActivityRelationship::Choice), Some(&1));
    }

    #[test]
    fn test_precision_calculation() {
        let result = FootprintsConformanceResult {
            is_conformant: false,
            fitness: 0.5,
            total_pairs: 4,
            matching_pairs: 2,
            mismatching_pairs: vec![],
            missing_relationships: vec![],
        };

        assert_eq!(result.precision(), 0.5);
    }

    #[test]
    fn test_missing_relationships() {
        let mut fp1 = Footprints::new();
        fp1.set_relationship("A", "B", ActivityRelationship::Causal);

        let mut fp2 = Footprints::new();
        fp2.set_relationship("A", "B", ActivityRelationship::Causal);
        fp2.set_relationship("B", "C", ActivityRelationship::Causal);

        let result = FootprintsConformanceChecker::compare_footprints(&fp1, &fp2);

        assert_eq!(result.missing_relationships.len(), 1);
    }
}

// ========================================================================
// MISSING WRAPPER FUNCTIONS FOR PYTHON PM4PY PARITY
// ========================================================================

/// Calculate fitness based on footprints comparison
pub fn fitness_footprints(log: &EventLog, fp: &Footprints) -> f64 {
    let log_fp = Footprints::from_log(log);
    let result = FootprintsConformanceChecker::compare_footprints(&log_fp, fp);
    result.fitness
}

/// Calculate precision based on footprints comparison
pub fn precision_footprints(log: &EventLog, fp: &Footprints) -> f64 {
    let log_fp = Footprints::from_log(log);
    let result = FootprintsConformanceChecker::compare_footprints(fp, &log_fp);
    result.precision()
}

/// Get diagnostics from footprints conformance checking
pub fn diagnostics_footprints(log: &EventLog, fp: &Footprints) -> FootprintsConformanceResult {
    let log_fp = Footprints::from_log(log);
    FootprintsConformanceChecker::compare_footprints(&log_fp, fp)
}
