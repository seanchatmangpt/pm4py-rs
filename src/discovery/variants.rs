//! Comprehensive Variant Detection, Analysis, and Filtering
//!
//! A complete framework for variant fingerprinting, frequency analysis, filtering, similarity,
//! and metrics computation. All operations are optimized for performance with deterministic outputs.
//!
//! # Features
//!
//! 1. **Variant Fingerprinting**: Deterministic hashing of activity sequences
//! 2. **Frequency Analysis**: Pareto-based variant ranking with coverage analysis
//! 3. **Variant Filtering**: Multiple strategies (frequency, coverage, pattern matching)
//! 4. **Similarity Analysis**: Edit distance and common subsequence computation
//! 5. **Variant Metrics**: Complexity, performance, and risk scoring

use crate::log::{EventLog, Trace};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// A variant represents a unique sequence of activities in a trace
#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Variant {
    pub activities: Vec<String>,
}

impl Variant {
    /// Create a new variant from an activity sequence
    pub fn new(activities: Vec<String>) -> Self {
        Self { activities }
    }

    /// Get variant length (number of activities)
    pub fn len(&self) -> usize {
        self.activities.len()
    }

    /// Check if variant is empty
    pub fn is_empty(&self) -> bool {
        self.activities.is_empty()
    }

    /// Get a string representation (comma-separated)
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        self.activities.join(",")
    }
}

/// Deterministic fingerprint for a variant (32-bit hash)
/// Same sequence always produces same fingerprint
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct VariantFingerprint(pub u32);

impl VariantFingerprint {
    /// Create fingerprint from variant
    /// O(n) complexity where n is number of activities
    pub fn compute(variant: &Variant) -> Self {
        let mut hasher = DefaultHasher::new();
        for activity in &variant.activities {
            activity.hash(&mut hasher);
        }
        let hash = hasher.finish();
        VariantFingerprint((hash as u32) ^ ((hash >> 32) as u32))
    }

    /// Create fingerprint from activity sequence
    pub fn from_activities(activities: &[String]) -> Self {
        let variant = Variant::new(activities.to_vec());
        Self::compute(&variant)
    }

    /// Get hash as hex string
    pub fn to_hex(&self) -> String {
        format!("{:08x}", self.0)
    }
}

/// Variant information with frequency and metrics
#[derive(Debug, Clone)]
pub struct VariantInfo {
    pub variant: Variant,
    pub fingerprint: VariantFingerprint,
    pub frequency: usize,
    pub trace_ids: Vec<String>,
}

impl VariantInfo {
    /// Create variant info with frequency tracking
    pub fn new(variant: Variant, frequency: usize, trace_ids: Vec<String>) -> Self {
        let fingerprint = VariantFingerprint::compute(&variant);
        Self {
            variant,
            fingerprint,
            frequency,
            trace_ids,
        }
    }

    /// Get coverage percentage (of total traces provided)
    pub fn coverage_percentage(&self, total_traces: usize) -> f64 {
        if total_traces == 0 {
            0.0
        } else {
            (self.frequency as f64 / total_traces as f64) * 100.0
        }
    }
}

/// Results of variant fingerprinting and frequency analysis
#[derive(Debug, Clone)]
pub struct VariantAnalysis {
    pub variants: Vec<VariantInfo>,
    pub total_traces: usize,
    pub unique_variants: usize,
}

impl VariantAnalysis {
    /// Discover and analyze variants from event log
    /// O(n) complexity where n is number of traces
    pub fn discover(log: &EventLog) -> Self {
        let mut variant_map: HashMap<Variant, (usize, Vec<String>)> = HashMap::new();

        // Extract variants and count frequencies
        for trace in &log.traces {
            let activities: Vec<String> = trace.events.iter().map(|e| e.activity.clone()).collect();
            let variant = Variant::new(activities);

            variant_map
                .entry(variant)
                .and_modify(|(count, ids)| {
                    *count += 1;
                    ids.push(trace.id.clone());
                })
                .or_insert((1, vec![trace.id.clone()]));
        }

        // Build variant info with Pareto ordering
        let mut variants: Vec<VariantInfo> = variant_map
            .into_iter()
            .map(|(variant, (frequency, trace_ids))| {
                VariantInfo::new(variant, frequency, trace_ids)
            })
            .collect();

        // Sort by frequency (descending) for Pareto analysis
        variants.sort_by(|a, b| b.frequency.cmp(&a.frequency));

        let unique_variants = variants.len();
        let total_traces = log.len();

        Self {
            variants,
            total_traces,
            unique_variants,
        }
    }

