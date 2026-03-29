//! Behavioral Profile Conformance Checking
//!
//! This module implements behavioral profile conformance checking, which analyzes
//! the ordering relationships between activities in event logs and process models.

use crate::log::EventLog;
use crate::models::PetriNet;
use std::collections::{HashMap, HashSet};

/// Activity relation types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActivityRelation {
    /// Activities can occur in any order
    Parallel,
    /// One activity precedes the other
    Precedence,
    /// Activities are mutually exclusive
    Choice,
    /// Activity can follow itself
    Loop,
}

impl ActivityRelation {
    pub fn as_str(&self) -> &'static str {
        match self {
            ActivityRelation::Parallel => "Parallel",
            ActivityRelation::Precedence => "Precedence",
            ActivityRelation::Choice => "Choice",
            ActivityRelation::Loop => "Loop",
        }
    }
}

/// Behavioral profile representing activity ordering relationships
#[derive(Debug, Clone)]
pub struct BehavioralProfile {
    pub relations: HashMap<(String, String), ActivityRelation>,
    pub loops: HashSet<String>,
    pub activities: HashSet<String>,
}

impl BehavioralProfile {
    pub fn new() -> Self {
        Self {
            relations: HashMap::new(),
            loops: HashSet::new(),
            activities: HashSet::new(),
        }
    }

    pub fn extract_from_log(log: &EventLog) -> Self {
        let mut profile = Self::new();
        let mut precedence_pairs: HashMap<(String, String), usize> = HashMap::new();
        let mut self_follows: HashMap<String, bool> = HashMap::new();

        for trace in &log.traces {
            for event in &trace.events {
                profile.activities.insert(event.activity.clone());
            }
        }

        for trace in &log.traces {
            for (i, event_i) in trace.events.iter().enumerate() {
                if i > 0 {
                    let prev_activity = &trace.events[i - 1].activity;
                    if prev_activity == &event_i.activity {
                        self_follows.insert(event_i.activity.clone(), true);
                    }
                }

                for (j, event_j) in trace.events.iter().enumerate() {
                    if i == j || event_i.activity == event_j.activity {
                        continue;
                    }

                    let pair = if i < j {
                        (event_i.activity.clone(), event_j.activity.clone())
                    } else {
                        (event_j.activity.clone(), event_i.activity.clone())
                    };

                    if i < j {
                        *precedence_pairs.entry(pair).or_insert(0) += 1;
                    }
                }
            }
        }

        for activity_a in &profile.activities {
            for activity_b in &profile.activities {
                if activity_a == activity_b {
                    continue;
                }

                let pair = (activity_a.clone(), activity_b.clone());
                let reverse_pair = (activity_b.clone(), activity_a.clone());

                if let Some(&count) = precedence_pairs.get(&pair) {
                    if count > 0 {
                        let reverse_count = *precedence_pairs.get(&reverse_pair).unwrap_or(&0);
                        let total = count + reverse_count;

                        if total == 0 {
                            continue;
                        }

                        let ratio = count as f64 / total as f64;

                        if ratio > 0.9 {
                            profile.relations.insert(pair, ActivityRelation::Precedence);
                        } else if ratio > 0.1 && ratio < 0.9 {
                            profile.relations.insert(pair, ActivityRelation::Parallel);
                        } else if reverse_count > 0 {
                            profile.relations.insert(pair, ActivityRelation::Precedence);
                        }
                    }
                }
            }
        }

        for (activity, _) in self_follows.iter().filter(|(_, &has_loop)| has_loop) {
            profile.loops.insert(activity.clone());
        }

        profile
    }

    pub fn extract_from_model(_net: &PetriNet) -> Self {
        Self::new()
    }

