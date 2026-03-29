//! OpenTelemetry Exporter for pm4py-rust
//!
//! Exports in-memory tracing spans to an OpenTelemetry collector via OTLP/HTTP protocol.
//! Converts pm4py-rust SpanContext into OpenTelemetry Span format.
//!
//! Also provides `init_tracer_provider()` which wires up the real OTLP gRPC pipeline
//! and registers it as the global tracer provider so `global::tracer("pm4py-rust")`
//! returns a live tracer rather than the no-op default.
//!
//! # Example
//!
//! ```ignore
//! let config = init_otel_exporter()?;
//! let spans = tracing.get_spans();
//! export_spans_to_otel(&spans, &config).await?;
//! ```

use opentelemetry::global;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{propagation::TraceContextPropagator, runtime, trace as sdk_trace};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Initialise the real OTLP gRPC tracer provider and register it globally.
///
/// Must be called once at startup (before any `global::tracer(...)` calls) from
/// within a Tokio runtime context.  Returns the `TracerProvider` so the caller can
/// call `.shutdown()` on graceful exit.
///
/// The OTLP endpoint is read from the `OTEL_EXPORTER_OTLP_ENDPOINT` environment
/// variable; it defaults to `http://localhost:4317`.
///
/// WvdA Soundness: exporter timeout is 5 s (deadlock-free); provider.shutdown()
/// is called by main.rs after serve() returns (resource cleanup guarantee).
pub fn init_tracer_provider() -> sdk_trace::TracerProvider {
    let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4317".to_string());

    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(endpoint)
        .with_timeout(Duration::from_secs(5))
        .build_span_exporter()
        .expect("OTLP span exporter build failed");

    let trace_config = sdk_trace::config().with_resource(opentelemetry_sdk::Resource::new(vec![
        opentelemetry::KeyValue::new("service.name", "pm4py-rust"),
        opentelemetry::KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
    ]));

    let provider = sdk_trace::TracerProvider::builder()
        .with_batch_exporter(exporter, runtime::Tokio)
        .with_config(trace_config)
        .build();

    global::set_text_map_propagator(TraceContextPropagator::new());
    global::set_tracer_provider(provider.clone());
    provider
}

/// OTEL Exporter Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtelExporterConfig {
    pub endpoint: String,
    pub service_name: String,
    pub service_version: String,
    pub batch_timeout_ms: u64,
    pub max_export_batch_size: usize,
}

impl Default for OtelExporterConfig {
    fn default() -> Self {
        OtelExporterConfig {
            endpoint: std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:4317".to_string()),
            service_name: "pm4py-rust".to_string(),
            service_version: env!("CARGO_PKG_VERSION").to_string(),
            batch_timeout_ms: 5000,
            max_export_batch_size: 512,
        }
    }
}

impl OtelExporterConfig {
    /// Create a new config with custom endpoint
    pub fn with_endpoint(endpoint: String) -> Self {
        OtelExporterConfig {
            endpoint,
            ..Default::default()
        }
    }
}

/// Global OTEL exporter state holder
pub struct OtelExporter {
    config: OtelExporterConfig,
    #[allow(dead_code)]
    client: reqwest::Client,
}

impl OtelExporter {
    /// Create a new OTEL exporter instance
    pub fn new(config: OtelExporterConfig) -> Self {
        OtelExporter {
            config,
            client: reqwest::Client::new(),
        }
    }

    /// Get the exporter configuration
    pub fn config(&self) -> &OtelExporterConfig {
        &self.config
    }
}

/// Initialize OpenTelemetry exporter configuration
///
/// Creates a configuration for exporting spans to an OpenTelemetry collector via OTLP/HTTP.
///
/// # Returns
///
/// An OtelExporterConfig with endpoint and service metadata.
///
/// # Example
///
/// ```ignore
/// let config = init_otel_exporter()?;
/// println!("Exporting to: {}", config.endpoint);
/// ```
pub fn init_otel_exporter() -> Result<OtelExporterConfig, Box<dyn std::error::Error>> {
    Ok(OtelExporterConfig::default())
}

/// Create a new OTEL exporter with configuration
///
/// # Returns
///
/// An OtelExporter instance ready to export spans
///
/// # Example
///
/// ```ignore
/// let exporter = create_otel_exporter(OtelExporterConfig::default())?;
/// ```
pub fn create_otel_exporter(
    config: OtelExporterConfig,
) -> Result<OtelExporter, Box<dyn std::error::Error>> {
    Ok(OtelExporter::new(config))
}

