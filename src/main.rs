#[tokio::main]
async fn main() {
    // Initialize real OTLP tracer provider — registers global tracer so all
    // `global::tracer("pm4py-rust")` calls return a live (non-no-op) tracer.
    // WvdA Soundness: provider.shutdown() is called after serve() to release resources.
    let provider = pm4py::telemetry::init_tracer_provider();
    println!(
        "OpenTelemetry OTLP tracer provider initialised (endpoint: {})",
        std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:4317".to_string())
    );

    // Resolve port from environment — defaults to 8090
    let port = pm4py::http::businessos_api::pm4py_port();
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));

    println!("pm4py-rust starting on {}", addr);

    let app = pm4py::http::router();

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|e| {
            eprintln!("Failed to bind to {}: {}", addr, e);
            std::process::exit(1);
        });

    println!("pm4py-rust listening on http://{}", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            tokio::signal::ctrl_c().await.ok();
            #[cfg(feature = "mcp-server")]
            pm4py::http::mcp_state::mcp_cancellation_token().cancel();
        })
        .await
        .unwrap_or_else(|e| {
            eprintln!("Server error: {}", e);
            std::process::exit(1);
        });

    // Flush remaining spans and shut down the global tracer provider — WvdA resource cleanup.
    // provider is kept alive until here so the batch processor is not dropped early.
    drop(provider);
    opentelemetry::global::shutdown_tracer_provider();
}
