/// Adaptive threshold adjustment — updates the healing detection threshold based on observed system behavior.
///
/// Span: `span.healing.adaptive.adjust`
/// Kind: `internal`
/// Stability: `development`
pub const HEALING_ADAPTIVE_ADJUST_SPAN: &str = "healing.adaptive.adjust";
/// Anomaly detection scan — identifies abnormal system behavior patterns for healing intervention.
///
/// Span: `span.healing.anomaly.detect`
/// Kind: `internal`
/// Stability: `development`
pub const HEALING_ANOMALY_DETECT_SPAN: &str = "healing.anomaly.detect";
/// Backpressure application — managing healing request flow under system overload.
///
/// Span: `span.healing.backpressure.apply`
/// Kind: `internal`
/// Stability: `development`
pub const HEALING_BACKPRESSURE_APPLY_SPAN: &str = "healing.backpressure.apply";
/// Detecting cascade failure pattern — identifying correlated failures and root cause.
///
/// Span: `span.healing.cascade.detect`
/// Kind: `internal`
/// Stability: `development`
pub const HEALING_CASCADE_DETECT_SPAN: &str = "healing.cascade.detect";
/// Healing checkpoint creation — capturing system state as a recovery checkpoint before risky operations.
///
/// Span: `span.healing.checkpoint.create`
/// Kind: `internal`
/// Stability: `development`
pub const HEALING_CHECKPOINT_CREATE_SPAN: &str = "healing.checkpoint.create";
/// Circuit breaker state transition — healing subsystem trips open to prevent cascade failures.
///
/// Span: `span.healing.circuit_breaker.trip`
/// Kind: `internal`
/// Stability: `development`
pub const HEALING_CIRCUIT_BREAKER_TRIP_SPAN: &str = "healing.circuit_breaker.trip";
/// Cold standby promotion — warming up and promoting a cold replica to primary during a healing failover.
///
/// Span: `span.healing.cold_standby.promote`
/// Kind: `internal`
/// Stability: `development`
pub const HEALING_COLD_STANDBY_PROMOTE_SPAN: &str = "healing.cold_standby.promote";
/// Classifies a system failure into a known failure mode with a confidence score.
///
/// Span: `span.healing.diagnosis`
/// Kind: `internal`
/// Stability: `development`
pub const HEALING_DIAGNOSIS_SPAN: &str = "healing.diagnosis";
/// Escalation to human operator when healing max attempts exceeded.
///
/// Span: `span.healing.escalation`
/// Kind: `internal`
/// Stability: `development`
pub const HEALING_ESCALATION_SPAN: &str = "healing.escalation";
/// Healing failover execution — transitioning service from a failing component to a standby replacement.
///
/// Span: `span.healing.failover.execute`
/// Kind: `internal`
/// Stability: `development`
pub const HEALING_FAILOVER_EXECUTE_SPAN: &str = "healing.failover.execute";
/// Process fingerprinting — computes a failure signature for pattern matching.
///
/// Span: `span.healing.fingerprint`
/// Kind: `internal`
/// Stability: `development`
pub const HEALING_FINGERPRINT_SPAN: &str = "healing.fingerprint";
/// Healing intervention scoring — evaluates the effectiveness of a completed healing intervention.
///
/// Span: `span.healing.intervention.score`
/// Kind: `internal`
/// Stability: `development`
pub const HEALING_INTERVENTION_SCORE_SPAN: &str = "healing.intervention.score";
/// Load shedding application — intentionally dropping requests to protect the system under overload conditions.
///
/// Span: `span.healing.load_shedding.apply`
/// Kind: `internal`
/// Stability: `development`
pub const HEALING_LOAD_SHEDDING_APPLY_SPAN: &str = "healing.load_shedding.apply";
/// Memory snapshot — capturing the current system state to enable fast recovery during healing.
///
/// Span: `span.healing.memory.snapshot`
/// Kind: `internal`
/// Stability: `development`
pub const HEALING_MEMORY_SNAPSHOT_SPAN: &str = "healing.memory.snapshot";
/// Measuring MTTR for a completed healing cycle — from failure detection to full recovery.
///
/// Span: `span.healing.mttr.measure`
/// Kind: `internal`
/// Stability: `development`
pub const HEALING_MTTR_MEASURE_SPAN: &str = "healing.mttr.measure";
/// Matching a failure signature against the healing pattern library to identify recovery action.
///
/// Span: `span.healing.pattern.match`
/// Kind: `internal`
/// Stability: `development`
pub const HEALING_PATTERN_MATCH_SPAN: &str = "healing.pattern.match";
/// Execution of a healing recovery playbook — structured series of remediation steps.
///
/// Span: `span.healing.playbook.execute`
/// Kind: `internal`
/// Stability: `development`
pub const HEALING_PLAYBOOK_EXECUTE_SPAN: &str = "healing.playbook.execute";
/// Predictive healing — forecasts failure probability within a time horizon using ML model.
///
/// Span: `span.healing.prediction.make`
/// Kind: `internal`
/// Stability: `development`
pub const HEALING_PREDICTION_MAKE_SPAN: &str = "healing.prediction.make";
/// Quarantine application — isolating a component to prevent cascade failures during healing.
///
/// Span: `span.healing.quarantine.apply`
/// Kind: `internal`
/// Stability: `development`
pub const HEALING_QUARANTINE_APPLY_SPAN: &str = "healing.quarantine.apply";
/// Rate limit enforcement — throttling healing attempts to prevent cascade recovery storms.
///
/// Span: `span.healing.rate_limit.enforce`
/// Kind: `internal`
/// Stability: `development`
pub const HEALING_RATE_LIMIT_ENFORCE_SPAN: &str = "healing.rate_limit.enforce";
/// Recovery simulation — running synthetic failure scenarios to validate healing playbooks and reflex arcs.
///
/// Span: `span.healing.recovery.simulate`
/// Kind: `internal`
/// Stability: `development`
pub const HEALING_RECOVERY_SIMULATE_SPAN: &str = "healing.recovery.simulate";
/// Bounded recovery loop execution — WvdA liveness-bounded healing iteration.
///
/// Span: `span.healing.recovery_loop`
/// Kind: `internal`
/// Stability: `development`
pub const HEALING_RECOVERY_LOOP_SPAN: &str = "healing.recovery_loop";
/// Execution of a healing reflex arc — automated recovery action triggered by a detected failure pattern.
///
/// Span: `span.healing.reflex_arc`
/// Kind: `internal`
/// Stability: `development`
pub const HEALING_REFLEX_ARC_SPAN: &str = "healing.reflex_arc";
/// Adaptive retry backoff execution — applying dynamic retry strategy during healing.
///
/// Span: `span.healing.retry.adaptive`
/// Kind: `internal`
/// Stability: `development`
pub const HEALING_RETRY_ADAPTIVE_SPAN: &str = "healing.retry.adaptive";
/// Rollback execution — reverting the system to a known-good checkpoint or snapshot after a healing failure.
///
/// Span: `span.healing.rollback.execute`
/// Kind: `internal`
/// Stability: `development`
pub const HEALING_ROLLBACK_EXECUTE_SPAN: &str = "healing.rollback.execute";
/// Triggering an autonomous self-healing action in response to a detected failure.
///
/// Span: `span.healing.self_healing.trigger`
/// Kind: `internal`
/// Stability: `development`
pub const HEALING_SELF_HEALING_TRIGGER_SPAN: &str = "healing.self_healing.trigger";
/// Detecting a healing surge and applying mitigation strategy.
///
/// Span: `span.healing.surge.detect`
/// Kind: `internal`
/// Stability: `development`
pub const HEALING_SURGE_DETECT_SPAN: &str = "healing.surge.detect";
/// Warm standby activation — promoting a warm replica to primary during a healing failover event.
///
/// Span: `span.healing.warm_standby.activate`
/// Kind: `internal`
/// Stability: `development`
pub const HEALING_WARM_STANDBY_ACTIVATE_SPAN: &str = "healing.warm_standby.activate";
