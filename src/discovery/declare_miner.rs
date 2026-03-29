//! Declare Miner - Constraint-Based Process Discovery
//!
//! Declare miner discovers process models by specifying constraints (declare)
//! that the model must satisfy. This is useful for flexible and declarative
//! process mining.

use crate::log::EventLog;
use std::collections::{HashMap, HashSet};

/// Declare constraint types
#[derive(Debug, Clone, PartialEq)]
pub enum DeclareConstraint {
    /// Activity A must always be followed by activity B
    Response {
        activity_a: String,
        activity_b: String,
    },
    /// If activity A occurs, activity B must have occurred before
    Precedence {
        activity_a: String,
        activity_b: String,
    },
    /// Activity A and B cannot occur in the same trace
    NegativeConstraint {
        activity_a: String,
        activity_b: String,
    },
    /// Activity A must always be followed by activity B (with no A in between)
    ChainResponse {
        activity_a: String,
        activity_b: String,
    },
    /// Activity A and B must not occur consecutively
    NotChainResponse {
        activity_a: String,
        activity_b: String,
    },
    /// Activity A must always be followed by activity B (eventually)
    RespondedExistence {
        activity_a: String,
        activity_b: String,
    },
    /// Activity A must always be preceded by activity B
    AltPrecedence {
        activity_a: String,
        activity_b: String,
    },
    /// Activity A and B cannot occur in the same trace
    CoExistence {
        activity_a: String,
        activity_b: String,
    },
}

impl DeclareConstraint {
    /// Check if a trace satisfies this constraint
    pub fn check_trace(&self, trace: &[String]) -> bool {
        match self {
            DeclareConstraint::Response {
                activity_a,
                activity_b,
            } => {
                // Every A must be followed by B
                let mut found_a = false;
                for activity in trace {
                    if activity == activity_a {
                        found_a = true;
                    } else if activity == activity_b && found_a {
                        found_a = false;
                    }
                }
                !found_a
            }
            DeclareConstraint::Precedence {
                activity_a,
                activity_b,
            } => {
                // If A occurs, B must have occurred before
                let mut seen_b = false;
                for activity in trace {
                    if activity == activity_b {
                        seen_b = true;
                    } else if activity == activity_a && !seen_b {
                        return false;
                    }
                }
                true
            }
            DeclareConstraint::NegativeConstraint {
                activity_a,
                activity_b,
            } => {
                // A and B cannot occur in the same trace
                let has_a = trace.contains(activity_a);
                let has_b = trace.contains(activity_b);
                !(has_a && has_b)
            }
            DeclareConstraint::ChainResponse {
                activity_a,
                activity_b,
            } => {
                // Every A must be immediately followed by B
                for window in trace.windows(2) {
                    if window[0] == *activity_a && window[1] != *activity_b {
                        return false;
                    }
                }
                true
            }
            DeclareConstraint::NotChainResponse {
                activity_a,
                activity_b,
            } => {
                // A and B must not occur consecutively (A followed by B)
                for window in trace.windows(2) {
                    if window[0] == *activity_a && window[1] == *activity_b {
                        return false;
                    }
                }
                true
            }
            DeclareConstraint::RespondedExistence {
                activity_a,
                activity_b,
            } => {
                // If A occurs, B must occur (anywhere in trace)
                let has_a = trace.contains(activity_a);
                let has_b = trace.contains(activity_b);
                !has_a || has_b
            }
            DeclareConstraint::AltPrecedence {
                activity_a,
                activity_b,
            } => {
                // Every A must be directly preceded by B
                for window in trace.windows(2) {
                    if window[1] == *activity_a && window[0] != *activity_b {
                        return false;
                    }
                }
                true
            }
            DeclareConstraint::CoExistence {
                activity_a,
                activity_b,
            } => {
                // A and B must either both occur or both not occur
                let has_a = trace.contains(activity_a);
                let has_b = trace.contains(activity_b);
                has_a == has_b
            }
        }
    }

    /// Get the activities involved in this constraint
    pub fn activities(&self) -> Vec<String> {
        match self {
            DeclareConstraint::Response {
                activity_a,
                activity_b,
            } => {
                vec![activity_a.clone(), activity_b.clone()]
            }
            DeclareConstraint::Precedence {
                activity_a,
                activity_b,
            } => {
                vec![activity_a.clone(), activity_b.clone()]
            }
            DeclareConstraint::NegativeConstraint {
                activity_a,
                activity_b,
            } => {
                vec![activity_a.clone(), activity_b.clone()]
            }
            DeclareConstraint::ChainResponse {
                activity_a,
                activity_b,
            } => {
                vec![activity_a.clone(), activity_b.clone()]
            }
            DeclareConstraint::NotChainResponse {
                activity_a,
                activity_b,
            } => {
                vec![activity_a.clone(), activity_b.clone()]
            }
            DeclareConstraint::RespondedExistence {
                activity_a,
                activity_b,
            } => {
                vec![activity_a.clone(), activity_b.clone()]
            }
            DeclareConstraint::AltPrecedence {
                activity_a,
                activity_b,
            } => {
                vec![activity_a.clone(), activity_b.clone()]
            }
            DeclareConstraint::CoExistence {
                activity_a,
                activity_b,
            } => {
                vec![activity_a.clone(), activity_b.clone()]
            }
        }
    }
}

