//! Object-Centric Event Log structures and operations
//!
//! This module provides the core structures for representing object-centric event logs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use uuid::Uuid;

/// Typed attribute value for OCEL 2.0 — supports all primitive types the standard allows.
///
/// The `#[serde(tag = "type", content = "value")]` encoding round-trips cleanly through
/// JSON: `{"type": "String", "value": "hello"}` or `{"type": "Integer", "value": 42}`.
///
/// `impl From<&str>` and `impl From<String>` ensure all existing callers of
/// `with_attribute("key", "value")` still compile without changes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "value")]
pub enum OcelTypedValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Timestamp(DateTime<Utc>),
}

impl OcelTypedValue {
    /// Returns a string slice if this value is a `String` variant.
    /// Used by `Object::get_attribute` to preserve the `Option<&str>` return type.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            OcelTypedValue::String(s) => Some(s.as_str()),
            _ => None,
        }
    }
}

impl From<&str> for OcelTypedValue {
    fn from(s: &str) -> Self {
        OcelTypedValue::String(s.to_string())
    }
}

impl From<String> for OcelTypedValue {
    fn from(s: String) -> Self {
        OcelTypedValue::String(s)
    }
}

impl From<i64> for OcelTypedValue {
    fn from(i: i64) -> Self {
        OcelTypedValue::Integer(i)
    }
}

impl From<f64> for OcelTypedValue {
    fn from(f: f64) -> Self {
        OcelTypedValue::Float(f)
    }
}

impl From<bool> for OcelTypedValue {
    fn from(b: bool) -> Self {
        OcelTypedValue::Boolean(b)
    }
}

impl From<DateTime<Utc>> for OcelTypedValue {
    fn from(dt: DateTime<Utc>) -> Self {
        OcelTypedValue::Timestamp(dt)
    }
}

/// Object-to-Object relationship for OCEL 2.0 inter-object interaction tracking.
///
/// The paper requires events to reference multiple objects and objects to reference
/// each other (O2O relationships). The `qualifier` field encodes the relationship type
/// (e.g. "contains", "part_of", "follows").
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ObjectRelationship {
    /// The ID of the related object.
    pub object_id: String,
    /// Qualifier describing the relationship type (e.g. "contains", "part_of").
    pub qualifier: String,
}

impl ObjectRelationship {
    pub fn new(object_id: impl Into<String>, qualifier: impl Into<String>) -> Self {
        Self {
            object_id: object_id.into(),
            qualifier: qualifier.into(),
        }
    }
}

/// Represents an object type in the object-centric model
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ObjectType {
    pub name: String,
    pub description: Option<String>,
}

impl ObjectType {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
        }
    }

    pub fn with_description(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: Some(description.into()),
        }
    }
}

/// Represents a single object instance in the process
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Object {
    pub id: String,
    pub object_type: ObjectType,
    pub state: Option<String>,
    pub lifecycle_stage: Option<String>,
    pub creation_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    /// Typed attribute values (OCEL 2.0 — supports String, Integer, Float, Boolean, Timestamp).
    pub attributes: BTreeMap<String, OcelTypedValue>,
    /// Object-to-Object relationships (OCEL 2.0 — inter-object interaction tracking).
    pub relationships: Vec<ObjectRelationship>,
}

impl Object {
    pub fn new(
        id: impl Into<String>,
        object_type: ObjectType,
        creation_time: DateTime<Utc>,
    ) -> Self {
        Self {
            id: id.into(),
            object_type,
            state: None,
            lifecycle_stage: None,
            creation_time,
            end_time: None,
            attributes: BTreeMap::new(),
            relationships: Vec::new(),
        }
    }

    pub fn with_state(mut self, state: impl Into<String>) -> Self {
        self.state = Some(state.into());
        self
    }

    pub fn with_lifecycle_stage(mut self, stage: impl Into<String>) -> Self {
        self.lifecycle_stage = Some(stage.into());
        self
    }

    pub fn with_end_time(mut self, end_time: DateTime<Utc>) -> Self {
        self.end_time = Some(end_time);
        self
    }

    /// Set a typed attribute.  Accepts any type that converts into `OcelTypedValue`,
    /// including `&str` and `String` (backward-compatible with pre-OCEL-2.0 callers).
    pub fn with_attribute(
        mut self,
        key: impl Into<String>,
        value: impl Into<OcelTypedValue>,
    ) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }

    /// Returns a string slice if the attribute is a `String` variant, `None` otherwise.
    /// Preserves the original `Option<&str>` return type for backward compatibility.
    pub fn get_attribute(&self, key: &str) -> Option<&str> {
        self.attributes.get(key).and_then(|v| v.as_str())
    }

    /// Add an O2O relationship to this object.
    pub fn with_relationship(mut self, rel: ObjectRelationship) -> Self {
        self.relationships.push(rel);
        self
    }
}

/// Represents the relationship between an event and the objects it involves
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventToObjectMapping {
    pub event_id: Uuid,
    pub object_ids: HashSet<String>,
    pub object_roles: BTreeMap<String, String>,
}

