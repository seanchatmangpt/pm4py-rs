/// A2A JSON-RPC 2.0 axum handlers for pm4py-rust.
///
/// Exposes two entry points:
/// - `a2a_handler` — POST / — handles tasks/send, tasks/get, tasks/cancel
/// - `a2a_agent_card` — GET /.well-known/agent-card.json
///
/// Task storage is a process-global InMemoryTaskStorage (bounded at 1000 tasks).
///
/// OTEL span hierarchy:
///   a2a.task.create / a2a.skill.invoke  ← new A2A semconv
///     └─ mcp.tool_execute              ← existing MCP executor span
use axum::extract::Json as AxumJson;
use opentelemetry::global;
use opentelemetry::trace::{Span, Tracer};
use opentelemetry::KeyValue;
use serde_json::{json, Value};
use std::sync::{Arc, LazyLock};
use tokio::time::{timeout, Duration};
use uuid::Uuid;

use crate::a2a::artifact::build_artifact;
use crate::a2a::dispatch::parse_message_command;
use crate::a2a::protocol::{
    A2AResponse, Artifact, Task, TaskStatus, INVALID_PARAMS, INVALID_REQUEST, METHOD_NOT_FOUND,
    TASK_NOT_FOUND,
};
use crate::a2a::skills::all_skills;
use crate::a2a::task_storage::InMemoryTaskStorage;
use crate::mcp::executor;
use crate::semconv::a2a_attributes::{
    a2a_transport, A2A_AGENT_NAME, A2A_ARTIFACT_COUNT, A2A_MESSAGE_ROLE, A2A_SKILL_ID, A2A_TASK_ID,
    A2A_TASK_STATE, A2A_TRANSPORT,
};
use crate::semconv::a2a_span_names::{
    A2A_AGENT_CARD_SERVE_SPAN, A2A_MESSAGE_RECEIVE_SPAN, A2A_SKILL_INVOKE_SPAN,
    A2A_TASK_COMPLETE_SPAN, A2A_TASK_CREATE_SPAN, A2A_TASK_UPDATE_SPAN,
};

const AGENT_NAME: &str = "pm4py-rust";
/// WvdA liveness budget: A2A tool calls must complete within 60 seconds.
const A2A_TOOL_TIMEOUT_MS: u64 = 60_000;

/// Process-global task storage — bounded at MAX_TASK_STORAGE (1000).
static TASK_STORAGE: LazyLock<Arc<InMemoryTaskStorage>> =
    LazyLock::new(|| Arc::new(InMemoryTaskStorage::new()));

// ── Public handlers ────────────────────────────────────────────────────────

/// POST / — A2A JSON-RPC 2.0 endpoint.
pub async fn a2a_handler(AxumJson(body): AxumJson<Value>) -> AxumJson<Value> {
    let id = body.get("id").cloned();

    // Validate jsonrpc version
    if body.get("jsonrpc").and_then(|v| v.as_str()) != Some("2.0") {
        return as_json(A2AResponse::error(
            id,
            INVALID_REQUEST,
            "jsonrpc must be '2.0'",
        ));
    }

    let method = body.get("method").and_then(|v| v.as_str()).unwrap_or("");
    let params = body.get("params").cloned().unwrap_or(Value::Null);

    match method {
        "tasks/send" => handle_tasks_send(id, params).await,
        "tasks/get" => handle_tasks_get(id, params),
        "tasks/cancel" => handle_tasks_cancel(id, params),
        _ => as_json(A2AResponse::error(
            id,
            METHOD_NOT_FOUND,
            &format!("method '{}' not found", method),
        )),
    }
}

/// GET /.well-known/agent-card.json — A2A typed agent discovery card.
pub async fn a2a_agent_card() -> AxumJson<Value> {
    let tracer = global::tracer("pm4py-rust");
    let mut span = tracer.start(A2A_AGENT_CARD_SERVE_SPAN);
    span.set_attribute(KeyValue::new(A2A_AGENT_NAME, AGENT_NAME));
    span.set_attribute(KeyValue::new("a2a.protocol.version", "0.2.1"));

    let skills: Vec<Value> = all_skills()
        .into_iter()
        .map(|s| {
            json!({
                "id": s.id,
                "name": s.name,
                "description": s.description,
                "inputModes": s.input_modes,
            })
        })
        .collect();

    AxumJson(json!({
        "name": AGENT_NAME,
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Process mining engine with Alpha Miner, token replay, DFG, OCEL 2.0, and LLM abstraction.",
        "url": "http://localhost:8090",
        "capabilities": ["tools"],
        "protocolVersion": "0.2.1",
        "skills": skills,
    }))
}

// ── Private dispatch helpers ───────────────────────────────────────────────

