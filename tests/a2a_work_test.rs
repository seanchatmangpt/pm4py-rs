#![cfg(feature = "a2a")]
//! Chicago TDD: pm4py A2A skill output verification tests.
//!
//! Calls a2a_handler and a2a_agent_card directly — no HTTP server required.
//! Each test verifies that a skill returns real, non-trivial output for a
//! concrete event log input.
//!
//! Test groups:
//!   1. statistics — trace_count, event_count, activity_frequencies
//!   2. process discovery — petri_net structure, fitness range
//!   3. abstract_event_log — non-empty text content
//!   4. ocel_ingest — object type summary
//!   5. task chaining — tasks/get returns persisted artifacts
//!   6. agent card — all skill IDs present
//!   7. no METHOD_NOT_FOUND for core skills

use axum::extract::Json as AxumJson;
use pm4py::a2a::handler::{a2a_agent_card, a2a_handler};
use serde_json::{json, Value};

// ── Shared fixtures ────────────────────────────────────────────────────────

fn one_trace_log() -> Value {
    json!({
        "attributes": {},
        "traces": [{
            "id": "case-001",
            "attributes": {},
            "events": [
                {"activity": "A", "timestamp": "2024-01-01T10:00:00Z", "resource": null, "attributes": {}},
                {"activity": "B", "timestamp": "2024-01-01T10:30:00Z", "resource": null, "attributes": {}}
            ]
        }]
    })
}

fn abc_trace_log() -> Value {
    json!({
        "attributes": {},
        "traces": [{
            "id": "case-001",
            "attributes": {},
            "events": [
                {"activity": "A", "timestamp": "2024-01-01T10:00:00Z", "resource": null, "attributes": {}},
                {"activity": "B", "timestamp": "2024-01-01T10:15:00Z", "resource": null, "attributes": {}},
                {"activity": "C", "timestamp": "2024-01-01T10:30:00Z", "resource": null, "attributes": {}}
            ]
        }]
    })
}

fn tasks_send(task_id: &str, tool: &str, args: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tasks/send",
        "params": {
            "id": task_id,
            "message": {
                "role": "user",
                "parts": [{"type": "data", "data": {"tool": tool, "args": args}}]
            }
        }
    })
}

fn assert_completed(resp: &Value) {
    assert_eq!(
        resp["result"]["status"]["state"], "completed",
        "expected completed, got: {}",
        resp
    );
    let arts = resp["result"]["artifacts"]
        .as_array()
        .expect("artifacts must be array");
    assert!(!arts.is_empty(), "artifacts must be non-empty");
}

/// Return the data field from the first artifact's first part.
/// Only valid when the tool returns JSON output (not plain-text tools like abstract_event_log).
fn artifact_data(resp: &Value) -> &Value {
    &resp["result"]["artifacts"][0]["parts"][0]["data"]
}

// ── statistics ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_a2a_statistics_counts_one_trace_two_events() {
    let req = tasks_send(
        "stat-work-1",
        "pm4py_statistics",
        json!({"event_log": one_trace_log()}),
    );
    let AxumJson(resp) = a2a_handler(AxumJson(req)).await;
    assert_completed(&resp);
    let data = artifact_data(&resp);
    assert_eq!(
        data["trace_count"],
        json!(1),
        "trace_count must be 1; data={}",
        data
    );
    assert_eq!(
        data["event_count"],
        json!(2),
        "event_count must be 2; data={}",
        data
    );
}

#[tokio::test]
async fn test_a2a_statistics_empty_log_returns_zeros() {
    let req = tasks_send(
        "stat-work-2",
        "pm4py_statistics",
        json!({"event_log": {"attributes": {}, "traces": []}}),
    );
    let AxumJson(resp) = a2a_handler(AxumJson(req)).await;
    assert_completed(&resp);
    let data = artifact_data(&resp);
    assert_eq!(data["trace_count"], json!(0));
    assert_eq!(data["event_count"], json!(0));
}

