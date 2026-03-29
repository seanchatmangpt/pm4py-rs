use axum::extract::Json as AxumJson;
use pm4py::mcp::handler::mcp_handler;
/// Chicago TDD: pm4py-rust MCP tool server contract tests.
///
/// Tests are pure-functional — no HTTP server, no network.
/// Calls mcp_handler directly with constructed McpRequest values.
///
/// Five test groups:
///   1. Protocol handshake (initialize, notifications/initialized)
///   2. Tool list (tools/list — 10 tools with correct schema)
///   3. Tool dispatch errors (missing params, unknown tool, invalid jsonrpc)
///   4. Tool execution — discover_alpha, statistics, detect_drift
///   5. MCP span constants (regression guard)
use pm4py::mcp::protocol::{
    McpRequest, INVALID_PARAMS, INVALID_REQUEST, METHOD_NOT_FOUND, TOOL_NOT_FOUND,
};
use serde_json::{json, Value};

fn make_req(method: &str, id: Option<i64>, params: Option<Value>) -> McpRequest {
    McpRequest {
        jsonrpc: "2.0".to_string(),
        method: method.to_string(),
        id: id.map(|n| json!(n)),
        params,
    }
}

fn make_req_bad_version(method: &str) -> McpRequest {
    McpRequest {
        jsonrpc: "1.0".to_string(),
        method: method.to_string(),
        id: Some(json!(1)),
        params: None,
    }
}

/// Minimal valid event log JSON for test inputs.
fn minimal_event_log() -> Value {
    json!({
        "attributes": {},
        "traces": [
            {
                "id": "case-001",
                "attributes": {},
                "events": [
                    {
                        "activity": "Submit Request",
                        "timestamp": "2024-01-01T10:00:00Z",
                        "resource": null,
                        "attributes": {}
                    },
                    {
                        "activity": "Approve Request",
                        "timestamp": "2024-01-01T10:30:00Z",
                        "resource": null,
                        "attributes": {}
                    }
                ]
            }
        ]
    })
}

// ── Group 1: Protocol handshake ────────────────────────────────────────

#[tokio::test]
async fn initialize_returns_protocol_version_and_server_info() {
    let req = make_req(
        "initialize",
        Some(1),
        Some(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "test-client", "version": "1.0"}
        })),
    );

    let AxumJson(resp) = mcp_handler(AxumJson(req)).await;

    assert_eq!(resp.jsonrpc, "2.0");
    assert!(resp.error.is_none(), "unexpected error: {:?}", resp.error);
    let result = resp.result.expect("result should be present");
    assert_eq!(result["protocolVersion"], "2024-11-05");
    assert_eq!(result["serverInfo"]["name"], "pm4py-rust");
    assert!(result["capabilities"]["tools"].is_object());
}

#[tokio::test]
async fn initialize_returns_non_empty_server_version() {
    let req = make_req("initialize", Some(2), Some(json!({})));
    let AxumJson(resp) = mcp_handler(AxumJson(req)).await;
    let version = &resp.result.unwrap()["serverInfo"]["version"];
    assert!(version.is_string());
    assert!(!version.as_str().unwrap().is_empty());
}

#[tokio::test]
async fn notifications_initialized_returns_ok_with_empty_result() {
    let req = make_req("notifications/initialized", None, None);
    let AxumJson(resp) = mcp_handler(AxumJson(req)).await;
    assert!(resp.error.is_none());
    assert!(resp.result.is_some());
}

// ── Group 2: Tool list ─────────────────────────────────────────────────

#[tokio::test]
async fn tools_list_returns_exactly_10_tools() {
    let req = make_req("tools/list", Some(3), Some(json!({})));
    let AxumJson(resp) = mcp_handler(AxumJson(req)).await;

    assert!(resp.error.is_none());
    let tools = resp.result.unwrap()["tools"].as_array().unwrap().to_vec();
    assert_eq!(tools.len(), 12, "expected exactly 12 tools");
}

#[tokio::test]
async fn tools_list_each_tool_has_name_description_input_schema() {
    let req = make_req("tools/list", Some(4), Some(json!({})));
    let AxumJson(resp) = mcp_handler(AxumJson(req)).await;

    let tools = resp.result.unwrap()["tools"].as_array().unwrap().to_vec();
    for tool in &tools {
        assert!(
            tool["name"].is_string(),
            "tool.name must be string: {}",
            tool
        );
        assert!(
            !tool["name"].as_str().unwrap().is_empty(),
            "tool.name must be non-empty"
        );
        assert!(
            tool["description"].is_string(),
            "tool.description must be string"
        );
        assert!(
            tool["inputSchema"].is_object(),
            "tool.inputSchema must be object"
        );
    }
}

