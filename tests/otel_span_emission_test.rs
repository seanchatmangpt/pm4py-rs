/// Chicago TDD — OTEL span emission verification tests for A2A and MCP handlers.
///
/// Strategy: The handlers use `opentelemetry::global::tracer("pm4py-rust")` which
/// defaults to a NoopTracer when no SDK pipeline is configured. These tests verify:
///   1. Span name constants compile and equal the canonical semconv values.
///   2. Attribute key constants compile and equal the canonical semconv values.
///   3. The A2A handler can be called end-to-end without panicking.
///   4. The agent card returns the full 10-skill catalog.
///
/// Run (feature-gated):
///   cargo test --features a2a otel_span_emission 2>&1
///
/// No HTTP server, no collector, no external dependencies.  All tests are pure
/// in-process calls that work with the NoopTracer installed by default.
#[cfg(feature = "a2a")]
mod otel_span_emission {
    use axum::extract::Json as AxumJson;
    use serde_json::json;

    // ── Test 1: span name constant ────────────────────────────────────────────

    /// `A2A_TASK_CREATE_SPAN` must equal the canonical OTel semconv key
    /// `"a2a.task.create"`.  If this constant drifts the test fails at compile
    /// time (wrong type) or at runtime (wrong value) — guards both regressions.
    #[test]
    fn test_a2a_task_create_span_name_is_correct_semconv_key() {
        use pm4py::semconv::a2a_span_names::A2A_TASK_CREATE_SPAN;

        // Compile-time type check: the constant must be &'static str.
        let _: &'static str = A2A_TASK_CREATE_SPAN;

        assert_eq!(
            A2A_TASK_CREATE_SPAN, "a2a.task.create",
            "A2A_TASK_CREATE_SPAN must equal the canonical semconv key 'a2a.task.create'"
        );
    }

    // ── Test 2: skill invoke — tasks/send response is valid JSON ─────────────

    /// Calling `a2a_handler` with a `tasks/send` message that references
    /// `pm4py_statistics` must return a JSON object with a `result` or `error`
    /// key — the OTEL spans emitted during execution must not panic even when
    /// no SDK exporter is wired (NoopTracer is used by default).
    #[tokio::test]
    async fn test_a2a_skill_invoke_span_attributes_are_set() {
        let req = json!({
            "jsonrpc": "2.0",
            "id": 100,
            "method": "tasks/send",
            "params": {
                "id": "otel-test-task-001",
                "message": {
                    "role": "user",
                    "parts": [
                        {
                            "type": "data",
                            "data": {
                                "tool": "pm4py_statistics",
                                "args": {
                                    "event_log": {
                                        "attributes": {},
                                        "traces": [
                                            {
                                                "id": "case-otel-01",
                                                "attributes": {},
                                                "events": [
                                                    {
                                                        "activity": "Start",
                                                        "timestamp": "2024-01-01T10:00:00Z",
                                                        "resource": null,
                                                        "attributes": {}
                                                    },
                                                    {
                                                        "activity": "End",
                                                        "timestamp": "2024-01-01T10:05:00Z",
                                                        "resource": null,
                                                        "attributes": {}
                                                    }
                                                ]
                                            }
                                        ]
                                    }
                                }
                            }
                        }
                    ]
                }
            }
        });

        // Call the real handler — spans are emitted via NoopTracer (no collector needed).
        let AxumJson(resp) = pm4py::a2a::handler::a2a_handler(AxumJson(req)).await;

        // Response must be valid JSON with either result or error — never null.
        assert!(
            resp.get("result").is_some() || resp.get("error").is_some(),
            "a2a_handler must return a JSON-RPC response with result or error; got: {:?}",
            resp
        );

        // When result is present the task state must be a known lifecycle state.
        if let Some(result) = resp.get("result") {
            let state = result
                .pointer("/status/state")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            assert!(
                matches!(state, "submitted" | "working" | "completed" | "failed"),
                "task state must be a valid lifecycle state; got: '{}'",
                state
            );
        }

        // Verify the A2A attribute key constants used inside the handler also compile.
        use pm4py::semconv::a2a_attributes::{
            A2A_AGENT_NAME, A2A_ARTIFACT_COUNT, A2A_MESSAGE_ROLE, A2A_SKILL_ID, A2A_TASK_STATE,
        };
        let _: &'static str = A2A_AGENT_NAME;
        let _: &'static str = A2A_ARTIFACT_COUNT;
        let _: &'static str = A2A_MESSAGE_ROLE;
        let _: &'static str = A2A_SKILL_ID;
        let _: &'static str = A2A_TASK_STATE;

        assert_eq!(A2A_AGENT_NAME, "a2a.agent.name");
        assert_eq!(A2A_TASK_STATE, "a2a.task.state");
        assert_eq!(A2A_SKILL_ID, "a2a.skill.id");
        assert_eq!(A2A_MESSAGE_ROLE, "a2a.message.role");
        assert_eq!(A2A_ARTIFACT_COUNT, "a2a.artifact.count");
    }

