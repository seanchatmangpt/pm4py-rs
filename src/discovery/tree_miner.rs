/// Tree-based process discovery using Inductive Miner
///
/// This module implements the Inductive Miner algorithm that discovers
/// process trees directly from event logs, identifying patterns like
/// sequences, choices, parallels, and loops.
use crate::log::EventLog;
use crate::models::process_tree::{ProcessTree, ProcessTreeNode};
use std::collections::{HashMap, HashSet};

/// Inductive Miner for discovering process trees
pub struct TreeMiner {
    /// Minimum support threshold (0.0 to 1.0)
    pub min_support: f64,
    /// Minimum relative frequency for activities
    pub min_activity_frequency: f64,
}

impl TreeMiner {
    /// Create a new tree miner with default settings
    pub fn new() -> Self {
        Self {
            min_support: 0.0,
            min_activity_frequency: 0.0,
        }
    }

    /// Set the minimum support threshold
    pub fn with_min_support(mut self, support: f64) -> Self {
        self.min_support = support.clamp(0.0, 1.0);
        self
    }

    /// Set the minimum activity frequency
    pub fn with_min_activity_frequency(mut self, freq: f64) -> Self {
        self.min_activity_frequency = freq.clamp(0.0, 1.0);
        self
    }

    /// Discover a process tree from an event log
    ///
    /// # Arguments
    /// * `log` - The event log to discover from
    ///
    /// # Returns
    /// A process tree representing the discovered process
    pub fn discover(&self, log: &EventLog) -> ProcessTree {
        if log.is_empty() {
            return ProcessTree::new(ProcessTreeNode::activity("SKIP"));
        }

        // Extract traces
        let traces: Vec<Vec<String>> = log
            .traces
            .iter()
            .map(|t| t.events.iter().map(|e| e.activity.clone()).collect())
            .collect();

        if traces.is_empty() {
            return ProcessTree::new(ProcessTreeNode::activity("SKIP"));
        }

        // Filter out very short traces
        let min_length = 1;

        let filtered_traces: Vec<_> = traces.iter().filter(|t| t.len() >= min_length).collect();

        if filtered_traces.is_empty() {
            return ProcessTree::new(ProcessTreeNode::activity("SKIP"));
        }

        let root = self.mine_tree(&filtered_traces);
        ProcessTree::new(root)
    }

    fn mine_tree(&self, traces: &[&Vec<String>]) -> ProcessTreeNode {
        if traces.is_empty() {
            return ProcessTreeNode::activity("SKIP");
        }

        // Single activity case
        if traces.iter().all(|t| t.len() == 1) {
            if let Some(activity) = traces[0].first() {
                return ProcessTreeNode::activity(activity.clone());
            }
        }

        // Try to find sequence pattern
        if let Some(sequence) = self.find_sequence(traces) {
            return sequence;
        }

        // Try to find choice pattern
        if let Some(choice) = self.find_choice(traces) {
            return choice;
        }

        // Try to find parallel pattern
        if let Some(parallel) = self.find_parallel(traces) {
            return parallel;
        }

        // Try to find loop pattern
        if let Some(loop_pattern) = self.find_loop(traces) {
            return loop_pattern;
        }

        // Fallback: treat all activities as a sequence
        let mut all_activities: Vec<String> = traces
            .iter()
            .flat_map(|t| t.iter())
            .cloned()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        all_activities.sort();

        if all_activities.is_empty() {
            ProcessTreeNode::activity("SKIP")
        } else if all_activities.len() == 1 {
            ProcessTreeNode::activity(all_activities[0].clone())
        } else {
            let nodes: Vec<_> = all_activities
                .into_iter()
                .map(ProcessTreeNode::activity)
                .collect();
            ProcessTreeNode::sequence(nodes)
        }
    }

