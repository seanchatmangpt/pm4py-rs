/// Conversion operations between Process Trees and Petri Nets
///
/// This module provides bidirectional conversion between process tree
/// and Petri net representations.
use crate::models::petri_net::{Arc, PetriNet, Place, Transition};
use crate::models::process_tree::{ProcessTree, ProcessTreeNode, TreeOperator};

/// Convert a process tree to a Petri net
///
/// This conversion creates a Petri net with visible transitions for
/// activities and invisible transitions for operators.
///
/// # Arguments
/// * `tree` - The process tree to convert
///
/// # Returns
/// A Petri net representation of the tree
pub fn tree_to_petri_net(tree: &ProcessTree) -> PetriNet {
    let mut net = PetriNet::new();

    // Create source place
    let source = Place::new("source").with_initial_marking(1);
    let source_id = source.id.clone();
    net.add_place(source);
    net.set_initial_place(source_id.clone());

    // Create sink place
    let sink = Place::new("sink");
    let sink_id = sink.id.clone();
    net.add_place(sink);
    net.set_final_place(sink_id.clone());

    // Convert tree recursively
    let mut place_counter = 0;
    let end_place = convert_node_to_petri(
        &tree.root,
        &mut net,
        &source_id,
        &sink_id,
        &mut place_counter,
    );

    // Connect last place to sink with an invisible transition
    let final_transition = Transition::new(format!("tau_{}", place_counter));
    let final_trans_id = final_transition.id.clone();
    net.add_transition(final_transition);
    net.add_arc(Arc::new(&end_place, &final_trans_id));
    net.add_arc(Arc::new(&final_trans_id, &sink_id));

    net
}

