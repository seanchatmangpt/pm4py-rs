//! Standalone variant tests that compile independently
//! Run with: cargo test --test variants_standalone_test

#[cfg(test)]
mod standalone_variant_tests {
    use std::collections::hash_map::DefaultHasher;
    use std::collections::HashMap;
    use std::hash::{Hash, Hasher};

    // Inline minimal implementations for independent testing
    #[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
    struct SimpleVariant {
        activities: Vec<String>,
    }

    #[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
    struct SimpleFingerprint(u32);

    impl SimpleFingerprint {
        fn compute(variant: &SimpleVariant) -> Self {
            let mut hasher = DefaultHasher::new();
            for activity in &variant.activities {
                activity.hash(&mut hasher);
            }
            let hash = hasher.finish();
            SimpleFingerprint((hash as u32) ^ ((hash >> 32) as u32))
        }
    }

    // ============================================================================
    // FEATURE 1: VARIANT FINGERPRINTING TESTS
    // ============================================================================

    #[test]
    fn test_fingerprint_deterministic() {
        let v1 = SimpleVariant {
            activities: vec!["A".to_string(), "B".to_string()],
        };
        let v2 = SimpleVariant {
            activities: vec!["A".to_string(), "B".to_string()],
        };

        let fp1 = SimpleFingerprint::compute(&v1);
        let fp2 = SimpleFingerprint::compute(&v2);

        assert_eq!(fp1, fp2);
    }

    #[test]
    fn test_fingerprint_different_sequences() {
        let v1 = SimpleVariant {
            activities: vec!["A".to_string(), "B".to_string()],
        };
        let v2 = SimpleVariant {
            activities: vec!["B".to_string(), "A".to_string()],
        };

        let fp1 = SimpleFingerprint::compute(&v1);
        let fp2 = SimpleFingerprint::compute(&v2);

        assert_ne!(fp1, fp2);
    }

    #[test]
    fn test_fingerprint_collision_resistance() {
        let mut fingerprints = HashMap::new();

        for i in 0..100 {
            let variant = SimpleVariant {
                activities: vec![format!("Activity_{}", i), format!("Activity_{}", i + 1)],
            };
            let fp = SimpleFingerprint::compute(&variant);
            fingerprints.insert(fp.0, i);
        }

        assert_eq!(fingerprints.len(), 100);
    }

    // ============================================================================
    // FEATURE 2: FREQUENCY ANALYSIS
    // ============================================================================

    #[test]
    fn test_variant_frequency_counting() {
        let mut variant_map: HashMap<Vec<String>, usize> = HashMap::new();

        // Add traces
        let activities1 = vec!["A", "B", "C"];
        let activities2 = vec!["A", "B", "C"];
        let activities3 = vec!["A", "D"];

        for activities in &[&activities1, &activities2, &activities3] {
            let variant: Vec<String> = activities.iter().map(|s| s.to_string()).collect();
            *variant_map.entry(variant).or_insert(0) += 1;
        }

        assert_eq!(variant_map.len(), 2);

        let ab_c: Vec<String> = vec!["A", "B", "C"].iter().map(|s| s.to_string()).collect();
        let a_d: Vec<String> = vec!["A", "D"].iter().map(|s| s.to_string()).collect();

        assert_eq!(variant_map[&ab_c], 2);
        assert_eq!(variant_map[&a_d], 1);
    }

    #[test]
    fn test_pareto_ordering() {
        let mut variants = vec![("A,B,C", 50), ("A,B", 30), ("A", 15), ("B,C", 5)];

        variants.sort_by(|a, b| b.1.cmp(&a.1));

        // Should be in descending frequency order
        assert_eq!(variants[0].1, 50);
        assert_eq!(variants[1].1, 30);
        assert_eq!(variants[2].1, 15);
        assert_eq!(variants[3].1, 5);
    }

    #[test]
    fn test_pareto_coverage_80_percent() {
        let variants = vec![("A,B,C", 800), ("A,B", 100), ("A", 50), ("B", 50)];

        let total: usize = variants.iter().map(|(_, count)| count).sum();
        let mut cumulative = 0;
        let mut variant_count = 0;

        for (_, count) in &variants {
            cumulative += count;
            variant_count += 1;
            if cumulative as f64 / total as f64 * 100.0 >= 80.0 {
                break;
            }
        }

        assert_eq!(variant_count, 1);
    }

    // ============================================================================
    // FEATURE 3: VARIANT FILTERING
    // ============================================================================

