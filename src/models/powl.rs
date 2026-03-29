//! Partially Ordered Workflow Language (POWL) model.
//!
//! POWL extends a Directly-Follows Graph with partial order semantics,
//! capturing concurrent activities (parallel_groups) and exclusive
//! choices (choice_groups) derived from the event log.

use serde::{Deserialize, Serialize};

/// Partially Ordered Workflow Language model.
///
/// Represents process behaviour as a partial order of activities, with
/// explicit annotations for exclusive choices and concurrent groups.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct POWLModel {
    /// Sorted list of unique activity names discovered in the log.
    pub activities: Vec<String>,
    /// Partial order edges as `(from_idx, to_idx)` index pairs into `activities`.
    /// Derived by taking all DFG edges after transitivity reduction.
    pub partial_order: Vec<(usize, usize)>,
    /// Groups of activity indices that are mutually exclusive (XOR choice):
    /// no two activities in the same group ever appear in the same trace.
    pub choice_groups: Vec<Vec<usize>>,
    /// Groups of activity indices that are always concurrent: they share a
    /// common predecessor and appear in both orderings across different traces.
    pub parallel_groups: Vec<Vec<usize>>,
}

impl POWLModel {
    /// Returns `true` if no activities have been discovered.
    pub fn is_empty(&self) -> bool {
        self.activities.is_empty()
    }

    /// Returns the number of distinct activities.
    pub fn activity_count(&self) -> usize {
        self.activities.len()
    }
}
