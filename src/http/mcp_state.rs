/// Shared CancellationToken for coordinated rmcp session + axum shutdown.
///
/// On Ctrl+C: main.rs cancels this token → rmcp StreamableHttpService drains
/// in-flight sessions → axum serve() returns → process exits cleanly.
/// WvdA liveness: token is created once and never re-created.

#[cfg(feature = "mcp-server")]
use tokio_util::sync::CancellationToken;

#[cfg(feature = "mcp-server")]
static MCP_CT: std::sync::OnceLock<CancellationToken> = std::sync::OnceLock::new();

/// Return the process-lifetime MCP CancellationToken.
/// Creates it on first call (OnceLock — thread-safe, no mutex needed).
#[cfg(feature = "mcp-server")]
pub fn mcp_cancellation_token() -> &'static CancellationToken {
    MCP_CT.get_or_init(CancellationToken::new)
}
