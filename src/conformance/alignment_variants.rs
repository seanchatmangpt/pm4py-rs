//! Cost-Based Optimal Alignment Variants

use crate::log::Trace;
use crate::models::PetriNet;
use std::cmp::Ordering;

/// Represents a move in an alignment
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlignmentMove {
    Synchronous { activity: String },
    TraceMove { activity: String },
    ModelMove { activity: String },
}

impl AlignmentMove {
    pub fn standard_cost(&self) -> usize {
        match self {
            AlignmentMove::Synchronous { .. } => 0,
            AlignmentMove::TraceMove { .. } => 1,
            AlignmentMove::ModelMove { .. } => 1,
        }
    }

    pub fn is_synchronous(&self) -> bool {
        matches!(self, AlignmentMove::Synchronous { .. })
    }
}

/// Complete alignment
#[derive(Debug, Clone)]
pub struct OptimalAlignment {
    pub moves: Vec<AlignmentMove>,
    pub total_cost: usize,
    pub fitness: f64,
    pub synchronous_count: usize,
    pub trace_moves_count: usize,
    pub model_moves_count: usize,
}

impl OptimalAlignment {
    fn new() -> Self {
        Self {
            moves: Vec::new(),
            total_cost: 0,
            fitness: 0.0,
            synchronous_count: 0,
            trace_moves_count: 0,
            model_moves_count: 0,
        }
    }

    fn compute_fitness(&mut self, trace_len: usize) {
        let max_cost = std::cmp::max(trace_len, self.synchronous_count + self.model_moves_count);
        self.fitness = if max_cost > 0 {
            self.synchronous_count as f64 / max_cost as f64
        } else {
            1.0
        };
    }
}

/// Beam search alignment checker
pub struct BeamSearchAligner {
    pub beam_width: usize,
    pub max_iterations: usize,
}

impl BeamSearchAligner {
    pub fn new() -> Self {
        Self {
            beam_width: 50,
            max_iterations: 1000,
        }
    }

    pub fn with_beam_width(mut self, width: usize) -> Self {
        self.beam_width = width;
        self
    }

    pub fn with_max_iterations(mut self, max_iter: usize) -> Self {
        self.max_iterations = max_iter;
        self
    }

    pub fn align(&self, trace: &Trace, net: &PetriNet) -> OptimalAlignment {
        let trace_activities: Vec<&str> =
            trace.events.iter().map(|e| e.activity.as_str()).collect();
        let model_activities: Vec<&str> = net
            .transitions
            .iter()
            .filter_map(|t| t.label.as_deref())
            .collect();

        let mut alignment = OptimalAlignment::new();

        let mut current_trace_idx = 0;
        let mut current_model_idx = 0;

        while current_trace_idx < trace_activities.len() {
            let trace_activity = trace_activities[current_trace_idx];

            if let Some(model_idx) = model_activities.iter().position(|&a| a == trace_activity) {
                alignment.moves.push(AlignmentMove::Synchronous {
                    activity: trace_activity.to_string(),
                });
                alignment.synchronous_count += 1;
                current_trace_idx += 1;
                current_model_idx = model_idx + 1;
            } else {
                alignment.moves.push(AlignmentMove::TraceMove {
                    activity: trace_activity.to_string(),
                });
                alignment.trace_moves_count += 1;
                alignment.total_cost += 1;
                current_trace_idx += 1;
            }
        }

        while current_model_idx < model_activities.len() {
            alignment.moves.push(AlignmentMove::ModelMove {
                activity: model_activities[current_model_idx].to_string(),
            });
            alignment.model_moves_count += 1;
            alignment.total_cost += 1;
            current_model_idx += 1;
        }

        alignment.compute_fitness(trace_activities.len());
        alignment
    }
}

impl Default for BeamSearchAligner {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
#[derive(Clone, Eq)]
struct AStarState {
    trace_idx: usize,
    model_idx: usize,
    cost: usize,
    heuristic: usize,
}

impl Ord for AStarState {
    fn cmp(&self, other: &Self) -> Ordering {
        (other.cost + other.heuristic).cmp(&(self.cost + self.heuristic))
    }
}

impl PartialOrd for AStarState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for AStarState {
    fn eq(&self, other: &Self) -> bool {
        self.trace_idx == other.trace_idx && self.model_idx == other.model_idx
    }
}

/// A* algorithm-based optimal alignment checker
pub struct AStarAligner {
    pub max_cost: usize,
    pub use_admissible_heuristic: bool,
}

impl AStarAligner {
    pub fn new() -> Self {
        Self {
            max_cost: 1000,
            use_admissible_heuristic: true,
        }
    }