/// Result of declare constraint discovery
#[derive(Debug, Clone)]
pub struct DeclareModel {
    pub constraints: Vec<DeclareConstraint>,
    pub support: HashMap<String, f64>,
}

impl DeclareModel {
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
            support: HashMap::new(),
        }
    }

    /// Add a constraint to the model
    pub fn add_constraint(&mut self, constraint: DeclareConstraint, support: f64) {
        let key = format!("{:?}", constraint);
        self.support.insert(key, support);
        self.constraints.push(constraint);
    }

    /// Check if a trace satisfies all constraints
    pub fn check_trace(&self, trace: &[String]) -> bool {
        self.constraints.iter().all(|c| c.check_trace(trace))
    }
}

impl Default for DeclareModel {
    fn default() -> Self {
        Self::new()
    }
}

/// Declare miner for constraint-based discovery
#[derive(Debug, Clone)]
pub struct DeclareMiner {
    pub min_support: f64,
}

impl DeclareMiner {
    pub fn new() -> Self {
        Self { min_support: 0.5 }
    }

    pub fn with_min_support(mut self, min_support: f64) -> Self {
        self.min_support = min_support;
        self
    }

    /// Discover declare constraints from event log
    pub fn discover(&self, log: &EventLog) -> DeclareModel {
        let mut model = DeclareModel::new();
        let activities: HashSet<String> = log.activities().into_iter().collect();

        // Discover Response constraints
        for activity_a in &activities {
            for activity_b in &activities {
                if activity_a != activity_b {
                    let support = self.calculate_response_support(log, activity_a, activity_b);
                    if support >= self.min_support {
                        model.add_constraint(
                            DeclareConstraint::Response {
                                activity_a: activity_a.clone(),
                                activity_b: activity_b.clone(),
                            },
                            support,
                        );
                    }
                }
            }
        }

        // Discover Precedence constraints
        for activity_a in &activities {
            for activity_b in &activities {
                if activity_a != activity_b {
                    let support = self.calculate_precedence_support(log, activity_a, activity_b);
                    if support >= self.min_support {
                        model.add_constraint(
                            DeclareConstraint::Precedence {
                                activity_a: activity_a.clone(),
                                activity_b: activity_b.clone(),
                            },
                            support,
                        );
                    }
                }
            }
        }

        model
    }

    fn calculate_response_support(&self, log: &EventLog, a: &str, b: &str) -> f64 {
        let mut satisfied = 0;
        let mut total = 0;

        for trace in &log.traces {
            let activities: Vec<&str> = trace.events.iter().map(|e| e.activity.as_str()).collect();
            let has_a = activities.contains(&a);
            let _has_b = activities.contains(&b);

            if has_a {
                total += 1;
                // Check if every A is followed by B
                let mut found_a = false;
                let ok = true;
                for activity in &activities {
                    if *activity == a {
                        found_a = true;
                    } else if *activity == b && found_a {
                        found_a = false;
                    }
                }
                if !found_a && ok {
                    satisfied += 1;
                }
            }
        }

        if total == 0 {
            1.0
        } else {
            satisfied as f64 / total as f64
        }
    }

    fn calculate_precedence_support(&self, log: &EventLog, a: &str, b: &str) -> f64 {
        let mut satisfied = 0;
        let mut total = 0;

        for trace in &log.traces {
            let activities: Vec<&str> = trace.events.iter().map(|e| e.activity.as_str()).collect();
            let has_a = activities.contains(&a);
            let _has_b = activities.contains(&b);

            if has_a {
                total += 1;
                // Check if all A's occur after the first B
                let b_pos = activities.iter().position(|x| *x == b);
                let a_pos = activities.iter().position(|x| *x == a);
                if let (Some(bp), Some(ap)) = (b_pos, a_pos) {
                    if bp < ap {
                        satisfied += 1;
                    }
                }
            }
        }

        if total == 0 {
            1.0
        } else {
            satisfied as f64 / total as f64
        }
    }
}

impl Default for DeclareMiner {
    fn default() -> Self {
        Self::new()
    }
}

/// Check conformance against a declare model
pub fn conformance_declare(log: &EventLog, model: &DeclareModel) -> (usize, usize) {
    let mut satisfying = 0;
    let mut total = 0;

    for trace in &log.traces {
        let activities: Vec<String> = trace.events.iter().map(|e| e.activity.clone()).collect();
        total += 1;
        if model.check_trace(&activities) {
            satisfying += 1;
        }
    }

    (satisfying, total)
}

/// Get declare constraint templates (all possible constraint types)
pub fn get_declare_constraint_templates() -> Vec<String> {
    vec![
        "Response".to_string(),
        "Precedence".to_string(),
        "NegativeConstraint".to_string(),
        "ChainResponse".to_string(),
        "NotChainResponse".to_string(),
        "RespondedExistence".to_string(),
        "AltPrecedence".to_string(),
        "CoExistence".to_string(),
    ]
}
