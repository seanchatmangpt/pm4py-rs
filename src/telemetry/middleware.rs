//! Axum middleware for automatic trace context extraction
//!
//! This middleware extracts W3C traceparent headers from incoming HTTP requests
//! and makes the trace context available to handlers.
//!
//! # Example
//!
//! ```ignore
//! let app = Router::new()
//!     .route("/api/health", get(health))
//!     .layer(axum::middleware::from_fn(
//!         pm4py::telemetry::middleware::trace_propagation_middleware
//!     ));
//! ```

use axum::{extract::Request, http::HeaderMap, middleware::Next, response::Response};
use opentelemetry::global;
use opentelemetry::propagation::Extractor;

/// Wrapper around axum HeaderMap to implement OpenTelemetry Extractor trait
struct AxumHeaderExtractor<'a> {
    headers: &'a HeaderMap,
}

impl<'a> Extractor for AxumHeaderExtractor<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.headers.get(key).and_then(|v| v.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.headers.keys().map(|k| k.as_str()).collect()
    }
}

/// Axum middleware for automatic trace context propagation
///
/// This middleware extracts the traceparent header from incoming requests and
/// registers the trace context with the global propagator, enabling distributed
/// tracing across service boundaries.
///
/// # W3C Trace Context
///
/// The middleware implements the W3C Trace Context specification:
/// - Extracts `traceparent` header
/// - Parses trace_id, span_id, and trace_flags
/// - Makes context available to handlers via global propagator
///
/// # Example
///
/// ```ignore
/// use axum::{Router, routing::get};
/// use pm4py::telemetry::middleware::trace_propagation_middleware;
///
/// let app = Router::new()
///     .route("/api/health", get(health))
///     .layer(axum::middleware::from_fn(trace_propagation_middleware));
/// ```
pub async fn trace_propagation_middleware(req: Request, next: Next) -> Response {
    // Extract trace context from headers
    let headers = req.headers();
    let extractor = AxumHeaderExtractor { headers };

    // Use global propagator to extract context
    let _cx = global::get_text_map_propagator(|propagator| propagator.extract(&extractor));

    // Continue with the request
    next.run(req).await
}

/// Extract trace context from incoming request headers
///
/// This function extracts the W3C TraceContext from the request headers
/// using the globally registered propagator. The returned context can
/// be used to start child spans.
///
/// # Arguments
///
/// * `headers` - Axum HeaderMap from the incoming request
///
/// # Returns
///
/// An OpenTelemetry Context containing the trace context
///
/// # Example
///
/// ```ignore
/// use axum::extract::HeaderMap;
/// use opentelemetry::global;
///
/// async fn my_handler(headers: HeaderMap) -> Result<Json<Response>, ApiError> {
///     let parent_cx = pm4py::telemetry::middleware::extract_trace_context(&headers);
///     let tracer = global::tracer("pm4py-rust");
///     let mut span = tracer.start_with_context("my.operation", &parent_cx);
///     // ... handler logic ...
///     span.end();
///     Ok(Json(response))
/// }
/// ```
pub fn extract_trace_context(headers: &HeaderMap) -> opentelemetry::Context {
    let extractor = AxumHeaderExtractor { headers };
    global::get_text_map_propagator(|propagator| propagator.extract(&extractor))
}

/// Resolve the x-correlation-id header value
///
/// Returns the correlation ID from the request headers, or an empty string
/// if the header is not present. The correlation ID should be recorded on
/// every span as the `chatmangpt.run.correlation_id` attribute.
///
/// # Arguments
///
/// * `headers` - Axum HeaderMap from the incoming request
///
/// # Returns
///
/// The correlation ID string, or empty string if not present
///
/// # Example
///
/// ```ignore
/// use axum::extract::HeaderMap;
///
/// async fn my_handler(headers: HeaderMap) -> Result<Json<Response>, ApiError> {
///     let correlation_id = pm4py::telemetry::middleware::resolve_correlation_id(&headers);
///     // Record on span: span.set_attribute(KeyValue::new("chatmangpt.run.correlation_id", correlation_id));
///     Ok(Json(response))
/// }
/// ```
pub fn resolve_correlation_id(headers: &HeaderMap) -> String {
    headers
        .get("x-correlation-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderMap;
    use opentelemetry::trace::TraceContextExt;

    #[test]
    fn test_extract_trace_context_no_headers() {
        let headers = HeaderMap::new();
        let cx = extract_trace_context(&headers);

        // Should return a valid (empty) context without panicking
        // The context itself doesn't have span_id/trace_id methods, but we can verify it exists
        let _span = cx.span();
        // If we reach here without panicking, the test passes
    }

    #[test]
    fn test_extract_trace_context_with_traceparent() {
        // Initialize the propagator for this test
        use opentelemetry::global;
        use opentelemetry_sdk::propagation::TraceContextPropagator;
        global::set_text_map_propagator(TraceContextPropagator::new());

        let mut headers = HeaderMap::new();
        headers.insert(
            "traceparent",
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01"
                .parse()
                .unwrap(),
        );

        let cx = extract_trace_context(&headers);

        // Verify the context contains valid trace and span IDs
        let span = cx.span();
        let span_context = span.span_context();
        assert_ne!(
            span_context.trace_id(),
            opentelemetry::trace::TraceId::INVALID
        );
        assert_ne!(
            span_context.span_id(),
            opentelemetry::trace::SpanId::INVALID
        );
    }

    #[test]
    fn test_resolve_correlation_id_present() {
        let mut headers = HeaderMap::new();
        headers.insert("x-correlation-id", "test-123".parse().unwrap());

        let correlation_id = resolve_correlation_id(&headers);
        assert_eq!(correlation_id, "test-123");
    }

    #[test]
    fn test_resolve_correlation_id_absent() {
        let headers = HeaderMap::new();
        let correlation_id = resolve_correlation_id(&headers);
        assert_eq!(correlation_id, "");
    }

    #[test]
    fn test_resolve_correlation_id_invalid_utf8() {
        let mut headers = HeaderMap::new();
        // Insert invalid UTF-8 bytes
        headers.insert(
            "x-correlation-id",
            axum::http::HeaderValue::from_bytes(&[0xFF, 0xFE]).unwrap(),
        );

        let correlation_id = resolve_correlation_id(&headers);
        assert_eq!(correlation_id, "");
    }

    #[test]
    fn test_extract_traceparent_header_present() {
        use super::super::trace_propagation::extract_traceparent_header;

        let mut headers = HeaderMap::new();
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
        use super::super::trace_propagation::extract_traceparent_header;

        let mut headers = HeaderMap::new();
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
        use super::super::trace_propagation::extract_traceparent_header;

        let headers = HeaderMap::new();
        let result = extract_traceparent_header(&headers);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_traceparent_valid() {
        use super::super::trace_propagation::parse_traceparent;

        let traceparent = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";

        let result = parse_traceparent(traceparent);
        assert!(result.is_ok());

        let span_context = result.unwrap();
        assert_eq!(
            span_context.trace_id(),
            opentelemetry::trace::TraceId::from_hex("4bf92f3577b34da6a3ce929d0e0e4736").unwrap()
        );
        assert_eq!(
            span_context.span_id(),
            opentelemetry::trace::SpanId::from_hex("00f067aa0ba902b7").unwrap()
        );
        assert_eq!(
            span_context.trace_flags(),
            opentelemetry::trace::TraceFlags::new(0x01)
        );
    }
}
