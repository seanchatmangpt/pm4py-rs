//! Directly-Follows Graph (DFG) Abstraction to Plain English
//!
//! Converts DFG structures into concise descriptions of activity flow patterns,
//! including frequency analysis, dominant transitions, and rework detection.

use crate::models::DirectlyFollowsGraph;
use std::collections::HashMap;

/// Rework pattern information
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct ReworkPattern {
    from: String,
    to: String,
    occurrence_count: usize,
}

/// Analyze DFG and return plain English abstraction
///
/// Output: "Top activities: Review (523), Approve (521), Document (478).
/// Frequent transitions: Review→Approve (94%), Approve→Close (89%).
/// Detected 1 loop: Approve→Review→Approve (12 cases)."
pub fn abstract_dfg(dfg: &DirectlyFollowsGraph) -> String {
    let mut output = Vec::new();

    // Top activities by frequency
    let top_activities = get_top_activities(&dfg.activity_frequency, 5);
    if !top_activities.is_empty() {
        let activity_str = top_activities
            .iter()
            .map(|(name, count)| format!("{} ({})", name, count))
            .collect::<Vec<_>>()
            .join(", ");
        output.push(format!("Top activities: {}.", activity_str));
    }

    // Most frequent transitions
    let frequent_transitions = get_frequent_transitions(&dfg.edges, 5);
    if !frequent_transitions.is_empty() {
        let _total_edges: usize = dfg.edges.iter().map(|e| e.frequency).sum();
        let transition_str = frequent_transitions
            .iter()
            .map(|(from, to, _freq, pct)| format!("{}→{} ({}%)", from, to, pct))
            .collect::<Vec<_>>()
            .join(", ");
        output.push(format!("Frequent transitions: {}.", transition_str));
    }

    // Detect rework loops
    let loops = detect_rework_loops(dfg);
    if !loops.is_empty() {
        let loop_str = loops
            .iter()
            .enumerate()
            .map(|(i, (from, to, count))| {
                if i < 3 {
                    format!("{}→{}→{} ({} cases)", from, to, from, count)
                } else {
                    String::new()
                }
            })
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(", ");

        if !loop_str.is_empty() {
            output.push(format!(
                "Detected {} rework loop(s): {}.",
                loops.len(),
                loop_str
            ));
        }
    }

    // Activity count summary
    output.push(format!(
        "Summary: {} unique activities, {} transition types, {} total edges.",
        dfg.nodes.len(),
        dfg.edges.len(),
        dfg.edges.iter().map(|e| e.frequency).sum::<usize>()
    ));

    output.join(" ")
}

/// Get top N activities by frequency
fn get_top_activities(
    activity_frequency: &std::collections::BTreeMap<String, usize>,
    n: usize,
) -> Vec<(String, usize)> {
    let mut activities: Vec<_> = activity_frequency
        .iter()
        .map(|(name, count)| (name.clone(), *count))
        .collect();

    activities.sort_by(|a, b| b.1.cmp(&a.1));
    activities.into_iter().take(n).collect()
}

/// Get most frequent transitions with percentage
fn get_frequent_transitions(
    edges: &[crate::models::DFGEdge],
    n: usize,
) -> Vec<(String, String, usize, u32)> {
    let total_frequency: usize = edges.iter().map(|e| e.frequency).sum();

    if total_frequency == 0 {
        return Vec::new();
    }

    let mut transitions: Vec<_> = edges
        .iter()
        .map(|e| {
            let percentage = ((e.frequency as f64 / total_frequency as f64) * 100.0) as u32;
            (e.from.clone(), e.to.clone(), e.frequency, percentage)
        })
        .collect();

    transitions.sort_by(|a, b| b.2.cmp(&a.2));
    transitions.into_iter().take(n).collect()
}

/// Detect rework loops (activities flowing back to earlier activities)
fn detect_rework_loops(dfg: &DirectlyFollowsGraph) -> Vec<(String, String, usize)> {
    let mut loops = Vec::new();

    // Build reachability map: for each activity, what can it reach?
    let mut reachability: HashMap<String, Vec<String>> = HashMap::new();

    for node in &dfg.nodes {
        let reachable = find_reachable(dfg, node);
        reachability.insert(node.clone(), reachable);
    }

    // Find cycles: if A→B and B can reach A, it's a loop
    for edge in &dfg.edges {
        if let Some(reachable) = reachability.get(&edge.to) {
            if reachable.contains(&edge.from) {
                loops.push((edge.from.clone(), edge.to.clone(), edge.frequency));
            }
        }
    }

    loops.sort_by(|a, b| b.2.cmp(&a.2));
    loops
}

/// Find all activities reachable from a given starting activity
fn find_reachable(dfg: &DirectlyFollowsGraph, start: &str) -> Vec<String> {
    let mut visited = std::collections::HashSet::new();
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(start.to_string());

    while let Some(current) = queue.pop_front() {
        if visited.contains(&current) {
            continue;
        }
        visited.insert(current.clone());

        // Find all edges outgoing from current activity
        for edge in &dfg.edges {
            if edge.from == current && !visited.contains(&edge.to) {
                queue.push_back(edge.to.clone());
            }
        }
    }

    visited.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::DFGEdge;

    #[test]
    fn test_abstract_dfg_simple() {
        let mut dfg = DirectlyFollowsGraph::new();

        // Add activities
        dfg.activity_frequency.insert("Review".to_string(), 100);
        dfg.activity_frequency.insert("Approve".to_string(), 95);
        dfg.activity_frequency.insert("Close".to_string(), 90);

        // Add edges
        let edge1 = DFGEdge::new("Review", "Approve");
        let edge2 = DFGEdge::new("Approve", "Close");
        dfg.edges.push(edge1);
        dfg.edges.push(edge2);
        dfg.nodes = vec![
            "Review".to_string(),
            "Approve".to_string(),
            "Close".to_string(),
        ];

        let abstract_str = abstract_dfg(&dfg);
        assert!(abstract_str.contains("Review"));
        assert!(abstract_str.contains("Approve"));
        assert!(abstract_str.contains("activities"));
    }

    #[test]
    fn test_get_top_activities() {
        let mut freq = std::collections::BTreeMap::new();
        freq.insert("A".to_string(), 100);
        freq.insert("B".to_string(), 50);
        freq.insert("C".to_string(), 25);

        let top = get_top_activities(&freq, 2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].0, "A");
        assert_eq!(top[1].0, "B");
    }
}
