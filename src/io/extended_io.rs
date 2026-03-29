//! Extended I/O Functions
//!
//! Additional file I/O functions for process mining artifacts.

use crate::log::EventLog;
use crate::models::{BPMNDiagram, PetriNet, ProcessTree};
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

/// Read DFG from file
///
/// Reads a directly-follows graph from a JSON or CSV file.
pub fn read_dfg(path: &Path) -> Result<HashMap<(String, String), usize>> {
    let content = std::fs::read_to_string(path)?;

    if path.extension().and_then(|e| e.to_str()) == Some("json") {
        // JSON format: { "edges": [["A", "B", 5], ["B", "C", 3]] }
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
            let mut dfg = HashMap::new();
            if let Some(edges) = data.get("edges").and_then(|e| e.as_array()) {
                for edge in edges {
                    if let Some(arr) = edge.as_array() {
                        if arr.len() == 3 {
                            if let (Some(from), Some(to), Some(count)) =
                                (arr[0].as_str(), arr[1].as_str(), arr[2].as_u64())
                            {
                                dfg.insert((from.to_string(), to.to_string()), count as usize);
                            }
                        }
                    }
                }
            }
            return Ok(dfg);
        }
    }

    // CSV format: from,to,count
    let mut dfg = HashMap::new();
    for line in content.lines().skip(1) {
        // Skip header
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() == 3 {
            if let Ok(count) = parts[2].trim().parse::<usize>() {
                dfg.insert(
                    (parts[0].trim().to_string(), parts[1].trim().to_string()),
                    count,
                );
            }
        }
    }

    Ok(dfg)
}

/// Write DFG to file
///
/// Writes a directly-follows graph to a JSON or CSV file.
pub fn write_dfg(dfg: &HashMap<(String, String), usize>, path: &Path) -> Result<()> {
    if path.extension().and_then(|e| e.to_str()) == Some("json") {
        let mut edges = Vec::new();
        for ((from, to), count) in dfg {
            edges.push(vec![from.clone(), to.clone(), count.to_string()]);
        }

        let data = serde_json::json!({ "edges": edges });
        std::fs::write(path, serde_json::to_string_pretty(&data)?)?;
    } else {
        // CSV format
        let mut content = String::from("from,to,count\n");
        for ((from, to), count) in dfg {
            content.push_str(&format!("{},{},{}\n", from, to, count));
        }
        std::fs::write(path, content)?;
    }

    Ok(())
}

/// Read Petri net from PNML file
///
/// Reads a Petri net in PNML (Petri Net Markup Language) format.
pub fn read_pnml(path: &Path) -> Result<PetriNet> {
    let content = std::fs::read_to_string(path)?;
    let document = roxmltree::Document::parse(&content)?;

    let mut net = PetriNet::new();

    // Parse places
    for node in document.descendants() {
        if node.tag_name().name() == "place" {
            let id = node
                .attribute("id")
                .unwrap_or(&uuid::Uuid::new_v4().to_string())
                .to_string();
            let initial_marking = node
                .descendants()
                .find(|n| n.tag_name().name() == "initialMarking")
                .and_then(|m| m.text())
                .and_then(|t| t.parse::<usize>().ok())
                .unwrap_or(0);

            let place = if initial_marking > 0 {
                crate::models::petri_net::Place::new(&id).with_initial_marking(initial_marking)
            } else {
                crate::models::petri_net::Place::new(&id)
            };

            net.add_place(place);
        }
    }

    // Parse transitions
    for node in document.descendants() {
        if node.tag_name().name() == "transition" {
            let id = node
                .attribute("id")
                .unwrap_or(&uuid::Uuid::new_v4().to_string())
                .to_string();
            let label = node.attribute("name").map(|s| s.to_string());

            let transition = if let Some(label) = label {
                crate::models::petri_net::Transition::new(&id).with_label(&label)
            } else {
                crate::models::petri_net::Transition::new(&id)
            };

            net.add_transition(transition);
        }
    }

    // Parse arcs
    for node in document.descendants() {
        if node.tag_name().name() == "arc" {
            let source = node.attribute("source").unwrap_or("").to_string();
            let target = node.attribute("target").unwrap_or("").to_string();

            let arc = crate::models::petri_net::Arc::new(source, target);
            net.add_arc(arc);
        }
    }

    Ok(net)
}

/// Write Petri net to PNML file
///
/// Writes a Petri net in PNML format.
pub fn write_pnml(net: &PetriNet, path: &Path) -> Result<()> {
    let mut pnml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    pnml.push_str("<pnml>\n");
    pnml.push_str("  <net id=\"net1\">\n");

    // Write places
    for place in &net.places {
        pnml.push_str(&format!("    <place id=\"{}\">\n", place.id));
        if place.initial_marking > 0 {
            pnml.push_str(&format!(
                "      <initialMarking>{}</initialMarking>\n",
                place.initial_marking
            ));
        }
        pnml.push_str("    </place>\n");
    }

    // Write transitions
    for transition in &net.transitions {
        pnml.push_str(&format!("    <transition id=\"{}\"", transition.id));
        if let Some(ref label) = transition.label {
            pnml.push_str(&format!(" name=\"{}\"", label));
        }
        pnml.push_str("/>\n");
    }

    // Write arcs
    for arc in &net.arcs {
        pnml.push_str(&format!(
            "    <arc source=\"{}\" target=\"{}\"/>\n",
            arc.from, arc.to
        ));
    }

    pnml.push_str("  </net>\n");
    pnml.push_str("</pnml>\n");

    std::fs::write(path, pnml)?;
    Ok(())
}

