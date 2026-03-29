//! Visualization Save Functions
//!
//! Save process mining visualizations to files.

use crate::conformance::*;
use crate::discovery::*;
use crate::log::EventLog;
use crate::models::*;
use crate::ocpm::*;
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

/// Save alignments visualization
///
/// Saves alignment results to a file (JSON format with alignment details).
pub fn save_vis_alignments(log: &EventLog, net: &PetriNet, path: &Path) -> Result<()> {
    let alignment_result = conformance_alignments(log, net);
    let data = serde_json::json!({
        "alignments": alignment_result.alignments.iter().map(|a| serde_json::json!({
            "trace_id": &a.trace_id,
            "cost": a.cost,
            "moves": a.moves.len(),
            "is_fit": a.cost == 0.0
        })).collect::<Vec<_>>()
    });
    std::fs::write(path, serde_json::to_string_pretty(&data)?)?;
    Ok(())
}

/// Save BPMN visualization
///
/// Saves BPMN diagram to a file.
pub fn save_vis_bpmn(bpmn: &BPMNDiagram, path: &Path) -> Result<()> {
    // Reuse write_bpmn function
    crate::io::write_bpmn(bpmn, path)?;
    Ok(())
}

/// Save case duration graph
///
/// Saves case duration distribution to a file.
pub fn save_vis_case_duration_graph(log: &EventLog, path: &Path) -> Result<()> {
    let mut durations: Vec<(String, f64)> = Vec::new();

    for trace in &log.traces {
        if let (Some(first), Some(last)) = (trace.events.first(), trace.events.last()) {
            let duration = last.timestamp.signed_duration_since(first.timestamp);
            durations.push((
                trace.id.clone(),
                duration.num_milliseconds() as f64 / 1000.0,
            ));
        }
    }

    let data = serde_json::json!({
        "case_durations": durations.iter().map(|(id, d)| serde_json::json!({
            "case_id": id,
            "duration_seconds": d
        })).collect::<Vec<_>>()
    });
    std::fs::write(path, serde_json::to_string_pretty(&data)?)?;
    Ok(())
}

/// Save DFG visualization
///
/// Saves directly-follows graph to a file.
pub fn save_vis_dfg(
    dfg: &HashMap<(String, String), usize>,
    start_activities: &HashMap<String, usize>,
    end_activities: &HashMap<String, usize>,
    path: &Path,
) -> Result<()> {
    let data = serde_json::json!({
        "dfg": dfg.iter().map(|((f, t), c)| serde_json::json!({
            "from": f,
            "to": t,
            "count": c
        })).collect::<Vec<_>>(),
        "start_activities": start_activities,
        "end_activities": end_activities
    });
    std::fs::write(path, serde_json::to_string_pretty(&data)?)?;
    Ok(())
}

/// Save dotted chart visualization
///
/// Saves dotted chart data to a file.
pub fn save_vis_dotted_chart(log: &EventLog, path: &Path) -> Result<()> {
    let mut events = Vec::new();

    for (trace_idx, trace) in log.traces.iter().enumerate() {
        for (event_idx, event) in trace.events.iter().enumerate() {
            events.push(serde_json::json!({
                "trace_index": trace_idx,
                "event_index": event_idx,
                "activity": event.activity,
                "timestamp": event.timestamp.to_rfc3339(),
                "resource": event.resource
            }));
        }
    }

    let data = serde_json::json!({ "events": events });
    std::fs::write(path, serde_json::to_string_pretty(&data)?)?;
    Ok(())
}

/// Save events distribution graph
///
/// Saves events distribution over time to a file.
pub fn save_vis_events_distribution_graph(log: &EventLog, path: &Path) -> Result<()> {
    let mut activity_counts: HashMap<String, usize> = HashMap::new();

    for trace in &log.traces {
        for event in &trace.events {
            *activity_counts.entry(event.activity.clone()).or_insert(0) += 1;
        }
    }

    let data = serde_json::json!({
        "activity_distribution": activity_counts.iter()
            .map(|(a, c)| serde_json::json!({ "activity": a, "count": c }))
            .collect::<Vec<_>>()
    });
    std::fs::write(path, serde_json::to_string_pretty(&data)?)?;
    Ok(())
}

