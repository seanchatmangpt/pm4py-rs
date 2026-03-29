//! OTel helpers for axum HTTP handlers.
//!
//! Provides W3C TraceContext extraction from axum `HeaderMap` and correlation ID
//! resolution.  The `opentelemetry-http` crate (v0.10) depends on `http` 0.2 while
//! axum 0.7 uses `http` 1.x, so we implement the `Extractor` trait locally rather
//! than wrapping `opentelemetry_http::HeaderExtractor`.

use axum::http::HeaderMap;
use opentelemetry::{global, propagation::Extractor, Context};

/// Wraps an axum `HeaderMap` (http 1.x) to implement `opentelemetry::propagation::Extractor`.
struct AxumHeaderExtractor<'a>(&'a HeaderMap);

impl<'a> Extractor for AxumHeaderExtractor<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|v| v.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(|k| k.as_str()).collect()
    }
}

/// Extract the W3C TraceContext from incoming request headers.
///
/// Uses the globally-registered propagator (set by `init_tracer_provider()` to
/// `TraceContextPropagator`).  Returns the extracted `Context` which should be
/// passed to `tracer.start_with_context(span_name, &parent_cx)` so the new span
/// becomes a child of the incoming trace.
pub fn extract_trace_context(headers: &HeaderMap) -> Context {
    global::get_text_map_propagator(|p| p.extract(&AxumHeaderExtractor(headers)))
}

/// Resolve the `x-correlation-id` header value, returning an empty string if absent.
///
/// The returned value should be recorded on every span as attribute
/// `chatmangpt.run.correlation_id`.
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

    #[test]
    fn test_resolve_correlation_id_present() {
        let mut headers = HeaderMap::new();
        headers.insert("x-correlation-id", "abc-123".parse().unwrap());
        assert_eq!(resolve_correlation_id(&headers), "abc-123");
    }

    #[test]
    fn test_resolve_correlation_id_absent() {
        let headers = HeaderMap::new();
        assert_eq!(resolve_correlation_id(&headers), "");
    }

    #[test]
    fn test_extract_trace_context_no_headers() {
        // With no traceparent header, returns an empty (root) context — no panic.
        let headers = HeaderMap::new();
        let _cx = extract_trace_context(&headers);
        // If we reach here without panicking the test passes.
    }

    #[test]
    fn test_axum_header_extractor_get() {
        let mut headers = HeaderMap::new();
        headers.insert("traceparent", "00-abc-def-01".parse().unwrap());
        let extractor = AxumHeaderExtractor(&headers);
        assert_eq!(extractor.get("traceparent"), Some("00-abc-def-01"));
        assert_eq!(extractor.get("missing"), None);
    }

    #[test]
    fn test_axum_header_extractor_keys() {
        let mut headers = HeaderMap::new();
        headers.insert("traceparent", "00-abc-def-01".parse().unwrap());
        headers.insert("tracestate", "vendor=value".parse().unwrap());
        let extractor = AxumHeaderExtractor(&headers);
        let keys = extractor.keys();
        assert!(keys.contains(&"traceparent"));
        assert!(keys.contains(&"tracestate"));
    }
}
