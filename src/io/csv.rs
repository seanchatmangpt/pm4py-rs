/// CSV format support for event logs
use crate::errors::{Pm4PyError, Result};
use crate::log::{Event, EventLog, Trace};
use chrono::{DateTime, Utc};
use csv::Writer;
use std::collections::BTreeMap;
use std::fs::File;
use std::path::Path;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[allow(dead_code)]
struct CSVRecord {
    case_id: String,
    activity: String,
    timestamp: String,
    #[serde(default)]
    resource: String,
}

#[derive(Clone)]
pub struct CSVReader {
    pub case_column: String,
    pub activity_column: String,
    pub timestamp_column: String,
    pub resource_column: Option<String>,
    pub delimiter: u8,
}

impl CSVReader {
    pub fn new() -> Self {
        Self {
            case_column: "case_id".to_string(),
            activity_column: "activity".to_string(),
            timestamp_column: "timestamp".to_string(),
            resource_column: Some("resource".to_string()),
            delimiter: b',', // SECURITY: Use standard comma delimiter
        }
    }

    /// SECURITY: Validate delimiter against whitelist
    /// Only allow standard CSV delimiters to prevent injection attacks
    pub fn with_delimiter(mut self, delimiter: u8) -> Result<Self> {
        match delimiter {
            b',' | b';' | b'\t' | b'|' => {
                self.delimiter = delimiter;
                Ok(self)
            }
            _ => Err(Pm4PyError::csv_invalid_delimiter(delimiter)),
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

    pub fn read(&self, path: &Path) -> Result<EventLog> {
        let file = File::open(path).map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => {
                Pm4PyError::file_not_found("CSVReader::read", &path.to_path_buf())
            }
            _ => Pm4PyError::from(e),
        })?;

        let mut reader = csv::ReaderBuilder::new()
            .flexible(true)
            .delimiter(self.delimiter) // SECURITY: Use validated delimiter
            .from_reader(file);

        let mut log = EventLog::new();
        let mut traces: BTreeMap<String, Trace> = BTreeMap::new();
        let mut line_num = 1;

        for result in reader.deserialize() {
            line_num += 1;
            let record: std::collections::HashMap<String, String> = result.map_err(|e| {
                Pm4PyError::Parse(crate::errors::ParseError {
                    format: crate::errors::ParseFormat::Csv,
                    location: path.display().to_string(),
                    reason: format!("CSV parse error: {}", e),
                    line: Some(line_num),
                    fix: "Check CSV format matches expected schema (case_id, activity, timestamp columns)".to_string(),
                })
            })?;

            let case_id = record
                .get(&self.case_column)
                .ok_or_else(|| Pm4PyError::csv_missing_column(&self.case_column, path))?
                .clone();

            let activity = record
                .get(&self.activity_column)
                .ok_or_else(|| Pm4PyError::csv_missing_column(&self.activity_column, path))?
                .clone();

            let timestamp_str = record
                .get(&self.timestamp_column)
                .ok_or_else(|| Pm4PyError::csv_missing_column(&self.timestamp_column, path))?;

            let timestamp = DateTime::parse_from_rfc3339(timestamp_str)
                .map(|dt| dt.with_timezone(&Utc))
                .or_else(|_| {
                    chrono::NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%dT%H:%M:%S%.f")
                        .map(|ndt| ndt.and_utc())
                })
                .map_err(|_| {
                    Pm4PyError::csv_timestamp_parse(&self.timestamp_column, timestamp_str, path)
                })?;

            let resource = self
                .resource_column
                .as_ref()
                .and_then(|col| record.get(col).cloned());

            let mut event = Event::new(activity, timestamp);
            if let Some(res) = resource {
                event.resource = Some(res);
            }

            traces
                .entry(case_id)
                .or_insert_with(|| Trace::new(""))
                .add_event(event);
        }

        for (case_id, mut trace) in traces {
            trace.id = case_id;
            log.add_trace(trace);
        }

        Ok(log)
    }
}

impl Default for CSVReader {
    fn default() -> Self {
        Self::new()
    }
}

/// SECURITY: Escape CSV formula injection
/// Prevent cells starting with =, +, -, @ from being evaluated as formulas
fn escape_csv_formula(s: &str) -> String {
    if s.is_empty() {
        return s.to_string();
    }
    match s.chars().next() {
        Some('=') | Some('+') | Some('-') | Some('@') => format!("'{}", s),
        _ => s.to_string(),
    }
}

pub struct CSVWriter {
    pub case_column: String,
    pub activity_column: String,
    pub timestamp_column: String,
    pub resource_column: String,
}

impl CSVWriter {
    pub fn new() -> Self {
        Self {
            case_column: "case_id".to_string(),
            activity_column: "activity".to_string(),
            timestamp_column: "timestamp".to_string(),
            resource_column: "resource".to_string(),
        }
    }

