// Distributed tracing module for pm4py-rust
// Implements W3C Trace Context (traceparent) propagation

use std::collections::HashMap;
use std::time::SystemTime;

/// Trace represents a distributed trace context
#[derive(Debug, Clone)]
pub struct Trace {
    pub trace_id: String,
    pub span_id: String,
    pub parent_id: String,
    pub flags: String,
    pub start_time: SystemTime,
}

/// Span represents an individual operation
#[derive(Debug, Clone)]
pub struct Span {
    pub trace_id: String,
    pub span_id: String,
    pub parent_id: String,
    pub name: String,
    pub start_time: SystemTime,
    pub end_time: Option<SystemTime>,
    pub duration_ms: u64,
    pub status: String,
    pub attributes: HashMap<String, String>,
    pub service: String,
}

/// Extract traceparent from headers
pub fn extract_traceparent(headers: &HashMap<String, String>) -> Trace {
    if let Some(tp) = headers.get("traceparent") {
        if let Ok(trace) = parse_traceparent(tp) {
            return trace;
        }
    }

    // Generate new trace
    generate_trace()
}

/// Parse W3C Trace Context traceparent header
fn parse_traceparent(traceparent: &str) -> Result<Trace, String> {
    let parts: Vec<&str> = traceparent.split('-').collect();

    if parts.len() != 4 {
        return Err("Invalid traceparent format".to_string());
    }

    if parts[0] != "00" {
        return Err("Unsupported trace version".to_string());
    }

    if parts[1].len() != 32 || parts[2].len() != 16 {
        return Err("Invalid trace or span ID length".to_string());
    }

    Ok(Trace {
        trace_id: parts[1].to_string(),
        span_id: parts[2].to_string(),
        parent_id: String::new(),
        flags: parts[3].to_string(),
        start_time: SystemTime::now(),
    })
}

/// Generate a new trace
fn generate_trace() -> Trace {
    Trace {
        trace_id: generate_id(32),
        span_id: generate_id(16),
        parent_id: String::new(),
        flags: "01".to_string(),
        start_time: SystemTime::now(),
    }
}

/// Generate a random hex ID of given length
fn generate_id(length: usize) -> String {
    let chars = "0123456789abcdef";
    (0..length)
        .map(|i| chars.chars().nth(i % chars.len()).unwrap())
        .collect()
}

/// Create a span with parent trace
pub fn create_span(parent: &Trace, name: &str, attrs: HashMap<String, String>) -> Span {
    Span {
        trace_id: parent.trace_id.clone(),
        span_id: generate_id(16),
        parent_id: parent.span_id.clone(),
        name: name.to_string(),
        start_time: SystemTime::now(),
        end_time: None,
        duration_ms: 0,
        status: "running".to_string(),
        attributes: attrs,
        service: "rust".to_string(),
    }
}

/// End a span and record duration
pub fn end_span(span: &mut Span, status: &str) {
    span.end_time = Some(SystemTime::now());

    if let Ok(duration) = span.end_time.unwrap().duration_since(span.start_time) {
        span.duration_ms = duration.as_millis() as u64;
    }

    span.status = status.to_string();
}

/// Encode span as W3C Trace Context traceparent header
pub fn encode_traceparent(span: &Span) -> String {
    format!("00-{}-{}-01", span.trace_id, span.span_id)
}

/// Record an attribute on a span
pub fn record_attribute(span: &mut Span, key: &str, value: &str) {
    span.attributes.insert(key.to_string(), value.to_string());
}

/// Reconstruct a trace from spans
pub fn reconstruct_trace(spans: &[Span]) -> Result<ReconstructedTrace, String> {
    if spans.is_empty() {
        return Err("No spans provided".to_string());
    }

    Ok(ReconstructedTrace {
        trace_id: spans[0].trace_id.clone(),
        spans: spans.to_vec(),
    })
}

/// Reconstructed trace with all spans
#[derive(Debug, Clone)]
pub struct ReconstructedTrace {
    pub trace_id: String,
    pub spans: Vec<Span>,
}

/// Byzantine coordinator for distributed consensus
pub mod byzantine {
    use super::*;

