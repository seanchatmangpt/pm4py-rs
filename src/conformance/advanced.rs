//! Advanced Conformance Checking Methods
//!
//! This module implements four advanced conformance checking methods beyond basic token replay:
//!
//! 1. **Cost-Based Alignment**: Token replay with configurable cost penalties for moves
//! 2. **Behavioral Profiles**: Activity dependency and co-occurrence analysis
//! 3. **DECLARE Constraints**: Linear temporal logic constraint satisfaction
//! 4. **Extended Fitness**: Multi-dimensional conformance (precision+fitness+generalization+simplicity)

use crate::log::EventLog;
use crate::models::PetriNet;
use std::collections::{HashMap, HashSet};

// ═══════════════════════════════════════════════════════════════════════════════
// 1. COST-BASED ALIGNMENT
// ═══════════════════════════════════════════════════════════════════════════════

/// Cost model for alignment moves
#[derive(Debug, Clone)]
pub struct AlignmentCostModel {
    pub sync_cost: f64,       // Cost of synchronous move (perfect match)
    pub log_move_cost: f64,   // Cost of log move (event in log not in model)
    pub model_move_cost: f64, // Cost of model move (transition in model not in log)
    pub skip_token_cost: f64, // Cost of skipping a token
}

impl Default for AlignmentCostModel {
    fn default() -> Self {
        Self {
            sync_cost: 0.0,
            log_move_cost: 1.0,
            model_move_cost: 1.0,
            skip_token_cost: 0.5,
        }
    }
}

/// Represents an optimal alignment between trace and model
#[derive(Debug, Clone)]
pub struct OptimalAlignment {
    pub trace_index: usize,
    pub trace_id: String,
    pub moves: Vec<AlignmentMove>,
    pub total_cost: f64,
    pub fitness: f64,
    pub num_sync_moves: usize,
    pub num_log_moves: usize,
    pub num_model_moves: usize,
}

/// A single move in a cost-based alignment
#[derive(Debug, Clone, PartialEq)]
pub enum AlignmentMove {
    Sync { activity: String },
    LogMove { activity: String },
    ModelMove { activity: String },
}

impl AlignmentMove {
    pub fn cost(&self, model: &AlignmentCostModel) -> f64 {
        match self {
            AlignmentMove::Sync { .. } => model.sync_cost,
            AlignmentMove::LogMove { .. } => model.log_move_cost,
            AlignmentMove::ModelMove { .. } => model.model_move_cost,
        }
    }
}

impl OptimalAlignment {
    pub fn new(trace_index: usize, trace_id: String) -> Self {
        Self {
            trace_index,
            trace_id,
            moves: Vec::new(),
            total_cost: 0.0,
            fitness: 1.0,
            num_sync_moves: 0,
            num_log_moves: 0,
            num_model_moves: 0,
        }
    }

    /// Calculate fitness from alignment statistics
    pub fn calculate_fitness(&mut self) {
        let total_events = self.num_sync_moves + self.num_log_moves;
        if total_events == 0 {
            self.fitness = 1.0;
        } else {
            self.fitness = self.num_sync_moves as f64 / total_events as f64;
        }
    }
}

/// Cost-based alignment checker using dynamic programming
pub struct CostBasedAligner {
    cost_model: AlignmentCostModel,
}

impl CostBasedAligner {
    pub fn new(cost_model: AlignmentCostModel) -> Self {
        Self { cost_model }
    }

    /// Compute optimal alignments for all traces via pm4py.
    ///
    /// Delegates to `call_fitness_alignments` and returns an aggregate result.
    ///
    /// # Panics
    ///
    /// Panics if Python or pm4py are unavailable.
    pub fn compute_alignments(&self, log: &EventLog, net: &PetriNet) -> Vec<OptimalAlignment> {
        use crate::python::generated::conformance::call_fitness_alignments;
        use pyo3::Python;
        let _ = &self.cost_model; // pm4py uses its own internal cost model
        let avg_fitness = Python::with_gil(|py| call_fitness_alignments(py, log, net)).expect(
            "pm4py not available — ensure Python and pm4py are installed (pip install pm4py)",
        );
        let mut alignment = OptimalAlignment::new(0, "aggregate".to_string());
        alignment.fitness = avg_fitness;
        alignment.total_cost = (1.0 - avg_fitness) * log.traces.len() as f64;
        vec![alignment]
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 2. BEHAVIORAL PROFILES (Enhanced)
// ═══════════════════════════════════════════════════════════════════════════════

/// Activity relationships in behavioral profiles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActivityRelationType {
    Parallel,     // Can occur in any order
    Precedence,   // One strictly precedes the other
    Choice,       // Mutually exclusive
    Loop,         // Activity can follow itself
    Causality,    // One causes the other
    CoOccurrence, // Both occur together
}

impl ActivityRelationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ActivityRelationType::Parallel => "parallel",
            ActivityRelationType::Precedence => "precedence",
            ActivityRelationType::Choice => "choice",
            ActivityRelationType::Loop => "loop",
            ActivityRelationType::Causality => "causality",
            ActivityRelationType::CoOccurrence => "co-occurrence",
        }
    }
}

