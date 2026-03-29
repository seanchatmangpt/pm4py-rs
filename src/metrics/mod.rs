/// Metrics and monitoring module for pm4py-rust
///
/// Provides comprehensive Prometheus-compatible metrics for production deployments.
pub mod prometheus;

pub use prometheus::{MetricsCollector, RequestGuard};

// Global metrics instance
use std::sync::OnceLock;

static METRICS: OnceLock<MetricsCollector> = OnceLock::new();

/// Get or initialize global metrics collector
pub fn metrics() -> &'static MetricsCollector {
    METRICS.get_or_init(MetricsCollector::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_metrics_singleton() {
        let m1 = metrics();
        let m2 = metrics();
        // Both references should point to the same instance (same pointer value)
        assert_eq!(m1 as *const _ as usize, m2 as *const _ as usize);
    }
}
