// Span name constants for use with opentelemetry tracer.
//
// These constants define the canonical span names emitted by pm4py-rust
// operations. Use them when starting a span to ensure names match the
// OTEL schema and appear correctly in Jaeger / OTLP backends.
//
// Example usage (when opentelemetry tracer is initialised):
//
// ```rust
// use opentelemetry::trace::Tracer;
// use pm4py::semconv::spans;
//
// let tracer = opentelemetry::global::tracer("pm4py-rust");
// let span = tracer.start(spans::PROCESS_MINING_DISCOVERY);
// ```

// ── Process Mining ──────────────────────────────────────────────────────────

/// Span emitted when a process discovery algorithm runs against an event log.
/// Pair with `process_attributes::PROCESS_MINING_ALGORITHM` and
/// `process_attributes::PROCESS_MINING_LOG_PATH` attributes.
pub const PROCESS_MINING_DISCOVERY: &str = "process.mining.discovery";

/// Span emitted when an XES / CSV event log is parsed from disk or stream.
pub const PROCESS_MINING_LOG_PARSE: &str = "process.mining.log.parse";

/// Span emitted when DFG (Directly-Follows Graph) statistics are computed.
pub const PROCESS_MINING_DFG_COMPUTE: &str = "process.mining.dfg.compute";

/// Span emitted when a single conformance deviation is detected during trace alignment.
/// Pair with `process_attributes::PROCESS_MINING_CONFORMANCE_DEVIATION_TYPE` and
/// `conformance_attributes::CONFORMANCE_FITNESS`.
pub const PROCESS_MINING_CONFORMANCE_DEVIATION: &str = "process.mining.conformance.deviation";

// ── Conformance ─────────────────────────────────────────────────────────────

/// Span emitted when a token-replay or alignment conformance check runs.
/// Pair with `conformance_attributes::CONFORMANCE_FITNESS` and
/// `conformance_attributes::CONFORMANCE_PRECISION` attributes.
pub const CONFORMANCE_CHECK: &str = "conformance.check";

/// Span emitted for a single trace alignment computation.
pub const CONFORMANCE_TRACE_ALIGN: &str = "conformance.trace.align";

/// Span emitted when precision / generalisation metrics are computed.
pub const CONFORMANCE_METRICS_COMPUTE: &str = "conformance.metrics.compute";

// ── Workflow ─────────────────────────────────────────────────────────────────

/// Span emitted when a YAWL workflow instance starts executing.
/// Pair with `workflow_attributes::WORKFLOW_ID` and `WORKFLOW_PATTERN`.
pub const WORKFLOW_EXECUTE: &str = "workflow.execute";

/// Span emitted per workflow step (activity transition).
pub const WORKFLOW_STEP_EXECUTE: &str = "workflow.step.execute";

/// Span emitted for N-out-of-M join evaluation — fires when N of M branches complete.
/// Pair with `workflow_attributes::WORKFLOW_REQUIRED_BRANCHES` and
/// `workflow_attributes::WORKFLOW_TOTAL_BRANCHES`.
pub const WORKFLOW_DISCRIMINATOR: &str = "workflow.discriminator";

// ── Healing ─────────────────────────────────────────────────────────────────

/// Span emitted when the healing agent diagnoses a failure mode.
/// Pair with `healing_attributes::HEALING_FAILURE_MODE` and
/// `healing_attributes::HEALING_CONFIDENCE`.
pub const HEALING_DIAGNOSIS: &str = "healing.diagnosis";

/// Span emitted when a reflex arc recovery action is applied.
pub const HEALING_RECOVERY: &str = "healing.recovery";

/// Span emitted when the healing engine computes a failure fingerprint for pattern matching.
/// Pair with `healing_attributes::HEALING_FINGERPRINT` and
/// `healing_attributes::HEALING_FAILURE_MODE`.
pub const HEALING_FINGERPRINT: &str = "healing.fingerprint";

/// Span emitted when the healing engine escalates to a human operator after
/// exhausting max retry attempts.
/// Pair with `healing_attributes::HEALING_ATTEMPT_NUMBER` and
/// `healing_attributes::HEALING_MAX_ATTEMPTS`.
pub const HEALING_ESCALATION: &str = "healing.escalation";

// ── BPMN ─────────────────────────────────────────────────────────────────────

/// Span emitted when a ProcessTree is converted to a BPMN diagram.
/// Pair with `process_attributes::PROCESS_MINING_ALGORITHM`.
pub const PROCESS_MINING_BPMN_CONVERT: &str = "process.mining.bpmn.convert";
