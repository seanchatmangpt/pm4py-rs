/// A2A JSON-RPC 2.0 protocol types for pm4py-rust.
///
/// Implements the Google Agent-to-Agent protocol subset needed for
/// task-based skill invocation.
use serde::{Deserialize, Serialize};
use serde_json::Value;

// ── JSON-RPC error codes ─────────────────────────────────────────────────────

pub const INVALID_REQUEST: i64 = -32600;
pub const INVALID_PARAMS: i64 = -32602;
pub const METHOD_NOT_FOUND: i64 = -32601;
pub const TASK_NOT_FOUND: i64 = -32001;

// ── Request / Response ───────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct A2ARequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct A2AResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<A2AError>,
}

impl A2AResponse {
    pub fn ok(id: Option<Value>, result: Value) -> Self {
        A2AResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: Option<Value>, code: i64, message: &str) -> Self {
        A2AResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(A2AError {
                code,
                message: message.to_string(),
            }),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct A2AError {
    pub code: i64,
    pub message: String,
}

// ── Task types ───────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: String,
    pub status: TaskStatus,
    pub artifacts: Vec<Artifact>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskStatus {
    pub state: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Artifact {
    pub parts: Vec<ArtifactPart>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArtifactPart {
    #[serde(rename = "type")]
    pub part_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

// ── AgentCard / Skill ────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Clone)]
pub struct AgentCard {
    pub name: String,
    pub version: String,
    pub description: String,
    pub url: String,
    pub capabilities: Vec<String>,
    pub skills: Vec<AgentSkill>,
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct AgentSkill {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(rename = "inputModes")]
    pub input_modes: Vec<String>,
}
