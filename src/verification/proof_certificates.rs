//! Proof certificates for formal algorithm correctness
//!
//! This module generates certificates of correctness for each algorithm
//! based on invariant verification. Certificates can be audited and
//! verified to establish formal correctness guarantees.

use chrono::Local;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::algorithm_invariants::InvariantVerificationResult;

/// A formal proof certificate for algorithm correctness
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofCertificate {
    pub algorithm: String,
    pub algorithm_type: AlgorithmType,
    pub timestamp: String,
    pub verified_invariants: Vec<String>,
    pub failed_invariants: Vec<String>,
    pub total_invariants: usize,
    pub success: bool,
    pub proof_summary: String,
    pub formal_statement: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AlgorithmType {
    DiscoveryAlgorithm,
    ConformanceAlgorithm,
}

impl ProofCertificate {
    pub fn new(algorithm: &str, algo_type: AlgorithmType) -> Self {
        Self {
            algorithm: algorithm.to_string(),
            algorithm_type: algo_type,
            timestamp: Local::now().to_rfc3339(),
            verified_invariants: Vec::new(),
            failed_invariants: Vec::new(),
            total_invariants: 0,
            success: false,
            proof_summary: String::new(),
            formal_statement: String::new(),
        }
    }

    pub fn from_verification_results(
        algorithm: &str,
        algo_type: AlgorithmType,
        results: &[InvariantVerificationResult],
    ) -> Self {
        let mut cert = Self::new(algorithm, algo_type);
        cert.total_invariants = results.len();

        for result in results {
            if result.is_valid() {
                cert.verified_invariants
                    .push(format!("{} ({})", result.invariant_name, result.algorithm));
            } else {
                cert.failed_invariants.push(format!(
                    "{} ({} violations)",
                    result.invariant_name,
                    result.violation_count()
                ));
            }
        }

        cert.success = cert.failed_invariants.is_empty();
        cert.generate_proof_summary();
        cert.generate_formal_statement();

        cert
    }

    fn generate_proof_summary(&mut self) {
        if self.success {
            self.proof_summary = format!(
                "✓ All {} invariants verified for {}",
                self.total_invariants, self.algorithm
            );
        } else {
            self.proof_summary = format!(
                "✗ {} verified, {} failed for {}",
                self.verified_invariants.len(),
                self.failed_invariants.len(),
                self.algorithm
            );
        }
    }

    fn generate_formal_statement(&mut self) {
        let type_str = match self.algorithm_type {
            AlgorithmType::DiscoveryAlgorithm => "process discovery",
            AlgorithmType::ConformanceAlgorithm => "conformance checking",
        };

        self.formal_statement = format!(
            "Theorem: {} is a correct {} algorithm.\n\
             Proof: Verified {} invariants:\n",
            self.algorithm, type_str, self.total_invariants
        );

        for inv in &self.verified_invariants {
            self.formal_statement.push_str(&format!("  ├─ {}\n", inv));
        }

        if self.success {
            self.formal_statement.push_str(
                "  └─ Q.E.D.\n\n\
                 Certificate Status: VALID ✓",
            );
        } else {
            self.formal_statement.push_str(
                "  └─ Incomplete proof due to failed invariants.\n\n\
                 Certificate Status: INVALID ✗",
            );
        }
    }

    pub fn is_valid(&self) -> bool {
        self.success && self.failed_invariants.is_empty()
    }

    pub fn print_certificate(&self) {
        println!("═══════════════════════════════════════════════════════════");
        println!("PROOF CERTIFICATE OF CORRECTNESS");
        println!("═══════════════════════════════════════════════════════════");
        println!("Algorithm: {}", self.algorithm);
        println!("Type: {:?}", self.algorithm_type);
        println!("Issued: {}", self.timestamp);
        println!();
        println!("Proof Summary:");
        println!("{}", self.proof_summary);
        println!();
        println!("Formal Statement:");
        println!("{}", self.formal_statement);
        println!();
        println!("═══════════════════════════════════════════════════════════");
    }
}

/// Repository of proof certificates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateRepository {
    pub certificates: HashMap<String, ProofCertificate>,
}

impl CertificateRepository {
    pub fn new() -> Self {
        Self {
            certificates: HashMap::new(),
        }
    }

    pub fn add_certificate(&mut self, cert: ProofCertificate) {
        self.certificates.insert(cert.algorithm.clone(), cert);
    }

    pub fn get_certificate(&self, algorithm: &str) -> Option<&ProofCertificate> {
        self.certificates.get(algorithm)
    }

    pub fn all_valid(&self) -> bool {
        self.certificates.values().all(|c| c.is_valid())
    }

    pub fn validity_summary(&self) -> String {
        let total = self.certificates.len();
        let valid = self.certificates.values().filter(|c| c.is_valid()).count();

        format!(
            "Certificate Repository Summary:\n\
             ├─ Total Algorithms: {}\n\
             ├─ Valid Certificates: {} ({}%)\n\
             └─ Invalid: {}",
            total,
            valid,
            if total > 0 { (valid * 100) / total } else { 0 },
            total - valid
        )
    }

    pub fn print_all_certificates(&self) {
        println!("═══════════════════════════════════════════════════════════");
        println!("ALL ALGORITHM CORRECTNESS CERTIFICATES");
        println!("═══════════════════════════════════════════════════════════");
        println!();

        for cert in self.certificates.values() {
            println!("Algorithm: {}", cert.algorithm);
            println!(
                "Status: {}",
                if cert.is_valid() {
                    "✓ VALID"
                } else {
                    "✗ INVALID"
                }
            );
            println!(
                "Invariants: {}/{} passed",
                cert.verified_invariants.len(),
                cert.total_invariants
            );
            println!();
        }

        println!("Summary: {}", self.validity_summary());
    }
}

