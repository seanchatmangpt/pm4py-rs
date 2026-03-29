/// Advanced filtering operations for event logs
use crate::log::EventLog;
use chrono::{DateTime, Utc};
use std::collections::BTreeMap;

/// Result of a filter operation
#[derive(Debug, Clone)]
pub struct FilterResult {
    pub log: EventLog,
    pub original_count: usize,
    pub filtered_count: usize,
}

impl FilterResult {
    pub fn new(log: EventLog, original_count: usize) -> Self {
        let filtered_count = log.len();
        Self {
            log,
            original_count,
            filtered_count,
        }
    }

    pub fn retention_rate(&self) -> f64 {
        if self.original_count == 0 {
            0.0
        } else {
            self.filtered_count as f64 / self.original_count as f64
        }
    }
}

/// Advanced filter for event logs
pub struct AdvancedFilter;

impl AdvancedFilter {
    /// Filter by activity variant (exact sequence)
    pub fn by_variant(log: &EventLog, activities: &[&str]) -> FilterResult {
        let mut filtered = EventLog::new();
        filtered.attributes = log.attributes.clone();

        for trace in &log.traces {
            let trace_activities: Vec<&str> =
                trace.events.iter().map(|e| e.activity.as_str()).collect();

            if trace_activities == activities {
                filtered.add_trace(trace.clone());
            }
        }

        FilterResult::new(filtered, log.len())
    }

    /// Filter by time range
    pub fn by_time_range(log: &EventLog, start: DateTime<Utc>, end: DateTime<Utc>) -> FilterResult {
        let mut filtered = EventLog::new();
        filtered.attributes = log.attributes.clone();

        for trace in &log.traces {
            if trace
                .events
                .iter()
                .any(|e| e.timestamp >= start && e.timestamp <= end)
            {
                filtered.add_trace(trace.clone());
            }
        }

        FilterResult::new(filtered, log.len())
    }

    /// Filter by resource/performer
    pub fn by_resource(log: &EventLog, resource: &str) -> FilterResult {
        let mut filtered = EventLog::new();
        filtered.attributes = log.attributes.clone();

        for trace in &log.traces {
            if trace
                .events
                .iter()
                .any(|e| e.resource.as_ref().map(|r| r == resource).unwrap_or(false))
            {
                filtered.add_trace(trace.clone());
            }
        }

        FilterResult::new(filtered, log.len())
    }

    /// Filter by frequency percentile (keep top N%)
    pub fn by_frequency_percentile(log: &EventLog, percentile: f64) -> FilterResult {
        if !(0.0..=1.0).contains(&percentile) {
            return FilterResult::new(log.clone(), log.len());
        }

        let mut variant_counts: BTreeMap<Vec<String>, usize> = BTreeMap::new();

        for trace in &log.traces {
            let variant: Vec<String> = trace.events.iter().map(|e| e.activity.clone()).collect();
            *variant_counts.entry(variant).or_insert(0) += 1;
        }

        let mut variants: Vec<_> = variant_counts.into_iter().collect();
        variants.sort_by(|a, b| b.1.cmp(&a.1));

        let total: usize = variants.iter().map(|v| v.1).sum();
        let mut cumulative = 0;
        let threshold = (total as f64 * percentile).ceil() as usize;

        let mut kept_variants = std::collections::HashSet::new();
        for (variant, count) in variants {
            if cumulative >= threshold {
                break;
            }
            kept_variants.insert(variant);
            cumulative += count;
        }

        let mut filtered = EventLog::new();
        filtered.attributes = log.attributes.clone();

        for trace in &log.traces {
            let variant: Vec<String> = trace.events.iter().map(|e| e.activity.clone()).collect();
            if kept_variants.contains(&variant) {
                filtered.add_trace(trace.clone());
            }
        }

        FilterResult::new(filtered, log.len())
    }

    /// Filter by case duration range (in seconds)
    pub fn by_case_duration(log: &EventLog, min_duration: f64, max_duration: f64) -> FilterResult {
        let mut filtered = EventLog::new();
        filtered.attributes = log.attributes.clone();

        for trace in &log.traces {
            if trace.events.is_empty() {
                continue;
            }

            let first_time = trace.events.first().unwrap().timestamp;
            let last_time = trace.events.last().unwrap().timestamp;

            let duration_seconds = (last_time - first_time).num_seconds() as f64;

            if duration_seconds >= min_duration && duration_seconds <= max_duration {
                filtered.add_trace(trace.clone());
            }
        }

        FilterResult::new(filtered, log.len())
    }