    /// Get top K variants by frequency
    /// O(k) complexity
    pub fn top_k(&self, k: usize) -> Vec<&VariantInfo> {
        self.variants.iter().take(k).collect()
    }

    /// Get cumulative coverage of top K variants
    /// O(k) complexity
    pub fn coverage_top_k(&self, k: usize) -> f64 {
        let total: usize = self.variants.iter().take(k).map(|v| v.frequency).sum();
        (total as f64 / self.total_traces as f64) * 100.0
    }

    /// Find Pareto frontier (80% coverage with minimum variants)
    /// Includes the variant that pushes cumulative coverage past the 80% threshold.
    /// Typically O(log n) iterations, each O(1)
    pub fn pareto_frontier(&self) -> Vec<&VariantInfo> {
        let mut cumulative = 0usize;
        let target_coverage = self.total_traces as f64 * 0.8;
        let mut result = Vec::new();

        for v in &self.variants {
            result.push(v);
            cumulative += v.frequency;
            if cumulative as f64 >= target_coverage {
                break;
            }
        }

        result
    }
}

/// Strategy for filtering variants
#[derive(Debug, Clone)]
pub enum FilterStrategy {
    /// Keep variants with frequency >= threshold
    MinimumFrequency { threshold: usize },
    /// Keep variants covering X% of traces
    CoveragePercentage { target: f64 },
    /// Keep top K variants by frequency
    TopK { k: usize },
    /// Keep variants matching regex pattern
    PatternMatch { pattern: String },
    /// Keep variants with all activities matching a set
    ActivityWhitelist { activities: Vec<String> },
}

/// Variant filtering with multiple strategies
#[derive(Debug, Clone)]
pub struct VariantFilter {
    pub strategy: FilterStrategy,
}

impl VariantFilter {
    /// Create new filter with strategy
    pub fn new(strategy: FilterStrategy) -> Self {
        Self { strategy }
    }

    /// Apply filter to variant analysis
    /// O(n) complexity for most strategies
    pub fn apply(&self, analysis: &VariantAnalysis) -> VariantAnalysis {
        let filtered_variants = match &self.strategy {
            FilterStrategy::MinimumFrequency { threshold } => analysis
                .variants
                .iter()
                .filter(|v| v.frequency >= *threshold)
                .cloned()
                .collect::<Vec<_>>(),
            FilterStrategy::CoveragePercentage { target } => {
                let target_traces = (analysis.total_traces as f64 * target / 100.0) as usize;
                let mut cumulative = 0usize;
                let mut result = Vec::new();
                for v in &analysis.variants {
                    result.push(v.clone());
                    cumulative += v.frequency;
                    if cumulative >= target_traces {
                        break;
                    }
                }
                result
            }
            FilterStrategy::TopK { k } => analysis.variants.iter().take(*k).cloned().collect(),
            FilterStrategy::PatternMatch { pattern } => analysis
                .variants
                .iter()
                .filter(|v| variant_matches_pattern(&v.variant, pattern))
                .cloned()
                .collect(),
            FilterStrategy::ActivityWhitelist { activities } => {
                let whitelist: std::collections::HashSet<_> = activities.iter().cloned().collect();
                analysis
                    .variants
                    .iter()
                    .filter(|v| v.variant.activities.iter().all(|a| whitelist.contains(a)))
                    .cloned()
                    .collect()
            }
        };

        let unique_variants = filtered_variants.len();
        let total_traces: usize = filtered_variants.iter().map(|v| v.frequency).sum();

        VariantAnalysis {
            variants: filtered_variants,
            total_traces,
            unique_variants,
        }
    }

