//! Temporal Profile Discovery and Conformance
//!
//! Temporal profiles capture time-based patterns in event logs.

use crate::log::EventLog;
use chrono::Duration;
use std::collections::HashMap;

/// Temporal profile containing time-based constraints
#[derive(Debug, Clone)]
pub struct TemporalProfile {
    /// Minimum time between activities
    pub min_time_between: HashMap<(String, String), Duration>,
    /// Maximum time between activities
    pub max_time_between: HashMap<(String, String), Duration>,
    /// Average time between activities
    pub avg_time_between: HashMap<(String, String), Duration>,
    /// Activity durations
    pub activity_durations: HashMap<String, Vec<Duration>>,
}

impl TemporalProfile {
    pub fn new() -> Self {
        Self {
            min_time_between: HashMap::new(),
            max_time_between: HashMap::new(),
            avg_time_between: HashMap::new(),
            activity_durations: HashMap::new(),
        }
    }
}

impl Default for TemporalProfile {
    fn default() -> Self {
        Self::new()
    }
}

/// Discover temporal profile from event log
pub fn discover_temporal_profile(log: &EventLog) -> TemporalProfile {
    let mut profile = TemporalProfile::new();
    let mut time_differences: HashMap<(String, String), Vec<Duration>> = HashMap::new();

    // Calculate time differences between activities
    for trace in &log.traces {
        for window in trace.events.windows(2) {
            let time_diff = window[1]
                .timestamp
                .signed_duration_since(window[0].timestamp);
            let key = (window[0].activity.clone(), window[1].activity.clone());
            time_differences
                .entry(key)
                .or_default()
                .push(time_diff.abs());

            // Track activity durations
            let act_dur = window[0]
                .timestamp
                .signed_duration_since(window[1].timestamp);
            profile
                .activity_durations
                .entry(window[0].activity.clone())
                .or_default()
                .push(act_dur.abs());
        }
    }

    // Calculate min, max, avg time between activities
    for (key, durations) in time_differences {
        if !durations.is_empty() {
            let min_dur = *durations
                .iter()
                .min_by_key(|d| d.num_milliseconds())
                .expect("durations is non-empty, checked above");
            let max_dur = *durations
                .iter()
                .max_by_key(|d| d.num_milliseconds())
                .expect("durations is non-empty, checked above");

            let sum: Duration = durations.iter().sum();
            let avg_dur = sum / durations.len() as i32;

            profile.min_time_between.insert(key.clone(), min_dur);
            profile.max_time_between.insert(key.clone(), max_dur);
            profile.avg_time_between.insert(key, avg_dur);
        }
    }

    profile
}

/// Conformance checking against temporal profile
#[derive(Debug, Clone)]
pub struct TemporalConformanceResult {
    pub deviating_traces: usize,
    pub deviations: Vec<String>,
}

/// Check conformance against temporal profile
pub fn conformance_temporal_profile(
    log: &EventLog,
    profile: &TemporalProfile,
    tolerance: f64, // Tolerance as percentage (0.0-1.0)
) -> TemporalConformanceResult {
    let mut deviating_traces = 0;
    let mut deviations = Vec::new();

    for trace in &log.traces {
        let mut trace_deviations = Vec::new();

        for window in trace.events.windows(2) {
            let key = (window[0].activity.clone(), window[1].activity.clone());

            if let Some(expected_min) = profile.min_time_between.get(&key) {
                let actual = window[1]
                    .timestamp
                    .signed_duration_since(window[0].timestamp);

                // tolerance is a percentage (0.0-1.0), convert to Duration
                let tolerance_ms = expected_min.num_milliseconds() as f64 * tolerance;
                let tolerance_dur = Duration::milliseconds(tolerance_ms as i64);

                if actual < *expected_min - tolerance_dur || actual > *expected_min + tolerance_dur
                {
                    trace_deviations.push(format!(
                        "Time deviation for {} -> {}: expected {:?}, got {:?}",
                        window[0].activity, window[1].activity, expected_min, actual
                    ));
                }
            }
        }

        if !trace_deviations.is_empty() {
            deviating_traces += 1;
            deviations.extend(trace_deviations);
        }
    }

    TemporalConformanceResult {
        deviating_traces,
        deviations,
    }
}

