//! Stdio transport entry point for Claude Desktop / AI agent integration.
//!
//! Claude Desktop config (`~/Library/Application Support/Claude/claude_desktop_config.json`):
//! ```json
//! {
//!   "mcpServers": {
//!     "pm4py-rust": {
//!       "command": "/path/to/mcp_stdio"
//!     }
//!   }
//! }
//! ```
//!
//! Build:
//! ```bash
//! cargo build --features mcp-server --bin mcp_stdio --release
//! ```
use rmcp::{transport::stdio, ServiceExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Log to stderr so stdout stays clean for JSON-RPC framing
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("pm4py=info".parse().unwrap()),
        )
        .init();

    let server = pm4py::mcp::rmcp_server::Pm4pyMcpServer::new()
        .serve(stdio())
        .await?;

    server.waiting().await?;
    Ok(())
}
