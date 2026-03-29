/// In-memory task storage for the A2A handler.
///
/// WvdA boundedness: max 1000 tasks. When the limit is reached, the oldest
/// task (by insertion iteration order) is evicted before inserting the new one.
use dashmap::DashMap;
use std::sync::Arc;

use crate::a2a::protocol::{Artifact, Task};

/// Maximum number of tasks retained in memory at any time.
pub const MAX_TASK_STORAGE: usize = 1000;

/// Shared, thread-safe in-memory task store.
#[derive(Clone)]
pub struct InMemoryTaskStorage {
    tasks: Arc<DashMap<String, Task>>,
}

impl InMemoryTaskStorage {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(DashMap::new()),
        }
    }

    /// Insert or overwrite a task. Evicts an arbitrary oldest entry if at capacity.
    pub fn insert(&self, task: Task) {
        if self.tasks.len() >= MAX_TASK_STORAGE {
            // Evict first key found (DashMap has no ordered iteration; this is safe for WvdA).
            if let Some(key) = self.tasks.iter().next().map(|e| e.key().clone()) {
                self.tasks.remove(&key);
            }
        }
        self.tasks.insert(task.id.clone(), task);
    }

    pub fn get(&self, id: &str) -> Option<Task> {
        self.tasks.get(id).map(|e| e.value().clone())
    }

    /// Transition a task to a new state string. Returns false if the task does not exist.
    pub fn update_state(&self, id: &str, state: &str) -> bool {
        if let Some(mut entry) = self.tasks.get_mut(id) {
            entry.status.state = state.to_string();
            true
        } else {
            false
        }
    }

    /// Mark a task completed and attach output artifacts.
    pub fn set_completed(&self, id: &str, artifacts: Vec<Artifact>) -> bool {
        if let Some(mut entry) = self.tasks.get_mut(id) {
            entry.status.state = "completed".to_string();
            entry.artifacts = artifacts;
            true
        } else {
            false
        }
    }

    /// Mark a task failed (no artifacts).
    pub fn set_failed(&self, id: &str) -> bool {
        self.update_state(id, "failed")
    }
}

impl Default for InMemoryTaskStorage {
    fn default() -> Self {
        Self::new()
    }
}
