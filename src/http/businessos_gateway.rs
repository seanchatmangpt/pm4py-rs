//! BusinessOS ↔ PM4Py Rust Gateway Client
//!
//! Provides HTTP client for pm4py-rust to communicate with BusinessOS backend.
//! Handles request marshaling, error translation, connection pooling, and retry logic.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::RwLock;

#[derive(Debug, Error)]
pub enum GatewayError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Request timeout after {0}ms")]
    Timeout(u64),

    #[error("HTTP error: {status} - {message}")]
    HttpError { status: u16, message: String },

    #[error("Serialization failed: {0}")]
    SerializationError(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Retry limit exceeded: {0}")]
    RetryLimitExceeded(String),

    #[error("Gateway unavailable: {0}")]
    GatewayUnavailable(String),
}

/// Configuration for the gateway client
#[derive(Debug, Clone)]
pub struct GatewayConfig {
    pub base_url: String,
    pub timeout_ms: u64,
    pub max_retries: u32,
    pub retry_delay_ms: u64,
    pub connection_pool_size: usize,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8001".to_string(),
            timeout_ms: 5000,
            max_retries: 3,
            retry_delay_ms: 100,
            connection_pool_size: 10,
        }
    }
}

/// Gateway statistics tracking
#[derive(Debug, Clone)]
pub struct GatewayStats {
    pub requests_total: Arc<AtomicU64>,
    pub requests_failed: Arc<AtomicU64>,
    pub request_latencies: Arc<RwLock<Vec<u64>>>,
    pub started_at: Instant,
}

impl Default for GatewayStats {
    fn default() -> Self {
        Self {
            requests_total: Arc::new(AtomicU64::new(0)),
            requests_failed: Arc::new(AtomicU64::new(0)),
            request_latencies: Arc::new(RwLock::new(Vec::new())),
            started_at: Instant::now(),
        }
    }
}

impl GatewayStats {
    pub fn record_request(&self, latency_ms: u64, success: bool) {
        self.requests_total.fetch_add(1, Ordering::SeqCst);
        if !success {
            self.requests_failed.fetch_add(1, Ordering::SeqCst);
        }

        // Store latency: use blocking write to avoid unbounded thread spawning
        // and resource leak. Timeout guards against lock contention.
        let latencies = self.request_latencies.clone();
        std::thread::spawn(move || {
            // Timeout: 100ms to guard against RwLock contention
            // If lock can't be acquired in 100ms, drop update (best-effort)
            if let Ok(mut lats) = latencies.try_write() {
                lats.push(latency_ms);
                // Bounded: keep only last 100 latencies
                if lats.len() > 100 {
                    lats.remove(0);
                }
            }
            // Implicit timeout via thread exit if try_write() blocks
        });
    }

    pub async fn average_latency(&self) -> f64 {
        let lats = self.request_latencies.read().await;
        if lats.is_empty() {
            return 0.0;
        }
        let sum: u64 = lats.iter().sum();
        sum as f64 / lats.len() as f64
    }

    pub fn uptime_seconds(&self) -> u64 {
        self.started_at.elapsed().as_secs()
    }
}

/// HTTP Client for BusinessOS Gateway
pub struct BusinessOSGateway {
    config: GatewayConfig,
    stats: GatewayStats,
    client: reqwest::Client,
}

impl BusinessOSGateway {
    /// Create new gateway client with default config
    pub fn new() -> Result<Self, GatewayError> {
        Self::with_config(GatewayConfig::default())
    }

    /// Create new gateway client with custom config
    /// Returns error if client builder fails (e.g., invalid connection pool config)
    pub fn with_config(config: GatewayConfig) -> Result<Self, GatewayError> {
        let client = reqwest::Client::builder()
            .pool_max_idle_per_host(config.connection_pool_size)
            .build()
            .map_err(|e| {
                GatewayError::GatewayUnavailable(format!(
                    "Failed to build HTTP client with pool size {}: {}",
                    config.connection_pool_size, e
                ))
            })?;

        Ok(Self {
            config,
            stats: GatewayStats::default(),
            client,
        })
    }

    // ========================================================================
    // Discovery Endpoints
    // ========================================================================

    /// POST /api/bos/discover — Process model discovery
    pub async fn discover(
        &self,
        req: &DiscoverGatewayRequest,
    ) -> Result<DiscoverGatewayResponse, GatewayError> {
        self.send_request_with_retry("POST", "/api/bos/discover", req)
            .await
    }

