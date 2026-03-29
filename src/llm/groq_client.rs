//! Groq API client for pm4py-rust LLM domain intelligence.
//!
//! Makes real HTTP calls to `api.groq.com` using the OpenAI-compatible
//! chat completions endpoint. No mocks — Armstrong principle: fail loudly.
//!
//! # Usage
//! ```no_run
//! use pm4py::llm::groq_client::{groq_chat_with_span, GroqMessage};
//!
//! # #[tokio::main]
//! # async fn main() {
//! let messages = vec![GroqMessage { role: "user".into(), content: "Hello".into() }];
//! let response = groq_chat_with_span("gsk_...", messages).await.unwrap();
//! println!("{}", response.choices[0].message.content);
//! # }
//! ```

use crate::semconv::llm_attributes::{LLM_MODEL, LLM_PROVIDER};
use crate::semconv::llm_span_names::LLM_INFERENCE_SPAN;
use opentelemetry::global;
use opentelemetry::trace::{Span, Tracer};
use opentelemetry::KeyValue;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

/// The Groq chat completions API endpoint.
pub const GROQ_API_URL: &str = "https://api.groq.com/openai/v1/chat/completions";

/// The default Groq model — fast inference, 128K context, tool support.
pub const GROQ_DEFAULT_MODEL: &str = "openai/gpt-oss-20b";

// ── Error type ────────────────────────────────────────────────────────────────

/// Errors from Groq API calls. Armstrong: no swallowing — every failure is explicit.
#[derive(Debug, Error)]
pub enum GroqError {
    /// HTTP error response from the Groq API (e.g., 401 Unauthorized, 429 Rate Limited).
    #[error("Groq HTTP {status}: {body}")]
    Http { status: u16, body: String },

    /// Network-level transport failure (connection refused, DNS failure, etc.).
    #[error("Groq network error: {0}")]
    Network(#[from] reqwest::Error),

    /// JSON deserialization of the response body failed.
    #[error("Groq response parse error: {0}")]
    Parse(String),
}

// ── Request / Response types ──────────────────────────────────────────────────

/// A single chat message in OpenAI-compatible format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroqMessage {
    pub role: String,
    pub content: String,
}

/// The request body sent to Groq's chat completions endpoint.
/// Uses `max_completion_tokens` + `reasoning_effort` for the reasoning model.
#[derive(Debug, Serialize)]
pub struct GroqRequest {
    pub model: String,
    pub messages: Vec<GroqMessage>,
    pub temperature: f32,
    pub max_completion_tokens: u32,
    pub reasoning_effort: String,
}

/// A single completion choice returned by Groq.
#[derive(Debug, Deserialize)]
pub struct GroqChoice {
    pub message: GroqMessage,
    pub finish_reason: String,
}

/// Token usage statistics from the Groq API response.
#[derive(Debug, Deserialize)]
pub struct GroqUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Full response from the Groq chat completions endpoint.
#[derive(Debug, Deserialize)]
pub struct GroqResponse {
    pub id: String,
    pub model: String,
    pub choices: Vec<GroqChoice>,
    pub usage: GroqUsage,
}

// ── Per-request configuration ─────────────────────────────────────────────────

/// Per-request configuration for Groq API calls.
///
/// Decouples token limits and temperature from the hardcoded defaults,
/// enabling OCEL-grounded RAG queries (which need 2048+ tokens) without
/// breaking the existing `groq_chat` callers (which use 512).
#[derive(Debug, Clone)]
pub struct GroqRequestConfig {
    /// Maximum completion tokens. Default: 512 (preserves existing behaviour).
    pub max_completion_tokens: u32,
    /// Sampling temperature. Default: 1.0 (preserves existing behaviour).
    pub temperature: f32,
    /// HTTP + read timeout in seconds. Default: 30.
    pub timeout_secs: u64,
}

impl Default for GroqRequestConfig {
    fn default() -> Self {
        Self {
            max_completion_tokens: 512,
            temperature: 1.0,
            timeout_secs: 30,
        }
    }
}

impl GroqRequestConfig {
    /// Configuration tuned for OCEL-grounded RAG queries:
    /// - 2048 tokens for full OCEL context + answer
    /// - Lower temperature (0.3) for factual grounding
    /// - Longer timeout (60 s) for larger payloads
    pub fn for_ocel_query() -> Self {
        Self {
            max_completion_tokens: 2048,
            temperature: 0.3,
            timeout_secs: 60,
        }
    }
}

// ── Core client function ──────────────────────────────────────────────────────

/// Call the Groq chat completions API with per-request configuration.
///
/// Armstrong principle: panics on initialization errors; returns `Err` on API/network errors.
pub async fn groq_chat_with_config(
    api_key: &str,
    messages: Vec<GroqMessage>,
    config: GroqRequestConfig,
) -> Result<GroqResponse, GroqError> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(config.timeout_secs))
        .build()
        .expect("Failed to build reqwest client — this is a programming error");

    let req_body = GroqRequest {
        model: GROQ_DEFAULT_MODEL.to_string(),
        messages,
        temperature: config.temperature,
        max_completion_tokens: config.max_completion_tokens,
        reasoning_effort: "medium".to_string(),
    };

    let http_resp = client
        .post(GROQ_API_URL)
        .bearer_auth(api_key)
        .json(&req_body)
        .send()
        .await?;

    let status = http_resp.status().as_u16();

    if !http_resp.status().is_success() {
        let body = http_resp.text().await.unwrap_or_default();
        return Err(GroqError::Http { status, body });
    }

    let parsed: GroqResponse = http_resp
        .json()
        .await
        .map_err(|e| GroqError::Parse(e.to_string()))?;

    Ok(parsed)
}

/// Call the Groq chat completions API with default configuration (512 tokens, 30s timeout).
///
/// This is the backward-compatible entry point. Delegates to `groq_chat_with_config`.
pub async fn groq_chat(
    api_key: &str,
    messages: Vec<GroqMessage>,
) -> Result<GroqResponse, GroqError> {
    groq_chat_with_config(api_key, messages, GroqRequestConfig::default()).await
}

// ── OTEL-instrumented wrapper ─────────────────────────────────────────────────

/// Call Groq and emit an `llm.inference` OTEL span with provider and model attributes.
///
/// The span is emitted regardless of whether a collector is running — if no
/// global tracer provider is configured, the span goes to the no-op tracer.
///
/// Attributes set on the span:
/// - `llm.provider = "groq"`
/// - `llm.model = "openai/gpt-oss-20b"`
pub async fn groq_chat_with_span(
    api_key: &str,
    messages: Vec<GroqMessage>,
) -> Result<GroqResponse, GroqError> {
    let tracer = global::tracer("pm4py-rust");
    let mut span = tracer.start(LLM_INFERENCE_SPAN);

    span.set_attribute(KeyValue::new(LLM_PROVIDER, "groq"));
    span.set_attribute(KeyValue::new(LLM_MODEL, GROQ_DEFAULT_MODEL));

    let result = groq_chat(api_key, messages).await;

    match &result {
        Ok(resp) => {
            span.set_attribute(KeyValue::new(
                "llm.token.input",
                resp.usage.prompt_tokens as i64,
            ));
            span.set_attribute(KeyValue::new(
                "llm.token.output",
                resp.usage.completion_tokens as i64,
            ));
        }
        Err(e) => {
            span.set_status(opentelemetry::trace::Status::error(e.to_string()));
        }
    }

    // span drops here — automatically ends
    result
}
