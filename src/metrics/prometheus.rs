use std::collections::HashMap;
/// Prometheus metrics collection and export for pm4py-rust
///
/// This module provides comprehensive monitoring for production deployments:
/// - HTTP endpoint: GET /metrics (Prometheus text format)
/// - Histograms: algorithm duration tracking
/// - Gauges: real-time resource usage
/// - Counters: error and request tracking
/// - Memory tracking: process and event log memory
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Main metrics collector - thread-safe singleton
pub struct MetricsCollector {
    // Histograms (duration tracking)
    discovery_durations: Arc<Mutex<Vec<Duration>>>,
    conformance_durations: Arc<Mutex<Vec<Duration>>>,
    statistics_durations: Arc<Mutex<Vec<Duration>>>,
    gc_durations: Arc<Mutex<Vec<Duration>>>,

    // Gauges (point-in-time values)
    active_requests: Arc<AtomicU64>,
    event_log_size_bytes: Arc<AtomicU64>,
    memory_usage_bytes: Arc<AtomicU64>,

    // Counters (cumulative)
    total_requests: Arc<AtomicU64>,
    request_errors_total: Arc<Mutex<HashMap<String, u64>>>,
    discovery_calls_total: Arc<AtomicU64>,
    conformance_calls_total: Arc<AtomicU64>,
    statistics_calls_total: Arc<AtomicU64>,

    // SLA tracking
    uptime_start: Instant,
    request_start_times: Arc<Mutex<HashMap<u64, Instant>>>,
    next_request_id: Arc<AtomicU64>,

    // Histogram bucket configuration
    histogram_buckets: Vec<f64>,
}

impl MetricsCollector {
    /// Create new metrics collector with standard buckets
    pub fn new() -> Self {
        // Standard Prometheus buckets: 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1, 2.5, 5, 10
        let histogram_buckets = vec![
            0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 60.0,
        ];

        Self {
            discovery_durations: Arc::new(Mutex::new(Vec::new())),
            conformance_durations: Arc::new(Mutex::new(Vec::new())),
            statistics_durations: Arc::new(Mutex::new(Vec::new())),
            gc_durations: Arc::new(Mutex::new(Vec::new())),
            active_requests: Arc::new(AtomicU64::new(0)),
            event_log_size_bytes: Arc::new(AtomicU64::new(0)),
            memory_usage_bytes: Arc::new(AtomicU64::new(0)),
            total_requests: Arc::new(AtomicU64::new(0)),
            request_errors_total: Arc::new(Mutex::new(HashMap::new())),
            discovery_calls_total: Arc::new(AtomicU64::new(0)),
            conformance_calls_total: Arc::new(AtomicU64::new(0)),
            statistics_calls_total: Arc::new(AtomicU64::new(0)),
            uptime_start: Instant::now(),
            request_start_times: Arc::new(Mutex::new(HashMap::new())),
            next_request_id: Arc::new(AtomicU64::new(1)),
            histogram_buckets,
        }
    }

    /// Start tracking a request
    pub fn start_request(&self) -> RequestGuard {
        let request_id = self.next_request_id.fetch_add(1, Ordering::SeqCst);
        self.active_requests.fetch_add(1, Ordering::SeqCst);
        self.total_requests.fetch_add(1, Ordering::SeqCst);

        let mut start_times = self
            .request_start_times
            .lock()
            .expect("request_start_times mutex not poisoned");
        start_times.insert(request_id, Instant::now());

        RequestGuard {
            request_id,
            metrics: self.clone(),
            error_type: None,
        }
    }

    /// Record discovery algorithm duration
    pub fn record_discovery_duration(&self, duration: Duration) {
        self.discovery_calls_total.fetch_add(1, Ordering::SeqCst);
        let mut durations = self
            .discovery_durations
            .lock()
            .expect("metrics mutex not poisoned");
        durations.push(duration);

        // Trim old data if exceeds 10k samples
        if durations.len() > 10000 {
            durations.drain(0..5000);
        }
    }

