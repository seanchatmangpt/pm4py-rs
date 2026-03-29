//! BOS ↔ BusinessOS Serialization Bridge
//!
//! This module provides:
//! - Type-safe conversion between Rust and Go representations
//! - JSON serialization with RFC3339 timestamps
//! - MessagePack binary format for efficient transport
//! - Comprehensive validation and error handling
//! - Round-trip conversion guarantees

use crate::log::{Event, EventLog, Trace};
use crate::models::{Arc, PetriNet, Place, Transition};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

// ============================================================================
// Error Types
// ============================================================================

/// Errors that can occur during serialization/deserialization
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BosError {
    /// JSON serialization failed
    SerializationFailed(String),
    /// JSON deserialization failed
    DeserializationFailed(String),
    /// Invalid timestamp format or value
    InvalidTimestamp(String),
    /// Invalid UUID format
    InvalidUuid(String),
    /// Collection size exceeded maximum
    MaxSizeExceeded { max: usize, actual: usize },
    /// Invalid activity name
    InvalidActivity { activity: String, reason: String },
    /// Invalid trace ID
    InvalidTraceId(String),
    /// Empty collection where non-empty required
    EmptyCollection(String),
    /// Validation error with field path
    ValidationError { field: String, reason: String },
    /// Referential integrity violation (e.g., missing arc target)
    ReferenceError { reference: String, target: String },
    /// Type conversion overflow
    OutOfRange { value: String, min: i64, max: i64 },
}

impl std::fmt::Display for BosError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SerializationFailed(msg) => write!(f, "Serialization failed: {}", msg),
            Self::DeserializationFailed(msg) => write!(f, "Deserialization failed: {}", msg),
            Self::InvalidTimestamp(msg) => write!(f, "Invalid timestamp: {}", msg),
            Self::InvalidUuid(msg) => write!(f, "Invalid UUID: {}", msg),
            Self::MaxSizeExceeded { max, actual } => {
                write!(f, "Max size {} exceeded, got {}", max, actual)
            }
            Self::InvalidActivity { activity, reason } => {
                write!(f, "Invalid activity '{}': {}", activity, reason)
            }
            Self::InvalidTraceId(id) => write!(f, "Invalid trace ID: {}", id),
            Self::EmptyCollection(what) => write!(f, "Empty {}, expected non-empty", what),
            Self::ValidationError { field, reason } => {
                write!(f, "Validation error at '{}': {}", field, reason)
            }
            Self::ReferenceError { reference, target } => {
                write!(f, "Reference '{}' target '{}' not found", reference, target)
            }
            Self::OutOfRange { value, min, max } => {
                write!(f, "Value {} out of range [{}, {}]", value, min, max)
            }
        }
    }
}

impl std::error::Error for BosError {}

pub type Result<T> = std::result::Result<T, BosError>;

// ============================================================================
// Go-Compatible Intermediate Representations
// ============================================================================

/// JSON-serializable Event (intermediate representation)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(crate = "serde")]
pub struct EventJson {
    pub activity: String,
    pub timestamp: String, // RFC3339
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource: Option<String>,
    pub attributes: BTreeMap<String, String>,
    pub id: String,
}

/// JSON-serializable Trace (intermediate representation)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(crate = "serde")]
pub struct TraceJson {
    pub id: String,
    pub events: Vec<EventJson>,
    pub attributes: BTreeMap<String, String>,
}

/// JSON-serializable EventLog (intermediate representation)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(crate = "serde")]
pub struct EventLogJson {
    pub traces: Vec<TraceJson>,
    pub attributes: BTreeMap<String, String>,
}

/// JSON-serializable Place
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(crate = "serde")]
pub struct PlaceJson {
    pub id: String,
    pub name: String,
    pub initial_marking: i32,
    pub final_marking: Option<i32>,
}

/// JSON-serializable Transition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(crate = "serde")]
pub struct TransitionJson {
    pub id: String,
    pub label: Option<String>,
    pub name: String,
}

/// JSON-serializable Arc
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(crate = "serde")]
pub struct ArcJson {
    pub from: String,
    pub to: String,
    pub weight: i32,
}

/// JSON-serializable PetriNet
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(crate = "serde")]
pub struct PetriNetJson {
    pub places: Vec<PlaceJson>,
    pub transitions: Vec<TransitionJson>,
    pub arcs: Vec<ArcJson>,
    pub initial_place: Option<String>,
    pub final_place: Option<String>,
}

// ============================================================================
// Validation Functions
// ============================================================================

