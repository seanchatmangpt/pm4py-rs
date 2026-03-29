//! Example: Export pm4py-rust tracing spans to OpenTelemetry collector
//!
//! This example demonstrates how to:
//! 1. Create a pm4py Tracing instance
//! 2. Perform process mining operations (which create spans)
//! 3. Export the spans to an OpenTelemetry collector
//!
//! # Running the example
//!
//! Start an OTEL collector locally (e.g., using Docker):
//! ```bash
//! docker run -p 4317:4317 otel/opentelemetry-collector:latest
//! ```
//!
//! Then run this example:
//! ```bash
//! cargo run --example otel_export
//! ```

use pm4py::observability::{export_spans_to_otel, OtelExporterConfig, Tracing};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    log::info!("pm4py-rust OTEL Export Example");
    log::info!("================================");

    // Create a tracing instance
    let tracing = Tracing::new();
    tracing.init_tracer()?;

    // Start a root span for discovery
    let mut root_span = tracing.start_span("discovery", HashMap::new(), None)?;

    // Simulate some operations (in real code, this would be actual mining)
    {
        let mut attrs = HashMap::new();
        attrs.insert("format".to_string(), "xes".to_string());

        let mut log_span =
            tracing.start_span("log.load", attrs.clone(), Some(root_span.span_id.clone()))?;

        // Simulate I/O
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        tracing.end_span(&mut log_span, "ok", None)?;
        log::info!("Completed span: log.load");
    }

    {
        let mut attrs = HashMap::new();
        attrs.insert("algorithm".to_string(), "alpha".to_string());

        let mut discovery_span =
            tracing.start_span("algorithm.discover", attrs, Some(root_span.span_id.clone()))?;

        // Simulate processing
        tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
        tracing.end_span(&mut discovery_span, "ok", None)?;
        log::info!("Completed span: algorithm.discover");
    }

    // End the root span
    tracing.end_span(&mut root_span, "ok", None)?;

    // Get all spans
    let spans = tracing.get_spans();
    log::info!("Collected {} spans total", spans.len());

    // Initialize OTEL exporter with default config
    let config = OtelExporterConfig::default();
    log::info!("OTEL Exporter endpoint: {}", config.endpoint);
    log::info!("Service: {}", config.service_name);
    log::info!("Version: {}", config.service_version);

    // Export spans to OTEL collector
    log::info!("Exporting {} spans to OTEL collector...", spans.len());
    match export_spans_to_otel(&spans, &config).await {
        Ok(_) => {
            log::info!("Successfully exported spans to OTEL collector");
        }
        Err(e) => {
            log::warn!("OTEL export failed (collector may be unavailable): {}", e);
            log::info!(
                "Spans would have been exported to: {}/v1/traces",
                config.endpoint
            );
        }
    }

    // Get metrics
    let metrics = tracing.get_metrics();
    log::info!("Collected {} metrics total", metrics.len());
    for metric in metrics.iter() {
        log::info!(
            "  - {}: {} ({:?})",
            metric.name,
            metric.value,
            metric.metric_type
        );
    }

    log::info!("\nExample completed successfully!");
    Ok(())
}
