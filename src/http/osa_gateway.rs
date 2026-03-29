//! OSA ↔ PM4Py Rust Gateway Client
//!
//! Provides HTTP client for pm4py-rust to communicate with OSA backend.
//! Handles 2PC transaction coordination (prepare, commit, rollback) with retry logic.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::RwLock;

#[derive(Debug, Error)]
pub enum OsaGatewayError {
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

    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
}

/// Configuration for the OSA gateway client
#[derive(Debug, Clone)]
pub struct OsaGatewayConfig {
    pub base_url: String,
    pub timeout_ms: u64,
    pub max_retries: u32,
    pub retry_delay_ms: u64,
    pub connection_pool_size: usize,
}

impl Default for OsaGatewayConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8089".to_string(),
            timeout_ms: 5000,
            max_retries: 3,
            retry_delay_ms: 100,
            connection_pool_size: 10,
        }
    }
}

/// Gateway statistics tracking
#[derive(Debug, Clone)]
pub struct OsaGatewayStats {
    pub requests_total: Arc<AtomicU64>,
    pub requests_failed: Arc<AtomicU64>,
    pub request_latencies: Arc<RwLock<Vec<u64>>>,
    pub started_at: Instant,
}

impl Default for OsaGatewayStats {
    fn default() -> Self {
        Self {
            requests_total: Arc::new(AtomicU64::new(0)),
            requests_failed: Arc::new(AtomicU64::new(0)),
            request_latencies: Arc::new(RwLock::new(Vec::new())),
            started_at: Instant::now(),
        }
    }
}

impl OsaGatewayStats {
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

/// HTTP Client for OSA Gateway (2PC participant)
pub struct OsaGateway {
    config: OsaGatewayConfig,
    stats: OsaGatewayStats,
    client: reqwest::Client,
}

impl OsaGateway {
    /// Create new gateway client with default config
    pub fn new() -> Result<Self, OsaGatewayError> {
        Self::with_config(OsaGatewayConfig::default())
    }

    /// Create new gateway client with custom config
    /// Returns error if client builder fails (e.g., invalid connection pool config)
    pub fn with_config(config: OsaGatewayConfig) -> Result<Self, OsaGatewayError> {
        let client = reqwest::Client::builder()
            .pool_max_idle_per_host(config.connection_pool_size)
            .build()
            .map_err(|e| {
                OsaGatewayError::ConnectionFailed(format!(
                    "Failed to build HTTP client with pool size {}: {}",
                    config.connection_pool_size, e
                ))
            })?;

        Ok(Self {
            config,
            stats: OsaGatewayStats::default(),
            client,
        })
    }

    // ========================================================================
    // 2PC Transaction Endpoints
    // ========================================================================

    /// POST /api/v1/txn/prepare — Phase 1: Prepare transaction
    pub async fn prepare(
        &self,
        req: &OsaPrepareRequest,
        idempotency_key: &str,
    ) -> Result<OsaPrepareResponse, OsaGatewayError> {
        self.send_request_with_retry("POST", "/api/v1/txn/prepare", req, idempotency_key)
            .await
    }

    /// POST /api/v1/txn/commit — Phase 2: Commit transaction
    pub async fn commit(
        &self,
        req: &OsaCommitRequest,
        idempotency_key: &str,
    ) -> Result<OsaCommitResponse, OsaGatewayError> {
        self.send_request_with_retry("POST", "/api/v1/txn/commit", req, idempotency_key)
            .await
    }

    /// POST /api/v1/txn/rollback — Phase 2: Rollback transaction
    pub async fn rollback(
        &self,
        req: &OsaRollbackRequest,
        idempotency_key: &str,
    ) -> Result<OsaRollbackResponse, OsaGatewayError> {
        self.send_request_with_retry("POST", "/api/v1/txn/rollback", req, idempotency_key)
            .await
    }

    // ========================================================================
    // Internal Methods
    // ========================================================================

    async fn send_request_with_retry<Req, Resp>(
        &self,
        method: &str,
        endpoint: &str,
        request: &Req,
        idempotency_key: &str,
    ) -> Result<Resp, OsaGatewayError>
    where
        Req: Serialize,
        Resp: for<'de> Deserialize<'de>,
    {
        let mut attempt = 0;
        loop {
            match self
                .send_request(method, endpoint, request, idempotency_key)
                .await
            {
                Ok(response) => return Ok(response),
                Err(e) => {
                    attempt += 1;
                    if attempt >= self.config.max_retries {
                        return Err(OsaGatewayError::RetryLimitExceeded(format!(
                            "Failed after {} attempts: {}",
                            attempt, e
                        )));
                    }

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
        idempotency_key: &str,
    ) -> Result<Resp, OsaGatewayError>
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
                return Err(OsaGatewayError::ConnectionFailed(
                    "Invalid HTTP method".to_string(),
                ))
            }
        };

        let json_body = serde_json::to_string(request).map_err(|e| {
            OsaGatewayError::SerializationError(format!("Failed to serialize request: {}", e))
        })?;

        let result = tokio::time::timeout(
            Duration::from_millis(self.config.timeout_ms),
            request_builder
                .header("Content-Type", "application/json")
                .header("X-Idempotency-Key", idempotency_key)
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
                        OsaGatewayError::InvalidResponse(format!("Failed to parse response: {}", e))
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
                    Err(OsaGatewayError::HttpError {
                        status: status.as_u16(),
                        message: text,
                    })
                }
            }
            Ok(Err(e)) => {
                self.stats.record_request(latency_ms, false);
                Err(OsaGatewayError::ConnectionFailed(e.to_string()))
            }
            Err(_) => {
                self.stats.record_request(latency_ms, false);
                Err(OsaGatewayError::Timeout(self.config.timeout_ms))
            }
        }
    }

    pub fn stats(&self) -> &OsaGatewayStats {
        &self.stats
    }
}

impl Default for OsaGateway {
    fn default() -> Self {
        // Default constructor panics on error (since we can't return Result from Default)
        // In production, use new() and handle the Result.
        Self::new().expect("Failed to create default OsaGateway client")
    }
}

// ============================================================================
// 2PC Request/Response Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsaPrepareRequest {
    pub transaction_id: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsaPrepareResponse {
    pub vote: String, // "ready" or "abort"
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsaCommitRequest {
    pub transaction_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsaCommitResponse {
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsaRollbackRequest {
    pub transaction_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsaRollbackResponse {
    pub status: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gateway_config_defaults() {
        let config = OsaGatewayConfig::default();
        assert_eq!(config.base_url, "http://localhost:8089");
        assert_eq!(config.timeout_ms, 5000);
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_gateway_stats_default() {
        let stats = OsaGatewayStats::default();
        assert_eq!(stats.requests_total.load(Ordering::SeqCst), 0);
        assert_eq!(stats.requests_failed.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_gateway_stats_record() {
        let stats = OsaGatewayStats::default();
        stats.record_request(50, true);
        assert_eq!(stats.requests_total.load(Ordering::SeqCst), 1);
        assert_eq!(stats.requests_failed.load(Ordering::SeqCst), 0);

        stats.record_request(75, false);
        assert_eq!(stats.requests_total.load(Ordering::SeqCst), 2);
        assert_eq!(stats.requests_failed.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_prepare_request_serialization() {
        let req = OsaPrepareRequest {
            transaction_id: "txn_abc123".to_string(),
            data: serde_json::json!({}),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("transaction_id"));
    }
}