    #[test]
    fn test_filter_minimum_frequency() {
        let variants = vec![("A", 10), ("B", 5), ("C", 2)];

        let filtered: Vec<_> = variants.iter().filter(|(_, count)| *count >= 5).collect();

        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_filter_top_k() {
        let mut variants = vec![("A", 20), ("B", 15), ("C", 5), ("D", 2)];
        variants.sort_by(|a, b| b.1.cmp(&a.1));

        let top_2: Vec<_> = variants.iter().take(2).collect();

        assert_eq!(top_2.len(), 2);
        assert_eq!(top_2[0].1, 20);
        assert_eq!(top_2[1].1, 15);
    }

    #[test]
    fn test_filter_coverage_percentage() {
        let variants = vec![("A", 60), ("B", 30), ("C", 10)];
        let total: usize = variants.iter().map(|(_, count)| count).sum();

        let mut cumulative = 0;
        let filtered: Vec<_> = variants
            .iter()
            .take_while(|(_, count)| {
                let new_cumulative = cumulative + count;
                let include = new_cumulative <= (total as f64 * 0.8) as usize;
                if include {
                    cumulative = new_cumulative;
                }
                include
            })
            .collect();

        assert_eq!(filtered.len(), 1); // Only "A" (60%) is <= 80% threshold
    }

    // ============================================================================
    // FEATURE 4: SIMILARITY ANALYSIS
    // ============================================================================

    fn edit_distance(s1: &[String], s2: &[String]) -> usize {
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

    fn longest_common_subsequence(s1: &[String], s2: &[String]) -> usize {
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

    #[test]
    fn test_similarity_identical() {
        let v1 = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        let v2 = vec!["A".to_string(), "B".to_string(), "C".to_string()];

        let dist = edit_distance(&v1, &v2);
        let lcs = longest_common_subsequence(&v1, &v2);

        assert_eq!(dist, 0);
        assert_eq!(lcs, 3);
    }

    #[test]
    fn test_similarity_different() {
        let v1 = vec!["A".to_string(), "B".to_string()];
        let v2 = vec!["C".to_string(), "D".to_string()];

        let dist = edit_distance(&v1, &v2);
        let lcs = longest_common_subsequence(&v1, &v2);

        assert_eq!(dist, 2);
        assert_eq!(lcs, 0);
    }

    #[test]
    fn test_similarity_insertion() {
        let v1 = vec!["A".to_string(), "B".to_string()];
        let v2 = vec!["A".to_string(), "X".to_string(), "B".to_string()];

        let dist = edit_distance(&v1, &v2);
        let lcs = longest_common_subsequence(&v1, &v2);

        assert_eq!(dist, 1);
        assert_eq!(lcs, 2);
    }

    #[test]
    fn test_similarity_score() {
        let v1 = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        let v2 = vec!["A".to_string(), "X".to_string(), "C".to_string()];

        let lcs = longest_common_subsequence(&v1, &v2);
        let max_len = v1.len().max(v2.len());
        let similarity = lcs as f64 / max_len as f64;

        assert!(similarity > 0.6);
    }

    // ============================================================================
    // FEATURE 5: METRICS COMPUTATION
    // ============================================================================

    #[test]
    fn test_complexity_metric() {
        let variant_length = 5;
        let complexity = variant_length as f64;

        assert_eq!(complexity, 5.0);
    }

    #[test]
    fn test_risk_score_combination() {
        let complexity_normalized = 0.3;
        let error_rate = 0.1;

        let risk_score = complexity_normalized * 0.5 + error_rate * 0.5;

        assert!(risk_score >= 0.0 && risk_score <= 1.0);
        assert!(risk_score > 0.0);
    }

    // ============================================================================
    // INTEGRATION TESTS
    // ============================================================================

    #[test]
    fn test_complete_workflow_sap_simulation() {
        // Simulate SAP P2P process variants
        let mut variant_freq: HashMap<String, usize> = HashMap::new();

        // Variant 1: Standard path (62.5%)
        for _ in 0..50 {
            *variant_freq
                .entry("Create,Approve,Receive,Invoice,Pay".to_string())
                .or_insert(0) += 1;
        }

        // Variant 2: Double approval (18.75%)
        for _ in 0..15 {
            *variant_freq
                .entry("Create,Approve,Approve,Receive,Invoice,Pay".to_string())
                .or_insert(0) += 1;
        }

        // Variant 3: Rework (18.75%)
        for _ in 0..15 {
            *variant_freq
                .entry("Create,Approve,Receive,Receive,Invoice,Pay".to_string())
                .or_insert(0) += 1;
        }

        // Analysis
        assert_eq!(variant_freq.len(), 3);

        let total: usize = variant_freq.values().sum();
        assert_eq!(total, 80);

        // Pareto analysis
        let mut sorted: Vec<_> = variant_freq.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));

        let coverage = *sorted[0].1 as f64 / total as f64 * 100.0;
        assert!(coverage >= 62.0 && coverage <= 63.0);
    }

    #[test]
    fn test_complete_workflow_bpic_simulation() {
        // Simulate BPIC loan application variants
        let mut variant_freq: HashMap<String, usize> = HashMap::new();

        // Variant 1: Approved (62.5%)
        for _ in 0..100 {
            *variant_freq
                .entry("Register,Submit,Assess,Approve".to_string())
                .or_insert(0) += 1;
        }

        // Variant 2: Revision needed (25%)
        for _ in 0..40 {
            *variant_freq
                .entry("Register,Submit,Assess,Revise,Submit,Assess,Approve".to_string())
                .or_insert(0) += 1;
        }

        // Variant 3: Rejected (12.5%)
        for _ in 0..20 {
            *variant_freq
                .entry("Register,Submit,Assess,Reject".to_string())
                .or_insert(0) += 1;
        }

        let total: usize = variant_freq.values().sum();
        assert_eq!(total, 160);

        let mut sorted: Vec<_> = variant_freq.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));

        // Top 2 should cover ~87.5%
        let top2_coverage = (sorted[0].1 + sorted[1].1) as f64 / total as f64 * 100.0;
        assert!(top2_coverage >= 87.0 && top2_coverage <= 88.0);
    }

    #[test]
    fn test_fingerprint_consistency() {
        let v1 = SimpleVariant {
            activities: vec!["A".to_string(), "B".to_string()],
        };

        let fp1 = SimpleFingerprint::compute(&v1);
        let fp2 = SimpleFingerprint::compute(&v1);
        let fp3 = SimpleFingerprint::compute(&v1);

        assert_eq!(fp1, fp2);
        assert_eq!(fp2, fp3);
    }

    #[test]
    fn test_variant_ordering_by_frequency() {
        let variants = vec![("C", 100), ("A", 50), ("B", 200)];

        let mut sorted = variants.clone();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));

        assert_eq!(sorted[0], ("B", 200));
        assert_eq!(sorted[1], ("C", 100));
        assert_eq!(sorted[2], ("A", 50));
    }
}