    /// Filter by minimum activity count
    pub fn by_min_length(log: &EventLog, min_length: usize) -> FilterResult {
        let mut filtered = EventLog::new();
        filtered.attributes = log.attributes.clone();

        for trace in &log.traces {
            if trace.len() >= min_length {
                filtered.add_trace(trace.clone());
            }
        }

        FilterResult::new(filtered, log.len())
    }

    /// Filter by maximum activity count
    pub fn by_max_length(log: &EventLog, max_length: usize) -> FilterResult {
        let mut filtered = EventLog::new();
        filtered.attributes = log.attributes.clone();

        for trace in &log.traces {
            if trace.len() <= max_length {
                filtered.add_trace(trace.clone());
            }
        }

        FilterResult::new(filtered, log.len())
    }

    /// Filter by starting activity
    pub fn by_start_activity(log: &EventLog, activity: &str) -> FilterResult {
        let mut filtered = EventLog::new();
        filtered.attributes = log.attributes.clone();

        for trace in &log.traces {
            if let Some(first) = trace.events.first() {
                if first.activity == activity {
                    filtered.add_trace(trace.clone());
                }
            }
        }

        FilterResult::new(filtered, log.len())
    }

    /// Filter by ending activity
    pub fn by_end_activity(log: &EventLog, activity: &str) -> FilterResult {
        let mut filtered = EventLog::new();
        filtered.attributes = log.attributes.clone();

        for trace in &log.traces {
            if let Some(last) = trace.events.last() {
                if last.activity == activity {
                    filtered.add_trace(trace.clone());
                }
            }
        }

        FilterResult::new(filtered, log.len())
    }

    /// Filter by containing activity
    pub fn by_containing_activity(log: &EventLog, activity: &str) -> FilterResult {
        let mut filtered = EventLog::new();
        filtered.attributes = log.attributes.clone();

        for trace in &log.traces {
            if trace.events.iter().any(|e| e.activity == activity) {
                filtered.add_trace(trace.clone());
            }
        }

        FilterResult::new(filtered, log.len())
    }

    /// Filter by not containing activity
    pub fn by_not_containing_activity(log: &EventLog, activity: &str) -> FilterResult {
        let mut filtered = EventLog::new();
        filtered.attributes = log.attributes.clone();

        for trace in &log.traces {
            if !trace.events.iter().any(|e| e.activity == activity) {
                filtered.add_trace(trace.clone());
            }
        }

        FilterResult::new(filtered, log.len())
    }

    /// Filter by attribute value
    pub fn by_attribute(log: &EventLog, attr_key: &str, attr_value: &str) -> FilterResult {
        let mut filtered = EventLog::new();
        filtered.attributes = log.attributes.clone();

        for trace in &log.traces {
            if trace.events.iter().any(|e| {
                e.get_attribute(attr_key)
                    .map(|v| v == attr_value)
                    .unwrap_or(false)
            }) {
                filtered.add_trace(trace.clone());
            }
        }

        FilterResult::new(filtered, log.len())
    }
}

/// Filter log to keep only traces with a specific size (number of events)
pub fn filter_case_size(log: &EventLog, min_size: usize, max_size: usize) -> EventLog {
    let mut filtered = EventLog::new();
    filtered.attributes = log.attributes.clone();

    for trace in &log.traces {
        let size = trace.events.len();
        if size >= min_size && size <= max_size {
            filtered.traces.push(trace.clone());
        }
    }

    filtered
}

/// Filter log to keep only traces that start with a specific sequence of activities
pub fn filter_trace_prefix(log: &EventLog, prefix_activities: &[String]) -> EventLog {
    let mut filtered = EventLog::new();
    filtered.attributes = log.attributes.clone();

    'outer: for trace in &log.traces {
        if trace.events.len() < prefix_activities.len() {
            continue;
        }

        for (i, expected_activity) in prefix_activities.iter().enumerate() {
            if i >= trace.events.len() || trace.events[i].activity != *expected_activity {
                continue 'outer;
            }
        }

        filtered.traces.push(trace.clone());
    }

    filtered
}

