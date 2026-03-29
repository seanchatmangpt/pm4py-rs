/// Chicago TDD — Enterprise connector integration tests.
///
/// T1: SapODataConnector parses SAP BTP OData event format into EventLog traces grouped by caseId
/// T2: SalesforceConnector groups Salesforce opportunity records by OwnerId as case
/// T3: ServiceNowConnector groups ServiceNow incident records by sys_id (incident number) as case
use pm4py::connectors::{
    ConnectorConfig, ConnectorType, EventLogExtractor, FieldMappings, SalesforceConnector,
    SapODataConnector, ServiceNowConnector,
};

fn sap_config(mock_response: &str) -> ConnectorConfig {
    ConnectorConfig {
        name: "test_sap_enterprise".to_string(),
        connector_type: ConnectorType::Sap,
        params: [
            (
                "base_url",
                "https://myhost.sap.com/sap/opu/odata/sap/API_PROCESS_ORDER_SRV",
            ),
            ("entity_set", "ProcessOrders"),
            ("auth_type", "basic"),
            ("username", "sap_user"),
            ("password", "sap_pass"),
            ("api_timeout_ms", "5000"),
            ("mock_response", mock_response),
        ]
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect(),
        field_mappings: FieldMappings::default(),
    }
}

fn salesforce_config(mock_response: &str) -> ConnectorConfig {
    ConnectorConfig {
        name: "test_sf_enterprise".to_string(),
        connector_type: ConnectorType::Salesforce,
        params: [
            ("instance_url", "https://myorg.salesforce.com"),
            (
                "soql_query",
                "SELECT Id, StageName, LastModifiedDate, OwnerId FROM Opportunity",
            ),
            ("mock_response", mock_response),
        ]
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect(),
        field_mappings: FieldMappings {
            case_id_field: "OwnerId".to_string(),
            activity_field: "StageName".to_string(),
            timestamp_field: "LastModifiedDate".to_string(),
            resource_field: Some("Id".to_string()),
        },
    }
}

fn servicenow_config(mock_response: &str) -> ConnectorConfig {
    ConnectorConfig {
        name: "test_sn_enterprise".to_string(),
        connector_type: ConnectorType::ServiceNow,
        params: [
            ("base_url", "https://myinstance.service-now.com"),
            ("table", "incident"),
            ("username", "admin"),
            ("password", "admin_pass"),
            ("api_timeout_ms", "5000"),
            ("mock_response", mock_response),
        ]
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect(),
        field_mappings: FieldMappings {
            case_id_field: "sys_id".to_string(),
            activity_field: "state".to_string(),
            timestamp_field: "sys_updated_on".to_string(),
            resource_field: Some("assigned_to".to_string()),
        },
    }
}

/// T1: SAP BTP OData events are grouped by caseId and sorted by timestamp.
///
/// Input: two events for case_1 (Submit → Approve) and one for case_2 (Submit).
/// Expected: 2 traces, 3 total events.
#[test]
fn test_sap_connector_parses_event_format() {
    let mock = r#"{
        "d": {
            "results": [
                {"case_id": "case_1", "activity": "Submit",  "timestamp": "/Date(1609459200000)/", "resource": "user_a"},
                {"case_id": "case_1", "activity": "Approve", "timestamp": "/Date(1609545600000)/", "resource": "user_b"},
                {"case_id": "case_2", "activity": "Submit",  "timestamp": "/Date(1609459200000)/", "resource": "user_c"}
            ]
        }
    }"#;

    let config = sap_config(mock);
    let result = SapODataConnector::extract(&config).expect("SAP extract should succeed");

    assert_eq!(
        result.metadata.extracted_case_count, 2,
        "expected 2 SAP cases"
    );
    assert_eq!(
        result.metadata.extracted_event_count, 3,
        "expected 3 SAP events"
    );

    // The case with 2 events should have them in timestamp order
    let case1 = result
        .log
        .traces
        .iter()
        .find(|t| t.id == "case_1")
        .expect("case_1 must exist");
    assert_eq!(case1.events.len(), 2, "case_1 must have 2 events");
    assert_eq!(
        case1.events[0].activity, "Submit",
        "first event must be Submit"
    );
    assert_eq!(
        case1.events[1].activity, "Approve",
        "second event must be Approve"
    );
}

