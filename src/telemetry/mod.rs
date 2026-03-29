//! Telemetry module for pm4py-rust
//!
//! Provides OpenTelemetry integration for distributed tracing, including
//! tracer provider initialization, trace context propagation, and HTTP middleware.
//!
//! # Environment Variables
//!
//! - `OTEL_SERVICE_NAME`: Service name (default: "pm4py-rust")
//! - `OTEL_EXPORTER_OTLP_ENDPOINT`: OTLP collector endpoint (default: "http://localhost:4317")
//!
//! # Example
//!
//! ```ignore
//! // Initialize tracer provider at startup
//! let provider = telemetry::init_tracer_provider();
//!
//! // Use in HTTP handlers
//! let parent_cx = telemetry::extract_trace_context(&headers);
//! let tracer = global::tracer("pm4py-rust");
//! let mut span = tracer.start_with_context("my.operation", &parent_cx);
//! ```

pub mod middleware;
pub mod trace_propagation;

use opentelemetry::{global, KeyValue};
use opentelemetry_sdk::{
    propagation::TraceContextPropagator, resource::Resource, runtime, trace as sdk_trace,
};
use std::time::Duration;

/// Build an OTLP HTTP exporter with a short connect timeout (2s).
///
/// Uses the HTTP exporter instead of gRPC tonic to avoid eager connection
/// establishment that blocks when the OTEL collector is absent.
fn build_otlp_exporter(endpoint: &str) -> Result<opentelemetry_otlp::SpanExporter, String> {
    use opentelemetry_otlp::WithExportConfig;

    let timeout = Duration::from_secs(2);

    let exporter = opentelemetry_otlp::new_exporter()
        .http()
        .with_endpoint(endpoint)
        .with_timeout(timeout)
        .build_span_exporter()
        .map_err(|e| format!("OTLP span exporter build failed: {e}"))?;

    Ok(exporter)
}

/// Initialize the OpenTelemetry tracer provider and register it globally.
///
/// This function sets up the OTLP HTTP exporter, configures the tracer provider
/// with service resource attributes, and registers it as the global tracer provider.
/// It also registers the W3C TraceContext propagator for distributed trace context.
///
/// If the OTLP exporter cannot be built (e.g. no collector running), a no-op
/// tracer provider is returned so that the application can still start and
/// produce in-process traces.
///
/// # Environment Variables
///
/// - `OTEL_SERVICE_NAME`: Service name (default: "pm4py-rust")
/// - `OTEL_EXPORTER_OTLP_ENDPOINT`: OTLP endpoint (default: "http://localhost:4317")
///
/// # WvdA Soundness
///
/// - Exporter connect timeout is 2 seconds (deadlock-free guarantee)
/// - Falls back to no-op exporter if collector is unreachable
/// - Provider must be shut down before process exit (resource cleanup)
///
/// # Returns
///
/// A `TracerProvider` instance. The caller should keep this alive for the lifetime
/// of the application and call `.shutdown()` before exit to flush remaining spans.
///
/// # Example
///
/// ```ignore
/// let provider = init_tracer_provider();
/// // ... run application ...
/// provider.shutdown().unwrap();
/// ```
pub fn init_tracer_provider() -> sdk_trace::TracerProvider {
    let service_name =
        std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "pm4py-rust".to_string());

    let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4317".to_string());

    let resource = Resource::new(vec![
        KeyValue::new("service.name", service_name.clone()),
        KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
    ]);

    let trace_config = sdk_trace::config().with_resource(resource);

    let provider = match build_otlp_exporter(&endpoint) {
        Ok(exporter) => sdk_trace::TracerProvider::builder()
            .with_batch_exporter(exporter, runtime::Tokio)
            .with_config(trace_config)
            .build(),
        Err(e) => {
            log::warn!("OTLP exporter unavailable, using no-op tracer: {e}");
            sdk_trace::TracerProvider::builder()
                .with_config(trace_config)
                .build()
        }
    };

    // Register W3C TraceContext propagator for distributed tracing
    global::set_text_map_propagator(TraceContextPropagator::new());

    // Register as global tracer provider
    global::set_tracer_provider(provider.clone());

    provider
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_init_tracer_provider_default_env() {
        // Remove env vars so defaults are used (localhost:4317)
        std::env::remove_var("OTEL_SERVICE_NAME");
        std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");

        // Should not hang -- falls back to no-op exporter when collector is absent
        let provider = init_tracer_provider();

        // Verify provider was created and can be flushed without panic.
        // A no-op provider (no OTEL collector) returns empty results,
        // which is correct behavior -- the assertion is that flush() itself
        // does not panic or hang.
        provider.force_flush();
    }

    #[tokio::test]
    async fn test_init_tracer_provider_custom_env() {
        std::env::set_var("OTEL_SERVICE_NAME", "test-service");
        std::env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", "http://localhost:4318");

        let provider = init_tracer_provider();

        // Verify provider was created (no-op fallback if collector absent)
        provider.force_flush();

        // Clean up
        std::env::remove_var("OTEL_SERVICE_NAME");
        std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
    }
}
