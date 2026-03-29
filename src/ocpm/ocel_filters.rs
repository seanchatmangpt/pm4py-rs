//! OCEL (Object-Centric) Filtering Functions
//!
//! Filter operations for object-centric event logs.

use crate::ocpm::ObjectCentricEventLog;
use crate::ocpm::*;
use std::collections::{HashMap, HashSet};

/// Filter OCEL by object type
///
/// Keep only events that involve objects of the specified types.
pub fn ocel_filter_object_type(
    ocel: &ObjectCentricEventLog,
    object_types: &[String],
) -> ObjectCentricEventLog {
    let type_set: HashSet<String> = object_types.iter().cloned().collect();
    let mut filtered = ObjectCentricEventLog::new();

    // Copy objects of matching types
    for object in ocel.objects.values() {
        if type_set.contains(&object.object_type.name) {
            filtered.add_object(object.clone());
        }
    }

    // Copy object types
    for obj_type in &ocel.object_types {
        if type_set.contains(&obj_type.name) {
            filtered.register_object_type(obj_type.clone());
        }
    }

    // Copy events that involve filtered objects
    for mapping in &ocel.event_object_mappings {
        let involves_filtered_objects = mapping.object_ids.iter().any(|obj_id| {
            if let Some(obj) = ocel.get_object(obj_id) {
                type_set.contains(&obj.object_type.name)
            } else {
                false
            }
        });

        if involves_filtered_objects {
            filtered.add_event_object_mapping(mapping.clone());
        }
    }

    // Copy events referenced by mappings
    for mapping in &filtered.event_object_mappings {
        if let Some((activity, timestamp, resource)) = ocel.events.get(&mapping.event_id) {
            filtered.events.insert(
                mapping.event_id,
                (activity.clone(), *timestamp, resource.clone()),
            );
        }
    }

    // Copy attributes
    filtered.attributes = ocel.attributes.clone();

    filtered
}

/// Filter OCEL by object IDs
///
/// Keep only events that involve the specified objects.
pub fn ocel_filter_object_ids(
    ocel: &ObjectCentricEventLog,
    object_ids: &[String],
) -> ObjectCentricEventLog {
    let id_set: HashSet<String> = object_ids.iter().cloned().collect();
    let mut filtered = ObjectCentricEventLog::new();

    // Copy specified objects
    for object_id in object_ids {
        if let Some(object) = ocel.get_object(object_id) {
            filtered.add_object(object.clone());
            filtered.register_object_type(object.object_type.clone());
        }
    }

    // Copy events that involve specified objects
    for mapping in &ocel.event_object_mappings {
        let involves_specified_objects = mapping
            .object_ids
            .iter()
            .any(|obj_id| id_set.contains(obj_id));

        if involves_specified_objects {
            filtered.add_event_object_mapping(mapping.clone());
        }
    }

    // Copy events referenced by mappings
    for mapping in &filtered.event_object_mappings {
        if let Some((activity, timestamp, resource)) = ocel.events.get(&mapping.event_id) {
            filtered.events.insert(
                mapping.event_id,
                (activity.clone(), *timestamp, resource.clone()),
            );
        }
    }

    // Copy attributes
    filtered.attributes = ocel.attributes.clone();

    filtered
}

/// Filter OCEL by time range
///
/// Keep only events within the specified time range.
pub fn ocel_filter_time_range(
    ocel: &ObjectCentricEventLog,
    start_time: chrono::DateTime<chrono::Utc>,
    end_time: chrono::DateTime<chrono::Utc>,
) -> ObjectCentricEventLog {
    let mut filtered = ObjectCentricEventLog::new();

    // Copy objects (all objects are kept)
    for object in ocel.objects.values() {
        filtered.add_object(object.clone());
    }

    // Copy object types
    for obj_type in &ocel.object_types {
        filtered.register_object_type(obj_type.clone());
    }

    // Copy events within time range
    for (event_id, (activity, timestamp, resource)) in &ocel.events {
        if *timestamp >= start_time && *timestamp <= end_time {
            filtered
                .events
                .insert(*event_id, (activity.clone(), *timestamp, resource.clone()));
        }
    }

    // Copy mappings for events that were kept
    for mapping in &ocel.event_object_mappings {
        if filtered.events.contains_key(&mapping.event_id) {
            filtered.add_event_object_mapping(mapping.clone());
        }
    }

    // Copy attributes
    filtered.attributes = ocel.attributes.clone();

    filtered
}

