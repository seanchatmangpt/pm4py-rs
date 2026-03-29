/// Common utility functions
use crate::log::EventLog;
use std::collections::HashMap;

/// XML entity escaping for secure XML generation
///
/// Escapes XML special characters to prevent XML injection vulnerabilities.
/// Handles: <, >, &, ", '
///
/// # Example
/// ```
/// use pm4py::utils::common::escape_xml_string;
/// let escaped = escape_xml_string("Price < $100 & \"discount\"");
/// assert_eq!(escaped, "Price &lt; $100 &amp; &quot;discount&quot;");
/// ```
pub fn escape_xml_string(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            '&' => "&amp;".to_string(),
            '"' => "&quot;".to_string(),
            '\'' => "&apos;".to_string(),
            other => other.to_string(),
        })
        .collect()
}

/// Merge multiple event logs
pub fn merge_logs(logs: &[EventLog]) -> EventLog {
    let mut merged = EventLog::new();

    for log in logs {
        for trace in &log.traces {
            merged.add_trace(trace.clone());
        }

        // Merge attributes from first log
        if merged.attributes.is_empty() {
            merged.attributes = log.attributes.clone();
        }
    }

    merged
}

/// Split log by trace attribute
pub fn split_by_attribute(log: &EventLog, attribute: &str) -> HashMap<String, EventLog> {
    let mut result: HashMap<String, EventLog> = HashMap::new();

    for trace in &log.traces {
        let key = trace
            .get_attribute(attribute)
            .unwrap_or("unknown")
            .to_string();

        let entry = result.entry(key).or_default();
        entry.add_trace(trace.clone());
    }

    result
}

/// Reverse traces
pub fn reverse_traces(log: &EventLog) -> EventLog {
    let mut reversed = EventLog::new();
    reversed.attributes = log.attributes.clone();

    for trace in &log.traces {
        let mut reversed_trace = trace.clone();
        reversed_trace.events.reverse();
        reversed.add_trace(reversed_trace);
    }

    reversed
}

/// Remove outliers (traces with unusual length)
pub fn remove_outliers(log: &EventLog, std_dev_multiplier: f64) -> EventLog {
    if log.is_empty() {
        return EventLog::new();
    }

    let lengths: Vec<usize> = log.traces.iter().map(|t| t.len()).collect();
    let mean = lengths.iter().sum::<usize>() as f64 / lengths.len() as f64;

    let variance = lengths
        .iter()
        .map(|l| (*l as f64 - mean).powi(2))
        .sum::<f64>()
        / lengths.len() as f64;

    let std_dev = variance.sqrt();
    let threshold = std_dev * std_dev_multiplier;

    let mut filtered = EventLog::new();
    filtered.attributes = log.attributes.clone();

    for trace in &log.traces {
        let length_diff = (trace.len() as f64 - mean).abs();
        if length_diff <= threshold {
            filtered.add_trace(trace.clone());
        }
    }

    filtered
}

/// Sample traces randomly from the log
pub fn sample_traces_random(log: &EventLog, n: usize) -> EventLog {
    let mut sampled = EventLog::new();
    sampled.attributes = log.attributes.clone();

    let total = log.len();
    if total <= n {
        return log.clone();
    }

    // Simple deterministic sampling (take every Nth trace)
    let step = total / n;
    for (i, trace) in log.traces.iter().enumerate() {
        if i % step == 0 && sampled.len() < n {
            sampled.add_trace(trace.clone());
        }
    }

    sampled
}

/// Concatenate two logs
pub fn concatenate_logs(log1: &EventLog, log2: &EventLog) -> EventLog {
    let mut result = log1.clone();
    for trace in &log2.traces {
        result.add_trace(trace.clone());
    }
    result
}

