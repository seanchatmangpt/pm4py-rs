# Advanced Topics

**Specialized techniques and extensions for PM4Py Rust**

---

## Table of Contents

1. [Custom Discovery Algorithms](#custom-discovery-algorithms)
2. [Advanced Conformance Checking](#advanced-conformance-checking)
3. [Building Domain-Specific Plugins](#building-domain-specific-plugins)
4. [Distributed Process Mining](#distributed-process-mining)
5. [Real-Time Streaming Mode](#real-time-streaming-mode)
6. [ML Integration](#ml-integration)

---

## Custom Discovery Algorithms

### Creating a Custom Miner

```rust
use pm4py::log::EventLog;
use pm4py::models::petri_net::{PetriNet, Place, Transition, Arc};

pub trait DiscoveryAlgorithm {
    fn discover(&self, log: &EventLog) -> PetriNet;
}

pub struct CustomMiner {
    threshold: f64,
}

impl CustomMiner {
    pub fn new() -> Self {
        Self { threshold: 0.5 }
    }

    pub fn with_threshold(threshold: f64) -> Self {
        Self { threshold }
    }
}

impl DiscoveryAlgorithm for CustomMiner {
    fn discover(&self, log: &EventLog) -> PetriNet {
        let mut net = PetriNet::new();

        // Step 1: Analyze direct follows
        let follows = self.extract_follows(log);

        // Step 2: Filter by threshold
        let significant = self.filter_by_frequency(&follows);

        // Step 3: Build Petri Net
        for (source, target) in significant {
            let src_place = net.add_place(Place::new(&source));
            let trans = net.add_transition(Transition::new(&target));
            let tgt_place = net.add_place(Place::new(&target));

            net.add_arc(Arc::new(src_place, trans, 1));
            net.add_arc(Arc::new(trans, tgt_place, 1));
        }

        net
    }
}

impl CustomMiner {
    fn extract_follows(&self, log: &EventLog) -> Vec<(String, String)> {
        let mut follows = Vec::new();

        for trace in log.traces() {
            for i in 0..trace.events.len() - 1 {
                let source = &trace.events[i].name;
                let target = &trace.events[i + 1].name;
                follows.push((source.clone(), target.clone()));
            }
        }

        follows
    }

    fn filter_by_frequency(&self, follows: &[(String, String)]) -> Vec<(String, String)> {
        use std::collections::HashMap;

        let mut counts = HashMap::new();
        for pair in follows {
            *counts.entry(pair.clone()).or_insert(0) += 1;
        }

        let threshold_count = (follows.len() as f64 * self.threshold) as usize;

        counts.into_iter()
            .filter(|(_, count)| *count >= threshold_count)
            .map(|(pair, _)| pair)
            .collect()
    }
}

// Usage
fn main() {
    let log = create_sample_log();
    let miner = CustomMiner::with_threshold(0.7);
    let net = miner.discover(&log);
    println!("Custom discovered: {} transitions", net.transitions.len());
}
```

### Advanced: ML-Driven Discovery

```rust
use pm4py::log::EventLog;
use pm4py::models::petri_net::PetriNet;

pub struct MLDrivenMiner {
    feature_weights: Vec<f64>,
}

impl MLDrivenMiner {
    pub fn new(weights: Vec<f64>) -> Self {
        Self { feature_weights: weights }
    }

    pub fn discover(&self, log: &EventLog) -> PetriNet {
        // Extract features from log
        let features = self.extract_features(log);

        // Score each possible connection
        let mut scores = Vec::new();
        for i in 0..log.statistics().num_activities {
            for j in 0..log.statistics().num_activities {
                let score = self.score_connection(i, j, &features);
                scores.push((i, j, score));
            }
        }

        // Build model from high-scoring connections
        let mut net = PetriNet::new();
        scores.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

        // Keep top 20% connections
        let cutoff = scores.len() / 5;
        for (src, tgt, _) in &scores[..cutoff] {
            // Add to model...
        }

        net
    }

    fn extract_features(&self, log: &EventLog) -> Vec<Vec<f64>> {
        // Example features:
        // - frequency of transition
        // - temporal distance
        // - co-occurrence in same trace
        // - entropy of following activities
        vec![]
    }

    fn score_connection(&self, src: usize, tgt: usize, features: &[Vec<f64>]) -> f64 {
        // Weighted combination of features
        features[src].iter()
            .zip(&self.feature_weights)
            .map(|(f, w)| f * w)
            .sum()
    }
}
```

---

## Advanced Conformance Checking

### Cost-Based Alignment

```rust
use pm4py::conformance::ConformanceChecker;
use pm4py::models::petri_net::PetriNet;
use pm4py::log::EventLog;

pub struct CostBasedAlignment {
    move_cost: f64,
    skip_cost: f64,
    sync_cost: f64,
}

impl CostBasedAlignment {
    pub fn new() -> Self {
        Self {
            move_cost: 1.0,      // Cost of activity in log but not model
            skip_cost: 2.0,      // Cost of activity in model but not log
            sync_cost: 0.0,      // Cost of perfect match
        }
    }

    pub fn check(&self, log: &EventLog, model: &PetriNet) -> Vec<AlignmentResult> {
        log.traces()
            .iter()
            .map(|trace| self.align_trace(trace, model))
            .collect()
    }

    fn align_trace(&self, trace: &Trace, model: &PetriNet) -> AlignmentResult {
        // Use Dijkstra's algorithm to find minimum-cost path
        use std::collections::BinaryHeap;

        let mut cost = 0.0;
        let mut alignment = Vec::new();

        // Simple alignment: greedy matching
        let mut model_pos = 0;
        let model_activities = model.get_activities();

        for event in &trace.events {
            if model_pos < model_activities.len() {
                if model_activities[model_pos] == event.name {
                    cost += self.sync_cost;
                    alignment.push((Some(&event.name), Some(&model_activities[model_pos])));
                    model_pos += 1;
                } else {
                    cost += self.move_cost;
                    alignment.push((Some(&event.name), None));
                }
            } else {
                cost += self.move_cost;
                alignment.push((Some(&event.name), None));
            }
        }

        while model_pos < model_activities.len() {
            cost += self.skip_cost;
            alignment.push((None, Some(&model_activities[model_pos])));
            model_pos += 1;
        }

        AlignmentResult {
            trace_id: "".to_string(),
            cost,
            alignment,
            fitness: 1.0 - (cost / (trace.events.len() as f64)).min(1.0),
        }
    }
}

#[derive(Debug)]
pub struct AlignmentResult {
    pub trace_id: String,
    pub cost: f64,
    pub alignment: Vec<(Option<&'static str>, Option<&'static str>)>,
    pub fitness: f64,
}
```

### Pattern-Based Conformance

```rust
pub struct PatternConformance {
    patterns: Vec<Pattern>,
}

pub struct Pattern {
    name: String,
    sequence: Vec<String>,
    frequency_threshold: f64,
}

impl PatternConformance {
    pub fn new() -> Self {
        Self {
            patterns: vec![
                Pattern {
                    name: "approval_flow".to_string(),
                    sequence: vec!["Request".to_string(), "Review".to_string(), "Approve".to_string()],
                    frequency_threshold: 0.8,
                },
                Pattern {
                    name: "rejection_flow".to_string(),
                    sequence: vec!["Request".to_string(), "Review".to_string(), "Reject".to_string()],
                    frequency_threshold: 0.1,
                },
            ],
        }
    }

    pub fn check(&self, log: &EventLog) -> PatternResults {
        let mut results = PatternResults::new();

        for pattern in &self.patterns {
            let matches = self.count_pattern_matches(log, pattern);
            let frequency = matches as f64 / log.traces().len() as f64;

            let conforms = frequency >= pattern.frequency_threshold;
            results.add_result(pattern.name.clone(), frequency, conforms);
        }

        results
    }

    fn count_pattern_matches(&self, log: &EventLog, pattern: &Pattern) -> usize {
        let mut count = 0;

        for trace in log.traces() {
            let activities: Vec<_> = trace.events.iter().map(|e| &e.name).collect();

            // Simple subsequence matching
            if self.contains_subsequence(&activities, &pattern.sequence) {
                count += 1;
            }
        }

        count
    }

    fn contains_subsequence(&self, sequence: &[&String], pattern: &[String]) -> bool {
        let mut pattern_idx = 0;

        for item in sequence {
            if pattern_idx < pattern.len() && item == &&pattern[pattern_idx] {
                pattern_idx += 1;
            }
        }

        pattern_idx == pattern.len()
    }
}

pub struct PatternResults {
    results: Vec<(String, f64, bool)>,
}

impl PatternResults {
    fn new() -> Self {
        Self { results: vec![] }
    }

    fn add_result(&mut self, name: String, frequency: f64, conforms: bool) {
        self.results.push((name, frequency, conforms));
    }
}
```

---

## Building Domain-Specific Plugins

### Healthcare Domain Plugin

```rust
use pm4py::log::EventLog;

pub struct HealthcarePlugin;

impl HealthcarePlugin {
    /// Check compliance with HIPAA workflow rules
    pub fn check_hipaa_compliance(log: &EventLog) -> ComplianceReport {
        let mut violations = Vec::new();

        // Rule 1: Patient lookup must be followed by consent
        for trace in log.traces() {
            let mut saw_lookup = false;
            let mut saw_consent = false;

            for event in &trace.events {
                if event.name == "PatientLookup" {
                    saw_lookup = true;
                    saw_consent = false;
                }
                if event.name == "ObtainConsent" {
                    saw_consent = true;
                }
            }

            if saw_lookup && !saw_consent {
                violations.push(format!(
                    "Trace {}: PatientLookup without consent",
                    trace.id
                ));
            }
        }

        ComplianceReport { violations }
    }

    /// Calculate clinical KPIs
    pub fn calculate_clinical_kpis(log: &EventLog) -> ClinicalMetrics {
        let mut total_waiting_time = 0.0;
        let mut admission_count = 0;

        for trace in log.traces() {
            // Find "Admission" and "FirstConsult"
            let admit_time = trace.events.iter()
                .find(|e| e.name == "Admission")
                .map(|e| e.timestamp);
            let consult_time = trace.events.iter()
                .find(|e| e.name == "FirstConsult")
                .map(|e| e.timestamp);

            if let (Some(admit), Some(consult)) = (admit_time, consult_time) {
                let waiting = (consult - admit).num_hours();
                total_waiting_time += waiting;
                admission_count += 1;
            }
        }

        ClinicalMetrics {
            avg_wait_hours: total_waiting_time / admission_count as f64,
            total_admissions: admission_count,
        }
    }
}

pub struct ComplianceReport {
    pub violations: Vec<String>,
}

pub struct ClinicalMetrics {
    pub avg_wait_hours: f64,
    pub total_admissions: usize,
}
```

### Finance Domain Plugin

```rust
pub struct FinancePlugin;

impl FinancePlugin {
    /// Detect potential fraud patterns
    pub fn detect_fraud_patterns(log: &EventLog) -> Vec<FraudAlert> {
        let mut alerts = Vec::new();

        for trace in log.traces() {
            // Pattern 1: High value transaction without approval
            let high_value = trace.events.iter()
                .any(|e| e.name.contains("HighValue") && e.name.contains("Transaction"));

            let has_approval = trace.events.iter()
                .any(|e| e.name == "ManagerApproval");

            if high_value && !has_approval {
                alerts.push(FraudAlert {
                    trace_id: trace.id.clone(),
                    pattern: "HighValueWithoutApproval".to_string(),
                    risk_score: 0.8,
                });
            }

            // Pattern 2: Multiple transactions in quick succession
            let transaction_count = trace.events.iter()
                .filter(|e| e.name.contains("Transaction"))
                .count();

            if transaction_count > 5 {
                alerts.push(FraudAlert {
                    trace_id: trace.id.clone(),
                    pattern: "RapidMultipleTransactions".to_string(),
                    risk_score: 0.6,
                });
            }
        }

        alerts
    }

    /// Calculate financial metrics
    pub fn calculate_financial_metrics(log: &EventLog) -> FinancialMetrics {
        // Implementation...
        FinancialMetrics {
            total_volume: 0.0,
            avg_transaction_size: 0.0,
            processing_cost_per_transaction: 0.0,
        }
    }
}

pub struct FraudAlert {
    pub trace_id: String,
    pub pattern: String,
    pub risk_score: f64,
}

pub struct FinancialMetrics {
    pub total_volume: f64,
    pub avg_transaction_size: f64,
    pub processing_cost_per_transaction: f64,
}
```

---

## Distributed Process Mining

### Map-Reduce Pattern

```rust
use rayon::prelude::*;
use std::collections::HashMap;

pub struct DistributedMiner {
    num_partitions: usize,
}

impl DistributedMiner {
    pub fn discover_distributed(&self, log: &EventLog) -> PetriNet {
        // 1. MAP: Partition log
        let partitions = self.partition_log(log);

        // 2. Discover on each partition
        let local_models: Vec<_> = partitions
            .par_iter()
            .map(|partition| {
                AlphaMiner::new().discover(partition)
            })
            .collect();

        // 3. REDUCE: Merge models
        self.merge_models(local_models)
    }

    fn partition_log(&self, log: &EventLog) -> Vec<EventLog> {
        let partition_size = (log.traces().len() + self.num_partitions - 1)
            / self.num_partitions;

        log.traces()
            .chunks(partition_size)
            .map(|chunk| EventLog::from_traces(chunk.to_vec()))
            .collect()
    }

    fn merge_models(&self, models: Vec<PetriNet>) -> PetriNet {
        let mut merged = PetriNet::new();

        // Merge all transitions and places
        for model in models {
            for place in model.places {
                merged.add_place(place);
            }
            for transition in model.transitions {
                merged.add_transition(transition);
            }
            for arc in model.arcs {
                merged.add_arc(arc);
            }
        }

        // Remove duplicates and normalize
        merged.normalize();
        merged
    }
}
```

---

## Real-Time Streaming Mode

```rust
use tokio::sync::mpsc;
use std::time::Duration;

pub struct StreamingMiner {
    window_size: usize,
    update_interval: Duration,
}

impl StreamingMiner {
    pub fn new(window_size: usize) -> Self {
        Self {
            window_size,
            update_interval: Duration::from_secs(60),
        }
    }

    pub async fn process_stream(
        &self,
        mut rx: mpsc::Receiver<Event>,
    ) -> mpsc::Receiver<StreamingResult> {
        let (tx, output_rx) = mpsc::channel(10);
        let window_size = self.window_size;

        tokio::spawn(async move {
            let mut window = Vec::new();

            while let Some(event) = rx.recv().await {
                window.push(event);

                // When window is full, discover model
                if window.len() >= window_size {
                    let log = EventLog::from_events(window.drain(..));
                    let model = AlphaMiner::new().discover(&log);

                    let result = StreamingResult {
                        timestamp: chrono::Utc::now(),
                        model,
                        window_events: log.num_events(),
                    };

                    let _ = tx.send(result).await;
                }
            }
        });

        output_rx
    }
}

pub struct StreamingResult {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub model: PetriNet,
    pub window_events: usize,
}
```

---

## ML Integration

### Next-Activity Prediction

```rust
pub struct NextActivityPredictor {
    models: HashMap<String, Vec<String>>,
}

impl NextActivityPredictor {
    pub fn new(log: &EventLog) -> Self {
        let mut models = HashMap::new();

        for trace in log.traces() {
            for i in 0..trace.events.len() - 1 {
                let current = &trace.events[i].name;
                let next = &trace.events[i + 1].name;

                models.entry(current.clone())
                    .or_insert_with(Vec::new)
                    .push(next.clone());
            }
        }

        Self { models }
    }

    pub fn predict_next(&self, current_activity: &str) -> Option<String> {
        let possibilities = self.models.get(current_activity)?;

        // Return most frequent
        let mut counts: HashMap<String, usize> = HashMap::new();
        for activity in possibilities {
            *counts.entry(activity.clone()).or_insert(0) += 1;
        }

        counts.into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(activity, _)| activity)
    }

    pub fn predict_probability(&self, current: &str, next: &str) -> f64 {
        let possibilities = match self.models.get(current) {
            Some(p) => p,
            None => return 0.0,
        };

        let count = possibilities.iter().filter(|a| *a == next).count();
        count as f64 / possibilities.len() as f64
    }
}

// Usage
fn main() {
    let log = load_log();
    let predictor = NextActivityPredictor::new(&log);

    if let Some(next) = predictor.predict_next("Order") {
        println!("After Order, next is: {}", next);
    }

    let prob = predictor.predict_probability("Order", "Payment");
    println!("Order → Payment probability: {:.2}%", prob * 100.0);
}
```

---

**Next:** See [Examples](../examples/) for working code samples of these advanced techniques.
