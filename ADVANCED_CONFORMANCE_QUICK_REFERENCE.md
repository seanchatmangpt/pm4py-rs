# Advanced Conformance Checking - Quick Reference

## At a Glance

| Method | Purpose | Key Type | Main API |
|--------|---------|----------|----------|
| **Cost-Based Alignment** | DP-based alignment with configurable costs | `CostBasedAligner` | `compute_alignments(log, net)` |
| **Behavioral Profiles** | Activity dependency & co-occurrence analysis | `BehavioralProfileAnalysis` | `from_log(log)` |
| **DECLARE Constraints** | Linear temporal logic constraint checking | `DeclareChecker` | `check_conformance(log, constraints)` |
| **Extended Fitness** | Multi-dimensional fitness (F,P,G,S) | `ExtendedFitnessCalculator` | `calculate(f,p,g,s,weights)` |

---

## Usage Examples

### 1. Cost-Based Alignment

```rust
use pm4py::conformance::{AlignmentCostModel, CostBasedAligner};

// Create custom cost model
let cost_model = AlignmentCostModel {
    sync_cost: 0.0,
    log_move_cost: 1.0,
    model_move_cost: 1.0,
    skip_token_cost: 0.5,
};

// Compute alignments
let aligner = CostBasedAligner::new(cost_model);
let alignments = aligner.compute_alignments(&log, &net);

// Analyze results
for alignment in &alignments {
    println!("Trace: {}, Fitness: {:.2}, Cost: {:.2}",
        alignment.trace_id, alignment.fitness, alignment.total_cost);
}
```

### 2. Behavioral Profiles

```rust
use pm4py::conformance::BehavioralProfileAnalysis;

// Extract profile from log
let profile = BehavioralProfileAnalysis::from_log(&log);

// Inspect activities
println!("Activities: {:?}", profile.activities);

// Inspect dependencies
for dep in &profile.dependencies {
    println!("{} -[{}]-> {} (confidence: {:.2})",
        dep.activity_a, dep.relation_type.as_str(),
        dep.activity_b, dep.confidence);
}

// Compare with model profile
let model_profile = BehavioralProfileAnalysis::from_log(&model_log);
let similarity = profile.compare_with_model_profile(&model_profile);
println!("Profile similarity: {:.2}", similarity);
```

### 3. DECLARE Constraints

```rust
use pm4py::conformance::{DeclareConstraint, DeclareChecker};

// Define constraints
let constraints = vec![
    DeclareConstraint::Existence {
        activity: "start".to_string(),
        min_occurrences: 1,
    },
    DeclareConstraint::Response {
        antecedent: "start".to_string(),
        consequent: "end".to_string(),
    },
    DeclareConstraint::Absence {
        activity: "error".to_string(),
    },
];

// Check conformance
let results = DeclareChecker::check_conformance(&log, &constraints);

// Aggregate score
let aggregate = DeclareChecker::aggregate_conformance(&results);
println!("Conformance: {:.2}%", aggregate * 100.0);

// Per-constraint results
for result in &results {
    println!("{}: {}/{} satisfied",
        result.constraint_id, result.satisfied,
        result.satisfied + result.violated);
}
```

### 4. Extended Fitness

```rust
use pm4py::conformance::{
    ExtendedFitnessCalculator, ExtendedFitnessWeights
};

// Get fitness metrics
let fitness = 0.9;
let precision = 0.85;
let generalization = 0.8;
let simplicity = 0.7;

// Calculate with equal weights
let scores = ExtendedFitnessCalculator::calculate_equal_weights(
    fitness, precision, generalization, simplicity
);
println!("Equal weighted: {:.3}", scores.weighted_score);

// Calculate with custom weights
let mut weights = ExtendedFitnessWeights::fitness_focused();
weights.normalize();
let scores = ExtendedFitnessCalculator::calculate(
    fitness, precision, generalization, simplicity, &weights
);
println!("Fitness-focused: {:.3}", scores.weighted_score);

// Estimate from model & log
let est_precision = ExtendedFitnessCalculator::estimate_precision(&log, &net);
let est_gen = ExtendedFitnessCalculator::estimate_generalization(&log);
let est_simp = ExtendedFitnessCalculator::estimate_simplicity(&net);
```

---

## Common Patterns

### Pattern: Conformance Assessment

```rust
// Step 1: Cost-based alignment for fitness
let aligner = CostBasedAligner::new(AlignmentCostModel::default());
let alignments = aligner.compute_alignments(&log, &net);
let fitness: f64 = alignments.iter()
    .map(|a| a.fitness)
    .sum::<f64>() / alignments.len() as f64;

// Step 2: Behavioral profile for structural conformance
let profile = BehavioralProfileAnalysis::from_log(&log);
let profile_score = profile.conformance_score;

// Step 3: DECLARE constraints for process rules
let constraints = vec![...]; // Define constraints
let results = DeclareChecker::check_conformance(&log, &constraints);
let declare_score = DeclareChecker::aggregate_conformance(&results);

// Step 4: Extended fitness for overall quality
let scores = ExtendedFitnessCalculator::calculate_equal_weights(
    fitness, profile_score, 0.8, 0.7
);
println!("Overall quality: {:.3}", scores.weighted_score);
```

### Pattern: Violation Detection

```rust
// Find problematic traces
let aligner = CostBasedAligner::new(AlignmentCostModel::default());
let alignments = aligner.compute_alignments(&log, &net);

let problematic: Vec<_> = alignments
    .iter()
    .filter(|a| a.fitness < 0.5)
    .collect();

for trace in problematic {
    println!("Trace {} has {} deviations",
        trace.trace_id, trace.num_log_moves);

    for move_item in &trace.moves {
        println!("  {:?}", move_item);
    }
}
```

