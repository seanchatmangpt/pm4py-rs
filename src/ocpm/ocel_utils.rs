//! OCEL (Object-Centric Event Log) Utility Functions
//!
//! Additional OCEL operations beyond basic discovery and conformance.

use crate::ocpm::*;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Flatten OCEL to traditional event log
pub fn ocel_flattening(ocel: &ObjectCentricEventLog) -> crate::log::EventLog {
    let mut flat_log = crate::log::EventLog::new();

    // Create a mapping from event IDs to object IDs
    let mut event_objects: HashMap<Uuid, Vec<String>> = HashMap::new();
    for mapping in &ocel.event_object_mappings {
        let obj_ids: Vec<String> = mapping.object_ids.iter().cloned().collect();
        event_objects.insert(mapping.event_id, obj_ids);
    }

    for (event_id, (activity, timestamp, _resource)) in &ocel.events {
        // Get objects involved in this event
        if let Some(obj_ids) = event_objects.get(event_id) {
            // Create a traditional event for each object involved
            for object_id in obj_ids {
                // Create trace name from object
                let trace_name = object_id.clone();

                // Find or create trace
                let trace =
                    if let Some(pos) = flat_log.traces.iter().position(|t| t.id == trace_name) {
                        &mut flat_log.traces[pos]
                    } else {
                        let mut new_trace = crate::log::Trace::new(&trace_name);
                        new_trace.id = trace_name.clone();
                        flat_log.traces.push(new_trace);
                        flat_log.traces.last_mut().unwrap()
                    };

                // Add event to trace
                let mut event = crate::log::Event::new(activity.clone(), *timestamp);
                event.id = *event_id;

                // Add OCEL-specific info
                event
                    .attributes
                    .insert("ocel:object_id".to_string(), object_id.clone());
                if let Some(obj) = ocel.get_object(object_id) {
                    event
                        .attributes
                        .insert("ocel:object_type".to_string(), obj.object_type.name.clone());
                }

                trace.events.push(event);
            }
        }
    }

    flat_log
}

/// Get OCEL object types summary
pub fn ocel_objects_summary(ocel: &ObjectCentricEventLog) -> HashMap<String, usize> {
    let mut summary = HashMap::new();

    for object in ocel.objects.values() {
        *summary.entry(object.object_type.name.clone()).or_insert(0) += 1;
    }

    summary
}

/// Get OCEL objects interaction summary
pub fn ocel_objects_interactions_summary(
    ocel: &ObjectCentricEventLog,
) -> HashMap<(String, String), usize> {
    let mut interactions: HashMap<(String, String), usize> = HashMap::new();

    // Build a mapping from event_id to object types
    let mut event_object_types: HashMap<Uuid, Vec<String>> = HashMap::new();
    for mapping in &ocel.event_object_mappings {
        let mut types = Vec::new();
        for obj_id in &mapping.object_ids {
            if let Some(obj) = ocel.get_object(obj_id) {
                types.push(obj.object_type.name.clone());
            }
        }
        event_object_types.insert(mapping.event_id, types);
    }

    // Count interactions
    for object_types in event_object_types.values() {
        for i in 0..object_types.len() {
            for j in (i + 1)..object_types.len() {
                let pair = (object_types[i].clone(), object_types[j].clone());
                *interactions.entry(pair).or_insert(0) += 1;
            }
        }
    }

    interactions
}

/// Get OCEL temporal summary
pub fn ocel_temporal_summary(ocel: &ObjectCentricEventLog) -> OcelTemporalSummary {
    let mut start_time = None;
    let mut end_time = None;

    for (_activity, timestamp, _resource) in ocel.events.values() {
        if start_time.is_none() || *timestamp < start_time.unwrap() {
            start_time = Some(*timestamp);
        }
        if end_time.is_none() || *timestamp > end_time.unwrap() {
            end_time = Some(*timestamp);
        }
    }

    OcelTemporalSummary {
        start_time,
        end_time,
        num_events: ocel.events.len(),
        num_objects: ocel.objects.len(),
        num_object_types: ocel.object_types.len(),
    }
}

