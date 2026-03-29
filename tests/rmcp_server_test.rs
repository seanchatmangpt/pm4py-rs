/// Chicago TDD: rmcp SDK integration tests for pm4py-rust.
///
/// Five test groups:
///   1. Semconv constants (no feature gate — regression guard after Weaver generate)
///   2. Server info (feature = "mcp-server")
///   3. Tool list via duplex transport (feature = "mcp-server")
///   4. Tool execution via duplex transport (feature = "mcp-server")
///   5. OTel span constants (feature = "mcp-server")
///
/// RED → GREEN sequence: groups 2-5 fail until rmcp_server.rs is implemented.
// ── Group 1: Semconv constants ────────────────────────────────────────────────
// These tests have NO feature gate — they run in every cargo test invocation.
// They act as schema conformance regression guards: if the weaver YAML is
// changed to remove the constant, the compile error surfaces here first.
use pm4py::semconv::mcp_attributes::mcp_connection_transport;
use pm4py::semconv::mcp_attributes::mcp_transport_type;
use pm4py::semconv::mcp_span_names::MCP_SESSION_CREATE_SPAN;

#[test]
fn mcp_connection_transport_has_streamable_http_value() {
    assert_eq!(mcp_connection_transport::STREAMABLE_HTTP, "streamable-http");
}

#[test]
fn mcp_transport_type_has_streamable_http_value() {
    assert_eq!(mcp_transport_type::STREAMABLE_HTTP, "streamable-http");
}

#[test]
fn mcp_session_create_span_constant_is_correct() {
    assert_eq!(MCP_SESSION_CREATE_SPAN, "mcp.session.create");
}

// ── Groups 2-5: rmcp SDK (requires mcp-server feature) ──────────────────────

#[cfg(feature = "mcp-server")]
mod rmcp_tests {
    use pm4py::mcp::rmcp_server::Pm4pyMcpServer;
    use rmcp::model::{CallToolRequestParams, ClientInfo};
    use rmcp::service::RunningService;
    use rmcp::{ClientHandler, RoleClient, ServiceExt};
    use serde_json::json;

    #[derive(Default)]
    struct TestClient;

    impl ClientHandler for TestClient {
        fn get_info(&self) -> ClientInfo {
            ClientInfo::default()
        }
    }

    async fn make_server_client() -> anyhow::Result<RunningService<RoleClient, TestClient>> {
        let (s_transport, c_transport) = tokio::io::duplex(65_536);
        tokio::spawn(async move {
            if let Ok(svc) = Pm4pyMcpServer::new().serve(s_transport).await {
                let _ = svc.waiting().await;
            }
        });
        Ok(TestClient.serve(c_transport).await?)
    }

    // ── Group 2: Server construction ─────────────────────────────────────────

    #[test]
    fn rmcp_server_can_be_constructed() {
        let _s = Pm4pyMcpServer::new();
    }

    #[tokio::test]
    async fn rmcp_server_info_has_correct_name() -> anyhow::Result<()> {
        let mut svc = make_server_client().await?;
        let name = svc.peer_info().unwrap().server_info.name.clone();
        assert_eq!(name, "pm4py-rust");
        svc.close().await.ok();
        Ok(())
    }

    #[tokio::test]
    async fn rmcp_server_info_has_tools_capability() -> anyhow::Result<()> {
        let mut svc = make_server_client().await?;
        let has_tools = svc.peer_info().unwrap().capabilities.tools.is_some();
        assert!(has_tools);
        svc.close().await.ok();
        Ok(())
    }

    // ── Group 3: Tool list ────────────────────────────────────────────────────

    #[tokio::test]
    async fn rmcp_tools_list_returns_12_tools() -> anyhow::Result<()> {
        let mut svc = make_server_client().await?;
        let tools = svc.list_all_tools().await?;
        assert_eq!(
            tools.len(),
            12,
            "Expected 12 tools, got: {:?}",
            tools.iter().map(|t| t.name.as_ref()).collect::<Vec<_>>()
        );
        svc.close().await.ok();
        Ok(())
    }

    #[tokio::test]
    async fn rmcp_tools_all_have_descriptions() -> anyhow::Result<()> {
        let mut svc = make_server_client().await?;
        let tools = svc.list_all_tools().await?;
        for tool in &tools {
            assert!(
                !tool.description.as_deref().unwrap_or("").is_empty(),
                "Tool '{}' has no description",
                tool.name
            );
        }
        svc.close().await.ok();
        Ok(())
    }