/// Validate an activity name
fn validate_activity(activity: &str) -> Result<()> {
    if activity.is_empty() {
        return Err(BosError::InvalidActivity {
            activity: activity.to_string(),
            reason: "Activity cannot be empty".to_string(),
        });
    }
    if activity.len() > 255 {
        return Err(BosError::InvalidActivity {
            activity: activity.to_string(),
            reason: format!("Activity exceeds 255 chars: {}", activity.len()),
        });
    }
    if activity != activity.trim() {
        return Err(BosError::InvalidActivity {
            activity: activity.to_string(),
            reason: "Activity has leading/trailing whitespace".to_string(),
        });
    }
    Ok(())
}

/// Validate a trace ID
fn validate_trace_id(id: &str) -> Result<()> {
    if id.is_empty() || id.len() > 100 {
        return Err(BosError::InvalidTraceId(id.to_string()));
    }
    Ok(())
}

/// Validate a timestamp (not in future)
fn validate_timestamp(ts: DateTime<Utc>) -> Result<()> {
    let now = Utc::now();
    if ts > now {
        return Err(BosError::InvalidTimestamp(format!(
            "Timestamp {} is in the future",
            ts.to_rfc3339()
        )));
    }
    Ok(())
}

/// Validate resource name
fn validate_resource(resource: &str) -> Result<()> {
    if resource.is_empty() || resource.len() > 100 {
        return Err(BosError::ValidationError {
            field: "resource".to_string(),
            reason: "Resource must be 1-100 chars".to_string(),
        });
    }
    Ok(())
}

/// Validate attributes map
fn validate_attributes(attrs: &BTreeMap<String, String>) -> Result<()> {
    if attrs.len() > 50 {
        return Err(BosError::MaxSizeExceeded {
            max: 50,
            actual: attrs.len(),
        });
    }
    for (k, v) in attrs {
        if k.is_empty() || k.len() > 50 {
            return Err(BosError::ValidationError {
                field: "attributes.key".to_string(),
                reason: format!("Key '{}' must be 1-50 chars", k),
            });
        }
        if v.len() > 1000 {
            return Err(BosError::ValidationError {
                field: format!("attributes.{}", k),
                reason: format!("Value exceeds 1000 chars: {}", v.len()),
            });
        }
    }
    Ok(())
}

/// Validate marking count
fn validate_marking(count: i32) -> Result<()> {
    if !(0..=1000).contains(&count) {
        return Err(BosError::OutOfRange {
            value: count.to_string(),
            min: 0,
            max: 1000,
        });
    }
    Ok(())
}

// ============================================================================
// Event Serialization
// ============================================================================

/// Convert Rust Event to JSON representation
pub fn event_to_json(event: &Event) -> Result<EventJson> {
    validate_activity(&event.activity)?;
    validate_timestamp(event.timestamp)?;
    if let Some(ref resource) = event.resource {
        validate_resource(resource)?;
    }
    validate_attributes(&event.attributes)?;

    Ok(EventJson {
        activity: event.activity.clone(),
        timestamp: event.timestamp.to_rfc3339(),
        resource: event.resource.clone(),
        attributes: event.attributes.clone(),
        id: event.id.to_string(),
    })
}

/// Convert JSON representation to Rust Event
pub fn event_from_json(json: &EventJson) -> Result<Event> {
    validate_activity(&json.activity)?;
    validate_attributes(&json.attributes)?;

    let timestamp = DateTime::parse_from_rfc3339(&json.timestamp)
        .map_err(|e| BosError::InvalidTimestamp(format!("Failed to parse timestamp: {}", e)))?
        .with_timezone(&Utc);

    validate_timestamp(timestamp)?;

    if let Some(ref resource) = json.resource {
        validate_resource(resource)?;
    }

    let id = Uuid::parse_str(&json.id)
        .map_err(|e| BosError::InvalidUuid(format!("Invalid UUID: {}", e)))?;

    Ok(Event {
        activity: json.activity.clone(),
        timestamp,
        resource: json.resource.clone(),
        attributes: json.attributes.clone(),
        id,
    })
}

// ============================================================================
// Trace Serialization
// ============================================================================

/// Convert Rust Trace to JSON representation
pub fn trace_to_json(trace: &Trace) -> Result<TraceJson> {
    validate_trace_id(&trace.id)?;
    validate_attributes(&trace.attributes)?;

    if trace.events.is_empty() {
        return Err(BosError::EmptyCollection("events".to_string()));
    }

    let events = trace
        .events
        .iter()
        .map(event_to_json)
        .collect::<Result<Vec<_>>>()?;

    Ok(TraceJson {
        id: trace.id.clone(),
        events,
        attributes: trace.attributes.clone(),
    })
}

