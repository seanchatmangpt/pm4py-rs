/// Chicago TDD RED tests — A2A server contract tests for pm4py-rust.
///
/// These tests are in the RED phase until the `src/a2a/` module is implemented (Step 5).
/// `cargo test --features a2a` will produce compile errors until the a2a module exists.
///
/// Test groups:
///   1. Agent card — skills, protocol version, well-formedness
///   2. Task lifecycle — create, get, unknown id
///   3. Skill dispatch — process discovery, statistics
///   4. Protocol errors — invalid method, missing params, bad version

#[cfg(feature = "a2a")]
mod a2a_agent_card_tests {
    use axum::extract::Json as AxumJson;
    use serde_json::json;

    #[tokio::test]
    async fn test_a2a_agent_card_has_10_skills() {
        // RED: pm4py::a2a::handler::a2a_agent_card does not exist yet
        let AxumJson(card) = pm4py::a2a::handler::a2a_agent_card().await;
        let skills = card
            .get("skills")
            .and_then(|s| s.as_array())
            .expect("agent card must have skills array");
        assert_eq!(skills.len(), 10, "pm4py-rust exposes exactly 10 A2A skills");
    }

    #[tokio::test]
    async fn test_a2a_protocol_version_in_agent_card() {
        let AxumJson(card) = pm4py::a2a::handler::a2a_agent_card().await;
        let version = card
            .get("version")
            .and_then(|v| v.as_str())
            .expect("agent card must have version");
        assert!(!version.is_empty(), "version must be non-empty");
    }

    #[tokio::test]
    async fn test_a2a_agent_card_each_skill_is_well_formed() {
        let AxumJson(card) = pm4py::a2a::handler::a2a_agent_card().await;
        let skills = card["skills"].as_array().unwrap();
        for skill in skills {
            assert!(
                skill.get("id").and_then(|v| v.as_str()).is_some(),
                "skill must have id: {:?}",
                skill
            );
            assert!(
                skill.get("name").and_then(|v| v.as_str()).is_some(),
                "skill must have name: {:?}",
                skill
            );
            assert!(
                skill.get("description").and_then(|v| v.as_str()).is_some(),
                "skill must have description: {:?}",
                skill
            );
        }
    }

    #[tokio::test]
    async fn test_a2a_agent_card_skill_ids_match_pm4py_tools() {
        let AxumJson(card) = pm4py::a2a::handler::a2a_agent_card().await;
        let skills = card["skills"].as_array().unwrap();
        let ids: Vec<&str> = skills.iter().filter_map(|s| s["id"].as_str()).collect();

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
        ];
        for expected_id in &expected {
            assert!(
                ids.contains(expected_id),
                "skill '{}' missing from agent card",
                expected_id
            );
        }
    }
}

#[cfg(feature = "a2a")]
mod a2a_task_lifecycle_tests {
    use axum::extract::Json as AxumJson;
    use serde_json::json;

    fn make_tasks_send(task_id: &str, tool: &str) -> serde_json::Value {
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tasks/send",
            "params": {
                "id": task_id,
                "message": {
                    "role": "user",
                    "parts": [
                        {
                            "type": "data",
                            "data": {
                                "tool": tool,
                                "args": {
                                    "event_log": {
                                        "attributes": {},
                                        "traces": []
                                    }
                                }
                            }
                        }
                    ]
                }
            }
        })
    }

    #[tokio::test]
    async fn test_a2a_task_create_returns_submitted_state() {
        // RED: pm4py::a2a::handler::a2a_handler does not exist yet
        let req = make_tasks_send("task-001", "pm4py_statistics");
        let AxumJson(resp) = pm4py::a2a::handler::a2a_handler(AxumJson(req)).await;
        let state = resp["result"]["status"]["state"]
            .as_str()
            .unwrap_or_default();
        // Task may complete synchronously; accepted states are submitted, working, or completed
        assert!(
            matches!(state, "submitted" | "working" | "completed"),
            "unexpected state: {}",
            state
        );
    }

    #[tokio::test]
    async fn test_a2a_task_get_returns_existing_task() {
        use serde_json::Value;
        let create_req = make_tasks_send("task-get-001", "pm4py_statistics");
        pm4py::a2a::handler::a2a_handler(AxumJson(create_req)).await;

        let get_req = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tasks/get",
            "params": { "id": "task-get-001" }
        });
        let AxumJson(resp) = pm4py::a2a::handler::a2a_handler(AxumJson(get_req)).await;
        let task_id = resp["result"]["id"].as_str().unwrap_or_default();
        assert_eq!(task_id, "task-get-001");
    }

    #[tokio::test]
    async fn test_a2a_task_get_unknown_id_returns_task_not_found() {
        let get_req = json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tasks/get",
            "params": { "id": "does-not-exist-xyz" }
        });
        let AxumJson(resp) = pm4py::a2a::handler::a2a_handler(AxumJson(get_req)).await;
        let code = resp["error"]["code"].as_i64().unwrap_or(0);
        assert_eq!(
            code, -32001,
            "unknown task must return TASK_NOT_FOUND (-32001)"
        );
    }
}