/// Filter log to keep only traces that end with a specific sequence of activities
pub fn filter_trace_suffix(log: &EventLog, suffix_activities: &[String]) -> EventLog {
    let mut filtered = EventLog::new();
    filtered.attributes = log.attributes.clone();

    'outer: for trace in &log.traces {
        if trace.events.len() < suffix_activities.len() {
            continue;
        }

        let trace_len = trace.events.len();
        let suffix_len = suffix_activities.len();

        for (i, expected_activity) in suffix_activities.iter().enumerate() {
            let trace_idx = trace_len - suffix_len + i;
            if trace.events[trace_idx].activity != *expected_activity {
                continue 'outer;
            }
        }

        filtered.traces.push(trace.clone());
    }

    filtered
}

/// Filter log to keep only top K variants by frequency
pub fn filter_variants_top_k(log: &EventLog, k: usize) -> EventLog {
    use std::collections::HashMap;

    let mut variant_counts = HashMap::new();

    for trace in &log.traces {
        let variant: String = trace
            .events
            .iter()
            .map(|e| e.activity.clone())
            .collect::<Vec<_>>()
            .join(",");
        *variant_counts.entry(variant).or_insert(0) += 1;
    }

    let mut variants_by_freq: Vec<_> = variant_counts.into_iter().collect();
    variants_by_freq.sort_by(|a, b| b.1.cmp(&a.1));
    variants_by_freq.truncate(k);

    let top_variants: std::collections::HashSet<String> =
        variants_by_freq.into_iter().map(|(v, _)| v).collect();

    let mut filtered = EventLog::new();
    filtered.attributes = log.attributes.clone();

    for trace in &log.traces {
        let variant: String = trace
            .events
            .iter()
            .map(|e| e.activity.clone())
            .collect::<Vec<_>>()
            .join(",");
        if top_variants.contains(&variant) {
            filtered.traces.push(trace.clone());
        }
    }

    filtered
}

/// Filter log to keep only traces where two activities are done by different resources
pub fn filter_activity_done_different_resources(
    log: &EventLog,
    activity_a: &str,
    activity_b: &str,
) -> EventLog {
    let mut filtered = EventLog::new();
    filtered.attributes = log.attributes.clone();

    for trace in &log.traces {
        let resource_a = trace
            .events
            .iter()
            .find(|e| e.activity == activity_a)
            .and_then(|e| e.resource.clone());

        let resource_b = trace
            .events
            .iter()
            .find(|e| e.activity == activity_b)
            .and_then(|e| e.resource.clone());

        match (&resource_a, &resource_b) {
            (Some(ra), Some(rb)) if ra != rb => {
                filtered.traces.push(trace.clone());
            }
            _ => {}
        }
    }

    filtered
}

/// Filter log to keep only traces that contain rework of a specific activity
pub fn filter_activities_rework(log: &EventLog, activity: &str) -> EventLog {
    let mut filtered = EventLog::new();
    filtered.attributes = log.attributes.clone();

    for trace in &log.traces {
        let count = trace
            .events
            .iter()
            .filter(|e| e.activity == activity)
            .count();

        if count > 1 {
            filtered.traces.push(trace.clone());
        }
    }

    filtered
}

/// Get all event attribute names from the log
pub fn get_event_attributes(log: &EventLog) -> Vec<String> {
    let mut attributes = std::collections::HashSet::new();

    for trace in &log.traces {
        for event in &trace.events {
            for key in event.attributes.keys() {
                attributes.insert(key.clone());
            }
        }
    }

    let mut attrs: Vec<String> = attributes.into_iter().collect();
    attrs.sort();
    attrs
}

/// Get all unique values for a specific event attribute
pub fn get_event_attribute_values(log: &EventLog, attribute_name: &str) -> Vec<String> {
    let mut values = std::collections::HashSet::new();

    for trace in &log.traces {
        for event in &trace.events {
            if let Some(value) = event.attributes.get(attribute_name) {
                values.insert(value.clone());
            }
        }
    }

    let mut vals: Vec<String> = values.into_iter().collect();
    vals.sort();
    vals
}

/// Get all trace attribute names from the log
pub fn get_trace_attributes(log: &EventLog) -> Vec<String> {
    let mut attributes = std::collections::HashSet::new();

    for trace in &log.traces {
        for key in trace.attributes.keys() {
            attributes.insert(key.clone());
        }
    }

    let mut attrs: Vec<String> = attributes.into_iter().collect();
    attrs.sort();
    attrs
}

/// Get all unique values for a specific trace attribute
pub fn get_trace_attribute_values(log: &EventLog, attribute_name: &str) -> Vec<String> {
    let mut values = std::collections::HashSet::new();

    for trace in &log.traces {
        if let Some(value) = trace.attributes.get(attribute_name) {
            values.insert(value.clone());
        }
    }

    let mut vals: Vec<String> = values.into_iter().collect();
    vals.sort();
    vals
}

