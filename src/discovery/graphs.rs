//! Extended graph discovery functions
//!
//! Implements additional discovery algorithms for process mining.

use crate::log::EventLog;
use crate::models::DirectlyFollowsGraph;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Eventually-Follows Graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventuallyFollowsGraph {
    pub activities: Vec<String>,
    pub edges: HashMap<(String, String), usize>,
}

impl EventuallyFollowsGraph {
    pub fn new() -> Self {
        Self {
            activities: Vec::new(),
            edges: HashMap::new(),
        }
    }
}

impl Default for EventuallyFollowsGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance DFG with timing metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceDFG {
    pub activities: Vec<String>,
    pub frequency: HashMap<(String, String), usize>,
    pub performance: HashMap<(String, String), PerformanceMetrics>,
}

impl PerformanceDFG {
    pub fn new() -> Self {
        Self {
            activities: Vec::new(),
            frequency: HashMap::new(),
            performance: HashMap::new(),
        }
    }
}

impl Default for PerformanceDFG {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance metrics for a DFG edge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub mean: i64,
    pub min: i64,
    pub max: i64,
    pub median: i64,
}

/// Discover basic directly-follows graph from event log
///
/// Returns a graph showing which activities directly follow each other.
/// Uses the existing DirectlyFollowsGraph structure.
pub fn directly_follows_graph(log: &EventLog) -> DirectlyFollowsGraph {
    DirectlyFollowsGraph::from_log(log)
}

/// Discover eventually-follows graph from event log
///
/// Returns a graph showing which activities eventually follow each other
/// (transitive closure of directly-follows).
pub fn eventually_follows_graph(log: &EventLog) -> EventuallyFollowsGraph {
    let mut graph = EventuallyFollowsGraph::new();
    let mut activity_set: HashSet<String> = HashSet::new();

    for trace in &log.traces {
        let activities: Vec<String> = trace.events.iter().map(|e| e.activity.clone()).collect();

        // For each pair (i, j) where i < j, activity[i] eventually follows activity[j]
        for i in 0..activities.len() {
            for j in (i + 1)..activities.len() {
                let from = activities[i].clone();
                let to = activities[j].clone();

                activity_set.insert(from.clone());
                activity_set.insert(to.clone());

                *graph.edges.entry((from, to)).or_insert(0) += 1;
            }
        }
    }

    graph.activities = activity_set.into_iter().collect();
    graph
}

/// Discover performance DFG from event log
///
/// Returns a directly-follows graph with timing information (aggregated
/// duration between activities).
pub fn discover_performance_dfg(log: &EventLog) -> PerformanceDFG {
    let mut dfg = PerformanceDFG::new();
    let mut durations: HashMap<(String, String), Vec<i64>> = HashMap::new();
    let mut activity_set: HashSet<String> = HashSet::new();

    for trace in &log.traces {
        for window in trace.events.windows(2) {
            let from = window[0].activity.clone();
            let to = window[1].activity.clone();
            let duration = window[1]
                .timestamp
                .signed_duration_since(window[0].timestamp)
                .num_milliseconds();

            activity_set.insert(from.clone());
            activity_set.insert(to.clone());

            durations
                .entry((from.clone(), to.clone()))
                .or_default()
                .push(duration);

            *dfg.frequency.entry((from, to)).or_insert(0) += 1;
        }
    }

    dfg.activities = activity_set.into_iter().collect();

    // Compute statistics for each edge
    for ((from, to), durs) in durations {
        let sum: i64 = durs.iter().sum();
        let avg = sum / durs.len() as i64;
        let min = durs.iter().min().copied().unwrap_or(0);
        let max = durs.iter().max().copied().unwrap_or(0);

        dfg.performance.insert(
            (from, to),
            PerformanceMetrics {
                mean: avg,
                min,
                max,
                median: median(&durs),
            },
        );
    }

    dfg
}

/// Compute median of a list of integers
fn median(values: &[i64]) -> i64 {
    if values.is_empty() {
        return 0;
    }

    let mut sorted = values.to_vec();
    sorted.sort();
    let len = sorted.len();

    if len % 2 == 0 {
        (sorted[len / 2 - 1] + sorted[len / 2]) / 2
    } else {
        sorted[len / 2]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::XESReader;
    use std::path::Path;

    #[test]
    fn test_directly_follows_graph() {
        let log = XESReader::new()
            .read(Path::new("/Users/sac/chatmangpt/test_simple.xes"))
            .unwrap();

        let graph = directly_follows_graph(&log);

        println!("Directly-follows graph:");
        println!("  Activities: {}", graph.nodes.len());
        println!("  Edges: {}", graph.edges.len());

        for edge in &graph.edges {
            println!(
                "    {} -> {} ({} times)",
                edge.from, edge.to, edge.frequency
            );
        }

        assert!(!graph.nodes.is_empty(), "Should have activities");
    }

    #[test]
    fn test_eventually_follows_graph() {
        let log = XESReader::new()
            .read(Path::new("/Users/sac/chatmangpt/test_simple.xes"))
            .unwrap();

        let graph = eventually_follows_graph(&log);

        println!("Eventually-follows graph:");
        println!("  Activities: {}", graph.activities.len());
        println!("  Edges: {}", graph.edges.len());

        assert!(!graph.activities.is_empty(), "Should have activities");
    }

    #[test]
    fn test_performance_dfg() {
        let log = XESReader::new()
            .read(Path::new("/Users/sac/chatmangpt/test_simple.xes"))
            .unwrap();

        let dfg = discover_performance_dfg(&log);

        println!("Performance DFG:");
        println!("  Frequency edges: {}", dfg.frequency.len());
        println!("  Performance edges: {}", dfg.performance.len());

        for ((from, to), perf) in &dfg.performance {
            println!(
                "    {} -> {}: mean={}ms, min={}ms, max={}ms",
                from, to, perf.mean, perf.min, perf.max
            );
        }

        assert!(!dfg.frequency.is_empty(), "Should have frequency data");
    }
}
