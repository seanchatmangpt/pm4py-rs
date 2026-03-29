use crate::log::EventLog;
use serde::{Deserialize, Serialize};
/// Directly-Follows Graph (DFG) representation
use std::collections::{BTreeMap, HashMap, HashSet};

/// Represents an edge in a DFG
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DFGEdge {
    pub from: String,
    pub to: String,
    pub frequency: usize,
}

impl DFGEdge {
    pub fn new(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            frequency: 1,
        }
    }
}

/// Represents a Directly-Follows Graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectlyFollowsGraph {
    pub nodes: Vec<String>,
    pub edges: Vec<DFGEdge>,
    pub start_activities: BTreeMap<String, usize>,
    pub end_activities: BTreeMap<String, usize>,
    pub activity_frequency: BTreeMap<String, usize>,
}

impl DirectlyFollowsGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            start_activities: BTreeMap::new(),
            end_activities: BTreeMap::new(),
            activity_frequency: BTreeMap::new(),
        }
    }

    /// Create a DFG from an event log (optimized version)
    pub fn from_log(log: &EventLog) -> Self {
        let mut dfg = DirectlyFollowsGraph::new();

        // Use HashSet for O(1) node lookups during construction
        let mut node_set = HashSet::new();
        // Use HashMap for O(1) edge lookups during construction: (from, to) -> edge_index
        let mut edge_map: HashMap<(String, String), usize> = HashMap::new();

        for trace in &log.traces {
            if trace.events.is_empty() {
                continue;
            }

            // Add start activity
            if let Some(first_event) = trace.events.first() {
                *dfg.start_activities
                    .entry(first_event.activity.clone())
                    .or_insert(0) += 1;
            }

            // Add end activity
            if let Some(last_event) = trace.events.last() {
                *dfg.end_activities
                    .entry(last_event.activity.clone())
                    .or_insert(0) += 1;
            }

            // Process directly-follows relations
            for i in 0..trace.events.len() {
                let activity = &trace.events[i].activity;

                // Add node using HashSet (O(1) instead of O(n))
                if node_set.insert(activity.clone()) {
                    dfg.nodes.push(activity.clone());
                }

                // Add activity frequency
                *dfg.activity_frequency.entry(activity.clone()).or_insert(0) += 1;

                // Add edges
                if i < trace.events.len() - 1 {
                    let next_activity = &trace.events[i + 1].activity;

                    // Add next node
                    if node_set.insert(next_activity.clone()) {
                        dfg.nodes.push(next_activity.clone());
                    }

                    // Find or create edge using HashMap (O(1) instead of O(m))
                    let edge_key = (activity.clone(), next_activity.clone());
                    if let Some(&edge_idx) = edge_map.get(&edge_key) {
                        dfg.edges[edge_idx].frequency += 1;
                    } else {
                        edge_map.insert(edge_key, dfg.edges.len());
                        dfg.edges.push(DFGEdge::new(activity, next_activity));
                    }
                }
            }
        }

        dfg.nodes.sort();
        dfg
    }

    /// Filter edges by minimum frequency
    pub fn filter_edges(&mut self, min_frequency: usize) {
        self.edges.retain(|e| e.frequency >= min_frequency);

        // Remove isolated nodes (nodes not participating in any edges)
        let used_activities: std::collections::HashSet<_> = self
            .edges
            .iter()
            .flat_map(|e| vec![&e.from, &e.to])
            .cloned()
            .collect();

        // Keep only nodes that have edges
        self.nodes.retain(|n| used_activities.contains(n));
    }

    /// Get edges from activity (optimized with pre-computed indices)
    pub fn get_edges_from(&self, activity: &str) -> Vec<&DFGEdge> {
        // Use iterator instead of collecting all results
        self.edges.iter().filter(|e| e.from == activity).collect()
    }

    /// Get edges to activity (optimized with pre-computed indices)
    pub fn get_edges_to(&self, activity: &str) -> Vec<&DFGEdge> {
        self.edges.iter().filter(|e| e.to == activity).collect()
    }

    /// Build outgoing edges index for faster lookups (call this after construction)
    pub fn build_outgoing_index(&self) -> HashMap<String, Vec<usize>> {
        let mut index: HashMap<String, Vec<usize>> = HashMap::new();
        for (idx, edge) in self.edges.iter().enumerate() {
            index.entry(edge.from.clone()).or_default().push(idx);
        }
        index
    }

    /// Build incoming edges index for faster lookups (call this after construction)
    pub fn build_incoming_index(&self) -> HashMap<String, Vec<usize>> {
        let mut index: HashMap<String, Vec<usize>> = HashMap::new();
        for (idx, edge) in self.edges.iter().enumerate() {
            index.entry(edge.to.clone()).or_default().push(idx);
        }
        index
    }

    /// Get parallel activities (activities that can occur in different order)
    /// Optimized using HashMap lookup instead of O(n²) nested loop
    pub fn parallel_activities(&self) -> Vec<(String, String)> {
        let mut parallels = Vec::new();

        // Create a map of edges for O(1) lookup
        let mut edge_set = HashSet::new();
        for edge in &self.edges {
            edge_set.insert((edge.from.as_str(), edge.to.as_str()));
        }

        // Check each edge for reverse edge
        let mut seen = HashSet::new();
        for edge in &self.edges {
            let reverse_key = (edge.to.as_str(), edge.from.as_str());
            let forward_key = (edge.from.clone(), edge.to.clone());

            if edge_set.contains(&reverse_key) && !seen.contains(&forward_key) {
                parallels.push(forward_key.clone());
                seen.insert(forward_key);
                seen.insert((edge.to.clone(), edge.from.clone()));
            }
        }

        parallels
    }

    /// Get choice points (activities with multiple outgoing edges)
    pub fn choice_points(&self) -> Vec<&str> {
        let mut choices = Vec::new();

        for node in &self.nodes {
            let outgoing = self.get_edges_from(node);
            if outgoing.len() > 1 {
                choices.push(node.as_str());
            }
        }

        choices
    }

    /// Check for loops
    pub fn has_loop(&self) -> bool {
        for node in &self.nodes {
            // Check if there's a path from any neighbor back to this node
            for edge in self.get_edges_from(node) {
                if self.has_path_to(&edge.to, node) {
                    return true;
                }
            }
        }
        false
    }

    /// Check if there's a path from source to target
    fn has_path_to(&self, source: &str, target: &str) -> bool {
        if source == target {
            return false;
        }

        let mut visited = std::collections::HashSet::new();
        let mut queue = vec![source];

        while let Some(current) = queue.pop() {
            if visited.contains(current) {
                continue;
            }
            visited.insert(current);

            for edge in self.get_edges_from(current) {
                if edge.to == target {
                    return true;
                }
                if !visited.contains(edge.to.as_str()) {
                    queue.push(&edge.to);
                }
            }
        }

        false
    }

    /// Calculate density of the graph
    pub fn density(&self) -> f64 {
        if self.nodes.len() <= 1 {
            return 0.0;
        }

        let max_edges = (self.nodes.len() * (self.nodes.len() - 1)) as f64;
        self.edges.len() as f64 / max_edges
    }
}