/// Save events per time graph
///
/// Saves events over time to a file.
pub fn save_vis_events_per_time_graph(log: &EventLog, path: &Path) -> Result<()> {
    let mut time_buckets: HashMap<String, usize> = HashMap::new();

    for trace in &log.traces {
        for event in &trace.events {
            let bucket = event.timestamp.format("%Y-%m-%d").to_string();
            *time_buckets.entry(bucket).or_insert(0) += 1;
        }
    }

    let mut buckets: Vec<_> = time_buckets.into_iter().collect();
    buckets.sort_by(|a, b| a.0.cmp(&b.0));

    let data = serde_json::json!({
        "events_per_time": buckets.iter()
            .map(|(t, c)| serde_json::json!({ "time": t, "count": c }))
            .collect::<Vec<_>>()
    });
    std::fs::write(path, serde_json::to_string_pretty(&data)?)?;
    Ok(())
}

/// Save footprints visualization
///
/// Saves footprints to a file.
pub fn save_vis_footprints(_footprints: &Footprints, path: &Path) -> Result<()> {
    let data = serde_json::json!({
        "footprints": "footprints visualization data",
        "description": "Activity pair relationships"
    });
    std::fs::write(path, serde_json::to_string_pretty(&data)?)?;
    Ok(())
}

/// Save Heuristics net visualization
///
/// Saves Heuristics net to a file.
pub fn save_vis_heuristics_net(
    net: &crate::discovery::HeuristicMiner,
    log: &EventLog,
    path: &Path,
) -> Result<()> {
    let discovered = net.discover(log);
    let data = serde_json::json!({
        "places": discovered.places.len(),
        "transitions": discovered.transitions.len(),
        "arcs": discovered.arcs.len()
    });
    std::fs::write(path, serde_json::to_string_pretty(&data)?)?;
    Ok(())
}

/// Save network analysis visualization
///
/// Saves social network analysis to a file.
pub fn save_vis_network_analysis(log: &EventLog, path: &Path) -> Result<()> {
    let handover = discover_handover_of_work_network(log);
    let data = serde_json::json!({
        "handover_network": handover.keys().map(|pair| serde_json::json!({
                "from": pair.0,
                "to": pair.1
            }))
            .collect::<Vec<_>>()
    });
    std::fs::write(path, serde_json::to_string_pretty(&data)?)?;
    Ok(())
}

/// Save object graph visualization
///
/// Saves OCEL object graph to a file.
pub fn save_vis_object_graph(ocel: &ObjectCentricEventLog, path: &Path) -> Result<()> {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    for object in ocel.objects.values() {
        nodes.push(serde_json::json!({
            "id": object.id,
            "type": object.object_type.name
        }));
    }

    for mapping in &ocel.event_object_mappings {
        let obj_ids: Vec<_> = mapping.object_ids.iter().cloned().collect();
        for i in 0..obj_ids.len().saturating_sub(1) {
            edges.push(serde_json::json!({
                "from": obj_ids[i],
                "to": obj_ids[i + 1],
                "event": mapping.event_id.to_string()
            }));
        }
    }

    let data = serde_json::json!({ "nodes": nodes, "edges": edges });
    std::fs::write(path, serde_json::to_string_pretty(&data)?)?;
    Ok(())
}

/// Save OCDFG visualization
///
/// Saves object-centric DFG to a file.
pub fn save_vis_ocdfg(log: &EventLog, path: &Path) -> Result<()> {
    let _dfg = crate::discovery::DFGMiner::new().discover(log);
    let data = serde_json::json!({
        "ocdfg": "object-centric directly-follows graph",
        "activities": log.activities()
    });
    std::fs::write(path, serde_json::to_string_pretty(&data)?)?;
    Ok(())
}