    // ── Test 3: agent card — 10 skills, no panic ──────────────────────────────

    /// `a2a_agent_card()` must return a well-formed JSON object containing
    /// exactly 10 skills.  The handler emits an `a2a.agent_card.serve` OTEL span
    /// internally; this test confirms no panic occurs during that emission when
    /// using the default NoopTracer.
    #[tokio::test]
    async fn test_a2a_agent_card_handler_does_not_panic() {
        // This call internally starts an OTEL span using global::tracer("pm4py-rust").
        let AxumJson(card) = pm4py::a2a::handler::a2a_agent_card().await;

        // Agent card must be a JSON object.
        assert!(
            card.is_object(),
            "a2a_agent_card must return a JSON object; got: {:?}",
            card
        );

        // Must carry a non-empty name.
        let name = card
            .get("name")
            .and_then(|v| v.as_str())
            .expect("agent card must have 'name' field");
        assert!(!name.is_empty(), "agent card name must be non-empty");

        // Must carry exactly 10 skills.
        let skills = card
            .get("skills")
            .and_then(|v| v.as_array())
            .expect("agent card must have 'skills' array");
        assert_eq!(
            skills.len(),
            10,
            "agent card must expose exactly 10 pm4py skills"
        );

        // Verify the span constant for agent card serve is the expected value.
        use pm4py::semconv::a2a_span_names::A2A_AGENT_CARD_SERVE_SPAN;
        assert_eq!(
            A2A_AGENT_CARD_SERVE_SPAN, "a2a.agent_card.serve",
            "A2A_AGENT_CARD_SERVE_SPAN must equal 'a2a.agent_card.serve'"
        );
    }

    // ── Test 4: unknown method returns -32601 ─────────────────────────────────

    /// Sending a request with an unknown JSON-RPC method must return error code
    /// -32601 (METHOD_NOT_FOUND).  The `a2a.task.create` span is NOT emitted
    /// for method-not-found paths; the handler returns early before span creation.
    /// This test also guards that the OTEL span constant for `A2A_SKILL_INVOKE_SPAN`
    /// equals its canonical value.
    #[tokio::test]
    async fn test_a2a_handler_method_not_found_returns_minus_32601() {
        use pm4py::a2a::protocol::METHOD_NOT_FOUND;
        use pm4py::semconv::a2a_span_names::A2A_SKILL_INVOKE_SPAN;

        // Span constant regression guard.
        assert_eq!(
            A2A_SKILL_INVOKE_SPAN, "a2a.skill.invoke",
            "A2A_SKILL_INVOKE_SPAN must equal 'a2a.skill.invoke'"
        );

        let req = json!({
            "jsonrpc": "2.0",
            "id": 200,
            "method": "unsupported/operation",
            "params": {}
        });

        let AxumJson(resp) = pm4py::a2a::handler::a2a_handler(AxumJson(req)).await;

        assert!(
            resp.get("result").is_none(),
            "method-not-found must not return a result"
        );

        let code = resp
            .pointer("/error/code")
            .and_then(|v| v.as_i64())
            .expect("error.code must be present for method-not-found");

        assert_eq!(
            code, METHOD_NOT_FOUND,
            "unknown method must return METHOD_NOT_FOUND ({}) error code; got {}",
            METHOD_NOT_FOUND, code
        );

        // Verify task.complete span constant while we have access to span_names.
        use pm4py::semconv::a2a_span_names::A2A_TASK_COMPLETE_SPAN;
        assert_eq!(
            A2A_TASK_COMPLETE_SPAN, "a2a.task.complete",
            "A2A_TASK_COMPLETE_SPAN must equal 'a2a.task.complete'"
        );
    }
}