/// T2: Salesforce opportunity records are grouped by OwnerId as the process case.
///
/// Input: 3 records — owner_1 has two stage changes, owner_2 has one.
/// Expected: 2 traces, 3 total events.
#[test]
fn test_salesforce_connector_groups_by_owner() {
    let mock = r#"{
        "records": [
            {"Id": "opp_a", "StageName": "Prospecting",        "LastModifiedDate": "2024-03-01T10:00:00Z", "OwnerId": "owner_1"},
            {"Id": "opp_b", "StageName": "Needs Analysis",     "LastModifiedDate": "2024-03-05T10:00:00Z", "OwnerId": "owner_1"},
            {"Id": "opp_c", "StageName": "Closed Won",         "LastModifiedDate": "2024-03-10T10:00:00Z", "OwnerId": "owner_2"}
        ]
    }"#;

    let config = salesforce_config(mock);
    let result = SalesforceConnector::extract(&config).expect("Salesforce extract should succeed");

    assert_eq!(
        result.metadata.extracted_case_count, 2,
        "expected 2 Salesforce cases (owners)"
    );
    assert_eq!(
        result.metadata.extracted_event_count, 3,
        "expected 3 Salesforce events"
    );

    let owner1_trace = result
        .log
        .traces
        .iter()
        .find(|t| t.id == "owner_1")
        .expect("owner_1 trace must exist");
    assert_eq!(
        owner1_trace.events.len(),
        2,
        "owner_1 must have 2 stage events"
    );

    let owner2_trace = result
        .log
        .traces
        .iter()
        .find(|t| t.id == "owner_2")
        .expect("owner_2 trace must exist");
    assert_eq!(
        owner2_trace.events.len(),
        1,
        "owner_2 must have 1 stage event"
    );
}

/// T3: ServiceNow incidents are grouped by sys_id; each state change is an event.
///
/// Input: INC0001 has two state changes (Open → Resolved), INC0002 has one (Open).
/// Expected: 2 traces, 3 total events.
#[test]
fn test_servicenow_connector_groups_by_incident_id() {
    let mock = r#"{
        "result": [
            {"sys_id": "INC0001", "state": "Open",     "sys_updated_on": "2024-01-15 09:00:00", "assigned_to": "agent_a"},
            {"sys_id": "INC0001", "state": "Resolved", "sys_updated_on": "2024-01-15 14:00:00", "assigned_to": "agent_b"},
            {"sys_id": "INC0002", "state": "Open",     "sys_updated_on": "2024-01-16 08:00:00", "assigned_to": "agent_c"}
        ]
    }"#;

    let config = servicenow_config(mock);
    let result = ServiceNowConnector::extract(&config).expect("ServiceNow extract should succeed");

    assert_eq!(
        result.metadata.extracted_case_count, 2,
        "expected 2 ServiceNow incidents"
    );
    assert_eq!(
        result.metadata.extracted_event_count, 3,
        "expected 3 ServiceNow events"
    );

    let inc1 = result
        .log
        .traces
        .iter()
        .find(|t| t.id == "INC0001")
        .expect("INC0001 trace must exist");
    assert_eq!(
        inc1.events.len(),
        2,
        "INC0001 must have 2 state-change events"
    );
    assert_eq!(
        inc1.events[0].activity, "Open",
        "first INC0001 event must be Open"
    );
    assert_eq!(
        inc1.events[1].activity, "Resolved",
        "second INC0001 event must be Resolved"
    );

    let inc2 = result
        .log
        .traces
        .iter()
        .find(|t| t.id == "INC0002")
        .expect("INC0002 trace must exist");
    assert_eq!(inc2.events.len(), 1, "INC0002 must have 1 event");
}