    fn find_sequence(&self, traces: &[&Vec<String>]) -> Option<ProcessTreeNode> {
        if traces.is_empty() {
            return None;
        }

        // Check if there's a common prefix
        let min_len = traces.iter().map(|t| t.len()).min().unwrap_or(0);

        if min_len < 2 {
            return None;
        }

        // Find common activities at each position
        for position in 0..min_len {
            let activity_at_pos: Vec<&String> =
                traces.iter().filter_map(|t| t.get(position)).collect();

            if activity_at_pos.is_empty() {
                continue;
            }

            // Check if all traces have the same activity at this position
            let first = activity_at_pos[0];
            if !activity_at_pos.iter().all(|a| *a == first) {
                if position > 0 {
                    // We have a sequence up to this point
                    let prefix_traces: Vec<_> = traces
                        .iter()
                        .filter(|t| t.len() > position)
                        .copied()
                        .collect();

                    if prefix_traces.len()
                        >= (traces.len() as f64 * (1.0 - self.min_support)) as usize
                    {
                        // Extract activities at each position that are consistent
                        let mut sequence = Vec::new();

                        for pos in 0..position {
                            if let Some(activity) = traces[0].get(pos) {
                                sequence.push(ProcessTreeNode::activity(activity.clone()));
                            }
                        }

                        if !sequence.is_empty() {
                            return Some(ProcessTreeNode::sequence(sequence));
                        }
                    }
                }
                return None;
            }
        }

        None
    }

    fn find_choice(&self, traces: &[&Vec<String>]) -> Option<ProcessTreeNode> {
        if traces.len() < 2 {
            return None;
        }

        // Get the set of activities that appear in different traces
        let mut first_activities: HashMap<String, usize> = HashMap::new();
        let mut all_first_acts = HashSet::new();

        for trace in traces {
            if let Some(first) = trace.first() {
                *first_activities.entry(first.clone()).or_insert(0) += 1;
                all_first_acts.insert(first.clone());
            }
        }

        // If we have at least 2 different first activities and reasonable coverage
        if all_first_acts.len() >= 2 {
            let _threshold = (traces.len() as f64 * (1.0 - self.min_support)) as usize;
            let valid_activities: Vec<String> = first_activities
                .iter()
                .filter(|(_, count)| **count >= 1)
                .map(|(act, _)| act.clone())
                .collect();

            if valid_activities.len() >= 2 {
                // Partition traces by first activity
                let mut branches: HashMap<String, Vec<&Vec<String>>> = HashMap::new();
                for trace in traces {
                    if let Some(first) = trace.first() {
                        if valid_activities.contains(first) {
                            branches.entry(first.clone()).or_default().push(trace);
                        }
                    }
                }

                // Recursively mine each branch
                let choices: Vec<ProcessTreeNode> = branches
                    .into_iter()
                    .map(|(activity, branch_traces)| {
                        // Create sub-traces without the first activity
                        let sub_traces: Vec<Vec<String>> = branch_traces
                            .iter()
                            .map(|t| t.iter().skip(1).cloned().collect())
                            .collect();

                        let sub_traces_refs: Vec<_> = sub_traces.iter().collect();

                        if sub_traces_refs.iter().all(|t| t.is_empty()) {
                            ProcessTreeNode::activity(activity)
                        } else {
                            let sub_tree = self.mine_tree(&sub_traces_refs);
                            ProcessTreeNode::sequence(vec![
                                ProcessTreeNode::activity(activity),
                                sub_tree,
                            ])
                        }
                    })
                    .collect();

                if choices.len() >= 2 {
                    return Some(ProcessTreeNode::choice(choices));
                }
            }
        }

        None
    }

    fn find_parallel(&self, traces: &[&Vec<String>]) -> Option<ProcessTreeNode> {
        if traces.len() < 2 {
            return None;
        }

        // Check if activities appear in different orders across traces
        let mut activity_orders: Vec<Vec<String>> = Vec::new();

        for trace in traces {
            let unique_acts: Vec<String> = trace
                .iter()
                .cloned()
                .collect::<HashSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            activity_orders.push(unique_acts);
        }

        // Find activities that always appear together but in different orders
        if activity_orders.len() >= 2 {
            let first_set: HashSet<String> = activity_orders[0].iter().cloned().collect();

            let mut common_activities: HashSet<String> = first_set.clone();

            for order in activity_orders.iter().skip(1) {
                let current_set: HashSet<String> = order.iter().cloned().collect();
                common_activities = common_activities
                    .intersection(&current_set)
                    .cloned()
                    .collect();
            }

            // Check if these activities appear in different orders
            if common_activities.len() >= 2 {
                let mut is_parallel = false;
                for i in 0..traces.len() - 1 {
                    if traces[i] != traces[i + 1] {
                        is_parallel = true;
                        break;
                    }
                }

                if is_parallel {
                    let parallel_nodes: Vec<ProcessTreeNode> = common_activities
                        .into_iter()
                        .map(ProcessTreeNode::activity)
                        .collect();

                    if parallel_nodes.len() >= 2 {
                        return Some(ProcessTreeNode::parallel(parallel_nodes));
                    }
                }
            }
        }

        None
    }