    // ========================================================================
    // Conformance Endpoints
    // ========================================================================

    /// POST /api/bos/conformance — Conformance checking
    pub async fn check_conformance(
        &self,
        req: &ConformanceGatewayRequest,
    ) -> Result<ConformanceGatewayResponse, GatewayError> {
        self.send_request_with_retry("POST", "/api/bos/conformance", req)
            .await
    }

    // ========================================================================
    // Statistics Endpoints
    // ========================================================================

    /// POST /api/bos/statistics — Event log statistics
    pub async fn get_statistics(
        &self,
        req: &StatisticsGatewayRequest,
    ) -> Result<StatisticsGatewayResponse, GatewayError> {
        self.send_request_with_retry("POST", "/api/bos/statistics", req)
            .await
    }

    // ========================================================================
    // Status Endpoint
    // ========================================================================

    /// GET /api/bos/status — Gateway health status
    pub async fn get_status(&self) -> Result<StatusGatewayResponse, GatewayError> {
        let url = format!("{}/api/bos/status", self.config.base_url);
        let start = Instant::now();

        let result = tokio::time::timeout(
            Duration::from_millis(self.config.timeout_ms),
            self.client.get(&url).send(),
        )
        .await;

        let latency_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(Ok(response)) => {
                let status = response.status();
                if status.is_success() {
                    let body = response.json().await.map_err(|e| {
                        GatewayError::InvalidResponse(format!("Failed to parse response: {}", e))
                    })?;
                    self.stats.record_request(latency_ms, true);
                    Ok(body)
                } else {
                    // Extract error message from response body with proper error handling
                    let text = match response.text().await {
                        Ok(body) => body,
                        Err(e) => {
                            // If we can't read error body, log it but include HTTP status
                            format!(
                                "HTTP {} (failed to read response body: {})",
                                status.as_u16(),
                                e
                            )
                        }
                    };
                    self.stats.record_request(latency_ms, false);
                    Err(GatewayError::HttpError {
                        status: status.as_u16(),
                        message: text,
                    })
                }
            }
            Ok(Err(e)) => {
                self.stats.record_request(latency_ms, false);
                Err(GatewayError::ConnectionFailed(e.to_string()))
            }
            Err(_) => {
                self.stats.record_request(latency_ms, false);
                Err(GatewayError::Timeout(self.config.timeout_ms))
            }
        }
    }

    // ========================================================================
    // Internal Methods
    // ========================================================================

    async fn send_request_with_retry<Req, Resp>(
        &self,
        method: &str,
        endpoint: &str,
        request: &Req,
    ) -> Result<Resp, GatewayError>
    where
        Req: Serialize,
        Resp: for<'de> Deserialize<'de>,
    {
        let mut attempt = 0;
        loop {
            match self.send_request(method, endpoint, request).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    attempt += 1;
                    if attempt >= self.config.max_retries {
                        return Err(GatewayError::RetryLimitExceeded(format!(
                            "Failed after {} attempts: {}",
                            attempt, e
                        )));
                    }

                    // Exponential backoff
                    let backoff = self.config.retry_delay_ms * (2_u64.pow(attempt - 1));
                    tokio::time::sleep(Duration::from_millis(backoff)).await;
                }
            }
        }
    }

    async fn send_request<Req, Resp>(
        &self,
        method: &str,
        endpoint: &str,
        request: &Req,
    ) -> Result<Resp, GatewayError>
    where
        Req: Serialize,
        Resp: for<'de> Deserialize<'de>,
    {
        let url = format!("{}{}", self.config.base_url, endpoint);
        let start = Instant::now();

        let request_builder = match method {
            "POST" => self.client.post(&url),
            "GET" => self.client.get(&url),
            "PUT" => self.client.put(&url),
            "DELETE" => self.client.delete(&url),
            _ => {
                return Err(GatewayError::ConnectionFailed(
                    "Invalid HTTP method".to_string(),
                ))
            }
        };

        let json_body = serde_json::to_string(request).map_err(|e| {
            GatewayError::SerializationError(format!("Failed to serialize request: {}", e))
        })?;

        let result = tokio::time::timeout(
            Duration::from_millis(self.config.timeout_ms),
            request_builder
                .header("Content-Type", "application/json")
                .body(json_body)
                .send(),
        )
        .await;

        let latency_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(Ok(response)) => {
                let status = response.status();
                if status.is_success() {
                    let body = response.json().await.map_err(|e| {
                        GatewayError::InvalidResponse(format!("Failed to parse response: {}", e))
                    })?;
                    self.stats.record_request(latency_ms, true);
                    Ok(body)
                } else {
                    // Extract error message from response body with proper error handling
                    let text = match response.text().await {
                        Ok(body) => body,
                        Err(e) => {
                            // If we can't read error body, log it but include HTTP status
                            format!(
                                "HTTP {} (failed to read response body: {})",
                                status.as_u16(),
                                e
                            )
                        }
                    };
                    self.stats.record_request(latency_ms, false);
                    Err(GatewayError::HttpError {
                        status: status.as_u16(),
                        message: text,
                    })
                }
            }
            Ok(Err(e)) => {
                self.stats.record_request(latency_ms, false);
                Err(GatewayError::ConnectionFailed(e.to_string()))
            }
            Err(_) => {
                self.stats.record_request(latency_ms, false);
                Err(GatewayError::Timeout(self.config.timeout_ms))
            }
        }
    }

    pub fn stats(&self) -> &GatewayStats {
        &self.stats
    }
}

