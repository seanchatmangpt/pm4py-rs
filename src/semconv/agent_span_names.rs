/// Agent capability catalog operation — registering or querying the catalog of agent capabilities.
///
/// Span: `span.agent.capability.catalog`
/// Kind: `internal`
/// Stability: `development`
pub const AGENT_CAPABILITY_CATALOG_SPAN: &str = "agent.capability.catalog";
/// Agent coordination operation — dispatching tasks to sub-agents in a topology.
///
/// Span: `span.agent.coordinate`
/// Kind: `client`
/// Stability: `development`
pub const AGENT_COORDINATE_SPAN: &str = "agent.coordinate";
/// An autonomous decision made by an agent — action selection with confidence scoring.
///
/// Span: `span.agent.decision`
/// Kind: `internal`
/// Stability: `development`
pub const AGENT_DECISION_SPAN: &str = "agent.decision";
/// Execution of an agent execution graph — traversing a DAG of agent steps to completion.
///
/// Span: `span.agent.execution.graph`
/// Kind: `internal`
/// Stability: `development`
pub const AGENT_EXECUTION_GRAPH_SPAN: &str = "agent.execution.graph";
/// Agent handoff — transfers control and state to another agent based on capability, load, or priority.
///
/// Span: `span.agent.handoff`
/// Kind: `producer`
/// Stability: `development`
pub const AGENT_HANDOFF_SPAN: &str = "agent.handoff";
/// LLM inference call made by an OSA agent.
///
/// Span: `span.agent.llm_predict`
/// Kind: `client`
/// Stability: `development`
pub const AGENT_LLM_PREDICT_SPAN: &str = "agent.llm_predict";
/// One iteration of the agent's main reasoning and action loop.
///
/// Span: `span.agent.loop`
/// Kind: `internal`
/// Stability: `development`
pub const AGENT_LOOP_SPAN: &str = "agent.loop";
/// Synchronizing agent memory state with a federated memory pool shared across agents.
///
/// Span: `span.agent.memory.federate`
/// Kind: `client`
/// Stability: `development`
pub const AGENT_MEMORY_FEDERATE_SPAN: &str = "agent.memory.federate";
/// Agent memory update — writing new information to agent working memory.
///
/// Span: `span.agent.memory.update`
/// Kind: `internal`
/// Stability: `development`
pub const AGENT_MEMORY_UPDATE_SPAN: &str = "agent.memory.update";
/// Execution of an agent pipeline stage — processes data through a defined transformation.
///
/// Span: `span.agent.pipeline.execute`
/// Kind: `internal`
/// Stability: `development`
pub const AGENT_PIPELINE_EXECUTE_SPAN: &str = "agent.pipeline.execute";
/// Agent reasoning trace — records the chain-of-thought steps an agent takes to reach a decision.
///
/// Span: `span.agent.reasoning.trace`
/// Kind: `internal`
/// Stability: `development`
pub const AGENT_REASONING_TRACE_SPAN: &str = "agent.reasoning.trace";
/// Agent spawning — creating a new child agent under the current supervision tree.
///
/// Span: `span.agent.spawn`
/// Kind: `internal`
/// Stability: `development`
pub const AGENT_SPAWN_SPAN: &str = "agent.spawn";
/// Agent spawn profiling — observing the performance characteristics of a child agent spawn operation.
///
/// Span: `span.agent.spawn.profile`
/// Kind: `internal`
/// Stability: `development`
pub const AGENT_SPAWN_PROFILE_SPAN: &str = "agent.spawn.profile";
/// Agent workflow checkpoint — capturing workflow state to enable resumption after interruption.
///
/// Span: `span.agent.workflow.checkpoint`
/// Kind: `internal`
/// Stability: `development`
pub const AGENT_WORKFLOW_CHECKPOINT_SPAN: &str = "agent.workflow.checkpoint";