    /// Apply to event log and return filtered log
    pub fn apply_to_log(&self, log: &EventLog, analysis: &VariantAnalysis) -> EventLog {
        let filtered_analysis = self.apply(analysis);
        let variant_set: std::collections::HashSet<_> = filtered_analysis
            .variants
            .iter()
            .map(|v| v.variant.clone())
            .collect();

        let mut filtered_log = log.clone();
        filtered_log.traces.retain(|trace| {
            let activities: Vec<String> = trace.events.iter().map(|e| e.activity.clone()).collect();
            variant_set.contains(&Variant::new(activities))
        });

        filtered_log
    }
}

/// Check if variant matches pattern (simple wildcard support)
fn variant_matches_pattern(variant: &Variant, pattern: &str) -> bool {
    let pattern_str = variant.to_string();
    if pattern.contains('*') {
        // Simple prefix/suffix matching
        if let Some(idx) = pattern.find('*') {
            let (prefix, suffix) = pattern.split_at(idx);
            let suffix = &suffix[1..];
            pattern_str.starts_with(prefix) && pattern_str.ends_with(suffix)
        } else {
            pattern_str == pattern
        }
    } else {
        pattern_str == pattern
    }
}

/// Similarity metrics between variants
#[derive(Debug, Clone)]
pub struct VariantSimilarity {
    /// Levenshtein distance (edit distance)
    pub edit_distance: usize,
    /// Longest common subsequence length
    pub lcs_length: usize,
    /// Similarity score 0.0-1.0
    pub similarity_score: f64,
}

impl VariantSimilarity {
    /// Compute similarity between two variants
    /// O(n*m) where n, m are variant lengths
    pub fn compute(v1: &Variant, v2: &Variant) -> Self {
        let edit_distance = edit_distance(&v1.activities, &v2.activities);
        let lcs_length = longest_common_subsequence(&v1.activities, &v2.activities);
        let max_len = v1.len().max(v2.len());
        let similarity_score = if max_len == 0 {
            1.0
        } else {
            (lcs_length as f64) / (max_len as f64)
        };

        Self {
            edit_distance,
            lcs_length,
            similarity_score,
        }
    }
}

/// Compute Levenshtein distance between two sequences
/// O(n*m) complexity
#[allow(clippy::needless_range_loop)]
fn edit_distance<T: PartialEq>(s1: &[T], s2: &[T]) -> usize {
    let len1 = s1.len();
    let len2 = s2.len();

    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    let mut dp = vec![vec![0; len2 + 1]; len1 + 1];

    for i in 0..=len1 {
        dp[i][0] = i;
    }
    for j in 0..=len2 {
        dp[0][j] = j;
    }

    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if s1[i - 1] == s2[j - 1] { 0 } else { 1 };
            dp[i][j] = std::cmp::min(
                std::cmp::min(dp[i - 1][j] + 1, dp[i][j - 1] + 1),
                dp[i - 1][j - 1] + cost,
            );
        }
    }

    dp[len1][len2]
}

/// Compute longest common subsequence length
/// O(n*m) complexity
fn longest_common_subsequence<T: PartialEq>(s1: &[T], s2: &[T]) -> usize {
    let len1 = s1.len();
    let len2 = s2.len();

    let mut dp = vec![vec![0; len2 + 1]; len1 + 1];

    for i in 1..=len1 {
        for j in 1..=len2 {
            if s1[i - 1] == s2[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = std::cmp::max(dp[i - 1][j], dp[i][j - 1]);
            }
        }
    }

    dp[len1][len2]
}

/// Metrics for a variant group
#[derive(Debug, Clone)]
pub struct VariantMetrics {
    /// Variant complexity: length * branching_factor
    pub complexity: f64,
    /// Average duration in milliseconds (if timing data available)
    pub avg_duration_ms: f64,
    /// Error rate: failed traces / total traces
    pub error_rate: f64,
    /// Risk score (0.0-1.0) combining complexity and error rate
    pub risk_score: f64,
}

