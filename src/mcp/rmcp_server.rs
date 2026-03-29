/// rmcp SDK adapter for pm4py-rust process mining tools.
///
/// Wraps the 12-tool executor via the official Rust MCP SDK (rmcp 1.3.0).
/// Exposes a `Pm4pyMcpServer` that implements `ServerHandler` and routes
/// calls through `#[tool_router]` / `#[tool]` proc-macros.
///
/// Transport endpoints:
/// - `/mcp/v2` — StreamableHttpService (SSE-capable, mounted in businessos_api.rs)
/// - `mcp_stdio` binary — stdio transport for Claude Desktop
///
/// The hand-rolled `/mcp` endpoint (handler.rs) is kept unchanged for backward compat.

#[cfg(feature = "mcp-server")]
pub mod inner {
    use rmcp::{
        handler::server::{router::tool::ToolRouter, wrapper::Parameters},
        model::{Implementation, ServerCapabilities, ServerInfo},
        schemars, tool, tool_handler, tool_router, ServerHandler,
    };
    use serde::Deserialize;
    use serde_json::{json, Value};

    use crate::semconv::mcp_span_names::MCP_TOOL_EXECUTE_SPAN;
    use opentelemetry::global;
    use opentelemetry::trace::Tracer;

    // ── Parameter structs ─────────────────────────────────────────────────────
    // Each struct derives JsonSchema (for rmcp schema generation) and Deserialize.

    #[derive(Debug, Deserialize, schemars::JsonSchema)]
    pub struct EventLogParam {
        pub event_log: Value,
    }

    #[derive(Debug, Deserialize, schemars::JsonSchema)]
    pub struct ConformanceParams {
        pub event_log: Value,
        pub petri_net: Value,
    }

    #[derive(Debug, Deserialize, schemars::JsonSchema)]
    pub struct ParseXesParams {
        pub xes_xml: String,
    }

    #[derive(Debug, Deserialize, schemars::JsonSchema)]
    pub struct DetectDriftParams {
        pub baseline: Value,
        pub recent: Value,
    }

    #[derive(Debug, Deserialize, schemars::JsonSchema)]
    pub struct AbstractPetriNetParams {
        pub petri_net: Value,
    }

    #[derive(Debug, Deserialize, schemars::JsonSchema)]
    pub struct AbstractDfgParams {
        pub dfg: Value,
    }

    #[derive(Debug, Deserialize, schemars::JsonSchema)]
    pub struct QueryParams {
        pub query: String,
        pub event_log: Option<Value>,
    }

    #[derive(Debug, Deserialize, schemars::JsonSchema)]
    pub struct PredictRemainingTimeParams {
        pub training_log: Value,
        pub partial_trace: Value,
    }

    #[derive(Debug, Deserialize, schemars::JsonSchema)]
    pub struct OcelIngestParams {
        pub ocel: Value,
    }

    // ── Server struct ─────────────────────────────────────────────────────────

    #[derive(Debug, Clone)]
    pub struct Pm4pyMcpServer {
        tool_router: ToolRouter<Self>,
    }

    // ── Tool implementations ──────────────────────────────────────────────────

    #[tool_router]
    impl Pm4pyMcpServer {
        pub fn new() -> Self {
            Self {
                tool_router: Self::tool_router(),
            }
        }