/// Filter log by trace attribute value
pub fn filter_trace_attribute(
    log: &EventLog,
    attribute_key: &str,
    attribute_value: &str,
) -> EventLog {
    let mut filtered = EventLog::new();
    filtered.attributes = log.attributes.clone();

    for trace in &log.traces {
        if let Some(value) = trace.get_attribute(attribute_key) {
            if value == attribute_value {
                filtered.add_trace(trace.clone());
            }
        }
    }

    filtered
}

/// Filter log by event attribute values (keep traces containing events with specified attribute value)
pub fn filter_event_attribute_values(
    log: &EventLog,
    attribute_key: &str,
    attribute_values: &[String],
) -> EventLog {
    let value_set: std::collections::HashSet<String> = attribute_values.iter().cloned().collect();
    let mut filtered = EventLog::new();
    filtered.attributes = log.attributes.clone();

    for trace in &log.traces {
        // Check if any event in the trace has one of the specified attribute values
        let has_matching_event = trace.events.iter().any(|event| {
            event
                .get_attribute(attribute_key)
                .map(|v| value_set.contains(v))
                .unwrap_or(false)
        });

        if has_matching_event {
            filtered.add_trace(trace.clone());
        }
    }

    filtered
}

/// Filter log by trace time range (keep traces that overlap with the time range)
pub fn filter_time_range(
    log: &EventLog,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
) -> EventLog {
    let mut filtered = EventLog::new();
    filtered.attributes = log.attributes.clone();

    for trace in &log.traces {
        // Check if trace overlaps with the time range
        let trace_start = trace.events.first().map(|e| e.timestamp);
        let trace_end = trace.events.last().map(|e| e.timestamp);

        if let (Some(ts), Some(te)) = (trace_start, trace_end) {
            // Check for overlap: trace starts before end_time AND trace ends after start_time
            if ts <= end_time && te >= start_time {
                filtered.add_trace(trace.clone());
            }
        }
    }

    filtered
}

/// Filter log by removing traces that contain a specific activity
pub fn filter_traces_containing_activity(log: &EventLog, activity: &str) -> EventLog {
    let mut filtered = EventLog::new();
    filtered.attributes = log.attributes.clone();

    for trace in &log.traces {
        let contains = trace.events.iter().any(|e| e.activity == activity);
        if !contains {
            filtered.add_trace(trace.clone());
        }
    }

    filtered
}

/// Filter log to keep only traces that contain a specific activity
pub fn filter_traces_with_activity(log: &EventLog, activity: &str) -> EventLog {
    let mut filtered = EventLog::new();
    filtered.attributes = log.attributes.clone();

    for trace in &log.traces {
        let contains = trace.events.iter().any(|e| e.activity == activity);
        if contains {
            filtered.add_trace(trace.clone());
        }
    }

    filtered
}

/// Chainable filter builder for complex filtering
pub struct FilterChain {
    log: EventLog,
    original_count: usize,
}

impl FilterChain {
    pub fn new(log: EventLog) -> Self {
        let original_count = log.len();
        Self {
            log,
            original_count,
        }
    }

    pub fn with_variant(self, activities: &[&str]) -> Self {
        let result = AdvancedFilter::by_variant(&self.log, activities);
        Self {
            log: result.log,
            original_count: self.original_count,
        }
    }

    pub fn with_time_range(self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        let result = AdvancedFilter::by_time_range(&self.log, start, end);
        Self {
            log: result.log,
            original_count: self.original_count,
        }
    }

    pub fn with_resource(self, resource: &str) -> Self {
        let result = AdvancedFilter::by_resource(&self.log, resource);
        Self {
            log: result.log,
            original_count: self.original_count,
        }
    }

    pub fn with_frequency_percentile(self, percentile: f64) -> Self {
        let result = AdvancedFilter::by_frequency_percentile(&self.log, percentile);
        Self {
            log: result.log,
            original_count: self.original_count,
        }
    }

    pub fn with_case_duration(self, min: f64, max: f64) -> Self {
        let result = AdvancedFilter::by_case_duration(&self.log, min, max);
        Self {
            log: result.log,
            original_count: self.original_count,
        }
    }

    pub fn with_min_length(self, min: usize) -> Self {
        let result = AdvancedFilter::by_min_length(&self.log, min);
        Self {
            log: result.log,
            original_count: self.original_count,
        }
    }