/// OCEL temporal summary
#[derive(Debug, Clone)]
pub struct OcelTemporalSummary {
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub num_events: usize,
    pub num_objects: usize,
    pub num_object_types: usize,
}

/// Get OCEL attribute names
pub fn ocel_get_attribute_names(ocel: &ObjectCentricEventLog) -> Vec<String> {
    let mut attributes = HashSet::new();

    // Log-level attributes
    for key in ocel.attributes.keys() {
        attributes.insert(key.clone());
    }

    // Object attributes
    for object in ocel.objects.values() {
        for key in object.attributes.keys() {
            attributes.insert(format!("object:{}", key));
        }
    }

    let mut attrs: Vec<String> = attributes.into_iter().collect();
    attrs.sort();
    attrs
}

/// Get OCEL object types
pub fn ocel_get_object_types(ocel: &ObjectCentricEventLog) -> Vec<String> {
    let mut types = Vec::new();

    for object_type in &ocel.object_types {
        types.push(object_type.name.clone());
    }

    types.sort();
    types
}

/// Get activities per object type
pub fn ocel_object_type_activities(
    ocel: &ObjectCentricEventLog,
) -> HashMap<String, HashSet<String>> {
    let mut activities: HashMap<String, HashSet<String>> = HashMap::new();

    // Build mapping from event_id to object types
    let mut event_object_types: HashMap<Uuid, Vec<String>> = HashMap::new();
    for mapping in &ocel.event_object_mappings {
        let mut types = Vec::new();
        for obj_id in &mapping.object_ids {
            if let Some(obj) = ocel.get_object(obj_id) {
                types.push(obj.object_type.name.clone());
            }
        }
        event_object_types.insert(mapping.event_id, types);
    }

    // Associate activities with object types
    for (event_id, (activity, _timestamp, _resource)) in &ocel.events {
        if let Some(obj_types) = event_object_types.get(event_id) {
            for obj_type in obj_types {
                activities
                    .entry(obj_type.clone())
                    .or_default()
                    .insert(activity.clone());
            }
        }
    }

    activities
}

/// Get object count per object type
pub fn ocel_objects_ot_count(ocel: &ObjectCentricEventLog) -> HashMap<String, usize> {
    let mut counts: HashMap<String, usize> = HashMap::new();

    for object in ocel.objects.values() {
        *counts.entry(object.object_type.name.clone()).or_insert(0) += 1;
    }

    counts
}

/// Sort OCEL events by additional column
pub fn ocel_sort_by_additional_column(
    ocel: &ObjectCentricEventLog,
    attribute_name: &str,
) -> Vec<String> {
    let mut event_ids: Vec<(String, String)> = Vec::new();

    for event_id in ocel.events.keys() {
        let sort_value = ocel
            .attributes
            .get(attribute_name)
            .cloned()
            .unwrap_or_else(|| "".to_string());

        event_ids.push((event_id.to_string(), sort_value));
    }

    event_ids.sort_by(|a, b| a.1.cmp(&b.1));

    event_ids.into_iter().map(|(id, _)| id).collect()
}

/// Add index-based timedelta to OCEL events
pub fn ocel_add_index_based_timedelta(
    ocel: &mut ObjectCentricEventLog,
    event_id: &str,
    timedelta_seconds: i64,
) {
    if let Ok(uuid) = Uuid::parse_str(event_id) {
        if let Some((activity, timestamp, resource)) = ocel.events.get(&uuid).cloned() {
            let new_time = timestamp + chrono::Duration::seconds(timedelta_seconds);
            ocel.events.insert(uuid, (activity, new_time, resource));
        }
    }
}

