//! W3C Trace Context traceparent parsing and encoding
//!
//! Implements the W3C Trace Context specification for extracting and injecting
//! distributed trace context from HTTP headers.
//!
//! # traceparent Format
//!
//! The traceparent header format is: `version-trace_id-parent_span_id-trace_flags`
//!
//! Example: `00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01`
//!
//! - version: 00 (current version)
//! - trace_id: 32 hex characters (128-bit)
//! - parent_span_id: 16 hex characters (64-bit)
//! - trace_flags: 2 hex characters (8-bit)

use opentelemetry::trace::{SpanContext, SpanId, TraceFlags, TraceId, TraceState};

/// Parse a W3C traceparent header value into an OpenTelemetry SpanContext
///
/// # Arguments
///
/// * `traceparent` - The traceparent header value (e.g., "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01")
///
/// # Returns
///
/// * `Ok(SpanContext)` - Parsed span context
/// * `Err(String)` - Error message if parsing fails
///
/// # Example
///
/// ```ignore
/// let traceparent = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";
/// let span_context = parse_traceparent(traceparent)?;
/// ```
pub fn parse_traceparent(traceparent: &str) -> Result<SpanContext, String> {
    let parts: Vec<&str> = traceparent.split('-').collect();

    if parts.len() != 4 {
        return Err(format!(
            "Invalid traceparent format: expected 4 parts, got {}",
            parts.len()
        ));
    }

    // Validate version
    let version = parts[0];
    if version != "00" {
        return Err(format!("Unsupported traceparent version: {}", version));
    }

    // Parse trace_id (32 hex chars)
    let trace_id = TraceId::from_hex(parts[1]).map_err(|e| format!("Invalid trace_id: {}", e))?;

    // Parse span_id (16 hex chars)
    let span_id = SpanId::from_hex(parts[2]).map_err(|e| format!("Invalid span_id: {}", e))?;

    // Parse trace_flags (2 hex chars)
    let flags_byte =
        u8::from_str_radix(parts[3], 16).map_err(|e| format!("Invalid trace_flags: {}", e))?;

    let trace_flags = TraceFlags::new(flags_byte);

    Ok(SpanContext::new(
        trace_id,
        span_id,
        trace_flags,
        false, // is_remote
        TraceState::default(),
    ))
}

/// Encode an OpenTelemetry SpanContext into a W3C traceparent header value
///
/// # Arguments
///
/// * `span_context` - The span context to encode
///
/// # Returns
///
/// A traceparent header string
///
/// # Example
///
/// ```ignore
/// let span_context = SpanContext::new(trace_id, span_id, trace_flags, false, TraceState::default());
/// let traceparent = encode_traceparent(&span_context);
/// assert!(traceparent.starts_with("00-"));
/// ```
pub fn encode_traceparent(span_context: &SpanContext) -> String {
    let trace_id = span_context.trace_id().to_string();
    let span_id = span_context.span_id().to_string();
    let flags = format!("{:02x}", span_context.trace_flags().to_u8());

    format!("00-{}-{}-{}", trace_id, span_id, flags)
}

/// Extract the traceparent value from a header map
///
/// This helper function extracts the `traceparent` header value, handling
/// both lowercase and capitalized variants.
///
/// # Arguments
///
/// * `headers` - Axum HeaderMap
///
/// # Returns
///
/// * `Some(String)` - The traceparent value if present
/// * `None` - No traceparent header found
///
/// # Example
///
/// ```ignore
/// let headers = HeaderMap::new();
/// let traceparent = extract_traceparent_header(&headers);
/// ```
pub fn extract_traceparent_header(headers: &axum::http::HeaderMap) -> Option<String> {
    // Try lowercase first (W3C standard)
    if let Some(value) = headers.get("traceparent") {
        return value.to_str().ok().map(|s| s.to_string());
    }

    // Fallback to capitalized (some implementations use this)
    if let Some(value) = headers.get("Traceparent") {
        return value.to_str().ok().map(|s| s.to_string());
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry::trace::{SpanId, TraceFlags, TraceId};

    #[test]
    fn test_parse_traceparent_valid() {
        let traceparent = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";

        let result = parse_traceparent(traceparent);
        assert!(result.is_ok());

        let span_context = result.unwrap();
        assert_eq!(
            span_context.trace_id(),
            TraceId::from_hex("4bf92f3577b34da6a3ce929d0e0e4736").unwrap()
        );
        assert_eq!(
            span_context.span_id(),
            SpanId::from_hex("00f067aa0ba902b7").unwrap()
        );
        assert_eq!(span_context.trace_flags(), TraceFlags::new(0x01));
    }

    #[test]
    fn test_parse_traceparent_invalid_format() {
        let result = parse_traceparent("invalid");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("expected 4 parts"));
    }

    #[test]
    fn test_parse_traceparent_invalid_version() {
        let result = parse_traceparent("01-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Unsupported traceparent version"));
    }

    #[test]
    fn test_parse_traceparent_invalid_trace_id() {
        let result = parse_traceparent("00-invalid-00f067aa0ba902b7-01");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid trace_id"));
    }

    #[test]
    fn test_parse_traceparent_invalid_span_id() {
        let result = parse_traceparent("00-4bf92f3577b34da6a3ce929d0e0e4736-invalid-01");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid span_id"));
    }

    #[test]
    fn test_encode_traceparent() {
        let trace_id = TraceId::from_hex("4bf92f3577b34da6a3ce929d0e0e4736").unwrap();
        let span_id = SpanId::from_hex("00f067aa0ba902b7").unwrap();
        let trace_flags = TraceFlags::new(0x01);

        let span_context =
            SpanContext::new(trace_id, span_id, trace_flags, false, TraceState::default());
        let encoded = encode_traceparent(&span_context);

        assert_eq!(
            encoded,
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01"
        );
    }

    #[test]
    fn test_encode_traceparent_sampled() {
        let trace_id = TraceId::from_hex("4bf92f3577b34da6a3ce929d0e0e4736").unwrap();
        let span_id = SpanId::from_hex("00f067aa0ba902b7").unwrap();
        let trace_flags = TraceFlags::SAMPLED;

        let span_context =
            SpanContext::new(trace_id, span_id, trace_flags, false, TraceState::default());
        let encoded = encode_traceparent(&span_context);

        assert!(encoded.ends_with("-01")); // 01 = sampled
    }

    #[test]
    fn test_extract_traceparent_header_present() {
        let mut headers = axum::http::HeaderMap::new();
        headers.insert(
            "traceparent",
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01"
                .parse()
                .unwrap(),
        );

        let result = extract_traceparent_header(&headers);
        assert!(result.is_some());
        assert_eq!(
            result.unwrap(),
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01"
        );
    }

    #[test]
    fn test_extract_traceparent_header_capitalized() {
        let mut headers = axum::http::HeaderMap::new();
        headers.insert(
            "Traceparent",
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01"
                .parse()
                .unwrap(),
        );

        let result = extract_traceparent_header(&headers);
        assert!(result.is_some());
    }

    #[test]
    fn test_extract_traceparent_header_absent() {
        let headers = axum::http::HeaderMap::new();
        let result = extract_traceparent_header(&headers);
        assert!(result.is_none());
    }

    #[test]
    fn test_roundtrip_traceparent() {
        let original = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";

        let span_context = parse_traceparent(original).unwrap();
        let encoded = encode_traceparent(&span_context);

        assert_eq!(original, encoded);
    }
}
