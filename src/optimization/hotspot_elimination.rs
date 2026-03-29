//! Hotspot Elimination Module
//!
//! Implements early-termination BFS, memoized calculations, and single-pass aggregation
//! for remaining computational hotspots.

use crate::discovery::variants::{Variant, VariantInfo};
use crate::models::PetriNet;
use std::collections::{HashMap, HashSet, VecDeque};

/// Optimization 4: BFS with early termination
/// Reachability checking that stops as soon as target is found
pub struct OptimizedReachabilityChecker;

impl OptimizedReachabilityChecker {
    /// Check if target marking is reachable from initial marking
    /// Stops immediately when target found (vs. exhaustive exploration)
    pub fn is_reachable(
        net: &PetriNet,
        initial: &HashMap<String, usize>,
        target: &HashMap<String, usize>,
        max_depth: usize,
    ) -> bool {
        let mut queue = VecDeque::new();
        let mut visited: Vec<HashMap<String, usize>> = Vec::new();

        queue.push_back((initial.clone(), 0));
        visited.push(initial.clone());

        while let Some((current, depth)) = queue.pop_front() {
            // Early termination: found target
            if current == *target {
                return true;
            }

            // Depth limit prevents explosion
            if depth >= max_depth {
                continue;
            }

            // Try firing each transition
            for transition in &net.transitions {
                let mut new_marking = current.clone();
                if net.fire_transition(&transition.id, &mut new_marking)
                    && !visited.iter().any(|m| m == &new_marking)
                {
                    visited.push(new_marking.clone());
                    queue.push_back((new_marking, depth + 1));
                }
            }
        }

        false
    }

    /// Count reachable states with early termination on threshold
    pub fn count_reachable_states_limited(
        net: &PetriNet,
        initial: &HashMap<String, usize>,
        limit: usize,
    ) -> usize {
        let mut visited: Vec<HashMap<String, usize>> = Vec::new();
        let mut queue = VecDeque::new();

        queue.push_back(initial.clone());
        visited.push(initial.clone());

        while let Some(current) = queue.pop_front() {
            // Early termination: hit limit
            if visited.len() >= limit {
                return visited.len();
            }

            // Try firing each transition
            for transition in &net.transitions {
                let mut new_marking = current.clone();
                if net.fire_transition(&transition.id, &mut new_marking)
                    && !visited.iter().any(|m| m == &new_marking)
                {
                    visited.push(new_marking.clone());
                    queue.push_back(new_marking);
                }
            }
        }

        visited.len()
    }
}

/// Memoization cache for expensive calculations
#[derive(Debug, Clone)]
pub struct CalculationMemoizer {
    reachability_cache: HashMap<(String, String), bool>,
    variant_metrics_cache: HashMap<String, VariantMetrics>,
}

/// Cached variant metrics to avoid recomputation
#[derive(Debug, Clone, PartialEq)]
pub struct VariantMetrics {
    pub length: usize,
    pub complexity_score: f64,
    pub rework_ratio: f64,
}

impl CalculationMemoizer {
    pub fn new() -> Self {
        Self {
            reachability_cache: HashMap::new(),
            variant_metrics_cache: HashMap::new(),
        }
    }

    /// Memoized reachability check
    pub fn is_reachable_memoized(
        &mut self,
        from: &str,
        to: &str,
        net: &PetriNet,
        initial: &HashMap<String, usize>,
    ) -> bool {
        let key = (from.to_string(), to.to_string());

        if let Some(&cached) = self.reachability_cache.get(&key) {
            return cached;
        }

        // Compute and cache
        let result = OptimizedReachabilityChecker::is_reachable(
            net,
            initial,
            &{
                let mut m = HashMap::new();
                m.insert(to.to_string(), 1);
                m
            },
            10,
        );

        self.reachability_cache.insert(key, result);
        result
    }

    /// Memoized variant metrics computation
    pub fn get_variant_metrics(&mut self, variant: &Variant) -> VariantMetrics {
        let key = variant.to_string();

        if let Some(cached) = self.variant_metrics_cache.get(&key) {
            return cached.clone();
        }

        let metrics = VariantMetrics {
            length: variant.len(),
            complexity_score: Self::compute_complexity(variant),
            rework_ratio: Self::compute_rework_ratio(variant),
        };

        self.variant_metrics_cache.insert(key, metrics.clone());
        metrics
    }

    /// Complexity: penalize long variants and repeated activities
    fn compute_complexity(variant: &Variant) -> f64 {
        let length = variant.len() as f64;
        let unique = variant.activities.iter().collect::<HashSet<_>>().len() as f64;

        // Complexity = length + (repetition factor)
        let repetition = length - unique;
        (length + repetition) / 10.0
    }

    /// Rework: % of activities that appear multiple times
    fn compute_rework_ratio(variant: &Variant) -> f64 {
        if variant.is_empty() {
            return 0.0;
        }

        let mut activity_counts: HashMap<&String, usize> = HashMap::new();
        for activity in &variant.activities {
            *activity_counts.entry(activity).or_insert(0) += 1;
        }

        let rework_count = activity_counts.values().filter(|&&c| c > 1).count();
        rework_count as f64 / activity_counts.len() as f64
    }

    pub fn clear(&mut self) {
        self.reachability_cache.clear();
        self.variant_metrics_cache.clear();
    }
}

impl Default for CalculationMemoizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Optimization 5: Variant frequency analysis with memoization
pub struct OptimizedVariantAnalyzer;

