pub mod advanced;
pub mod alignment;
pub mod alignment_variants;
pub mod alignments;
pub mod behavioral_profile;
pub mod footprints;
pub mod four_spectrum;
pub mod generalization;
pub mod ocdfg_conformance;
pub mod ocel_conformance;
pub mod precision;
pub mod simplicity;
pub mod soundness_checker;
/// Conformance checking algorithms and quality metrics
///
/// This module provides comprehensive conformance checking and process model
/// quality assessment capabilities:
///
/// # Conformance Checking
///
/// Compares observed behavior (event log) with modeled behavior (process model).
/// Key metrics:
///
/// - **Fitness**: How well the model explains the log (0.0 = no fit, 1.0 = perfect fit)
/// - **Precision**: How specific the model is (0.0 = too general, 1.0 = perfectly specific)
/// - **Generalization**: How well the model generalizes to unseen behavior
/// - **Simplicity**: How simple/reasonable the model is
///
/// # Algorithms
///
/// ## Token Replay
///
/// The simplest conformance checking technique. Tokens are placed in the initial place
/// and propagated through the net. Missing tokens and remaining tokens measure deviations.
///
/// ```rust
/// use pm4py::{TokenReplay, ConformanceChecker, EventLog, PetriNet};
///
/// let event_log = EventLog::new();
/// let model = PetriNet::new();
/// let checker = TokenReplay::new();
/// let result = checker.check(&event_log, &model);
/// println!("Fitness: {}", result.fitness);
/// ```
///
/// ## Alignments
///
/// Computes optimal alignments between log traces and model paths. Provides detailed
/// diagnostics on where deviations occur.
///
/// ```rust,no_run
/// use pm4py::{AlignmentChecker, ConformanceChecker, EventLog, PetriNet};
///
/// let event_log = EventLog::new();
/// let model = PetriNet::new();
/// let checker = AlignmentChecker::new();
/// let result = checker.check(&event_log, &model);
/// // Result contains detailed alignment moves
/// ```
///
/// ## Footprints
///
/// Fast conformance checking based on activity relations (directly follows, eventually follows).
/// Much faster than alignments but less detailed.
///
/// ## Four-Spectrum Metric
///
/// Combines all four quality dimensions into a single score:
///
/// ```rust,ignore
/// use pm4py::conformance::{FourSpectrum, SpectrumWeights};
/// use pm4py::{EventLog, PetriNet};
///
/// let event_log = EventLog::new();
/// let model = PetriNet::new();
/// let weights = SpectrumWeights {
///     fitness: 0.4,
///     precision: 0.3,
///     generalization: 0.2,
///     simplicity: 0.1,
/// };
///
/// let spectrum = FourSpectrum::new(weights);
/// let result = spectrum.calculate(&event_log, &model);
/// println!("Overall quality: {}", result.overall_score);
/// ```
///
/// ## Soundness Checking
///
/// Formal verification that a Petri net is sound (deadlock-free, proper completion, liveness).
///
/// ```rust,no_run
/// use pm4py::conformance::SoundnessChecker;
/// use pm4py::PetriNet;
///
/// let model = PetriNet::new();
/// let checker = SoundnessChecker::new(model);
/// let proof = checker.check();
/// // proof.is_sound indicates whether the net is sound
/// println!("Is sound: {}", proof.is_sound);
/// ```
///
/// # Quality Dimensions
///
/// ## Fitness (0.0 - 1.0)
/// Measures how well the model explains the observed behavior.
/// - 1.0: All traces fit perfectly
/// - 0.0: No traces fit
///
/// ## Precision (0.0 - 1.0)
/// Measures how specific the model is (does it allow too much behavior?).
/// - 1.0: Model only allows observed behavior
/// - 0.0: Model allows any behavior (flower model)
///
/// ## Generalization (0.0 - 1.0)
/// Measures how well the model will fit future, unseen behavior.
/// - 1.0: Model generalizes well
/// - 0.0: Model overfits to this specific log
///
/// ## Simplicity (0.0 - 1.0)
/// Measures how simple/reasonable the model is.
/// - 1.0: Very simple model
/// - 0.0: Overly complex model
///
/// # Advanced Features
///
/// - **Behavioral Profiles**: Analyze activity ordering and relationships
/// - **DECLARE Constraints**: Check declarative constraints
/// - **Temporal Profiles**: Check timing constraints
/// - **OCEL Conformance**: Object-centric conformance checking
pub mod token_replay;
pub mod token_replay_advanced;