impl VariantMetrics {
    /// Compute metrics for a variant from log traces
    pub fn compute(variant: &VariantInfo, traces: &[Trace]) -> Self {
        let complexity = variant.variant.len() as f64;
        let avg_duration_ms = compute_average_duration(traces);
        let error_rate = compute_error_rate(traces);
        // Normalize complexity: 8 activities = 1.0 (fully complex).
        // Typical SAP/BPIC processes have 4-6 steps; 8+ is unusual and risky.
        let risk_score = (complexity / 8.0).min(1.0) * 0.5 + error_rate * 0.5;

        Self {
            complexity,
            avg_duration_ms,
            error_rate,
            risk_score,
        }
    }
}

/// Compute average duration from traces in milliseconds
fn compute_average_duration(traces: &[Trace]) -> f64 {
    if traces.is_empty() {
        return 0.0;
    }

    let mut total_duration: i64 = 0;
    let mut count = 0;

    for trace in traces {
        if trace.events.len() >= 2 {
            if let (Some(first), Some(last)) = (trace.events.first(), trace.events.last()) {
                let duration = last.timestamp.signed_duration_since(first.timestamp);
                total_duration += duration.num_milliseconds();
                count += 1;
            }
        }
    }

    if count == 0 {
        0.0
    } else {
        total_duration as f64 / count as f64
    }
}