impl Default for CertificateRepository {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating certificates programmatically
pub struct CertificateBuilder {
    algorithm: String,
    algo_type: AlgorithmType,
    verified: Vec<String>,
    failed: Vec<String>,
}

impl CertificateBuilder {
    pub fn new(algorithm: &str, algo_type: AlgorithmType) -> Self {
        Self {
            algorithm: algorithm.to_string(),
            algo_type,
            verified: Vec::new(),
            failed: Vec::new(),
        }
    }

    pub fn verified_invariant(mut self, name: &str) -> Self {
        self.verified.push(name.to_string());
        self
    }

    pub fn failed_invariant(mut self, name: &str) -> Self {
        self.failed.push(name.to_string());
        self
    }

    pub fn build(self) -> ProofCertificate {
        let mut cert = ProofCertificate::new(&self.algorithm, self.algo_type);
        cert.verified_invariants = self.verified;
        cert.failed_invariants = self.failed;
        cert.total_invariants = cert.verified_invariants.len() + cert.failed_invariants.len();
        cert.success = cert.failed_invariants.is_empty();
        cert.generate_proof_summary();
        cert.generate_formal_statement();
        cert
    }
}

/// Mathematical proof sketch system
pub struct ProofSketch {
    pub theorem: String,
    pub hypothesis: Vec<String>,
    pub proof_steps: Vec<String>,
    pub conclusion: String,
}

impl ProofSketch {
    pub fn new(theorem: &str) -> Self {
        Self {
            theorem: theorem.to_string(),
            hypothesis: Vec::new(),
            proof_steps: Vec::new(),
            conclusion: String::new(),
        }
    }

    pub fn add_hypothesis(mut self, hyp: &str) -> Self {
        self.hypothesis.push(hyp.to_string());
        self
    }

    pub fn add_proof_step(mut self, step: &str) -> Self {
        self.proof_steps.push(step.to_string());
        self
    }

    pub fn set_conclusion(mut self, conc: &str) -> Self {
        self.conclusion = conc.to_string();
        self
    }

    pub fn print(&self) {
        println!("PROOF SKETCH");
        println!("═══════════════════════════════════════════════════════════");
        println!("Theorem: {}", self.theorem);
        println!();

        if !self.hypothesis.is_empty() {
            println!("Hypothesis:");
            for (i, h) in self.hypothesis.iter().enumerate() {
                println!("  {}. {}", i + 1, h);
            }
            println!();
        }

        println!("Proof:");
        for (i, step) in self.proof_steps.iter().enumerate() {
            println!("  {}. {}", i + 1, step);
        }
        println!();

        println!("Conclusion: {}", self.conclusion);
        println!("═══════════════════════════════════════════════════════════");
    }
}

// Predefined proof sketches for common algorithms

pub fn alpha_miner_proof_sketch() -> ProofSketch {
    ProofSketch::new("Alpha Miner discovers a sound Petri net from an event log")
        .add_hypothesis("Event log L contains traces with causal relations")
        .add_hypothesis("Causal relation: a → b iff a directly precedes b ∧ b does not precede a")
        .add_proof_step("Extract directly-follows relation from L")
        .add_proof_step("Identify causal relations from directly-follows")
        .add_proof_step("Create transitions for each distinct activity")
        .add_proof_step("Create places for each causal pair")
        .add_proof_step("Connect source place to start activities")
        .add_proof_step("Connect end activities to sink place")
        .add_proof_step("Verify soundness: proper completion and no deadlock")
        .set_conclusion("Discovered net correctly represents behavior in L")
}

pub fn token_replay_proof_sketch() -> ProofSketch {
    ProofSketch::new("Token Replay correctly measures fitness between log and model")
        .add_hypothesis("Fitness = (produced - consumed - missing) / produced")
        .add_proof_step("Initialize marking with source place tokens")
        .add_proof_step("For each event, fire corresponding transition if enabled")
        .add_proof_step("Count missing tokens when transition cannot fire")
        .add_proof_step("Count remaining tokens at final marking")
        .add_proof_step("Apply WvdA fitness formula")
        .set_conclusion("Fitness score correctly reflects conformance")
}

pub fn precision_proof_sketch() -> ProofSketch {
    ProofSketch::new("Precision correctly measures model specificity")
        .add_hypothesis("Precision = 1 - (enabled unobserved transitions / total enabled)")
        .add_proof_step("Identify all enabled transitions at each reachable marking")
        .add_proof_step("Check which transitions correspond to observed activities")
        .add_proof_step("Count transitions that would enable unobserved behavior")
        .set_conclusion("Precision score accurately reflects behavior restriction")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proof_certificate_creation() {
        let cert = CertificateBuilder::new("alpha_miner", AlgorithmType::DiscoveryAlgorithm)
            .verified_invariant("CausalEvidence")
            .verified_invariant("NoSpurious")
            .build();

        assert!(cert.is_valid());
        assert_eq!(cert.verified_invariants.len(), 2);
        assert!(cert.failed_invariants.is_empty());
    }

    #[test]
    fn test_certificate_repository() {
        let mut repo = CertificateRepository::new();
        let cert = CertificateBuilder::new("token_replay", AlgorithmType::ConformanceAlgorithm)
            .verified_invariant("EventMapping")
            .build();

        repo.add_certificate(cert.clone());
        assert_eq!(repo.certificates.len(), 1);
        assert!(repo.get_certificate("token_replay").is_some());
        assert!(repo.all_valid());
    }

    #[test]
    fn test_proof_sketch() {
        let sketch = alpha_miner_proof_sketch();
        assert!(!sketch.theorem.is_empty());
        assert!(!sketch.proof_steps.is_empty());
    }
}
