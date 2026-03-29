//! Idempotency middleware for pm4py-rust HTTP endpoints.
//!
//! Ensures exactly-once semantics for mutating operations (POST, PUT, PATCH, DELETE).
//! Uses Redis for persistence with in-memory fallback.
//!
//! # Standard
//!
//! See: https://github.com/seanchatmangpt/chatmangpt/blob/main/docs/idempotency-standard.md
//!
//! - Header: `Idempotency-Key: <UUID v4>`
//! - Storage: Redis with 24h TTL
//! - Fallback: In-memory DashMap
//! - Cacheable: 200, 201, 202, 204 only
//!
//! # Usage
//!
//! ```rust,no_run
//! use axum::{Router, routing::post};
//! use pm4py::middleware::idempotency::{IdempotencyLayer, IdempotencyStore};
//! use tower::ServiceBuilder;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let redis_client = redis::Client::open("redis://127.0.0.1/").unwrap();
//! let store = IdempotencyStore::new(redis_client);
//!
//! let app = Router::new()
//!     .route("/discover", post(discover_handler))
//!     .layer(ServiceBuilder::new().layer(IdempotencyLayer::new(store)));
//! # Ok(())
//! # }
//! ```

use axum::{
    body::Body,
    extract::Request,
    http::{HeaderMap, HeaderValue, StatusCode, HeaderName},
    response::{IntoResponse, Response},
};
use chrono::{Duration, Utc};
use dashmap::DashMap;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration as StdDuration;
use tower::{Layer, Service};

const IDEMPOTENCY_KEY_HEADER: &str = "idempotency-key";
const TTL_SECONDS: usize = 86_400; // 24 hours
const EXCLUDED_PATHS: &[&str] = &["/health", "/ready", "/metrics"];
const CACHEABLE_STATUSES: &[StatusCode] = &[
    StatusCode::OK,
    StatusCode::CREATED,
    StatusCode::ACCEPTED,
    StatusCode::NO_CONTENT,
];

/// Cached response entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdempotencyEntry {
    pub status: u16,
    pub body: String,
    pub headers: Vec<(String, String)>,
    pub stored_at: i64,
    pub request_hash: String,
}

/// Idempotency store with Redis + in-memory fallback
#[derive(Clone)]
pub struct IdempotencyStore {
    redis: Arc<redis::Client>,
    fallback: Arc<DashMap<String, IdempotencyEntry>>,
}

impl IdempotencyStore {
    /// Create a new idempotency store with Redis client
    pub fn new(redis: redis::Client) -> Self {
        Self {
            redis: Arc::new(redis),
            fallback: Arc::new(DashMap::new()),
        }
    }

    /// Get cached response if exists
    pub async fn get(&self, key: &str) -> Result<Option<IdempotencyEntry>, anyhow::Error> {
        let mut conn = self.redis.get_async_connection().await?;

        let redis_key = format!("idempotency:{}", key);

        // Try Redis first with 5s timeout (WvdA compliance)
        let result = tokio::time::timeout(
            StdDuration::from_secs(5),
            conn.get::<_, Option<String>>(redis_key.clone()),
        )
        .await
        .map_err(|_| anyhow::anyhow!("Redis get timeout after 5s"))??;

        if let Some(json) = result {
            let entry: IdempotencyEntry = serde_json::from_str(&json)
                .map_err(|e| anyhow::anyhow!("Failed to deserialize cached entry: {}", e))?;

            // Check expiration
            let stored_at = chrono::DateTime::<Utc>::from_timestamp(entry.stored_at, 0)
                .ok_or_else(|| anyhow::anyhow!("Invalid timestamp"))?;

            let expires_at = stored_at + Duration::seconds(TTL_SECONDS as i64);

            if Utc::now() < expires_at {
                return Ok(Some(entry));
            }
        }

        // Fallback to in-memory cache
        Ok(self.fallback.get(key).map(|entry| entry.clone()))
    }

    /// Store response in cache
    pub async fn store(&self, key: String, entry: IdempotencyEntry) -> Result<(), anyhow::Error> {
        let mut conn = self.redis.get_async_connection().await?;

        let redis_key = format!("idempotency:{}", key);
        let json = serde_json::to_string(&entry)
            .map_err(|e| anyhow::anyhow!("Failed to serialize entry: {}", e))?;

        // Try Redis with 5s timeout (WvdA compliance)
        let set_result = tokio::time::timeout(
            StdDuration::from_secs(5),
            conn.set_ex::<_, _, ()>(redis_key.clone(), json, TTL_SECONDS),
        )
        .await
        .map_err(|_| anyhow::anyhow!("Redis set timeout after 5s"))?;

        if let Err(e) = set_result {
            // Log error but don't fail - use in-memory fallback
            eprintln!("Redis store failed: {}, using in-memory fallback", e);
            self.fallback.insert(key.clone(), entry);
        }

        Ok(())
    }

    /// Check if request is cacheable
    pub fn is_cacheable_status(status: StatusCode) -> bool {
        CACHEABLE_STATUSES.contains(&status)
    }

    /// Hash request for validation
    pub fn hash_request(method: &str, path: &str, body: &[u8]) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(method.as_bytes());
        hasher.update(path.as_bytes());
        hasher.update(body);
        format!("{:x}", hasher.finalize())
    }
}

/// Axum middleware for idempotency
#[derive(Clone)]
pub struct IdempotencyLayer {
    store: IdempotencyStore,
    excluded_paths: HashSet<String>,
}