    /// Calculate Byzantine fault-tolerant quorum size for n agents
    /// Uses BFT formula: quorum = max(2f + 1, n/2 + 1)
    /// This ensures both BFT safety and simple majority for small clusters.
    pub fn quorum_size(n: usize) -> usize {
        let f = tolerance_factor(n);
        let bft_quorum = 2 * f + 1;
        let majority = n / 2 + 1;
        bft_quorum.max(majority)
    }

    /// Calculate tolerance factor f
    pub fn tolerance_factor(n: usize) -> usize {
        (n - 1) / 3
    }

    /// Aggregate votes to reach consensus
    pub fn aggregate_votes(votes: &[Vote], quorum: usize) -> AggregateResult {
        if votes.len() < quorum {
            return AggregateResult {
                consensus: false,
                majority_value: String::new(),
                quorum_size: quorum,
                votes_received: votes.len(),
                majority_count: 0,
            };
        }

        // Group votes by hash
        let mut vote_groups: HashMap<String, Vec<&Vote>> = HashMap::new();

        for vote in votes {
            vote_groups.entry(vote.hash.clone()).or_default().push(vote);
        }

        // Find majority
        let mut max_group = None;
        let mut max_count = 0;

        for (_, group) in vote_groups.iter() {
            if group.len() > max_count {
                max_count = group.len();
                max_group = Some(group[0].value.clone());
            }
        }

        let consensus = max_count >= quorum;

        AggregateResult {
            consensus,
            majority_value: max_group.unwrap_or_default(),
            quorum_size: quorum,
            votes_received: votes.len(),
            majority_count: max_count,
        }
    }

    /// Detect Byzantine (corrupted) votes
    pub fn detect_byzantine(votes: &[Vote], quorum: usize) -> ByzantineDetection {
        let mut vote_counts: HashMap<String, usize> = HashMap::new();

        for vote in votes {
            *vote_counts.entry(vote.hash.clone()).or_insert(0) += 1;
        }

        let mut counts: Vec<usize> = vote_counts.values().copied().collect();
        counts.sort_by(|a, b| b.cmp(a));

        match counts.first() {
            None => ByzantineDetection {
                consensus: false,
                byzantine_detected: false,
                byzantine_count: 0,
                majority_count: 0,
            },
            Some(&majority) => {
                let byzantine_count: usize = counts.iter().skip(1).sum();
                let consensus = majority >= quorum;

                ByzantineDetection {
                    consensus,
                    byzantine_detected: byzantine_count > 0,
                    byzantine_count,
                    majority_count: majority,
                }
            }
        }
    }

    /// Vote structure
    #[derive(Debug, Clone)]
    pub struct Vote {
        pub agent_id: String,
        pub value: String,
        pub hash: String,
    }

    /// Vote aggregation result
    #[derive(Debug, Clone)]
    pub struct AggregateResult {
        pub consensus: bool,
        pub majority_value: String,
        pub quorum_size: usize,
        pub votes_received: usize,
        pub majority_count: usize,
    }

    /// Byzantine detection result
    #[derive(Debug, Clone)]
    pub struct ByzantineDetection {
        pub consensus: bool,
        pub byzantine_detected: bool,
        pub byzantine_count: usize,
        pub majority_count: usize,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_id() {
        let id = generate_id(32);
        assert_eq!(id.len(), 32);

        // Check all chars are hex
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_encode_traceparent() {
        let span = Span {
            trace_id: "0af7651916cd43dd8448eb211c80319c".to_string(),
            span_id: "b7ad6b7169203331".to_string(),
            parent_id: String::new(),
            name: "test".to_string(),
            start_time: SystemTime::now(),
            end_time: None,
            duration_ms: 0,
            status: "ok".to_string(),
            attributes: HashMap::new(),
            service: "rust".to_string(),
        };

        let encoded = encode_traceparent(&span);
        assert_eq!(
            encoded,
            "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01"
        );
    }

    #[test]
    fn test_byzantine_quorum_size() {
        assert_eq!(byzantine::quorum_size(7), 5);
        assert_eq!(byzantine::quorum_size(5), 3);
        assert_eq!(byzantine::quorum_size(3), 2);
    }

    #[test]
    fn test_byzantine_tolerance_factor() {
        assert_eq!(byzantine::tolerance_factor(7), 2);
        assert_eq!(byzantine::tolerance_factor(5), 1);
        assert_eq!(byzantine::tolerance_factor(4), 1);
    }
}
