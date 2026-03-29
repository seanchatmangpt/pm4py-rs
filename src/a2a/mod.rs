/// A2A (Agent-to-Agent) protocol module for pm4py-rust.
///
/// Exposes all 10 pm4py process mining tools as A2A skills with:
/// - JSON-RPC 2.0 task lifecycle (submitted → working → completed/failed/canceled)
/// - Typed AgentCard at GET /.well-known/agent-card.json
/// - OTEL spans (a2a.task.create, a2a.skill.invoke, a2a.task.complete, a2a.agent_card.serve)
/// - WvdA 60s tool timeout, MAX_TASK_STORAGE = 1000
///
/// Feature-gated: compile with `--features a2a`.
pub mod artifact;
pub mod dispatch;
pub mod handler;
pub mod protocol;
pub mod skills;
pub mod task;
pub mod task_storage;
