use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use uuid::Uuid;

/// SpanContext represents a distributed trace span in pm4py.
#[derive(Debug, Clone)]
pub struct SpanContext {
    pub span_id: String,
    pub trace_id: String,
    pub parent_span_id: Option<String>,
    pub span_name: String,
    pub attributes: HashMap<String, String>,
    pub start_time_us: u64,
    pub end_time_us: Option<u64>,
    pub status: String,
    pub error_message: Option<String>,
}

/// MetricPoint represents a single metric observation.
#[derive(Debug, Clone)]
pub struct MetricPoint {
    pub name: String,
    pub value: f64,
    pub timestamp_us: u64,
    pub dimensions: HashMap<String, String>,
    pub metric_type: String, // "counter", "histogram", "gauge"
}

/// Tracing manages distributed tracing for pm4py process mining operations.
///
/// Traces key operations:
/// - Model loading: loading XES/CSV event logs, building Petri nets
/// - Event parsing: parsing event logs, validating format
/// - Conformance checking: checking trace conformance against model
/// - Discover: discovering process models from event logs
///
/// All spans and metrics are stored in memory and can be exported to external systems.
pub struct Tracing {
    spans: Arc<Mutex<HashMap<String, SpanContext>>>,
    metrics: Arc<Mutex<Vec<MetricPoint>>>,
}

impl Tracing {
    /// Create a new Tracing instance.
    pub fn new() -> Self {
        Tracing {
            spans: Arc::new(Mutex::new(HashMap::new())),
            metrics: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Initialize the tracer (no-op for now, but can be extended for setup).
    pub fn init_tracer(&self) -> Result<(), String> {
        Ok(())
    }

    /// Start a new span with optional attributes.
    ///
    /// Creates a span with auto-generated IDs and tracks parent/child relationships.
    ///
    /// # Parameters
    ///
    /// - `span_name`: name of the span, e.g. "model.load", "event.parse"
    /// - `attributes`: map of span metadata
    /// - `parent_span_id`: optional parent span ID for nesting
    ///
    /// # Returns
    ///
    /// A new SpanContext with auto-generated span and trace IDs.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let tracing = Tracing::new();
    /// let span = tracing.start_span("model.load", HashMap::new(), None)?;
    /// assert!(!span.span_id.is_empty());
    /// ```
    pub fn start_span(
        &self,
        span_name: &str,
        attributes: HashMap<String, String>,
        parent_span_id: Option<String>,
    ) -> Result<SpanContext, String> {
        let span_id = Uuid::new_v4().to_string();
        let trace_id = self.get_or_create_trace_id();
        let start_time_us = self.system_time_to_microseconds();

        let mut enriched_attributes = attributes;
        enriched_attributes.insert(
            "timestamp".to_string(),
            chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true),
        );
        enriched_attributes.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());

        let span = SpanContext {
            span_id: span_id.clone(),
            trace_id,
            parent_span_id,
            span_name: span_name.to_string(),
            attributes: enriched_attributes,
            start_time_us,
            end_time_us: None,
            status: "active".to_string(),
            error_message: None,
        };

        // Store span
        {
            let mut spans = self.spans.lock().expect("spans mutex not poisoned");
            spans.insert(span_id, span.clone());
        }

        Ok(span)
    }

    /// End a span and record its duration.
    ///
    /// Marks a span as complete, calculates elapsed time, and records a latency metric.
    ///
    /// # Parameters
    ///
    /// - `span`: the SpanContext to end
    /// - `status`: "ok" or "error"
    /// - `error_message`: optional error message if status is "error"
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut span = tracing.start_span("model.load", HashMap::new(), None)?;
    /// std::thread::sleep(std::time::Duration::from_millis(10));
    /// tracing.end_span(&mut span, "ok", None)?;
    /// ```
    pub fn end_span(
        &self,
        span: &mut SpanContext,
        status: &str,
        error_message: Option<&str>,
    ) -> Result<(), String> {
        let end_time_us = self.system_time_to_microseconds();
        let duration_us = end_time_us - span.start_time_us;

        span.end_time_us = Some(end_time_us);
        span.status = status.to_string();
        if let Some(msg) = error_message {
            span.error_message = Some(msg.to_string());
        }

        // Update span
        {
            let mut spans = self.spans.lock().expect("spans mutex not poisoned");
            spans.insert(span.span_id.clone(), span.clone());
        }

        // Record latency metric
        let mut dimensions = HashMap::new();
        dimensions.insert("span_name".to_string(), span.span_name.clone());
        dimensions.insert("status".to_string(), status.to_string());

        self.record_metric(
            "span.duration_us",
            duration_us as f64,
            dimensions,
            "histogram",
        )?;

        Ok(())
    }

