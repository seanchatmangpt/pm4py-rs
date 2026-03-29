/// JSON format support for event logs (including OCEL format)
use crate::log::{Event, EventLog, Trace};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_json::{Map, Value};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;

/// JSON reader for flat event logs
pub struct JsonEventLogReader {
    pub case_column: String,
    pub activity_column: String,
    pub timestamp_column: String,
    pub resource_column: Option<String>,
}

impl JsonEventLogReader {
    pub fn new() -> Self {
        Self {
            case_column: "case_id".to_string(),
            activity_column: "activity".to_string(),
            timestamp_column: "timestamp".to_string(),
            resource_column: Some("resource".to_string()),
        }
    }

    pub fn with_case_column(mut self, col: impl Into<String>) -> Self {
        self.case_column = col.into();
        self
    }

    pub fn with_activity_column(mut self, col: impl Into<String>) -> Self {
        self.activity_column = col.into();
        self
    }

    pub fn with_timestamp_column(mut self, col: impl Into<String>) -> Self {
        self.timestamp_column = col.into();
        self
    }

    pub fn with_resource_column(mut self, col: Option<impl Into<String>>) -> Self {
        self.resource_column = col.map(|c| c.into());
        self
    }

    /// Read JSON event log from file
    pub fn read(&self, path: &Path) -> Result<EventLog> {
        let content = fs::read_to_string(path)?;
        self.read_from_string(&content)
    }

    /// Read JSON event log from string
    pub fn read_from_string(&self, json_str: &str) -> Result<EventLog> {
        let value: Value = serde_json::from_str(json_str)?;

        if let Value::Array(events) = value {
            return self.read_events_array(&events);
        }

        if let Value::Object(obj) = value {
            if let Some(Value::Array(events)) = obj.get("events") {
                return self.read_events_array(events);
            }
        }

        Err(anyhow::anyhow!(
            "Invalid JSON format: expected array or object with 'events' field"
        ))
    }

    fn read_events_array(&self, events: &[Value]) -> Result<EventLog> {
        let mut log = EventLog::new();
        let mut traces: BTreeMap<String, Trace> = BTreeMap::new();

        for event_value in events {
            if let Value::Object(event_obj) = event_value {
                let case_id = event_obj
                    .get(&self.case_column)
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing case column"))?
                    .to_string();

                let activity = event_obj
                    .get(&self.activity_column)
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing activity column"))?
                    .to_string();

                let timestamp_str = event_obj
                    .get(&self.timestamp_column)
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing timestamp column"))?;

                let timestamp =
                    DateTime::parse_from_rfc3339(timestamp_str).map(|dt| dt.with_timezone(&Utc))?;

                let mut event = Event::new(activity, timestamp);

                if let Some(resource_col) = &self.resource_column {
                    if let Some(Value::String(resource)) = event_obj.get(resource_col) {
                        event.resource = Some(resource.clone());
                    }
                }

                for (key, value) in event_obj {
                    if key != &self.case_column
                        && key != &self.activity_column
                        && key != &self.timestamp_column
                        && Some(key) != self.resource_column.as_ref()
                    {
                        if let Some(s) = value.as_str() {
                            event = event.with_attribute(key, s);
                        } else if let Some(n) = value.as_i64() {
                            event = event.with_attribute(key, n.to_string());
                        } else if let Some(n) = value.as_f64() {
                            event = event.with_attribute(key, n.to_string());
                        }
                    }
                }

                traces
                    .entry(case_id)
                    .or_insert_with(|| Trace::new(""))
                    .add_event(event);
            }
        }

        for (case_id, mut trace) in traces {
            trace.id = case_id;
            log.add_trace(trace);
        }

        Ok(log)
    }
}

impl Default for JsonEventLogReader {
    fn default() -> Self {
        Self::new()
    }
}

/// JSON writer for flat event logs
pub struct JsonEventLogWriter {
    pub case_column: String,
    pub activity_column: String,
    pub timestamp_column: String,
    pub resource_column: String,
    pub indent: bool,
}

impl JsonEventLogWriter {
    pub fn new() -> Self {
        Self {
            case_column: "case_id".to_string(),
            activity_column: "activity".to_string(),
            timestamp_column: "timestamp".to_string(),
            resource_column: "resource".to_string(),
            indent: true,
        }
    }

    pub fn with_indent(mut self, indent: bool) -> Self {
        self.indent = indent;
        self
    }

    /// Write event log to JSON file
    pub fn write(&self, log: &EventLog, path: &Path) -> Result<()> {
        let json_str = self.write_to_string(log)?;
        fs::write(path, json_str)?;
        Ok(())
    }