/// Compute error rate from traces (based on activity name containing "error" or "fail")
fn compute_error_rate(traces: &[Trace]) -> f64 {
    if traces.is_empty() {
        return 0.0;
    }

    let error_count = traces
        .iter()
        .filter(|trace| {
            trace.events.iter().any(|e| {
                let lower = e.activity.to_lowercase();
                lower.contains("error") || lower.contains("fail")
            })
        })
        .count();

    error_count as f64 / traces.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_trace(id: &str, activities: &[&str]) -> Trace {
        let mut trace = Trace::new(id);
        let mut timestamp = Utc::now();
        for activity in activities {
            let event = crate::log::Event::new(*activity, timestamp);
            trace.add_event(event);
            timestamp = timestamp + chrono::Duration::seconds(1);
        }
        trace
    }

    #[test]
    fn test_variant_creation() {
        let activities = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        let variant = Variant::new(activities.clone());

        assert_eq!(variant.len(), 3);
        assert!(!variant.is_empty());
        assert_eq!(variant.to_string(), "A,B,C");
    }

    #[test]
    fn test_variant_fingerprint_deterministic() {
        let activities = vec!["A".to_string(), "B".to_string()];
        let v1 = Variant::new(activities.clone());
        let v2 = Variant::new(activities.clone());

        let fp1 = VariantFingerprint::compute(&v1);
        let fp2 = VariantFingerprint::compute(&v2);

        assert_eq!(fp1, fp2);
    }

    #[test]
    fn test_variant_fingerprint_different() {
        let v1 = Variant::new(vec!["A".to_string(), "B".to_string()]);
        let v2 = Variant::new(vec!["A".to_string(), "C".to_string()]);

        let fp1 = VariantFingerprint::compute(&v1);
        let fp2 = VariantFingerprint::compute(&v2);

        assert_ne!(fp1, fp2);
    }

    #[test]
    fn test_variant_analysis_simple() {
        let mut log = EventLog::new();
        log.add_trace(create_trace("1", &["A", "B", "C"]));
        log.add_trace(create_trace("2", &["A", "B", "C"]));
        log.add_trace(create_trace("3", &["A", "B", "D"]));

        let analysis = VariantAnalysis::discover(&log);

        assert_eq!(analysis.total_traces, 3);
        assert_eq!(analysis.unique_variants, 2);
        assert_eq!(analysis.variants[0].frequency, 2); // Top variant
    }

    #[test]
    fn test_variant_analysis_empty_log() {
        let log = EventLog::new();
        let analysis = VariantAnalysis::discover(&log);

        assert_eq!(analysis.total_traces, 0);
        assert_eq!(analysis.unique_variants, 0);
    }

    #[test]
    fn test_variant_analysis_single_variant() {
        let mut log = EventLog::new();
        for i in 0..5 {
            log.add_trace(create_trace(&i.to_string(), &["A", "B", "C"]));
        }

        let analysis = VariantAnalysis::discover(&log);

        assert_eq!(analysis.total_traces, 5);
        assert_eq!(analysis.unique_variants, 1);
        assert_eq!(analysis.variants[0].frequency, 5);
    }

    #[test]
    fn test_top_k_variants() {
        let mut log = EventLog::new();
        log.add_trace(create_trace("1", &["A", "B"]));
        log.add_trace(create_trace("2", &["A", "B"]));
        log.add_trace(create_trace("3", &["A", "B", "C"]));
        log.add_trace(create_trace("4", &["A", "B", "C"]));
        log.add_trace(create_trace("5", &["A", "B", "C", "D"]));

        let analysis = VariantAnalysis::discover(&log);
        let top2 = analysis.top_k(2);

        assert_eq!(top2.len(), 2);
        assert_eq!(top2[0].frequency, 2); // A,B,C: 2 occurrences
        assert_eq!(top2[1].frequency, 2); // A,B: 2 occurrences
    }

    #[test]
    fn test_coverage_top_k() {
        let mut log = EventLog::new();
        for _i in 0..80 {
            log.add_trace(create_trace("a", &["A", "B"]));
        }
        for _i in 0..15 {
            log.add_trace(create_trace("b", &["A", "B", "C"]));
        }
        for _i in 0..5 {
            log.add_trace(create_trace("c", &["A", "B", "C", "D"]));
        }

        let analysis = VariantAnalysis::discover(&log);
        let coverage = analysis.coverage_top_k(1);

        assert!(coverage >= 79.9 && coverage <= 80.1);
    }

    #[test]
    fn test_pareto_frontier() {
        let mut log = EventLog::new();
        for _i in 0..800 {
            log.add_trace(create_trace("a", &["A"]));
        }
        for _i in 0..100 {
            log.add_trace(create_trace("b", &["B"]));
        }
        for _i in 0..50 {
            log.add_trace(create_trace("c", &["C"]));
        }
        for _i in 0..50 {
            log.add_trace(create_trace("d", &["D"]));
        }

        let analysis = VariantAnalysis::discover(&log);
        let frontier = analysis.pareto_frontier();

        let coverage: usize = frontier.iter().map(|v| v.frequency).sum();
        let coverage_pct = coverage as f64 / analysis.total_traces as f64 * 100.0;
        assert!(coverage_pct >= 80.0 && coverage_pct <= 85.0);
    }

    #[test]
    fn test_filter_minimum_frequency() {
        let mut log = EventLog::new();
        log.add_trace(create_trace("1", &["A", "B"]));
        log.add_trace(create_trace("2", &["A", "B"]));
        log.add_trace(create_trace("3", &["A", "B"]));
        log.add_trace(create_trace("4", &["A", "B", "C"]));

        let analysis = VariantAnalysis::discover(&log);
        let filter = VariantFilter::new(FilterStrategy::MinimumFrequency { threshold: 2 });
        let filtered = filter.apply(&analysis);

        // Only variant ["A", "B"] with frequency 3 >= threshold 2 passes filter
        // Variant ["A", "B", "C"] with frequency 1 < threshold 2 is removed
        assert_eq!(filtered.unique_variants, 1);
    }

    #[test]
    fn test_filter_top_k() {
        let mut log = EventLog::new();
        for _i in 0..10 {
            log.add_trace(create_trace("1", &["A"]));
        }
        for _i in 0..5 {
            log.add_trace(create_trace("2", &["B"]));
        }
        for _i in 0..3 {
            log.add_trace(create_trace("3", &["C"]));
        }

        let analysis = VariantAnalysis::discover(&log);
        let filter = VariantFilter::new(FilterStrategy::TopK { k: 2 });
        let filtered = filter.apply(&analysis);

        assert_eq!(filtered.unique_variants, 2);
        assert_eq!(filtered.variants[0].frequency, 10);
        assert_eq!(filtered.variants[1].frequency, 5);
    }

    #[test]
    fn test_filter_coverage_percentage() {
        let mut log = EventLog::new();
        for _i in 0..70 {
            log.add_trace(create_trace("1", &["A"]));
        }
        for _i in 0..20 {
            log.add_trace(create_trace("2", &["B"]));
        }
        for _i in 0..10 {
            log.add_trace(create_trace("3", &["C"]));
        }

        let analysis = VariantAnalysis::discover(&log);
        let filter = VariantFilter::new(FilterStrategy::CoveragePercentage { target: 90.0 });
        let filtered = filter.apply(&analysis);

        let coverage: usize = filtered.variants.iter().map(|v| v.frequency).sum();
        assert!(coverage >= 90);
    }

    #[test]
    fn test_filter_activity_whitelist() {
        let mut log = EventLog::new();
        log.add_trace(create_trace("1", &["A", "B"]));
        log.add_trace(create_trace("2", &["A", "B", "C"]));
        log.add_trace(create_trace("3", &["A", "D"]));

        let analysis = VariantAnalysis::discover(&log);
        let filter = VariantFilter::new(FilterStrategy::ActivityWhitelist {
            activities: vec!["A".to_string(), "B".to_string()],
        });
        let filtered = filter.apply(&analysis);

        assert_eq!(filtered.unique_variants, 1); // Only A,B
    }

    #[test]
    fn test_variant_similarity_identical() {
        let v1 = Variant::new(vec!["A".to_string(), "B".to_string(), "C".to_string()]);
        let v2 = Variant::new(vec!["A".to_string(), "B".to_string(), "C".to_string()]);

        let sim = VariantSimilarity::compute(&v1, &v2);

        assert_eq!(sim.edit_distance, 0);
        assert_eq!(sim.lcs_length, 3);
        assert_eq!(sim.similarity_score, 1.0);
    }

    #[test]
    fn test_variant_similarity_completely_different() {
        let v1 = Variant::new(vec!["A".to_string(), "B".to_string()]);
        let v2 = Variant::new(vec!["C".to_string(), "D".to_string()]);

        let sim = VariantSimilarity::compute(&v1, &v2);

        assert_eq!(sim.edit_distance, 2);
        assert_eq!(sim.lcs_length, 0);
        assert_eq!(sim.similarity_score, 0.0);
    }

    #[test]
    fn test_variant_similarity_partial() {
        let v1 = Variant::new(vec!["A".to_string(), "B".to_string(), "C".to_string()]);
        let v2 = Variant::new(vec!["A".to_string(), "X".to_string(), "C".to_string()]);

        let sim = VariantSimilarity::compute(&v1, &v2);

        assert!(sim.similarity_score > 0.5);
        assert!(sim.similarity_score < 1.0);
    }

    #[test]
    fn test_variant_metrics_computation() {
        let trace1 = create_trace("1", &["A", "B"]);
        let trace2 = create_trace("2", &["A", "B"]);

        let variant = VariantInfo::new(
            Variant::new(vec!["A".to_string(), "B".to_string()]),
            2,
            vec!["1".to_string(), "2".to_string()],
        );

        let metrics = VariantMetrics::compute(&variant, &[trace1, trace2]);

        assert_eq!(metrics.complexity, 2.0);
        assert!(metrics.avg_duration_ms > 0.0);
        assert!(metrics.risk_score >= 0.0 && metrics.risk_score <= 1.0);
    }

    #[test]
    fn test_variant_info_coverage_percentage() {
        let variant = VariantInfo::new(
            Variant::new(vec!["A".to_string(), "B".to_string()]),
            25,
            vec!["1".to_string()],
        );

        let coverage = variant.coverage_percentage(100);
        assert_eq!(coverage, 25.0);
    }

    #[test]
    fn test_edit_distance_basic() {
        let s1 = vec!["A", "B", "C"];
        let s2 = vec!["A", "B", "C"];
        assert_eq!(edit_distance(&s1, &s2), 0);

        let s1 = vec!["A", "B"];
        let s2 = vec!["A", "B", "C"];
        assert_eq!(edit_distance(&s1, &s2), 1);
    }

    #[test]
    fn test_longest_common_subsequence() {
        let s1 = vec!["A", "B", "C"];
        let s2 = vec!["A", "B", "C"];
        assert_eq!(longest_common_subsequence(&s1, &s2), 3);

        let s1 = vec!["A", "B", "C"];
        let s2 = vec!["A", "X", "C"];
        assert_eq!(longest_common_subsequence(&s1, &s2), 2);
    }
}
