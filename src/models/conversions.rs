//! Model Conversion Utilities
//!
//! Convert between different process model representations.

use crate::models::*;
use std::collections::{HashMap, HashSet};

/// Convert Petri net to process tree
pub fn petri_net_to_process_tree(net: &PetriNet) -> Option<ProcessTree> {
    if net.places.is_empty() || net.transitions.is_empty() {
        return None;
    }

    // Create a sequence tree from transitions
    let children: Vec<ProcessTreeNode> = net
        .transitions
        .iter()
        .filter_map(|t| t.label.as_ref())
        .map(|label| ProcessTreeNode::Activity(label.clone()))
        .collect();

    let root = if children.len() > 1 {
        ProcessTreeNode::Operator {
            operator: TreeOperator::Sequence,
            children,
            id: uuid::Uuid::new_v4().to_string(),
        }
    } else if children.len() == 1 {
        children[0].clone()
    } else {
        ProcessTreeNode::Activity("tau".to_string())
    };

    let tree = ProcessTree::new(root);

    Some(tree)
}

/// Convert process tree to Petri net
pub fn process_tree_to_petri_net(tree: &ProcessTree) -> PetriNet {
    let mut net = PetriNet::new();

    match &tree.root {
        ProcessTreeNode::Operator {
            operator, children, ..
        } => match operator {
            TreeOperator::Sequence => {
                let mut prev_place = None;
                for (i, _child) in children.iter().enumerate() {
                    let place = crate::models::petri_net::Place::new(format!("p_{}", i));
                    let place_id = place.id.clone();
                    net.add_place(place);
                    if let Some(prev_id) = prev_place {
                        let transition =
                            crate::models::petri_net::Transition::new(format!("tau_{}", i));
                        let transition_id = transition.id.clone();
                        net.add_transition(transition);
                        net.add_arc(crate::models::petri_net::Arc::new(
                            prev_id,
                            transition_id.clone(),
                        ));
                        net.add_arc(crate::models::petri_net::Arc::new(
                            transition_id,
                            place_id.clone(),
                        ));
                    }
                    prev_place = Some(place_id);
                }
            }
            TreeOperator::Parallel => {
                let source = crate::models::petri_net::Place::new("source").with_initial_marking(1);
                let source_id = source.id.clone();
                let sink = crate::models::petri_net::Place::new("sink");
                let sink_id = sink.id.clone();
                net.add_place(source);
                net.add_place(sink);
                net.initial_place = Some(source_id.clone());
                net.final_place = Some(sink_id.clone());
                for (i, _child) in children.iter().enumerate() {
                    let transition =
                        crate::models::petri_net::Transition::new(format!("parallel_{}", i));
                    let transition_id = transition.id.clone();
                    net.add_transition(transition);
                    net.add_arc(crate::models::petri_net::Arc::new(
                        source_id.clone(),
                        transition_id.clone(),
                    ));
                    net.add_arc(crate::models::petri_net::Arc::new(
                        transition_id,
                        sink_id.clone(),
                    ));
                }
            }
            TreeOperator::Choice => {
                let source = crate::models::petri_net::Place::new("source").with_initial_marking(1);
                let source_id = source.id.clone();
                let sink = crate::models::petri_net::Place::new("sink");
                let sink_id = sink.id.clone();
                net.add_place(source);
                net.add_place(sink);
                net.initial_place = Some(source_id.clone());
                net.final_place = Some(sink_id.clone());
                for (i, _child) in children.iter().enumerate() {
                    let transition =
                        crate::models::petri_net::Transition::new(format!("choice_{}", i));
                    let transition_id = transition.id.clone();
                    net.add_transition(transition);
                    net.add_arc(crate::models::petri_net::Arc::new(
                        source_id.clone(),
                        transition_id.clone(),
                    ));
                    net.add_arc(crate::models::petri_net::Arc::new(
                        transition_id,
                        sink_id.clone(),
                    ));
                }
            }
            TreeOperator::Loop => {
                let source = crate::models::petri_net::Place::new("source").with_initial_marking(1);
                let source_id = source.id.clone();
                let sink = crate::models::petri_net::Place::new("sink");
                let sink_id = sink.id.clone();
                net.add_place(source);
                net.add_place(sink);
                net.initial_place = Some(source_id.clone());
                net.final_place = Some(sink_id.clone());
                let do_transition = crate::models::petri_net::Transition::new("do");
                let redo_transition = crate::models::petri_net::Transition::new("redo");
                let do_id = do_transition.id.clone();
                let redo_id = redo_transition.id.clone();
                net.add_transition(do_transition);
                net.add_transition(redo_transition);
                net.add_arc(crate::models::petri_net::Arc::new(
                    source_id.clone(),
                    do_id.clone(),
                ));
                net.add_arc(crate::models::petri_net::Arc::new(do_id, sink_id.clone()));
                net.add_arc(crate::models::petri_net::Arc::new(
                    sink_id.clone(),
                    redo_id.clone(),
                ));
                net.add_arc(crate::models::petri_net::Arc::new(
                    redo_id,
                    source_id.clone(),
                ));
            }
        },
        ProcessTreeNode::Activity(name) => {
            let source = crate::models::petri_net::Place::new("source").with_initial_marking(1);
            let source_id = source.id.clone();
            let sink = crate::models::petri_net::Place::new("sink");
            let sink_id = sink.id.clone();
            net.add_place(source);
            net.add_place(sink);
            net.initial_place = Some(source_id.clone());
            net.final_place = Some(sink_id.clone());
            let transition =
                crate::models::petri_net::Transition::new(name.clone()).with_label(name.clone());
            let transition_id = transition.id.clone();
            net.add_transition(transition);
            net.add_arc(crate::models::petri_net::Arc::new(
                source_id,
                transition_id.clone(),
            ));
            net.add_arc(crate::models::petri_net::Arc::new(transition_id, sink_id));
        }
    }

    net
}

