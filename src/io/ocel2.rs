//! Streaming JSON Format Support
//!
//! Provides memory-efficient streaming JSON processing for large event logs.
//! This module allows reading and writing JSON files incrementally without
//! loading the entire file into memory.
//!
//! # Examples
//!
//! ```ignore
//! use pm4py::io::ocel2::Ocel2Reader;
//! use pm4py::io::LogReader;
//! use std::path::Path;
//!
//! let reader = Ocel2Reader::new();
//! let log = reader.read(Path::new("large_events.jsonl")).unwrap();
//! ```

use crate::log::{Event, EventLog, Trace};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

/// Streaming JSON reader for processing large event logs
pub struct Ocel2Reader {
    /// Case ID column name
    pub case_column: String,
    /// Activity column name
    pub activity_column: String,
    /// Timestamp column name
    pub timestamp_column: String,
    /// Resource column name (optional)
    pub resource_column: Option<String>,
    /// Buffer size for reading
    pub buffer_size: usize,
}

impl Ocel2Reader {
    /// Create a new streaming JSON reader
    pub fn new() -> Self {
        Self {
            case_column: "case_id".to_string(),
            activity_column: "activity".to_string(),
            timestamp_column: "timestamp".to_string(),
            resource_column: Some("resource".to_string()),
            buffer_size: 8192,
        }
    }

    /// Set case column name
    pub fn with_case_column(mut self, col: impl Into<String>) -> Self {
        self.case_column = col.into();
        self
    }

    /// Set activity column name
    pub fn with_activity_column(mut self, col: impl Into<String>) -> Self {
        self.activity_column = col.into();
        self
    }

    /// Set timestamp column name
    pub fn with_timestamp_column(mut self, col: impl Into<String>) -> Self {
        self.timestamp_column = col.into();
        self
    }

    /// Set resource column name
    pub fn with_resource_column(mut self, col: Option<impl Into<String>>) -> Self {
        self.resource_column = col.map(|c| c.into());
        self
    }

    /// Set buffer size for reading (in bytes)
    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// Read streaming JSON (OCEL 2.0 JSON format - one JSON object per line)
    pub fn read(&self, path: &Path) -> Result<EventLog> {
        let file = File::open(path)?;
        let reader = BufReader::with_capacity(self.buffer_size, file);

        let mut log = EventLog::new();
        let mut traces: BTreeMap<String, Trace> = BTreeMap::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;

            // Skip empty lines and comments
            if line.trim().is_empty() || line.trim_start().starts_with('#') {
                continue;
            }

            match serde_json::from_str::<Value>(&line) {
                Ok(json) => {
                    // Extract required fields
                    let case_id = json[&self.case_column]
                        .as_str()
                        .ok_or_else(|| {
                            anyhow!(
                                "Line {}: Missing case column '{}'",
                                line_num,
                                self.case_column
                            )
                        })?
                        .to_string();

                    let activity = json[&self.activity_column]
                        .as_str()
                        .ok_or_else(|| {
                            anyhow!(
                                "Line {}: Missing activity column '{}'",
                                line_num,
                                self.activity_column
                            )
                        })?
                        .to_string();

                    let timestamp_str = json[&self.timestamp_column].as_str().ok_or_else(|| {
                        anyhow!(
                            "Line {}: Missing timestamp column '{}'",
                            line_num,
                            self.timestamp_column
                        )
                    })?;

                    // Parse timestamp
                    let timestamp = DateTime::parse_from_rfc3339(timestamp_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .or_else(|_| {
                            DateTime::parse_from_rfc2822(timestamp_str)
                                .map(|dt| dt.with_timezone(&Utc))
                        })
                        .map_err(|_| anyhow!("Line {}: Invalid timestamp format", line_num))?;

                    // Create event
                    let mut event = Event::new(activity, timestamp);

                    // Add resource if present
                    if let Some(resource_col) = &self.resource_column {
                        if let Some(resource) = json[resource_col].as_str() {
                            event = event.with_resource(resource);
                        }
                    }

                    // Add all other fields as attributes
                    if let Some(obj) = json.as_object() {
                        for (key, value) in obj {
                            if key != &self.case_column
                                && key != &self.activity_column
                                && key != &self.timestamp_column
                                && Some(key) != self.resource_column.as_ref()
                            {
                                if let Some(val_str) = value.as_str() {
                                    event = event.with_attribute(key, val_str);
                                }
                            }
                        }
                    }

                    // Add event to trace
                    traces
                        .entry(case_id.clone())
                        .or_insert_with(|| Trace::new(case_id.clone()))
                        .add_event(event);
                }
                Err(e) => {
                    return Err(anyhow!("Line {}: JSON parse error: {}", line_num, e));
                }
            }
        }

        // Convert traces to log
        for (_case_id, trace) in traces {
            log.add_trace(trace);
        }

        Ok(log)
    }
}