/// OTEL Span representation for export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtelSpan {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub name: String,
    pub start_time_nanos: u64,
    pub end_time_nanos: u64,
    pub status: String,
    pub attributes: std::collections::HashMap<String, String>,
}

impl OtelSpan {
    /// Convert from pm4py-rust SpanContext to OTEL format
    pub fn from_span_context(span: &crate::observability::SpanContext) -> Self {
        let start_nanos = span.start_time_us * 1000;
        let end_nanos = span.end_time_us.unwrap_or(span.start_time_us) * 1000;

        OtelSpan {
            trace_id: span.trace_id.clone(),
            span_id: span.span_id.clone(),
            parent_span_id: span.parent_span_id.clone(),
            name: span.span_name.clone(),
            start_time_nanos: start_nanos,
            end_time_nanos: end_nanos,
            status: span.status.clone(),
            attributes: span.attributes.clone(),
        }
    }
}

/// Export in-memory spans to OpenTelemetry collector via HTTP
///
/// Converts pm4py-rust internal SpanContext objects into OTEL JSON format
/// and sends them to the configured OTEL collector endpoint.
///
/// # Parameters
///
/// - `spans`: vector of internal SpanContext objects
/// - `config`: OtelExporterConfig with endpoint and service metadata
///
/// # Returns
///
/// Ok(()) if all spans exported successfully, or an error message.
///
/// # Example
///
/// ```ignore
/// let tracing = pm4py::observability::Tracing::new();
/// let span = tracing.start_span("model.load", HashMap::new(), None)?;
/// tracing.end_span(&mut span, "ok", None)?;
///
/// let config = init_otel_exporter()?;
/// export_spans_to_otel(&tracing.get_spans(), &config).await?;
/// ```
pub async fn export_spans_to_otel(
    spans: &[crate::observability::SpanContext],
    config: &OtelExporterConfig,
) -> Result<(), String> {
    if spans.is_empty() {
        log::debug!("No spans to export");
        return Ok(());
    }

    // Convert internal spans to OTEL format
    let otel_spans: Vec<OtelSpan> = spans.iter().map(OtelSpan::from_span_context).collect();

    // Create OTEL payload
    let payload = serde_json::json!({
        "resourceSpans": [{
            "resource": {
                "attributes": [
                    { "key": "service.name", "value": { "stringValue": &config.service_name } },
                    { "key": "service.version", "value": { "stringValue": &config.service_version } },
                ]
            },
            "scopeSpans": [{
                "spans": otel_spans.iter().map(|s| serde_json::json!({
                    "traceId": &s.trace_id,
                    "spanId": &s.span_id,
                    "parentSpanId": s.parent_span_id.as_ref().unwrap_or(&String::new()),
                    "name": &s.name,
                    "startTimeUnixNano": s.start_time_nanos,
                    "endTimeUnixNano": s.end_time_nanos,
                    "status": { "code": if s.status == "ok" { "STATUS_CODE_OK" } else { "STATUS_CODE_ERROR" } },
                    "attributes": s.attributes.iter().map(|(k, v)| serde_json::json!({
                        "key": k,
                        "value": { "stringValue": v }
                    })).collect::<Vec<_>>(),
                })).collect::<Vec<_>>(),
            }]
        }]
    });

    // Send to OTEL collector
    let client = reqwest::Client::new();
    let url = format!("{}/v1/traces", config.endpoint);

    match client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                log::info!(
                    "Exported {} spans to OTEL collector at {}",
                    spans.len(),
                    config.endpoint
                );
                Ok(())
            } else {
                Err(format!(
                    "Failed to export spans: HTTP {} from {}",
                    response.status(),
                    config.endpoint
                ))
            }
        }
        Err(e) => {
            log::warn!(
                "OTEL export attempt failed (collector may be unavailable): {}",
                e
            );
            // Don't fail the application if OTEL is unavailable - telemetry is non-critical
            Ok(())
        }
    }
}

