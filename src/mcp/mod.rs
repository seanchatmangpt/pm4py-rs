pub(crate) mod executor;
/// MCP tool server for pm4py-rust process mining engine.
///
/// Exposes 10 process mining tools via MCP 2024-11-05 HTTP transport.
/// Stateless JSON-RPC handler — no sessions, no streaming.
///
/// OSA integration: add to ~/.osa/mcp.json
/// ```json
/// {
///   "mcpServers": {
///     "pm4py-rust": {
///       "transport": "http",
///       "url": "http://localhost:8090/mcp"
///     }
///   }
/// }
/// ```
pub mod handler;
pub mod protocol;
pub mod tools;

#[cfg(feature = "mcp-server")]
pub mod rmcp_server;