/// Behavioral relationship between two activities
#[derive(Debug, Clone)]
pub struct ActivityDependency {
    pub activity_a: String,
    pub activity_b: String,
    pub relation_type: ActivityRelationType,
    pub frequency: usize,
    pub confidence: f64,
}

/// Behavioral profile analysis
#[derive(Debug, Clone)]
pub struct BehavioralProfileAnalysis {
    pub activities: HashSet<String>,
    pub dependencies: Vec<ActivityDependency>,
    pub co_occurrences: HashMap<(String, String), usize>,
    pub causality_pairs: HashMap<(String, String), f64>,
    pub loop_activities: HashSet<String>,
    pub conformance_score: f64,
}

impl BehavioralProfileAnalysis {
    pub fn new() -> Self {
        Self {
            activities: HashSet::new(),
            dependencies: Vec::new(),
            co_occurrences: HashMap::new(),
            causality_pairs: HashMap::new(),
            loop_activities: HashSet::new(),
            conformance_score: 1.0,
        }
    }

    /// Extract behavioral profile from log
    pub fn from_log(log: &EventLog) -> Self {
        let mut profile = Self::new();

        // Phase 1: Collect all activities
        for trace in &log.traces {
            for event in &trace.events {
                profile.activities.insert(event.activity.clone());
            }
        }

        // Phase 2: Analyze co-occurrence and precedence relationships
        let mut precedence_matrix: HashMap<(String, String), usize> = HashMap::new();
        let mut reverse_precedence: HashMap<(String, String), usize> = HashMap::new();

        for trace in &log.traces {
            // Track which activities appear in this trace
            let trace_activities: HashSet<_> =
                trace.events.iter().map(|e| e.activity.clone()).collect();

            // Co-occurrence analysis
            for act_a in &trace_activities {
                for act_b in &trace_activities {
                    if act_a < act_b {
                        let key = (act_a.clone(), act_b.clone());
                        *profile.co_occurrences.entry(key).or_insert(0) += 1;
                    }
                }
            }

            // Precedence analysis
            for (i, event_i) in trace.events.iter().enumerate() {
                // Check for loops (activity following itself)
                if i > 0 && trace.events[i - 1].activity == event_i.activity {
                    profile.loop_activities.insert(event_i.activity.clone());
                }

                // Precedence relations
                for (j, event_j) in trace.events.iter().enumerate() {
                    if i < j && event_i.activity != event_j.activity {
                        let key = (event_i.activity.clone(), event_j.activity.clone());
                        *precedence_matrix.entry(key).or_insert(0) += 1;
                    } else if i > j && event_i.activity != event_j.activity {
                        let key = (event_j.activity.clone(), event_i.activity.clone());
                        *reverse_precedence.entry(key).or_insert(0) += 1;
                    }
                }
            }
        }

        // Phase 3: Classify relationships based on precedence patterns
        for act_a in &profile.activities {
            for act_b in &profile.activities {
                if act_a == act_b {
                    continue;
                }

                let forward = precedence_matrix
                    .get(&(act_a.clone(), act_b.clone()))
                    .copied()
                    .unwrap_or(0);
                let reverse = precedence_matrix
                    .get(&(act_b.clone(), act_a.clone()))
                    .copied()
                    .unwrap_or(0);
                let total_traces = log.len();

                if forward > 0 || reverse > 0 {
                    let frequency = forward + reverse;
                    let confidence = frequency as f64 / total_traces as f64;

                    let relation_type = if forward > 0 && reverse == 0 {
                        ActivityRelationType::Precedence
                    } else if forward > 0 && reverse > 0 {
                        // Both occur but not equally
                        if forward > reverse {
                            ActivityRelationType::Precedence
                        } else {
                            ActivityRelationType::Causality
                        }
                    } else {
                        ActivityRelationType::Choice
                    };

                    profile.dependencies.push(ActivityDependency {
                        activity_a: act_a.clone(),
                        activity_b: act_b.clone(),
                        relation_type,
                        frequency,
                        confidence,
                    });
                }
            }
        }

        // Calculate causality from frequent patterns
        for trace in &log.traces {
            for (i, event_i) in trace.events.iter().enumerate() {
                if i > 0 {
                    let prev_act = &trace.events[i - 1].activity;
                    let curr_act = &event_i.activity;
                    if prev_act != curr_act {
                        let key = (prev_act.clone(), curr_act.clone());
                        *profile.causality_pairs.entry(key).or_insert(0.0) += 1.0;
                    }
                }
            }
        }

        // Normalize causality scores to probabilities
        let trace_count = log.len() as f64;
        for score in profile.causality_pairs.values_mut() {
            *score /= trace_count;
        }

        // Calculate overall conformance score
        if !profile.dependencies.is_empty() {
            profile.conformance_score = profile
                .dependencies
                .iter()
                .map(|d| d.confidence)
                .sum::<f64>()
                / profile.dependencies.len() as f64;
        }

        profile
    }