/// Get log summary statistics
pub fn log_summary(log: &EventLog) -> LogSummary {
    LogSummary {
        num_traces: log.len(),
        num_events: log.num_events(),
        num_activities: log.activities().len(),
        min_trace_length: log.traces.iter().map(|t| t.len()).min().unwrap_or(0),
        max_trace_length: log.traces.iter().map(|t| t.len()).max().unwrap_or(0),
        avg_trace_length: if log.is_empty() {
            0.0
        } else {
            log.num_events() as f64 / log.len() as f64
        },
    }
}

/// Log summary statistics
#[derive(Debug, Clone)]
pub struct LogSummary {
    pub num_traces: usize,
    pub num_events: usize,
    pub num_activities: usize,
    pub min_trace_length: usize,
    pub max_trace_length: usize,
    pub avg_trace_length: f64,
}

/// Apply a function to each trace in the log
pub fn transform_traces<F>(log: &EventLog, f: F) -> EventLog
where
    F: Fn(&crate::log::Trace) -> crate::log::Trace,
{
    let mut transformed = EventLog::new();
    transformed.attributes = log.attributes.clone();

    for trace in &log.traces {
        transformed.add_trace(f(trace));
    }

    transformed
}

/// Filter traces by a predicate
pub fn filter_traces<F>(log: &EventLog, predicate: F) -> EventLog
where
    F: Fn(&crate::log::Trace) -> bool,
{
    let mut filtered = EventLog::new();
    filtered.attributes = log.attributes.clone();

    for trace in &log.traces {
        if predicate(trace) {
            filtered.add_trace(trace.clone());
        }
    }

    filtered
}

/// Sort traces by a key function
pub fn sort_traces<F, K>(log: &EventLog, key_fn: F) -> EventLog
where
    F: Fn(&crate::log::Trace) -> K,
    K: Ord,
{
    let mut sorted = log.clone();
    sorted.traces.sort_by_key(|t| key_fn(t));
    sorted
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::{Event, Trace};
    use chrono::Utc;

    fn create_test_log(id: usize) -> EventLog {
        let mut log = EventLog::new();
        let now = Utc::now();

        let mut trace = Trace::new(format!("case_{}", id));
        trace.add_event(Event::new("a", now));
        log.add_trace(trace);

        log
    }

    #[test]
    fn test_merge_logs() {
        let log1 = create_test_log(1);
        let log2 = create_test_log(2);

        let merged = merge_logs(&[log1, log2]);

        assert_eq!(merged.len(), 2);
    }

    #[test]
    fn test_reverse_traces() {
        let mut log = EventLog::new();
        let now = Utc::now();

        let mut trace = Trace::new("case_1");
        trace.add_event(Event::new("a", now));
        trace.add_event(Event::new("b", now));
        log.add_trace(trace);

        let reversed = reverse_traces(&log);

        assert_eq!(reversed.traces[0].events[0].activity, "b");
    }

    #[test]
    fn test_escape_xml_string_basic() {
        assert_eq!(escape_xml_string("hello"), "hello");
        assert_eq!(escape_xml_string("<"), "&lt;");
        assert_eq!(escape_xml_string(">"), "&gt;");
        assert_eq!(escape_xml_string("&"), "&amp;");
        assert_eq!(escape_xml_string("\""), "&quot;");
        assert_eq!(escape_xml_string("'"), "&apos;");
    }

    #[test]
    fn test_escape_xml_string_complex() {
        assert_eq!(
            escape_xml_string("Price < $100 & \"discount\""),
            "Price &lt; $100 &amp; &quot;discount&quot;"
        );
        assert_eq!(
            escape_xml_string("<script>alert('xss')</script>"),
            "&lt;script&gt;alert(&apos;xss&apos;)&lt;/script&gt;"
        );
    }

    #[test]
    fn test_escape_xml_string_mixed() {
        let input = "Activity: <Purchase> & [Confirm] price=\"$50\"";
        let expected = "Activity: &lt;Purchase&gt; &amp; [Confirm] price=&quot;$50&quot;";
        assert_eq!(escape_xml_string(input), expected);
    }
}
