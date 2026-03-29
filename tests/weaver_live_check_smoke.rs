//! Weaver live-check smoke: exports one semconv span via OTLP gRPC (tonic) to the
//! receiver started by `semconv/live-check/run-all-live-checks.sh`.
//!
//! Set `WEAVER_LIVE_CHECK=true` and `WEAVER_OTLP_ENDPOINT` (default `http://localhost:4317`).

use std::time::Duration;

use opentelemetry::trace::{TraceContextExt, TraceError, Tracer};
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::Config;
use opentelemetry_sdk::Resource;

fn env_bool(name: &str) -> bool {
    std::env::var(name)
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false)
}

#[tokio::test(flavor = "current_thread")]
async fn weaver_live_check_smoke_exports_one_span() {
    // Intended for the live-check pipeline; skip when unset so local `cargo test` stays quiet.
    if !env_bool("WEAVER_LIVE_CHECK") {
        return;
    }

    let weaver_endpoint =
        std::env::var("WEAVER_OTLP_ENDPOINT").unwrap_or_else(|_| "http://localhost:4317".into());

    let mut resource_kvs = vec![
        KeyValue::new("service.name", "pm4py-rust"),
        KeyValue::new("service.namespace", "chatmangpt"),
    ];
    let cid_opt = std::env::var("CHATMANGPT_CORRELATION_ID")
        .ok()
        .filter(|s| !s.is_empty());
    if let Some(ref cid) = cid_opt {
        resource_kvs.push(KeyValue::new("chatmangpt.run.correlation_id", cid.clone()));
    }
    let resource = Resource::new(resource_kvs);

    let exporter_res: Result<_, TraceError> = (|| {
        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_trace_config(Config::default().with_resource(resource))
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint(weaver_endpoint),
            )
            .install_simple()?;

        Ok(tracer)
    })();

    let tracer = match exporter_res {
        Ok(t) => t,
        Err(e) => panic!("opentelemetry otlp exporter init failed: {e}"),
    };

    let span_name = "span.process.mining.discovery";
    let algo = "inductive";
    let log_path = "/tmp/pm4py-rust-live-check.xes";

    tracer.in_span(span_name, |cx| {
        if let Some(ref cid) = cid_opt {
            cx.span()
                .set_attribute(KeyValue::new("chatmangpt.run.correlation_id", cid.clone()));
        }
        cx.span()
            .set_attribute(KeyValue::new("process.mining.algorithm", algo));
        cx.span()
            .set_attribute(KeyValue::new("process.mining.log_path", log_path));
    });

    // Allow the simple processor to export before the test process exits.
    tokio::time::sleep(Duration::from_millis(150)).await;
}