#[allow(clippy::only_used_in_recursion)]
fn convert_node_to_petri(
    node: &ProcessTreeNode,
    net: &mut PetriNet,
    in_place: &str,
    sink: &str,
    counter: &mut usize,
) -> String {
    match node {
        ProcessTreeNode::Activity(activity) => {
            // Create output place for this activity
            let out_place = Place::new(format!("p_{}", counter));
            *counter += 1;
            let out_place_id = out_place.id.clone();
            net.add_place(out_place);

            // Create transition for activity
            let trans = Transition::new(activity).with_label(activity);
            let trans_id = trans.id.clone();
            net.add_transition(trans);

            // Connect: in_place -> transition -> out_place
            net.add_arc(Arc::new(in_place, &trans_id));
            net.add_arc(Arc::new(&trans_id, &out_place_id));

            out_place_id
        }
        ProcessTreeNode::Operator {
            operator, children, ..
        } => {
            match operator {
                TreeOperator::Sequence => {
                    // Connect children sequentially
                    let mut current_place = in_place.to_string();

                    for child in children.iter() {
                        current_place =
                            convert_node_to_petri(child, net, &current_place, sink, counter);
                    }

                    current_place
                }
                TreeOperator::Choice => {
                    // Create XOR split and join
                    let split = Place::new(format!("p_{}", counter));
                    *counter += 1;
                    let split_id = split.id.clone();
                    net.add_place(split);

                    // Add invisible transition for split
                    let split_trans = Transition::new(format!("tau_{}", counter));
                    *counter += 1;
                    let split_trans_id = split_trans.id.clone();
                    net.add_transition(split_trans);
                    net.add_arc(Arc::new(in_place, &split_trans_id));
                    net.add_arc(Arc::new(&split_trans_id, &split_id));

                    // Convert each branch
                    let mut join_inputs = Vec::new();
                    for child in children {
                        let out = convert_node_to_petri(child, net, &split_id, sink, counter);
                        join_inputs.push(out);
                    }

                    // Create join place
                    let join = Place::new(format!("p_{}", counter));
                    *counter += 1;
                    let join_id = join.id.clone();
                    net.add_place(join);

                    // Add invisible transition for join
                    for input in join_inputs {
                        let join_trans = Transition::new(format!("tau_{}", counter));
                        *counter += 1;
                        let join_trans_id = join_trans.id.clone();
                        net.add_transition(join_trans);
                        net.add_arc(Arc::new(&input, &join_trans_id));
                        net.add_arc(Arc::new(&join_trans_id, &join_id));
                    }

                    join_id
                }
                TreeOperator::Parallel => {
                    // Create AND split and join
                    let split = Place::new(format!("p_{}", counter));
                    *counter += 1;
                    let split_id = split.id.clone();
                    net.add_place(split);

                    // Add invisible transition for split
                    let split_trans = Transition::new(format!("tau_{}", counter));
                    *counter += 1;
                    let split_trans_id = split_trans.id.clone();
                    net.add_transition(split_trans);
                    net.add_arc(Arc::new(in_place, &split_trans_id));
                    net.add_arc(Arc::new(&split_trans_id, &split_id));

                    // Convert each branch
                    let mut join_inputs = Vec::new();
                    for child in children {
                        let out = convert_node_to_petri(child, net, &split_id, sink, counter);
                        join_inputs.push(out);
                    }

                    // Create join place
                    let join = Place::new(format!("p_{}", counter));
                    *counter += 1;
                    let join_id = join.id.clone();
                    net.add_place(join);

                    // Add invisible transition for join
                    for input in join_inputs {
                        let join_trans = Transition::new(format!("tau_{}", counter));
                        *counter += 1;
                        let join_trans_id = join_trans.id.clone();
                        net.add_transition(join_trans);
                        net.add_arc(Arc::new(&input, &join_trans_id));
                        net.add_arc(Arc::new(&join_trans_id, &join_id));
                    }

                    join_id
                }
                TreeOperator::Loop => {
                    // Create loop with body and exit condition
                    if children.len() == 2 {
                        let body = &children[0];
                        let exit = &children[1];

                        // Create loop place
                        let loop_place = Place::new(format!("p_{}", counter));
                        *counter += 1;
                        let loop_place_id = loop_place.id.clone();
                        net.add_place(loop_place);

                        // Add invisible transition from entry to loop
                        let entry_trans = Transition::new(format!("tau_{}", counter));
                        *counter += 1;
                        let entry_trans_id = entry_trans.id.clone();
                        net.add_transition(entry_trans);
                        net.add_arc(Arc::new(in_place, &entry_trans_id));
                        net.add_arc(Arc::new(&entry_trans_id, &loop_place_id));

                        // Convert body
                        let body_out =
                            convert_node_to_petri(body, net, &loop_place_id, sink, counter);

                        // Create choice between repeat and exit
                        let choice = Place::new(format!("p_{}", counter));
                        *counter += 1;
                        let choice_id = choice.id.clone();
                        net.add_place(choice);

                        // Add invisible transition from body to choice
                        let body_trans = Transition::new(format!("tau_{}", counter));
                        *counter += 1;
                        let body_trans_id = body_trans.id.clone();
                        net.add_transition(body_trans);
                        net.add_arc(Arc::new(&body_out, &body_trans_id));
                        net.add_arc(Arc::new(&body_trans_id, &choice_id));

                        // Back to loop
                        let loop_trans = Transition::new(format!("tau_{}", counter));
                        *counter += 1;
                        let loop_trans_id = loop_trans.id.clone();
                        net.add_transition(loop_trans);
                        net.add_arc(Arc::new(&choice_id, &loop_trans_id));
                        net.add_arc(Arc::new(&loop_trans_id, &loop_place_id));

                        // Exit path
                        convert_node_to_petri(exit, net, &choice_id, sink, counter)
                    } else {
                        in_place.to_string()
                    }
                }
            }
        }
    }
}

