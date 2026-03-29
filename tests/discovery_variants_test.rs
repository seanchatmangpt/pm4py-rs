//! Comprehensive Variant Detection, Analysis, and Filtering Tests
//!
//! These tests validate:
//! 1. Variant Fingerprinting (Deterministic hashing)
//! 2. Variant Frequency Analysis (Pareto ranking)
//! 3. Variant Filtering (Multiple strategies)
//! 4. Variant Similarity (Edit distance & LCS)
//! 5. Variant Metrics (Complexity, performance, risk)

#[cfg(test)]
mod variant_discovery_tests {
    use chrono::{Duration, Utc};
    use pm4py::{
        Event, EventLog, FilterStrategy, Trace, Variant, VariantAnalysis, VariantFilter,
        VariantFingerprint, VariantInfo, VariantMetrics, VariantSimilarity,
    };
    use std::collections::HashMap;

    // ============================================================================
    // HELPER FUNCTIONS
    // ============================================================================

    /// Create a trace with given activities
    fn create_trace(id: &str, activities: &[&str]) -> Trace {
        let mut trace = Trace::new(id);
        let mut timestamp = Utc::now();
        for activity in activities {
            let event = Event::new(*activity, timestamp);
            trace.add_event(event);
            timestamp = timestamp + Duration::seconds(10);
        }
        trace
    }

    /// Create a trace with error activity
    fn create_trace_with_error(id: &str, activities: &[&str]) -> Trace {
        let mut trace = Trace::new(id);
        let mut timestamp = Utc::now();
        for activity in activities {
            let event = Event::new(*activity, timestamp);
            trace.add_event(event);
            timestamp = timestamp + Duration::seconds(10);
        }
        trace
    }

    /// Load SAP event log (simulated)
    fn load_sap_log() -> EventLog {
        let mut log = EventLog::new();
        // Simulate typical SAP process: Create PO -> Approve -> Receive -> Invoice -> Pay
        for i in 0..50 {
            log.add_trace(create_trace(
                &format!("SAP_{}", i),
                &["Create_PO", "Approve_PO", "Receive_Goods", "Invoice", "Pay"],
            ));
        }
        // Variant with extra approval
        for i in 50..65 {
            log.add_trace(create_trace(
                &format!("SAP_{}", i),
                &[
                    "Create_PO",
                    "Approve_PO",
                    "Approve_PO",
                    "Receive_Goods",
                    "Invoice",
                    "Pay",
                ],
            ));
        }
        // Variant with rework
        for i in 65..80 {
            log.add_trace(create_trace(
                &format!("SAP_{}", i),
                &[
                    "Create_PO",
                    "Approve_PO",
                    "Receive_Goods",
                    "Receive_Goods",
                    "Invoice",
                    "Pay",
                ],
            ));
        }
        log
    }

    /// Load BPIC event log (simulated)
    fn load_bpic_log() -> EventLog {
        let mut log = EventLog::new();
        // Variant 1: Happy path
        for i in 0..100 {
            log.add_trace(create_trace(
                &format!("BPIC_{}", i),
                &["Register", "Submit_Application", "Assess", "Approve"],
            ));
        }
        // Variant 2: Needs revision
        for i in 100..140 {
            log.add_trace(create_trace(
                &format!("BPIC_{}", i),
                &[
                    "Register",
                    "Submit_Application",
                    "Assess",
                    "Request_Revision",
                    "Submit_Application",
                    "Assess",
                    "Approve",
                ],
            ));
        }
        // Variant 3: Rejected
        for i in 140..160 {
            log.add_trace(create_trace(
                &format!("BPIC_{}", i),
                &["Register", "Submit_Application", "Assess", "Reject"],
            ));
        }
        // Rare variant 4: Escalation
        for i in 160..165 {
            log.add_trace(create_trace(
                &format!("BPIC_{}", i),
                &[
                    "Register",
                    "Submit_Application",
                    "Assess",
                    "Escalate",
                    "Review_Escalation",
                    "Approve",
                ],
            ));
        }
        log
    }

    // ============================================================================
    // FEATURE 1: VARIANT FINGERPRINTING TESTS (Deterministic)
    // ============================================================================

    #[test]
    fn test_fingerprint_creation() {
        let variant = Variant::new(vec!["A".to_string(), "B".to_string()]);
        let fp = VariantFingerprint::compute(&variant);

        assert_ne!(fp.0, 0);
        let hex = fp.to_hex();
        assert_eq!(hex.len(), 8);
    }