    #[tokio::test]
    async fn rmcp_tools_contain_expected_names() -> anyhow::Result<()> {
        let mut svc = make_server_client().await?;
        let tools = svc.list_all_tools().await?;
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
            "pm4py_decision_mine",
            "pm4py_predict_remaining_time",
            "pm4py_ocel_ingest",
        ];
        for name in expected {
            assert!(
                tools.iter().any(|t| t.name.as_ref() == name),
                "Tool '{}' not found in list",
                name
            );
        }
        svc.close().await.ok();
        Ok(())
    }

    // ── Group 4: Tool execution ───────────────────────────────────────────────

    fn minimal_event_log() -> serde_json::Value {
        json!({
            "attributes": {},
            "traces": [{
                "id": "case-001",
                "attributes": {},
                "events": [
                    { "activity": "A", "timestamp": "2024-01-01T10:00:00Z", "resource": null, "attributes": {} },
                    { "activity": "B", "timestamp": "2024-01-01T11:00:00Z", "resource": null, "attributes": {} }
                ]
            }]
        })
    }

    #[tokio::test]
    async fn rmcp_discover_alpha_returns_petri_net() -> anyhow::Result<()> {
        let mut svc = make_server_client().await?;
        let result = svc
            .call_tool(
                CallToolRequestParams::new("pm4py_discover_alpha").with_arguments(
                    json!({ "event_log": minimal_event_log() })
                        .as_object()
                        .unwrap()
                        .clone(),
                ),
            )
            .await?;
        assert!(!result.content.is_empty());
        let text = &result.content[0].raw.as_text().unwrap().text;
        assert!(
            text.contains("alpha_miner"),
            "Expected algorithm in result, got: {}",
            text
        );
        svc.close().await.ok();
        Ok(())
    }

    #[tokio::test]
    async fn rmcp_statistics_returns_trace_count() -> anyhow::Result<()> {
        let mut svc = make_server_client().await?;
        let result = svc
            .call_tool(
                CallToolRequestParams::new("pm4py_statistics").with_arguments(
                    json!({ "event_log": minimal_event_log() })
                        .as_object()
                        .unwrap()
                        .clone(),
                ),
            )
            .await?;
        let text = &result.content[0].raw.as_text().unwrap().text;
        assert!(
            text.contains("trace_count"),
            "Expected trace_count in result: {}",
            text
        );
        svc.close().await.ok();
        Ok(())
    }

    #[tokio::test]
    async fn rmcp_unknown_tool_returns_error() -> anyhow::Result<()> {
        let mut svc = make_server_client().await?;
        let result = svc
            .call_tool(CallToolRequestParams::new("nonexistent_tool"))
            .await;
        // Unknown tools should return an error (either Err or isError=true)
        match result {
            Err(_) => {}
            Ok(r) => assert!(
                r.is_error.unwrap_or(false),
                "Expected isError=true for unknown tool"
            ),
        }
        svc.close().await.ok();
        Ok(())
    }

    #[tokio::test]
    async fn rmcp_query_fallback_without_credentials() -> anyhow::Result<()> {
        let mut svc = make_server_client().await?;
        let result = svc
            .call_tool(
                CallToolRequestParams::new("pm4py_query").with_arguments(
                    json!({ "query": "What are the bottlenecks?" })
                        .as_object()
                        .unwrap()
                        .clone(),
                ),
            )
            .await?;
        let text = &result.content[0].raw.as_text().unwrap().text;
        // Falls back to "Set GROQ_API_KEY..." message
        assert!(
            text.contains("query"),
            "Expected query in fallback result: {}",
            text
        );
        svc.close().await.ok();
        Ok(())
    }

    // ── Group 5: OTel span constants (schema conformance) ────────────────────

    #[test]
    fn mcp_tool_execute_span_constant_is_correct() {
        use pm4py::semconv::mcp_span_names::MCP_TOOL_EXECUTE_SPAN;
        assert_eq!(MCP_TOOL_EXECUTE_SPAN, "mcp.tool_execute");
    }

    #[test]
    fn mcp_session_create_span_is_defined() {
        use pm4py::semconv::mcp_span_names::MCP_SESSION_CREATE_SPAN;
        assert_eq!(MCP_SESSION_CREATE_SPAN, "mcp.session.create");
    }
}