/// Convert a Petri net to a process tree (simplified)
///
/// This is a simplified conversion that works well for structured nets.
/// Complex Petri nets may not convert perfectly.
///
/// # Arguments
/// * `net` - The Petri net to convert
///
/// # Returns
/// A process tree representation of the net
pub fn petri_net_to_tree(net: &PetriNet) -> ProcessTree {
    if net.transitions.is_empty() {
        return ProcessTree::new(ProcessTreeNode::activity("SKIP"));
    }

    // Find the source and sink places
    let source_id = net.initial_place.clone();
    let sink_id = net.final_place.clone();

    if source_id.is_none() || sink_id.is_none() {
        // Fallback: create sequence of all visible transitions
        let activities: Vec<ProcessTreeNode> = net
            .visible_transitions()
            .iter()
            .map(|t| ProcessTreeNode::activity(t.label.clone().unwrap_or_else(|| t.name.clone())))
            .collect();

        if activities.is_empty() {
            return ProcessTree::new(ProcessTreeNode::activity("SKIP"));
        } else if activities.len() == 1 {
            return ProcessTree::new(activities.into_iter().next().unwrap());
        } else {
            return ProcessTree::new(ProcessTreeNode::sequence(activities));
        }
    }

    let source = source_id.unwrap();
    let sink = sink_id.unwrap();

    // Try to reconstruct tree structure
    let root = reconstruct_tree_from_net(net, &source, &sink);

    ProcessTree::new(root)
}

#[allow(dead_code)]
fn follow_invisible_transitions(net: &PetriNet, trans_id: &str) -> Option<String> {
    // If a transition leads to a place, and that place only leads to invisible transitions,
    // follow through and return the next visible transition
    if let Some(trans) = net.get_transition(trans_id) {
        if trans.is_invisible() {
            let out_arcs = net.get_arcs_from(trans_id);
            if out_arcs.len() == 1 {
                let next_place = &out_arcs[0].to;
                let next_arcs = net.get_arcs_from(next_place);
                for arc in next_arcs {
                    if let Some(next_trans) = net.get_transition(&arc.to) {
                        if !next_trans.is_invisible() {
                            return Some(arc.to.clone());
                        }
                    }
                }
            }
        }
    }
    Some(trans_id.to_string())
}