### Pattern: Profile Comparison

```rust
let log_profile = BehavioralProfileAnalysis::from_log(&actual_log);
let model_profile = BehavioralProfileAnalysis::from_log(&model_log);

let similarity = log_profile.compare_with_model_profile(&model_profile);

if similarity > 0.8 {
    println!("Log follows model closely");
} else if similarity > 0.5 {
    println!("Moderate deviations from model");
} else {
    println!("Significant model drift detected");

    // Find unmodeled activities
    for activity in &log_profile.activities {
        if !model_profile.activities.contains(activity) {
            println!("  Unmodeled activity: {}", activity);
        }
    }
}
```

---

## Test Fixtures Available

```rust
use conformance_advanced_test::*;

fn create_simple_petri_net() -> PetriNet
fn create_perfect_fit_log() -> EventLog      // A→B→C sequences
fn create_partial_fit_log() -> EventLog      // Mixed conformance
fn create_looping_log() -> EventLog          // B→B loops
fn create_parallel_activities_log() -> EventLog  // Multiple orderings
```

---

## Type Quick Reference

### Cost-Based Alignment
```rust
pub struct AlignmentCostModel {
    pub sync_cost: f64,
    pub log_move_cost: f64,
    pub model_move_cost: f64,
    pub skip_token_cost: f64,
}

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

pub enum AlignmentMove {
    Sync { activity: String },
    LogMove { activity: String },
    ModelMove { activity: String },
}
```

### Behavioral Profiles
```rust
pub enum ActivityRelationType {
    Parallel, Precedence, Choice, Loop, Causality, CoOccurrence,
}

pub struct ActivityDependency {
    pub activity_a: String,
    pub activity_b: String,
    pub relation_type: ActivityRelationType,
    pub frequency: usize,
    pub confidence: f64,
}

pub struct BehavioralProfileAnalysis {
    pub activities: HashSet<String>,
    pub dependencies: Vec<ActivityDependency>,
    pub co_occurrences: HashMap<(String, String), usize>,
    pub causality_pairs: HashMap<(String, String), f64>,
    pub loop_activities: HashSet<String>,
    pub conformance_score: f64,
}
```

### DECLARE Constraints
```rust
pub enum DeclareConstraint {
    Existence { activity, min_occurrences },
    Absence { activity },
    Response { antecedent, consequent },
    Precedence { antecedent, consequent },
    Succession { antecedent, consequent },
    ChainResponse { antecedent, consequent },
    NegativeConstraint { activity_a, activity_b },
    Cardinality { activity, count },
}

pub struct DeclareConformanceResult {
    pub constraint_id: String,
    pub satisfied: usize,
    pub violated: usize,
    pub vacuous: usize,
    pub conformance_score: f64,
}
```

### Extended Fitness
```rust
pub struct ExtendedFitnessScores {
    pub fitness: f64,
    pub precision: f64,
    pub generalization: f64,
    pub simplicity: f64,
    pub weighted_score: f64,
}

pub struct ExtendedFitnessWeights {
    pub fitness_weight: f64,
    pub precision_weight: f64,
    pub generalization_weight: f64,
    pub simplicity_weight: f64,
}
```

---

## Computational Complexity

| Method | Time | Space |
|--------|------|-------|
| Cost-Based Alignment | O(n × m) | O(n × m) |
| Behavioral Profiles | O(t × e²) | O(a²) |
| DECLARE Constraints | O(c × t × e) | O(c) |
| Extended Fitness | O(1) | O(1) |

Where: n=traces, m=max_length, t=traces, e=events, a=activities, c=constraints

---

## Mathematical Formulas

### Cost-Based Alignment Fitness
```
fitness = sync_moves / (sync_moves + log_moves)
total_cost = Σ cost(move_i)
```

### Behavioral Profile Confidence
```
confidence = frequency / total_traces
co_occurrence = activities appearing in same trace
causality = P(activity_b | activity_a immediately follows)
```

### DECLARE Conformance
```
conformance_score = satisfied / (satisfied + violated)
aggregate = Σ(scores) / count
```

### Extended Fitness
```
weighted_score = w_f×F + w_p×P + w_g×G + w_s×S
where Σ(w_i) = 1.0
```

---

## Common Issues & Solutions

### Issue: Low Cost-Based Alignment Fitness
- **Cause**: Log contains activities not in model
- **Solution**: Check model covers all observed activities

### Issue: High Dependencies in Behavioral Profile
- **Cause**: Strict ordering in log
- **Solution**: Verify log doesn't contain noise/errors

### Issue: Failed DECLARE Constraints
- **Cause**: Process deviates from expected constraints
- **Solution**: Revise constraints or investigate violations

### Issue: Low Extended Fitness Score
- **Cause**: Poor model quality in multiple dimensions
- **Solution**: Focus on dimension with lowest score

---

## Performance Tips

1. **Batch Processing**: Process multiple alignments together
2. **Reuse Profiles**: Extract profile once, compare multiple times
3. **Constraint Filtering**: Check cheap constraints first
4. **Parallel Traces**: Use rayon for large log processing
5. **Incremental Update**: For streaming, incrementally update profiles

---

## References

- Implementation: `/Users/sac/chatmangpt/pm4py-rust/src/conformance/advanced.rs`
- Tests: `/Users/sac/chatmangpt/pm4py-rust/tests/conformance_advanced_test.rs`
- Full Docs: `/Users/sac/chatmangpt/pm4py-rust/ADVANCED_CONFORMANCE_IMPLEMENTATION.md`