#[cfg(feature = "a2a")]
mod a2a_skill_dispatch_tests {
    use axum::extract::Json as AxumJson;
    use serde_json::json;

    fn minimal_log_with_events() -> serde_json::Value {
        json!({
            "attributes": {},
            "traces": [
                {
                    "id": "case-001",
                    "attributes": {},
                    "events": [
                        {
                            "activity": "A",
                            "timestamp": "2024-01-01T10:00:00Z",
                            "resource": null,
                            "attributes": {}
                        },
                        {
                            "activity": "B",
                            "timestamp": "2024-01-01T10:30:00Z",
                            "resource": null,
                            "attributes": {}
                        }
                    ]
                }
            ]
        })
    }

    #[tokio::test]
    async fn test_a2a_skill_invoke_process_discovery_returns_petri_net() {
        let req = json!({
            "jsonrpc": "2.0",
            "id": 10,
            "method": "tasks/send",
            "params": {
                "id": "skill-disc-001",
                "message": {
                    "role": "user",
                    "parts": [
                        {
                            "type": "data",
                            "data": {
                                "tool": "pm4py_discover_alpha",
                                "args": { "event_log": minimal_log_with_events() }
                            }
                        }
                    ]
                }
            }
        });
        let AxumJson(resp) = pm4py::a2a::handler::a2a_handler(AxumJson(req)).await;
        // Completed task should have artifacts
        let state = resp["result"]["status"]["state"]
            .as_str()
            .unwrap_or_default();
        assert_eq!(state, "completed", "discovery task must complete");
        let artifacts = resp["result"]["artifacts"]
            .as_array()
            .expect("must have artifacts");
        assert!(!artifacts.is_empty(), "must produce at least one artifact");
    }

    #[tokio::test]
    async fn test_a2a_skill_invoke_statistics_returns_counts() {
        let req = json!({
            "jsonrpc": "2.0",
            "id": 11,
            "method": "tasks/send",
            "params": {
                "id": "skill-stats-001",
                "message": {
                    "role": "user",
                    "parts": [
                        {
                            "type": "data",
                            "data": {
                                "tool": "pm4py_statistics",
                                "args": { "event_log": minimal_log_with_events() }
                            }
                        }
                    ]
                }
            }
        });
        let AxumJson(resp) = pm4py::a2a::handler::a2a_handler(AxumJson(req)).await;
        let state = resp["result"]["status"]["state"]
            .as_str()
            .unwrap_or_default();
        assert_eq!(state, "completed");
        let artifacts = resp["result"]["artifacts"]
            .as_array()
            .expect("must have artifacts");
        assert!(!artifacts.is_empty());
    }
}

#[cfg(feature = "a2a")]
mod a2a_protocol_error_tests {
    use axum::extract::Json as AxumJson;
    use serde_json::json;

    #[tokio::test]
    async fn test_a2a_json_rpc_invalid_method_returns_method_not_found() {
        let req = json!({
            "jsonrpc": "2.0",
            "id": 20,
            "method": "unknown/method",
            "params": {}
        });
        let AxumJson(resp) = pm4py::a2a::handler::a2a_handler(AxumJson(req)).await;
        let code = resp["error"]["code"].as_i64().unwrap_or(0);
        assert_eq!(code, -32601, "unknown method must return METHOD_NOT_FOUND");
    }

    #[tokio::test]
    async fn test_a2a_tasks_send_missing_message_returns_invalid_params() {
        let req = json!({
            "jsonrpc": "2.0",
            "id": 21,
            "method": "tasks/send",
            "params": {
                "id": "bad-task"
                // missing "message"
            }
        });
        let AxumJson(resp) = pm4py::a2a::handler::a2a_handler(AxumJson(req)).await;
        let code = resp["error"]["code"].as_i64().unwrap_or(0);
        assert_eq!(code, -32602, "missing message must return INVALID_PARAMS");
    }

    #[tokio::test]
    async fn test_a2a_bad_jsonrpc_version_returns_invalid_request() {
        let req = json!({
            "jsonrpc": "1.0",
            "id": 22,
            "method": "tasks/send",
            "params": {}
        });
        let AxumJson(resp) = pm4py::a2a::handler::a2a_handler(AxumJson(req)).await;
        let code = resp["error"]["code"].as_i64().unwrap_or(0);
        assert_eq!(
            code, -32600,
            "wrong jsonrpc version must return INVALID_REQUEST"
        );
    }
}
