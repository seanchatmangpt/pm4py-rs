//! Concrete implementations of invariant checkers for each algorithm

use crate::log::EventLog;
use crate::models::PetriNet;

use super::algorithm_invariants::*;

/// Master invariant verification suite runner
pub struct InvariantSuite;

impl InvariantSuite {
    /// Run all discovery algorithm invariants
    pub fn verify_all_discovery_algorithms(
        log: &EventLog,
        net: &PetriNet,
    ) -> Vec<InvariantVerificationResult> {
        vec![
            // Alpha Miner invariants
            AlphaMinerInvariant::verify_causal_evidence(log, net),
            AlphaMinerInvariant::verify_no_spurious_transitions(log, net),
            // Inductive Miner invariants
            InductiveMinerInvariant::verify_split_validity(log, net),
            // Heuristic Miner invariants
            HeuristicMinerInvariant::verify_frequency_threshold(log, net, 0.0),
            // Causal Net Miner invariants
            CausalNetMinerInvariant::verify_dependency_correctness(log, net),
            // Split Miner invariants
            SplitMinerInvariant::verify_split_frequency(log, net),
            // ILP Miner invariants
            ILPMinerInvariant::verify_constraint_satisfaction(log, net),
            // Declare Miner invariants
            DeclareMinerInvariant::verify_constraint_enforcement(log, net),
        ]
    }

    /// Run all conformance algorithm invariants
    pub fn verify_all_conformance_algorithms(
        log: &EventLog,
        net: &PetriNet,
    ) -> Vec<InvariantVerificationResult> {
        vec![
            // Token Replay invariants
            TokenReplayInvariant::verify_event_transition_mapping(log, net),
            TokenReplayInvariant::verify_token_conservation(log, net),
            // Precision invariants
            PrecisionInvariant::verify_no_unobserved_behavior(log, net),
            // Generalization invariants
            GeneralizationInvariant::verify_all_traces_replayable(log, net),
            // Alignment invariants
            AlignmentInvariant::verify_cost_non_negativity(log, net),
            AlignmentInvariant::verify_cost_symmetry(),
            // Behavioral Profile invariants
            BehavioralProfileInvariant::verify_relation_transitivity(log, net),
            // Extended Fitness invariants
            ExtendedFitnessInvariant::verify_score_normalization(0.5),
            ExtendedFitnessInvariant::verify_quality_dimensions(0.8, 0.7, 0.75),
        ]
    }

    /// Run all invariants (both discovery and conformance)
    pub fn verify_complete_suite(
        log: &EventLog,
        net: &PetriNet,
    ) -> Vec<InvariantVerificationResult> {
        let mut results = Self::verify_all_discovery_algorithms(log, net);
        results.extend(Self::verify_all_conformance_algorithms(log, net));
        results
    }

    /// Generate invariant verification report
    pub fn generate_report(results: &[InvariantVerificationResult]) -> String {
        let total = results.len();
        let passed = results.iter().filter(|r| r.is_valid()).count();
        let failed = total - passed;
        let total_violations: usize = results.iter().map(|r| r.violation_count()).sum();

        let mut report = format!(
            "═══════════════════════════════════════════════════════════\n\
             ALGORITHM INVARIANT VERIFICATION REPORT\n\
             ═══════════════════════════════════════════════════════════\n\n\
             Summary:\n\
             ├─ Total Invariants: {}\n\
             ├─ Passed: {} ({}%)\n\
             ├─ Failed: {} ({}%)\n\
             └─ Total Violations: {}\n\n",
            total,
            passed,
            if total > 0 { (passed * 100) / total } else { 0 },
            failed,
            if total > 0 { (failed * 100) / total } else { 0 },
            total_violations
        );

        report.push_str("Discovery Algorithm Invariants:\n");
        report.push_str("────────────────────────────────────\n");
        for result in results
            .iter()
            .filter(|r| r.algorithm.ends_with("_miner") && !r.algorithm.ends_with("ilp_miner"))
        {
            report.push_str(&format!(
                "├─ {}: {} ({} violations)\n",
                result.invariant_name,
                if result.is_valid() { "✓" } else { "✗" },
                result.violation_count()
            ));
        }

        report.push_str("\nConformance Algorithm Invariants:\n");
        report.push_str("────────────────────────────────────\n");
        for result in results
            .iter()
            .filter(|r| !r.algorithm.ends_with("_miner") || r.algorithm == "ilp_miner")
        {
            report.push_str(&format!(
                "├─ {}: {} ({} violations)\n",
                result.invariant_name,
                if result.is_valid() { "✓" } else { "✗" },
                result.violation_count()
            ));
        }

        if total_violations > 0 {
            report.push_str("\n\nViolations:\n");
            report.push_str("────────────────────────────────────\n");
            for result in results.iter().filter(|r| !r.is_valid()) {
                report.push_str(&format!(
                    "\n{} ({})\n",
                    result.invariant_name, result.algorithm
                ));
                for violation in &result.violations {
                    report.push_str(&format!("  └─ {}\n", violation));
                }
            }
        }

        report
    }
}