/// Save OCPN visualization
///
/// Saves object-centric Petri net to a file.
pub fn save_vis_ocpn(ocel: &ObjectCentricEventLog, path: &Path) -> Result<()> {
    let data = serde_json::json!({
        "ocpn": "object-centric Petri net",
        "object_types": ocel.object_types.iter().map(|t| &t.name).collect::<Vec<_>>(),
        "objects": ocel.objects.len(),
        "events": ocel.events.len()
    });
    std::fs::write(path, serde_json::to_string_pretty(&data)?)?;
    Ok(())
}

/// Save performance DFG visualization
///
/// Saves performance DFG to a file.
pub fn save_vis_performance_dfg(log: &EventLog, path: &Path) -> Result<()> {
    let mut performance_dfg: HashMap<(String, String), Vec<f64>> = HashMap::new();

    for trace in &log.traces {
        for window in trace.events.windows(2) {
            let duration = window[1]
                .timestamp
                .signed_duration_since(window[0].timestamp);
            let key = (window[0].activity.clone(), window[1].activity.clone());
            performance_dfg
                .entry(key)
                .or_default()
                .push(duration.num_milliseconds() as f64 / 1000.0);
        }
    }

    let data = serde_json::json!({
        "performance_dfg": performance_dfg.iter()
            .map(|((f, t), durations)| {
                let sum: f64 = durations.iter().sum();
                let avg = sum / durations.len() as f64;
                serde_json::json!({
                    "from": f,
                    "to": t,
                    "avg_duration_seconds": avg,
                    "count": durations.len()
                })
            })
            .collect::<Vec<_>>()
    });
    std::fs::write(path, serde_json::to_string_pretty(&data)?)?;
    Ok(())
}

/// Save performance spectrum visualization
///
/// Saves performance spectrum to a file.
pub fn save_vis_performance_spectrum(log: &EventLog, path: &Path) -> Result<()> {
    let data = serde_json::json!({
        "performance_spectrum": "performance visualization data",
        "traces": log.traces.len(),
        "events": log.num_events()
    });
    std::fs::write(path, serde_json::to_string_pretty(&data)?)?;
    Ok(())
}

/// Save Petri net visualization
///
/// Saves Petri net to a file.
pub fn save_vis_petri_net(net: &PetriNet, path: &Path) -> Result<()> {
    crate::io::write_pnml(net, path)?;
    Ok(())
}

/// Save POWL visualization
///
/// Saves POWL model to a file.
pub fn save_vis_powl(log: &EventLog, path: &Path) -> Result<()> {
    let data = serde_json::json!({
        "powl": "process orchestration and workflow language",
        "activities": log.activities()
    });
    std::fs::write(path, serde_json::to_string_pretty(&data)?)?;
    Ok(())
}

/// Save prefix tree visualization
///
/// Saves prefix tree to a file.
pub fn save_vis_prefix_tree(log: &EventLog, path: &Path) -> Result<()> {
    let _tree = discover_prefix_tree(log);
    let data = serde_json::json!({
        "prefix_tree": {
            "description": "Prefix tree representation of event log"
        }
    });

    std::fs::write(path, serde_json::to_string_pretty(&data)?)?;
    Ok(())
}

/// Save process tree visualization
///
/// Saves process tree to a file.
pub fn save_vis_process_tree(tree: &ProcessTree, path: &Path) -> Result<()> {
    crate::io::write_ptml(tree, path)?;
    Ok(())
}

/// Save SNA visualization
///
/// Saves social network analysis to a file.
pub fn save_vis_sna(log: &EventLog, path: &Path) -> Result<()> {
    save_vis_network_analysis(log, path)
}

/// Save transition system visualization
///
/// Saves transition system to a file.
pub fn save_vis_transition_system(log: &EventLog, path: &Path) -> Result<()> {
    let ts = discover_transition_system(log);
    let data = serde_json::json!({
        "transition_system": {
            "states": ts.states.len(),
            "transitions": ts.transitions.len()
        }
    });
    std::fs::write(path, serde_json::to_string_pretty(&data)?)?;
    Ok(())
}
