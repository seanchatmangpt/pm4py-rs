/// BOS ↔ BusinessOS Gateway Performance Benchmark
///
/// Measures latency, throughput, and resource usage for cross-system data flow.
/// Run with: `cargo bench --bench businessos_gateway_perf`
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Mock BOS gateway for benchmarking
mod mock_gateway {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::sync::Mutex;

    /// Event structure for serialization testing
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    pub struct Event {
        pub id: String,
        pub timestamp: u64,
        pub activity: String,
        pub resource: String,
        pub metadata: HashMap<String, String>,
    }

    /// Mock gateway client
    pub struct GatewayClient {
        events: Arc<Mutex<Vec<Event>>>,
        rpc_latency_ms: u64,
    }

    impl GatewayClient {
        pub fn new(rpc_latency_ms: u64) -> Self {
            GatewayClient {
                events: Arc::new(Mutex::new(Vec::new())),
                rpc_latency_ms,
            }
        }

        /// Simulate HTTP RPC call to BusinessOS
        pub fn send_event(&self, event: Event) -> Result<String, String> {
            // Simulate network latency
            let start = std::time::Instant::now();

            // Simulate serialization overhead (JSON encoding)
            let json = serde_json::to_string(&event)
                .map_err(|e| format!("Serialization failed: {}", e))?;

            // Simulate network round-trip
            std::thread::sleep(std::time::Duration::from_millis(self.rpc_latency_ms));

            // Simulate deserialization (server-side)
            let _: Event = serde_json::from_str(&json)
                .map_err(|e| format!("Deserialization failed: {}", e))?;

            // Store for validation
            let mut events = self.events.lock().unwrap();
            let event_id = event.id.clone();
            events.push(event);

            Ok(event_id)
        }

        /// Batch send events (optimized path)
        pub fn send_batch(&self, events: Vec<Event>) -> Result<Vec<String>, String> {
            let start = std::time::Instant::now();

            // Single JSON serialization for batch
            let json = serde_json::to_string(&events)
                .map_err(|e| format!("Batch serialization failed: {}", e))?;

            // Single network round-trip
            std::thread::sleep(std::time::Duration::from_millis(self.rpc_latency_ms));

            // Single deserialization
            let _: Vec<Event> = serde_json::from_str(&json)
                .map_err(|e| format!("Batch deserialization failed: {}", e))?;

            // Store and return IDs
            let ids: Vec<String> = events.iter().map(|e| e.id.clone()).collect();
            let mut stored = self.events.lock().unwrap();
            stored.extend(events);

            Ok(ids)
        }

        /// Simulate WebSocket streaming (low-latency)
        pub fn stream_event(&self, event: Event) -> Result<String, String> {
            // WebSocket serialization (faster, no full HTTP overhead)
            let json = serde_json::to_string(&event)
                .map_err(|e| format!("Stream serialization failed: {}", e))?;

            // WebSocket latency: ~10ms instead of 50ms for HTTP
            std::thread::sleep(std::time::Duration::from_millis(self.rpc_latency_ms / 5));

            let event_id = event.id.clone();
            let mut events = self.events.lock().unwrap();
            events.push(event);

            Ok(event_id)
        }

        /// Query events (simulate database round-trip)
        pub fn query_events(&self, limit: usize) -> Result<Vec<Event>, String> {
            // Database query latency: ~50ms for indexed query
            std::thread::sleep(std::time::Duration::from_millis(50));

            let events = self.events.lock().unwrap();
            Ok(events.iter().take(limit).cloned().collect())
        }

        /// Clear stored events
        pub fn clear(&self) {
            self.events.lock().unwrap().clear();
        }

        /// Get event count
        pub fn event_count(&self) -> usize {
            self.events.lock().unwrap().len()
        }
    }
}

/// Helper to create test event
fn create_event(id: u32) -> mock_gateway::Event {
    let mut metadata = HashMap::new();
    metadata.insert("source".to_string(), "process_mining".to_string());
    metadata.insert("model_id".to_string(), format!("model_{}", id % 10));

    mock_gateway::Event {
        id: format!("evt_{}", id),
        timestamp: 1700000000 + id as u64,
        activity: format!("activity_{}", id % 20),
        resource: format!("resource_{}", id % 5),
        metadata,
    }
}