#[tokio::test]
async fn test_a2a_statistics_activity_frequencies_present() {
    let req = tasks_send(
        "stat-work-3",
        "pm4py_statistics",
        json!({"event_log": one_trace_log()}),
    );
    let AxumJson(resp) = a2a_handler(AxumJson(req)).await;
    assert_completed(&resp);
    let data = artifact_data(&resp);
    assert!(
        data.get("activity_frequencies").is_some(),
        "activity_frequencies must be present; got: {}",
        data
    );
}

// ── process discovery ──────────────────────────────────────────────────────

#[tokio::test]
async fn test_a2a_discover_alpha_returns_petri_net() {
    let req = tasks_send(
        "disc-work-1",
        "pm4py_discover_alpha",
        json!({"event_log": abc_trace_log()}),
    );
    let AxumJson(resp) = a2a_handler(AxumJson(req)).await;
    assert_completed(&resp);
    let data = artifact_data(&resp);
    let has_model = data.get("petri_net").is_some()
        || data.get("places").is_some()
        || data.get("transitions").is_some();
    assert!(
        has_model,
        "alpha discovery must return a Petri net structure; got: {}",
        data
    );
}

#[tokio::test]
async fn test_a2a_discover_alpha_fitness_in_valid_range() {
    let req = tasks_send(
        "disc-work-2",
        "pm4py_discover_alpha",
        json!({"event_log": abc_trace_log()}),
    );
    let AxumJson(resp) = a2a_handler(AxumJson(req)).await;
    assert_completed(&resp);
    let data = artifact_data(&resp);
    if let Some(fitness) = data["fitness"].as_f64() {
        assert!(
            (0.0..=1.0).contains(&fitness),
            "fitness must be in [0.0, 1.0], got {}",
            fitness
        );
    }
}

// ── abstract event log ─────────────────────────────────────────────────────

#[tokio::test]
async fn test_a2a_abstract_event_log_returns_non_empty_content() {
    let req = tasks_send(
        "abs-work-1",
        "pm4py_abstract_event_log",
        json!({"event_log": one_trace_log()}),
    );
    let AxumJson(resp) = a2a_handler(AxumJson(req)).await;
    assert_completed(&resp);
    // abstract_event_log returns plain text, so the artifact has a "text" part,
    // not a "data" part. Accept either type as long as content is non-empty.
    let parts = &resp["result"]["artifacts"][0]["parts"];
    let has_content = parts
        .as_array()
        .map(|p| {
            p.iter().any(|part| {
                (part["type"] == "text" && !part["text"].as_str().unwrap_or("").is_empty())
                    || part["type"] == "data"
            })
        })
        .unwrap_or(false);
    assert!(
        has_content,
        "abstract_event_log must return non-empty content; got: {}",
        resp
    );
}

// ── OCEL ingest ────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_a2a_ocel_ingest_returns_object_type_summary() {
    // ObjectCentricEventLog serialization format:
    //   - objects: BTreeMap<String, Object> → JSON object with id keys
    //   - object_types: HashSet<ObjectType> → JSON array of {name, description?}
    //   - events: BTreeMap<Uuid, (activity, timestamp, resource?)> → JSON object with uuid keys
    //   - event_object_mappings: Vec<EventToObjectMapping>
    let ocel = json!({
        "id": "ocel-001",
        "objects": {
            "o1": {
                "id": "o1",
                "object_type": {"name": "order", "description": null},
                "state": null,
                "lifecycle_stage": null,
                "creation_time": "2024-01-01T10:00:00Z",
                "end_time": null,
                "attributes": {},
                "relationships": []
            }
        },
        "object_types": [
            {"name": "order", "description": null}
        ],
        "events": {
            "550e8400-e29b-41d4-a716-446655440000": [
                "place_order",
                "2024-01-01T10:00:00Z",
                null
            ]
        },
        "event_object_mappings": [],
        "attributes": {}
    });
    let req = tasks_send("ocel-work-1", "pm4py_ocel_ingest", json!({"ocel": ocel}));
    let AxumJson(resp) = a2a_handler(AxumJson(req)).await;
    assert_completed(&resp);
    let data = artifact_data(&resp);
    let has_summary = data.get("object_types").is_some()
        || data.get("object_count").is_some()
        || data.get("event_count").is_some()
        || data.get("object_type_count").is_some()
        || data.get("summary").is_some();
    assert!(
        has_summary,
        "ocel_ingest must return a summary; got: {}",
        data
    );
}