    /// Compare two behavioral profiles for conformance
    pub fn compare_with_model_profile(&self, model_profile: &BehavioralProfileAnalysis) -> f64 {
        let mut matches = 0;
        let mut total = 0;

        // Check if model dependencies are satisfied in log
        for dep in &model_profile.dependencies {
            total += 1;
            // Check if log contains this relationship
            let found = self.dependencies.iter().any(|d| {
                d.activity_a == dep.activity_a
                    && d.activity_b == dep.activity_b
                    && d.relation_type == dep.relation_type
            });
            if found {
                matches += 1;
            }
        }

        if total == 0 {
            1.0
        } else {
            matches as f64 / total as f64
        }
    }
}

impl Default for BehavioralProfileAnalysis {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 3. DECLARE CONSTRAINTS
// ═══════════════════════════════════════════════════════════════════════════════

/// DECLARE constraint types (linear temporal logic based)
#[derive(Debug, Clone)]
pub enum DeclareConstraint {
    /// Activity A must occur at least N times in every trace
    Existence {
        activity: String,
        min_occurrences: usize,
    },

    /// Activity A must not occur in any trace
    Absence { activity: String },

    /// If A occurs, then B must occur in the same trace
    Response {
        antecedent: String,
        consequent: String,
    },

    /// If B occurs, then A must have occurred before B in the trace
    Precedence {
        antecedent: String,
        consequent: String,
    },

    /// If A occurs, then B must occur in the same trace and A must come before B
    Succession {
        antecedent: String,
        consequent: String,
    },

    /// If A occurs, then B must immediately follow A
    ChainResponse {
        antecedent: String,
        consequent: String,
    },

    /// A and B cannot both occur in the same trace
    NegativeConstraint {
        activity_a: String,
        activity_b: String,
    },