/// Benchmark: Single event RPC latency (100x calls)
fn bench_rpc_latency_100x(c: &mut Criterion) {
    let mut group = c.benchmark_group("rpc_latency");
    group.sample_size(100);

    for rpc_latency in [10, 50, 100].iter() {
        group.throughput(Throughput::Elements(100));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}ms", rpc_latency)),
            rpc_latency,
            |b, &latency| {
                let gateway = mock_gateway::GatewayClient::new(latency);

                b.iter(|| {
                    for i in 0..100 {
                        let event = create_event(i);
                        let _ = gateway.send_event(black_box(event));
                    }
                    gateway.clear();
                })
            },
        );
    }
    group.finish();
}

/// Benchmark: Single event RPC latency (1000x calls)
fn bench_rpc_latency_1000x(c: &mut Criterion) {
    let mut group = c.benchmark_group("rpc_latency_large");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(10));

    group.throughput(Throughput::Elements(1000));
    group.bench_function("1000_individual_rpcs", |b| {
        let gateway = mock_gateway::GatewayClient::new(50);

        b.iter(|| {
            for i in 0..1000 {
                let event = create_event(i);
                let _ = gateway.send_event(black_box(event));
            }
            gateway.clear();
        })
    });
    group.finish();
}

/// Benchmark: Batch operations (100x batch of 100 events)
fn bench_batch_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_operations");
    group.sample_size(50);

    for batch_size in [10, 100, 500, 1000].iter() {
        group.throughput(Throughput::Elements(*batch_size as u64 * 100));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("batch_{}", batch_size)),
            batch_size,
            |b, &size| {
                let gateway = mock_gateway::GatewayClient::new(50);

                b.iter(|| {
                    for batch_num in 0..100 {
                        let mut batch = Vec::with_capacity(size);
                        for i in 0..size {
                            let event_id = (batch_num * 1000 + i) as u32;
                            batch.push(create_event(event_id));
                        }
                        let _ = gateway.send_batch(black_box(batch));
                    }
                    gateway.clear();
                })
            },
        );
    }
    group.finish();
}

/// Benchmark: Serialization overhead (JSON encoding)
fn bench_serialization_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");
    group.sample_size(1000);

    for event_count in [1, 10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*event_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_events", event_count)),
            event_count,
            |b, &count| {
                let mut events = Vec::with_capacity(count);
                for i in 0..count {
                    events.push(create_event(i as u32));
                }

                b.iter(|| {
                    let _json = serde_json::to_string(&black_box(&events)).unwrap();
                })
            },
        );
    }
    group.finish();
}

/// Benchmark: Deserialization overhead (JSON decoding)
fn bench_deserialization_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("deserialization");
    group.sample_size(1000);

    for event_count in [1, 10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*event_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_events", event_count)),
            event_count,
            |b, &count| {
                let mut events = Vec::with_capacity(count);
                for i in 0..count {
                    events.push(create_event(i as u32));
                }
                let json = serde_json::to_string(&events).unwrap();

                b.iter(|| {
                    let _: Vec<mock_gateway::Event> =
                        serde_json::from_str(&black_box(&json)).unwrap();
                })
            },
        );
    }
    group.finish();
}

/// Benchmark: Batch vs individual (throughput comparison)
fn bench_batch_vs_individual(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_comparison");
    group.sample_size(50);
    group.measurement_time(Duration::from_secs(10));

    let total_events = 10000;

    group.throughput(Throughput::Elements(total_events as u64));
    group.bench_function("individual_events", |b| {
        let gateway = mock_gateway::GatewayClient::new(50);

        b.iter(|| {
            for i in 0..total_events {
                let event = create_event(i as u32);
                let _ = gateway.send_event(black_box(event));
            }
            gateway.clear();
        })
    });

    group.throughput(Throughput::Elements(total_events as u64));
    group.bench_function("batched_1000", |b| {
        let gateway = mock_gateway::GatewayClient::new(50);

        b.iter(|| {
            let mut batch = Vec::with_capacity(1000);
            for i in 0..total_events {
                batch.push(create_event(i as u32));
                if batch.len() == 1000 {
                    let _ = gateway.send_batch(black_box(batch.clone()));
                    batch.clear();
                }
            }
            if !batch.is_empty() {
                let _ = gateway.send_batch(black_box(batch));
            }
            gateway.clear();
        })
    });

    group.finish();
}

/// Benchmark: WebSocket streaming latency
fn bench_websocket_streaming(c: &mut Criterion) {
    let mut group = c.benchmark_group("websocket_streaming");
    group.sample_size(100);

    for event_count in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*event_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_events", event_count)),
            event_count,
            |b, &count| {
                let gateway = mock_gateway::GatewayClient::new(50);

                b.iter(|| {
                    for i in 0..count {
                        let event = create_event(i as u32);
                        let _ = gateway.stream_event(black_box(event));
                    }
                    gateway.clear();
                })
            },
        );
    }
    group.finish();
}