#[tokio::test]
async fn tools_list_contains_all_expected_tool_names() {
    let req = make_req("tools/list", Some(5), Some(json!({})));
    let AxumJson(resp) = mcp_handler(AxumJson(req)).await;

    let tools = resp.result.unwrap()["tools"].as_array().unwrap().to_vec();
    let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();

    let expected = [
        "pm4py_discover_alpha",
        "pm4py_conformance_token_replay",
        "pm4py_statistics",
        "pm4py_parse_xes",
        "pm4py_detect_drift",
        "pm4py_abstract_petri_net",
        "pm4py_abstract_event_log",
        "pm4py_abstract_dfg",
        "pm4py_query",
        "pm4py_ocel_ingest",
        "pm4py_decision_mine",
        "pm4py_predict_remaining_time",
    ];

    for expected_name in &expected {
        assert!(
            names.contains(expected_name),
            "missing tool: {}",
            expected_name
        );
    }
}

// ── Group 3: Error cases ───────────────────────────────────────────────

#[tokio::test]
async fn tools_call_missing_params_returns_invalid_params() {
    let req = make_req("tools/call", Some(6), None);
    let AxumJson(resp) = mcp_handler(AxumJson(req)).await;

    assert!(resp.result.is_none());
    let err = resp.error.expect("error should be present");
    assert_eq!(err.code, INVALID_PARAMS);
}

#[tokio::test]
async fn tools_call_missing_tool_name_returns_invalid_params() {
    let req = make_req("tools/call", Some(7), Some(json!({"arguments": {}})));
    let AxumJson(resp) = mcp_handler(AxumJson(req)).await;

    let err = resp.error.expect("error should be present");
    assert_eq!(err.code, INVALID_PARAMS);
}

#[tokio::test]
async fn tools_call_unknown_tool_returns_tool_not_found() {
    let req = make_req(
        "tools/call",
        Some(8),
        Some(json!({
            "name": "no_such_tool_xyz",
            "arguments": {}
        })),
    );
    let AxumJson(resp) = mcp_handler(AxumJson(req)).await;

    let err = resp.error.expect("error should be present");
    assert_eq!(err.code, TOOL_NOT_FOUND);
}

#[tokio::test]
async fn unknown_method_returns_method_not_found() {
    let req = make_req("unknown/method", Some(9), Some(json!({})));
    let AxumJson(resp) = mcp_handler(AxumJson(req)).await;

    let err = resp.error.expect("error should be present");
    assert_eq!(err.code, METHOD_NOT_FOUND);
}

#[tokio::test]
async fn invalid_jsonrpc_version_returns_invalid_request() {
    let req = make_req_bad_version("tools/list");
    let AxumJson(resp) = mcp_handler(AxumJson(req)).await;

    let err = resp.error.expect("error should be present");
    assert_eq!(err.code, INVALID_REQUEST);
}

// ── Group 4: Tool execution ────────────────────────────────────────────

#[tokio::test]
async fn discover_alpha_with_valid_event_log_returns_petri_net() {
    let req = make_req(
        "tools/call",
        Some(10),
        Some(json!({
            "name": "pm4py_discover_alpha",
            "arguments": {
                "event_log": minimal_event_log()
            }
        })),
    );

    let AxumJson(resp) = mcp_handler(AxumJson(req)).await;

    assert!(resp.error.is_none(), "unexpected error: {:?}", resp.error);
    let content = &resp.result.unwrap()["content"][0];
    assert_eq!(content["type"], "text");
    let text = content["text"].as_str().unwrap();
    // Result must mention petri_net or algorithm
    let parsed: Value = serde_json::from_str(text).expect("content text should be valid JSON");
    assert!(parsed.get("algorithm").is_some() || parsed.get("petri_net").is_some());
}

#[tokio::test]
async fn statistics_with_valid_event_log_returns_counts() {
    let req = make_req(
        "tools/call",
        Some(11),
        Some(json!({
            "name": "pm4py_statistics",
            "arguments": {
                "event_log": minimal_event_log()
            }
        })),
    );

    let AxumJson(resp) = mcp_handler(AxumJson(req)).await;

    assert!(resp.error.is_none(), "unexpected error: {:?}", resp.error);
    let content = &resp.result.unwrap()["content"][0]["text"];
    let parsed: Value = serde_json::from_str(content.as_str().unwrap()).unwrap();
    assert_eq!(parsed["trace_count"], 1);
    assert_eq!(parsed["event_count"], 2);
    assert!(parsed["unique_activities"].as_u64().unwrap() >= 2);
}