    pub fn with_max_length(self, max: usize) -> Self {
        let result = AdvancedFilter::by_max_length(&self.log, max);
        Self {
            log: result.log,
            original_count: self.original_count,
        }
    }

    pub fn with_start_activity(self, activity: &str) -> Self {
        let result = AdvancedFilter::by_start_activity(&self.log, activity);
        Self {
            log: result.log,
            original_count: self.original_count,
        }
    }

    pub fn with_end_activity(self, activity: &str) -> Self {
        let result = AdvancedFilter::by_end_activity(&self.log, activity);
        Self {
            log: result.log,
            original_count: self.original_count,
        }
    }

    pub fn with_containing_activity(self, activity: &str) -> Self {
        let result = AdvancedFilter::by_containing_activity(&self.log, activity);
        Self {
            log: result.log,
            original_count: self.original_count,
        }
    }

    pub fn build(self) -> FilterResult {
        FilterResult::new(self.log, self.original_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::{Event, Trace};
    use chrono::Utc;

    fn create_test_log() -> EventLog {
        let mut log = EventLog::new();
        let now = Utc::now();

        let mut trace1 = Trace::new("case_1");
        trace1.add_event(Event::new("a", now).with_resource("user1"));
        trace1.add_event(Event::new("b", now).with_resource("user2"));
        trace1.add_event(Event::new("c", now).with_resource("user1"));
        log.add_trace(trace1);

        let mut trace2 = Trace::new("case_2");
        trace2.add_event(Event::new("a", now).with_resource("user2"));
        trace2.add_event(Event::new("b", now).with_resource("user2"));
        log.add_trace(trace2);

        let mut trace3 = Trace::new("case_3");
        trace3.add_event(Event::new("a", now).with_resource("user1"));
        trace3.add_event(Event::new("b", now).with_resource("user1"));
        trace3.add_event(Event::new("c", now).with_resource("user1"));
        log.add_trace(trace3);

        log
    }

    #[test]
    fn test_filter_by_variant() {
        let log = create_test_log();
        let result = AdvancedFilter::by_variant(&log, &["a", "b", "c"]);

        assert_eq!(result.filtered_count, 2);
        assert_eq!(result.original_count, 3);
    }

    #[test]
    fn test_filter_by_resource() {
        let log = create_test_log();
        let result = AdvancedFilter::by_resource(&log, "user1");

        assert_eq!(result.filtered_count, 2);
    }

    #[test]
    fn test_filter_by_min_length() {
        let log = create_test_log();
        let result = AdvancedFilter::by_min_length(&log, 3);

        assert_eq!(result.filtered_count, 2);
    }

    #[test]
    fn test_filter_by_max_length() {
        let log = create_test_log();
        let result = AdvancedFilter::by_max_length(&log, 2);

        assert_eq!(result.filtered_count, 1);
    }

    #[test]
    fn test_filter_by_start_activity() {
        let log = create_test_log();
        let result = AdvancedFilter::by_start_activity(&log, "a");

        assert_eq!(result.filtered_count, 3);
    }

    #[test]
    fn test_filter_by_end_activity() {
        let log = create_test_log();
        let result = AdvancedFilter::by_end_activity(&log, "c");

        assert_eq!(result.filtered_count, 2);
    }

    #[test]
    fn test_filter_by_containing_activity() {
        let log = create_test_log();
        let result = AdvancedFilter::by_containing_activity(&log, "b");

        assert_eq!(result.filtered_count, 3);
    }

    #[test]
    fn test_filter_by_not_containing_activity() {
        let log = create_test_log();
        let result = AdvancedFilter::by_not_containing_activity(&log, "c");

        assert_eq!(result.filtered_count, 1);
    }

    #[test]
    fn test_filter_chain() {
        let log = create_test_log();
        let result = FilterChain::new(log)
            .with_min_length(2)
            .with_start_activity("a")
            .build();

        assert!(result.filtered_count > 0);
        assert!(result.retention_rate() <= 1.0);
    }

    #[test]
    fn test_filter_result_retention_rate() {
        let log = EventLog::new();
        let result = FilterResult::new(log, 100);

        assert_eq!(result.retention_rate(), 0.0);
    }

    #[test]
    fn test_filter_by_frequency_percentile() {
        let log = create_test_log();
        let result = AdvancedFilter::by_frequency_percentile(&log, 0.8);

        assert!(result.filtered_count > 0);
        assert!(result.filtered_count <= result.original_count);
    }
}
