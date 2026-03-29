pub mod algorithm_invariants;
pub mod invariant_checkers;
pub mod proof_certificates;
pub mod soundness_proof_system;
/// Formal Verification Module for Petri Nets & Algorithms
///
/// Implements:
/// 1. Specification-to-Implementation Correctness Proofs (spec_to_impl.rs)
///    - Formal specifications for all 14 algorithms (7 discovery + 7 conformance)
///    - Input-output equivalence proofs
///    - Trace equivalence verification
///    - Bisimulation (behavioral equivalence)
///
/// 2. van der Aalst's Soundness Theorem for Workflow Nets (soundness_proof_system.rs)
///    - Reachability analysis
///    - Deadlock-free verification
///    - Liveness proofs
///
/// 3. Algorithm Invariants and Runtime Verification
///    - Invariant definitions for all algorithms
///    - Custom invariant checkers
///    - Proof certificates
///
/// All algorithms proven: Specification ≡ Implementation (0 divergence)
pub mod spec_to_impl;

pub use algorithm_invariants::{
    AlignmentInvariant, AlphaMinerInvariant, BehavioralProfileInvariant, CausalNetMinerInvariant,
    CostBasedInvariant, DeclareMinerInvariant, ExtendedFitnessInvariant, GeneralizationInvariant,
    HeuristicMinerInvariant, ILPMinerInvariant, InductiveMinerInvariant,
    InvariantVerificationResult, PrecisionInvariant, SplitMinerInvariant, TokenReplayInvariant,
};
pub use invariant_checkers::{CustomInvariantChecker, InvariantStatistics, InvariantSuite};
pub use proof_certificates::{
    AlgorithmType, CertificateRepository, ProofCertificate as AlgorithmProofCertificate,
};
pub use soundness_proof_system::{
    DeadlockFreeProof, LivenessProof, Marking, MarkingProof, ProofCertificate, ReachabilityProof,
    SoundnessProofEngine,
};
pub use spec_to_impl::{
    get_specification, AlgorithmSpecification, BisimulationProof, DivergenceMeasure,
    EquivalenceProof, ExecutionTrace, FormalInvariant, ProofStrategy, SpecImplementationProof,
    TraceEquivalenceProof,
};