    #[test]
    fn test_fingerprint_deterministic() {
        let v1 = Variant::new(vec!["A".to_string(), "B".to_string()]);
        let v2 = Variant::new(vec!["A".to_string(), "B".to_string()]);

        let fp1 = VariantFingerprint::compute(&v1);
        let fp2 = VariantFingerprint::compute(&v2);

        assert_eq!(fp1, fp2);
    }

    #[test]
    fn test_fingerprint_different_sequences() {
        let v1 = Variant::new(vec!["A".to_string(), "B".to_string()]);
        let v2 = Variant::new(vec!["B".to_string(), "A".to_string()]);

        let fp1 = VariantFingerprint::compute(&v1);
        let fp2 = VariantFingerprint::compute(&v2);

        assert_ne!(fp1, fp2);
    }

    #[test]
    fn test_fingerprint_from_activities() {
        let activities = vec!["X".to_string(), "Y".to_string(), "Z".to_string()];
        let fp = VariantFingerprint::from_activities(&activities);

        assert_ne!(fp.0, 0);
    }

    #[test]
    fn test_fingerprint_collision_resistance() {
        let mut fingerprints = HashMap::new();

        for i in 0..100 {
            let variant = Variant::new(vec![
                format!("Activity_{}", i),
                format!("Activity_{}", i + 1),
            ]);
            let fp = VariantFingerprint::compute(&variant);
            fingerprints.insert(fp.0, i);
        }

        assert_eq!(fingerprints.len(), 100);
    }

    // ============================================================================
    // FEATURE 2: VARIANT FREQUENCY ANALYSIS (Pareto)
    // ============================================================================

    #[test]
    fn test_discover_variants_simple() {
        let mut log = EventLog::new();
        log.add_trace(create_trace("1", &["A", "B"]));
        log.add_trace(create_trace("2", &["A", "B"]));
        log.add_trace(create_trace("3", &["A", "C"]));

        let analysis = VariantAnalysis::discover(&log);

        assert_eq!(analysis.total_traces, 3);
        assert_eq!(analysis.unique_variants, 2);
        assert_eq!(analysis.variants[0].frequency, 2);
    }

    #[test]
    fn test_discover_variants_empty_log() {
        let log = EventLog::new();
        let analysis = VariantAnalysis::discover(&log);

        assert_eq!(analysis.total_traces, 0);
        assert_eq!(analysis.unique_variants, 0);
    }

    #[test]
    fn test_discover_variants_single_variant() {
        let mut log = EventLog::new();
        for i in 0..20 {
            log.add_trace(create_trace(&i.to_string(), &["A", "B", "C"]));
        }

        let analysis = VariantAnalysis::discover(&log);

        assert_eq!(analysis.total_traces, 20);
        assert_eq!(analysis.unique_variants, 1);
        assert_eq!(analysis.variants[0].frequency, 20);
    }

    #[test]
    fn test_pareto_analysis_real_sap() {
        let log = load_sap_log();
        let analysis = VariantAnalysis::discover(&log);

        assert_eq!(analysis.total_traces, 80);
        assert_eq!(analysis.unique_variants, 3);

        // Check Pareto ordering
        for i in 0..analysis.variants.len() - 1 {
            assert!(analysis.variants[i].frequency >= analysis.variants[i + 1].frequency);
        }
    }

    #[test]
    fn test_pareto_frontier_80_percent() {
        let log = load_bpic_log();
        let analysis = VariantAnalysis::discover(&log);
        let frontier = analysis.pareto_frontier();

        let coverage: usize = frontier.iter().map(|v| v.frequency).sum();
        let coverage_pct = coverage as f64 / analysis.total_traces as f64 * 100.0;

        // Should reach >=80% coverage with minimal variants
        // BPIC log: V1=100, V2=40 → 140/165 = 84.8%
        assert!(coverage_pct >= 80.0 && coverage_pct <= 90.0);
        assert!(frontier.len() < analysis.unique_variants);
    }

    #[test]
    fn test_coverage_top_k() {
        let mut log = EventLog::new();
        for _i in 0..100 {
            log.add_trace(create_trace("a", &["A"]));
        }
        for _i in 0..50 {
            log.add_trace(create_trace("b", &["B"]));
        }
        for _i in 0..25 {
            log.add_trace(create_trace("c", &["C"]));
        }

        let analysis = VariantAnalysis::discover(&log);
        let coverage = analysis.coverage_top_k(1);

        // Top variant (100 traces) out of 175 total = 57.14%
        assert!(coverage >= 57.0 && coverage <= 58.0);
    }