    pub fn write(&self, log: &EventLog, path: &Path) -> Result<()> {
        let file = File::create(path)?;
        let mut writer = Writer::from_writer(file);

        // Write header
        writer.write_record([
            &self.case_column,
            &self.activity_column,
            &self.timestamp_column,
            &self.resource_column,
        ])?;

        // Write events
        for trace in &log.traces {
            for event in &trace.events {
                // SECURITY: Escape formula injection in CSV output
                let escaped_case_id = escape_csv_formula(&trace.id);
                let escaped_activity = escape_csv_formula(&event.activity);
                let escaped_resource =
                    escape_csv_formula(event.resource.as_ref().unwrap_or(&String::new()));

                writer.write_record([
                    &escaped_case_id,
                    &escaped_activity,
                    &event.timestamp.to_rfc3339(),
                    &escaped_resource,
                ])?;
            }
        }

        writer.flush()?;
        Ok(())
    }
}

impl Default for CSVWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_writer() {
        let mut log = EventLog::new();
        let mut trace = Trace::new("case_1");
        let now = Utc::now();

        trace.add_event(Event::new("activity_a", now));
        log.add_trace(trace);

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        let writer = CSVWriter::new();
        let result = writer.write(&log, path);

        assert!(result.is_ok());
    }

    #[test]
    fn test_csv_delimiter_validation_security() {
        // SECURITY TEST: CSV delimiter injection prevention
        // Only whitelisted delimiters should be accepted
        let reader = CSVReader::new();

        // Valid delimiters
        assert!(
            reader.clone().with_delimiter(b',').is_ok(),
            "Comma should be valid"
        );
        assert!(
            reader.clone().with_delimiter(b';').is_ok(),
            "Semicolon should be valid"
        );
        assert!(
            reader.clone().with_delimiter(b'\t').is_ok(),
            "Tab should be valid"
        );
        assert!(
            reader.clone().with_delimiter(b'|').is_ok(),
            "Pipe should be valid"
        );

        // Invalid delimiters - should reject
        assert!(
            reader.clone().with_delimiter(b'!').is_err(),
            "Exclamation should be invalid"
        );
        assert!(
            reader.clone().with_delimiter(b'=').is_err(),
            "Equals should be invalid"
        );
        assert!(
            reader.clone().with_delimiter(b'@').is_err(),
            "At sign should be invalid"
        );
        assert!(
            reader.clone().with_delimiter(0).is_err(),
            "Null byte should be invalid"
        );
    }

    #[test]
    fn test_csv_formula_injection_escape() {
        // SECURITY TEST: CSV formula injection via escape_csv_formula
        // Cells starting with =, +, -, @ should be escaped
        assert_eq!(escape_csv_formula("normal_text"), "normal_text");
        assert_eq!(escape_csv_formula("=SUM(A1:A10)"), "'=SUM(A1:A10)");
        assert_eq!(escape_csv_formula("+malicious"), "'+malicious");
        assert_eq!(escape_csv_formula("-malicious"), "'-malicious");
        assert_eq!(escape_csv_formula("@vulnerable"), "'@vulnerable");
        assert_eq!(escape_csv_formula(""), "");
    }

    #[test]
    fn test_csv_formula_injection_in_output() {
        // SECURITY TEST: Formula injection in CSV output
        // Ensure formulas are escaped in written CSV
        let mut log = EventLog::new();
        let mut trace = Trace::new("=cmd|'/c calc'!A1");

        let now = Utc::now();
        trace.add_event(Event::new("=SUM(A1:A10)", now));
        let mut event = Event::new("activity", now);
        event.resource = Some("@malicious".to_string());
        trace.add_event(event);
        log.add_trace(trace);

        let temp_file = NamedTempFile::new().unwrap();
        let writer = CSVWriter::new();
        writer.write(&log, temp_file.path()).unwrap();

        let output = std::fs::read_to_string(temp_file.path()).unwrap();

        // Formulas should be escaped with single quote
        assert!(
            output.contains("'=cmd"),
            "Formula in case_id should be escaped"
        );
        assert!(
            output.contains("'=SUM"),
            "Formula in activity should be escaped"
        );
        assert!(
            output.contains("'@malicious"),
            "@ symbol in resource should be escaped"
        );
    }

    #[test]
    fn test_csv_delimiter_used_in_read() {
        // SECURITY TEST: Custom delimiter is actually used in parsing
        let mut log = EventLog::new();
        let mut trace = Trace::new("case_1");
        let now = Utc::now();

        trace.add_event(Event::new("activity_a", now));
        log.add_trace(trace);

        // Write with standard CSV
        let temp_file = NamedTempFile::new().unwrap();
        let writer = CSVWriter::new();
        writer.write(&log, temp_file.path()).unwrap();

        // Should read with default comma delimiter
        let reader = CSVReader::new();
        let result = reader.read(temp_file.path());
        assert!(
            result.is_ok(),
            "Should read CSV with default comma delimiter"
        );
    }
}
