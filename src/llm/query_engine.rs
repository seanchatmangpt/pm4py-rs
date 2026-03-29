//! Natural Language Query Engine for Process Intelligence
//!
//! Parses NL queries and routes them to appropriate abstractors.
//! Supports simple keyword-based routing for common queries.

use super::abstract_ocel::abstract_ocel;
use super::domain2_intelligence::{
    answer_causal_question, format_intelligence_answer, IntelligenceAnswer, IntelligenceContext,
};
use super::domain_classifier::{classify_domain, QueryDomain};
use super::groq_client::{groq_chat_with_config, GroqMessage, GroqRequestConfig};
use super::{abstract_dfg, abstract_event_log, abstract_petri_net};
use crate::log::EventLog;
use crate::models::{DirectlyFollowsGraph, PetriNet};
use crate::ocpm::object_log::ObjectCentricEventLog;
use serde::{Deserialize, Serialize};

/// Result of a process intelligence query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub query: String,
    pub response: String,
    pub response_length: usize,
    pub model_type: String,
    /// Which domain handled this query
    pub domain: QueryDomain,
    /// Structured intelligence answer (only present for Domain 2 queries)
    pub intelligence: Option<IntelligenceAnswer>,
}

/// Query types
#[derive(Debug, Clone, Copy, PartialEq)]
enum QueryType {
    Bottleneck,
    Duration,
    Variant,
    Rework,
    Soundness,
    Resource,
    Activity,
    General,
}

/// Execute a natural language query grounded in OCEL process data via Groq LLM.
///
/// This implements Connection 4 from "No AI Without PI": RAG over real process logs.
///
/// When `ocel` and `api_key` are both `Some`, the OCEL log is abstracted into a structured
/// system message via `abstract_ocel()` and sent to Groq as context. The LLM is instructed
/// to answer ONLY from the process data — no hallucination.
///
/// Falls back to the synchronous `query()` when `api_key` is absent (offline mode).
///
/// # Arguments
/// - `q` — natural language question about the process
/// - `ocel` — optional OCEL 2.0 log to ground the answer
/// - `log` — optional traditional event log (passed to fallback `query()`)
/// - `model` — optional Petri net (passed to fallback `query()`)
/// - `dfg` — optional DFG (passed to fallback `query()`)
/// - `api_key` — optional Groq API key; when absent, falls back to offline mode
pub async fn query_with_ocel_llm(
    q: &str,
    ocel: Option<&ObjectCentricEventLog>,
    log: Option<&EventLog>,
    model: Option<&PetriNet>,
    dfg: Option<&DirectlyFollowsGraph>,
    api_key: Option<&str>,
) -> QueryResult {
    // If we have both an OCEL log and an API key, use RAG grounding
    if let (Some(ocel_log), Some(key)) = (ocel, api_key) {
        let process_context = abstract_ocel(ocel_log);

        let messages = vec![
            GroqMessage {
                role: "system".to_string(),
                content: format!(
                    "You are a process mining expert. Answer the user's question using ONLY \
                     the process data provided below. Do not use external knowledge or make \
                     assumptions beyond what the data shows. If the data does not contain \
                     enough information, say so explicitly.\n\n{}",
                    process_context
                ),
            },
            GroqMessage {
                role: "user".to_string(),
                content: q.to_string(),
            },
        ];

        match groq_chat_with_config(key, messages, GroqRequestConfig::for_ocel_query()).await {
            Ok(resp) => {
                let response = resp
                    .choices
                    .into_iter()
                    .next()
                    .map(|c| c.message.content)
                    .unwrap_or_else(|| "No response generated.".to_string());
                let response_length = response.len();
                return QueryResult {
                    query: q.to_string(),
                    response,
                    response_length,
                    model_type: "ocel_llm".to_string(),
                    domain: QueryDomain::Intelligence,
                    intelligence: None,
                };
            }
            Err(e) => {
                // LLM call failed — fall through to offline mode
                let response = format!("LLM query failed ({}). Offline analysis:", e);
                let fallback = query(q, log, model, dfg);
                return QueryResult {
                    query: q.to_string(),
                    response: format!("{} {}", response, fallback.response),
                    response_length: fallback.response_length,
                    model_type: fallback.model_type,
                    domain: fallback.domain,
                    intelligence: fallback.intelligence,
                };
            }
        }
    }

    // No OCEL or no API key — fall back to synchronous offline query
    query(q, log, model, dfg)
}