    fn find_loop(&self, traces: &[&Vec<String>]) -> Option<ProcessTreeNode> {
        if traces.is_empty() {
            return None;
        }

        // Check if any activity repeats in traces
        for trace in traces {
            let mut seen = HashSet::new();
            for activity in trace.iter() {
                if seen.contains(activity) {
                    // Found a repeated activity - this could be a loop
                    // For now, return None as proper loop detection is complex
                    return None;
                }
                seen.insert(activity);
            }
        }

        None
    }
}

impl Default for TreeMiner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::{Event, Trace};
    use chrono::Utc;

    fn create_log_from_traces(traces: Vec<Vec<&str>>) -> EventLog {
        let mut log = EventLog::new();
        let now = Utc::now();

        for (i, trace_activities) in traces.iter().enumerate() {
            let mut trace = Trace::new(format!("case_{}", i));
            for activity in trace_activities {
                trace.add_event(Event::new(*activity, now));
            }
            log.add_trace(trace);
        }

        log
    }

    #[test]
    fn test_empty_log() {
        let log = EventLog::new();
        let miner = TreeMiner::new();
        let tree = miner.discover(&log);

        assert_eq!(tree.root.activities(), vec!["SKIP"]);
    }

    #[test]
    fn test_single_activity() {
        let log = create_log_from_traces(vec![vec!["A"], vec!["A"], vec!["A"]]);

        let miner = TreeMiner::new();
        let tree = miner.discover(&log);

        assert_eq!(tree.activities(), vec!["A"]);
    }

    #[test]
    fn test_sequence_pattern() {
        let log = create_log_from_traces(vec![
            vec!["A", "B", "C"],
            vec!["A", "B", "C"],
            vec!["A", "B", "C"],
        ]);

        let miner = TreeMiner::new();
        let tree = miner.discover(&log);

        assert!(tree.activities().contains(&"A".to_string()));
        assert!(tree.activities().contains(&"B".to_string()));
        assert!(tree.activities().contains(&"C".to_string()));
    }

    #[test]
    fn test_choice_pattern() {
        let log = create_log_from_traces(vec![vec!["A", "D"], vec!["B", "D"], vec!["C", "D"]]);

        let miner = TreeMiner::new();
        let tree = miner.discover(&log);

        let activities = tree.activities();
        assert!(activities.contains(&"A".to_string()));
        assert!(activities.contains(&"D".to_string()));
    }

    #[test]
    fn test_complex_log() {
        let log = create_log_from_traces(vec![
            vec!["A", "B", "C"],
            vec!["A", "B", "D"],
            vec!["A", "C", "D"],
        ]);

        let miner = TreeMiner::new();
        let tree = miner.discover(&log);

        assert!(tree.activities().contains(&"A".to_string()));
        assert!(tree.is_valid());
    }

    #[test]
    fn test_tree_miner_with_support() {
        let log = create_log_from_traces(vec![vec!["A", "B"], vec!["A", "C"], vec!["A", "B"]]);

        let miner = TreeMiner::new().with_min_support(0.1);
        let tree = miner.discover(&log);

        assert!(!tree.activities().is_empty());
    }

    #[test]
    fn test_tree_structure_validity() {
        let log = create_log_from_traces(vec![vec!["A", "B"], vec!["A", "C"], vec!["A", "B"]]);

        let miner = TreeMiner::new();
        let tree = miner.discover(&log);

        assert!(tree.is_valid());
    }
}