/// Filter OCEL by activity
///
/// Keep only events with the specified activities.
pub fn ocel_filter_activities(
    ocel: &ObjectCentricEventLog,
    activities: &[String],
) -> ObjectCentricEventLog {
    let activity_set: HashSet<String> = activities.iter().cloned().collect();
    let mut filtered = ObjectCentricEventLog::new();

    // Copy objects (all objects are kept)
    for object in ocel.objects.values() {
        filtered.add_object(object.clone());
    }

    // Copy object types
    for obj_type in &ocel.object_types {
        filtered.register_object_type(obj_type.clone());
    }

    // Copy events with matching activities
    for (event_id, (activity, timestamp, resource)) in &ocel.events {
        if activity_set.contains(activity) {
            filtered
                .events
                .insert(*event_id, (activity.clone(), *timestamp, resource.clone()));
        }
    }

    // Copy mappings for events that were kept
    for mapping in &ocel.event_object_mappings {
        if filtered.events.contains_key(&mapping.event_id) {
            filtered.add_event_object_mapping(mapping.clone());
        }
    }

    // Copy attributes
    filtered.attributes = ocel.attributes.clone();

    filtered
}

/// Filter OCEL by object attribute
///
/// Keep only objects with specified attribute values.
pub fn ocel_filter_object_attribute(
    ocel: &ObjectCentricEventLog,
    attribute_name: &str,
    attribute_value: &str,
) -> ObjectCentricEventLog {
    let mut filtered = ObjectCentricEventLog::new();
    let mut matching_object_ids = HashSet::new();

    // Find objects with matching attribute
    for object in ocel.objects.values() {
        if let Some(value) = object.get_attribute(attribute_name) {
            if value == attribute_value {
                matching_object_ids.insert(object.id.clone());
                filtered.add_object(object.clone());
                filtered.register_object_type(object.object_type.clone());
            }
        }
    }

    // Copy events that involve matching objects
    for mapping in &ocel.event_object_mappings {
        let involves_matching_objects = mapping
            .object_ids
            .iter()
            .any(|obj_id| matching_object_ids.contains(obj_id));

        if involves_matching_objects {
            filtered.add_event_object_mapping(mapping.clone());
        }
    }

    // Copy events referenced by mappings
    for mapping in &filtered.event_object_mappings {
        if let Some((activity, timestamp, resource)) = ocel.events.get(&mapping.event_id) {
            filtered.events.insert(
                mapping.event_id,
                (activity.clone(), *timestamp, resource.clone()),
            );
        }
    }

    // Copy attributes
    filtered.attributes = ocel.attributes.clone();

    filtered
}

/// Filter OCEL to connected components
///
/// Keep only events and objects in connected components of minimum size.
pub fn ocel_filter_connected_components(
    ocel: &ObjectCentricEventLog,
    min_component_size: usize,
) -> ObjectCentricEventLog {
    let components = sample_ocel_connected_components(ocel, min_component_size);

    if components.is_empty() {
        return ObjectCentricEventLog::new();
    }

    // Flatten components into a set of object IDs
    let mut object_id_set = HashSet::new();
    for component in components {
        for obj_id in component {
            object_id_set.insert(obj_id);
        }
    }

    // Use object ID filter
    let object_ids: Vec<String> = object_id_set.into_iter().collect();
    ocel_filter_object_ids(ocel, &object_ids)
}

