//! Extended statistics functions for event logs
//!
//! Additional statistical analysis beyond basic metrics.

use crate::log::EventLog;
use std::collections::HashMap;

/// Get activity position summary in traces
pub fn get_activity_position_summary(log: &EventLog) -> HashMap<String, Vec<usize>> {
    let mut positions: HashMap<String, Vec<usize>> = HashMap::new();

    for trace in &log.traces {
        for (idx, event) in trace.events.iter().enumerate() {
            positions
                .entry(event.activity.clone())
                .or_default()
                .push(idx);
        }
    }

    positions
}

/// Get frequent trace segments (n-grams)
pub fn get_frequent_trace_segments(
    log: &EventLog,
    min_length: usize,
    max_length: usize,
    min_support: usize,
) -> Vec<(Vec<String>, usize)> {
    let mut segment_counts: HashMap<Vec<String>, usize> = HashMap::new();

    for trace in &log.traces {
        let activities: Vec<String> = trace.events.iter().map(|e| e.activity.clone()).collect();

        for len in min_length..=max_length.min(activities.len()) {
            for start in 0..=(activities.len().saturating_sub(len)) {
                if start + len <= activities.len() {
                    let segment: Vec<String> = activities[start..start + len].to_vec();
                    *segment_counts.entry(segment).or_insert(0) += 1;
                }
            }
        }
    }

    segment_counts
        .into_iter()
        .filter(|(_, count)| *count >= min_support)
        .collect()
}

/// Get case arrival rate (average time between case starts)
pub fn get_case_arrival_average(log: &EventLog) -> Option<f64> {
    let mut start_times: Vec<chrono::DateTime<chrono::Utc>> = Vec::new();

    for trace in &log.traces {
        if let Some(first_event) = trace.events.first() {
            start_times.push(first_event.timestamp);
        }
    }

    if start_times.len() < 2 {
        return None;
    }

    start_times.sort();

    let mut total_diff = 0.0;
    for window in start_times.windows(2) {
        let diff = (window[1] - window[0]).num_seconds();
        total_diff += diff as f64;
    }

    Some(total_diff / (start_times.len() - 1) as f64)
}

/// Get case overlap (concurrent cases)
pub fn get_case_overlap(log: &EventLog) -> f64 {
    if log.traces.is_empty() {
        return 0.0;
    }

    let mut intervals = Vec::new();

    for trace in &log.traces {
        if let (Some(first), Some(last)) = (trace.events.first(), trace.events.last()) {
            intervals.push((first.timestamp, last.timestamp));
        }
    }

    let mut overlap_count = 0;
    let mut total_pairs = 0;

    for i in 0..intervals.len() {
        for j in (i + 1)..intervals.len() {
            total_pairs += 1;

            let (start1, end1) = intervals[i];
            let (start2, end2) = intervals[j];

            // Check if intervals overlap
            let overlap = start1 < end2 && start2 < end1;
            if overlap {
                overlap_count += 1;
            }
        }
    }

    if total_pairs == 0 {
        0.0
    } else {
        overlap_count as f64 / total_pairs as f64
    }
}

/// Get all unique prefixes from the log
pub fn get_prefixes_from_log(log: &EventLog, max_length: usize) -> Vec<Vec<String>> {
    let mut prefixes: std::collections::HashSet<Vec<String>> = std::collections::HashSet::new();

    for trace in &log.traces {
        let activities: Vec<String> = trace.events.iter().map(|e| e.activity.clone()).collect();

        for len in 1..=max_length.min(activities.len()) {
            let prefix: Vec<String> = activities[..len].to_vec();
            prefixes.insert(prefix);
        }
    }

    let mut result: Vec<Vec<String>> = prefixes.into_iter().collect();
    result.sort();
    result
}

/// Get variants as tuples (instead of comma-separated strings)
pub fn get_variants_as_tuples(log: &EventLog) -> Vec<(Vec<String>, usize)> {
    let mut variant_counts: HashMap<Vec<String>, usize> = HashMap::new();

    for trace in &log.traces {
        let variant: Vec<String> = trace.events.iter().map(|e| e.activity.clone()).collect();
        *variant_counts.entry(variant).or_insert(0) += 1;
    }

    let mut variants: Vec<_> = variant_counts.into_iter().collect();
    variants.sort_by(|a, b| b.1.cmp(&a.1));
    variants
}

/// Get rework cases per activity
pub fn get_rework_cases_per_activity(log: &EventLog) -> HashMap<String, Vec<String>> {
    let mut rework_cases: HashMap<String, Vec<String>> = HashMap::new();

    for trace in &log.traces {
        let case_id = trace
            .attributes
            .get("concept:name")
            .cloned()
            .unwrap_or_else(|| format!("case_{}", trace.id));

        let mut activity_counts: HashMap<String, usize> = HashMap::new();
        for event in &trace.events {
            *activity_counts.entry(event.activity.clone()).or_insert(0) += 1;
        }

        for (activity, count) in activity_counts {
            if count > 1 {
                rework_cases
                    .entry(activity)
                    .or_default()
                    .push(case_id.clone());
            }
        }
    }

    rework_cases
}

