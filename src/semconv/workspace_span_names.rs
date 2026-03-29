/// Tracking a workspace activity — records type, duration, and context of user/agent actions.
///
/// Span: `span.workspace.activity.track`
/// Kind: `internal`
/// Stability: `development`
pub const WORKSPACE_ACTIVITY_TRACK_SPAN: &str = "workspace.activity.track";
/// Saving a workspace checkpoint — persisting agent state and task queue for recovery.
///
/// Span: `span.workspace.checkpoint.save`
/// Kind: `internal`
/// Stability: `development`
pub const WORKSPACE_CHECKPOINT_SAVE_SPAN: &str = "workspace.checkpoint.save";
/// Creating a context checkpoint — snapshot of current workspace state for potential rollback.
///
/// Span: `span.workspace.context.checkpoint`
/// Kind: `internal`
/// Stability: `development`
pub const WORKSPACE_CONTEXT_CHECKPOINT_SPAN: &str = "workspace.context.checkpoint";
/// Creating a compressed snapshot of workspace context for persistence or recovery.
///
/// Span: `span.workspace.context.snapshot`
/// Kind: `internal`
/// Stability: `development`
pub const WORKSPACE_CONTEXT_SNAPSHOT_SPAN: &str = "workspace.context.snapshot";
/// Context window update — tokens added or pruned from the workspace context.
///
/// Span: `span.workspace.context.update`
/// Kind: `internal`
/// Stability: `development`
pub const WORKSPACE_CONTEXT_UPDATE_SPAN: &str = "workspace.context.update";
/// Workspace memory compaction — reducing memory footprint by consolidating and pruning stored context items.
///
/// Span: `span.workspace.memory.compact`
/// Kind: `internal`
/// Stability: `development`
pub const WORKSPACE_MEMORY_COMPACT_SPAN: &str = "workspace.memory.compact";
/// Orchestrating work distribution across agents in the workspace.
///
/// Span: `span.workspace.orchestrate`
/// Kind: `internal`
/// Stability: `development`
pub const WORKSPACE_ORCHESTRATE_SPAN: &str = "workspace.orchestrate";
/// Ending a workspace session — recording final metrics and persisting session state.
///
/// Span: `span.workspace.session.end`
/// Kind: `internal`
/// Stability: `development`
pub const WORKSPACE_SESSION_END_SPAN: &str = "workspace.session.end";
/// Workspace session initialization — agent begins processing in a new session context.
///
/// Span: `span.workspace.session.start`
/// Kind: `internal`
/// Stability: `development`
pub const WORKSPACE_SESSION_START_SPAN: &str = "workspace.session.start";
/// Sharing a workspace with other agents — granting access with defined permissions and scope.
///
/// Span: `span.workspace.share`
/// Kind: `internal`
/// Stability: `development`
pub const WORKSPACE_SHARE_SPAN: &str = "workspace.share";
/// Tool invocation within a workspace session.
///
/// Span: `span.workspace.tool.invoke`
/// Kind: `internal`
/// Stability: `development`
pub const WORKSPACE_TOOL_INVOKE_SPAN: &str = "workspace.tool.invoke";
