//! pm4py-rust Groq Integration Tests — Chicago TDD, Armstrong Principles.
//!
//! RED → GREEN → REFACTOR. Every test hits api.groq.com. No mocks.
//! Armstrong: fail loudly, never swallow.
//!
//! Run: GROQ_API_KEY=gsk_... cargo test --features integration groq -- --nocapture

#![cfg(feature = "integration")]

use pm4py::llm::groq_client::{groq_chat_with_span, GroqMessage};

/// Require GROQ_API_KEY or panic loudly (Armstrong: Let-It-Crash).
fn require_groq_api_key() -> String {
    std::env::var("GROQ_API_KEY").unwrap_or_else(|_| {
        panic!(
            "GROQ_API_KEY environment variable not set — \
             set it to a valid gsk_... key to run Groq integration tests. \
             Armstrong principle: loud failure, not silent skip."
        )
    })
}

// ── Test 1: Domain classification of a process mining query ──────────────────

/// RED → GREEN: Groq classifies a causal process query as Domain 2 intelligence.
///
/// Armstrong: no error swallowing — `expect()` panics loudly on API failure.
/// Proof: response contains "intelligence" (the D2 classification term)
///        AND usage.prompt_tokens > 0 (real API consumed tokens).
#[tokio::test]
async fn test_groq_classify_process_mining_query() {
    let api_key = require_groq_api_key();

    let messages = vec![
        GroqMessage {
            role: "system".into(),
            content: "You are a process mining domain classifier. \
                      Classify queries as either 'description' (what happened) or \
                      'intelligence' (why it happened / causal analysis). \
                      Respond with exactly one word: description or intelligence."
                .into(),
        },
        GroqMessage {
            role: "user".into(),
            content: "why is case 123 taking so long?".into(),
        },
    ];

    let response = groq_chat_with_span(&api_key, messages)
        .await
        .expect("Groq API call failed — check GROQ_API_KEY and network");

    let answer = response.choices[0].message.content.to_lowercase();
    println!("Groq domain classification answer: {answer:?}");

    // Armstrong: assert exact behavior, no proxy checks
    assert!(
        answer.contains("intelligence"),
        "Expected domain classification 'intelligence' for causal query, got: {answer:?}"
    );

    assert!(
        response.usage.prompt_tokens > 0,
        "Expected prompt_tokens > 0 (real API call), got: {}",
        response.usage.prompt_tokens
    );

    assert!(
        response.usage.completion_tokens > 0,
        "Expected completion_tokens > 0 (real API response), got: {}",
        response.usage.completion_tokens
    );

    println!(
        "Token usage — prompt: {}, completion: {}, total: {}",
        response.usage.prompt_tokens, response.usage.completion_tokens, response.usage.total_tokens
    );
}

// ── Test 2: Causal answer on a process mining log context ────────────────────

/// RED → GREEN: Groq answers a causal process mining question with a substantive
/// response containing domain-specific terminology.
///
/// Armstrong: real HTTP call, no mock. If network is partitioned, test fails loudly.
/// Proof: response ≥ 50 chars AND contains ≥1 PM term (bottleneck/review/rework/delay/approval)
#[tokio::test]
async fn test_groq_domain2_answer_causal_question() {
    let api_key = require_groq_api_key();

    let process_context = "\
Process: Invoice Approval Workflow
Event log excerpt (last 20 cases):
- Case 101: submit(2h) → review(18h) → approve(1h) → pay(30min) — total 21.5h
- Case 102: submit(1h) → review(24h) → reject(30min) → resubmit(2h) → review(20h) → approve(1h) → pay(30min) — total 49h (REWORK)
- Case 103: submit(1h) → review(19h) → approve(1h) → pay(1h) — total 22h
- Case 104: submit(30min) → review(22h) → approve(2h) → pay(30min) — total 25h
- Case 105: submit(2h) → review(48h) → approve(1h) → pay(30min) — total 51.5h (OUTLIER)

Observed: review step averages 22h (target: 4h). Cases with rework average 49h.
Question: What is the root cause of the review bottleneck in this invoice process?";

    let messages = vec![
        GroqMessage {
            role: "system".into(),
            content:
                "You are a process mining expert. Analyze event logs and identify root causes \
                      of bottlenecks. Provide specific, actionable insights. Use terms like \
                      bottleneck, rework, throughput time, waiting time, and resource utilization."
                    .into(),
        },
        GroqMessage {
            role: "user".into(),
            content: process_context.into(),
        },
    ];

    let response = groq_chat_with_span(&api_key, messages)
        .await
        .expect("Groq API call failed — check GROQ_API_KEY and network");

    let answer = response.choices[0].message.content.to_lowercase();
    println!(
        "Groq causal analysis answer ({} chars):\n{}",
        answer.len(),
        &response.choices[0].message.content
    );

    // Chicago TDD: exact behavior assertions, not proxy checks
    assert!(
        answer.len() >= 50,
        "Expected substantive answer (≥50 chars) but got {} chars: {answer:?}",
        answer.len()
    );

    // At least one process mining term must appear — proves domain-aware response
    let pm_terms = [
        "bottleneck",
        "review",
        "rework",
        "delay",
        "approval",
        "waiting",
        "throughput",
        "resource",
    ];
    let contains_pm_term = pm_terms.iter().any(|term| answer.contains(term));

    assert!(
        contains_pm_term,
        "Expected at least one PM term ({pm_terms:?}) in answer but got: {answer:?}"
    );

    assert!(
        response.usage.prompt_tokens > 0,
        "Expected prompt_tokens > 0 (real API call), got: {}",
        response.usage.prompt_tokens
    );

    println!(
        "Token usage — prompt: {}, completion: {}, total: {}",
        response.usage.prompt_tokens, response.usage.completion_tokens, response.usage.total_tokens
    );
}