impl Default for Ocel2Reader {
    fn default() -> Self {
        Self::new()
    }
}

/// Streaming JSON writer for memory-efficient output
pub struct StreamingJsonWriter {
    /// Case column name
    pub case_column: String,
    /// Activity column name
    pub activity_column: String,
    /// Timestamp column name
    pub timestamp_column: String,
    /// Resource column name (optional)
    pub resource_column: Option<String>,
    /// Pretty print each line
    pub pretty: bool,
}

impl StreamingJsonWriter {
    /// Create a new streaming JSON writer
    pub fn new() -> Self {
        Self {
            case_column: "case_id".to_string(),
            activity_column: "activity".to_string(),
            timestamp_column: "timestamp".to_string(),
            resource_column: Some("resource".to_string()),
            pretty: false,
        }
    }

    /// Set case column name
    pub fn with_case_column(mut self, col: impl Into<String>) -> Self {
        self.case_column = col.into();
        self
    }

    /// Set activity column name
    pub fn with_activity_column(mut self, col: impl Into<String>) -> Self {
        self.activity_column = col.into();
        self
    }

    /// Set timestamp column name
    pub fn with_timestamp_column(mut self, col: impl Into<String>) -> Self {
        self.timestamp_column = col.into();
        self
    }

    /// Set resource column name
    pub fn with_resource_column(mut self, col: Option<impl Into<String>>) -> Self {
        self.resource_column = col.map(|c| c.into());
        self
    }

    /// Enable/disable pretty printing
    pub fn with_pretty(mut self, pretty: bool) -> Self {
        self.pretty = pretty;
        self
    }

    /// Validate JSON event format
    pub fn validate_event_format(&self, json: &Value) -> Result<()> {
        if !json.is_object() {
            return Err(anyhow!("Event must be a JSON object"));
        }

        if json[&self.case_column].as_str().is_none() {
            return Err(anyhow!("Missing case column '{}'", self.case_column));
        }

        if json[&self.activity_column].as_str().is_none() {
            return Err(anyhow!(
                "Missing activity column '{}'",
                self.activity_column
            ));
        }

        if json[&self.timestamp_column].as_str().is_none() {
            return Err(anyhow!(
                "Missing timestamp column '{}'",
                self.timestamp_column
            ));
        }

        Ok(())
    }

    /// Write EventLog as streaming JSON (OCEL 2.0 JSON format)
    pub fn write(&self, log: &EventLog, path: &Path) -> Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;

        for trace in &log.traces {
            for event in &trace.events {
                let mut event_obj = serde_json::Map::new();

                // Add required fields
                event_obj.insert(self.case_column.clone(), Value::String(trace.id.clone()));
                event_obj.insert(
                    self.activity_column.clone(),
                    Value::String(event.activity.clone()),
                );
                event_obj.insert(
                    self.timestamp_column.clone(),
                    Value::String(event.timestamp.to_rfc3339()),
                );

                // Add resource if present
                if let Some(resource_col) = &self.resource_column {
                    if let Some(resource) = &event.resource {
                        event_obj.insert(resource_col.clone(), Value::String(resource.clone()));
                    }
                }

                // Add attributes
                for (key, value) in &event.attributes {
                    event_obj.insert(key.clone(), Value::String(value.clone()));
                }

                // Write as single line (OCEL 2.0 JSON format)
                let event_json = Value::Object(event_obj);
                let line = if self.pretty {
                    serde_json::to_string_pretty(&event_json)?
                } else {
                    serde_json::to_string(&event_json)?
                };

                writeln!(file, "{}", line)?;
            }
        }