/// Read BPMN from file
///
/// Reads a BPMN diagram from XML format.
pub fn read_bpmn(path: &Path) -> Result<BPMNDiagram> {
    let content = std::fs::read_to_string(path)?;
    let document = roxmltree::Document::parse(&content)?;

    let mut diagram = BPMNDiagram::new("Imported from BPMN");

    // Parse tasks
    for node in document.descendants() {
        if node.tag_name().name() == "task" {
            let id = node
                .attribute("id")
                .unwrap_or(&uuid::Uuid::new_v4().to_string())
                .to_string();
            let name = node.attribute("name").unwrap_or("Task").to_string();

            let task =
                crate::models::bpmn::Task::new(&name, crate::models::bpmn::TaskType::UserTask);
            diagram.tasks.insert(id, task);
        }
    }

    // Parse events
    for node in document.descendants() {
        if node.tag_name().name() == "startEvent" {
            let id = node
                .attribute("id")
                .unwrap_or(&uuid::Uuid::new_v4().to_string())
                .to_string();
            let event =
                crate::models::bpmn::Event::new("Start", crate::models::bpmn::EventType::Start);
            diagram.events.insert(id, event);
        } else if node.tag_name().name() == "endEvent" {
            let id = node
                .attribute("id")
                .unwrap_or(&uuid::Uuid::new_v4().to_string())
                .to_string();
            let event = crate::models::bpmn::Event::new("End", crate::models::bpmn::EventType::End);
            diagram.events.insert(id, event);
        }
    }

    // Parse sequence flows
    for node in document.descendants() {
        if node.tag_name().name() == "sequenceFlow" {
            let id = node
                .attribute("id")
                .unwrap_or(&uuid::Uuid::new_v4().to_string())
                .to_string();
            let source = node.attribute("sourceRef").unwrap_or("").to_string();
            let target = node.attribute("targetRef").unwrap_or("").to_string();

            let flow = crate::models::bpmn::SequenceFlow::new(source, target);
            diagram.flows.insert(id, flow);
        }
    }

    Ok(diagram)
}

/// Write BPMN to file
///
/// Writes a BPMN diagram to XML format.
pub fn write_bpmn(diagram: &BPMNDiagram, path: &Path) -> Result<()> {
    let mut bpmn = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    bpmn.push_str(
        "<bpmn:definitions xmlns:bpmn=\"http://www.omg.org/spec/BPMN/20100524/MODEL\">\n",
    );

    // Write tasks
    for (id, task) in &diagram.tasks {
        bpmn.push_str(&format!(
            "  <bpmn:task id=\"{}\" name=\"{}\"/>\n",
            id, task.name
        ));
    }

    // Write events
    for (id, event) in &diagram.events {
        match event.event_type {
            crate::models::bpmn::EventType::Start => {
                bpmn.push_str(&format!(
                    "  <bpmn:startEvent id=\"{}\" name=\"{}\"/>\n",
                    id, event.name
                ));
            }
            crate::models::bpmn::EventType::End => {
                bpmn.push_str(&format!(
                    "  <bpmn:endEvent id=\"{}\" name=\"{}\"/>\n",
                    id, event.name
                ));
            }
            _ => {}
        }
    }

    // Write sequence flows
    for (id, flow) in &diagram.flows {
        bpmn.push_str(&format!(
            "  <bpmn:sequenceFlow id=\"{}\" sourceRef=\"{}\" targetRef=\"{}\"/>\n",
            id, flow.source_id, flow.target_id
        ));
    }

    bpmn.push_str("</bpmn:definitions>\n");

    std::fs::write(path, bpmn)?;
    Ok(())
}