    /// Record conformance check duration
    pub fn record_conformance_duration(&self, duration: Duration) {
        self.conformance_calls_total.fetch_add(1, Ordering::SeqCst);
        let mut durations = self
            .conformance_durations
            .lock()
            .expect("metrics mutex not poisoned");
        durations.push(duration);

        if durations.len() > 10000 {
            durations.drain(0..5000);
        }
    }

    /// Record statistics computation duration
    pub fn record_statistics_duration(&self, duration: Duration) {
        self.statistics_calls_total.fetch_add(1, Ordering::SeqCst);
        let mut durations = self
            .statistics_durations
            .lock()
            .expect("metrics mutex not poisoned");
        durations.push(duration);

        if durations.len() > 10000 {
            durations.drain(0..5000);
        }
    }

    /// Record garbage collection pause
    pub fn record_gc_duration(&self, duration: Duration) {
        let mut durations = self
            .gc_durations
            .lock()
            .expect("metrics mutex not poisoned");
        durations.push(duration);

        if durations.len() > 10000 {
            durations.drain(0..5000);
        }
    }

    /// Update event log size
    pub fn set_event_log_size(&self, bytes: u64) {
        self.event_log_size_bytes.store(bytes, Ordering::SeqCst);
    }

    /// Update process memory usage
    pub fn set_memory_usage(&self, bytes: u64) {
        self.memory_usage_bytes.store(bytes, Ordering::SeqCst);
    }

    /// Record request error
    pub fn record_error(&self, error_type: &str) {
        let mut errors = self
            .request_errors_total
            .lock()
            .expect("metrics mutex not poisoned");
        *errors.entry(error_type.to_string()).or_insert(0) += 1;
    }

    /// End request tracking
    pub fn end_request(&self, request_id: u64) {
        self.active_requests.fetch_sub(1, Ordering::SeqCst);
        let mut start_times = self
            .request_start_times
            .lock()
            .expect("request_start_times mutex not poisoned");
        start_times.remove(&request_id);
    }

    /// Get uptime in seconds
    pub fn uptime_seconds(&self) -> u64 {
        self.uptime_start.elapsed().as_secs()
    }

