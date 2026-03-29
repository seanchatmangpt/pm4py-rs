/// Webhook connector — parses an inbound JSON payload into an EventLog
use super::{
    ConnectorConfig, ConnectorError, EventLogExtractor, ExtractionMetadata, ExtractionResult,
    FieldMappings,
};
use crate::log::{Event, EventLog, Trace};
use chrono::DateTime;
use serde_json::Value;
use std::collections::BTreeMap;

pub struct WebhookConnector;

impl WebhookConnector {
    pub fn validate_config(config: &ConnectorConfig) -> Result<(), ConnectorError> {
        if !config.params.contains_key("payload_json") {
            return Err(ConnectorError::ConfigError(
                "missing required param: payload_json".to_string(),
            ));
        }
        Ok(())
    }

    pub fn extract(config: &ConnectorConfig) -> Result<ExtractionResult, ConnectorError> {
        Self::validate_config(config)?;
        let start = std::time::Instant::now();

        let payload_str = config.params.get("payload_json").ok_or_else(|| {
            ConnectorError::ConfigError("missing required param: payload_json".to_string())
        })?;

        let payload: Value = serde_json::from_str(payload_str)
            .map_err(|e| ConnectorError::ExtractionError(e.to_string()))?;

        let events_arr = payload
            .get("events")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                ConnectorError::SchemaMappingError(
                    "missing or non-array 'events' field in payload".to_string(),
                )
            })?;

        let log = build_log_from_events(events_arr, &config.field_mappings)?;

        let event_count: usize = log.traces.iter().map(|t| t.events.len()).sum();
        let case_count = log.traces.len();

        let metadata = ExtractionMetadata {
            connector_name: config.name.clone(),
            source_record_count: events_arr.len(),
            extracted_event_count: event_count,
            extracted_case_count: case_count,
            extraction_time_ms: start.elapsed().as_millis(),
            warnings: vec![],
        };

        Ok(ExtractionResult { log, metadata })
    }
}

/// Group JSON event objects by case_id and build an EventLog
fn build_log_from_events(
    events: &[Value],
    mappings: &FieldMappings,
) -> Result<EventLog, ConnectorError> {
    // Use an IndexMap to preserve insertion order of cases
    let mut case_map: std::collections::BTreeMap<String, Vec<(DateTime<chrono::Utc>, Event)>> =
        BTreeMap::new();

    for (idx, event_val) in events.iter().enumerate() {
        let obj = event_val.as_object().ok_or_else(|| {
            ConnectorError::SchemaMappingError(format!("event[{}] is not a JSON object", idx))
        })?;

        // Extract case_id
        let case_id = obj
            .get(&mappings.case_id_field)
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ConnectorError::SchemaMappingError(format!(
                    "event[{}] missing field '{}'",
                    idx, mappings.case_id_field
                ))
            })?
            .to_string();

        // Extract activity
        let activity = obj
            .get(&mappings.activity_field)
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ConnectorError::SchemaMappingError(format!(
                    "event[{}] missing field '{}'",
                    idx, mappings.activity_field
                ))
            })?
            .to_string();

        // Extract and parse timestamp
        let timestamp_str = obj
            .get(&mappings.timestamp_field)
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ConnectorError::SchemaMappingError(format!(
                    "event[{}] missing field '{}'",
                    idx, mappings.timestamp_field
                ))
            })?;

        let timestamp = DateTime::parse_from_rfc3339(timestamp_str)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .or_else(|_| {
                chrono::NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%dT%H:%M:%S%.f")
                    .map(|ndt| ndt.and_utc())
            })
            .map_err(|e| {
                ConnectorError::SchemaMappingError(format!(
                    "event[{}] timestamp '{}' parse error: {}",
                    idx, timestamp_str, e
                ))
            })?;

        // Build Event
        let mut ev = Event::new(activity, timestamp);

        // Optional resource
        if let Some(res_field) = &mappings.resource_field {
            if let Some(res) = obj.get(res_field).and_then(|v| v.as_str()) {
                ev.resource = Some(res.to_string());
            }
        }

        // Any remaining string fields become attributes
        for (k, v) in obj {
            if k != &mappings.case_id_field
                && k != &mappings.activity_field
                && k != &mappings.timestamp_field
            {
                if let Some(s) = v.as_str() {
                    ev.attributes.insert(k.clone(), s.to_string());
                }
            }
        }

        case_map.entry(case_id).or_default().push((timestamp, ev));
    }

    let mut log = EventLog::new();
    for (case_id, mut timed_events) in case_map {
        // Sort events within each trace by timestamp
        timed_events.sort_by_key(|(ts, _)| *ts);
        let mut trace = Trace::new(case_id);
        for (_, ev) in timed_events {
            trace.add_event(ev);
        }
        log.add_trace(trace);
    }

    Ok(log)
}