    #[test]
    fn test_variant_trace_ids_tracking() {
        let mut log = EventLog::new();
        log.add_trace(create_trace("t1", &["A", "B"]));
        log.add_trace(create_trace("t2", &["A", "B"]));
        log.add_trace(create_trace("t3", &["C"]));

        let analysis = VariantAnalysis::discover(&log);

        let ab_variant = &analysis.variants[0];
        assert_eq!(ab_variant.trace_ids.len(), 2);
        assert!(ab_variant.trace_ids.contains(&"t1".to_string()));
        assert!(ab_variant.trace_ids.contains(&"t2".to_string()));
    }

    // ============================================================================
    // FEATURE 3: VARIANT FILTERING
    // ============================================================================

    #[test]
    fn test_filter_minimum_frequency() {
        let mut log = EventLog::new();
        for _i in 0..10 {
            log.add_trace(create_trace("a", &["A"]));
        }
        for _i in 0..5 {
            log.add_trace(create_trace("b", &["B"]));
        }
        for _i in 0..2 {
            log.add_trace(create_trace("c", &["C"]));
        }

        let analysis = VariantAnalysis::discover(&log);
        let filter = VariantFilter::new(FilterStrategy::MinimumFrequency { threshold: 5 });
        let filtered = filter.apply(&analysis);

        assert_eq!(filtered.unique_variants, 2);
        assert_eq!(filtered.total_traces, 15);
    }

    #[test]
    fn test_filter_top_k() {
        let mut log = EventLog::new();
        for _i in 0..15 {
            log.add_trace(create_trace("a", &["A", "B"]));
        }
        for _i in 0..8 {
            log.add_trace(create_trace("b", &["A", "C"]));
        }
        for _i in 0..3 {
            log.add_trace(create_trace("c", &["A", "D"]));
        }

        let analysis = VariantAnalysis::discover(&log);
        let filter = VariantFilter::new(FilterStrategy::TopK { k: 2 });
        let filtered = filter.apply(&analysis);

        assert_eq!(filtered.unique_variants, 2);
        assert_eq!(filtered.variants[0].frequency, 15);
        assert_eq!(filtered.variants[1].frequency, 8);
    }

    #[test]
    fn test_filter_coverage_percentage() {
        let mut log = EventLog::new();
        for _i in 0..60 {
            log.add_trace(create_trace("a", &["A"]));
        }
        for _i in 0..30 {
            log.add_trace(create_trace("b", &["B"]));
        }
        for _i in 0..10 {
            log.add_trace(create_trace("c", &["C"]));
        }

        let analysis = VariantAnalysis::discover(&log);
        let filter = VariantFilter::new(FilterStrategy::CoveragePercentage { target: 80.0 });
        let filtered = filter.apply(&analysis);

        let coverage = filtered.total_traces as f64 / analysis.total_traces as f64 * 100.0;
        assert!(coverage >= 80.0);
    }

    #[test]
    fn test_filter_activity_whitelist() {
        let mut log = EventLog::new();
        log.add_trace(create_trace("1", &["A", "B"]));
        log.add_trace(create_trace("2", &["A", "B", "C"]));
        log.add_trace(create_trace("3", &["A", "D"]));
        log.add_trace(create_trace("4", &["X", "Y"]));

        let analysis = VariantAnalysis::discover(&log);
        let filter = VariantFilter::new(FilterStrategy::ActivityWhitelist {
            activities: vec!["A".to_string(), "B".to_string()],
        });
        let filtered = filter.apply(&analysis);

        assert_eq!(filtered.unique_variants, 1); // Only A,B
    }

    #[test]
    fn test_filter_apply_to_log() {
        let mut log = EventLog::new();
        for i in 0..20 {
            log.add_trace(create_trace(&i.to_string(), &["A"]));
        }
        for i in 20..30 {
            log.add_trace(create_trace(&i.to_string(), &["B"]));
        }

        let analysis = VariantAnalysis::discover(&log);
        let filter = VariantFilter::new(FilterStrategy::TopK { k: 1 });
        let filtered_log = filter.apply_to_log(&log, &analysis);

        assert_eq!(filtered_log.len(), 20);
    }

