//! LLM Process Intelligence Module
//!
//! Converts process models (Petri nets, DFGs, event logs) into plain-English narratives
//! suitable for language models to reason about process state, bottlenecks, and anomalies.
//!
//! Modules:
//! - abstract_petri_net: Petri net → English abstraction
//! - abstract_dfg: Directly-Follows Graph → English abstraction
//! - abstract_event_log: Event log → English abstraction
//! - query_engine: Natural language query routing and execution

pub mod abstract_dfg;
pub mod abstract_event_log;
pub mod abstract_ocel;
pub mod abstract_petri_net;
pub mod domain2_intelligence;
pub mod domain_classifier;
pub mod groq_client;
pub mod query_engine;

pub use abstract_dfg::abstract_dfg;
pub use abstract_event_log::abstract_event_log;
pub use abstract_ocel::abstract_ocel;
pub use abstract_petri_net::abstract_petri_net;
pub use domain2_intelligence::{
    answer_causal_question, format_intelligence_answer, IntelligenceAnswer,
};
pub use domain_classifier::{classify_domain, DomainClassification, QueryDomain};
pub use groq_client::{
    groq_chat, groq_chat_with_config, groq_chat_with_span, GroqError, GroqMessage,
    GroqRequestConfig, GroqResponse, GROQ_DEFAULT_MODEL,
};
pub use query_engine::{query, query_with_ocel_llm, QueryResult};