/// Helper to verify invariants with custom thresholds
pub struct CustomInvariantChecker {
    pub frequency_threshold: f64,
    pub fitness_threshold: f64,
    pub precision_threshold: f64,
}

impl CustomInvariantChecker {
    pub fn new() -> Self {
        Self {
            frequency_threshold: 0.0,
            fitness_threshold: 0.0,
            precision_threshold: 0.0,
        }
    }

    pub fn with_frequency_threshold(mut self, threshold: f64) -> Self {
        self.frequency_threshold = threshold;
        self
    }

    pub fn with_fitness_threshold(mut self, threshold: f64) -> Self {
        self.fitness_threshold = threshold;
        self
    }

    pub fn with_precision_threshold(mut self, threshold: f64) -> Self {
        self.precision_threshold = threshold;
        self
    }

    pub fn verify_discovery_invariants(
        &self,
        log: &EventLog,
        net: &PetriNet,
    ) -> Vec<InvariantVerificationResult> {
        vec![
            AlphaMinerInvariant::verify_causal_evidence(log, net),
            AlphaMinerInvariant::verify_no_spurious_transitions(log, net),
            InductiveMinerInvariant::verify_split_validity(log, net),
            HeuristicMinerInvariant::verify_frequency_threshold(log, net, self.frequency_threshold),
            CausalNetMinerInvariant::verify_dependency_correctness(log, net),
            SplitMinerInvariant::verify_split_frequency(log, net),
            ILPMinerInvariant::verify_constraint_satisfaction(log, net),
            DeclareMinerInvariant::verify_constraint_enforcement(log, net),
        ]
    }
}

impl Default for CustomInvariantChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistical validator for invariant results
pub struct InvariantStatistics {
    pub total_invariants: usize,
    pub passed_invariants: usize,
    pub failed_invariants: usize,
    pub total_violations: usize,
    pub success_rate: f64,
    pub average_violations_per_failure: f64,
}

impl InvariantStatistics {
    pub fn from_results(results: &[InvariantVerificationResult]) -> Self {
        let total = results.len();
        let passed = results.iter().filter(|r| r.is_valid()).count();
        let failed = total - passed;
        let total_violations: usize = results.iter().map(|r| r.violation_count()).sum();
        let avg_violations = if failed > 0 {
            total_violations as f64 / failed as f64
        } else {
            0.0
        };

        Self {
            total_invariants: total,
            passed_invariants: passed,
            failed_invariants: failed,
            total_violations,
            success_rate: if total > 0 {
                passed as f64 / total as f64
            } else {
                0.0
            },
            average_violations_per_failure: avg_violations,
        }
    }

    pub fn is_complete_success(&self) -> bool {
        self.failed_invariants == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invariant_statistics() {
        let mut results = vec![];

        let mut result1 = InvariantVerificationResult::new("test1", "alg1");
        result1.passed = true;

        let mut result2 = InvariantVerificationResult::new("test2", "alg2");
        result2.passed = false;
        result2.add_violation("violation 1".to_string());
        result2.add_violation("violation 2".to_string());

        results.push(result1);
        results.push(result2);

        let stats = InvariantStatistics::from_results(&results);

        assert_eq!(stats.total_invariants, 2);
        assert_eq!(stats.passed_invariants, 1);
        assert_eq!(stats.failed_invariants, 1);
        assert_eq!(stats.total_violations, 2);
        assert!((stats.success_rate - 0.5).abs() < 0.001);
    }
}
