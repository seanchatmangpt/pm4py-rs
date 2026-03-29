use serde::{Deserialize, Serialize};
/// Footprints-based process model representation
///
/// Footprints are a lightweight representation of process behavior based on
/// activity pair relationships. This module provides efficient analysis of
/// directly-follows, parallel, causal, and choice relationships between activities.
use std::collections::{HashMap, HashSet};

/// Represents the relationship between two activities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActivityRelationship {
    /// Activity a is directly followed by b in some trace
    DirectlyFollows,
    /// Both (a,b) and (b,a) occur in some traces - truly parallel
    Parallel,
    /// Activity a can be followed by b, but b is never followed by a
    Causal,
    /// Neither (a,b) nor (b,a) occur - mutually exclusive
    Choice,
}

/// A pair of activities and their relationship
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ActivityPair {
    pub activity_a: String,
    pub activity_b: String,
    pub relationship: ActivityRelationship,
}

/// Footprints matrix - all activity pair relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Footprints {
    /// Mapping from (activity_a, activity_b) to relationship
    relationships: HashMap<(String, String), ActivityRelationship>,
    /// All activities observed
    activities: HashSet<String>,
}

impl Footprints {
    /// Create a new empty footprints
    pub fn new() -> Self {
        Self {
            relationships: HashMap::new(),
            activities: HashSet::new(),
        }
    }

    /// Create footprints from an event log
    ///
    /// # Arguments
    /// * `traces` - Collection of traces where each trace is a sequence of activity names
    ///
    /// # Algorithm
    /// 1. For each trace, identify directly-follows pairs
    /// 2. Determine relationship for each pair:
    ///    - If both (a>b) and (b>a) exist → Parallel
    ///    - If only (a>b) exists → Causal
    ///    - If neither exists → Choice
    pub fn from_traces<S: AsRef<str>>(traces: &[Vec<S>]) -> Self {
        let mut directly_follows: HashMap<(String, String), bool> = HashMap::new();
        let mut activities = HashSet::new();

        // Collect directly-follows relationships
        for trace in traces {
            for window in trace.windows(2) {
                let a = window[0].as_ref().to_string();
                let b = window[1].as_ref().to_string();

                activities.insert(a.clone());
                activities.insert(b.clone());

                directly_follows.insert((a, b), true);
            }

            // Add single activities
            for activity in trace {
                activities.insert(activity.as_ref().to_string());
            }
        }

        // Determine relationships
        let mut relationships = HashMap::new();

        let activities_vec: Vec<_> = activities.iter().cloned().collect();
        for i in 0..activities_vec.len() {
            for j in 0..activities_vec.len() {
                if i == j {
                    continue;
                }

                let a = &activities_vec[i];
                let b = &activities_vec[j];

                let ab_follows = directly_follows.contains_key(&(a.clone(), b.clone()));
                let ba_follows = directly_follows.contains_key(&(b.clone(), a.clone()));

                let relationship = if ab_follows && ba_follows {
                    ActivityRelationship::Parallel
                } else if ab_follows {
                    ActivityRelationship::Causal
                } else if ba_follows {
                    // This is (b>a), so (a→b) is opposite direction
                    // Skip - we'll catch this when j<i
                    continue;
                } else {
                    ActivityRelationship::Choice
                };

                relationships.insert((a.clone(), b.clone()), relationship);
            }
        }

        Self {
            relationships,
            activities,
        }
    }

    /// Get relationship between two activities
    pub fn get_relationship(&self, a: &str, b: &str) -> Option<ActivityRelationship> {
        self.relationships
            .get(&(a.to_string(), b.to_string()))
            .copied()
    }

    /// Create footprints from an event log
    ///
    /// Convenience method that extracts traces from an EventLog.
    pub fn from_log(log: &crate::log::EventLog) -> Self {
        let traces: Vec<Vec<String>> = log
            .traces
            .iter()
            .map(|trace| trace.events.iter().map(|e| e.activity.clone()).collect())
            .collect();
        Self::from_traces(&traces)
    }

    /// Set relationship between two activities
    pub fn set_relationship(
        &mut self,
        a: impl Into<String>,
        b: impl Into<String>,
        rel: ActivityRelationship,
    ) {
        let a = a.into();
        let b = b.into();
        self.activities.insert(a.clone());
        self.activities.insert(b.clone());
        self.relationships.insert((a, b), rel);
    }

    /// Get all activities
    pub fn activities(&self) -> &HashSet<String> {
        &self.activities
    }

    /// Get all relationships
    pub fn relationships(&self) -> &HashMap<(String, String), ActivityRelationship> {
        &self.relationships
    }

    /// Get all activity pairs with their relationships
    pub fn pairs(&self) -> Vec<ActivityPair> {
        self.relationships
            .iter()
            .map(|((a, b), rel)| ActivityPair {
                activity_a: a.clone(),
                activity_b: b.clone(),
                relationship: *rel,
            })
            .collect()
    }