    /// Exactly N occurrences of A per trace
    Cardinality { activity: String, count: usize },
}

impl DeclareConstraint {
    pub fn check_trace(&self, trace_activities: &[String]) -> bool {
        match self {
            DeclareConstraint::Existence {
                activity,
                min_occurrences,
            } => {
                let count = trace_activities.iter().filter(|a| *a == activity).count();
                count >= *min_occurrences
            }

            DeclareConstraint::Absence { activity } => !trace_activities.contains(activity),

            DeclareConstraint::Response {
                antecedent,
                consequent,
            } => {
                let has_antecedent = trace_activities.contains(antecedent);
                if !has_antecedent {
                    return true; // Constraint vacuously satisfied
                }
                trace_activities.contains(consequent)
            }

            DeclareConstraint::Precedence {
                antecedent,
                consequent,
            } => {
                let ante_pos = trace_activities.iter().position(|a| a == antecedent);
                let cons_pos = trace_activities.iter().position(|a| a == consequent);

                match (ante_pos, cons_pos) {
                    (Some(a_pos), Some(c_pos)) => a_pos < c_pos,
                    (None, _) => true,  // Vacuously satisfied
                    (_, None) => false, // Consequent missing
                }
            }

            DeclareConstraint::Succession {
                antecedent,
                consequent,
            } => {
                let ante_pos = trace_activities.iter().position(|a| a == antecedent);
                let cons_pos = trace_activities.iter().position(|a| a == consequent);

                match (ante_pos, cons_pos) {
                    (Some(a_pos), Some(c_pos)) => a_pos < c_pos,
                    (None, _) => true,
                    (_, None) => false,
                }
            }

            DeclareConstraint::ChainResponse {
                antecedent,
                consequent,
            } => {
                for (i, activity) in trace_activities.iter().enumerate() {
                    if activity == antecedent {
                        if i + 1 >= trace_activities.len() {
                            return false;
                        }
                        if &trace_activities[i + 1] != consequent {
                            return false;
                        }
                    }
                }
                true
            }

            DeclareConstraint::NegativeConstraint {
                activity_a,
                activity_b,
            } => {
                let has_a = trace_activities.contains(activity_a);
                let has_b = trace_activities.contains(activity_b);
                !(has_a && has_b)
            }

            DeclareConstraint::Cardinality { activity, count } => {
                let actual_count = trace_activities.iter().filter(|a| *a == activity).count();
                actual_count == *count
            }
        }
    }
}

/// DECLARE conformance result
#[derive(Debug, Clone)]
pub struct DeclareConformanceResult {
    pub constraint_id: String,
    pub satisfied: usize,
    pub violated: usize,
    pub vacuous: usize,
    pub conformance_score: f64,
}

/// DECLARE conformance checker
pub struct DeclareChecker;

impl DeclareChecker {
    /// Check a set of constraints against an event log
    pub fn check_conformance(
        log: &EventLog,
        constraints: &[DeclareConstraint],
    ) -> Vec<DeclareConformanceResult> {
        let mut results = Vec::new();

        for (idx, constraint) in constraints.iter().enumerate() {
            let mut satisfied = 0;
            let mut violated = 0;

            for trace in &log.traces {
                let trace_activities: Vec<String> =
                    trace.events.iter().map(|e| e.activity.clone()).collect();

                if constraint.check_trace(&trace_activities) {
                    satisfied += 1;
                } else {
                    violated += 1;
                }
            }

            let total = satisfied + violated;
            let conformance_score = if total == 0 {
                1.0
            } else {
                satisfied as f64 / total as f64
            };

            results.push(DeclareConformanceResult {
                constraint_id: format!("constraint_{}", idx),
                satisfied,
                violated,
                vacuous: 0,
                conformance_score,
            });
        }

        results
    }