async fn handle_tasks_send(id: Option<Value>, params: Value) -> AxumJson<Value> {
    // Span: a2a.message.receive — outermost span for this HTTP request
    let tracer_recv = global::tracer("pm4py-rust");
    let mut recv_span = tracer_recv.start(A2A_MESSAGE_RECEIVE_SPAN);
    recv_span.set_attribute(KeyValue::new(A2A_TRANSPORT, a2a_transport::HTTP));

    let task_id = params
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| "")
        .to_string();

    let task_id = if task_id.is_empty() {
        Uuid::new_v4().to_string()
    } else {
        task_id
    };

    let message = match params.get("message") {
        Some(m) => m.clone(),
        None => {
            return as_json(A2AResponse::error(
                id,
                INVALID_PARAMS,
                "params.message is required",
            ))
        }
    };

    // Extract role for OTEL
    let role = message
        .get("role")
        .and_then(|r| r.as_str())
        .unwrap_or("user")
        .to_string();

    // Create task in submitted state and persist it
    let task = Task {
        id: task_id.clone(),
        status: TaskStatus {
            state: "submitted".to_string(),
        },
        artifacts: vec![],
    };
    TASK_STORAGE.insert(task);

    // Span: a2a.task.create
    let tracer = global::tracer("pm4py-rust");
    let mut create_span = tracer.start(A2A_TASK_CREATE_SPAN);
    create_span.set_attribute(KeyValue::new(A2A_AGENT_NAME, AGENT_NAME));
    create_span.set_attribute(KeyValue::new(A2A_TASK_ID, task_id.clone()));
    create_span.set_attribute(KeyValue::new(A2A_TASK_STATE, "submitted"));
    create_span.set_attribute(KeyValue::new(A2A_TRANSPORT, a2a_transport::HTTP));
    create_span.set_attribute(KeyValue::new(A2A_MESSAGE_ROLE, role.clone()));

    // Parse tool invocation from message parts
    let (tool_name, tool_args) = match parse_message_command(&message) {
        Some(cmd) => cmd,
        None => {
            TASK_STORAGE.set_failed(&task_id);
            create_span.set_attribute(KeyValue::new(A2A_TASK_STATE, "failed"));
            return as_json(A2AResponse::error(
                id,
                INVALID_PARAMS,
                "message.parts must contain a tool invocation",
            ));
        }
    };

    // Span: a2a.skill.invoke (child of task.create)
    let mut skill_span = tracer.start(A2A_SKILL_INVOKE_SPAN);
    skill_span.set_attribute(KeyValue::new(A2A_AGENT_NAME, AGENT_NAME));
    skill_span.set_attribute(KeyValue::new(A2A_SKILL_ID, tool_name.clone()));
    create_span.set_attribute(KeyValue::new(A2A_SKILL_ID, tool_name.clone()));

    TASK_STORAGE.update_state(&task_id, "working");

    // Span: a2a.task.update — records state transition to "working"
    let tracer_update = global::tracer("pm4py-rust");
    let mut update_span = tracer_update.start(A2A_TASK_UPDATE_SPAN);
    update_span.set_attribute(KeyValue::new(A2A_TASK_ID, task_id.clone()));
    update_span.set_attribute(KeyValue::new(A2A_TASK_STATE, "working"));
    drop(update_span);

    // Execute tool with WvdA 60s timeout
    let exec_result = timeout(
        Duration::from_millis(A2A_TOOL_TIMEOUT_MS),
        executor::execute_tool(&tool_name, tool_args),
    )
    .await;

    let artifacts: Vec<Artifact> = match exec_result {
        Ok(Ok(output)) => vec![build_artifact(output)],
        Ok(Err(err)) => vec![build_artifact(format!("{{\"error\":{:?}}}", err))],
        Err(_) => {
            TASK_STORAGE.set_failed(&task_id);
            return as_json(A2AResponse::error(
                id,
                -32000,
                &format!(
                    "tool '{}' timed out after {}ms",
                    tool_name, A2A_TOOL_TIMEOUT_MS
                ),
            ));
        }
    };

    let artifact_count = artifacts.len() as i64;
    TASK_STORAGE.set_completed(&task_id, artifacts.clone());

    // Span: a2a.task.complete
    let mut complete_span = tracer.start(A2A_TASK_COMPLETE_SPAN);
    complete_span.set_attribute(KeyValue::new(A2A_AGENT_NAME, AGENT_NAME));
    complete_span.set_attribute(KeyValue::new(A2A_TASK_STATE, "completed"));
    complete_span.set_attribute(KeyValue::new(A2A_ARTIFACT_COUNT, artifact_count));
    create_span.set_attribute(KeyValue::new(A2A_TASK_STATE, "completed"));

    let task = Task {
        id: task_id.clone(),
        status: TaskStatus {
            state: "completed".to_string(),
        },
        artifacts,
    };

    as_json(A2AResponse::ok(
        id,
        serde_json::to_value(task).unwrap_or(Value::Null),
    ))
}

fn handle_tasks_get(id: Option<Value>, params: Value) -> AxumJson<Value> {
    let task_id = params.get("id").and_then(|v| v.as_str()).unwrap_or("");
    match TASK_STORAGE.get(task_id) {
        Some(task) => as_json(A2AResponse::ok(
            id,
            serde_json::to_value(task).unwrap_or(Value::Null),
        )),
        None => as_json(A2AResponse::error(
            id,
            TASK_NOT_FOUND,
            &format!("task '{}' not found", task_id),
        )),
    }
}

fn handle_tasks_cancel(id: Option<Value>, params: Value) -> AxumJson<Value> {
    let task_id = params.get("id").and_then(|v| v.as_str()).unwrap_or("");
    if TASK_STORAGE.update_state(task_id, "canceled") {
        let task = TASK_STORAGE.get(task_id).unwrap();
        as_json(A2AResponse::ok(
            id,
            serde_json::to_value(task).unwrap_or(Value::Null),
        ))
    } else {
        as_json(A2AResponse::error(
            id,
            TASK_NOT_FOUND,
            &format!("task '{}' not found", task_id),
        ))
    }
}

// ── Helpers ────────────────────────────────────────────────────────────────

fn as_json(resp: A2AResponse) -> AxumJson<Value> {
    AxumJson(serde_json::to_value(resp).unwrap_or(Value::Null))
}
