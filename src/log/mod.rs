use chrono::{DateTime, Utc};
/// Event log data structures and operations
///
/// This module provides the core structures for representing event logs:
/// - Event: A single activity occurrence
/// - Trace: A sequence of events for a process instance
/// - EventLog: A collection of traces
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

pub mod advanced_filters;
pub mod dfg_filters;
pub mod operations;
pub mod statistical_filters;
pub mod temporal_filter;
pub mod trace_abstraction;

pub use advanced_filters::{
    filter_activities_rework, filter_activity_done_different_resources, filter_case_size,
    filter_event_attribute_values, filter_time_range, filter_trace_attribute, filter_trace_prefix,
    filter_trace_suffix, filter_traces_containing_activity, filter_traces_with_activity,
    filter_variants_top_k, get_event_attribute_values, get_event_attributes,
    get_trace_attribute_values, get_trace_attributes, AdvancedFilter, FilterChain, FilterResult,
};
pub use dfg_filters::{
    filter_between, filter_dfg_activities_percentage, filter_dfg_paths_percentage,
    filter_directly_follows_relation, filter_eventually_follows_relation,
    filter_four_eyes_principle, filter_log_relative_occurrence_event_attribute,
    filter_paths_performance, filter_variants_by_coverage_percentage,
};
pub use operations::*;
pub use statistical_filters::{OutlierAnalysis, OutlierDetectionMethod, StatisticalFilter};
pub use temporal_filter::{BusinessHours, TemporalFilter, TemporalStatistics, TimeRange};
pub use trace_abstraction::{AbstractionRule, AbstractionStatistics, ActivityAbstractor};

/// Represents a single event in a process
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Event {
    /// Activity name
    pub activity: String,

    /// Event timestamp
    pub timestamp: DateTime<Utc>,

    /// Resource that performed the activity
    pub resource: Option<String>,

    /// Custom attributes
    pub attributes: BTreeMap<String, String>,

    /// Event ID
    #[serde(default = "uuid::Uuid::new_v4")]
    pub id: Uuid,
}

impl Event {
    /// Create a new event
    pub fn new(activity: impl Into<String>, timestamp: DateTime<Utc>) -> Self {
        Self {
            activity: activity.into(),
            timestamp,
            resource: None,
            attributes: BTreeMap::new(),
            id: Uuid::new_v4(),
        }
    }

    /// Set the resource
    pub fn with_resource(mut self, resource: impl Into<String>) -> Self {
        self.resource = Some(resource.into());
        self
    }

    /// Add a custom attribute
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }

    /// Get an attribute value
    pub fn get_attribute(&self, key: &str) -> Option<&str> {
        self.attributes.get(key).map(|s| s.as_str())
    }
}

/// Represents a trace (sequence of events) for a process instance
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Trace {
    /// Trace ID (case ID)
    pub id: String,

    /// Sequence of events
    pub events: Vec<Event>,

    /// Custom attributes for the trace
    pub attributes: BTreeMap<String, String>,
}

impl Trace {
    /// Create a new empty trace
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            events: Vec::new(),
            attributes: BTreeMap::new(),
        }
    }

    /// Add an event to the trace
    pub fn add_event(&mut self, event: Event) {
        self.events.push(event);
    }

    /// Get number of events in trace
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Check if trace is empty
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Get events sorted by timestamp
    pub fn events_sorted(&self) -> Vec<&Event> {
        let mut events = self.events.iter().collect::<Vec<_>>();
        events.sort_by_key(|e| e.timestamp);
        events
    }

    /// Add a custom attribute
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }

    /// Get an attribute value
    pub fn get_attribute(&self, key: &str) -> Option<&str> {
        self.attributes.get(key).map(|s| s.as_str())
    }
}

/// Represents a complete event log (collection of traces)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventLog {
    /// Collection of traces
    pub traces: Vec<Trace>,

    /// Global attributes
    pub attributes: BTreeMap<String, String>,
}

impl EventLog {
    /// Create a new empty event log
    pub fn new() -> Self {
        Self {
            traces: Vec::new(),
            attributes: BTreeMap::new(),
        }
    }

    /// Add a trace to the log
    pub fn add_trace(&mut self, trace: Trace) {
        self.traces.push(trace);
    }

    /// Get number of traces
    pub fn len(&self) -> usize {
        self.traces.len()
    }

    /// Check if log is empty
    pub fn is_empty(&self) -> bool {
        self.traces.is_empty()
    }

    /// Get total number of events
    pub fn num_events(&self) -> usize {
        self.traces.iter().map(|t| t.len()).sum()
    }

    /// Get all unique activities
    pub fn activities(&self) -> Vec<String> {
        let mut activities: Vec<_> = self
            .traces
            .iter()
            .flat_map(|t| t.events.iter().map(|e| e.activity.clone()))
            .collect();
        activities.sort();
        activities.dedup();
        activities
    }

    /// Add a global attribute
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }

    /// Get attribute value
    pub fn get_attribute(&self, key: &str) -> Option<&str> {
        self.attributes.get(key).map(|s| s.as_str())
    }

    /// Filter traces by activity
    pub fn filter_by_activity(&self, activity: &str) -> EventLog {
        let mut filtered = EventLog::new();
        filtered.attributes = self.attributes.clone();

        for trace in &self.traces {
            if trace.events.iter().any(|e| e.activity == activity) {
                filtered.add_trace(trace.clone());
            }
        }

        filtered
    }

    /// Filter traces by minimum length
    pub fn filter_by_min_length(&self, min_length: usize) -> EventLog {
        let mut filtered = EventLog::new();
        filtered.attributes = self.attributes.clone();

        for trace in &self.traces {
            if trace.len() >= min_length {
                filtered.add_trace(trace.clone());
            }
        }

        filtered
    }

    /// Get traces for a specific case ID
    pub fn get_trace(&self, case_id: &str) -> Option<&Trace> {
        self.traces.iter().find(|t| t.id == case_id)
    }

    /// Get mutable trace by case ID
    pub fn get_trace_mut(&mut self, case_id: &str) -> Option<&mut Trace> {
        self.traces.iter_mut().find(|t| t.id == case_id)
    }
}

impl Default for EventLog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_event_creation() {
        let now = Utc::now();
        let event = Event::new("activity_a", now);

        assert_eq!(event.activity, "activity_a");
        assert_eq!(event.timestamp, now);
        assert_eq!(event.resource, None);
    }

    #[test]
    fn test_trace_operations() {
        let mut trace = Trace::new("case_1");
        let now = Utc::now();

        trace.add_event(Event::new("activity_a", now));
        trace.add_event(Event::new("activity_b", now));

        assert_eq!(trace.len(), 2);
        assert!(!trace.is_empty());
    }

    #[test]
    fn test_event_log_operations() {
        let mut log = EventLog::new();
        let mut trace = Trace::new("case_1");
        let now = Utc::now();

        trace.add_event(Event::new("activity_a", now));
        trace.add_event(Event::new("activity_b", now));

        log.add_trace(trace);

        assert_eq!(log.len(), 1);
        assert_eq!(log.num_events(), 2);
        assert_eq!(log.activities(), vec!["activity_a", "activity_b"]);
    }
}