impl EventToObjectMapping {
    pub fn new(event_id: Uuid) -> Self {
        Self {
            event_id,
            object_ids: HashSet::new(),
            object_roles: BTreeMap::new(),
        }
    }

    pub fn add_object(&mut self, object_id: impl Into<String>) {
        self.object_ids.insert(object_id.into());
    }

    pub fn add_object_with_role(&mut self, object_id: impl Into<String>, role: impl Into<String>) {
        let obj_id = object_id.into();
        self.object_ids.insert(obj_id.clone());
        self.object_roles.insert(obj_id, role.into());
    }

    pub fn get_object_role(&self, object_id: &str) -> Option<&str> {
        self.object_roles.get(object_id).map(|s| s.as_str())
    }
}

/// Object-centric event log that tracks multiple object types and their lifecycles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectCentricEventLog {
    pub id: String,
    pub objects: BTreeMap<String, Object>,
    pub object_types: HashSet<ObjectType>,
    pub events: BTreeMap<Uuid, (String, DateTime<Utc>, Option<String>)>,
    pub event_object_mappings: Vec<EventToObjectMapping>,
    pub attributes: BTreeMap<String, String>,
}

impl ObjectCentricEventLog {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            objects: BTreeMap::new(),
            object_types: HashSet::new(),
            events: BTreeMap::new(),
            event_object_mappings: Vec::new(),
            attributes: BTreeMap::new(),
        }
    }

    pub fn with_id(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            objects: BTreeMap::new(),
            object_types: HashSet::new(),
            events: BTreeMap::new(),
            event_object_mappings: Vec::new(),
            attributes: BTreeMap::new(),
        }
    }

    pub fn register_object_type(&mut self, object_type: ObjectType) {
        self.object_types.insert(object_type);
    }

    pub fn add_object(&mut self, object: Object) {
        self.register_object_type(object.object_type.clone());
        self.objects.insert(object.id.clone(), object);
    }

    pub fn get_object(&self, object_id: &str) -> Option<&Object> {
        self.objects.get(object_id)
    }

    pub fn get_object_mut(&mut self, object_id: &str) -> Option<&mut Object> {
        self.objects.get_mut(object_id)
    }

    pub fn add_event(
        &mut self,
        event_id: Uuid,
        activity: impl Into<String>,
        timestamp: DateTime<Utc>,
        resource: Option<String>,
    ) {
        self.events
            .insert(event_id, (activity.into(), timestamp, resource));
    }

    pub fn add_event_object_mapping(&mut self, mapping: EventToObjectMapping) {
        self.event_object_mappings.push(mapping);
    }

    pub fn get_events_for_object(&self, object_id: &str) -> Vec<(Uuid, String, DateTime<Utc>)> {
        self.event_object_mappings
            .iter()
            .filter(|m| m.object_ids.contains(object_id))
            .filter_map(|m| {
                self.events
                    .get(&m.event_id)
                    .map(|(activity, ts, _)| (m.event_id, activity.clone(), *ts))
            })
            .collect()
    }

    pub fn get_objects_in_event(&self, event_id: Uuid) -> Vec<String> {
        self.event_object_mappings
            .iter()
            .find(|m| m.event_id == event_id)
            .map(|m| m.object_ids.iter().cloned().collect())
            .unwrap_or_default()
    }

    pub fn get_objects_by_type(&self, type_name: &str) -> Vec<&Object> {
        self.objects
            .values()
            .filter(|obj| obj.object_type.name == type_name)
            .collect()
    }

    pub fn get_lifecycle_for_object(&self, object_id: &str) -> Vec<(Uuid, String, DateTime<Utc>)> {
        let mut events = self.get_events_for_object(object_id);
        events.sort_by_key(|(_, _, ts)| *ts);
        events
    }

    pub fn count_objects_by_type(&self) -> BTreeMap<String, usize> {
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        for obj in self.objects.values() {
            *counts.entry(obj.object_type.name.clone()).or_insert(0) += 1;
        }
        counts
    }

    pub fn num_objects(&self) -> usize {
        self.objects.len()
    }

    pub fn num_object_types(&self) -> usize {
        self.object_types.len()
    }

    pub fn num_events(&self) -> usize {
        self.events.len()
    }

    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }

    pub fn get_attribute(&self, key: &str) -> Option<&str> {
        self.attributes.get(key).map(|s| s.as_str())
    }
}