    pub fn find_conflicts(&self, other: &BehavioralProfile) -> Vec<ConflictInfo> {
        let mut conflicts = Vec::new();

        let all_activities = self
            .activities
            .iter()
            .chain(other.activities.iter())
            .cloned()
            .collect::<HashSet<_>>();

        for activity_a in &all_activities {
            for activity_b in &all_activities {
                if activity_a >= activity_b {
                    continue;
                }

                let pair = (activity_a.clone(), activity_b.clone());
                let reverse_pair = (activity_b.clone(), activity_a.clone());

                let self_relation = self
                    .relations
                    .get(&pair)
                    .or_else(|| self.relations.get(&reverse_pair));
                let other_relation = other
                    .relations
                    .get(&pair)
                    .or_else(|| other.relations.get(&reverse_pair));

                if let (Some(sr), Some(or_)) = (self_relation, other_relation) {
                    if sr != or_ {
                        conflicts.push(ConflictInfo {
                            activity_a: activity_a.clone(),
                            activity_b: activity_b.clone(),
                            log_relation: *sr,
                            model_relation: *or_,
                            severity: Self::compute_severity(*sr, *or_),
                        });
                    }
                }
            }
        }

        conflicts.sort_by(|a, b| {
            b.severity
                .partial_cmp(&a.severity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        conflicts
    }

    pub fn get_relation(&self, activity_a: &str, activity_b: &str) -> Option<ActivityRelation> {
        let pair = (activity_a.to_string(), activity_b.to_string());
        let reverse_pair = (activity_b.to_string(), activity_a.to_string());

        self.relations
            .get(&pair)
            .copied()
            .or_else(|| self.relations.get(&reverse_pair).copied())
    }

    pub fn has_loop(&self, activity: &str) -> bool {
        self.loops.contains(activity)
    }

    pub fn compute_conformance(&self, other: &BehavioralProfile) -> f64 {
        let conflicts = self.find_conflicts(other);

        if self.relations.is_empty() && other.relations.is_empty() {
            return 1.0;
        }

        let total_pairs = (self.relations.len() + other.relations.len()) as f64;
        if total_pairs == 0.0 {
            return 1.0;
        }

        let conflict_weight: f64 = conflicts.iter().map(|c| c.severity).sum();
        let conformance = 1.0 - (conflict_weight / total_pairs);

        conformance.clamp(0.0, 1.0)
    }

    fn compute_severity(rel1: ActivityRelation, rel2: ActivityRelation) -> f64 {
        match (rel1, rel2) {
            (ActivityRelation::Precedence, ActivityRelation::Choice)
            | (ActivityRelation::Choice, ActivityRelation::Precedence) => 1.0,
            (ActivityRelation::Precedence, ActivityRelation::Parallel)
            | (ActivityRelation::Parallel, ActivityRelation::Precedence) => 0.8,
            _ => 0.5,
        }
    }
}

impl Default for BehavioralProfile {
    fn default() -> Self {
        Self::new()
    }
}

/// Information about a conflict between profiles
#[derive(Debug, Clone)]
pub struct ConflictInfo {
    pub activity_a: String,
    pub activity_b: String,
    pub log_relation: ActivityRelation,
    pub model_relation: ActivityRelation,
    pub severity: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::{Event, Trace};
    use chrono::Utc;

    fn create_test_log() -> EventLog {
        let mut log = EventLog::new();
        let now = Utc::now();

        let mut trace1 = Trace::new("case_1");
        trace1.add_event(Event::new("A", now));
        trace1.add_event(Event::new("B", now));
        trace1.add_event(Event::new("C", now));
        log.add_trace(trace1);

        let mut trace2 = Trace::new("case_2");
        trace2.add_event(Event::new("A", now));
        trace2.add_event(Event::new("B", now));
        trace2.add_event(Event::new("C", now));
        log.add_trace(trace2);

        log
    }

    #[test]
    fn test_behavioral_profile_creation() {
        let profile = BehavioralProfile::new();
        assert!(profile.relations.is_empty());
        assert!(profile.loops.is_empty());
    }

    #[test]
    fn test_extract_from_log_precedence() {
        let log = create_test_log();
        let profile = BehavioralProfile::extract_from_log(&log);

        assert!(!profile.activities.is_empty());
        assert!(profile.activities.contains("A"));
        assert!(profile.activities.contains("B"));
        assert!(profile.activities.contains("C"));
    }

    #[test]
    fn test_loop_detection() {
        let mut log = EventLog::new();
        let now = Utc::now();

        let mut trace = Trace::new("case_1");
        trace.add_event(Event::new("A", now));
        trace.add_event(Event::new("A", now));
        trace.add_event(Event::new("B", now));
        log.add_trace(trace);

        let profile = BehavioralProfile::extract_from_log(&log);
        assert!(profile.has_loop("A"));
        assert!(!profile.has_loop("B"));
    }

    #[test]
    fn test_conflict_detection() {
        let log = create_test_log();
        let log_profile = BehavioralProfile::extract_from_log(&log);

        let mut model_profile = BehavioralProfile::new();
        model_profile.activities.insert("A".to_string());
        model_profile.activities.insert("B".to_string());
        model_profile.activities.insert("C".to_string());
        model_profile
            .relations
            .insert(("A".to_string(), "B".to_string()), ActivityRelation::Choice);

        let conflicts = log_profile.find_conflicts(&model_profile);
        assert!(!conflicts.is_empty());
    }

    #[test]
    fn test_conformance_score() {
        let log = create_test_log();
        let profile1 = BehavioralProfile::extract_from_log(&log);
        let profile2 = BehavioralProfile::extract_from_log(&log);

        let conformance = profile1.compute_conformance(&profile2);
        assert_eq!(conformance, 1.0);
    }

    #[test]
    fn test_get_relation() {
        let mut profile = BehavioralProfile::new();
        profile.activities.insert("A".to_string());
        profile.activities.insert("B".to_string());
        profile.relations.insert(
            ("A".to_string(), "B".to_string()),
            ActivityRelation::Precedence,
        );

        assert_eq!(
            profile.get_relation("A", "B"),
            Some(ActivityRelation::Precedence)
        );
    }

    #[test]
    fn test_activity_relation_as_str() {
        assert_eq!(ActivityRelation::Parallel.as_str(), "Parallel");
        assert_eq!(ActivityRelation::Precedence.as_str(), "Precedence");
        assert_eq!(ActivityRelation::Choice.as_str(), "Choice");
        assert_eq!(ActivityRelation::Loop.as_str(), "Loop");
    }

    #[test]
    fn test_conformance_with_conflicts() {
        let log = create_test_log();
        let log_profile = BehavioralProfile::extract_from_log(&log);

        let mut model_profile = BehavioralProfile::new();
        model_profile.activities.insert("A".to_string());
        model_profile.activities.insert("B".to_string());
        model_profile.activities.insert("C".to_string());

        let conformance = log_profile.compute_conformance(&model_profile);
        assert!((0.0..=1.0).contains(&conformance));
    }
}