    /// Calculate histogram percentiles
    #[allow(dead_code)]
    fn calculate_percentiles(durations: &[Duration]) -> (f64, f64, f64) {
        if durations.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let mut sorted: Vec<f64> = durations.iter().map(|d| d.as_secs_f64()).collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).expect("duration values must not be NaN"));

        let p50_idx = (sorted.len() / 2).max(1) - 1;
        let p95_idx = ((sorted.len() * 95) / 100).max(1) - 1;
        let p99_idx = ((sorted.len() * 99) / 100).max(1) - 1;

        (sorted[p50_idx], sorted[p95_idx], sorted[p99_idx])
    }

    /// Export metrics in Prometheus text format
    pub fn export_prometheus(&self) -> String {
        let mut output = String::new();

        // HELP and TYPE declarations
        output.push_str("# HELP discovery_duration_seconds Time spent in discovery algorithm\n");
        output.push_str("# TYPE discovery_duration_seconds histogram\n");

        output.push_str("# HELP conformance_duration_seconds Time spent in conformance check\n");
        output.push_str("# TYPE conformance_duration_seconds histogram\n");

        output.push_str("# HELP statistics_duration_seconds Time spent computing statistics\n");
        output.push_str("# TYPE statistics_duration_seconds histogram\n");

        output.push_str("# HELP active_requests Current number of active requests\n");
        output.push_str("# TYPE active_requests gauge\n");

        output.push_str("# HELP event_log_size_bytes Size of event log in memory\n");
        output.push_str("# TYPE event_log_size_bytes gauge\n");

        output.push_str("# HELP request_errors_total Total number of request errors\n");
        output.push_str("# TYPE request_errors_total counter\n");

        output.push_str("# HELP memory_usage_bytes Current process memory usage\n");
        output.push_str("# TYPE memory_usage_bytes gauge\n");

        output.push_str("# HELP gc_duration_seconds Garbage collection pause time\n");
        output.push_str("# TYPE gc_duration_seconds histogram\n");

        output.push_str("# HELP uptime_seconds Total uptime in seconds\n");
        output.push_str("# TYPE uptime_seconds counter\n");

        output.push_str("# HELP total_requests Total number of requests processed\n");
        output.push_str("# TYPE total_requests counter\n");

        output.push_str("# HELP discovery_calls_total Total discovery algorithm calls\n");
        output.push_str("# TYPE discovery_calls_total counter\n");

        output.push_str("# HELP conformance_calls_total Total conformance checks\n");
        output.push_str("# TYPE conformance_calls_total counter\n");

        output.push_str("# HELP statistics_calls_total Total statistics computations\n");
        output.push_str("# TYPE statistics_calls_total counter\n");

        // Gauges
        output.push_str(&format!(
            "active_requests {}\n",
            self.active_requests.load(Ordering::SeqCst)
        ));

        output.push_str(&format!(
            "event_log_size_bytes {}\n",
            self.event_log_size_bytes.load(Ordering::SeqCst)
        ));

        output.push_str(&format!(
            "memory_usage_bytes {}\n",
            self.memory_usage_bytes.load(Ordering::SeqCst)
        ));

        // Counters
        output.push_str(&format!("uptime_seconds {}\n", self.uptime_seconds()));

        output.push_str(&format!(
            "total_requests {}\n",
            self.total_requests.load(Ordering::SeqCst)
        ));

        output.push_str(&format!(
            "discovery_calls_total {}\n",
            self.discovery_calls_total.load(Ordering::SeqCst)
        ));

        output.push_str(&format!(
            "conformance_calls_total {}\n",
            self.conformance_calls_total.load(Ordering::SeqCst)
        ));

        output.push_str(&format!(
            "statistics_calls_total {}\n",
            self.statistics_calls_total.load(Ordering::SeqCst)
        ));

        // Error counters
        let errors = self
            .request_errors_total
            .lock()
            .expect("metrics mutex not poisoned");
        for (error_type, count) in errors.iter() {
            output.push_str(&format!(
                "request_errors_total{{type=\"{}\"}} {}\n",
                error_type, count
            ));
        }

        // Histograms with percentiles
        output.push_str(
            &self.export_histogram_metrics(
                "discovery_duration_seconds",
                &self
                    .discovery_durations
                    .lock()
                    .expect("metrics mutex not poisoned"),
            ),
        );

        output.push_str(
            &self.export_histogram_metrics(
                "conformance_duration_seconds",
                &self
                    .conformance_durations
                    .lock()
                    .expect("metrics mutex not poisoned"),
            ),
        );

        output.push_str(
            &self.export_histogram_metrics(
                "statistics_duration_seconds",
                &self
                    .statistics_durations
                    .lock()
                    .expect("metrics mutex not poisoned"),
            ),
        );

        output.push_str(
            &self.export_histogram_metrics(
                "gc_duration_seconds",
                &self
                    .gc_durations
                    .lock()
                    .expect("metrics mutex not poisoned"),
            ),
        );

        output
    }

    /// Export histogram in Prometheus format
    fn export_histogram_metrics(&self, name: &str, durations: &[Duration]) -> String {
        let mut output = String::new();

        let total_count = durations.len();
        let sum: f64 = durations.iter().map(|d| d.as_secs_f64()).sum();

        // Bucket distribution
        for &bucket in &self.histogram_buckets {
            let count = durations
                .iter()
                .filter(|d| d.as_secs_f64() <= bucket)
                .count();
            output.push_str(&format!("{}{{le=\"{}\"}} {}\n", name, bucket, count));
        }

        output.push_str(&format!("{}{{le=\"+Inf\"}} {}\n", name, total_count));

        output.push_str(&format!("{}_sum {}\n", name, sum));
        output.push_str(&format!("{}_count {}\n", name, total_count));

        output
    }
}