pub use advanced::{
    ActivityDependency, ActivityRelationType, AlignmentCostModel,
    AlignmentMove as AdvancedAlignmentMove, BehavioralProfileAnalysis, CostBasedAligner,
    DeclareChecker, DeclareConformanceResult, DeclareConstraint, ExtendedFitnessCalculator,
    ExtendedFitnessScores, ExtendedFitnessWeights, OptimalAlignment,
};
pub use alignment::AlignmentChecker;
pub use alignment_variants::{
    AStarAligner, AlignmentMove, BeamSearchAligner, OptimalAlignment as OptimalAlignmentVariant,
    StreamingAligner,
};
pub use alignments::{
    conformance_alignments, diagnostics_alignments, fitness_alignments, get_alignment_costs,
    get_num_deviations, precision_alignments, AlignmentMove as NewAlignmentMove, AlignmentResult,
    TraceAlignment,
};
pub use behavioral_profile::{ActivityRelation, BehavioralProfile, ConflictInfo};
pub use footprints::{
    diagnostics_footprints, fitness_footprints, precision_footprints, FootprintsConformanceChecker,
    FootprintsConformanceResult,
};
pub use four_spectrum::{FourSpectrum, SpectrumResult, SpectrumWeights};
pub use generalization::Generalization;
pub use ocdfg_conformance::*;
pub use ocel_conformance::*;
pub use precision::Precision;
pub use simplicity::{ComplexityBreakdown, Simplicity};
pub use soundness_checker::{SoundnessChecker, SoundnessProof, SoundnessViolation};
pub use token_replay::{diagnostics_token_based_replay, precision_token_based_replay, TokenReplay};
pub use token_replay_advanced::{
    AdvancedTokenReplayResult, HeuristicTokenAllocator, WeightedTokenReplay,
};

use crate::log::EventLog;
use crate::models::PetriNet;

/// Result of conformance checking
///
/// Contains the four key quality dimensions for process model assessment.
///
/// # Fields
///
/// - `is_conformant`: Whether the log conforms to the model (fitness >= threshold)
/// - `fitness`: How well the model explains the log (0.0 - 1.0)
/// - `precision`: How specific the model is (0.0 - 1.0)
/// - `generalization`: How well the model generalizes (0.0 - 1.0)
///
/// # Example
///
/// ```rust
/// use pm4py::{TokenReplay, ConformanceChecker, EventLog, PetriNet};
///
/// let event_log = EventLog::new();
/// let model = PetriNet::new();
/// let checker = TokenReplay::new();
/// let result = checker.check(&event_log, &model);
///
/// if result.is_conformant {
///     println!("Model conforms! Fitness: {:.2}", result.fitness);
/// } else {
///     println!("Model does not conform. Fitness: {:.2}", result.fitness);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ConformanceResult {
    /// Whether the log conforms to the model (typically fitness >= 0.8)
    pub is_conformant: bool,
    /// How well the model explains the observed behavior (0.0 - 1.0)
    pub fitness: f64,
    /// How specific the model is (0.0 = too general, 1.0 = perfectly specific)
    pub precision: f64,
    /// How well the model generalizes to unseen behavior (0.0 - 1.0)
    pub generalization: f64,
}

/// Trait for conformance checking algorithms
///
/// All conformance checkers implement this trait, providing a unified interface
/// for comparing event logs with process models.
///
/// # Example
///
/// ```rust
/// use pm4py::{ConformanceChecker, TokenReplay, AlignmentChecker, EventLog, PetriNet};
///
/// fn check_conformance(event_log: &EventLog, model: &PetriNet) {
///     // Simple token replay
///     let token_checker = TokenReplay::new();
///     let result1 = token_checker.check(event_log, model);
///
///     // Detailed alignment-based checking
///     let align_checker = AlignmentChecker::new();
///     let result2 = align_checker.check(event_log, model);
/// }
/// ```
///
/// # Implementations
///
/// - [`TokenReplay`]: Fast, simple fitness checking
/// - [`AlignmentChecker`]: Detailed alignment-based checking
/// - [`FootprintsConformanceChecker`]: Fast footprint-based checking
pub trait ConformanceChecker {
    /// Check conformance between a log and a model
    ///
    /// # Parameters
    ///
    /// - `log`: The event log containing observed behavior
    /// - `net`: The process model to check against
    ///
    /// # Returns
    ///
    /// A [`ConformanceResult`] containing fitness, precision, and generalization scores
    ///
    /// # Example
    ///
    /// ```rust
    /// use pm4py::{ConformanceChecker, TokenReplay, EventLog, PetriNet};
    ///
    /// let event_log = EventLog::new();
    /// let model = PetriNet::new();
    /// let checker = TokenReplay::new();
    /// let result = checker.check(&event_log, &model);
    /// assert!(result.fitness >= 0.0, "Model fitness should be non-negative");
    /// ```
    fn check(&self, log: &EventLog, net: &PetriNet) -> ConformanceResult;
}
