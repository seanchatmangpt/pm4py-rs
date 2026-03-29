/// Span emitted when the agent card endpoint is served.
///
/// Span: `span.a2a.agent_card.serve`
/// Kind: `server`
/// Stability: `development`
pub const A2A_AGENT_CARD_SERVE_SPAN: &str = "a2a.agent_card.serve";
/// Running an A2A capability auction — agents bid for task allocation based on capability and cost.
///
/// Span: `span.a2a.auction.run`
/// Kind: `internal`
/// Stability: `development`
pub const A2A_AUCTION_RUN_SPAN: &str = "a2a.auction.run";
/// Bid evaluation — scoring and ranking agent bids to select the best provider for a task.
///
/// Span: `span.a2a.bid.evaluate`
/// Kind: `internal`
/// Stability: `development`
pub const A2A_BID_EVALUATE_SPAN: &str = "a2a.bid.evaluate";
/// An agent-to-agent call — one ChatmanGPT service invoking another via the A2A protocol.
///
/// Span: `span.a2a.call`
/// Kind: `client`
/// Stability: `development`
pub const A2A_CALL_SPAN: &str = "a2a.call";
/// Canceling an A2A task via tasks/cancel JSON-RPC call. Emitted by Canopy.Telemetry.A2AHandler when a task cancel request is processed.
///
/// Span: `span.a2a.cancel`
/// Kind: `client`
/// Stability: `development`
pub const A2A_CANCEL_SPAN: &str = "a2a.cancel";
/// Matching a capability request to available agents — selecting best provider.
///
/// Span: `span.a2a.capability.match`
/// Kind: `internal`
/// Stability: `development`
pub const A2A_CAPABILITY_MATCH_SPAN: &str = "a2a.capability.match";
/// Capability negotiation between two A2A agents — determining what capabilities can be fulfilled.
///
/// Span: `span.a2a.capability.negotiate`
/// Kind: `client`
/// Stability: `development`
pub const A2A_CAPABILITY_NEGOTIATE_SPAN: &str = "a2a.capability.negotiate";
/// Registration of an agent capability in the A2A capability registry.
///
/// Span: `span.a2a.capability.register`
/// Kind: `server`
/// Stability: `development`
pub const A2A_CAPABILITY_REGISTER_SPAN: &str = "a2a.capability.register";
/// Contract amendment — negotiating a modification to an existing A2A service contract.
///
/// Span: `span.a2a.contract.amend`
/// Kind: `client`
/// Stability: `development`
pub const A2A_CONTRACT_AMEND_SPAN: &str = "a2a.contract.amend";
/// Initiating or updating an A2A contract dispute between agents.
///
/// Span: `span.a2a.contract.dispute`
/// Kind: `internal`
/// Stability: `development`
pub const A2A_CONTRACT_DISPUTE_SPAN: &str = "a2a.contract.dispute";
/// Execution of an A2A service contract — running contract obligations and tracking progress toward completion.
///
/// Span: `span.a2a.contract.execute`
/// Kind: `internal`
/// Stability: `development`
pub const A2A_CONTRACT_EXECUTE_SPAN: &str = "a2a.contract.execute";
/// Negotiation of an A2A service contract — establishing terms, SLA, and obligations between two agents.
///
/// Span: `span.a2a.contract.negotiate`
/// Kind: `client`
/// Stability: `development`
pub const A2A_CONTRACT_NEGOTIATE_SPAN: &str = "a2a.contract.negotiate";
/// Creation of an A2A deal between two agents.
///
/// Span: `span.a2a.create_deal`
/// Kind: `server`
/// Stability: `development`
pub const A2A_CREATE_DEAL_SPAN: &str = "a2a.create_deal";
/// Status transition of an A2A deal through its lifecycle (pending → active → completed).
///
/// Span: `span.a2a.deal.status_transition`
/// Kind: `internal`
/// Stability: `development`
pub const A2A_DEAL_STATUS_TRANSITION_SPAN: &str = "a2a.deal.status_transition";
/// Resolution of an A2A dispute between agents — arbitration and settlement process.
///
/// Span: `span.a2a.dispute.resolve`
/// Kind: `internal`
/// Stability: `development`
pub const A2A_DISPUTE_RESOLVE_SPAN: &str = "a2a.dispute.resolve";
/// A2A escrow creation — establishing a payment escrow for a deal between two agents.
///
/// Span: `span.a2a.escrow.create`
/// Kind: `server`
/// Stability: `development`
pub const A2A_ESCROW_CREATE_SPAN: &str = "a2a.escrow.create";
/// A2A escrow release — settling a payment escrow upon deal completion or dispute resolution.
///
/// Span: `span.a2a.escrow.release`
/// Kind: `server`
/// Stability: `development`
pub const A2A_ESCROW_RELEASE_SPAN: &str = "a2a.escrow.release";
/// Transfer of knowledge or capability data between agents via A2A.
///
/// Span: `span.a2a.knowledge.transfer`
/// Kind: `producer`
/// Stability: `development`
pub const A2A_KNOWLEDGE_TRANSFER_SPAN: &str = "a2a.knowledge.transfer";
/// Receiving an A2A message/send JSON-RPC call via A2A.Plug. Emitted by Canopy.Telemetry.A2AHandler when the server receives a message.
///
/// Span: `span.a2a.message`
/// Kind: `server`
/// Stability: `development`
pub const A2A_MESSAGE_SPAN: &str = "a2a.message";
/// Batched delivery of multiple A2A messages — aggregates messages for efficient transport.
///
/// Span: `span.a2a.message.batch`
/// Kind: `producer`
/// Stability: `development`
pub const A2A_MESSAGE_BATCH_SPAN: &str = "a2a.message.batch";
/// Span emitted when an A2A agent receives an incoming message.
///
/// Span: `span.a2a.message.receive`
/// Kind: `server`
/// Stability: `development`
pub const A2A_MESSAGE_RECEIVE_SPAN: &str = "a2a.message.receive";
/// Routing of an A2A message to the appropriate target agent based on priority and routing rules.
///
/// Span: `span.a2a.message.route`
/// Kind: `producer`
/// Stability: `development`
pub const A2A_MESSAGE_ROUTE_SPAN: &str = "a2a.message.route";
/// Multi-round deal negotiation between two agents.
///
/// Span: `span.a2a.negotiate`
/// Kind: `client`
/// Stability: `development`
pub const A2A_NEGOTIATE_SPAN: &str = "a2a.negotiate";
/// State machine transition in an A2A multi-round negotiation protocol.
///
/// Span: `span.a2a.negotiation.state_transition`
/// Kind: `internal`
/// Stability: `development`
pub const A2A_NEGOTIATION_STATE_TRANSITION_SPAN: &str = "a2a.negotiation.state_transition";
/// Applying a penalty or reward to an agent based on contract performance — updates trust score and balance.
///
/// Span: `span.a2a.penalty.apply`
/// Kind: `server`
/// Stability: `development`
pub const A2A_PENALTY_APPLY_SPAN: &str = "a2a.penalty.apply";
/// A2A protocol version negotiation between two agents — determining compatible protocol version.
///
/// Span: `span.a2a.protocol.negotiate`
/// Kind: `client`
/// Stability: `development`
pub const A2A_PROTOCOL_NEGOTIATE_SPAN: &str = "a2a.protocol.negotiate";
/// A2A reputation decay event — applying time-based or violation-triggered reputation score reduction.
///
/// Span: `span.a2a.reputation.decay`
/// Kind: `internal`
/// Stability: `development`
pub const A2A_REPUTATION_DECAY_SPAN: &str = "a2a.reputation.decay";
/// Updating an agent's reputation score based on the outcome of a completed interaction.
///
/// Span: `span.a2a.reputation.update`
/// Kind: `internal`
/// Stability: `development`
pub const A2A_REPUTATION_UPDATE_SPAN: &str = "a2a.reputation.update";
/// Span emitted when an A2A agent dispatches a skill for execution.
///
/// Span: `span.a2a.skill.invoke`
/// Kind: `internal`
/// Stability: `development`
pub const A2A_SKILL_INVOKE_SPAN: &str = "a2a.skill.invoke";
/// SLA validation for an A2A operation — measures actual latency against deadline.
///
/// Span: `span.a2a.sla.check`
/// Kind: `internal`
/// Stability: `development`
pub const A2A_SLA_CHECK_SPAN: &str = "a2a.sla.check";
/// SLO evaluation — assessing whether A2A operation met service level objectives.
///
/// Span: `span.a2a.slo.evaluate`
/// Kind: `internal`
/// Stability: `development`
pub const A2A_SLO_EVALUATE_SPAN: &str = "a2a.slo.evaluate";
/// Span emitted when an A2A task reaches a terminal state (completed or failed).
///
/// Span: `span.a2a.task.complete`
/// Kind: `internal`
/// Stability: `development`
pub const A2A_TASK_COMPLETE_SPAN: &str = "a2a.task.complete";
/// Span emitted when an A2A task is created via tasks/send.
///
/// Span: `span.a2a.task.create`
/// Kind: `server`
/// Stability: `development`
pub const A2A_TASK_CREATE_SPAN: &str = "a2a.task.create";
/// Delegation of a task from one agent to another via A2A.
///
/// Span: `span.a2a.task.delegate`
/// Kind: `producer`
/// Stability: `development`
pub const A2A_TASK_DELEGATE_SPAN: &str = "a2a.task.delegate";
/// Span emitted when an A2A task state transitions (e.g., submitted→working).
///
/// Span: `span.a2a.task.update`
/// Kind: `internal`
/// Stability: `development`
pub const A2A_TASK_UPDATE_SPAN: &str = "a2a.task.update";
/// Evaluation of an agent's trust score based on reputation history and interaction outcomes.
///
/// Span: `span.a2a.trust.evaluate`
/// Kind: `internal`
/// Stability: `development`
pub const A2A_TRUST_EVALUATE_SPAN: &str = "a2a.trust.evaluate";
/// Federated trust evaluation — agent joins or queries a trust ring for cross-federation capability authorization.
///
/// Span: `span.a2a.trust.federate`
/// Kind: `client`
/// Stability: `development`
pub const A2A_TRUST_FEDERATE_SPAN: &str = "a2a.trust.federate";