        Ok(())
    }
}

impl Default for StreamingJsonWriter {
    fn default() -> Self {
        Self::new()
    }
}

use crate::io::{LogReader, LogWriter};

impl LogReader for Ocel2Reader {
    fn read(&self, path: &Path) -> Result<EventLog> {
        Self::read(self, path)
    }
}

impl LogWriter for StreamingJsonWriter {
    fn write(&self, log: &EventLog, path: &Path) -> Result<()> {
        Self::write(self, log, path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use tempfile::NamedTempFile;

    #[test]
    fn test_streaming_reader_creation() {
        let reader = Ocel2Reader::new();
        assert_eq!(reader.case_column, "case_id");
        assert_eq!(reader.activity_column, "activity");
        assert_eq!(reader.timestamp_column, "timestamp");
    }

    #[test]
    fn test_streaming_writer_creation() {
        let writer = StreamingJsonWriter::new();
        assert_eq!(writer.case_column, "case_id");
        assert_eq!(writer.activity_column, "activity");
    }

    #[test]
    fn test_ocel2_round_trip() {
        let mut log = EventLog::new();
        let mut trace = Trace::new("case_1");

        let now = Utc::now();
        trace.add_event(Event::new("start", now).with_resource("user_1"));
        trace.add_event(Event::new("process", now).with_resource("user_2"));

        log.add_trace(trace);

        let writer = StreamingJsonWriter::new();
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        assert!(writer.write(&log, path).is_ok());

        let reader = Ocel2Reader::new();
        let loaded_log = reader.read(path).unwrap();

        assert_eq!(log.len(), loaded_log.len());
        assert_eq!(log.num_events(), loaded_log.num_events());
    }

    #[test]
    fn test_streaming_reader_with_custom_columns() {
        let reader = Ocel2Reader::new()
            .with_case_column("trace_id")
            .with_activity_column("action")
            .with_timestamp_column("ts");

        assert_eq!(reader.case_column, "trace_id");
        assert_eq!(reader.activity_column, "action");
        assert_eq!(reader.timestamp_column, "ts");
    }

    #[test]
    fn test_streaming_writer_with_custom_columns() {
        let writer = StreamingJsonWriter::new()
            .with_case_column("trace_id")
            .with_activity_column("action");

        assert_eq!(writer.case_column, "trace_id");
        assert_eq!(writer.activity_column, "action");
    }

    #[test]
    fn test_ocel2_validation_valid_event() {
        let writer = StreamingJsonWriter::new();
        let event_json = serde_json::json!({
            "case_id": "case_1",
            "activity": "start",
            "timestamp": "2024-01-01T00:00:00Z",
            "resource": "user_1"
        });

        assert!(writer.validate_event_format(&event_json).is_ok());
    }

    #[test]
    fn test_ocel2_validation_missing_field() {
        let writer = StreamingJsonWriter::new();
        let event_json = serde_json::json!({
            "case_id": "case_1",
            "activity": "start"
        });

        assert!(writer.validate_event_format(&event_json).is_err());
    }

    #[test]
    fn test_ocel2_validation_non_object() {
        let writer = StreamingJsonWriter::new();
        let event_json = serde_json::json!([1, 2, 3]);

        assert!(writer.validate_event_format(&event_json).is_err());
    }

    #[test]
    fn test_streaming_reader_builder_pattern() {
        let reader = Ocel2Reader::new()
            .with_buffer_size(16384)
            .with_resource_column(Some("executor"));

        assert_eq!(reader.buffer_size, 16384);
        assert_eq!(reader.resource_column, Some("executor".to_string()));
    }
}