    /// Convert footprints to a matrix representation for visualization
    ///
    /// Returns a tuple of (activities, matrix) where matrix\[i\]\[j\] represents
    /// the relationship between activities\[i\] and activities\[j\]
    pub fn to_matrix(&self) -> (Vec<String>, Vec<Vec<Option<ActivityRelationship>>>) {
        let mut activities: Vec<String> = self.activities.iter().cloned().collect();
        activities.sort();

        let n = activities.len();
        let mut matrix = vec![vec![None; n]; n];

        for i in 0..n {
            for j in 0..n {
                if i != j {
                    if let Some(rel) = self.get_relationship(&activities[i], &activities[j]) {
                        matrix[i][j] = Some(rel);
                    }
                }
            }
        }

        (activities, matrix)
    }

    /// Compare with another footprints and return mismatch pairs
    pub fn compare(
        &self,
        other: &Footprints,
    ) -> Vec<(String, String, ActivityRelationship, ActivityRelationship)> {
        let mut mismatches = Vec::new();

        // Check if activity sets differ
        let our_activities = self.activities();
        let other_activities = other.activities();
        if our_activities != other_activities {
            // Return a mismatch for each difference
            for (a, b) in self.relationships.keys() {
                if let Some(our_rel) = self.get_relationship(a, b) {
                    if let Some(other_rel) = other.get_relationship(a, b) {
                        if our_rel != other_rel {
                            mismatches.push((a.clone(), b.clone(), our_rel, other_rel));
                        }
                    } else {
                        // Relationship exists in us but not in other - that's a mismatch
                        mismatches.push((
                            a.clone(),
                            b.clone(),
                            our_rel,
                            ActivityRelationship::Choice,
                        ));
                    }
                }
            }
            for (a, b) in other.relationships.keys() {
                if !self.relationships.contains_key(&(a.clone(), b.clone())) {
                    if let Some(other_rel) = other.get_relationship(a, b) {
                        mismatches.push((
                            a.clone(),
                            b.clone(),
                            ActivityRelationship::Choice,
                            other_rel,
                        ));
                    }
                }
            }
            return mismatches;
        }

        // If activities match, compare relationships
        for ((a, b), our_rel) in &self.relationships {
            if let Some(other_rel) = other.get_relationship(a, b) {
                if *our_rel != other_rel {
                    mismatches.push((a.clone(), b.clone(), *our_rel, other_rel));
                }
            }
        }

        mismatches
    }

    /// Count activity pairs with each relationship type
    pub fn relationship_counts(&self) -> HashMap<ActivityRelationship, usize> {
        let mut counts = HashMap::new();
        for rel in self.relationships.values() {
            *counts.entry(*rel).or_insert(0) += 1;
        }
        counts
    }
}

impl Default for Footprints {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_footprints_creation() {
        let traces = vec![vec!["A", "B", "C"], vec!["A", "B", "C"]];

        let fp = Footprints::from_traces(&traces);

        assert!(fp.activities().contains("A"));
        assert!(fp.activities().contains("B"));
        assert!(fp.activities().contains("C"));
    }

    #[test]
    fn test_directly_follows() {
        let traces = vec![vec!["A", "B", "C"]];
        let fp = Footprints::from_traces(&traces);

        assert_eq!(
            fp.get_relationship("A", "B"),
            Some(ActivityRelationship::Causal)
        );
        assert_eq!(
            fp.get_relationship("B", "C"),
            Some(ActivityRelationship::Causal)
        );
    }

    #[test]
    fn test_parallel_relationship() {
        let traces = vec![vec!["A", "B"], vec!["B", "A"]];
        let fp = Footprints::from_traces(&traces);

        // When both orders occur
        assert_eq!(
            fp.get_relationship("A", "B"),
            Some(ActivityRelationship::Parallel)
        );
    }

    #[test]
    fn test_choice_relationship() {
        let traces = vec![vec!["A", "B"], vec!["A", "C"]];
        let fp = Footprints::from_traces(&traces);

        // B and C never follow each other
        assert_eq!(
            fp.get_relationship("B", "C"),
            Some(ActivityRelationship::Choice)
        );
    }

    #[test]
    fn test_matrix_conversion() {
        let traces = vec![vec!["A", "B"]];
        let fp = Footprints::from_traces(&traces);

        let (activities, matrix) = fp.to_matrix();
        assert_eq!(activities.len(), 2);
        assert_eq!(matrix.len(), 2);
    }

    #[test]
    fn test_set_relationship() {
        let mut fp = Footprints::new();
        fp.set_relationship("A", "B", ActivityRelationship::Causal);

        assert_eq!(
            fp.get_relationship("A", "B"),
            Some(ActivityRelationship::Causal)
        );
        assert!(fp.activities().contains("A"));
        assert!(fp.activities().contains("B"));
    }

    #[test]
    fn test_compare_footprints() {
        let traces1 = vec![vec!["A", "B", "C"]];
        let traces2 = vec![vec!["A", "B", "D"]];

        let fp1 = Footprints::from_traces(&traces1);
        let fp2 = Footprints::from_traces(&traces2);

        let mismatches = fp1.compare(&fp2);
        assert!(!mismatches.is_empty());
    }

    #[test]
    fn test_relationship_counts() {
        let traces = vec![vec!["A", "B", "C"], vec!["B", "A"]];
        let fp = Footprints::from_traces(&traces);

        let counts = fp.relationship_counts();
        assert!(counts.get(&ActivityRelationship::Parallel).is_some());
    }
}
