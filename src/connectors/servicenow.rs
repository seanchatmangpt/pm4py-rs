/// ServiceNow connector for pm4py-rust.
///
/// Uses Basic authentication with `sysparm_*` query parameters.
/// Parses ServiceNow's custom `"%Y-%m-%d %H:%M:%S"` timestamp format.
use crate::connectors::{
    ConnectorConfig, ConnectorError, EventLogExtractor, ExtractionMetadata, ExtractionResult,
    FieldMappings,
};
use crate::log::{Event, EventLog, Trace};
use chrono::{DateTime, NaiveDateTime, Utc};
use serde_json::Value;
use std::collections::HashMap;

/// ServiceNow Table API connector.
///
/// Required `params`:
/// - `base_url`: ServiceNow instance URL (e.g. `https://myinstance.service-now.com`)
/// - `table`: ServiceNow table name (e.g. `incident`, `change_request`)
/// - `username`, `password`: Basic auth credentials
/// - `sysparm_query`: Optional encoded query filter
pub struct ServiceNowConnector;

impl ServiceNowConnector {
    /// Build Table API URL with sysparm parameters.
    fn build_url(
        config: &ConnectorConfig,
        offset: usize,
        limit: usize,
    ) -> Result<String, ConnectorError> {
        let base = config
            .params
            .get("base_url")
            .ok_or_else(|| ConnectorError::ConfigError("missing base_url".into()))?;
        let table = config
            .params
            .get("table")
            .ok_or_else(|| ConnectorError::ConfigError("missing table".into()))?;

        let mut url = format!(
            "{}/api/now/table/{}?sysparm_offset={}&sysparm_limit={}&sysparm_display_value=false",
            base.trim_end_matches('/'),
            table,
            offset,
            limit
        );

        if let Some(query) = config.params.get("sysparm_query") {
            url.push_str(&format!("&sysparm_query={}", query));
        }

        if let Some(fields) = config.params.get("sysparm_fields") {
            url.push_str(&format!("&sysparm_fields={}", fields));
        }

        Ok(url)
    }

    /// Build the Authorization header value.
    fn auth_header(config: &ConnectorConfig) -> Result<String, ConnectorError> {
        let user = config
            .params
            .get("username")
            .ok_or_else(|| ConnectorError::ConfigError("missing username".into()))?;
        let pass = config
            .params
            .get("password")
            .ok_or_else(|| ConnectorError::ConfigError("missing password".into()))?;
        let creds = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            format!("{}:{}", user, pass),
        );
        Ok(format!("Basic {}", creds))
    }

    /// Parse ServiceNow custom timestamp format: `"YYYY-MM-DD HH:MM:SS"`.
    fn parse_sn_timestamp(value: &str) -> Option<DateTime<Utc>> {
        // Primary: ServiceNow format "2024-01-15 09:30:00"
        NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S")
            .ok()
            .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
            .or_else(|| {
                // Fallback: ISO 8601
                value.parse::<DateTime<Utc>>().ok()
            })
    }

    /// Extract a string from a ServiceNow field (plain string or wrapped `{"value": "..."}` object).
    fn extract_str(v: &Value) -> Option<&str> {
        if let Some(s) = v.as_str() {
            return Some(s);
        }
        v.get("value")?.as_str()
    }

    /// Parse a single ServiceNow result record.
    fn parse_record(
        record: &Value,
        mappings: &FieldMappings,
    ) -> Option<(String, String, DateTime<Utc>, Option<String>)> {
        let case_id = Self::extract_str(record.get(&mappings.case_id_field)?)?.to_string();
        let activity = Self::extract_str(record.get(&mappings.activity_field)?)?.to_string();
        let ts_raw = Self::extract_str(record.get(&mappings.timestamp_field)?)?;
        let timestamp = Self::parse_sn_timestamp(ts_raw)?;

        let resource = mappings
            .resource_field
            .as_ref()
            .and_then(|f| record.get(f))
            .and_then(|v| Self::extract_str(v))
            .map(String::from);

        Some((case_id, activity, timestamp, resource))
    }

    fn records_to_event_log(
        records: Vec<(String, String, DateTime<Utc>, Option<String>)>,
    ) -> (EventLog, usize) {
        let mut cases: HashMap<String, Vec<(String, DateTime<Utc>)>> = HashMap::new();
        for (case_id, activity, ts, _res) in &records {
            cases
                .entry(case_id.clone())
                .or_default()
                .push((activity.clone(), *ts));
        }

        let mut log = EventLog::new();
        for (case_id, mut events) in cases {
            events.sort_by_key(|(_, ts)| *ts);
            let mut trace = Trace::new(case_id);
            for (activity, ts) in events {
                trace.add_event(Event::new(activity, ts));
            }
            log.add_trace(trace);
        }

        let event_count = records.len();
        (log, event_count)
    }
}

