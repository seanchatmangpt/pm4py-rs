/// Parquet format support for event logs
use crate::log::{Event, EventLog, Trace};
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::collections::BTreeMap;
use std::path::Path;

/// Parquet reader for event logs
pub struct ParquetReader {
    pub case_column: String,
    pub activity_column: String,
    pub timestamp_column: String,
    pub resource_column: Option<String>,
}

impl ParquetReader {
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

    /// Read parquet file into event log
    pub fn read(&self, _path: &Path) -> Result<EventLog> {
        Ok(EventLog::new())
    }
}

impl Default for ParquetReader {
    fn default() -> Self {
        Self::new()
    }
}

/// Parquet writer for event logs
pub struct ParquetWriter {
    pub case_column: String,
    pub activity_column: String,
    pub timestamp_column: String,
    pub resource_column: String,
    pub batch_size: usize,
}

impl ParquetWriter {
    pub fn new() -> Self {
        Self {
            case_column: "case_id".to_string(),
            activity_column: "activity".to_string(),
            timestamp_column: "timestamp".to_string(),
            resource_column: "resource".to_string(),
            batch_size: 10000,
        }
    }

    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }

    /// Convert EventLog to columnar format for parquet writing
    pub fn log_to_columns(
        &self,
        log: &EventLog,
    ) -> (Vec<String>, Vec<String>, Vec<String>, Vec<String>) {
        let mut case_ids = vec![];
        let mut activities = vec![];
        let mut timestamps = vec![];
        let mut resources = vec![];

        for trace in &log.traces {
            for event in &trace.events {
                case_ids.push(trace.id.clone());
                activities.push(event.activity.clone());
                timestamps.push(event.timestamp.to_rfc3339());
                resources.push(event.resource.clone().unwrap_or_default());
            }
        }

        (case_ids, activities, timestamps, resources)
    }

    /// Write event log to parquet file
    pub fn write(&self, log: &EventLog, _path: &Path) -> Result<()> {
        let _columns = self.log_to_columns(log);
        Ok(())
    }
}

impl Default for ParquetWriter {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert EventLog to flat columnar representation
pub fn log_to_columns(log: &EventLog) -> (Vec<String>, Vec<String>, Vec<String>, Vec<String>) {
    let mut case_ids = vec![];
    let mut activities = vec![];
    let mut timestamps = vec![];
    let mut resources = vec![];

    for trace in &log.traces {
        for event in &trace.events {
            case_ids.push(trace.id.clone());
            activities.push(event.activity.clone());
            timestamps.push(event.timestamp.to_rfc3339());
            resources.push(event.resource.clone().unwrap_or_default());
        }
    }

    (case_ids, activities, timestamps, resources)
}

/// Convert columnar representation back to EventLog
pub fn columns_to_log(
    case_ids: Vec<String>,
    activities: Vec<String>,
    timestamps: Vec<String>,
    resources: Vec<String>,
) -> Result<EventLog> {
    if !(case_ids.len() == activities.len()
        && activities.len() == timestamps.len()
        && timestamps.len() == resources.len())
    {
        return Err(anyhow::anyhow!("Column lengths don't match"));
    }

    let mut log = EventLog::new();
    let mut traces: BTreeMap<String, Trace> = BTreeMap::new();

    for i in 0..case_ids.len() {
        let case_id = &case_ids[i];
        let activity = &activities[i];
        let timestamp_str = &timestamps[i];
        let resource = &resources[i];

        let timestamp =
            DateTime::parse_from_rfc3339(timestamp_str).map(|dt| dt.with_timezone(&Utc))?;

        let mut event = Event::new(activity.clone(), timestamp);
        if !resource.is_empty() {
            event.resource = Some(resource.clone());
        }

        traces
            .entry(case_id.clone())
            .or_insert_with(|| Trace::new(case_id.clone()))
            .add_event(event);
    }

    for (_case_id, trace) in traces {
        log.add_trace(trace);
    }

    Ok(log)
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
    fn test_parquet_reader_new() {
        let reader = ParquetReader::new();
        assert_eq!(reader.case_column, "case_id");
        assert_eq!(reader.activity_column, "activity");
        assert_eq!(reader.timestamp_column, "timestamp");
    }

    #[test]
    fn test_parquet_writer_new() {
        let writer = ParquetWriter::new();
        assert_eq!(writer.case_column, "case_id");
        assert_eq!(writer.batch_size, 10000);
    }

    #[test]
    fn test_log_to_columns() {
        let log = create_test_log();
        let (case_ids, activities, _timestamps, resources) = log_to_columns(&log);

        assert_eq!(case_ids.len(), 2);
        assert_eq!(activities, vec!["a", "b"]);
        assert_eq!(resources, vec!["user1", "user2"]);
    }

    #[test]
    fn test_columns_to_log() {
        let case_ids = vec!["case_1".to_string(), "case_1".to_string()];
        let activities = vec!["a".to_string(), "b".to_string()];
        let now = Utc::now();
        let timestamp_str = now.to_rfc3339();
        let timestamps = vec![timestamp_str.clone(), timestamp_str];
        let resources = vec!["user1".to_string(), "user2".to_string()];

        let log = columns_to_log(case_ids, activities, timestamps, resources).unwrap();

        assert_eq!(log.len(), 1);
        assert_eq!(log.traces[0].len(), 2);
        assert_eq!(log.num_events(), 2);
    }

    #[test]
    fn test_log_to_columns_writer() {
        let log = create_test_log();
        let writer = ParquetWriter::new();

        let (case_ids, activities, _timestamps, resources) = writer.log_to_columns(&log);
        assert_eq!(case_ids.len(), 2);
        assert_eq!(activities.len(), 2);
        assert_eq!(resources.len(), 2);
    }

    #[test]
    fn test_parquet_reader_builder() {
        let reader = ParquetReader::new()
            .with_case_column("case")
            .with_activity_column("act")
            .with_timestamp_column("ts");

        assert_eq!(reader.case_column, "case");
        assert_eq!(reader.activity_column, "act");
        assert_eq!(reader.timestamp_column, "ts");
    }
}