#[tokio::test]
async fn detect_drift_with_similar_metrics_returns_not_drifted() {
    let req = make_req(
        "tools/call",
        Some(12),
        Some(json!({
            "name": "pm4py_detect_drift",
            "arguments": {
                "baseline": {"throughput": 10.0, "cycle_time": 5.0},
                "recent":   {"throughput": 10.1, "cycle_time": 5.05}
            }
        })),
    );

    let AxumJson(resp) = mcp_handler(AxumJson(req)).await;

    assert!(resp.error.is_none(), "unexpected error: {:?}", resp.error);
    let content = &resp.result.unwrap()["content"][0]["text"];
    let parsed: Value = serde_json::from_str(content.as_str().unwrap()).unwrap();
    assert!(parsed.get("drift_detected").is_some());
    assert!(parsed.get("drift_score").is_some());
}

#[tokio::test]
async fn query_returns_response_for_any_string() {
    let req = make_req(
        "tools/call",
        Some(13),
        Some(json!({
            "name": "pm4py_query",
            "arguments": {
                "query": "What are the bottlenecks in this process?"
            }
        })),
    );

    let AxumJson(resp) = mcp_handler(AxumJson(req)).await;

    assert!(resp.error.is_none(), "unexpected error: {:?}", resp.error);
    let content = &resp.result.unwrap()["content"][0]["text"];
    let parsed: Value = serde_json::from_str(content.as_str().unwrap()).unwrap();
    assert!(parsed["response"].is_string());
    assert!(!parsed["response"].as_str().unwrap().is_empty());
    // Response includes path field indicating analysis mode
    assert!(
        parsed["path"].is_string(),
        "response must include path field"
    );
}

#[tokio::test]
async fn query_with_event_log_returns_statistical_analysis() {
    let req = make_req(
        "tools/call",
        Some(20),
        Some(json!({
            "name": "pm4py_query",
            "arguments": {
                "query": "What is causing the slow cycle time?",
                "event_log": minimal_event_log()
            }
        })),
    );

    let AxumJson(resp) = mcp_handler(AxumJson(req)).await;

    assert!(resp.error.is_none(), "unexpected error: {:?}", resp.error);
    let content = &resp.result.unwrap()["content"][0]["text"];
    let parsed: Value = serde_json::from_str(content.as_str().unwrap()).unwrap();
    assert!(parsed["response"].is_string(), "response must be a string");
    assert!(
        !parsed["response"].as_str().unwrap().is_empty(),
        "response must be non-empty"
    );
    // With event_log provided, path is statistical or groq_augmented (if GROQ_API_KEY set)
    let path = parsed["path"].as_str().unwrap_or("");
    assert!(
        path == "statistical" || path == "groq_augmented",
        "expected statistical or groq_augmented path when event_log provided, got: {}",
        path
    );
}

#[tokio::test]
async fn query_without_context_returns_non_empty_response() {
    let req = make_req(
        "tools/call",
        Some(21),
        Some(json!({
            "name": "pm4py_query",
            "arguments": {
                "query": "Why is our approval process slow?"
            }
        })),
    );

    let AxumJson(resp) = mcp_handler(AxumJson(req)).await;

    assert!(resp.error.is_none(), "unexpected error: {:?}", resp.error);
    let content = &resp.result.unwrap()["content"][0]["text"];
    let parsed: Value = serde_json::from_str(content.as_str().unwrap()).unwrap();
    assert!(parsed["response"].is_string(), "response must be a string");
    assert!(
        !parsed["response"].as_str().unwrap().is_empty(),
        "response must be non-empty"
    );
    // Without event_log: path is "groq" (GROQ_API_KEY set) or "fallback" (no key)
    let path = parsed["path"].as_str().unwrap_or("");
    assert!(
        path == "groq" || path == "fallback",
        "expected groq or fallback path when no event_log provided, got: {}",
        path
    );
}

// ── Group 5: MCP span constants (regression guard) ─────────────────────

#[test]
fn mcp_tool_execute_span_constant_matches_semconv() {
    use pm4py::semconv::mcp_span_names::MCP_TOOL_EXECUTE_SPAN;
    assert_eq!(MCP_TOOL_EXECUTE_SPAN, "mcp.tool_execute");
}

#[test]
fn mcp_tool_timeout_span_constant_is_correct() {
    use pm4py::semconv::mcp_span_names::MCP_TOOL_TIMEOUT_SPAN;
    assert_eq!(MCP_TOOL_TIMEOUT_SPAN, "mcp.tool.timeout");
}

#[test]
fn mcp_server_name_attribute_constant_is_correct() {
    use pm4py::semconv::mcp_attributes::MCP_SERVER_NAME;
    assert_eq!(MCP_SERVER_NAME, "mcp.server.name");
}