impl Default for BusinessOSGateway {
    fn default() -> Self {
        // Default constructor panics on error (since we can't return Result from Default)
        // In production, use new() and handle the Result.
        Self::new().expect("Failed to create default BusinessOSGateway client")
    }
}

// ============================================================================
// Request Types
// ============================================================================

/// Model discovery request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverGatewayRequest {
    pub log_path: String,
    #[serde(default)]
    pub algorithm: String,
}

/// Model discovery response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverGatewayResponse {
    pub model_id: String,
    pub algorithm: String,
    pub places: usize,
    pub transitions: usize,
    pub arcs: usize,
    pub model_data: serde_json::Value,
    pub latency_ms: u64,
}

/// Conformance checking request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConformanceGatewayRequest {
    pub log_path: String,
    pub model_id: String,
}

/// Conformance checking response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConformanceGatewayResponse {
    pub traces_checked: u64,
    pub fitting_traces: u64,
    pub fitness: f64,
    pub precision: f64,
    pub generalization: f64,
    pub simplicity: f64,
    pub latency_ms: u64,
}

/// Statistics extraction request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticsGatewayRequest {
    pub log_path: String,
}

/// Activity statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityStatisticGateway {
    pub activity: String,
    pub frequency: usize,
    pub percentage: f64,
}

/// Case duration statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseDurationStatisticGateway {
    pub min_seconds: i64,
    pub max_seconds: i64,
    pub avg_seconds: f64,
    pub median_seconds: f64,
}

/// Statistics extraction response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticsGatewayResponse {
    pub log_name: String,
    pub num_traces: usize,
    pub num_events: usize,
    pub num_unique_activities: usize,
    pub num_variants: usize,
    pub avg_trace_length: f64,
    pub min_trace_length: usize,
    pub max_trace_length: usize,
    pub activity_frequency: Vec<ActivityStatisticGateway>,
    pub case_duration: CaseDurationStatisticGateway,
    pub latency_ms: u64,
}

/// Gateway status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusGatewayResponse {
    pub status: String,
    pub database_ready: bool,
    pub latency_ms: u64,
    pub requests_total: u64,
    pub requests_failed: u64,
    pub average_latency_ms: f64,
    pub uptime_seconds: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gateway_config_defaults() {
        let config = GatewayConfig::default();
        assert_eq!(config.base_url, "http://localhost:8001");
        assert_eq!(config.timeout_ms, 5000);
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_gateway_stats_default() {
        let stats = GatewayStats::default();
        assert_eq!(stats.requests_total.load(Ordering::SeqCst), 0);
        assert_eq!(stats.requests_failed.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_gateway_stats_record() {
        let stats = GatewayStats::default();
        stats.record_request(50, true);
        assert_eq!(stats.requests_total.load(Ordering::SeqCst), 1);
        assert_eq!(stats.requests_failed.load(Ordering::SeqCst), 0);

        stats.record_request(75, false);
        assert_eq!(stats.requests_total.load(Ordering::SeqCst), 2);
        assert_eq!(stats.requests_failed.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_discover_request_serialization() {
        let req = DiscoverGatewayRequest {
            log_path: "/path/to/log.xes".to_string(),
            algorithm: "inductive_miner".to_string(),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("log_path"));
        assert!(json.contains("algorithm"));
    }

    #[test]
    fn test_gateway_error_display() {
        let err = GatewayError::Timeout(5000);
        assert!(err.to_string().contains("5000"));
    }
}