impl Default for DirectlyFollowsGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::{Event, EventLog, Trace};
    use chrono::{Duration, Utc};

    fn create_test_log() -> EventLog {
        let mut log = EventLog::new();
        let mut trace = Trace::new("case_1");
        let now = Utc::now();

        trace.add_event(Event::new("a", now));
        trace.add_event(Event::new("b", now + Duration::seconds(1)));
        trace.add_event(Event::new("c", now + Duration::seconds(2)));

        log.add_trace(trace);
        log
    }

    // ========== BASIC TESTS ==========

    #[test]
    fn test_dfg_edge_creation() {
        let edge = DFGEdge::new("A", "B");
        assert_eq!(edge.from, "A");
        assert_eq!(edge.to, "B");
        assert_eq!(edge.frequency, 1);
    }

    #[test]
    fn test_dfg_new() {
        let dfg = DirectlyFollowsGraph::new();
        assert_eq!(dfg.nodes.len(), 0);
        assert_eq!(dfg.edges.len(), 0);
        assert!(dfg.start_activities.is_empty());
    }

    #[test]
    fn test_dfg_default() {
        let dfg = DirectlyFollowsGraph::default();
        assert_eq!(dfg.nodes.len(), 0);
        assert_eq!(dfg.edges.len(), 0);
    }

    #[test]
    fn test_dfg_creation() {
        let log = create_test_log();
        let dfg = DirectlyFollowsGraph::from_log(&log);

        assert_eq!(dfg.nodes.len(), 3);
        assert_eq!(dfg.edges.len(), 2);
        assert_eq!(dfg.nodes, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_dfg_from_empty_log() {
        let log = EventLog::new();
        let dfg = DirectlyFollowsGraph::from_log(&log);

        assert_eq!(dfg.nodes.len(), 0);
        assert_eq!(dfg.edges.len(), 0);
        assert!(dfg.start_activities.is_empty());
    }

    // ========== START/END ACTIVITY TESTS ==========

    #[test]
    fn test_start_end_activities() {
        let log = create_test_log();
        let dfg = DirectlyFollowsGraph::from_log(&log);

        assert_eq!(dfg.start_activities.get("a"), Some(&1));
        assert_eq!(dfg.end_activities.get("c"), Some(&1));
    }

    #[test]
    fn test_start_end_multiple_traces() {
        let mut log = EventLog::new();
        let now = Utc::now();

        for i in 0..3 {
            let mut trace = Trace::new(format!("case_{}", i));
            trace.add_event(Event::new("a", now));
            trace.add_event(Event::new("b", now + Duration::seconds(1)));
            trace.add_event(Event::new("c", now + Duration::seconds(2)));
            log.add_trace(trace);
        }

        let dfg = DirectlyFollowsGraph::from_log(&log);
        assert_eq!(dfg.start_activities.get("a"), Some(&3));
        assert_eq!(dfg.end_activities.get("c"), Some(&3));
    }

    #[test]
    fn test_start_end_different_activities() {
        let mut log = EventLog::new();
        let now = Utc::now();

        let mut trace1 = Trace::new("case_1");
        trace1.add_event(Event::new("a", now));
        trace1.add_event(Event::new("b", now + Duration::seconds(1)));
        log.add_trace(trace1);

        let mut trace2 = Trace::new("case_2");
        trace2.add_event(Event::new("x", now));
        trace2.add_event(Event::new("y", now + Duration::seconds(1)));
        log.add_trace(trace2);

        let dfg = DirectlyFollowsGraph::from_log(&log);
        assert_eq!(dfg.start_activities.len(), 2);
        assert_eq!(dfg.end_activities.len(), 2);
    }

    // ========== EDGE AND FREQUENCY TESTS ==========

    #[test]
    fn test_edge_frequency_counting() {
        let mut log = EventLog::new();
        let now = Utc::now();

        for _ in 0..5 {
            let mut trace = Trace::new("case");
            trace.add_event(Event::new("a", now));
            trace.add_event(Event::new("b", now + Duration::seconds(1)));
            log.add_trace(trace);
        }

        let dfg = DirectlyFollowsGraph::from_log(&log);
        let a_to_b = dfg.edges.iter().find(|e| e.from == "a" && e.to == "b");
        assert_eq!(a_to_b.unwrap().frequency, 5);
    }

    #[test]
    fn test_activity_frequency() {
        let mut log = EventLog::new();
        let now = Utc::now();

        for _ in 0..4 {
            let mut trace = Trace::new("case");
            trace.add_event(Event::new("a", now));
            trace.add_event(Event::new("b", now + Duration::seconds(1)));
            log.add_trace(trace);
        }

        let dfg = DirectlyFollowsGraph::from_log(&log);
        assert_eq!(dfg.activity_frequency.get("a"), Some(&4));
        assert_eq!(dfg.activity_frequency.get("b"), Some(&4));
    }

    #[test]
    fn test_multiple_edges() {
        let mut log = EventLog::new();
        let now = Utc::now();

        let mut trace = Trace::new("case_1");
        trace.add_event(Event::new("a", now));
        trace.add_event(Event::new("b", now + Duration::seconds(1)));
        trace.add_event(Event::new("c", now + Duration::seconds(2)));
        log.add_trace(trace);

        let dfg = DirectlyFollowsGraph::from_log(&log);
        assert_eq!(dfg.edges.len(), 2);
        assert!(dfg.edges.iter().any(|e| e.from == "a" && e.to == "b"));
        assert!(dfg.edges.iter().any(|e| e.from == "b" && e.to == "c"));
    }

    // ========== FILTER TESTS ==========

    #[test]
    fn test_filter_edges_by_frequency() {
        let mut log = EventLog::new();
        let now = Utc::now();

        // A -> B: 5 times
        for _ in 0..5 {
            let mut trace = Trace::new("case");
            trace.add_event(Event::new("a", now));
            trace.add_event(Event::new("b", now + Duration::seconds(1)));
            log.add_trace(trace);
        }

        // A -> C: 1 time
        let mut trace = Trace::new("rare");
        trace.add_event(Event::new("a", now));
        trace.add_event(Event::new("c", now + Duration::seconds(1)));
        log.add_trace(trace);

        let mut dfg = DirectlyFollowsGraph::from_log(&log);
        assert_eq!(dfg.edges.len(), 2);

        dfg.filter_edges(2);
        assert_eq!(dfg.edges.len(), 1);
        assert!(dfg.edges.iter().any(|e| e.from == "a" && e.to == "b"));
    }

    #[test]
    fn test_filter_removes_isolated_nodes() {
        let mut log = EventLog::new();
        let now = Utc::now();

        let mut trace1 = Trace::new("case_1");
        trace1.add_event(Event::new("a", now));
        trace1.add_event(Event::new("b", now + Duration::seconds(1)));
        log.add_trace(trace1);

        let mut trace2 = Trace::new("case_2");
        trace2.add_event(Event::new("x", now));
        log.add_trace(trace2);

        let mut dfg = DirectlyFollowsGraph::from_log(&log);
        assert!(dfg.nodes.contains(&"x".to_string()));

        dfg.filter_edges(1);
        // x should be removed as it has no edges
        assert!(!dfg.nodes.contains(&"x".to_string()));
    }

    // ========== GRAPH ANALYSIS TESTS ==========

    #[test]
    fn test_get_edges_from() {
        let log = create_test_log();
        let dfg = DirectlyFollowsGraph::from_log(&log);

        let edges_from_a = dfg.get_edges_from("a");
        assert_eq!(edges_from_a.len(), 1);
        assert_eq!(edges_from_a[0].to, "b");
    }

    #[test]
    fn test_get_edges_to() {
        let log = create_test_log();
        let dfg = DirectlyFollowsGraph::from_log(&log);

        let edges_to_c = dfg.get_edges_to("c");
        assert_eq!(edges_to_c.len(), 1);
        assert_eq!(edges_to_c[0].from, "b");
    }

    #[test]
    fn test_choice_points() {
        let mut log = EventLog::new();
        let now = Utc::now();

        let mut trace1 = Trace::new("case_1");
        trace1.add_event(Event::new("a", now));
        trace1.add_event(Event::new("b", now + Duration::seconds(1)));
        trace1.add_event(Event::new("d", now + Duration::seconds(2)));
        log.add_trace(trace1);

        let mut trace2 = Trace::new("case_2");
        trace2.add_event(Event::new("a", now));
        trace2.add_event(Event::new("c", now + Duration::seconds(1)));
        trace2.add_event(Event::new("d", now + Duration::seconds(2)));
        log.add_trace(trace2);

        let dfg = DirectlyFollowsGraph::from_log(&log);
        let choices = dfg.choice_points();

        assert!(choices.contains(&"a"));
        assert!(!choices.contains(&"b"));
        assert!(!choices.contains(&"c"));
    }

    #[test]
    fn test_no_choice_points() {
        let log = create_test_log();
        let dfg = DirectlyFollowsGraph::from_log(&log);

        let choices = dfg.choice_points();
        assert!(choices.is_empty());
    }

    #[test]
    fn test_parallel_activities() {
        let mut log = EventLog::new();
        let now = Utc::now();

        let mut trace1 = Trace::new("case_1");
        trace1.add_event(Event::new("a", now));
        trace1.add_event(Event::new("b", now + Duration::seconds(1)));
        trace1.add_event(Event::new("c", now + Duration::seconds(2)));
        log.add_trace(trace1);

        let mut trace2 = Trace::new("case_2");
        trace2.add_event(Event::new("a", now));
        trace2.add_event(Event::new("c", now + Duration::seconds(1)));
        trace2.add_event(Event::new("b", now + Duration::seconds(2)));
        log.add_trace(trace2);

        let dfg = DirectlyFollowsGraph::from_log(&log);
        let parallel = dfg.parallel_activities();

        assert!(
            parallel.contains(&("b".to_string(), "c".to_string()))
                || parallel.contains(&("c".to_string(), "b".to_string()))
        );
    }

    #[test]
    fn test_has_loop_true() {
        let mut log = EventLog::new();
        let now = Utc::now();

        let mut trace = Trace::new("case_1");
        trace.add_event(Event::new("a", now));
        trace.add_event(Event::new("b", now + Duration::seconds(1)));
        trace.add_event(Event::new("a", now + Duration::seconds(2))); // Loop back to a
        trace.add_event(Event::new("c", now + Duration::seconds(3)));
        log.add_trace(trace);

        let dfg = DirectlyFollowsGraph::from_log(&log);
        assert!(dfg.has_loop());
    }

    #[test]
    fn test_has_loop_false() {
        let log = create_test_log();
        let dfg = DirectlyFollowsGraph::from_log(&log);
        assert!(!dfg.has_loop());
    }

    #[test]
    fn test_density() {
        let log = create_test_log();
        let dfg = DirectlyFollowsGraph::from_log(&log);

        let density = dfg.density();
        assert!(density >= 0.0 && density <= 1.0);
    }

    // ========== EDGE CASES ==========

    #[test]
    fn test_single_activity_trace() {
        let mut log = EventLog::new();
        let mut trace = Trace::new("case_1");
        let now = Utc::now();
        trace.add_event(Event::new("a", now));
        log.add_trace(trace);

        let dfg = DirectlyFollowsGraph::from_log(&log);
        assert_eq!(dfg.nodes.len(), 1);
        assert_eq!(dfg.edges.len(), 0);
        assert_eq!(dfg.start_activities.get("a"), Some(&1));
        assert_eq!(dfg.end_activities.get("a"), Some(&1));
    }

    #[test]
    fn test_duplicate_consecutive_activities() {
        let mut log = EventLog::new();
        let now = Utc::now();

        let mut trace = Trace::new("case_1");
        trace.add_event(Event::new("a", now));
        trace.add_event(Event::new("a", now + Duration::seconds(1))); // Duplicate
        trace.add_event(Event::new("b", now + Duration::seconds(2)));
        log.add_trace(trace);

        let dfg = DirectlyFollowsGraph::from_log(&log);
        assert!(dfg.edges.iter().any(|e| e.from == "a" && e.to == "a"));
    }
}