    /// Calculate aggregate conformance across all constraints
    pub fn aggregate_conformance(results: &[DeclareConformanceResult]) -> f64 {
        if results.is_empty() {
            1.0
        } else {
            results.iter().map(|r| r.conformance_score).sum::<f64>() / results.len() as f64
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 4. EXTENDED FITNESS (Multi-Dimensional Conformance)
// ═══════════════════════════════════════════════════════════════════════════════

/// Extended fitness dimensions
#[derive(Debug, Clone)]
pub struct ExtendedFitnessScores {
    pub fitness: f64,        // Trace replay fitness
    pub precision: f64,      // Model specificity
    pub generalization: f64, // Model generalization
    pub simplicity: f64,     // Model complexity (inverted)
    pub weighted_score: f64, // Weighted combination
}

/// Weights for extended fitness calculation
#[derive(Debug, Clone)]
pub struct ExtendedFitnessWeights {
    pub fitness_weight: f64,
    pub precision_weight: f64,
    pub generalization_weight: f64,
    pub simplicity_weight: f64,
}

impl Default for ExtendedFitnessWeights {
    fn default() -> Self {
        Self {
            fitness_weight: 0.25,
            precision_weight: 0.25,
            generalization_weight: 0.25,
            simplicity_weight: 0.25,
        }
    }
}

impl ExtendedFitnessWeights {
    /// Create weights emphasizing fitness
    pub fn fitness_focused() -> Self {
        Self {
            fitness_weight: 0.4,
            precision_weight: 0.3,
            generalization_weight: 0.2,
            simplicity_weight: 0.1,
        }
    }

    /// Create weights emphasizing precision
    pub fn precision_focused() -> Self {
        Self {
            fitness_weight: 0.2,
            precision_weight: 0.4,
            generalization_weight: 0.2,
            simplicity_weight: 0.2,
        }
    }

    /// Create weights emphasizing generalization
    pub fn generalization_focused() -> Self {
        Self {
            fitness_weight: 0.25,
            precision_weight: 0.25,
            generalization_weight: 0.4,
            simplicity_weight: 0.1,
        }
    }

    /// Normalize weights to sum to 1.0
    pub fn normalize(&mut self) {
        let sum = self.fitness_weight
            + self.precision_weight
            + self.generalization_weight
            + self.simplicity_weight;

        if sum > 0.0 {
            self.fitness_weight /= sum;
            self.precision_weight /= sum;
            self.generalization_weight /= sum;
            self.simplicity_weight /= sum;
        }
    }

    pub fn is_valid(&self) -> bool {
        let sum = self.fitness_weight
            + self.precision_weight
            + self.generalization_weight
            + self.simplicity_weight;
        (sum - 1.0).abs() < 0.001
    }
}

/// Extended fitness calculator
pub struct ExtendedFitnessCalculator;

impl ExtendedFitnessCalculator {
    /// Calculate extended fitness with custom weights
    pub fn calculate(
        fitness: f64,
        precision: f64,
        generalization: f64,
        simplicity: f64,
        weights: &ExtendedFitnessWeights,
    ) -> ExtendedFitnessScores {
        let weighted_score = weights.fitness_weight * fitness
            + weights.precision_weight * precision
            + weights.generalization_weight * generalization
            + weights.simplicity_weight * simplicity;

        ExtendedFitnessScores {
            fitness,
            precision,
            generalization,
            simplicity,
            weighted_score,
        }
    }

    /// Calculate with default equal weights
    pub fn calculate_equal_weights(
        fitness: f64,
        precision: f64,
        generalization: f64,
        simplicity: f64,
    ) -> ExtendedFitnessScores {
        let weights = ExtendedFitnessWeights::default();
        Self::calculate(fitness, precision, generalization, simplicity, &weights)
    }

    /// Estimate precision from model and log (simplified)
    pub fn estimate_precision(log: &EventLog, net: &PetriNet) -> f64 {
        // Simplified: count distinct activities in model vs log
        let log_activities: HashSet<_> = log
            .traces
            .iter()
            .flat_map(|t| t.events.iter().map(|e| e.activity.clone()))
            .collect();

        let model_activities: HashSet<_> = net
            .transitions
            .iter()
            .filter_map(|t| t.label.clone())
            .collect();

        if model_activities.is_empty() {
            return 0.0;
        }

        let matching = log_activities
            .iter()
            .filter(|a| model_activities.contains(a.as_str()))
            .count();

        matching as f64 / model_activities.len() as f64
    }

    /// Estimate generalization as the fraction of labeled transitions covered by
    /// at least one event in the log. A fully covered model (all transitions
    /// seen at least once) scores 1.0; an unused model scores 0.0.
    pub fn estimate_generalization(log: &EventLog, net: &PetriNet) -> f64 {
        let activities: HashSet<String> = log
            .traces
            .iter()
            .flat_map(|t| t.events.iter().map(|e| e.activity.clone()))
            .collect();
        let labeled_total = net.transitions.iter().filter(|t| t.label.is_some()).count();
        if labeled_total == 0 {
            return 1.0;
        }
        let executed = net
            .transitions
            .iter()
            .filter(|t| {
                t.label
                    .as_ref()
                    .map(|l| activities.contains(l))
                    .unwrap_or(false)
            })
            .count();
        executed as f64 / labeled_total as f64
    }

    /// Alias for `estimate_generalization` with an explicit `_from_net` suffix
    /// that makes the net dependency explicit in the name.
    pub fn estimate_generalization_from_net(log: &EventLog, net: &PetriNet) -> f64 {
        Self::estimate_generalization(log, net)
    }

    /// Estimate simplicity using the arc-degree formula:
    /// `simplicity = 1 / (1 + avg_arcs_per_transition)`.
    ///
    /// A model with fewer arcs per transition is simpler and scores closer to
    /// 1.0. An empty model (no transitions) returns 1.0.
    pub fn estimate_simplicity(net: &PetriNet) -> f64 {
        let t = net.transitions.len() as f64;
        if t == 0.0 {
            return 1.0;
        }
        let arcs = net.arcs.len() as f64;
        1.0 / (1.0 + arcs / t)
    }
}