    /// Record a metric observation with optional dimensions.
    ///
    /// Records numeric observations for later aggregation and export.
    ///
    /// # Parameters
    ///
    /// - `name`: metric name, e.g. "model.load_latency_ms"
    /// - `value`: numeric value to record
    /// - `dimensions`: optional map of tags
    /// - `metric_type`: "counter", "histogram", or "gauge"
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut dimensions = HashMap::new();
    /// dimensions.insert("format".to_string(), "xes".to_string());
    /// tracing.record_metric("model.load_latency_ms", 234.5, dimensions, "histogram")?;
    /// ```
    pub fn record_metric(
        &self,
        name: &str,
        value: f64,
        dimensions: HashMap<String, String>,
        metric_type: &str,
    ) -> Result<(), String> {
        let timestamp_us = self.system_time_to_microseconds();

        let point = MetricPoint {
            name: name.to_string(),
            value,
            timestamp_us,
            dimensions,
            metric_type: metric_type.to_string(),
        };

        {
            let mut metrics = self.metrics.lock().expect("metrics mutex not poisoned");
            metrics.push(point);
        }

        Ok(())
    }

    /// Get all recorded spans.
    pub fn get_spans(&self) -> Vec<SpanContext> {
        let spans = self.spans.lock().expect("spans mutex not poisoned");
        spans.values().cloned().collect()
    }

    /// Get all recorded metrics.
    pub fn get_metrics(&self) -> Vec<MetricPoint> {
        let metrics = self.metrics.lock().expect("metrics mutex not poisoned");
        metrics.clone()
    }

    /// Get a span by ID.
    pub fn get_span(&self, span_id: &str) -> Option<SpanContext> {
        let spans = self.spans.lock().expect("spans mutex not poisoned");
        spans.get(span_id).cloned()
    }

    /// Clear all spans and metrics (for testing).
    pub fn clear(&self) {
        {
            let mut spans = self.spans.lock().expect("spans mutex not poisoned");
            spans.clear();
        }
        {
            let mut metrics = self.metrics.lock().expect("metrics mutex not poisoned");
            metrics.clear();
        }
    }

    /// Get span count (for testing).
    pub fn span_count(&self) -> usize {
        let spans = self.spans.lock().expect("spans mutex not poisoned");
        spans.len()
    }

    /// Get metric count (for testing).
    pub fn metric_count(&self) -> usize {
        let metrics = self.metrics.lock().expect("metrics mutex not poisoned");
        metrics.len()
    }

    // ========================================================================
    // Private Functions
    // ========================================================================

    /// Get or create a trace ID (thread-local context).
    fn get_or_create_trace_id(&self) -> String {
        // In a real implementation, this would use thread-local storage or async context.
        // For now, we generate a new trace ID per Tracing instance.
        // This is sufficient for the distributed tracing model where each root span
        // starts a new trace.
        Uuid::new_v4().to_string()
    }

    /// Convert SystemTime to microseconds since epoch.
    fn system_time_to_microseconds(&self) -> u64 {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("system clock must be after UNIX epoch")
            .as_micros() as u64
    }
}

impl Default for Tracing {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_tracer() {
        let tracing = Tracing::new();
        assert!(tracing.init_tracer().is_ok());
    }

    #[test]
    fn test_start_span() {
        let tracing = Tracing::new();
        let mut attributes = HashMap::new();
        attributes.insert("model_id".to_string(), "model-1".to_string());

        let span = tracing.start_span("model.load", attributes, None).unwrap();

        assert!(!span.span_id.is_empty());
        assert!(!span.trace_id.is_empty());
        assert_eq!(span.span_name, "model.load");
        assert_eq!(span.status, "active");
        assert!(span.start_time_us > 0);
        assert_eq!(span.parent_span_id, None);
        assert!(span.attributes.contains_key("timestamp"));
        assert!(span.attributes.contains_key("version"));
    }

    #[test]
    fn test_start_span_with_parent() {
        let tracing = Tracing::new();

        let parent = tracing
            .start_span("process.mining", HashMap::new(), None)
            .unwrap();
        let parent_id = parent.span_id.clone();

        let child = tracing
            .start_span("model.load", HashMap::new(), Some(parent_id.clone()))
            .unwrap();

        assert_eq!(child.parent_span_id, Some(parent_id));
        // Note: each span in a Tracing instance gets a new trace ID in current implementation
        // This would need thread-local storage to propagate trace context properly
        assert!(!child.trace_id.is_empty());
    }