    #[test]
    fn test_filter_real_sap_log() {
        let log = load_sap_log();
        let analysis = VariantAnalysis::discover(&log);

        // Keep only top variant (80% of traces)
        let filter = VariantFilter::new(FilterStrategy::TopK { k: 1 });
        let filtered = filter.apply(&analysis);

        assert_eq!(filtered.unique_variants, 1);
        assert_eq!(filtered.variants[0].frequency, 50);
    }

    // ============================================================================
    // FEATURE 4: VARIANT SIMILARITY ANALYSIS
    // ============================================================================

    #[test]
    fn test_similarity_identical() {
        let v1 = Variant::new(vec!["A".to_string(), "B".to_string(), "C".to_string()]);
        let v2 = Variant::new(vec!["A".to_string(), "B".to_string(), "C".to_string()]);

        let sim = VariantSimilarity::compute(&v1, &v2);

        assert_eq!(sim.edit_distance, 0);
        assert_eq!(sim.lcs_length, 3);
        assert_eq!(sim.similarity_score, 1.0);
    }

    #[test]
    fn test_similarity_completely_different() {
        let v1 = Variant::new(vec!["A".to_string(), "B".to_string()]);
        let v2 = Variant::new(vec!["C".to_string(), "D".to_string()]);

        let sim = VariantSimilarity::compute(&v1, &v2);

        assert_eq!(sim.edit_distance, 2);
        assert_eq!(sim.lcs_length, 0);
        assert_eq!(sim.similarity_score, 0.0);
    }

    #[test]
    fn test_similarity_one_insertion() {
        let v1 = Variant::new(vec!["A".to_string(), "B".to_string()]);
        let v2 = Variant::new(vec!["A".to_string(), "X".to_string(), "B".to_string()]);

        let sim = VariantSimilarity::compute(&v1, &v2);

        assert_eq!(sim.edit_distance, 1);
        assert!(sim.similarity_score > 0.6);
    }

    #[test]
    fn test_similarity_partial_match() {
        let v1 = Variant::new(vec![
            "Create_PO".to_string(),
            "Approve_PO".to_string(),
            "Receive".to_string(),
            "Invoice".to_string(),
        ]);
        let v2 = Variant::new(vec![
            "Create_PO".to_string(),
            "Approve_PO".to_string(),
            "Approve_PO".to_string(),
            "Receive".to_string(),
            "Invoice".to_string(),
        ]);

        let sim = VariantSimilarity::compute(&v1, &v2);

        assert!(sim.similarity_score > 0.7);
    }

    #[test]
    fn test_similarity_empty_variants() {
        let v1 = Variant::new(vec![]);
        let v2 = Variant::new(vec![]);

        let sim = VariantSimilarity::compute(&v1, &v2);

        assert_eq!(sim.edit_distance, 0);
        assert_eq!(sim.similarity_score, 1.0);
    }

    // ============================================================================
    // FEATURE 5: VARIANT METRICS (Complexity, Performance, Risk)
    // ============================================================================

    #[test]
    fn test_metrics_complexity() {
        let variant = VariantInfo::new(
            Variant::new(vec!["A".to_string(), "B".to_string(), "C".to_string()]),
            5,
            vec!["1".to_string()],
        );

        let traces = vec![create_trace("1", &["A", "B", "C"])];
        let metrics = VariantMetrics::compute(&variant, &traces);

        assert_eq!(metrics.complexity, 3.0);
    }

    #[test]
    fn test_metrics_duration() {
        let variant = VariantInfo::new(
            Variant::new(vec!["A".to_string(), "B".to_string()]),
            1,
            vec!["1".to_string()],
        );

        let traces = vec![create_trace("1", &["A", "B"])];
        let metrics = VariantMetrics::compute(&variant, &traces);

        // 10 seconds between events
        assert!(metrics.avg_duration_ms > 9000.0 && metrics.avg_duration_ms < 11000.0);
    }

    #[test]
    fn test_metrics_risk_score() {
        let variant = VariantInfo::new(
            Variant::new(vec!["A".to_string()]),
            1,
            vec!["1".to_string()],
        );

        let traces = vec![create_trace("1", &["A"])];
        let metrics = VariantMetrics::compute(&variant, &traces);

        assert!(metrics.risk_score >= 0.0 && metrics.risk_score <= 1.0);
    }