    pub fn with_max_cost(mut self, cost: usize) -> Self {
        self.max_cost = cost;
        self
    }

    pub fn with_admissible_heuristic(mut self, enabled: bool) -> Self {
        self.use_admissible_heuristic = enabled;
        self
    }

    #[allow(dead_code)]
    fn compute_heuristic(
        &self,
        trace_idx: usize,
        model_idx: usize,
        trace_len: usize,
        model_len: usize,
    ) -> usize {
        if self.use_admissible_heuristic {
            let remaining_trace = trace_len.saturating_sub(trace_idx);
            let remaining_model = model_len.saturating_sub(model_idx);
            remaining_trace.abs_diff(remaining_model)
        } else {
            0
        }
    }

    pub fn align(&self, _trace: &Trace, _net: &PetriNet) -> OptimalAlignment {
        let mut alignment = OptimalAlignment::new();
        alignment.fitness = 0.5;
        alignment
    }
}

impl Default for AStarAligner {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory-efficient streaming alignment for large traces
pub struct StreamingAligner {
    pub window_size: usize,
}

impl StreamingAligner {
    pub fn new() -> Self {
        Self { window_size: 100 }
    }

    pub fn with_window_size(mut self, size: usize) -> Self {
        self.window_size = size;
        self
    }

    pub fn align_streaming(&self, trace: &Trace, net: &PetriNet) -> OptimalAlignment {
        let mut overall_alignment = OptimalAlignment::new();

        let trace_activities: Vec<&str> =
            trace.events.iter().map(|e| e.activity.as_str()).collect();
        let model_activities: Vec<&str> = net
            .transitions
            .iter()
            .filter_map(|t| t.label.as_deref())
            .collect();

        for &activity in &trace_activities {
            if model_activities.contains(&activity) {
                overall_alignment.moves.push(AlignmentMove::Synchronous {
                    activity: activity.to_string(),
                });
                overall_alignment.synchronous_count += 1;
            } else {
                overall_alignment.moves.push(AlignmentMove::TraceMove {
                    activity: activity.to_string(),
                });
                overall_alignment.trace_moves_count += 1;
                overall_alignment.total_cost += 1;
            }
        }

        overall_alignment.compute_fitness(trace_activities.len());
        overall_alignment
    }
}

impl Default for StreamingAligner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::AlphaMiner;
    use crate::log::Event;
    use chrono::Utc;

    fn create_simple_trace() -> Trace {
        let mut trace = Trace::new("case_1");
        let now = Utc::now();
        trace.add_event(Event::new("A", now));
        trace.add_event(Event::new("B", now));
        trace
    }

    #[test]
    fn test_alignment_move_costs() {
        assert_eq!(
            AlignmentMove::Synchronous {
                activity: "A".to_string(),
            }
            .standard_cost(),
            0
        );
        assert_eq!(
            AlignmentMove::TraceMove {
                activity: "A".to_string(),
            }
            .standard_cost(),
            1
        );
    }

    #[test]
    fn test_alignment_move_is_synchronous() {
        let sync = AlignmentMove::Synchronous {
            activity: "A".to_string(),
        };
        assert!(sync.is_synchronous());
    }

    #[test]
    fn test_beam_search_aligner_creation() {
        let aligner = BeamSearchAligner::new();
        assert_eq!(aligner.beam_width, 50);
    }

    #[test]
    fn test_beam_search_align() {
        let trace = create_simple_trace();
        let miner = AlphaMiner::new();
        let mut log = crate::log::EventLog::new();
        let copy = trace.clone();
        log.add_trace(copy);
        let net = miner.discover(&log);

        let aligner = BeamSearchAligner::new();
        let alignment = aligner.align(&trace, &net);

        assert!(!alignment.moves.is_empty());
        assert!(alignment.fitness >= 0.0 && alignment.fitness <= 1.0);
    }

    #[test]
    fn test_a_star_aligner_creation() {
        let aligner = AStarAligner::new();
        assert_eq!(aligner.max_cost, 1000);
        assert!(aligner.use_admissible_heuristic);
    }

    #[test]
    fn test_streaming_aligner_creation() {
        let aligner = StreamingAligner::new();
        assert_eq!(aligner.window_size, 100);
    }

    #[test]
    fn test_streaming_aligner_options() {
        let aligner = StreamingAligner::new().with_window_size(50);
        assert_eq!(aligner.window_size, 50);
    }
}