/// Execute a natural language process intelligence query
///
/// Supports queries like:
/// - "What is the bottleneck?" → analyzes Petri net for bottlenecks
/// - "What's the case duration?" → analyzes event log durations
/// - "Show me variants" → lists top execution traces
/// - "Detect rework" → finds loops and rework patterns
///
/// Returns: JSON with query, response, and metadata
pub fn query(
    q: &str,
    log: Option<&EventLog>,
    model: Option<&PetriNet>,
    dfg: Option<&DirectlyFollowsGraph>,
) -> QueryResult {
    // --- Domain classification (Domain 1 vs Domain 2) ---
    let classification = classify_domain(q);

    if classification.domain == QueryDomain::Intelligence {
        let ctx = IntelligenceContext {
            log,
            petri_net: model,
        };
        let answer = answer_causal_question(q, &ctx);
        let response = format_intelligence_answer(&answer);
        let response_length = response.len();
        return QueryResult {
            query: q.to_string(),
            domain: QueryDomain::Intelligence,
            response_length,
            model_type: if log.is_some() {
                "event_log".to_string()
            } else {
                "none".to_string()
            },
            response,
            intelligence: Some(answer),
        };
    }

    // --- Domain 1: existing keyword-based routing ---
    let query_type = classify_query(q);

    let response = match query_type {
        QueryType::Bottleneck => {
            if let Some(pn) = model {
                abstract_petri_net(pn)
            } else {
                "Cannot analyze bottlenecks without a Petri net model.".to_string()
            }
        }
        QueryType::Duration => {
            if let Some(event_log) = log {
                abstract_event_log(event_log)
            } else {
                "Cannot analyze durations without an event log.".to_string()
            }
        }
        QueryType::Variant => {
            if let Some(event_log) = log {
                abstract_event_log(event_log)
            } else {
                "Cannot analyze variants without an event log.".to_string()
            }
        }
        QueryType::Rework => {
            if let Some(pn) = model {
                abstract_petri_net(pn)
            } else if let Some(dfg_model) = dfg {
                abstract_dfg(dfg_model)
            } else {
                "Cannot detect rework without a model.".to_string()
            }
        }
        QueryType::Soundness => {
            if let Some(pn) = model {
                abstract_petri_net(pn)
            } else {
                "Cannot check soundness without a Petri net model.".to_string()
            }
        }
        QueryType::Resource => {
            if let Some(event_log) = log {
                abstract_event_log(event_log)
            } else {
                "Cannot analyze resources without an event log.".to_string()
            }
        }
        QueryType::Activity => {
            if let Some(dfg_model) = dfg {
                abstract_dfg(dfg_model)
            } else if let Some(event_log) = log {
                abstract_event_log(event_log)
            } else {
                "Cannot analyze activities without a model or event log.".to_string()
            }
        }
        QueryType::General => {
            // Default: provide general analysis from all available sources
            if let Some(pn) = model {
                abstract_petri_net(pn)
            } else if let Some(event_log) = log {
                abstract_event_log(event_log)
            } else if let Some(dfg_model) = dfg {
                abstract_dfg(dfg_model)
            } else {
                "No process model or event log provided.".to_string()
            }
        }
    };

    QueryResult {
        query: q.to_string(),
        response_length: response.len(),
        model_type: determine_model_type(query_type, model, dfg, log),
        response,
        domain: QueryDomain::Description,
        intelligence: None,
    }
}

/// Classify query into a type based on keywords
fn classify_query(q: &str) -> QueryType {
    let q_lower = q.to_lowercase();

    if q_lower.contains("bottleneck") || q_lower.contains("slow") || q_lower.contains("congestion")
    {
        QueryType::Bottleneck
    } else if q_lower.contains("duration")
        || q_lower.contains("time")
        || q_lower.contains("how long")
        || q_lower.contains("latency")
    {
        QueryType::Duration
    } else if q_lower.contains("variant") || q_lower.contains("trace") || q_lower.contains("path") {
        QueryType::Variant
    } else if q_lower.contains("rework")
        || q_lower.contains("loop")
        || q_lower.contains("cycle")
        || q_lower.contains("redo")
    {
        QueryType::Rework
    } else if q_lower.contains("sound")
        || q_lower.contains("deadlock")
        || q_lower.contains("dead end")
    {
        QueryType::Soundness
    } else if q_lower.contains("resource") || q_lower.contains("user") || q_lower.contains("staff")
    {
        QueryType::Resource
    } else if q_lower.contains("activity") || q_lower.contains("step") || q_lower.contains("action")
    {
        QueryType::Activity
    } else {
        QueryType::General
    }
}

/// Determine which model type was used to answer the query
fn determine_model_type(
    query_type: QueryType,
    model: Option<&PetriNet>,
    dfg: Option<&DirectlyFollowsGraph>,
    log: Option<&EventLog>,
) -> String {
    match query_type {
        QueryType::Bottleneck | QueryType::Soundness | QueryType::Rework => {
            if model.is_some() {
                "petri_net".to_string()
            } else if dfg.is_some() {
                "dfg".to_string()
            } else {
                "none".to_string()
            }
        }
        QueryType::Duration | QueryType::Variant | QueryType::Resource => {
            if log.is_some() {
                "event_log".to_string()
            } else {
                "none".to_string()
            }
        }
        QueryType::Activity => {
            if dfg.is_some() {
                "dfg".to_string()
            } else if log.is_some() {
                "event_log".to_string()
            } else {
                "none".to_string()
            }
        }
        QueryType::General => {
            if model.is_some() {
                "petri_net".to_string()
            } else if dfg.is_some() {
                "dfg".to_string()
            } else if log.is_some() {
                "event_log".to_string()
            } else {
                "none".to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_query_bottleneck() {
        assert_eq!(
            classify_query("What is the bottleneck?"),
            QueryType::Bottleneck
        );
        assert_eq!(
            classify_query("Where is the slowdown?"),
            QueryType::Bottleneck
        );
    }

    #[test]
    fn test_classify_query_duration() {
        assert_eq!(
            classify_query("What is the case duration?"),
            QueryType::Duration
        );
        assert_eq!(
            classify_query("How long does it take?"),
            QueryType::Duration
        );
    }

    #[test]
    fn test_classify_query_rework() {
        assert_eq!(classify_query("Detect rework patterns"), QueryType::Rework);
        assert_eq!(classify_query("Show me loops"), QueryType::Rework);
    }

    #[test]
    fn test_query_without_models() {
        let result = query("What is the bottleneck?", None, None, None);
        assert!(result.response.contains("Petri net"));
        assert!(result.model_type == "none");
    }
}