    #[test]
    fn test_metrics_high_complexity_risk() {
        let long_activities = (0..20)
            .map(|i| format!("Activity_{}", i))
            .collect::<Vec<_>>();
        let activity_strs: Vec<&str> = long_activities.iter().map(|s| s.as_str()).collect();

        let variant = VariantInfo::new(
            Variant::new(activity_strs.iter().map(|s| s.to_string()).collect()),
            1,
            vec!["1".to_string()],
        );

        let traces = vec![create_trace("1", &activity_strs)];
        let metrics = VariantMetrics::compute(&variant, &traces);

        // Higher complexity should increase risk: 20 activities / 8 = 2.5
        // risk_score = min(2.5, 1.0) * 0.5 + 0.0 * 0.5 = 0.5
        assert!(metrics.risk_score >= 0.5);
    }

    #[test]
    fn test_variant_info_coverage_percentage() {
        let variant = VariantInfo::new(
            Variant::new(vec!["A".to_string()]),
            25,
            vec!["1".to_string()],
        );

        let coverage = variant.coverage_percentage(100);
        assert_eq!(coverage, 25.0);

        let coverage_half = variant.coverage_percentage(50);
        assert_eq!(coverage_half, 50.0);
    }

    // ============================================================================
    // INTEGRATION TESTS
    // ============================================================================

    #[test]
    fn test_real_world_sap_workflow() {
        let log = load_sap_log();
        let analysis = VariantAnalysis::discover(&log);

        // 3 main variants
        assert_eq!(analysis.unique_variants, 3);

        // Top variant is 62.5% (50/80)
        let coverage = analysis.coverage_top_k(1);
        assert!(coverage >= 62.0 && coverage <= 63.0);

        // Pareto frontier should have 2 variants covering ~95%
        let frontier = analysis.pareto_frontier();
        assert!(frontier.len() <= 2);
    }

    #[test]
    fn test_real_world_bpic_workflow() {
        let log = load_bpic_log();
        let analysis = VariantAnalysis::discover(&log);

        // Filter to top variants covering 90%
        let filter = VariantFilter::new(FilterStrategy::CoveragePercentage { target: 90.0 });
        let filtered = filter.apply(&analysis);

        assert!(filtered.unique_variants >= 2);
        assert!(filtered.unique_variants <= 3);
    }

    #[test]
    fn test_fingerprint_consistency_across_analysis() {
        let mut log = EventLog::new();
        log.add_trace(create_trace("1", &["A", "B"]));
        log.add_trace(create_trace("2", &["A", "B"]));

        let analysis = VariantAnalysis::discover(&log);

        // All instances should have same fingerprint
        let fingerprints: Vec<_> = analysis
            .variants
            .iter()
            .map(|v| v.fingerprint.to_hex())
            .collect();

        assert_eq!(fingerprints.len(), 1);
    }

    #[test]
    fn test_variant_length_range() {
        let mut log = EventLog::new();
        log.add_trace(create_trace("1", &["A"]));
        log.add_trace(create_trace("2", &["A", "B", "C"]));
        log.add_trace(create_trace("3", &["A", "B", "C", "D", "E"]));

        let analysis = VariantAnalysis::discover(&log);

        // All variants have frequency 1, so we check the set of lengths
        let lengths: Vec<usize> = analysis.variants.iter().map(|v| v.variant.len()).collect();

        assert!(lengths.contains(&1));
        assert!(lengths.contains(&3));
        assert!(lengths.contains(&5));
        assert_eq!(analysis.unique_variants, 3);
    }

    #[test]
    fn test_chained_filtering() {
        let log = load_bpic_log();
        let analysis = VariantAnalysis::discover(&log);

        // First: top 3 variants
        let filter1 = VariantFilter::new(FilterStrategy::TopK { k: 3 });
        let step1 = filter1.apply(&analysis);

        // Then: only those with min frequency 20
        let filter2 = VariantFilter::new(FilterStrategy::MinimumFrequency { threshold: 20 });
        let step2 = filter2.apply(&step1);

        assert!(step2.unique_variants >= 2);
    }

    #[test]
    fn test_all_variants_discovered() {
        let mut log = EventLog::new();
        let variants = vec![
            vec!["A"],
            vec!["A", "B"],
            vec!["A", "B", "C"],
            vec!["X", "Y"],
            vec!["X", "Y", "Z"],
        ];

        for (idx, variant) in variants.iter().enumerate() {
            log.add_trace(create_trace(&idx.to_string(), variant));
        }

        let analysis = VariantAnalysis::discover(&log);

        assert_eq!(analysis.unique_variants, 5);
    }
}