/// Filter OCEL by number of events per object
///
/// Keep only objects involved in at least/at most N events.
pub fn ocel_filter_object_event_count(
    ocel: &ObjectCentricEventLog,
    min_events: usize,
    max_events: usize,
) -> ObjectCentricEventLog {
    let mut matching_object_ids = Vec::new();

    for object_id in ocel.objects.keys() {
        let event_count = ocel.get_events_for_object(object_id).len();
        if event_count >= min_events && event_count <= max_events {
            matching_object_ids.push(object_id.clone());
        }
    }

    ocel_filter_object_ids(ocel, &matching_object_ids)
}

/// Filter OCEL by object lifecycle stage
///
/// Keep only objects in specified lifecycle stages.
pub fn ocel_filter_lifecycle_stage(
    ocel: &ObjectCentricEventLog,
    stages: &[String],
) -> ObjectCentricEventLog {
    let stage_set: HashSet<String> = stages.iter().cloned().collect();
    let mut matching_object_ids = Vec::new();

    for object in ocel.objects.values() {
        if let Some(ref stage) = object.lifecycle_stage {
            if stage_set.contains(stage) {
                matching_object_ids.push(object.id.clone());
            }
        }
    }

    ocel_filter_object_ids(ocel, &matching_object_ids)
}

/// Filter OCEL by connected component activity
///
/// Keep only events in connected components containing the specified activity.
pub fn filter_ocel_cc_activity(
    ocel: &ObjectCentricEventLog,
    activity: &str,
) -> ObjectCentricEventLog {
    // Find events with the specified activity
    let mut matching_event_ids = Vec::new();
    for (event_id, (act, _, _)) in &ocel.events {
        if act == activity {
            matching_event_ids.push(*event_id);
        }
    }

    // Find connected components containing these events
    let components = sample_ocel_connected_components(ocel, 1);
    let mut keep_object_ids = HashSet::new();

    for component in components {
        let component_has_activity = component.iter().any(|obj_id| {
            let events = ocel.get_events_for_object(obj_id);
            events
                .iter()
                .any(|(eid, _, _)| matching_event_ids.contains(eid))
        });

        if component_has_activity {
            for obj_id in component {
                keep_object_ids.insert(obj_id.clone());
            }
        }
    }

    let object_ids: Vec<String> = keep_object_ids.into_iter().collect();
    ocel_filter_object_ids(ocel, &object_ids)
}

/// Filter OCEL by connected component length
///
/// Keep only connected components within the specified size range.
pub fn filter_ocel_cc_length(
    ocel: &ObjectCentricEventLog,
    min_length: usize,
    max_length: usize,
) -> ObjectCentricEventLog {
    let components = sample_ocel_connected_components(ocel, 1);
    let mut keep_object_ids = HashSet::new();

    for component in components {
        if component.len() >= min_length && component.len() <= max_length {
            for obj_id in component {
                keep_object_ids.insert(obj_id.clone());
            }
        }
    }

    let object_ids: Vec<String> = keep_object_ids.into_iter().collect();
    ocel_filter_object_ids(ocel, &object_ids)
}

/// Filter OCEL by connected component object
///
/// Keep only connected components containing the specified object.
pub fn filter_ocel_cc_object(
    ocel: &ObjectCentricEventLog,
    object_id: &str,
) -> ObjectCentricEventLog {
    let components = sample_ocel_connected_components(ocel, 1);

    for component in components {
        if component.iter().any(|id| id == object_id) {
            let object_ids: Vec<String> = component.into_iter().collect();
            return ocel_filter_object_ids(ocel, &object_ids);
        }
    }

    ObjectCentricEventLog::new()
}

/// Filter OCEL by connected component object type
///
/// Keep only connected components containing objects of the specified type.
pub fn filter_ocel_cc_otype(
    ocel: &ObjectCentricEventLog,
    object_type: &str,
) -> ObjectCentricEventLog {
    let components = sample_ocel_connected_components(ocel, 1);
    let mut keep_object_ids = HashSet::new();

    for component in components {
        let component_has_type = component.iter().any(|obj_id| {
            if let Some(obj) = ocel.get_object(obj_id) {
                obj.object_type.name == object_type
            } else {
                false
            }
        });

        if component_has_type {
            for obj_id in component {
                keep_object_ids.insert(obj_id.clone());
            }
        }
    }

    let object_ids: Vec<String> = keep_object_ids.into_iter().collect();
    ocel_filter_object_ids(ocel, &object_ids)
}