impl OptimizedVariantAnalyzer {
    /// Compute variant frequencies with memoized metrics
    pub fn analyze_with_metrics(
        variants_info: Vec<VariantInfo>,
    ) -> Vec<(VariantInfo, VariantMetrics)> {
        let mut memoizer = CalculationMemoizer::new();

        variants_info
            .into_iter()
            .map(|info| {
                let metrics = memoizer.get_variant_metrics(&info.variant);
                (info, metrics)
            })
            .collect()
    }
}

/// Single-scan frequency aggregation with early termination
pub struct SingleScanAggregator;

impl SingleScanAggregator {
    /// Aggregate variant frequencies in single pass, skip below threshold
    pub fn aggregate_threshold(
        variants: Vec<Variant>,
        min_frequency: usize,
    ) -> HashMap<Variant, usize> {
        let mut freq_map = HashMap::new();

        // Single pass: insert and count
        for variant in variants {
            *freq_map.entry(variant).or_insert(0) += 1;
        }

        // Filter in-place: keep only above threshold
        freq_map.retain(|_, count| *count >= min_frequency);

        freq_map
    }

    /// Get top-k frequent variants in single pass with early termination
    pub fn get_top_k(variants: Vec<Variant>, k: usize) -> Vec<(Variant, usize)> {
        let mut freq_map = HashMap::new();

        // Single pass: count
        for variant in variants {
            *freq_map.entry(variant).or_insert(0) += 1;
        }

        // Convert and partial sort: O(n log k) instead of O(n log n)
        let mut result: Vec<_> = freq_map.into_iter().collect();

        // Partial sort: only need top k
        if result.len() > k {
            result.select_nth_unstable_by(k - 1, |a, b| b.1.cmp(&a.1));
            result.truncate(k);
        } else {
            result.sort_by(|a, b| b.1.cmp(&a.1));
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Arc, Place, Transition};

    fn create_simple_net() -> PetriNet {
        let mut net = PetriNet::new();

        let p1 = Place::new("p1").with_initial_marking(1);
        let p2 = Place::new("p2");
        let t1 = Transition::new("t1");

        let p1_id = p1.id.clone();
        let p2_id = p2.id.clone();
        let t1_id = t1.id.clone();

        net.add_place(p1);
        net.add_place(p2);
        net.add_transition(t1);

        net.add_arc(Arc::new(&p1_id, &t1_id));
        net.add_arc(Arc::new(&t1_id, &p2_id));

        net
    }

    #[test]
    fn test_reachability_checker() {
        let net = create_simple_net();
        let mut initial = HashMap::new();
        initial.insert(net.places[0].id.clone(), 1);

        let mut target = HashMap::new();
        // After firing, zero-token places are pruned from the marking,
        // so the target should only contain places with tokens > 0.
        target.insert(net.places[1].id.clone(), 1);

        let reachable = OptimizedReachabilityChecker::is_reachable(&net, &initial, &target, 10);
        assert!(reachable);
    }

    #[test]
    fn test_reachability_checker_limited() {
        let net = create_simple_net();
        let initial = HashMap::new();

        let count =
            OptimizedReachabilityChecker::count_reachable_states_limited(&net, &initial, 100);
        assert!(count > 0);
    }

    #[test]
    fn test_memoizer_reachability() {
        let net = create_simple_net();
        let mut memoizer = CalculationMemoizer::new();
        let mut initial = HashMap::new();
        initial.insert(net.places[0].id.clone(), 1);

        // First call computes
        let result1 = memoizer.is_reachable_memoized("p1", "p2", &net, &initial);
        // Second call should be cached
        let result2 = memoizer.is_reachable_memoized("p1", "p2", &net, &initial);

        assert_eq!(result1, result2);
        assert_eq!(memoizer.reachability_cache.len(), 1);
    }

    #[test]
    fn test_variant_metrics_memoization() {
        let variant = Variant::new(vec!["a".to_string(), "b".to_string(), "a".to_string()]);
        let mut memoizer = CalculationMemoizer::new();

        let metrics1 = memoizer.get_variant_metrics(&variant);
        let metrics2 = memoizer.get_variant_metrics(&variant);

        assert_eq!(metrics1, metrics2);
        assert_eq!(metrics1.length, 3);
        assert!(metrics1.rework_ratio > 0.0);
    }

    #[test]
    fn test_complexity_computation() {
        let variant1 = Variant::new(vec!["a".to_string(), "b".to_string()]);
        let variant2 = Variant::new(vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "a".to_string(),
            "b".to_string(),
        ]);

        let complexity1 = CalculationMemoizer::compute_complexity(&variant1);
        let complexity2 = CalculationMemoizer::compute_complexity(&variant2);

        // More repetitions = higher complexity
        assert!(complexity2 > complexity1);
    }

    #[test]
    fn test_single_scan_aggregator() {
        let variants = vec![
            Variant::new(vec!["a".to_string(), "b".to_string()]),
            Variant::new(vec!["a".to_string(), "b".to_string()]),
            Variant::new(vec!["c".to_string()]),
        ];

        let result = SingleScanAggregator::aggregate_threshold(variants, 2);
        assert_eq!(result.len(), 1); // Only "a,b" meets threshold
    }

    #[test]
    fn test_top_k_variants() {
        let variants = vec![
            Variant::new(vec!["a".to_string()]),
            Variant::new(vec!["a".to_string()]),
            Variant::new(vec!["a".to_string()]),
            Variant::new(vec!["b".to_string()]),
            Variant::new(vec!["b".to_string()]),
            Variant::new(vec!["c".to_string()]),
        ];

        let top_2 = SingleScanAggregator::get_top_k(variants, 2);
        assert_eq!(top_2.len(), 2);
        assert_eq!(top_2[0].1, 3); // "a" appears 3 times
        assert_eq!(top_2[1].1, 2); // "b" appears 2 times
    }
}