/// Get enabled transitions from a Petri net
pub fn get_enabled_transitions(
    net: &crate::models::PetriNet,
    marking: &HashMap<String, usize>,
) -> Vec<String> {
    let mut enabled = Vec::new();

    for transition in &net.transitions {
        // Check if all input places have enough tokens
        let mut can_fire = true;

        for arc in &net.arcs {
            if arc.to == transition.id {
                if let Some(place) = net.get_place(&arc.from) {
                    let available = *marking.get(&place.id).unwrap_or(&0);
                    if available < 1 {
                        can_fire = false;
                        break;
                    }
                }
            }
        }

        if can_fire {
            enabled.push(transition.id.clone());
        }
    }

    enabled
}

/// Get minimum self-distance for activities
pub fn get_minimum_self_distances(log: &EventLog, activity: &str) -> usize {
    let mut min_distance = usize::MAX;

    for trace in &log.traces {
        let positions: Vec<usize> = trace
            .events
            .iter()
            .enumerate()
            .filter(|(_, e)| e.activity == activity)
            .map(|(i, _)| i)
            .collect();

        for window in positions.windows(2) {
            let distance = window[1] - window[0];
            if distance > 0 && distance < min_distance {
                min_distance = distance;
            }
        }
    }

    if min_distance == usize::MAX {
        0
    } else {
        min_distance
    }
}

/// Get minimum self-distance witnesses (example traces)
pub fn get_minimum_self_distance_witnesses(log: &EventLog, activity: &str) -> Vec<Vec<String>> {
    let mut witnesses = Vec::new();
    let mut min_distance = usize::MAX;

    // First find the minimum distance
    for trace in &log.traces {
        let positions: Vec<usize> = trace
            .events
            .iter()
            .enumerate()
            .filter(|(_, e)| e.activity == activity)
            .map(|(i, _)| i)
            .collect();

        for window in positions.windows(2) {
            let distance = window[1] - window[0];
            if distance > 0 && distance < min_distance {
                min_distance = distance;
            }
        }
    }

    // Then collect traces with that distance
    if min_distance < usize::MAX {
        for trace in &log.traces {
            let positions: Vec<usize> = trace
                .events
                .iter()
                .enumerate()
                .filter(|(_, e)| e.activity == activity)
                .map(|(i, _)| i)
                .collect();

            for window in positions.windows(2) {
                let distance = window[1] - window[0];
                if distance == min_distance {
                    let witness: Vec<String> =
                        trace.events.iter().map(|e| e.activity.clone()).collect();
                    witnesses.push(witness);
                    break;
                }
            }
        }
    }

    witnesses
}

/// Check if a Petri net is a workflow net (well-formed)
pub fn check_is_workflow_net(net: &crate::models::PetriNet) -> bool {
    // A workflow net must have:
    // 1. Exactly one source place (no incoming arcs)
    // 2. Exactly one sink place (no outgoing arcs)
    // 3. All other places are on a path from source to sink

    let mut source_candidates = Vec::new();
    let mut sink_candidates = Vec::new();

    for place in &net.places {
        let incoming_count = net.arcs.iter().filter(|a| a.to == place.id).count();
        let outgoing_count = net.arcs.iter().filter(|a| a.from == place.id).count();

        if incoming_count == 0 {
            source_candidates.push(place.id.clone());
        }
        if outgoing_count == 0 {
            sink_candidates.push(place.id.clone());
        }
    }

    source_candidates.len() == 1 && sink_candidates.len() == 1
}

/// Check if a Petri net is sound
pub fn check_soundness(net: &crate::models::PetriNet) -> bool {
    use crate::models::petri_net_analysis::PetriNetAnalyzer;
    PetriNetAnalyzer::check_soundness(net).is_sound
}

/// Check if a log fits a Petri net
pub fn check_is_fitting(log: &EventLog, net: &crate::models::PetriNet) -> bool {
    // Use token replay to check fitting
    use crate::conformance::TokenReplay;

    let replay = TokenReplay::new();
    let result = replay.check(log, net);

    result.is_conformant
}
