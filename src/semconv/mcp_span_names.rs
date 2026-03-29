/// An MCP tool invocation — request from an agent to execute a named tool via MCP protocol.
///
/// Span: `span.mcp.call`
/// Kind: `client`
/// Stability: `development`
pub const MCP_CALL_SPAN: &str = "mcp.call";
/// MCP client-server connection establishment — transport negotiation and capability exchange.
///
/// Span: `span.mcp.connection.establish`
/// Kind: `client`
/// Stability: `development`
pub const MCP_CONNECTION_ESTABLISH_SPAN: &str = "mcp.connection.establish";
/// Acquiring a connection from the MCP connection pool for use in a client-server interaction.
///
/// Span: `span.mcp.connection.pool.acquire`
/// Kind: `internal`
/// Stability: `development`
pub const MCP_CONNECTION_POOL_ACQUIRE_SPAN: &str = "mcp.connection.pool.acquire";
/// MCP tool discovery — listing available tools from a connected server.
///
/// Span: `span.mcp.registry.discover`
/// Kind: `client`
/// Stability: `development`
pub const MCP_REGISTRY_DISCOVER_SPAN: &str = "mcp.registry.discover";
/// Reading an MCP resource — fetching content from a resource URI exposed by an MCP server.
///
/// Span: `span.mcp.resource.read`
/// Kind: `client`
/// Stability: `development`
pub const MCP_RESOURCE_READ_SPAN: &str = "mcp.resource.read";
/// Health check of an MCP server — verifying tool availability and server responsiveness.
///
/// Span: `span.mcp.server.health_check`
/// Kind: `internal`
/// Stability: `development`
pub const MCP_SERVER_HEALTH_CHECK_SPAN: &str = "mcp.server.health_check";
/// Collecting aggregated metrics from an MCP server instance.
///
/// Span: `span.mcp.server.metrics.collect`
/// Kind: `internal`
/// Stability: `development`
pub const MCP_SERVER_METRICS_COLLECT_SPAN: &str = "mcp.server.metrics.collect";
/// New MCP session allocation by the StreamableHttpService server.
///
/// Span: `span.mcp.session.create`
/// Kind: `server`
/// Stability: `development`
pub const MCP_SESSION_CREATE_SPAN: &str = "mcp.session.create";
/// MCP tool analytics recording — capturing tool usage statistics for performance monitoring and capacity planning.
///
/// Span: `span.mcp.tool.analytics.record`
/// Kind: `internal`
/// Stability: `development`
pub const MCP_TOOL_ANALYTICS_RECORD_SPAN: &str = "mcp.tool.analytics.record";
/// MCP tool cache lookup — checking response cache before executing tool.
///
/// Span: `span.mcp.tool.cache.lookup`
/// Kind: `internal`
/// Stability: `development`
pub const MCP_TOOL_CACHE_LOOKUP_SPAN: &str = "mcp.tool.cache.lookup";
/// Composition of multiple MCP tools into a chain — sequential, parallel, or fallback execution.
///
/// Span: `span.mcp.tool.compose`
/// Kind: `internal`
/// Stability: `development`
pub const MCP_TOOL_COMPOSE_SPAN: &str = "mcp.tool.compose";
/// MCP tool deprecation lifecycle event — marking a tool as deprecated and scheduling its removal.
///
/// Span: `span.mcp.tool.deprecate`
/// Kind: `internal`
/// Stability: `development`
pub const MCP_TOOL_DEPRECATE_SPAN: &str = "mcp.tool.deprecate";
/// A retry attempt for a previously failed MCP tool execution.
///
/// Span: `span.mcp.tool.retry`
/// Kind: `client`
/// Stability: `development`
pub const MCP_TOOL_RETRY_SPAN: &str = "mcp.tool.retry";
/// MCP tool execution timed out — tool did not respond within the configured budget.
///
/// Span: `span.mcp.tool.timeout`
/// Kind: `client`
/// Stability: `development`
pub const MCP_TOOL_TIMEOUT_SPAN: &str = "mcp.tool.timeout";
/// Validating MCP tool input/output schema before execution.
///
/// Span: `span.mcp.tool.validate`
/// Kind: `internal`
/// Stability: `development`
pub const MCP_TOOL_VALIDATE_SPAN: &str = "mcp.tool.validate";
/// Version compatibility check for an MCP tool — validates client version against server tool version.
///
/// Span: `span.mcp.tool.version_check`
/// Kind: `internal`
/// Stability: `development`
pub const MCP_TOOL_VERSION_CHECK_SPAN: &str = "mcp.tool.version_check";
/// Server-side execution of an MCP tool — the handler running the tool logic.
///
/// Span: `span.mcp.tool_execute`
/// Kind: `server`
/// Stability: `development`
pub const MCP_TOOL_EXECUTE_SPAN: &str = "mcp.tool_execute";
/// Establishment of an MCP transport connection — initial handshake and protocol negotiation.
///
/// Span: `span.mcp.transport.connect`
/// Kind: `client`
/// Stability: `development`
pub const MCP_TRANSPORT_CONNECT_SPAN: &str = "mcp.transport.connect";