impl Clone for MetricsCollector {
    fn clone(&self) -> Self {
        Self {
            discovery_durations: Arc::clone(&self.discovery_durations),
            conformance_durations: Arc::clone(&self.conformance_durations),
            statistics_durations: Arc::clone(&self.statistics_durations),
            gc_durations: Arc::clone(&self.gc_durations),
            active_requests: Arc::clone(&self.active_requests),
            event_log_size_bytes: Arc::clone(&self.event_log_size_bytes),
            memory_usage_bytes: Arc::clone(&self.memory_usage_bytes),
            total_requests: Arc::clone(&self.total_requests),
            request_errors_total: Arc::clone(&self.request_errors_total),
            discovery_calls_total: Arc::clone(&self.discovery_calls_total),
            conformance_calls_total: Arc::clone(&self.conformance_calls_total),
            statistics_calls_total: Arc::clone(&self.statistics_calls_total),
            uptime_start: self.uptime_start,
            request_start_times: Arc::clone(&self.request_start_times),
            next_request_id: Arc::clone(&self.next_request_id),
            histogram_buckets: self.histogram_buckets.clone(),
        }
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// RAII guard for request tracking
pub struct RequestGuard {
    request_id: u64,
    metrics: MetricsCollector,
    error_type: Option<String>,
}

impl RequestGuard {
    /// Mark request as having specific error type
    pub fn set_error(&mut self, error_type: &str) {
        self.error_type = Some(error_type.to_string());
    }
}

impl Drop for RequestGuard {
    fn drop(&mut self) {
        if let Some(ref error_type) = self.error_type {
            self.metrics.record_error(error_type);
        }
        self.metrics.end_request(self.request_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collector_creation() {
        let metrics = MetricsCollector::new();
        assert_eq!(metrics.uptime_seconds(), 0);
        assert_eq!(metrics.active_requests.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_request_tracking() {
        let metrics = MetricsCollector::new();

        {
            let _guard = metrics.start_request();
            assert_eq!(metrics.active_requests.load(Ordering::SeqCst), 1);
        }

        assert_eq!(metrics.active_requests.load(Ordering::SeqCst), 0);
        assert_eq!(metrics.total_requests.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_discovery_duration_tracking() {
        let metrics = MetricsCollector::new();
        metrics.record_discovery_duration(Duration::from_millis(50));
        metrics.record_discovery_duration(Duration::from_millis(100));

        assert_eq!(metrics.discovery_calls_total.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_error_tracking() {
        let metrics = MetricsCollector::new();
        metrics.record_error("validation_error");
        metrics.record_error("validation_error");
        metrics.record_error("timeout");

        let errors = metrics
            .request_errors_total
            .lock()
            .expect("metrics mutex not poisoned");
        assert_eq!(errors.get("validation_error"), Some(&2));
        assert_eq!(errors.get("timeout"), Some(&1));
    }

    #[test]
    fn test_prometheus_export_format() {
        let metrics = MetricsCollector::new();
        metrics.set_event_log_size(1024000);
        metrics.set_memory_usage(536870912);

        let output = metrics.export_prometheus();

        assert!(output.contains("event_log_size_bytes 1024000"));
        assert!(output.contains("memory_usage_bytes 536870912"));
        assert!(output.contains("# TYPE discovery_duration_seconds histogram"));
        assert!(output.contains("active_requests 0"));
    }

    #[test]
    fn test_histogram_buckets() {
        let metrics = MetricsCollector::new();

        // Record durations across different magnitudes
        metrics.record_discovery_duration(Duration::from_millis(5));
        metrics.record_discovery_duration(Duration::from_millis(50));
        metrics.record_discovery_duration(Duration::from_secs(1));
        metrics.record_discovery_duration(Duration::from_secs(5));

        let output = metrics.export_prometheus();

        // Verify histogram output structure
        assert!(output.contains("discovery_duration_seconds_sum"));
        assert!(output.contains("discovery_duration_seconds_count 4"));
        assert!(output.contains("discovery_duration_seconds{le=\""));
    }

    #[test]
    fn test_gauge_updates() {
        let metrics = MetricsCollector::new();

        metrics.set_event_log_size(1000);
        assert_eq!(metrics.event_log_size_bytes.load(Ordering::SeqCst), 1000);

        metrics.set_memory_usage(2000);
        assert_eq!(metrics.memory_usage_bytes.load(Ordering::SeqCst), 2000);
    }
}