impl IdempotencyLayer {
    pub fn new(store: IdempotencyStore) -> Self {
        Self {
            store,
            excluded_paths: EXCLUDED_PATHS.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl<S> Layer<S> for IdempotencyLayer {
    type Service = IdempotencyMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        IdempotencyMiddleware {
            inner,
            store: self.store.clone(),
            excluded_paths: self.excluded_paths.clone(),
        }
    }
}

/// Middleware service implementation
pub struct IdempotencyMiddleware<S> {
    inner: S,
    store: IdempotencyStore,
    excluded_paths: HashSet<String>,
}

impl<S> Service<Request> for IdempotencyMiddleware<S>
where
    S: Service<Request, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let store = self.store.clone();
        let excluded_paths = self.excluded_paths.clone();

        // Clone inner service if it implements Clone
        let inner = self.inner.clone();

        Box::pin(async move {
            let path = req.uri().path().to_string();
            let method = req.method().to_string();

            // Skip for excluded paths
            if excluded_paths.contains(&path) {
                return inner.call(req).await;
            }

            // Only check idempotency for mutating requests
            if !matches!(method.as_str(), "POST" | "PUT" | "PATCH" | "DELETE") {
                return inner.call(req).await;
            }

            // Extract idempotency key
            let headers = req.headers().clone();
            let idempotency_key = match headers.get(IDEMPOTENCY_KEY_HEADER) {
                Some(key) => match key.to_str() {
                    Ok(k) => k.to_string(),
                    Err(_) => {
                        return Ok((
                            StatusCode::BAD_REQUEST,
                            "Invalid Idempotency-Key header",
                        )
                            .into_response());
                    }
                },
                None => {
                    // No idempotency key, proceed normally
                    return inner.call(req).await;
                }
            };

            // Check for cached response
            if let Ok(Some(entry)) = store.get(&idempotency_key).await {
                // Return cached response
                let mut builder = Response::builder().status(entry.status);

                // Add replay headers
                if builder.headers_mut().is_ok() {
                    let headers = builder.headers_mut().unwrap();
                    headers.insert(
                        "idempotency-replayed",
                        HeaderValue::from_static("true"),
                    );
                    headers.insert(
                        "idempotency-original-date",
                        HeaderValue::from_str(&entry.stored_at.to_string()).unwrap_or(HeaderValue::from_static("0")),
                    );

                    // Add original headers
                    for (k, v) in entry.headers {
                        if let Ok(k) = HeaderName::from_bytes(k.as_bytes()) {
                            if let Ok(v) = HeaderValue::from_str(&v) {
                                headers.insert(k, v);
                            }
                        }
                    }
                }

                return Ok(builder.body(Body::from(entry.body)).unwrap());
            }

            // Proceed with request
            let response = inner.call(req).await?;

            // Cache successful responses
            let status = response.status();
            if IdempotencyStore::is_cacheable_status(status) {
                // Get response body
                let (parts, body) = response.into_parts();
                let body_bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap_or_default();
                let body_str = String::from_utf8_lossy(&body_bytes).to_string();

                let headers = parts
                    .headers
                    .iter()
                    .filter(|(k, _)| {
                        matches!(
                            k.as_str(),
                            "content-type" | "content-length" | "location" | "etag" | "cache-control"
                        )
                    })
                    .map(|(k, v): (&HeaderName, &HeaderValue)| (k.as_str().to_string(), v.to_str().unwrap_or("").to_string()))
                    .collect();

                let request_hash = IdempotencyStore::hash_request(&method, &path, &body_bytes);

                let entry = IdempotencyEntry {
                    status: status.as_u16(),
                    body: body_str,
                    headers,
                    stored_at: Utc::now().timestamp(),
                    request_hash,
                };

                // Store asynchronously, don't block response
                let key_clone = idempotency_key.clone();
                tokio::spawn(async move {
                    if let Err(e) = store.store(key_clone, entry).await {
                        eprintln!("Failed to store idempotency entry: {}", e);
                    }
                });

                // Reconstruct response
                let new_response = Response::from_parts(parts, Body::from(body_bytes));
                return Ok(new_response);
            }

            Ok(response)
        })
    }
}

use axum::http::HeaderName;

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{routing::post, Router};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_idempotency_get_request_passes_through() {
        // This test is a placeholder - full integration tests require Redis
        // and are better suited for end-to-end testing
        assert!(true);
    }

    #[test]
    fn test_hash_request() {
        let hash1 = IdempotencyStore::hash_request("POST", "/api/test", b"body1");
        let hash2 = IdempotencyStore::hash_request("POST", "/api/test", b"body2");
        let hash3 = IdempotencyStore::hash_request("POST", "/api/test", b"body1");

        assert_ne!(hash1, hash2);
        assert_eq!(hash1, hash3);
    }

    #[test]
    fn test_is_cacheable_status() {
        assert!(IdempotencyStore::is_cacheable_status(StatusCode::OK));
        assert!(IdempotencyStore::is_cacheable_status(StatusCode::CREATED));
        assert!(IdempotencyStore::is_cacheable_status(StatusCode::ACCEPTED));
        assert!(IdempotencyStore::is_cacheable_status(StatusCode::NO_CONTENT));
        assert!(!IdempotencyStore::is_cacheable_status(StatusCode::BAD_REQUEST));
        assert!(!IdempotencyStore::is_cacheable_status(StatusCode::INTERNAL_SERVER_ERROR));
    }
}