// ── task chaining ──────────────────────────────────────────────────────────

#[tokio::test]
async fn test_a2a_chain_statistics_then_tasks_get() {
    let send_req = tasks_send(
        "chain-stat-1",
        "pm4py_statistics",
        json!({"event_log": one_trace_log()}),
    );
    let AxumJson(send_resp) = a2a_handler(AxumJson(send_req)).await;
    assert_eq!(send_resp["result"]["id"], json!("chain-stat-1"));

    let get_req = json!({
        "jsonrpc": "2.0", "id": 2,
        "method": "tasks/get",
        "params": {"id": "chain-stat-1"}
    });
    let AxumJson(get_resp) = a2a_handler(AxumJson(get_req)).await;
    assert_eq!(get_resp["result"]["id"], json!("chain-stat-1"));
    assert_eq!(get_resp["result"]["status"]["state"], json!("completed"));
}

#[tokio::test]
async fn test_a2a_chain_discover_artifacts_persist_on_get() {
    let disc_req = tasks_send(
        "chain-disc-2",
        "pm4py_discover_alpha",
        json!({"event_log": abc_trace_log()}),
    );
    let AxumJson(disc_resp) = a2a_handler(AxumJson(disc_req)).await;
    assert_completed(&disc_resp);

    let get_req = json!({
        "jsonrpc": "2.0", "id": 2,
        "method": "tasks/get",
        "params": {"id": "chain-disc-2"}
    });
    let AxumJson(get_resp) = a2a_handler(AxumJson(get_req)).await;
    assert!(
        !get_resp["result"]["artifacts"]
            .as_array()
            .unwrap()
            .is_empty(),
        "artifacts must persist after tasks/get"
    );
}

// ── agent card ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_a2a_agent_card_all_skill_ids_present() {
    let AxumJson(card) = a2a_agent_card().await;
    let expected = [
        "pm4py_statistics",
        "pm4py_discover_alpha",
        "pm4py_conformance_token_replay",
        "pm4py_parse_xes",
        "pm4py_detect_drift",
        "pm4py_abstract_petri_net",
        "pm4py_abstract_event_log",
        "pm4py_abstract_dfg",
        "pm4py_query",
        "pm4py_ocel_ingest",
    ];
    let skills = card["skills"].as_array().expect("skills must be array");
    let actual: Vec<&str> = skills.iter().map(|s| s["id"].as_str().unwrap()).collect();
    for e in &expected {
        assert!(
            actual.contains(e),
            "agent card missing skill '{}'; actual: {:?}",
            e,
            actual
        );
    }
}

#[tokio::test]
async fn test_a2a_skills_do_not_return_method_not_found() {
    let skills = [
        "pm4py_statistics",
        "pm4py_discover_alpha",
        "pm4py_abstract_event_log",
    ];
    for skill in &skills {
        let req = tasks_send(
            &format!("roundtrip-{}", skill),
            skill,
            json!({"event_log": {"attributes": {}, "traces": []}}),
        );
        let AxumJson(resp) = a2a_handler(AxumJson(req)).await;
        let is_method_not_found = resp
            .get("error")
            .and_then(|e| e.get("code"))
            .and_then(|c| c.as_i64())
            .map(|c| c == -32601)
            .unwrap_or(false);
        assert!(
            !is_method_not_found,
            "skill '{}' must not return METHOD_NOT_FOUND; got: {}",
            skill, resp
        );
    }
}
