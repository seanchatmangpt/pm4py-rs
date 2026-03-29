/// Enterprise connector framework for pm4py-rust
///
/// Provides an `EventLogExtractor` trait and concrete connectors for:
/// - CSV files (wrapping `crate::io::csv::CSVReader`)
/// - Webhook inbound JSON payloads
/// - SAP OData v2/v4 (Basic/Bearer auth, $skip/$top pagination)
/// - Salesforce (OAuth2 password grant, SOQL queries, nextRecordsUrl pagination)
/// - ServiceNow (Basic auth, sysparm_* params, custom timestamp format)
pub mod csv;
pub mod salesforce;
pub mod sap;
pub mod servicenow;
pub mod webhook;

pub use csv::CsvConnector;
pub use salesforce::SalesforceConnector;
pub use sap::SapODataConnector;
pub use servicenow::ServiceNowConnector;
pub use webhook::WebhookConnector;

use crate::log::EventLog;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for any connector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorConfig {
    pub name: String,
    pub connector_type: ConnectorType,
    pub params: HashMap<String, String>,
    pub field_mappings: FieldMappings,
}

/// Which connector implementation to use
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ConnectorType {
    Csv,
    Webhook,
    Sap,
    Salesforce,
    ServiceNow,
}

/// Field name mappings from source schema to pm4py-rust EventLog fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldMappings {
    pub case_id_field: String,
    pub activity_field: String,
    pub timestamp_field: String,
    pub resource_field: Option<String>,
}

impl Default for FieldMappings {
    fn default() -> Self {
        Self {
            case_id_field: "case_id".to_string(),
            activity_field: "activity".to_string(),
            timestamp_field: "timestamp".to_string(),
            resource_field: Some("resource".to_string()),
        }
    }
}

/// Metadata about a completed extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionMetadata {
    pub connector_name: String,
    pub source_record_count: usize,
    pub extracted_event_count: usize,
    pub extracted_case_count: usize,
    pub extraction_time_ms: u128,
    pub warnings: Vec<String>,
}

/// Successful extraction result containing the EventLog and metadata
pub struct ExtractionResult {
    pub log: EventLog,
    pub metadata: ExtractionMetadata,
}

/// Errors that can occur during connector operations
#[derive(Debug)]
pub enum ConnectorError {
    ConfigError(String),
    ConnectionError(String),
    ExtractionError(String),
    SchemaMappingError(String),
}

impl std::fmt::Display for ConnectorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectorError::ConfigError(s) => write!(f, "Config error: {}", s),
            ConnectorError::ConnectionError(s) => write!(f, "Connection error: {}", s),
            ConnectorError::ExtractionError(s) => write!(f, "Extraction error: {}", s),
            ConnectorError::SchemaMappingError(s) => write!(f, "Schema mapping error: {}", s),
        }
    }
}

/// Trait implemented by all connector types.
/// Each connector validates its config and extracts an EventLog from a source.
pub trait EventLogExtractor: Send + Sync {
    fn validate_config(config: &ConnectorConfig) -> Result<(), ConnectorError>
    where
        Self: Sized;

    fn extract(config: &ConnectorConfig) -> Result<ExtractionResult, ConnectorError>
    where
        Self: Sized;
}