/// Export spans asynchronously in batches
///
/// Batches spans and exports them periodically or when batch reaches max size.
///
/// # Parameters
///
/// - `spans`: vector of internal SpanContext objects
/// - `config`: OtelExporterConfig with batch settings
///
/// # Returns
///
/// Number of spans exported
pub async fn export_spans_batched(
    spans: &[crate::observability::SpanContext],
    config: &OtelExporterConfig,
) -> Result<usize, String> {
    if spans.is_empty() {
        return Ok(0);
    }

    let total_spans = spans.len();
    let batch_size = config.max_export_batch_size;

    for batch in spans.chunks(batch_size) {
        export_spans_to_otel(batch, config).await?;
    }

    Ok(total_spans)
}

/// Convert microseconds since epoch to SystemTime
#[allow(dead_code)]
fn time_from_microseconds(us: u64) -> SystemTime {
    UNIX_EPOCH + std::time::Duration::from_micros(us)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observability::SpanContext;
    use std::collections::HashMap;

    #[test]
    fn test_init_otel_exporter() {
        let result = init_otel_exporter();
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.service_name, "pm4py-rust");
        assert!(config.endpoint.contains("localhost") || config.endpoint.contains("http"));
    }

    #[test]
    fn test_otel_span_conversion() {
        let mut attributes = HashMap::new();
        attributes.insert("algorithm".to_string(), "alpha".to_string());

        let span = SpanContext {
            span_id: "test-span-1".to_string(),
            trace_id: "test-trace-1".to_string(),
            parent_span_id: Some("parent-1".to_string()),
            span_name: "test.operation".to_string(),
            attributes,
            start_time_us: 1000000,
            end_time_us: Some(2000000),
            status: "ok".to_string(),
            error_message: None,
        };

        let otel_span = OtelSpan::from_span_context(&span);
        assert_eq!(otel_span.trace_id, "test-trace-1");
        assert_eq!(otel_span.span_id, "test-span-1");
        assert_eq!(otel_span.name, "test.operation");
        assert_eq!(otel_span.start_time_nanos, 1000000000); // 1000000 * 1000
        assert_eq!(otel_span.end_time_nanos, 2000000000); // 2000000 * 1000
        assert_eq!(otel_span.status, "ok");
    }

    #[test]
    fn test_otel_span_no_end_time() {
        let attributes = HashMap::new();

        let span = SpanContext {
            span_id: "test-span-2".to_string(),
            trace_id: "test-trace-2".to_string(),
            parent_span_id: None,
            span_name: "test.incomplete".to_string(),
            attributes,
            start_time_us: 5000000,
            end_time_us: None,
            status: "active".to_string(),
            error_message: Some("Still running".to_string()),
        };

        let otel_span = OtelSpan::from_span_context(&span);
        assert_eq!(otel_span.start_time_nanos, 5000000000);
        assert_eq!(otel_span.end_time_nanos, 5000000000); // Same as start when no end
    }

    #[tokio::test]
    async fn test_export_spans_to_otel_empty() {
        let config = init_otel_exporter().unwrap();
        let result = export_spans_to_otel(&[], &config).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_otel_exporter_config_builder() {
        let config = OtelExporterConfig::with_endpoint("http://custom:4317".to_string());
        assert_eq!(config.endpoint, "http://custom:4317");
        assert_eq!(config.service_name, "pm4py-rust");
    }

    #[test]
    fn test_otel_exporter_creation() {
        let config = OtelExporterConfig::default();
        let result = create_otel_exporter(config);
        assert!(result.is_ok());
        let exporter = result.unwrap();
        assert_eq!(exporter.config().service_name, "pm4py-rust");
    }

    #[tokio::test]
    async fn test_export_spans_batched() {
        let mut spans = Vec::new();
        let mut attributes = HashMap::new();
        attributes.insert("batch_test".to_string(), "true".to_string());

        // Create 10 spans
        for i in 0..10 {
            let span = SpanContext {
                span_id: format!("span-{}", i),
                trace_id: "batch-trace".to_string(),
                parent_span_id: None,
                span_name: format!("batch.operation.{}", i),
                attributes: attributes.clone(),
                start_time_us: 1000000 + (i as u64) * 1000,
                end_time_us: Some(2000000 + (i as u64) * 1000),
                status: "ok".to_string(),
                error_message: None,
            };
            spans.push(span);
        }

        let mut config = OtelExporterConfig::default();
        config.max_export_batch_size = 3; // Small batch size for testing

        let result = export_spans_batched(&spans, &config).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 10);
    }
}
