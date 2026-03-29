/// Root span for a YAWL workflow case. One span per case_id. Encapsulates the full lifecycle from INSTANCE_CREATED to INSTANCE_COMPLETED or INSTANCE_CANCELLED. The yawl.case.id is the correlation key linking all task execution spans.
///
/// Span: `span.yawl.case`
/// Kind: `internal`
/// Stability: `development`
pub const YAWL_CASE_SPAN: &str = "yawl.case";
/// Client span for launching a new YAWL case via the embedded server HTTP endpoint (POST /api/cases/launch). Emitted by CaseLifecycle GenServer.
///
/// Span: `span.yawl.case.launch`
/// Kind: `client`
/// Stability: `development`
pub const YAWL_CASE_LAUNCH_SPAN: &str = "yawl.case.launch";
/// Span for a single YAWL task execution within a case. Child of span.yawl.case. Covers the full task lifecycle: TASK_ENABLED → TASK_STARTED (tokens consumed) → TASK_COMPLETED (tokens produced). The yawl.token.consumed and yawl.token.produced attributes record Petri net token flow.
///
/// Span: `span.yawl.task.execution`
/// Kind: `internal`
/// Stability: `development`
pub const YAWL_TASK_EXECUTION_SPAN: &str = "yawl.task.execution";
/// Client span for completing (checking in) a YAWL work item via the embedded server (POST /api/cases/{id}/workitems/{wid}/complete). Emitted by CaseLifecycle GenServer.
///
/// Span: `span.yawl.workitem.complete`
/// Kind: `client`
/// Stability: `development`
pub const YAWL_WORKITEM_COMPLETE_SPAN: &str = "yawl.workitem.complete";