/// Discover a BPMN diagram from a process tree using recursive conversion.
///
/// Each `ProcessTreeNode` maps to a BPMN construct:
/// - `Activity` → `Task`
/// - `Sequence` → chain of tasks/gateways
/// - `Choice` → XOR-split gateway + XOR-join gateway
/// - `Parallel` → AND-split gateway + AND-join gateway
/// - `Loop` → XOR-gateway with a back-edge from body exit to loop entry
///
/// The returned diagram always has a start event and an end event.
pub fn discover_bpmn_inductive(tree: &crate::models::ProcessTree) -> crate::models::BPMNDiagram {
    use crate::models::bpmn::{Event, EventType, SequenceFlow};

    let name = tree
        .name
        .as_deref()
        .unwrap_or("Discovered Process")
        .to_string();
    let mut bpmn = crate::models::BPMNDiagram::new(&name);

    // Add start event
    let start_event = Event::new("Start", EventType::Start);
    let start_id = bpmn.add_event(start_event);

    // Recursively convert the root node, threading from start_id
    let exit_id = convert_tree_node_to_bpmn(&tree.root, &mut bpmn, &start_id);

    // Add end event and connect from the tree exit
    let end_event = Event::new("End", EventType::End);
    let end_id = bpmn.add_event(end_event);
    bpmn.add_flow(SequenceFlow::new(&exit_id, &end_id));

    bpmn
}

/// Recursively convert a `ProcessTreeNode` into BPMN elements.
///
/// Adds nodes and flows to `diagram`.  `entry_id` is the ID of the node
/// from which flow enters this sub-tree.  Returns the ID of the node
/// from which flow exits (the last element of the sub-tree).
fn convert_tree_node_to_bpmn(
    node: &crate::models::ProcessTreeNode,
    diagram: &mut crate::models::BPMNDiagram,
    entry_id: &str,
) -> String {
    use crate::models::bpmn::{Gateway, GatewayType, SequenceFlow, Task, TaskType};
    use crate::models::{ProcessTreeNode, TreeOperator};

    match node {
        // Leaf: create a Task and connect from entry
        ProcessTreeNode::Activity(name) => {
            let task = Task::new(name, TaskType::AutomaticTask);
            let task_id = diagram.add_task(task);
            diagram.add_flow(SequenceFlow::new(entry_id, task_id.as_str()));
            task_id
        }

        ProcessTreeNode::Operator {
            operator, children, ..
        } => match operator {
            // Sequence: thread children one after another
            TreeOperator::Sequence => {
                let mut current = entry_id.to_string();
                for child in children {
                    current = convert_tree_node_to_bpmn(child, diagram, &current);
                }
                current
            }

            // Choice (XOR): split gateway → branches → join gateway
            TreeOperator::Choice => {
                let split = Gateway::new("XOR-split", GatewayType::ExclusiveXor);
                let split_id = diagram.add_gateway(split);
                diagram.add_flow(SequenceFlow::new(entry_id, split_id.as_str()));

                let join = Gateway::new("XOR-join", GatewayType::ExclusiveXor);
                let join_id = diagram.add_gateway(join);

                for child in children {
                    let branch_exit = convert_tree_node_to_bpmn(child, diagram, &split_id);
                    diagram.add_flow(SequenceFlow::new(branch_exit.as_str(), join_id.as_str()));
                }
                join_id
            }

            // Parallel (AND): split gateway → parallel branches → join gateway
            TreeOperator::Parallel => {
                let split = Gateway::new("AND-split", GatewayType::Parallel);
                let split_id = diagram.add_gateway(split);
                diagram.add_flow(SequenceFlow::new(entry_id, split_id.as_str()));

                let join = Gateway::new("AND-join", GatewayType::Parallel);
                let join_id = diagram.add_gateway(join);

                for child in children {
                    let branch_exit = convert_tree_node_to_bpmn(child, diagram, &split_id);
                    diagram.add_flow(SequenceFlow::new(branch_exit.as_str(), join_id.as_str()));
                }
                join_id
            }

            // Loop: XOR gateway acts as loop-back point; body connects back to it
            TreeOperator::Loop => {
                let loop_gw = Gateway::new("Loop", GatewayType::ExclusiveXor);
                let loop_gw_id = diagram.add_gateway(loop_gw);
                diagram.add_flow(SequenceFlow::new(entry_id, loop_gw_id.as_str()));

                // First child is the loop body; subsequent children are the redo/exit parts
                if let Some(body) = children.first() {
                    let body_exit = convert_tree_node_to_bpmn(body, diagram, &loop_gw_id);
                    // Back-edge: body exit → loop gateway (loop repetition)
                    diagram.add_flow(SequenceFlow::new(body_exit.as_str(), loop_gw_id.as_str()));
                }
                loop_gw_id
            }
        },
    }
}