/// Convert JSON representation to Rust Trace
pub fn trace_from_json(json: &TraceJson) -> Result<Trace> {
    validate_trace_id(&json.id)?;
    validate_attributes(&json.attributes)?;

    if json.events.is_empty() {
        return Err(BosError::EmptyCollection("events".to_string()));
    }

    let events = json
        .events
        .iter()
        .map(event_from_json)
        .collect::<Result<Vec<_>>>()?;

    Ok(Trace {
        id: json.id.clone(),
        events,
        attributes: json.attributes.clone(),
    })
}

// ============================================================================
// EventLog Serialization
// ============================================================================

/// Convert Rust EventLog to JSON representation
pub fn event_log_to_json(log: &EventLog) -> Result<EventLogJson> {
    validate_attributes(&log.attributes)?;

    let traces = log
        .traces
        .iter()
        .map(trace_to_json)
        .collect::<Result<Vec<_>>>()?;

    Ok(EventLogJson {
        traces,
        attributes: log.attributes.clone(),
    })
}

/// Convert JSON representation to Rust EventLog
pub fn event_log_from_json(json: &EventLogJson) -> Result<EventLog> {
    validate_attributes(&json.attributes)?;

    let traces = json
        .traces
        .iter()
        .map(trace_from_json)
        .collect::<Result<Vec<_>>>()?;

    Ok(EventLog {
        traces,
        attributes: json.attributes.clone(),
    })
}

/// Convert EventLog to JSON string (with error context)
pub fn event_log_to_json_string(log: &EventLog) -> Result<String> {
    let json = event_log_to_json(log)?;
    serde_json::to_string(&json)
        .map_err(|e| BosError::SerializationFailed(format!("JSON serialization: {}", e)))
}

/// Parse EventLog from JSON string (with error context)
pub fn event_log_from_json_string(json_str: &str) -> Result<EventLog> {
    let json: EventLogJson = serde_json::from_str(json_str)
        .map_err(|e| BosError::DeserializationFailed(format!("JSON parsing: {}", e)))?;
    event_log_from_json(&json)
}

// ============================================================================
// PetriNet Serialization
// ============================================================================