        #[tool(
            description = "Discover a Petri net process model using the Alpha Miner algorithm from an event log."
        )]
        async fn pm4py_discover_alpha(
            &self,
            Parameters(EventLogParam { event_log }): Parameters<EventLogParam>,
        ) -> Result<String, String> {
            let _span = global::tracer("pm4py-rust").start(MCP_TOOL_EXECUTE_SPAN);
            crate::mcp::executor::execute_tool(
                "pm4py_discover_alpha",
                json!({ "event_log": event_log }),
            )
            .await
        }

        #[tool(
            description = "Check conformance between an event log and a Petri net using token replay."
        )]
        async fn pm4py_conformance_token_replay(
            &self,
            Parameters(ConformanceParams {
                event_log,
                petri_net,
            }): Parameters<ConformanceParams>,
        ) -> Result<String, String> {
            let _span = global::tracer("pm4py-rust").start(MCP_TOOL_EXECUTE_SPAN);
            crate::mcp::executor::execute_tool(
                "pm4py_conformance_token_replay",
                json!({ "event_log": event_log, "petri_net": petri_net }),
            )
            .await
        }

        #[tool(
            description = "Compute statistics (trace count, event count, activity frequencies, bottlenecks) from an event log."
        )]
        async fn pm4py_statistics(
            &self,
            Parameters(EventLogParam { event_log }): Parameters<EventLogParam>,
        ) -> Result<String, String> {
            let _span = global::tracer("pm4py-rust").start(MCP_TOOL_EXECUTE_SPAN);
            crate::mcp::executor::execute_tool(
                "pm4py_statistics",
                json!({ "event_log": event_log }),
            )
            .await
        }

        #[tool(description = "Parse an XES XML string into a structured event log JSON.")]
        async fn pm4py_parse_xes(
            &self,
            Parameters(ParseXesParams { xes_xml }): Parameters<ParseXesParams>,
        ) -> Result<String, String> {
            let _span = global::tracer("pm4py-rust").start(MCP_TOOL_EXECUTE_SPAN);
            crate::mcp::executor::execute_tool("pm4py_parse_xes", json!({ "xes_xml": xes_xml }))
                .await
        }

        #[tool(
            description = "Detect concept drift between a baseline and recent activity frequency distribution."
        )]
        async fn pm4py_detect_drift(
            &self,
            Parameters(DetectDriftParams { baseline, recent }): Parameters<DetectDriftParams>,
        ) -> Result<String, String> {
            let _span = global::tracer("pm4py-rust").start(MCP_TOOL_EXECUTE_SPAN);
            crate::mcp::executor::execute_tool(
                "pm4py_detect_drift",
                json!({ "baseline": baseline, "recent": recent }),
            )
            .await
        }

        #[tool(
            description = "Generate a natural-language description of a Petri net for LLM consumption."
        )]
        async fn pm4py_abstract_petri_net(
            &self,
            Parameters(AbstractPetriNetParams { petri_net }): Parameters<AbstractPetriNetParams>,
        ) -> Result<String, String> {
            let _span = global::tracer("pm4py-rust").start(MCP_TOOL_EXECUTE_SPAN);
            crate::mcp::executor::execute_tool(
                "pm4py_abstract_petri_net",
                json!({ "petri_net": petri_net }),
            )
            .await
        }

        #[tool(
            description = "Generate a natural-language summary of an event log for LLM consumption."
        )]
        async fn pm4py_abstract_event_log(
            &self,
            Parameters(EventLogParam { event_log }): Parameters<EventLogParam>,
        ) -> Result<String, String> {
            let _span = global::tracer("pm4py-rust").start(MCP_TOOL_EXECUTE_SPAN);
            crate::mcp::executor::execute_tool(
                "pm4py_abstract_event_log",
                json!({ "event_log": event_log }),
            )
            .await
        }

        #[tool(
            description = "Generate a natural-language description of a Directly-Follows Graph (DFG)."
        )]
        async fn pm4py_abstract_dfg(
            &self,
            Parameters(AbstractDfgParams { dfg }): Parameters<AbstractDfgParams>,
        ) -> Result<String, String> {
            let _span = global::tracer("pm4py-rust").start(MCP_TOOL_EXECUTE_SPAN);
            crate::mcp::executor::execute_tool("pm4py_abstract_dfg", json!({ "dfg": dfg })).await
        }

        #[tool(
            description = "Answer a natural-language query about a process. Optionally provide event_log for statistical grounding; set GROQ_API_KEY env var for LLM enrichment."
        )]
        async fn pm4py_query(
            &self,
            Parameters(QueryParams { query, event_log }): Parameters<QueryParams>,
        ) -> Result<String, String> {
            let _span = global::tracer("pm4py-rust").start(MCP_TOOL_EXECUTE_SPAN);
            let mut args = json!({ "query": query });
            if let Some(el) = event_log {
                args["event_log"] = el;
            }
            crate::mcp::executor::execute_tool("pm4py_query", args).await
        }

        #[tool(
            description = "Mine decision rules from an event log — discover branching conditions at choice points."
        )]
        async fn pm4py_decision_mine(
            &self,
            Parameters(EventLogParam { event_log }): Parameters<EventLogParam>,
        ) -> Result<String, String> {
            let _span = global::tracer("pm4py-rust").start(MCP_TOOL_EXECUTE_SPAN);
            crate::mcp::executor::execute_tool(
                "pm4py_decision_mine",
                json!({ "event_log": event_log }),
            )
            .await
        }

        #[tool(
            description = "Predict remaining cycle time for a partial trace using a training event log as reference."
        )]
        async fn pm4py_predict_remaining_time(
            &self,
            Parameters(PredictRemainingTimeParams {
                training_log,
                partial_trace,
            }): Parameters<PredictRemainingTimeParams>,
        ) -> Result<String, String> {
            let _span = global::tracer("pm4py-rust").start(MCP_TOOL_EXECUTE_SPAN);
            crate::mcp::executor::execute_tool(
                "pm4py_predict_remaining_time",
                json!({ "training_log": training_log, "partial_trace": partial_trace }),
            )
            .await
        }

        #[tool(
            description = "Ingest an Object-Centric Event Log (OCEL 2.0) and return summary statistics by object type."
        )]
        async fn pm4py_ocel_ingest(
            &self,
            Parameters(OcelIngestParams { ocel }): Parameters<OcelIngestParams>,
        ) -> Result<String, String> {
            let _span = global::tracer("pm4py-rust").start(MCP_TOOL_EXECUTE_SPAN);
            crate::mcp::executor::execute_tool("pm4py_ocel_ingest", json!({ "ocel": ocel })).await
        }
    }

    // ── ServerHandler ─────────────────────────────────────────────────────────

    #[tool_handler]
    impl ServerHandler for Pm4pyMcpServer {
        fn get_info(&self) -> ServerInfo {
            ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
                .with_server_info(Implementation::new("pm4py-rust", env!("CARGO_PKG_VERSION")))
        }
    }
}

#[cfg(feature = "mcp-server")]
pub use inner::Pm4pyMcpServer;