/// Filter OCEL events
///
/// Keep only events with the specified event IDs.
pub fn filter_ocel_events(
    ocel: &ObjectCentricEventLog,
    event_ids: &[uuid::Uuid],
) -> ObjectCentricEventLog {
    let event_id_set: HashSet<uuid::Uuid> = event_ids.iter().cloned().collect();
    let mut filtered = ObjectCentricEventLog::new();

    // Copy all objects
    for object in ocel.objects.values() {
        filtered.add_object(object.clone());
    }

    // Copy all object types
    for obj_type in &ocel.object_types {
        filtered.register_object_type(obj_type.clone());
    }

    // Copy only specified events
    for (event_id, (activity, timestamp, resource)) in &ocel.events {
        if event_id_set.contains(event_id) {
            filtered
                .events
                .insert(*event_id, (activity.clone(), *timestamp, resource.clone()));
        }
    }

    // Copy mappings for events that were kept
    for mapping in &ocel.event_object_mappings {
        if filtered.events.contains_key(&mapping.event_id) {
            filtered.add_event_object_mapping(mapping.clone());
        }
    }

    // Copy attributes
    filtered.attributes = ocel.attributes.clone();

    filtered
}

/// Filter OCEL events by timestamp
///
/// Keep only events within the specified timestamp range.
pub fn filter_ocel_events_timestamp(
    ocel: &ObjectCentricEventLog,
    min_timestamp: chrono::DateTime<chrono::Utc>,
    max_timestamp: chrono::DateTime<chrono::Utc>,
) -> ObjectCentricEventLog {
    ocel_filter_time_range(ocel, min_timestamp, max_timestamp)
}

/// Filter OCEL end events per object type
///
/// Keep only the last event for each object of specified types.
pub fn filter_ocel_end_events_per_object_type(
    ocel: &ObjectCentricEventLog,
    object_types: &[String],
) -> ObjectCentricEventLog {
    let type_set: HashSet<String> = object_types.iter().cloned().collect();
    let mut filtered = ObjectCentricEventLog::new();
    let mut end_event_ids = HashSet::new();

    // Find end events for each object of specified types
    for object in ocel.objects.values() {
        if type_set.contains(&object.object_type.name) {
            let events = ocel.get_events_for_object(&object.id);

            if let Some((last_event_id, _, _)) = events.last() {
                end_event_ids.insert(*last_event_id);
            }

            filtered.add_object(object.clone());
            filtered.register_object_type(object.object_type.clone());
        }
    }

    // Copy end events
    for (event_id, (activity, timestamp, resource)) in &ocel.events {
        if end_event_ids.contains(event_id) {
            filtered
                .events
                .insert(*event_id, (activity.clone(), *timestamp, resource.clone()));
        }
    }

    // Copy mappings for end events
    for mapping in &ocel.event_object_mappings {
        if filtered.events.contains_key(&mapping.event_id) {
            filtered.add_event_object_mapping(mapping.clone());
        }
    }

    // Copy attributes
    filtered.attributes = ocel.attributes.clone();

    filtered
}

/// Filter OCEL by object count per type
///
/// Keep only objects from types with at least/at most the specified count.
pub fn filter_ocel_object_per_type_count(
    ocel: &ObjectCentricEventLog,
    min_count: usize,
    max_count: usize,
) -> ObjectCentricEventLog {
    let mut type_counts: HashMap<String, usize> = HashMap::new();
    let mut matching_types = HashSet::new();

    // Count objects per type
    for object in ocel.objects.values() {
        *type_counts
            .entry(object.object_type.name.clone())
            .or_insert(0) += 1;
    }

    // Find types within count range
    for (type_name, count) in type_counts {
        if count >= min_count && count <= max_count {
            matching_types.insert(type_name);
        }
    }

    // Use object type filter
    let types: Vec<String> = matching_types.into_iter().collect();
    ocel_filter_object_type(ocel, &types)
}
