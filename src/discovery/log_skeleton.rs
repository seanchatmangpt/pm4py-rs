//! Log Skeleton Discovery
//!
//! Log Skeleton is a declarative process mining technique that discovers
//! constraints from event logs.

use crate::log::EventLog;
use std::collections::{HashMap, HashSet};

/// Log Skeleton containing discovered constraints
#[derive(Debug, Clone)]
pub struct LogSkeleton {
    /// Equivalence relations (activities that always follow each other)
    pub equivalence: Vec<(String, String)>,
    /// Always-after relations
    pub after: Vec<(String, String)>,
    /// Always-before relations
    pub before: Vec<(String, String)>,
    /// Never-together relations (mutex)
    pub never_together: Vec<(String, String)>,
    /// Directly-follows relations
    pub directly_follows: Vec<(String, String)>,
}

impl LogSkeleton {
    pub fn new() -> Self {
        Self {
            equivalence: Vec::new(),
            after: Vec::new(),
            before: Vec::new(),
            never_together: Vec::new(),
            directly_follows: Vec::new(),
        }
    }
}

impl Default for LogSkeleton {
    fn default() -> Self {
        Self::new()
    }
}

/// Log Skeleton Miner
#[derive(Debug, Clone)]
pub struct LogSkeletonMiner {
    pub confidence_threshold: f64,
}

impl Default for LogSkeletonMiner {
    fn default() -> Self {
        Self::new()
    }
}

impl LogSkeletonMiner {
    pub fn new() -> Self {
        Self {
            confidence_threshold: 0.0,
        }
    }

    pub fn with_confidence(mut self, threshold: f64) -> Self {
        self.confidence_threshold = threshold;
        self
    }

    pub fn discover(&self, log: &EventLog) -> LogSkeleton {
        let activities = log.activities();
        let traces: Vec<Vec<String>> = log
            .traces
            .iter()
            .map(|t| t.events.iter().map(|e| e.activity.clone()).collect())
            .collect();

        let mut skeleton = LogSkeleton::new();

        // Discover directly-follows relations
        let df_pairs = self.discover_directly_follows(&traces);
        skeleton.directly_follows = df_pairs.clone();

        // Discover equivalence relations
        skeleton.equivalence = self.discover_equivalence(&traces, &df_pairs);

        // Discover after relations
        skeleton.after = self.discover_after(&traces);

        // Discover before relations
        skeleton.before = self.discover_before(&traces);

        // Discover never-together relations
        skeleton.never_together = self.discover_never_together(&traces, &activities);

        skeleton
    }

    fn discover_directly_follows(&self, traces: &[Vec<String>]) -> Vec<(String, String)> {
        let mut df_counts: HashMap<(String, String), usize> = HashMap::new();

        for trace in traces {
            for window in trace.windows(2) {
                let pair = (window[0].clone(), window[1].clone());
                *df_counts.entry(pair).or_insert(0) += 1;
            }
        }

        df_counts.keys().cloned().collect()
    }

    fn discover_equivalence(
        &self,
        traces: &[Vec<String>],
        df_pairs: &[(String, String)],
    ) -> Vec<(String, String)> {
        let mut equivalence = Vec::new();
        let activities: HashSet<String> = traces.iter().flat_map(|t| t.iter().cloned()).collect();

        for act1 in &activities {
            for act2 in &activities {
                if act1 >= act2 {
                    continue;
                }

                // Check if act1 and act2 always appear in the same order
                let mut always_same_order = true;
                let mut found_together = false;

                for trace in traces {
                    let idx1 = trace.iter().position(|a| a == act1);
                    let idx2 = trace.iter().position(|a| a == act2);

                    if let (Some(i1), Some(i2)) = (idx1, idx2) {
                        found_together = true;
                        if !(i1 < i2 && df_pairs.contains(&(act1.clone(), act2.clone()))
                            || i2 < i1 && df_pairs.contains(&(act2.clone(), act1.clone())))
                        {
                            always_same_order = false;
                        }
                    }
                }

                if found_together && always_same_order {
                    equivalence.push((act1.clone(), act2.clone()));
                }
            }
        }

        equivalence
    }

    fn discover_after(&self, traces: &[Vec<String>]) -> Vec<(String, String)> {
        let mut after_map: HashMap<String, HashSet<String>> = HashMap::new();

        for trace in traces {
            for (i, event) in trace.iter().enumerate() {
                for later_event in trace.iter().skip(i + 1) {
                    after_map
                        .entry(event.clone())
                        .or_default()
                        .insert(later_event.clone());
                }
            }
        }

        after_map
            .into_iter()
            .flat_map(|(from, to_set)| to_set.into_iter().map(move |to| (from.clone(), to)))
            .collect()
    }

    fn discover_before(&self, traces: &[Vec<String>]) -> Vec<(String, String)> {
        let mut before_map: HashMap<String, HashSet<String>> = HashMap::new();

        for trace in traces {
            for (i, event) in trace.iter().enumerate() {
                for earlier_event in trace.iter().take(i) {
                    before_map
                        .entry(event.clone())
                        .or_default()
                        .insert(earlier_event.clone());
                }
            }
        }

        before_map
            .into_iter()
            .flat_map(|(to, from_set)| from_set.into_iter().map(move |from| (from, to.clone())))
            .collect()
    }

    fn discover_never_together(
        &self,
        traces: &[Vec<String>],
        activities: &[String],
    ) -> Vec<(String, String)> {
        let mut never_together = Vec::new();

        for i in 0..activities.len() {
            for j in (i + 1)..activities.len() {
                let act1 = &activities[i];
                let act2 = &activities[j];

                // Check if act1 and act2 never appear in the same trace
                let never_together_flag = traces.iter().all(|trace| {
                    let has_act1 = trace.contains(act1);
                    let has_act2 = trace.contains(act2);
                    !(has_act1 && has_act2)
                });

                if never_together_flag {
                    never_together.push((act1.clone(), act2.clone()));
                }
            }
        }

        never_together
    }
}

/// Conformance checking using Log Skeleton
#[derive(Debug, Clone)]
pub struct LogSkeletonConformanceResult {
    pub deviating_traces: usize,
    pub deviating_factors: Vec<String>,
}

/// Check conformance against a log skeleton
pub fn conformance_log_skeleton(
    log: &EventLog,
    skeleton: &LogSkeleton,
) -> LogSkeletonConformanceResult {
    let mut deviating_traces = 0;
    let mut deviating_factors = Vec::new();

    for trace in &log.traces {
        let activities: Vec<String> = trace.events.iter().map(|e| e.activity.clone()).collect();

        // Check equivalence violations
        for (act1, act2) in &skeleton.equivalence {
            let has_act1 = activities.contains(act1);
            let has_act2 = activities.contains(act2);
            if has_act1 && has_act2 {
                // Check order
                let idx1 = activities.iter().position(|a| a == act1);
                let idx2 = activities.iter().position(|a| a == act2);
                if let (Some(i1), Some(i2)) = (idx1, idx2) {
                    if i1 > i2 {
                        deviating_factors
                            .push(format!("Equivalence violation: {} after {}", act1, act2));
                    }
                }
            }
        }

        // Check never-together violations
        for (act1, act2) in &skeleton.never_together {
            let has_act1 = activities.contains(act1);
            let has_act2 = activities.contains(act2);
            if has_act1 && has_act2 {
                deviating_factors.push(format!("Never-together violation: {} and {}", act1, act2));
            }
        }

        if !deviating_factors.is_empty() {
            deviating_traces += 1;
        }
    }

    LogSkeletonConformanceResult {
        deviating_traces,
        deviating_factors,
    }
}