impl Default for ObjectCentricEventLog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_type_creation() {
        let obj_type = ObjectType::new("order");
        assert_eq!(obj_type.name, "order");
        assert_eq!(obj_type.description, None);
    }

    #[test]
    fn test_object_type_with_description() {
        let obj_type = ObjectType::with_description("order", "Customer order");
        assert_eq!(obj_type.name, "order");
        assert_eq!(obj_type.description, Some("Customer order".to_string()));
    }

    #[test]
    fn test_object_creation() {
        let now = Utc::now();
        let obj_type = ObjectType::new("order");
        let obj = Object::new("order_123", obj_type, now);

        assert_eq!(obj.id, "order_123");
        assert_eq!(obj.object_type.name, "order");
        assert_eq!(obj.creation_time, now);
        assert_eq!(obj.state, None);
    }

    #[test]
    fn test_object_with_state_and_lifecycle() {
        let now = Utc::now();
        let obj_type = ObjectType::new("order");
        let obj = Object::new("order_123", obj_type, now)
            .with_state("pending")
            .with_lifecycle_stage("processing");

        assert_eq!(obj.state, Some("pending".to_string()));
        assert_eq!(obj.lifecycle_stage, Some("processing".to_string()));
    }

    #[test]
    fn test_object_attributes_string() {
        let now = Utc::now();
        let obj_type = ObjectType::new("order");
        let obj = Object::new("order_123", obj_type, now)
            .with_attribute("customer", "john")
            .with_attribute("amount", "100");

        assert_eq!(obj.get_attribute("customer"), Some("john"));
        assert_eq!(obj.get_attribute("amount"), Some("100"));
        assert_eq!(obj.get_attribute("nonexistent"), None);
    }

    #[test]
    fn test_ocel_typed_value_integer_returns_none_for_as_str() {
        let now = Utc::now();
        let obj_type = ObjectType::new("order");
        let mut obj = Object::new("order_123", obj_type, now);
        obj.attributes
            .insert("priority".to_string(), OcelTypedValue::Integer(42));
        // Integer does not implement as_str
        assert_eq!(obj.get_attribute("priority"), None);
        // But we can retrieve the typed value directly
        assert_eq!(
            obj.attributes.get("priority"),
            Some(&OcelTypedValue::Integer(42))
        );
    }

    #[test]
    fn test_ocel_typed_value_from_impls() {
        let v: OcelTypedValue = "hello".into();
        assert_eq!(v, OcelTypedValue::String("hello".to_string()));

        let v: OcelTypedValue = String::from("world").into();
        assert_eq!(v, OcelTypedValue::String("world".to_string()));

        let v: OcelTypedValue = 42i64.into();
        assert_eq!(v, OcelTypedValue::Integer(42));

        let v: OcelTypedValue = 3.14f64.into();
        assert_eq!(v, OcelTypedValue::Float(3.14));

        let v: OcelTypedValue = true.into();
        assert_eq!(v, OcelTypedValue::Boolean(true));
    }

    #[test]
    fn test_object_relationship() {
        let now = Utc::now();
        let obj_type = ObjectType::new("order");
        let rel = ObjectRelationship::new("item_1", "contains");
        let obj = Object::new("order_123", obj_type, now).with_relationship(rel);

        assert_eq!(obj.relationships.len(), 1);
        assert_eq!(obj.relationships[0].object_id, "item_1");
        assert_eq!(obj.relationships[0].qualifier, "contains");
    }

    #[test]
    fn test_event_to_object_mapping() {
        let event_id = Uuid::new_v4();
        let mut mapping = EventToObjectMapping::new(event_id);

        mapping.add_object("order_123");
        mapping.add_object_with_role("item_456", "line_item");

        assert!(mapping.object_ids.contains("order_123"));
        assert!(mapping.object_ids.contains("item_456"));
        assert_eq!(mapping.get_object_role("item_456"), Some("line_item"));
    }

    #[test]
    fn test_object_centric_event_log_operations() {
        let mut log = ObjectCentricEventLog::new();
        let now = Utc::now();
        let order_type = ObjectType::new("order");
        let item_type = ObjectType::new("item");

        let order = Object::new("order_123", order_type, now).with_state("pending");
        let item = Object::new("item_456", item_type, now).with_state("allocated");

        log.add_object(order);
        log.add_object(item);

        assert_eq!(log.num_objects(), 2);
        assert_eq!(log.num_object_types(), 2);
        assert!(log.get_object("order_123").is_some());
        assert!(log.get_object("item_456").is_some());
    }

    #[test]
    fn test_object_centric_log_event_mapping() {
        let mut log = ObjectCentricEventLog::new();
        let now = Utc::now();
        let event_id = Uuid::new_v4();
        let order_type = ObjectType::new("order");

        let order = Object::new("order_123", order_type, now);
        log.add_object(order);
        log.add_event(event_id, "process_order", now, Some("user1".to_string()));

        let mut mapping = EventToObjectMapping::new(event_id);
        mapping.add_object("order_123");
        log.add_event_object_mapping(mapping);

        let events_for_order = log.get_events_for_object("order_123");
        assert_eq!(events_for_order.len(), 1);
        assert_eq!(events_for_order[0].1, "process_order");
    }

    #[test]
    fn test_object_centric_log_count_by_type() {
        let mut log = ObjectCentricEventLog::new();
        let now = Utc::now();
        let order_type = ObjectType::new("order");

        log.add_object(Object::new("order_1", order_type.clone(), now));
        log.add_object(Object::new("order_2", order_type.clone(), now));
        log.add_object(Object::new("order_3", order_type.clone(), now));

        let counts = log.count_objects_by_type();
        assert_eq!(counts.get("order"), Some(&3));
    }
}
