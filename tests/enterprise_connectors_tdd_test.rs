/// Chicago TDD — Enterprise connector WvdA soundness + security tests.
///
/// RED tests define requirements; GREEN tests prove implementation.
///
/// T1: sap_validate_rejects_missing_timeout — WvdA deadlock-freedom
/// T2: servicenow_validate_rejects_missing_timeout — WvdA deadlock-freedom
/// T3: salesforce_validate_rejects_non_https_url — security: HTTPS only
/// T4: sap_extract_span_name_matches_semconv — schema-compiled constant
/// T5: salesforce_extract_span_name_matches_semconv — schema-compiled constant
/// T6: servicenow_extract_span_name_matches_semconv — schema-compiled constant
/// T7: connector_type_values_match_semconv — type enum consistency
use pm4py::connectors::{
    ConnectorConfig, ConnectorType, EventLogExtractor, FieldMappings, SalesforceConnector,
    SapODataConnector, ServiceNowConnector,
};
use pm4py::semconv::connector_attributes;
use pm4py::semconv::connector_span_names;

fn sap_config_without_timeout() -> ConnectorConfig {
    ConnectorConfig {
        name: "sap_wvda_test".to_string(),
        connector_type: ConnectorType::Sap,
        params: [
            ("base_url", "https://host.sap.com"),
            ("entity_set", "Orders"),
        ]
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect(),
        field_mappings: FieldMappings::default(),
    }
}

fn sap_config_with_timeout() -> ConnectorConfig {
    ConnectorConfig {
        name: "sap_wvda_test".to_string(),
        connector_type: ConnectorType::Sap,
        params: [
            ("base_url", "https://host.sap.com"),
            ("entity_set", "Orders"),
            ("api_timeout_ms", "5000"),
        ]
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect(),
        field_mappings: FieldMappings::default(),
    }
}

fn servicenow_config_without_timeout() -> ConnectorConfig {
    ConnectorConfig {
        name: "sn_wvda_test".to_string(),
        connector_type: ConnectorType::ServiceNow,
        params: [
            ("base_url", "https://myinstance.service-now.com"),
            ("table", "incident"),
            ("username", "admin"),
            ("password", "pass"),
        ]
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect(),
        field_mappings: FieldMappings::default(),
    }
}

fn salesforce_config_with_http_url() -> ConnectorConfig {
    ConnectorConfig {
        name: "sf_security_test".to_string(),
        connector_type: ConnectorType::Salesforce,
        params: [
            ("instance_url", "http://myorg.salesforce.com"),
            ("soql_query", "SELECT Id FROM Task"),
        ]
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect(),
        field_mappings: FieldMappings::default(),
    }
}

// ─── T1: SAP WvdA timeout requirement ────────────────────────────────────────

#[test]
fn sap_validate_rejects_missing_timeout() {
    let config = sap_config_without_timeout();
    let result = SapODataConnector::validate_config(&config);
    assert!(
        result.is_err(),
        "SAP validate_config must reject config without api_timeout_ms (WvdA deadlock-freedom)"
    );
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("api_timeout_ms"),
        "Error must mention api_timeout_ms, got: {}",
        err_msg
    );
}

#[test]
fn sap_validate_accepts_config_with_timeout() {
    let config = sap_config_with_timeout();
    assert!(
        SapODataConnector::validate_config(&config).is_ok(),
        "SAP validate_config must accept config with api_timeout_ms"
    );
}

// ─── T2: ServiceNow WvdA timeout requirement ─────────────────────────────────

#[test]
fn servicenow_validate_rejects_missing_timeout() {
    let config = servicenow_config_without_timeout();
    let result = ServiceNowConnector::validate_config(&config);
    assert!(
        result.is_err(),
        "ServiceNow validate_config must reject config without api_timeout_ms (WvdA)"
    );
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("api_timeout_ms"),
        "Error must mention api_timeout_ms, got: {}",
        err_msg
    );
}

// ─── T3: Salesforce HTTPS security requirement ───────────────────────────────

#[test]
fn salesforce_validate_rejects_non_https_url() {
    let config = salesforce_config_with_http_url();
    let result = SalesforceConnector::validate_config(&config);
    assert!(
        result.is_err(),
        "Salesforce validate_config must reject http:// instance_url (security)"
    );
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.to_lowercase().contains("https"),
        "Error must mention https requirement, got: {}",
        err_msg
    );
}

// ─── T4-T6: Semconv span name constants ──────────────────────────────────────

#[test]
fn sap_extract_span_name_matches_semconv() {
    assert_eq!(
        connector_span_names::CONNECTOR_SAP_EXTRACT_SPAN,
        "connector.sap.extract"
    );
}

#[test]
fn salesforce_extract_span_name_matches_semconv() {
    assert_eq!(
        connector_span_names::CONNECTOR_SALESFORCE_EXTRACT_SPAN,
        "connector.salesforce.extract"
    );
}

#[test]
fn servicenow_extract_span_name_matches_semconv() {
    assert_eq!(
        connector_span_names::CONNECTOR_SERVICENOW_EXTRACT_SPAN,
        "connector.servicenow.extract"
    );
}

// ─── T7: Connector type enum consistency ─────────────────────────────────────

#[test]
fn connector_type_sap_value_matches_semconv() {
    assert_eq!(connector_attributes::connector_type::SAP, "sap");
}

#[test]
fn connector_type_salesforce_value_matches_semconv() {
    assert_eq!(
        connector_attributes::connector_type::SALESFORCE,
        "salesforce"
    );
}

#[test]
fn connector_type_servicenow_value_matches_semconv() {
    assert_eq!(
        connector_attributes::connector_type::SERVICENOW,
        "servicenow"
    );
}

#[test]
fn connector_api_timeout_ms_attribute_matches_semconv() {
    assert_eq!(
        connector_attributes::CONNECTOR_API_TIMEOUT_MS,
        "connector.api_timeout_ms"
    );
}