/// Merge duplicate OCEL events.
///
/// Groups events by `(activity, timestamp, sorted_object_ids)`.  For each
/// group, the first event (by UUID sort order) is kept and all others are
/// removed from `ocel.events` and their mappings are dropped.
pub fn ocel_merge_duplicates(ocel: &mut ObjectCentricEventLog) {
    // Build a map from event_id → sorted object_ids for fast lookup.
    let mut event_objects: HashMap<Uuid, Vec<String>> = HashMap::new();
    for mapping in &ocel.event_object_mappings {
        let mut obj_ids: Vec<String> = mapping.object_ids.iter().cloned().collect();
        obj_ids.sort();
        event_objects.insert(mapping.event_id, obj_ids);
    }

    // Group event_ids by their deduplication key.
    // Key = (activity, timestamp_rfc3339, sorted_object_ids)
    // We use a deterministic sort of the event_ids within each group so the
    // "first" kept entry is stable across runs.
    let mut groups: HashMap<(String, String, Vec<String>), Vec<Uuid>> = HashMap::new();
    for (event_id, (activity, timestamp, _)) in &ocel.events {
        let obj_ids = event_objects.get(event_id).cloned().unwrap_or_default();
        let key = (activity.clone(), timestamp.to_rfc3339(), obj_ids);
        groups.entry(key).or_default().push(*event_id);
    }

    // Collect the UUIDs to remove (all but the first in each group).
    let mut to_remove: Vec<Uuid> = Vec::new();
    for mut ids in groups.into_values() {
        if ids.len() > 1 {
            // Sort so the result is deterministic, keep first.
            ids.sort();
            to_remove.extend_from_slice(&ids[1..]);
        }
    }

    // Remove duplicate events and their mappings.
    let remove_set: HashSet<Uuid> = to_remove.iter().copied().collect();
    for id in &remove_set {
        ocel.events.remove(id);
    }
    ocel.event_object_mappings
        .retain(|m| !remove_set.contains(&m.event_id));
}

/// Drop duplicate OCEL events
pub fn ocel_drop_duplicates(ocel: &mut ObjectCentricEventLog) -> usize {
    let mut seen = HashSet::new();
    let mut to_remove = Vec::new();

    // Build mapping from event_id to object IDs
    let mut event_objects: HashMap<Uuid, Vec<String>> = HashMap::new();
    for mapping in &ocel.event_object_mappings {
        let obj_ids: Vec<String> = mapping.object_ids.iter().cloned().collect();
        event_objects.insert(mapping.event_id, obj_ids);
    }

    for (event_id, (activity, timestamp, _resource)) in &ocel.events {
        let obj_ids = event_objects.get(event_id).cloned().unwrap_or_default();
        let key = (activity.clone(), *timestamp, obj_ids.clone());

        if seen.contains(&key) {
            to_remove.push(*event_id);
        } else {
            seen.insert(key);
        }
    }

    let count = to_remove.len();
    for id in to_remove {
        ocel.events.remove(&id);
    }

    count
}

/// Sample OCEL connected components
pub fn sample_ocel_connected_components(
    ocel: &ObjectCentricEventLog,
    min_size: usize,
) -> Vec<Vec<String>> {
    let mut components: Vec<Vec<String>> = Vec::new();
    let mut visited = HashSet::new();

    // Build object interaction graph
    let mut graph: HashMap<String, Vec<String>> = HashMap::new();

    for mapping in &ocel.event_object_mappings {
        for obj_id in &mapping.object_ids {
            graph.entry(obj_id.clone()).or_default();
            for other_obj in &mapping.object_ids {
                if obj_id != other_obj {
                    graph.get_mut(obj_id).unwrap().push(other_obj.clone());
                }
            }
        }
    }

    // Find connected components
    for obj_id in graph.keys() {
        if visited.contains(obj_id) {
            continue;
        }

        let mut component = Vec::new();
        let mut stack = vec![obj_id.clone()];

        while let Some(current) = stack.pop() {
            if visited.contains(&current) {
                continue;
            }

            visited.insert(current.clone());
            component.push(current.clone());

            if let Some(neighbors) = graph.get(&current) {
                for neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        stack.push(neighbor.clone());
                    }
                }
            }
        }

        if component.len() >= min_size {
            components.push(component);
        }
    }

    components
}

/// Sample OCEL objects
pub fn sample_ocel_objects(ocel: &ObjectCentricEventLog, num_objects: usize) -> Vec<String> {
    let object_ids: Vec<String> = ocel.objects.keys().cloned().collect();

    if object_ids.len() <= num_objects {
        object_ids
    } else {
        // Simple random sampling - in production, use proper RNG
        object_ids.into_iter().take(num_objects).collect()
    }
}