    /// Convert event log to JSON string
    pub fn write_to_string(&self, log: &EventLog) -> Result<String> {
        let mut events = vec![];

        for trace in &log.traces {
            for event in &trace.events {
                let mut event_obj = Map::new();

                event_obj.insert(self.case_column.clone(), Value::String(trace.id.clone()));
                event_obj.insert(
                    self.activity_column.clone(),
                    Value::String(event.activity.clone()),
                );
                event_obj.insert(
                    self.timestamp_column.clone(),
                    Value::String(event.timestamp.to_rfc3339()),
                );

                if let Some(resource) = &event.resource {
                    event_obj.insert(
                        self.resource_column.clone(),
                        Value::String(resource.clone()),
                    );
                }

                for (key, value) in &event.attributes {
                    event_obj.insert(key.clone(), Value::String(value.clone()));
                }

                events.push(Value::Object(event_obj));
            }
        }

        let result = if self.indent {
            serde_json::to_string_pretty(&events)?
        } else {
            serde_json::to_string(&events)?
        };

        Ok(result)
    }
}

impl Default for JsonEventLogWriter {
    fn default() -> Self {
        Self::new()
    }
}

/// OCEL (Object-Centric Event Log) reader
pub struct OcelReader;

impl Default for OcelReader {
    fn default() -> Self {
        Self::new()
    }
}

impl OcelReader {
    pub fn new() -> Self {
        Self
    }

    /// Read OCEL format from file
    pub fn read(&self, path: &Path) -> Result<(EventLog, OcelMetadata)> {
        let content = fs::read_to_string(path)?;
        self.read_from_string(&content)
    }

    /// Read OCEL format from string
    pub fn read_from_string(&self, json_str: &str) -> Result<(EventLog, OcelMetadata)> {
        let value: Value = serde_json::from_str(json_str)?;

        if let Value::Object(obj) = value {
            if let Some(Value::Object(ocel)) = obj.get("ocel") {
                return self.parse_ocel(ocel);
            }
        }

        Err(anyhow::anyhow!(
            "Invalid OCEL format: expected object with 'ocel' field"
        ))
    }

    fn parse_ocel(&self, ocel: &Map<String, Value>) -> Result<(EventLog, OcelMetadata)> {
        let mut log = EventLog::new();
        let mut metadata = OcelMetadata::new();

        if let Some(Value::Array(events)) = ocel.get("events") {
            let mut traces: BTreeMap<String, Trace> = BTreeMap::new();

            for event_value in events {
                if let Value::Object(event_obj) = event_value {
                    let event_id = event_obj
                        .get("event_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string();

                    let activity = event_obj
                        .get("activity")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| anyhow::anyhow!("Missing activity"))?
                        .to_string();

                    let timestamp_str = event_obj
                        .get("timestamp")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| anyhow::anyhow!("Missing timestamp"))?;

                    let timestamp = DateTime::parse_from_rfc3339(timestamp_str)
                        .map(|dt| dt.with_timezone(&Utc))?;

                    let mut event = Event::new(activity, timestamp);

                    if let Some(Value::Object(attrs)) = event_obj.get("attributes") {
                        for (key, value) in attrs {
                            if let Some(s) = value.as_str() {
                                event = event.with_attribute(key, s);
                            }
                        }
                    }

                    let case_id = event_obj
                        .get("omap")
                        .and_then(|v| {
                            if let Value::Array(arr) = v {
                                arr.first().and_then(|v| v.as_str()).map(|s| s.to_string())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_else(|| "case_1".to_string());

                    if let Some(Value::Array(omap)) = event_obj.get("omap") {
                        metadata.event_objects.insert(
                            event_id,
                            omap.iter()
                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                .collect(),
                        );
                    }

                    traces
                        .entry(case_id)
                        .or_insert_with(|| Trace::new(""))
                        .add_event(event);
                }
            }

            for (case_id, mut trace) in traces {
                trace.id = case_id;
                log.add_trace(trace);
            }
        }

        if let Some(Value::Array(objects)) = ocel.get("objects") {
            for obj_value in objects {
                if let Value::Object(obj_obj) = obj_value {
                    if let Some(Value::String(obj_id)) = obj_obj.get("object_id") {
                        let obj_type = obj_obj
                            .get("object_type")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string();

                        let mut attributes = HashMap::new();
                        if let Some(Value::Object(attrs)) = obj_obj.get("attributes") {
                            for (key, value) in attrs {
                                if let Some(s) = value.as_str() {
                                    attributes.insert(key.clone(), s.to_string());
                                }
                            }
                        }

                        metadata.objects.insert(
                            obj_id.clone(),
                            OcelObject {
                                id: obj_id.clone(),
                                obj_type,
                                attributes,
                            },
                        );
                    }
                }
            }
        }

        Ok((log, metadata))
    }
}

/// OCEL metadata and structure
#[derive(Debug, Clone)]
pub struct OcelObject {
    pub id: String,
    pub obj_type: String,
    pub attributes: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct OcelMetadata {
    pub objects: HashMap<String, OcelObject>,
    pub event_objects: HashMap<String, Vec<String>>,
}

impl OcelMetadata {
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
            event_objects: HashMap::new(),
        }
    }
}

impl Default for OcelMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// OCEL writer
pub struct OcelWriter;

impl OcelWriter {
    pub fn new() -> Self {
        Self
    }

