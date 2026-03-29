/// MCP JSON-RPC handler for pm4py-rust process mining tools.
///
/// Stateless HTTP handler — no session, no streaming.
/// Each POST is an independent JSON-RPC call.
///
/// WvdA boundedness guards:
/// - Event log max 10,000 events enforced before heavy algorithms
/// - 25s tokio timeout wraps all tool calls
/// - Violation returns TOOL_EXECUTION_ERROR (-32000)
use axum::extract::Json as AxumJson;
use opentelemetry::global;
use opentelemetry::trace::{Span, Tracer};
use opentelemetry::KeyValue;
use serde_json::{json, Value};
use tokio::time::{timeout, Duration};

use crate::mcp::protocol::{
    ContentItem, McpRequest, McpResponse, INVALID_PARAMS, INVALID_REQUEST, METHOD_NOT_FOUND,
    TOOL_EXECUTION_ERROR, TOOL_NOT_FOUND,
};
use crate::mcp::tools::{all_tools, find_tool};
use crate::semconv::mcp_attributes::{
    MCP_PROTOCOL_VERSION, MCP_SERVER_NAME, MCP_TOOL_INPUT_SIZE, MCP_TOOL_NAME,
    MCP_TOOL_OUTPUT_SIZE, MCP_TOOL_TIMEOUT_MS,
};
use crate::semconv::mcp_span_names::{MCP_TOOL_EXECUTE_SPAN, MCP_TOOL_TIMEOUT_SPAN};

const TOOL_TIMEOUT_MS: u64 = 25_000;
const PROTOCOL_VERSION: &str = "2024-11-05";
const SERVER_NAME: &str = "pm4py-rust";
const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Main MCP JSON-RPC handler. Axum entry point for POST /mcp.
pub async fn mcp_handler(AxumJson(req): AxumJson<McpRequest>) -> AxumJson<McpResponse> {
    if req.jsonrpc != "2.0" {
        return AxumJson(McpResponse::error(
            req.id,
            INVALID_REQUEST,
            "Invalid jsonrpc version",
        ));
    }

    let response = match req.method.as_str() {
        "initialize" => handle_initialize(req.id),
        "notifications/initialized" => return AxumJson(McpResponse::ok(None, json!({}))),
        "tools/list" => handle_tools_list(req.id),
        "tools/call" => handle_tools_call(req.id, req.params).await,
        _ => McpResponse::error(req.id, METHOD_NOT_FOUND, "Method not found"),
    };

    AxumJson(response)
}

fn handle_initialize(id: Option<Value>) -> McpResponse {
    McpResponse::ok(
        id,
        json!({
            "protocolVersion": PROTOCOL_VERSION,
            "capabilities": {
                "tools": { "listChanged": false }
            },
            "serverInfo": {
                "name": SERVER_NAME,
                "version": SERVER_VERSION
            }
        }),
    )
}

fn handle_tools_list(id: Option<Value>) -> McpResponse {
    let tools: Vec<Value> = all_tools()
        .into_iter()
        .map(|t| {
            json!({
                "name": t.name,
                "description": t.description,
                "inputSchema": t.input_schema
            })
        })
        .collect();

    McpResponse::ok(id, json!({ "tools": tools }))
}

async fn handle_tools_call(id: Option<Value>, params: Option<Value>) -> McpResponse {
    let params = match params {
        Some(p) => p,
        None => return McpResponse::error(id, INVALID_PARAMS, "Missing params"),
    };

    let tool_name = match params.get("name").and_then(|v| v.as_str()) {
        Some(n) => n.to_string(),
        None => return McpResponse::error(id, INVALID_PARAMS, "Missing tool name"),
    };

    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));

    if find_tool(&tool_name).is_none() {
        return McpResponse::error(
            id,
            TOOL_NOT_FOUND,
            &format!("Tool '{}' not found", tool_name),
        );
    }

    let input_size = arguments.to_string().len() as i64;

    let tracer = global::tracer("pm4py-rust");
    let mut span = tracer.start(MCP_TOOL_EXECUTE_SPAN);
    span.set_attribute(KeyValue::new(MCP_SERVER_NAME, SERVER_NAME));
    span.set_attribute(KeyValue::new(MCP_PROTOCOL_VERSION, PROTOCOL_VERSION));
    span.set_attribute(KeyValue::new(MCP_TOOL_NAME, tool_name.clone()));
    span.set_attribute(KeyValue::new(MCP_TOOL_INPUT_SIZE, input_size));
    span.set_attribute(KeyValue::new(MCP_TOOL_TIMEOUT_MS, TOOL_TIMEOUT_MS as i64));

    let tool_name_clone = tool_name.clone();
    let args_clone = arguments.clone();

    let result = timeout(
        Duration::from_millis(TOOL_TIMEOUT_MS),
        crate::mcp::executor::execute_tool(&tool_name_clone, args_clone),
    )
    .await;

    match result {
        Ok(Ok(content_text)) => {
            let output_size = content_text.len() as i64;
            span.set_attribute(KeyValue::new(MCP_TOOL_OUTPUT_SIZE, output_size));
            McpResponse::ok(
                id,
                json!({
                    "content": [ContentItem::text(content_text)]
                }),
            )
        }
        Ok(Err(err)) => McpResponse::error(id, TOOL_EXECUTION_ERROR, &err),
        Err(_elapsed) => {
            let mut timeout_span = tracer.start(MCP_TOOL_TIMEOUT_SPAN);
            timeout_span.set_attribute(KeyValue::new(MCP_TOOL_NAME, tool_name));
            timeout_span.set_attribute(KeyValue::new(MCP_TOOL_TIMEOUT_MS, TOOL_TIMEOUT_MS as i64));
            McpResponse::error(id, TOOL_EXECUTION_ERROR, "Tool execution timed out")
        }
    }
}
