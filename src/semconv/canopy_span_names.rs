/// Canopy adapter invocation — calling an external service via a Canopy adapter.
///
/// Span: `span.canopy.adapter_call`
/// Kind: `client`
/// Stability: `development`
pub const CANOPY_ADAPTER_CALL_SPAN: &str = "canopy.adapter_call";
/// Broadcast of a signal or command to all connected agents.
///
/// Span: `span.canopy.broadcast`
/// Kind: `producer`
/// Stability: `development`
pub const CANOPY_BROADCAST_SPAN: &str = "canopy.broadcast";
/// Command dispatch through the Canopy workspace protocol.
///
/// Span: `span.canopy.command`
/// Kind: `producer`
/// Stability: `development`
pub const CANOPY_COMMAND_SPAN: &str = "canopy.command";
/// Canopy heartbeat dispatch — periodic health signal sent to connected services.
///
/// Span: `span.canopy.heartbeat`
/// Kind: `internal`
/// Stability: `development`
pub const CANOPY_HEARTBEAT_SPAN: &str = "canopy.heartbeat";
/// Individual heartbeat probe — one RTT measurement to a single OSA node.
///
/// Span: `span.canopy.heartbeat.probe`
/// Kind: `internal`
/// Stability: `development`
pub const CANOPY_HEARTBEAT_PROBE_SPAN: &str = "canopy.heartbeat.probe";
/// Canopy workspace session creation — initializing a new collaboration session.
///
/// Span: `span.canopy.session.create`
/// Kind: `server`
/// Stability: `development`
pub const CANOPY_SESSION_CREATE_SPAN: &str = "canopy.session.create";
/// Creating a point-in-time snapshot of the canopy workspace state.
///
/// Span: `span.canopy.snapshot.create`
/// Kind: `internal`
/// Stability: `development`
pub const CANOPY_SNAPSHOT_CREATE_SPAN: &str = "canopy.snapshot.create";
/// Reconciling workspace state between peers — resolving conflicts and applying updates.
///
/// Span: `span.canopy.workspace.reconcile`
/// Kind: `internal`
/// Stability: `development`
pub const CANOPY_WORKSPACE_RECONCILE_SPAN: &str = "canopy.workspace.reconcile";
/// Synchronization of workspace state across connected agents.
///
/// Span: `span.canopy.workspace.sync`
/// Kind: `internal`
/// Stability: `development`
pub const CANOPY_WORKSPACE_SYNC_SPAN: &str = "canopy.workspace.sync";
