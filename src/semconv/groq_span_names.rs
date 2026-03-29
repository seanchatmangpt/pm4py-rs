/// Span for a Groq LLM call that produces a YAWL workflow routing decision. Bridges the Groq response to a YAWL workflow action (launch_case, start_workitem, complete_workitem, checkpoint). The decision.wcp_pattern identifies which WCP pattern the LLM decision is targeting.
///
/// Span: `span.groq.workflow.decision`
/// Kind: `client`
/// Stability: `development`
pub const GROQ_WORKFLOW_DECISION_SPAN: &str = "groq.workflow.decision";