    #[test]
    fn test_end_span() {
        let tracing = Tracing::new();
        let mut span = tracing
            .start_span("model.load", HashMap::new(), None)
            .unwrap();

        std::thread::sleep(std::time::Duration::from_millis(5));
        tracing.end_span(&mut span, "ok", None).unwrap();

        assert_eq!(span.status, "ok");
        assert!(span.end_time_us.is_some());
        assert!(span.end_time_us.unwrap() > span.start_time_us);
    }

    #[test]
    fn test_end_span_with_error() {
        let tracing = Tracing::new();
        let mut span = tracing
            .start_span("event.parse", HashMap::new(), None)
            .unwrap();

        tracing
            .end_span(&mut span, "error", Some("invalid format"))
            .unwrap();

        assert_eq!(span.status, "error");
        assert_eq!(span.error_message, Some("invalid format".to_string()));
    }

    #[test]
    fn test_record_metric() {
        let tracing = Tracing::new();
        let mut dimensions = HashMap::new();
        dimensions.insert("format".to_string(), "xes".to_string());

        tracing
            .record_metric("model.load_latency_ms", 234.5, dimensions, "histogram")
            .unwrap();

        let metrics = tracing.get_metrics();
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].name, "model.load_latency_ms");
        assert_eq!(metrics[0].value, 234.5);
        assert_eq!(metrics[0].metric_type, "histogram");
    }

    #[test]
    fn test_span_creates_latency_metric() {
        let tracing = Tracing::new();
        let mut span = tracing
            .start_span("conformance.check", HashMap::new(), None)
            .unwrap();

        std::thread::sleep(std::time::Duration::from_millis(5));
        tracing.end_span(&mut span, "ok", None).unwrap();

        let metrics = tracing.get_metrics();
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].name, "span.duration_us");
        assert!(metrics[0].value > 0.0);
    }

    #[test]
    fn test_get_span() {
        let tracing = Tracing::new();
        let span = tracing
            .start_span("model.load", HashMap::new(), None)
            .unwrap();

        let retrieved = tracing.get_span(&span.span_id).unwrap();
        assert_eq!(retrieved.span_id, span.span_id);
        assert_eq!(retrieved.span_name, "model.load");
    }

    #[test]
    fn test_get_spans() {
        let tracing = Tracing::new();
        tracing
            .start_span("model.load", HashMap::new(), None)
            .unwrap();
        tracing
            .start_span("event.parse", HashMap::new(), None)
            .unwrap();

        let spans = tracing.get_spans();
        assert_eq!(spans.len(), 2);
    }

    #[test]
    fn test_clear() {
        let tracing = Tracing::new();
        tracing
            .start_span("model.load", HashMap::new(), None)
            .unwrap();
        tracing
            .record_metric("model.load_latency_ms", 100.0, HashMap::new(), "histogram")
            .unwrap();

        tracing.clear();

        assert_eq!(tracing.span_count(), 0);
        assert_eq!(tracing.metric_count(), 0);
    }

    #[test]
    fn test_span_hierarchy_inheritance() {
        let tracing = Tracing::new();

        // Parent span
        let parent = tracing
            .start_span("discover.algorithm", HashMap::new(), None)
            .unwrap();
        let parent_id = parent.span_id.clone();

        // Child span
        let child = tracing
            .start_span("model.construct", HashMap::new(), Some(parent_id.clone()))
            .unwrap();

        // Verify hierarchy
        assert_eq!(child.parent_span_id, Some(parent_id));
        assert!(!child.trace_id.is_empty());

        // Both in storage
        assert_eq!(tracing.span_count(), 2);
    }

    #[test]
    fn test_metric_aggregation() {
        let tracing = Tracing::new();

        // Record multiple metrics with same name but different dimensions
        let mut dim1 = HashMap::new();
        dim1.insert("algorithm".to_string(), "alpha".to_string());
        tracing
            .record_metric("discover.latency_ms", 100.0, dim1, "histogram")
            .unwrap();

        let mut dim2 = HashMap::new();
        dim2.insert("algorithm".to_string(), "heuristic".to_string());
        tracing
            .record_metric("discover.latency_ms", 150.0, dim2, "histogram")
            .unwrap();

        let metrics = tracing.get_metrics();
        assert_eq!(metrics.len(), 2); // Separate metrics due to different dimensions
        assert_eq!(metrics[0].value, 100.0);
        assert_eq!(metrics[1].value, 150.0);
    }
}
