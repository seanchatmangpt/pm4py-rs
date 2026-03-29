/// Cancellation of a workflow region — all in-flight activities in region halted.
///
/// Span: `span.workflow.cancel_region`
/// Kind: `internal`
/// Stability: `development`
pub const WORKFLOW_CANCEL_REGION_SPAN: &str = "workflow.cancel_region";
/// Critical section execution — ensures atomic sequential execution of enclosed activities.
///
/// Span: `span.workflow.critical_section`
/// Kind: `internal`
/// Stability: `development`
pub const WORKFLOW_CRITICAL_SECTION_SPAN: &str = "workflow.critical_section";
/// Deferred exclusive choice — decision deferred until first branch fires.
///
/// Span: `span.workflow.deferred_choice`
/// Kind: `internal`
/// Stability: `development`
pub const WORKFLOW_DEFERRED_CHOICE_SPAN: &str = "workflow.deferred_choice";
/// N-out-of-M join evaluation — fires when N of M branches complete.
///
/// Span: `span.workflow.discriminator`
/// Kind: `internal`
/// Stability: `development`
pub const WORKFLOW_DISCRIMINATOR_SPAN: &str = "workflow.discriminator";
/// Exclusive choice pattern (WP-4) — XOR split, exactly one branch is selected based on condition.
///
/// Span: `span.workflow.exclusive_choice`
/// Kind: `internal`
/// Stability: `development`
pub const WORKFLOW_EXCLUSIVE_CHOICE_SPAN: &str = "workflow.exclusive_choice";
/// Execution of a single workflow step or activity in the YAWL workflow engine.
///
/// Span: `span.workflow.execute`
/// Kind: `internal`
/// Stability: `development`
pub const WORKFLOW_EXECUTE_SPAN: &str = "workflow.execute";
/// Interleaved routing execution — activities in a set run one at a time in arbitrary order.
///
/// Span: `span.workflow.interleaved_routing`
/// Kind: `internal`
/// Stability: `development`
pub const WORKFLOW_INTERLEAVED_ROUTING_SPAN: &str = "workflow.interleaved_routing";
/// Milestone gate check — execution blocked until milestone condition met.
///
/// Span: `span.workflow.milestone`
/// Kind: `internal`
/// Stability: `development`
pub const WORKFLOW_MILESTONE_SPAN: &str = "workflow.milestone";
/// Multi-choice pattern (WP-6) — one or more branches selected based on runtime conditions.
///
/// Span: `span.workflow.multi_choice`
/// Kind: `internal`
/// Stability: `development`
pub const WORKFLOW_MULTI_CHOICE_SPAN: &str = "workflow.multi_choice";
/// Multi-instance activity execution — N parallel instances of same activity.
///
/// Span: `span.workflow.multi_instance`
/// Kind: `internal`
/// Stability: `development`
pub const WORKFLOW_MULTI_INSTANCE_SPAN: &str = "workflow.multi_instance";
/// Parallel split pattern (WP-2) — single thread of control splits into N concurrent branches.
///
/// Span: `span.workflow.parallel_split`
/// Kind: `internal`
/// Stability: `development`
pub const WORKFLOW_PARALLEL_SPLIT_SPAN: &str = "workflow.parallel_split";
/// Persistent trigger activation — trigger that persists in the environment until explicitly consumed.
///
/// Span: `span.workflow.persistent_trigger`
/// Kind: `producer`
/// Stability: `development`
pub const WORKFLOW_PERSISTENT_TRIGGER_SPAN: &str = "workflow.persistent_trigger";
/// Sequence pattern (WP-1) — activities execute in strict serial order.
///
/// Span: `span.workflow.sequence`
/// Kind: `internal`
/// Stability: `development`
pub const WORKFLOW_SEQUENCE_SPAN: &str = "workflow.sequence";
/// Simple merge pattern (WP-5) — merges two or more alternative branches without synchronization.
///
/// Span: `span.workflow.simple_merge`
/// Kind: `internal`
/// Stability: `development`
pub const WORKFLOW_SIMPLE_MERGE_SPAN: &str = "workflow.simple_merge";
/// Structured loop iteration — while-do execution with bounded iteration count.
///
/// Span: `span.workflow.structured_loop`
/// Kind: `internal`
/// Stability: `development`
pub const WORKFLOW_STRUCTURED_LOOP_SPAN: &str = "workflow.structured_loop";
/// Structured synchronizing merge (WP-7) — merges branches, waiting for all that were activated.
///
/// Span: `span.workflow.structured_sync_merge`
/// Kind: `internal`
/// Stability: `development`
pub const WORKFLOW_STRUCTURED_SYNC_MERGE_SPAN: &str = "workflow.structured_sync_merge";
/// Synchronization pattern (WP-3) — waits for ALL concurrent branches to complete before merging.
///
/// Span: `span.workflow.synchronization`
/// Kind: `internal`
/// Stability: `development`
pub const WORKFLOW_SYNCHRONIZATION_SPAN: &str = "workflow.synchronization";
/// State transition within a workflow — moving from one state to another.
///
/// Span: `span.workflow.transition`
/// Kind: `internal`
/// Stability: `development`
pub const WORKFLOW_TRANSITION_SPAN: &str = "workflow.transition";
