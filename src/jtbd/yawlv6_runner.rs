//! YAWL v6 checkpoint span emitter for Canopy JTBD runner integration.
//!
//! Emits an OTEL span named `jtbd.yawlv6.checkpoint` for each YAWL v6 workflow
//! checkpoint, recording the checkpoint name as a span attribute.
//!
//! This is a point-in-time marker — the span starts and ends immediately.

use opentelemetry::trace::{Span, Tracer};
use opentelemetry::KeyValue;

use crate::semconv::yawlv6_span_names::JTBD_YAWLV6_CHECKPOINT_SPAN;

/// Attribute key for the checkpoint name recorded on the span.
pub const CHECKPOINT_NAME_ATTR: &str = "checkpoint.name";

/// Emit a YAWL v6 checkpoint span.
///
/// Creates an OTEL span named `jtbd.yawlv6.checkpoint`, sets the
/// `checkpoint.name` attribute, and ends it immediately. This is a
/// point-in-time execution marker — no side effects beyond the span.
///
/// # Arguments
/// * `tracer` — OpenTelemetry tracer instance
/// * `checkpoint_name` — human-readable name for this checkpoint (e.g. `"process_start"`)
///
/// # WvdA
/// Pure instrumentation: deterministic, bounded, no blocking operations.
pub fn emit_checkpoint<T: Tracer>(tracer: &T, checkpoint_name: &str) {
    let mut span = tracer.start(JTBD_YAWLV6_CHECKPOINT_SPAN);
    span.set_attribute(KeyValue::new(
        CHECKPOINT_NAME_ATTR,
        checkpoint_name.to_string(),
    ));
    drop(span); // end span immediately — point-in-time marker
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkpoint_span_name_constant_matches_semconv() {
        // Verify the constant from semconv is the expected span name
        assert_eq!(JTBD_YAWLV6_CHECKPOINT_SPAN, "jtbd.yawlv6.checkpoint");
    }

    #[test]
    fn test_checkpoint_name_attr_key() {
        assert_eq!(CHECKPOINT_NAME_ATTR, "checkpoint.name");
    }
}