fn reconstruct_tree_from_net(net: &PetriNet, current: &str, sink: &str) -> ProcessTreeNode {
    // Get outgoing arcs from current place
    let out_arcs = net.get_arcs_from(current);

    if out_arcs.is_empty() {
        return ProcessTreeNode::activity("SKIP");
    }

    // Get the transitions we can fire (skip invisible ones)
    let mut next_transitions = Vec::new();
    for arc in &out_arcs {
        if let Some(trans) = net.get_transition(&arc.to) {
            if !trans.is_invisible() {
                next_transitions.push(trans);
            }
        }
    }

    if next_transitions.is_empty() {
        return ProcessTreeNode::activity("SKIP");
    }

    // Simple heuristic: if one transition, follow it
    if next_transitions.len() == 1 {
        let trans = next_transitions[0];

        // Get the place after this transition
        let trans_arcs = net.get_arcs_from(&trans.id);
        if trans_arcs.is_empty() {
            let label = trans.label.clone().unwrap_or_else(|| trans.name.clone());
            return ProcessTreeNode::activity(label);
        }

        // Follow the output places
        if trans_arcs.len() == 1 {
            let next_place = &trans_arcs[0].to;
            let label = trans.label.clone().unwrap_or_else(|| trans.name.clone());

            if next_place == sink {
                ProcessTreeNode::activity(label)
            } else {
                let rest = reconstruct_tree_from_net(net, next_place, sink);
                ProcessTreeNode::sequence(vec![ProcessTreeNode::activity(label), rest])
            }
        } else {
            // Multiple outputs - treat as parallel
            let label = trans.label.clone().unwrap_or_else(|| trans.name.clone());
            let children: Vec<_> = trans_arcs
                .iter()
                .map(|a| reconstruct_tree_from_net(net, &a.to, sink))
                .collect();

            if children
                .iter()
                .all(|c| matches!(c, ProcessTreeNode::Activity(_)))
            {
                ProcessTreeNode::sequence(vec![
                    ProcessTreeNode::activity(label),
                    ProcessTreeNode::parallel(children),
                ])
            } else {
                ProcessTreeNode::activity(label)
            }
        }
    } else {
        // Multiple transitions - treat as choice
        let children: Vec<ProcessTreeNode> = next_transitions
            .iter()
            .map(|t| {
                let label = t.label.clone().unwrap_or_else(|| t.name.clone());
                let out_arcs = net.get_arcs_from(&t.id);

                if out_arcs.is_empty() {
                    ProcessTreeNode::activity(label)
                } else {
                    let next_place = &out_arcs[0].to;
                    if next_place == sink {
                        ProcessTreeNode::activity(label)
                    } else {
                        let rest = reconstruct_tree_from_net(net, next_place, sink);
                        ProcessTreeNode::sequence(vec![ProcessTreeNode::activity(label), rest])
                    }
                }
            })
            .collect();

        ProcessTreeNode::choice(children)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_activity_to_petri() {
        let tree = ProcessTree::new(ProcessTreeNode::activity("A"));
        let net = tree_to_petri_net(&tree);

        assert!(!net.transitions.is_empty());
        assert!(!net.places.is_empty());
        assert!(net.is_workflow_net());
    }

    #[test]
    fn test_sequence_to_petri() {
        let tree = ProcessTree::new(ProcessTreeNode::sequence(vec![
            ProcessTreeNode::activity("A"),
            ProcessTreeNode::activity("B"),
        ]));

        let net = tree_to_petri_net(&tree);

        assert_eq!(net.visible_transitions().len(), 2);
        assert!(net.is_workflow_net());
    }

    #[test]
    fn test_choice_to_petri() {
        let tree = ProcessTree::new(ProcessTreeNode::choice(vec![
            ProcessTreeNode::activity("A"),
            ProcessTreeNode::activity("B"),
        ]));

        let net = tree_to_petri_net(&tree);

        assert_eq!(net.visible_transitions().len(), 2);
    }

    #[test]
    fn test_parallel_to_petri() {
        let tree = ProcessTree::new(ProcessTreeNode::parallel(vec![
            ProcessTreeNode::activity("A"),
            ProcessTreeNode::activity("B"),
        ]));

        let net = tree_to_petri_net(&tree);

        assert_eq!(net.visible_transitions().len(), 2);
    }

    #[test]
    fn test_loop_to_petri() {
        let tree = ProcessTree::new(ProcessTreeNode::loop_node(
            ProcessTreeNode::activity("A"),
            ProcessTreeNode::activity("B"),
        ));

        let net = tree_to_petri_net(&tree);

        assert_eq!(net.visible_transitions().len(), 2);
    }

    #[test]
    fn test_complex_tree_to_petri() {
        let tree = ProcessTree::new(ProcessTreeNode::sequence(vec![
            ProcessTreeNode::activity("A"),
            ProcessTreeNode::choice(vec![
                ProcessTreeNode::activity("B"),
                ProcessTreeNode::activity("C"),
            ]),
            ProcessTreeNode::activity("D"),
        ]));

        let net = tree_to_petri_net(&tree);

        assert_eq!(net.visible_transitions().len(), 4);
        assert!(net.is_workflow_net());
    }

    #[test]
    fn test_petri_to_tree_simple() {
        let mut net = PetriNet::new();

        let source = Place::new("source").with_initial_marking(1);
        let source_id = source.id.clone();
        net.add_place(source);
        net.set_initial_place(source_id.clone());

        let sink = Place::new("sink");
        let sink_id = sink.id.clone();
        net.add_place(sink);
        net.set_final_place(sink_id.clone());

        let trans = Transition::new("A").with_label("A");
        let trans_id = trans.id.clone();
        net.add_transition(trans);

        net.add_arc(Arc::new(&source_id, &trans_id));
        net.add_arc(Arc::new(&trans_id, &sink_id));

        let tree = petri_net_to_tree(&net);
        assert_eq!(tree.activities().len(), 1);
    }

    #[test]
    fn test_bidirectional_conversion() {
        let original_tree = ProcessTree::new(ProcessTreeNode::sequence(vec![
            ProcessTreeNode::activity("A"),
            ProcessTreeNode::activity("B"),
        ]));

        let net = tree_to_petri_net(&original_tree);
        let recovered_tree = petri_net_to_tree(&net);

        // Verify conversion completes and produces valid output
        assert!(!net.places.is_empty());
        assert!(!net.transitions.is_empty());
        assert!(!recovered_tree.activities().is_empty());
        // Original activities should be present in recovered tree (may have additional SKIP activities)
        let original_acts = original_tree.activities();
        let recovered_acts = recovered_tree.activities();
        assert!(original_acts.iter().all(|a| recovered_acts.contains(a)));
    }
}