/// Read process tree from PTML file
///
/// Reads a process tree in PTML (Process Tree Markup Language) format.
pub fn read_ptml(path: &Path) -> Result<ProcessTree> {
    let content = std::fs::read_to_string(path)?;
    let document = roxmltree::Document::parse(&content)?;

    fn parse_node(node: &roxmltree::Node) -> Option<crate::models::ProcessTreeNode> {
        let operator = node.attribute("operator");
        let name = node.attribute("name");

        if let Some(op) = operator {
            let children: Vec<crate::models::ProcessTreeNode> = node
                .children()
                .filter(|n| n.is_element())
                .filter_map(|n| parse_node(&n))
                .collect();

            let tree_op = match op {
                "sequence" => crate::models::TreeOperator::Sequence,
                "parallel" => crate::models::TreeOperator::Parallel,
                "choice" => crate::models::TreeOperator::Choice,
                "loop" => crate::models::TreeOperator::Loop,
                _ => crate::models::TreeOperator::Sequence,
            };

            Some(crate::models::ProcessTreeNode::Operator {
                operator: tree_op,
                children,
                id: uuid::Uuid::new_v4().to_string(),
            })
        } else if let Some(n) = name {
            Some(crate::models::ProcessTreeNode::Activity(n.to_string()))
        } else {
            Some(crate::models::ProcessTreeNode::Activity("tau".to_string()))
        }
    }

    let root_node = document
        .root()
        .children()
        .find(|n| n.is_element())
        .ok_or_else(|| anyhow::anyhow!("No root element found in PTML"))?;
    let root = parse_node(&root_node)
        .unwrap_or_else(|| crate::models::ProcessTreeNode::Activity("empty".to_string()));

    Ok(ProcessTree::new(root))
}

/// Write process tree to PTML file
///
/// Writes a process tree in PTML format.
pub fn write_ptml(tree: &ProcessTree, path: &Path) -> Result<()> {
    fn write_node(node: &crate::models::ProcessTreeNode, indent: usize) -> String {
        let mut result = String::new();
        let spaces = "  ".repeat(indent);

        match node {
            crate::models::ProcessTreeNode::Operator {
                operator, children, ..
            } => {
                let op_name = match operator {
                    crate::models::TreeOperator::Sequence => "sequence",
                    crate::models::TreeOperator::Parallel => "parallel",
                    crate::models::TreeOperator::Choice => "choice",
                    crate::models::TreeOperator::Loop => "loop",
                };

                result.push_str(&format!("{}<node operator=\"{}\">\n", spaces, op_name));
                for child in children {
                    result.push_str(&write_node(child, indent + 1));
                }
                result.push_str(&format!("{}</node>\n", spaces));
            }
            crate::models::ProcessTreeNode::Activity(name) => {
                result.push_str(&format!("{}<node name=\"{}\"/>\n", spaces, name));
            }
        }

        result
    }

    let mut ptml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    ptml.push_str("<ptml>\n");
    ptml.push_str(&write_node(&tree.root, 1));
    ptml.push_str("</ptml>\n");

    std::fs::write(path, ptml)?;
    Ok(())
}

/// Deserialize event log from bytes
///
/// Deserializes an event log from a serialized byte format.
pub fn deserialize_log(data: &[u8]) -> Result<EventLog> {
    bincode::deserialize(data).map_err(|e| anyhow::anyhow!("Failed to deserialize log: {}", e))
}

/// Serialize event log to bytes
///
/// Serializes an event log to a byte format.
pub fn serialize_log(log: &EventLog) -> Result<Vec<u8>> {
    bincode::serialize(log).map_err(|e| anyhow::anyhow!("Failed to serialize log: {}", e))
}

/// Format event log as dataframe-like structure
///
/// Converts event log to a tabular format similar to pandas DataFrame.
pub fn format_dataframe(log: &EventLog) -> Vec<Vec<String>> {
    let mut rows = Vec::new();

    // Header
    rows.push(vec![
        "case_id".to_string(),
        "activity".to_string(),
        "timestamp".to_string(),
        "resource".to_string(),
    ]);

    // Data rows
    for trace in &log.traces {
        for event in &trace.events {
            rows.push(vec![
                trace.id.clone(),
                event.activity.clone(),
                event.timestamp.to_rfc3339(),
                event.resource.clone().unwrap_or_else(|| "".to_string()),
            ]);
        }
    }

    rows
}

/// Reduce invisible transitions in Petri net
///
/// Removes or consolidates invisible (silent) transitions from a Petri net.
pub fn reduce_petri_net_invisibles(net: &mut PetriNet) {
    // Find transitions without labels (invisible transitions)
    let invisible_transitions: Vec<String> = net
        .transitions
        .iter()
        .filter(|t| t.label.is_none() || t.label.as_ref().map(|l| l.is_empty()).unwrap_or(false))
        .map(|t| t.id.clone())
        .collect();

    // Remove invisible transitions
    for trans_id in invisible_transitions {
        net.transitions.retain(|t| t.id != trans_id);

        // Remove arcs connected to this transition
        net.arcs.retain(|a| a.from != trans_id && a.to != trans_id);

        // Connect places directly if they were connected through invisible transition
        let incoming_places: Vec<String> = net
            .places
            .iter()
            .filter(|p| net.arcs.iter().any(|a| a.from == p.id && a.to == trans_id))
            .map(|p| p.id.clone())
            .collect();

        let outgoing_places: Vec<String> = net
            .places
            .iter()
            .filter(|p| net.arcs.iter().any(|a| a.from == trans_id && a.to == p.id))
            .map(|p| p.id.clone())
            .collect();

        // Connect each incoming place to each outgoing place
        for from in &incoming_places {
            for to in &outgoing_places {
                let arc = crate::models::petri_net::Arc::new(from.clone(), to.clone());
                net.add_arc(arc);
            }
        }
    }
}
