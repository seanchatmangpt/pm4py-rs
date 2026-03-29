/// Committing a decided value as a block in the HotStuff BFT log.
///
/// Span: `span.consensus.block.commit`
/// Kind: `internal`
/// Stability: `development`
pub const CONSENSUS_BLOCK_COMMIT_SPAN: &str = "consensus.block.commit";
/// Byzantine fault recovery — adjusts quorum and restores consensus after detecting byzantine behavior.
///
/// Span: `span.consensus.byzantine.recover`
/// Kind: `internal`
/// Stability: `development`
pub const CONSENSUS_BYZANTINE_RECOVER_SPAN: &str = "consensus.byzantine.recover";
/// Epoch advancement — consensus protocol advances to a new epoch after configuration change or key rotation.
///
/// Span: `span.consensus.epoch.advance`
/// Kind: `internal`
/// Stability: `development`
pub const CONSENSUS_EPOCH_ADVANCE_SPAN: &str = "consensus.epoch.advance";
/// Epoch finalization — collecting signatures and committing the final state of a consensus epoch.
///
/// Span: `span.consensus.epoch.finalize`
/// Kind: `internal`
/// Stability: `development`
pub const CONSENSUS_EPOCH_FINALIZE_SPAN: &str = "consensus.epoch.finalize";
/// Epoch key rotation — rotating cryptographic keys for a consensus epoch after a configuration change or compromise.
///
/// Span: `span.consensus.epoch.key_rotate`
/// Kind: `internal`
/// Stability: `development`
pub const CONSENSUS_EPOCH_KEY_ROTATE_SPAN: &str = "consensus.epoch.key_rotate";
/// Epoch quorum snapshot — capturing the quorum membership set at an epoch boundary.
///
/// Span: `span.consensus.epoch.quorum_snapshot`
/// Kind: `internal`
/// Stability: `development`
pub const CONSENSUS_EPOCH_QUORUM_SNAPSHOT_SPAN: &str = "consensus.epoch.quorum_snapshot";
/// Epoch transition in the consensus protocol — moving from one epoch to the next.
///
/// Span: `span.consensus.epoch.transition`
/// Kind: `internal`
/// Stability: `development`
pub const CONSENSUS_EPOCH_TRANSITION_SPAN: &str = "consensus.epoch.transition";
/// Fork detection in the consensus chain — identifies diverged branches and applies resolution strategy.
///
/// Span: `span.consensus.fork.detect`
/// Kind: `internal`
/// Stability: `development`
pub const CONSENSUS_FORK_DETECT_SPAN: &str = "consensus.fork.detect";
/// Leader rotation event — current leader yields and new leader is selected via scoring.
///
/// Span: `span.consensus.leader.rotate`
/// Kind: `internal`
/// Stability: `development`
pub const CONSENSUS_LEADER_ROTATE_SPAN: &str = "consensus.leader.rotate";
/// Leader election event in HotStuff BFT — new leader selected after view change.
///
/// Span: `span.consensus.leader_election`
/// Kind: `internal`
/// Stability: `development`
pub const CONSENSUS_LEADER_ELECTION_SPAN: &str = "consensus.leader_election";
/// Verifying liveness of the consensus protocol — confirming progress is being made.
///
/// Span: `span.consensus.liveness.check`
/// Kind: `internal`
/// Stability: `development`
pub const CONSENSUS_LIVENESS_CHECK_SPAN: &str = "consensus.liveness.check";
/// Network recovery — restoring consensus network connectivity after partition or node failure.
///
/// Span: `span.consensus.network.recovery`
/// Kind: `internal`
/// Stability: `development`
pub const CONSENSUS_NETWORK_RECOVERY_SPAN: &str = "consensus.network.recovery";
/// Network topology snapshot — capturing current consensus cluster topology for analysis and fault diagnosis.
///
/// Span: `span.consensus.network.topology`
/// Kind: `internal`
/// Stability: `development`
pub const CONSENSUS_NETWORK_TOPOLOGY_SPAN: &str = "consensus.network.topology";
/// Network partition recovery — restoring consensus after a partition splits the replica set.
///
/// Span: `span.consensus.partition.recover`
/// Kind: `internal`
/// Stability: `development`
pub const CONSENSUS_PARTITION_RECOVER_SPAN: &str = "consensus.partition.recover";
/// Quorum growth operation — adding new replicas to expand the consensus quorum size.
///
/// Span: `span.consensus.quorum.grow`
/// Kind: `internal`
/// Stability: `development`
pub const CONSENSUS_QUORUM_GROW_SPAN: &str = "consensus.quorum.grow";
/// Quorum shrink operation — removing nodes from the consensus quorum safely.
///
/// Span: `span.consensus.quorum.shrink`
/// Kind: `internal`
/// Stability: `development`
pub const CONSENSUS_QUORUM_SHRINK_SPAN: &str = "consensus.quorum.shrink";
/// Synchronization of a replica to catch up with the consensus leader.
///
/// Span: `span.consensus.replica.sync`
/// Kind: `internal`
/// Stability: `development`
pub const CONSENSUS_REPLICA_SYNC_SPAN: &str = "consensus.replica.sync";
/// A single round in the OSA HotStuff BFT consensus protocol.
///
/// Span: `span.consensus.round`
/// Kind: `internal`
/// Stability: `development`
pub const CONSENSUS_ROUND_SPAN: &str = "consensus.round";
/// Checking consensus safety — validating that quorum meets safety threshold before committing.
///
/// Span: `span.consensus.safety.check`
/// Kind: `internal`
/// Stability: `development`
pub const CONSENSUS_SAFETY_CHECK_SPAN: &str = "consensus.safety.check";
/// Ongoing safety monitoring — continuously verifies BFT safety invariants across replica set.
///
/// Span: `span.consensus.safety.monitor`
/// Kind: `internal`
/// Stability: `development`
pub const CONSENSUS_SAFETY_MONITOR_SPAN: &str = "consensus.safety.monitor";
/// Safety violation detected in the consensus protocol — double voting, equivocation, or quorum breach.
///
/// Span: `span.consensus.safety.violation`
/// Kind: `internal`
/// Stability: `development`
pub const CONSENSUS_SAFETY_VIOLATION_SPAN: &str = "consensus.safety.violation";
/// Consensus threshold adaptation — dynamically adjusting the quorum threshold based on observed fault rates and network conditions.
///
/// Span: `span.consensus.threshold.adapt`
/// Kind: `internal`
/// Stability: `development`
pub const CONSENSUS_THRESHOLD_ADAPT_SPAN: &str = "consensus.threshold.adapt";
/// Consensus threshold voting — executing a threshold-based vote among replicas.
///
/// Span: `span.consensus.threshold.vote`
/// Kind: `internal`
/// Stability: `development`
pub const CONSENSUS_THRESHOLD_VOTE_SPAN: &str = "consensus.threshold.vote";
/// View timeout event — current view timed out, triggering view change protocol.
///
/// Span: `span.consensus.timeout_event`
/// Kind: `internal`
/// Stability: `development`
pub const CONSENSUS_TIMEOUT_EVENT_SPAN: &str = "consensus.timeout_event";
/// View change event — leader timeout triggered, transitioning to new leader.
///
/// Span: `span.consensus.view_change`
/// Kind: `internal`
/// Stability: `development`
pub const CONSENSUS_VIEW_CHANGE_SPAN: &str = "consensus.view_change";
/// Optimized view change with exponential backoff — reduces thrashing during network instability.
///
/// Span: `span.consensus.view_change.optimize`
/// Kind: `internal`
/// Stability: `development`
pub const CONSENSUS_VIEW_CHANGE_OPTIMIZE_SPAN: &str = "consensus.view_change.optimize";
/// Casting or receiving a single vote in a HotStuff BFT round.
///
/// Span: `span.consensus.vote`
/// Kind: `internal`
/// Stability: `development`
pub const CONSENSUS_VOTE_SPAN: &str = "consensus.vote";
