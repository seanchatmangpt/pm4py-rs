/// Event correlation — linking multiple events into a causal chain for distributed tracing.
///
/// Span: `span.event.correlate`
/// Kind: `internal`
/// Stability: `development`
pub const EVENT_CORRELATE_SPAN: &str = "event.correlate";
/// Delivering an event to registered handlers in the event bus.
///
/// Span: `span.event.deliver`
/// Kind: `internal`
/// Stability: `development`
pub const EVENT_DELIVER_SPAN: &str = "event.deliver";
/// Emission of a structured log event to the event bus.
///
/// Span: `span.event.emit`
/// Kind: `producer`
/// Stability: `development`
pub const EVENT_EMIT_SPAN: &str = "event.emit";
/// Processing of a received structured log event from the bus.
///
/// Span: `span.event.process`
/// Kind: `consumer`
/// Stability: `development`
pub const EVENT_PROCESS_SPAN: &str = "event.process";
/// Event replay — re-processing a previously emitted event for recovery or audit.
///
/// Span: `span.event.replay`
/// Kind: `internal`
/// Stability: `development`
pub const EVENT_REPLAY_SPAN: &str = "event.replay";
/// Routing an event to subscribers based on routing strategy and filters.
///
/// Span: `span.event.route`
/// Kind: `internal`
/// Stability: `development`
pub const EVENT_ROUTE_SPAN: &str = "event.route";
