/// Typed A2A task state machine.
///
/// Maps protocol state strings ("submitted", "working", ...) to a typed enum and
/// enforces valid transitions. Adapted from ggen-core task.rs.tera pattern.
///
/// State diagram:
///   Pending → InProgress → Completed
///                        → Failed
///             (any) → Canceled
use std::fmt;

/// All valid A2A task states.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Canceled,
}

impl TaskStatus {
    /// Returns true if transitioning to `target` is a valid A2A protocol move.
    pub fn can_transition_to(&self, target: &TaskStatus) -> bool {
        match (self, target) {
            // From Pending: can start working or cancel
            (TaskStatus::Pending, TaskStatus::InProgress) => true,
            (TaskStatus::Pending, TaskStatus::Canceled) => true,
            // From InProgress: can complete, fail, or cancel
            (TaskStatus::InProgress, TaskStatus::Completed) => true,
            (TaskStatus::InProgress, TaskStatus::Failed) => true,
            (TaskStatus::InProgress, TaskStatus::Canceled) => true,
            // Terminal states have no valid transitions
            _ => false,
        }
    }

    /// Convert to the A2A protocol string representation.
    pub fn to_a2a_state_str(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "submitted",
            TaskStatus::InProgress => "working",
            TaskStatus::Completed => "completed",
            TaskStatus::Failed => "failed",
            TaskStatus::Canceled => "canceled",
        }
    }

    /// Parse from A2A protocol string.
    pub fn from_a2a_state_str(s: &str) -> Option<Self> {
        match s {
            "submitted" => Some(TaskStatus::Pending),
            "working" => Some(TaskStatus::InProgress),
            "completed" => Some(TaskStatus::Completed),
            "failed" => Some(TaskStatus::Failed),
            "canceled" => Some(TaskStatus::Canceled),
            _ => None,
        }
    }

    /// Returns true if this is a terminal state (no further transitions possible).
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Canceled
        )
    }
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_a2a_state_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_can_transition_to_in_progress() {
        assert!(TaskStatus::Pending.can_transition_to(&TaskStatus::InProgress));
    }

    #[test]
    fn pending_can_transition_to_canceled() {
        assert!(TaskStatus::Pending.can_transition_to(&TaskStatus::Canceled));
    }

    #[test]
    fn in_progress_can_transition_to_completed() {
        assert!(TaskStatus::InProgress.can_transition_to(&TaskStatus::Completed));
    }

    #[test]
    fn in_progress_can_transition_to_failed() {
        assert!(TaskStatus::InProgress.can_transition_to(&TaskStatus::Failed));
    }

    #[test]
    fn completed_cannot_transition_to_any() {
        assert!(!TaskStatus::Completed.can_transition_to(&TaskStatus::Pending));
        assert!(!TaskStatus::Completed.can_transition_to(&TaskStatus::InProgress));
        assert!(!TaskStatus::Completed.can_transition_to(&TaskStatus::Failed));
        assert!(!TaskStatus::Completed.can_transition_to(&TaskStatus::Canceled));
    }

    #[test]
    fn a2a_state_strings_round_trip() {
        let states = [
            TaskStatus::Pending,
            TaskStatus::InProgress,
            TaskStatus::Completed,
            TaskStatus::Failed,
            TaskStatus::Canceled,
        ];
        for s in &states {
            let str_repr = s.to_a2a_state_str();
            let parsed = TaskStatus::from_a2a_state_str(str_repr);
            assert_eq!(parsed.as_ref(), Some(s), "round-trip failed for {:?}", s);
        }
    }

    #[test]
    fn terminal_states_identified() {
        assert!(TaskStatus::Completed.is_terminal());
        assert!(TaskStatus::Failed.is_terminal());
        assert!(TaskStatus::Canceled.is_terminal());
        assert!(!TaskStatus::Pending.is_terminal());
        assert!(!TaskStatus::InProgress.is_terminal());
    }
}
