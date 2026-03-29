//! Shared OTEL test helper for integration tests.
//!
//! Provides `TracerGuard` and `init_if_enabled()` for tests that
//! optionally initialise a real OTLP tracer provider when
//! `OTEL_EXPORTER_OTLP_ENDPOINT` is set in the environment.
//!
//! When the env var is absent the functions are no-ops, so tests
//! always compile and pass without a live collector.

use opentelemetry::global;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{propagation::TraceContextPropagator, runtime, trace as sdk_trace};
use std::time::Duration;

/// Holds the tracer provider for the lifetime of a test.
/// Shuts down the provider when dropped.
pub struct TracerGuard {
    _provider: sdk_trace::TracerProvider,
}

impl Drop for TracerGuard {
    fn drop(&mut self) {
        global::shutdown_tracer_provider();
    }
}

/// Initialise the OTLP tracer provider if `OTEL_EXPORTER_OTLP_ENDPOINT`
/// is set.  Returns `None` (no-op) when the env var is absent so that
/// unit / integration tests always pass without a live collector.
pub fn init_if_enabled() -> Option<TracerGuard> {
    let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok()?;

    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(endpoint)
        .with_timeout(Duration::from_secs(5))
        .build_span_exporter()
        .ok()?;

    let trace_config = sdk_trace::config().with_resource(opentelemetry_sdk::Resource::new(vec![
        opentelemetry::KeyValue::new("service.name", "pm4py-rust-test"),
    ]));

    let provider = sdk_trace::TracerProvider::builder()
        .with_batch_exporter(exporter, runtime::Tokio)
        .with_config(trace_config)
        .build();

    global::set_text_map_propagator(TraceContextPropagator::new());
    global::set_tracer_provider(provider.clone());

    Some(TracerGuard {
        _provider: provider,
    })
}