/// Convert Petri net to BPMN
pub fn petri_net_to_bpmn(net: &PetriNet) -> BPMNDiagram {
    let mut diagram = BPMNDiagram::new("Converted from Petri Net");
    for transition in &net.transitions {
        if let Some(ref label) = transition.label {
            let task = bpmn::Task::new(label.as_str(), bpmn::TaskType::UserTask);
            diagram.add_task(task);
        }
    }
    for place in &net.places {
        if place.initial_marking > 0 {
            let event = bpmn::Event::new("Start", bpmn::EventType::Start);
            let id = diagram.add_event(event);
            diagram.start_event_id = Some(id);
        } else if place.final_marking.is_some() {
            let event = bpmn::Event::new("End", bpmn::EventType::End);
            let id = diagram.add_event(event);
            diagram.end_event_ids.insert(id);
        }
    }
    for arc in &net.arcs {
        let flow = bpmn::SequenceFlow::new(arc.from.clone(), arc.to.clone());
        diagram.add_flow(flow);
    }
    diagram
}

/// Convert BPMN to Petri net
pub fn bpmn_to_petri_net(diagram: &BPMNDiagram) -> PetriNet {
    let mut net = PetriNet::new();
    for task in diagram.tasks.values() {
        let transition =
            crate::models::petri_net::Transition::new(&task.name).with_label(&task.name);
        net.add_transition(transition);
    }
    for gateway in diagram.gateways.values() {
        let place = crate::models::petri_net::Place::new(&gateway.name);
        net.add_place(place);
    }
    for event in diagram.events.values() {
        let place = crate::models::petri_net::Place::new(&event.name);
        if event.event_type == bpmn::EventType::Start {
            let initial_place = place.with_initial_marking(1);
            net.initial_place = Some(initial_place.id.clone());
            net.add_place(initial_place);
        } else {
            net.add_place(place);
        }
    }
    for flow in diagram.flows.values() {
        net.add_arc(crate::models::petri_net::Arc::new(
            flow.source_id.clone(),
            flow.target_id.clone(),
        ));
    }
    net
}

/// Convert DFG to Petri net
pub fn dfg_to_petri_net(
    dfg: &HashMap<(String, String), usize>,
    start_activities: &HashMap<String, usize>,
    end_activities: &HashMap<String, usize>,
) -> PetriNet {
    let mut net = PetriNet::new();
    let source = crate::models::petri_net::Place::new("source").with_initial_marking(1);
    let source_id = source.id.clone();
    net.add_place(source);
    net.initial_place = Some(source_id.clone());
    let sink = crate::models::petri_net::Place::new("sink");
    let sink_id = sink.id.clone();
    net.add_place(sink);
    net.final_place = Some(sink_id.clone());
    let mut activity_to_transition: HashMap<String, String> = HashMap::new();
    let mut all_activities: HashSet<String> = HashSet::new();
    for (from, to) in dfg.keys() {
        all_activities.insert(from.clone());
        all_activities.insert(to.clone());
    }
    for activity in &all_activities {
        let transition = crate::models::petri_net::Transition::new(activity.clone())
            .with_label(activity.clone());
        let transition_id = transition.id.clone();
        net.add_transition(transition);
        activity_to_transition.insert(activity.clone(), transition_id);
    }
    for activity in start_activities.keys() {
        if let Some(transition_id) = activity_to_transition.get(activity) {
            net.add_arc(crate::models::petri_net::Arc::new(
                source_id.clone(),
                transition_id.clone(),
            ));
        }
    }
    for activity in end_activities.keys() {
        if let Some(transition_id) = activity_to_transition.get(activity) {
            net.add_arc(crate::models::petri_net::Arc::new(
                transition_id.clone(),
                sink_id.clone(),
            ));
        }
    }
    for (from, to) in dfg.keys() {
        if let (Some(from_trans), Some(to_trans)) = (
            activity_to_transition.get(from),
            activity_to_transition.get(to),
        ) {
            let place = crate::models::petri_net::Place::new(format!("p_{}_{}", from, to));
            let place_id = place.id.clone();
            net.add_place(place);
            net.add_arc(crate::models::petri_net::Arc::new(
                from_trans.clone(),
                place_id.clone(),
            ));
            net.add_arc(crate::models::petri_net::Arc::new(
                place_id,
                to_trans.clone(),
            ));
        }
    }
    net
}

/// Convert process tree to BPMN
pub fn process_tree_to_bpmn(tree: &ProcessTree) -> BPMNDiagram {
    let net = process_tree_to_petri_net(tree);
    petri_net_to_bpmn(&net)
}

/// Convert BPMN to process tree
pub fn bpmn_to_process_tree(diagram: &BPMNDiagram) -> Option<ProcessTree> {
    let net = bpmn_to_petri_net(diagram);
    petri_net_to_process_tree(&net)
}

/// Simplify process tree by removing redundant nodes
pub fn simplify_process_tree(tree: &ProcessTree) -> ProcessTree {
    let mut simplified = tree.clone();
    simplify_recursive(&mut simplified.root);
    simplified
}

fn simplify_recursive(node: &mut ProcessTreeNode) {
    match node {
        ProcessTreeNode::Operator {
            operator: _,
            children,
            ..
        } => {
            if children.len() == 1 {
                *node = children[0].clone();
            } else {
                for child in children {
                    simplify_recursive(child);
                }
            }
        }
        ProcessTreeNode::Activity(_) => {}
    }
}
