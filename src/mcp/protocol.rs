/// MCP JSON-RPC 2.0 protocol types for pm4py-rust MCP tool server.
///
/// Implements MCP 2024-11-05 spec: stateless JSON-RPC over HTTP.
/// No session management needed — pm4py-rust is a stateless tool server.
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC 2.0 request
#[derive(Debug, Deserialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub method: String,
    pub id: Option<Value>,
    pub params: Option<Value>,
}

/// JSON-RPC 2.0 response (success)
#[derive(Debug, Serialize)]
pub struct McpResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<McpError>,
}

impl McpResponse {
    pub fn ok(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: Option<Value>, code: i64, message: &str) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(McpError {
                code,
                message: message.to_string(),
                data: None,
            }),
        }
    }
}

/// JSON-RPC 2.0 error object
#[derive(Debug, Serialize)]
pub struct McpError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

// Standard JSON-RPC error codes
pub const PARSE_ERROR: i64 = -32700;
pub const INVALID_REQUEST: i64 = -32600;
pub const METHOD_NOT_FOUND: i64 = -32601;
pub const INVALID_PARAMS: i64 = -32602;
pub const TOOL_NOT_FOUND: i64 = -32001;
pub const TOOL_EXECUTION_ERROR: i64 = -32000;

/// MCP content item (text result)
#[derive(Debug, Serialize)]
pub struct ContentItem {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

impl ContentItem {
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            content_type: "text".to_string(),
            text: text.into(),
        }
    }
}

/// MCP tool definition (returned by tools/list)
#[derive(Debug, Serialize, Clone)]
pub struct ToolDefinition {
    pub name: &'static str,
    pub description: &'static str,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

impl ToolDefinition {
    pub const fn new_lazy(name: &'static str, description: &'static str) -> ToolDefinitionBuilder {
        ToolDefinitionBuilder { name, description }
    }
}

pub struct ToolDefinitionBuilder {
    pub name: &'static str,
    pub description: &'static str,
}