/// Benchmark: Memory usage for event buffering
fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    group.sample_size(10);

    for event_count in [1000, 10000, 100000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_events", event_count)),
            event_count,
            |b, &count| {
                b.iter_custom(|iters| {
                    let mut total_duration = Duration::ZERO;

                    for _ in 0..iters {
                        let start = Instant::now();

                        // Allocate and populate events
                        let mut events = Vec::with_capacity(count);
                        for i in 0..count {
                            events.push(create_event(i as u32));
                        }

                        // Serialize to JSON (in-memory buffer)
                        let _json = serde_json::to_string(&events).unwrap();

                        total_duration += start.elapsed();
                    }

                    total_duration
                })
            },
        );
    }
    group.finish();
}

/// Benchmark: Gateway latency breakdown
fn bench_latency_breakdown(c: &mut Criterion) {
    let mut group = c.benchmark_group("latency_breakdown");
    group.sample_size(100);

    // Single event: measure individual components
    group.bench_function("single_event_breakdown", |b| {
        b.iter(|| {
            let gateway = mock_gateway::GatewayClient::new(50);
            let event = black_box(create_event(1));
            let _ = gateway.send_event(event);
        })
    });

    // 100 events: measure serialization ratio
    group.bench_function("100_events_serialization", |b| {
        let mut events = Vec::with_capacity(100);
        for i in 0..100 {
            events.push(create_event(i as u32));
        }

        b.iter(|| {
            let _json = serde_json::to_string(&black_box(&events)).unwrap();
        })
    });

    // 1000 events: measure end-to-end
    group.bench_function("1000_events_end_to_end", |b| {
        let gateway = mock_gateway::GatewayClient::new(50);
        let mut events = Vec::with_capacity(1000);
        for i in 0..1000 {
            events.push(create_event(i as u32));
        }

        b.iter(|| {
            let _ = gateway.send_batch(black_box(events.clone()));
        })
    });

    group.finish();
}

/// Benchmark: Query performance (simulated database)
fn bench_query_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_performance");
    group.sample_size(50);

    for stored_events in [1000, 10000, 100000].iter() {
        group.throughput(Throughput::Elements(*stored_events as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_stored", stored_events)),
            stored_events,
            |b, &count| {
                let gateway = mock_gateway::GatewayClient::new(50);

                // Pre-populate with events
                let mut batch = Vec::with_capacity(1000);
                for i in 0..count {
                    batch.push(create_event(i as u32));
                    if batch.len() == 1000 {
                        let _ = gateway.send_batch(batch.clone());
                        batch.clear();
                    }
                }
                if !batch.is_empty() {
                    let _ = gateway.send_batch(batch);
                }

                b.iter(|| {
                    let _ = gateway.query_events(100);
                })
            },
        );
    }
    group.finish();
}

/// Benchmark: Concurrent request simulation
fn bench_concurrent_requests(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_requests");
    group.sample_size(20);
    group.measurement_time(Duration::from_secs(10));

    for concurrency in [1, 5, 10, 50].iter() {
        group.throughput(Throughput::Elements((*concurrency * 100) as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_concurrent", concurrency)),
            concurrency,
            |b, &concurrent| {
                use std::sync::Arc;
                use std::sync::Mutex;
                use std::thread;

                b.iter(|| {
                    let gateway = Arc::new(mock_gateway::GatewayClient::new(50));
                    let mut handles = vec![];

                    for t in 0..concurrent {
                        let g = Arc::clone(&gateway);
                        let handle = thread::spawn(move || {
                            for i in 0..100 {
                                let event_id = t * 100 + i;
                                let event = create_event(event_id as u32);
                                let _ = g.send_event(event);
                            }
                        });
                        handles.push(handle);
                    }

                    for handle in handles {
                        let _ = handle.join();
                    }

                    gateway.clear();
                })
            },
        );
    }
    group.finish();
}

// Register benchmark groups
criterion_group!(
    benches,
    bench_rpc_latency_100x,
    bench_rpc_latency_1000x,
    bench_batch_operations,
    bench_serialization_overhead,
    bench_deserialization_overhead,
    bench_batch_vs_individual,
    bench_websocket_streaming,
    bench_memory_usage,
    bench_latency_breakdown,
    bench_query_performance,
    bench_concurrent_requests,
);

criterion_main!(benches);