/// Convert Rust PetriNet to JSON representation
pub fn petri_net_to_json(net: &PetriNet) -> Result<PetriNetJson> {
    if net.places.is_empty() {
        return Err(BosError::EmptyCollection("places".to_string()));
    }
    if net.transitions.is_empty() {
        return Err(BosError::EmptyCollection("transitions".to_string()));
    }

    // Convert places
    let places = net
        .places
        .iter()
        .map(|p| {
            validate_marking(p.initial_marking as i32)?;
            if let Some(final_m) = p.final_marking {
                validate_marking(final_m as i32)?;
            }
            Ok(PlaceJson {
                id: p.id.clone(),
                name: p.name.clone(),
                initial_marking: p.initial_marking as i32,
                final_marking: p.final_marking.map(|m| m as i32),
            })
        })
        .collect::<Result<Vec<_>>>()?;

    // Convert transitions
    let transitions = net
        .transitions
        .iter()
        .map(|t| {
            Ok(TransitionJson {
                id: t.id.clone(),
                label: t.label.clone(),
                name: t.name.clone(),
            })
        })
        .collect::<Result<Vec<_>>>()?;

    // Convert arcs and validate references
    let place_ids: std::collections::HashSet<_> = net.places.iter().map(|p| p.id.clone()).collect();
    let trans_ids: std::collections::HashSet<_> =
        net.transitions.iter().map(|t| t.id.clone()).collect();

    let arcs = net
        .arcs
        .iter()
        .map(|a| {
            if !place_ids.contains(&a.from) && !trans_ids.contains(&a.from) {
                return Err(BosError::ReferenceError {
                    reference: "arc.from".to_string(),
                    target: a.from.clone(),
                });
            }
            if !place_ids.contains(&a.to) && !trans_ids.contains(&a.to) {
                return Err(BosError::ReferenceError {
                    reference: "arc.to".to_string(),
                    target: a.to.clone(),
                });
            }
            validate_marking(a.weight as i32)?;
            Ok(ArcJson {
                from: a.from.clone(),
                to: a.to.clone(),
                weight: a.weight as i32,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(PetriNetJson {
        places,
        transitions,
        arcs,
        initial_place: net.initial_place.clone(),
        final_place: net.final_place.clone(),
    })
}

/// Convert JSON representation to Rust PetriNet
pub fn petri_net_from_json(json: &PetriNetJson) -> Result<PetriNet> {
    if json.places.is_empty() {
        return Err(BosError::EmptyCollection("places".to_string()));
    }
    if json.transitions.is_empty() {
        return Err(BosError::EmptyCollection("transitions".to_string()));
    }

    // Convert places
    let places = json
        .places
        .iter()
        .map(|p| {
            validate_marking(p.initial_marking)?;
            if let Some(final_m) = p.final_marking {
                validate_marking(final_m)?;
            }
            Ok(Place {
                id: p.id.clone(),
                name: p.name.clone(),
                initial_marking: p.initial_marking as usize,
                final_marking: p.final_marking.map(|m| m as usize),
            })
        })
        .collect::<Result<Vec<_>>>()?;

    // Convert transitions
    let transitions = json
        .transitions
        .iter()
        .map(|t| {
            Ok(Transition {
                id: t.id.clone(),
                label: t.label.clone(),
                name: t.name.clone(),
            })
        })
        .collect::<Result<Vec<_>>>()?;

    // Convert arcs and validate references
    let place_ids: std::collections::HashSet<_> = places.iter().map(|p| p.id.clone()).collect();
    let trans_ids: std::collections::HashSet<_> =
        transitions.iter().map(|t| t.id.clone()).collect();

    let arcs = json
        .arcs
        .iter()
        .map(|a| {
            if !place_ids.contains(&a.from) && !trans_ids.contains(&a.from) {
                return Err(BosError::ReferenceError {
                    reference: "arc.from".to_string(),
                    target: a.from.clone(),
                });
            }
            if !place_ids.contains(&a.to) && !trans_ids.contains(&a.to) {
                return Err(BosError::ReferenceError {
                    reference: "arc.to".to_string(),
                    target: a.to.clone(),
                });
            }
            validate_marking(a.weight)?;
            Ok(Arc {
                from: a.from.clone(),
                to: a.to.clone(),
                weight: a.weight as usize,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(PetriNet {
        places,
        transitions,
        arcs,
        initial_place: json.initial_place.clone(),
        final_place: json.final_place.clone(),
    })
}

/// Convert PetriNet to JSON string
pub fn petri_net_to_json_string(net: &PetriNet) -> Result<String> {
    let json = petri_net_to_json(net)?;
    serde_json::to_string(&json)
        .map_err(|e| BosError::SerializationFailed(format!("JSON serialization: {}", e)))
}

/// Parse PetriNet from JSON string
pub fn petri_net_from_json_string(json_str: &str) -> Result<PetriNet> {
    let json: PetriNetJson = serde_json::from_str(json_str)
        .map_err(|e| BosError::DeserializationFailed(format!("JSON parsing: {}", e)))?;
    petri_net_from_json(&json)
}

// ============================================================================
// MessagePack Binary Format (Optional)
// ============================================================================

/// Serialize EventLog to MessagePack bytes (compact binary format)
#[cfg(feature = "msgpack")]
pub fn event_log_to_msgpack(log: &EventLog) -> Result<Vec<u8>> {
    let json = event_log_to_json(log)?;
    rmp_serde::to_vec(&json)
        .map_err(|e| BosError::SerializationFailed(format!("MessagePack serialization: {}", e)))
}

/// Deserialize EventLog from MessagePack bytes
#[cfg(feature = "msgpack")]
pub fn event_log_from_msgpack(bytes: &[u8]) -> Result<EventLog> {
    let json: EventLogJson = rmp_serde::from_slice(bytes)
        .map_err(|e| BosError::DeserializationFailed(format!("MessagePack parsing: {}", e)))?;
    event_log_from_json(&json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_event_round_trip() {
        let mut event = Event::new("Test Activity", Utc::now());
        event.resource = Some("User1".to_string());
        event
            .attributes
            .insert("key".to_string(), "value".to_string());

        let json = event_to_json(&event).unwrap();
        let restored = event_from_json(&json).unwrap();

        assert_eq!(restored.activity, event.activity);
        assert_eq!(restored.timestamp, event.timestamp);
        assert_eq!(restored.resource, event.resource);
        assert_eq!(restored.attributes, event.attributes);
    }

    #[test]
    fn test_trace_validation() {
        let mut trace = Trace::new("case-1");
        trace.add_event(Event::new("Activity1", Utc::now()));

        trace_to_json(&trace).unwrap();
    }

    #[test]
    fn test_empty_trace_rejected() {
        let trace = Trace::new("case-1");
        assert!(trace_to_json(&trace).is_err());
    }

    #[test]
    fn test_invalid_activity() {
        let event = Event::new("", Utc::now());
        assert!(event_to_json(&event).is_err());
    }

    #[test]
    fn test_event_log_json_string() {
        let mut log = EventLog::new();
        let mut trace = Trace::new("case-1");
        trace.add_event(Event::new("Activity1", Utc::now()));
        log.add_trace(trace);

        let json_str = event_log_to_json_string(&log).unwrap();
        let restored = event_log_from_json_string(&json_str).unwrap();

        assert_eq!(restored.traces.len(), 1);
        assert_eq!(restored.traces[0].id, "case-1");
    }
}