    /// Write OCEL format to file
    pub fn write(&self, log: &EventLog, metadata: &OcelMetadata, path: &Path) -> Result<()> {
        let json_str = self.write_to_string(log, metadata)?;
        fs::write(path, json_str)?;
        Ok(())
    }

    /// Convert to OCEL JSON string
    pub fn write_to_string(&self, log: &EventLog, metadata: &OcelMetadata) -> Result<String> {
        let mut events = vec![];

        for trace in &log.traces {
            for (idx, event) in trace.events.iter().enumerate() {
                let event_id = format!("{}_{}", trace.id, idx);
                let omap: Vec<Value> = metadata
                    .event_objects
                    .get(&event_id)
                    .map(|objs| objs.iter().map(|o| Value::String(o.clone())).collect())
                    .unwrap_or_else(|| vec![Value::String(trace.id.clone())]);

                let mut event_obj = Map::new();
                event_obj.insert("event_id".to_string(), Value::String(event_id));
                event_obj.insert(
                    "activity".to_string(),
                    Value::String(event.activity.clone()),
                );
                event_obj.insert(
                    "timestamp".to_string(),
                    Value::String(event.timestamp.to_rfc3339()),
                );
                event_obj.insert("omap".to_string(), Value::Array(omap));

                if !event.attributes.is_empty() {
                    let attrs: Map<String, Value> = event
                        .attributes
                        .iter()
                        .map(|(k, v)| (k.clone(), Value::String(v.clone())))
                        .collect();
                    event_obj.insert("attributes".to_string(), Value::Object(attrs));
                }

                events.push(Value::Object(event_obj));
            }
        }

        let mut objects = vec![];
        for obj in metadata.objects.values() {
            let mut obj_json = Map::new();
            obj_json.insert("object_id".to_string(), Value::String(obj.id.clone()));
            obj_json.insert(
                "object_type".to_string(),
                Value::String(obj.obj_type.clone()),
            );

            let attrs: Map<String, Value> = obj
                .attributes
                .iter()
                .map(|(k, v)| (k.clone(), Value::String(v.clone())))
                .collect();
            obj_json.insert("attributes".to_string(), Value::Object(attrs));

            objects.push(Value::Object(obj_json));
        }

        let mut ocel = Map::new();
        ocel.insert("events".to_string(), Value::Array(events));
        ocel.insert("objects".to_string(), Value::Array(objects));

        let mut result_obj = Map::new();
        result_obj.insert("ocel".to_string(), Value::Object(ocel));

        let result = serde_json::to_string_pretty(&Value::Object(result_obj))?;
        Ok(result)
    }
}

impl Default for OcelWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::{Event, EventLog, Trace};
    use chrono::Utc;

    fn create_test_log() -> EventLog {
        let mut log = EventLog::new();
        let mut trace = Trace::new("case_1");
        let now = Utc::now();

        trace.add_event(Event::new("a", now).with_resource("user1"));
        trace.add_event(Event::new("b", now).with_resource("user2"));

        log.add_trace(trace);
        log
    }

    #[test]
    fn test_json_reader_new() {
        let reader = JsonEventLogReader::new();
        assert_eq!(reader.case_column, "case_id");
        assert_eq!(reader.activity_column, "activity");
    }

    #[test]
    fn test_json_writer_to_string() {
        let log = create_test_log();
        let writer = JsonEventLogWriter::new();

        let json_str = writer.write_to_string(&log).unwrap();
        assert!(!json_str.is_empty());
        assert!(json_str.contains("case_id"));
        assert!(json_str.contains("activity"));
    }

    #[test]
    fn test_json_read_write_roundtrip() {
        let log = create_test_log();
        let writer = JsonEventLogWriter::new();
        let json_str = writer.write_to_string(&log).unwrap();

        let reader = JsonEventLogReader::new();
        let log2 = reader.read_from_string(&json_str).unwrap();

        assert_eq!(log2.len(), log.len());
        assert_eq!(log2.num_events(), log.num_events());
    }

    #[test]
    fn test_ocel_writer() {
        let log = create_test_log();
        let metadata = OcelMetadata::new();
        let writer = OcelWriter::new();

        let json_str = writer.write_to_string(&log, &metadata).unwrap();
        assert!(json_str.contains("ocel"));
        assert!(json_str.contains("events"));
        assert!(json_str.contains("objects"));
    }

    #[test]
    fn test_ocel_reader() {
        let log = create_test_log();
        let metadata = OcelMetadata::new();
        let writer = OcelWriter::new();
        let json_str = writer.write_to_string(&log, &metadata).unwrap();

        let reader = OcelReader::new();
        let (log2, _metadata) = reader.read_from_string(&json_str).unwrap();

        assert_eq!(log2.len(), log.len());
        assert_eq!(log2.num_events(), log.num_events());
    }
}