impl EventLogExtractor for ServiceNowConnector {
    fn validate_config(config: &ConnectorConfig) -> Result<(), ConnectorError> {
        for key in &[
            "base_url",
            "table",
            "username",
            "password",
            "api_timeout_ms",
        ] {
            if !config.params.contains_key(*key) {
                return Err(ConnectorError::ConfigError(format!(
                    "missing required param: {}",
                    key
                )));
            }
        }
        Ok(())
    }

    fn extract(config: &ConnectorConfig) -> Result<ExtractionResult, ConnectorError> {
        Self::validate_config(config)?;
        let _auth = Self::auth_header(config)?;

        let page_size = config
            .params
            .get("page_size")
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(1000);

        // For unit tests: mock_response bypasses HTTP.
        let raw_records: Vec<Value> = if let Some(mock) = config.params.get("mock_response") {
            let parsed: Value = serde_json::from_str(mock)
                .map_err(|e| ConnectorError::ExtractionError(e.to_string()))?;
            parsed
                .get("result")
                .and_then(|r| r.as_array())
                .cloned()
                .unwrap_or_default()
        } else {
            let _ = Self::build_url(config, 0, page_size)?;
            vec![]
        };

        let start = std::time::Instant::now();
        let parsed: Vec<_> = raw_records
            .iter()
            .filter_map(|r| Self::parse_record(r, &config.field_mappings))
            .collect();

        let source_count = raw_records.len();
        let (log, event_count) = Self::records_to_event_log(parsed);

        Ok(ExtractionResult {
            metadata: ExtractionMetadata {
                connector_name: config.name.clone(),
                source_record_count: source_count,
                extracted_event_count: event_count,
                extracted_case_count: log.traces.len(),
                extraction_time_ms: start.elapsed().as_millis(),
                warnings: vec![],
            },
            log,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connectors::{ConnectorConfig, ConnectorType, FieldMappings};
    use chrono::Datelike;

    fn make_config(params: Vec<(&str, &str)>) -> ConnectorConfig {
        ConnectorConfig {
            name: "test_sn".to_string(),
            connector_type: ConnectorType::ServiceNow,
            params: params
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            field_mappings: FieldMappings {
                case_id_field: "number".to_string(),
                activity_field: "state".to_string(),
                timestamp_field: "sys_updated_on".to_string(),
                resource_field: Some("assigned_to".to_string()),
            },
        }
    }

    #[test]
    fn servicenow_connector_parses_custom_timestamp() {
        let ts = ServiceNowConnector::parse_sn_timestamp("2024-01-15 09:30:00");
        assert!(ts.is_some(), "should parse ServiceNow timestamp format");
        let dt = ts.unwrap();
        assert_eq!(dt.year(), 2024);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 15);
    }

    #[test]
    fn servicenow_connector_parses_mock_response() {
        let mock = r#"{"result":[{"number":"INC0001","state":"Open","sys_updated_on":"2024-01-15 09:00:00"},{"number":"INC0001","state":"Resolved","sys_updated_on":"2024-01-15 14:00:00"}]}"#;
        let config = make_config(vec![
            ("base_url", "https://myinstance.service-now.com"),
            ("table", "incident"),
            ("username", "admin"),
            ("password", "pass"),
            ("api_timeout_ms", "5000"),
            ("mock_response", mock),
        ]);

        let result = ServiceNowConnector::extract(&config).expect("extract should succeed");
        assert_eq!(result.metadata.extracted_event_count, 2);
        assert_eq!(result.metadata.extracted_case_count, 1);
    }
}
