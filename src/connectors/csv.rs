/// CSV connector — wraps `crate::io::csv::CSVReader` to implement `EventLogExtractor`
use super::{
    ConnectorConfig, ConnectorError, EventLogExtractor, ExtractionMetadata, ExtractionResult,
};

pub struct CsvConnector;

impl CsvConnector {
    pub fn validate_config(config: &ConnectorConfig) -> Result<(), ConnectorError> {
        if !config.params.contains_key("file_path") {
            return Err(ConnectorError::ConfigError(
                "missing required param: file_path".to_string(),
            ));
        }
        Ok(())
    }

    pub fn extract(config: &ConnectorConfig) -> Result<ExtractionResult, ConnectorError> {
        Self::validate_config(config)?;
        let start = std::time::Instant::now();

        let file_path = config.params.get("file_path").ok_or_else(|| {
            ConnectorError::ConfigError("missing required param: file_path".to_string())
        })?;

        // Build a CSVReader configured from the ConnectorConfig field mappings
        let reader = crate::io::csv::CSVReader::new()
            .with_case_column(config.field_mappings.case_id_field.clone())
            .with_activity_column(config.field_mappings.activity_field.clone())
            .with_timestamp_column(config.field_mappings.timestamp_field.clone())
            .with_resource_column(config.field_mappings.resource_field.clone());

        let log = reader
            .read(std::path::Path::new(file_path))
            .map_err(|e| ConnectorError::ExtractionError(e.to_string()))?;

        let event_count: usize = log.traces.iter().map(|t| t.events.len()).sum();
        let case_count = log.traces.len();

        let metadata = ExtractionMetadata {
            connector_name: config.name.clone(),
            source_record_count: event_count,
            extracted_event_count: event_count,
            extracted_case_count: case_count,
            extraction_time_ms: start.elapsed().as_millis(),
            warnings: vec![],
        };

        Ok(ExtractionResult { log, metadata })
    }
}

impl EventLogExtractor for CsvConnector {
    fn validate_config(config: &ConnectorConfig) -> Result<(), ConnectorError> {
        CsvConnector::validate_config(config)
    }

    fn extract(config: &ConnectorConfig) -> Result<ExtractionResult, ConnectorError> {
        CsvConnector::extract(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connectors::{ConnectorType, FieldMappings};
    use std::collections::HashMap;

    fn make_config(params: HashMap<String, String>) -> ConnectorConfig {
        ConnectorConfig {
            name: "test-csv".to_string(),
            connector_type: ConnectorType::Csv,
            params,
            field_mappings: FieldMappings::default(),
        }
    }

    #[test]
    fn validate_config_rejects_missing_file_path() {
        let config = make_config(HashMap::new());
        assert!(
            CsvConnector::validate_config(&config).is_err(),
            "must reject config without file_path"
        );
    }

    #[test]
    fn validate_config_accepts_file_path() {
        let mut params = HashMap::new();
        params.insert("file_path".to_string(), "/data/log.csv".to_string());
        let config = make_config(params);
        assert!(CsvConnector::validate_config(&config).is_ok());
    }

    #[test]
    fn connector_config_json_round_trip() {
        let config = ConnectorConfig {
            name: "my-connector".to_string(),
            connector_type: ConnectorType::Csv,
            params: [("file_path".to_string(), "/data/log.csv".to_string())]
                .into_iter()
                .collect(),
            field_mappings: FieldMappings::default(),
        };
        let json = serde_json::to_string(&config).expect("config must serialize to JSON");
        let restored: ConnectorConfig =
            serde_json::from_str(&json).expect("config must deserialize from JSON");
        assert_eq!(restored.connector_type, ConnectorType::Csv);
        assert_eq!(restored.name, "my-connector");
    }

    #[test]
    fn extract_returns_error_for_nonexistent_file() {
        let mut params = HashMap::new();
        params.insert(
            "file_path".to_string(),
            "/tmp/nonexistent_pm4py_test.csv".to_string(),
        );
        let config = make_config(params);
        let result = CsvConnector::extract(&config);
        assert!(
            result.is_err(),
            "extract must fail for non-existent file path"
        );
    }

    #[test]
    fn extract_reads_real_csv_file() {
        use crate::io::csv::CSVWriter;
        use crate::log::{Event, EventLog, Trace};
        use tempfile::NamedTempFile;

        // Create a real temp CSV file
        let mut log = EventLog::new();
        let mut trace = Trace::new("case_1");
        let now = chrono::Utc::now();
        trace.add_event(Event::new("activity_a", now));
        trace.add_event(Event::new("activity_b", now));
        log.add_trace(trace);

        let temp_file = NamedTempFile::new().expect("temp file creation must succeed");
        let writer = CSVWriter::new();
        writer
            .write(&log, temp_file.path())
            .expect("test log write must succeed");

        let mut params = HashMap::new();
        params.insert(
            "file_path".to_string(),
            temp_file
                .path()
                .to_str()
                .expect("temp file path must be valid UTF-8")
                .to_string(),
        );
        let config = make_config(params);
        let result = CsvConnector::extract(&config).expect("csv extraction must succeed");

        assert_eq!(result.metadata.extracted_case_count, 1);
        assert_eq!(result.metadata.extracted_event_count, 2);
        assert_eq!(result.log.traces.len(), 1);
    }
}