impl EventLogExtractor for WebhookConnector {
    fn validate_config(config: &ConnectorConfig) -> Result<(), ConnectorError> {
        WebhookConnector::validate_config(config)
    }

    fn extract(config: &ConnectorConfig) -> Result<ExtractionResult, ConnectorError> {
        WebhookConnector::extract(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connectors::{ConnectorType, FieldMappings};
    use std::collections::HashMap;

    fn make_config(params: HashMap<String, String>) -> ConnectorConfig {
        ConnectorConfig {
            name: "test-webhook".to_string(),
            connector_type: ConnectorType::Webhook,
            params,
            field_mappings: FieldMappings::default(),
        }
    }

    #[test]
    fn validate_config_rejects_missing_payload_json() {
        let config = make_config(HashMap::new());
        assert!(WebhookConnector::validate_config(&config).is_err());
    }

    #[test]
    fn validate_config_accepts_payload_json() {
        let mut params = HashMap::new();
        params.insert("payload_json".to_string(), r#"{"events":[]}"#.to_string());
        let config = make_config(params);
        assert!(WebhookConnector::validate_config(&config).is_ok());
    }

    #[test]
    fn extract_parses_webhook_payload_into_event_log() {
        let payload = serde_json::json!({
            "events": [
                {
                    "case_id": "case_1",
                    "activity": "register",
                    "timestamp": "2024-01-01T08:00:00Z",
                    "resource": "alice"
                },
                {
                    "case_id": "case_1",
                    "activity": "approve",
                    "timestamp": "2024-01-01T09:00:00Z",
                    "resource": "bob"
                },
                {
                    "case_id": "case_2",
                    "activity": "register",
                    "timestamp": "2024-01-01T08:30:00Z"
                }
            ]
        });

        let mut params = HashMap::new();
        params.insert("payload_json".to_string(), payload.to_string());
        let config = make_config(params);

        let result = WebhookConnector::extract(&config).expect("webhook extraction should succeed");
        assert_eq!(result.metadata.extracted_case_count, 2);
        assert_eq!(result.metadata.extracted_event_count, 3);
        assert_eq!(result.metadata.source_record_count, 3);
        assert_eq!(result.log.traces.len(), 2);
    }

    #[test]
    fn extract_returns_error_for_invalid_json() {
        let mut params = HashMap::new();
        params.insert("payload_json".to_string(), "not-json".to_string());
        let config = make_config(params);
        assert!(WebhookConnector::extract(&config).is_err());
    }

    #[test]
    fn extract_returns_error_for_missing_events_key() {
        let mut params = HashMap::new();
        params.insert("payload_json".to_string(), r#"{"data":[]}"#.to_string());
        let config = make_config(params);
        assert!(WebhookConnector::extract(&config).is_err());
    }

    #[test]
    fn extract_events_are_sorted_by_timestamp_within_trace() {
        let payload = serde_json::json!({
            "events": [
                {
                    "case_id": "case_1",
                    "activity": "approve",
                    "timestamp": "2024-01-01T09:00:00Z"
                },
                {
                    "case_id": "case_1",
                    "activity": "register",
                    "timestamp": "2024-01-01T08:00:00Z"
                }
            ]
        });

        let mut params = HashMap::new();
        params.insert("payload_json".to_string(), payload.to_string());
        let config = make_config(params);

        let result = WebhookConnector::extract(&config).expect("webhook extraction should succeed");
        let trace = &result.log.traces[0];
        assert_eq!(trace.events[0].activity, "register");
        assert_eq!(trace.events[1].activity, "approve");
    }

    #[test]
    fn connector_config_webhook_json_round_trip() {
        let config = ConnectorConfig {
            name: "wh-connector".to_string(),
            connector_type: ConnectorType::Webhook,
            params: [("payload_json".to_string(), r#"{"events":[]}"#.to_string())]
                .into_iter()
                .collect(),
            field_mappings: FieldMappings::default(),
        };
        let json = serde_json::to_string(&config).expect("config must serialize to JSON");
        let restored: ConnectorConfig =
            serde_json::from_str(&json).expect("config must deserialize from JSON");
        assert_eq!(restored.connector_type, ConnectorType::Webhook);
    }
}
