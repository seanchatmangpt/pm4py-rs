/// BPMN Execution Semantics
///
/// Token flow semantics for BPMN gateways and execution rules.
use crate::models::bpmn::*;
use std::collections::{HashMap, HashSet, VecDeque};

/// Token in a BPMN execution
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Token {
    pub id: String,
    pub location: String, // Element ID where token is located
}

impl Token {
    pub fn new(location: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            location: location.into(),
        }
    }
}

/// State of BPMN execution
#[derive(Debug, Clone)]
pub struct ExecutionState {
    pub tokens: Vec<Token>,
    pub enabled_flows: HashSet<String>,
    pub completed_tasks: HashSet<String>,
    pub current_step: usize,
}

impl ExecutionState {
    pub fn new() -> Self {
        Self {
            tokens: Vec::new(),
            enabled_flows: HashSet::new(),
            completed_tasks: HashSet::new(),
            current_step: 0,
        }
    }

    pub fn add_token(&mut self, token: Token) {
        self.tokens.push(token);
    }

    pub fn enable_flow(&mut self, flow_id: String) {
        self.enabled_flows.insert(flow_id);
    }

    pub fn complete_task(&mut self, task_id: String) {
        self.completed_tasks.insert(task_id);
    }

    pub fn has_tokens_at(&self, location: &str) -> bool {
        self.tokens.iter().any(|t| t.location == location)
    }

    pub fn move_tokens(&mut self, from: &str, to: &str) {
        for token in &mut self.tokens {
            if token.location == from {
                token.location = to.to_string();
            }
        }
    }
}

impl Default for ExecutionState {
    fn default() -> Self {
        Self::new()
    }
}

/// BPMN execution engine
pub struct BPMNExecutor;

impl BPMNExecutor {
    /// Execute BPMN diagram with a sequence of activities
    ///
    /// Returns execution state after each step
    pub fn execute(
        diagram: &BPMNDiagram,
        activity_sequence: &[&str],
    ) -> Result<ExecutionState, String> {
        diagram.validate()?;

        let mut state = ExecutionState::new();

        // Start with initial token at start event
        if let Some(start_id) = &diagram.start_event_id {
            state.add_token(Token::new(start_id.clone()));
        }

        // Map activity names to task IDs
        let mut activity_to_task: HashMap<String, String> = HashMap::new();
        for (id, task) in &diagram.tasks {
            activity_to_task.insert(task.name.clone(), id.clone());
        }

        // Process each activity in sequence
        for activity in activity_sequence {
            if let Some(task_id) = activity_to_task.get(*activity) {
                // Move tokens to this task
                Self::move_tokens_to_task(&mut state, diagram, task_id)?;
                state.complete_task(task_id.clone());
                // Move tokens out of task to next element(s)
                Self::fire_task_completion(&mut state, diagram, task_id)?;
            } else {
                return Err(format!("Unknown activity: {}", activity));
            }
        }

        // Move tokens to end event(s)
        if let Some(start_id) = &diagram.start_event_id {
            Self::move_to_end(&mut state, diagram, start_id)?;
        }

        Ok(state)
    }

    /// Move tokens to a task
    fn move_tokens_to_task(
        state: &mut ExecutionState,
        diagram: &BPMNDiagram,
        task_id: &str,
    ) -> Result<(), String> {
        // Find incoming flows to this task
        for flow in diagram.flows.values() {
            if flow.target_id == task_id {
                // Check if source has tokens
                let has_source_tokens = state.has_tokens_at(&flow.source_id);
                if has_source_tokens || diagram.events.values().any(|e| e.id == flow.source_id) {
                    state.move_tokens(&flow.source_id, task_id);
                    return Ok(());
                }
            }
        }

        Ok(())
    }

    /// Handle task completion and token progression
    fn fire_task_completion(
        state: &mut ExecutionState,
        diagram: &BPMNDiagram,
        task_id: &str,
    ) -> Result<(), String> {
        let _task = diagram.tasks.get(task_id).ok_or("Task not found")?;

        // Check outgoing flows
        for flow in diagram.flows.values() {
            if flow.source_id == task_id {
                // Check if condition is satisfied (simplified: no conditions block)
                if flow.condition.is_none() {
                    state.enable_flow(flow.id.clone());

                    // Move to target
                    if let Some(_gateway) = diagram.gateways.get(&flow.target_id) {
                        // Handle gateway
                        Self::handle_gateway_entry(state, diagram, &flow.target_id)?;
                    } else {
                        state.move_tokens(task_id, &flow.target_id);
                    }
                }
            }
        }

        Ok(())
    }

    /// Handle token entry into a gateway
    fn handle_gateway_entry(
        state: &mut ExecutionState,
        diagram: &BPMNDiagram,
        gateway_id: &str,
    ) -> Result<(), String> {
        let gateway = diagram
            .gateways
            .get(gateway_id)
            .ok_or("Gateway not found")?;

        match gateway.gateway_type {
            GatewayType::Parallel => {
                // Split: token goes to all outgoing flows
                for flow in diagram.flows.values() {
                    if flow.source_id == gateway_id {
                        state.move_tokens(gateway_id, &flow.target_id);
                    }
                }
            }
            GatewayType::ExclusiveXor => {
                // Split: token goes to ONE outgoing flow
                // (simplified: take first with no condition)
                for flow in diagram.flows.values() {
                    if flow.source_id == gateway_id && flow.condition.is_none() {
                        state.move_tokens(gateway_id, &flow.target_id);
                        break;
                    }
                }
            }
            GatewayType::Inclusive => {
                // Split: token goes to one or more outgoing flows
                for flow in diagram.flows.values() {
                    if flow.source_id == gateway_id {
                        state.move_tokens(gateway_id, &flow.target_id);
                    }
                }
            }
            GatewayType::EventBased => {
                // Wait for event
                for flow in diagram.flows.values() {
                    if flow.source_id == gateway_id {
                        state.move_tokens(gateway_id, &flow.target_id);
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    /// Move tokens toward end event
    fn move_to_end(
        state: &mut ExecutionState,
        diagram: &BPMNDiagram,
        _start_id: &str,
    ) -> Result<(), String> {
        // Simplified: move any remaining tokens to an end event
        for end_id in &diagram.end_event_ids {
            for token in &mut state.tokens {
                if !diagram.end_event_ids.contains(&token.location) {
                    token.location = end_id.clone();
                }
            }
        }

        Ok(())
    }

    /// Simulate execution trace and collect reachable states
    pub fn reachable_activities(diagram: &BPMNDiagram) -> Result<HashSet<String>, String> {
        diagram.validate()?;

        let mut reachable = HashSet::new();
        let mut queue = VecDeque::new();

        if let Some(start_id) = &diagram.start_event_id {
            queue.push_back(start_id.clone());
        }

        let mut visited = HashSet::new();

        while let Some(element_id) = queue.pop_front() {
            if visited.contains(&element_id) {
                continue;
            }
            visited.insert(element_id.clone());

            // Check if this is a task
            if let Some(task) = diagram.tasks.get(&element_id) {
                reachable.insert(task.name.clone());
            }

            // Add outgoing targets to queue
            for flow in diagram.flows.values() {
                if flow.source_id == element_id {
                    queue.push_back(flow.target_id.clone());
                }
            }
        }

        Ok(reachable)
    }
}

/// Test if a sequence of activities is valid according to BPMN rules
pub fn validate_sequence(diagram: &BPMNDiagram, sequence: &[&str]) -> Result<bool, String> {
    diagram.validate()?;

    // Simple validation: all activities must exist in diagram
    let activities: HashSet<_> = diagram.tasks.values().map(|t| t.name.as_str()).collect();

    for activity in sequence {
        if !activities.contains(activity) {
            return Ok(false);
        }
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_simple_diagram() -> BPMNDiagram {
        let mut diagram = BPMNDiagram::new("Test");
        let start = Event::new("Start", EventType::Start);
        let task1 = Task::new("Task1", TaskType::UserTask);
        let task2 = Task::new("Task2", TaskType::UserTask);
        let end = Event::new("End", EventType::End);

        let start_id = diagram.add_event(start);
        let task1_id = diagram.add_task(task1);
        let task2_id = diagram.add_task(task2);
        let end_id = diagram.add_event(end);

        diagram.add_flow(SequenceFlow::new(start_id, task1_id.clone()));
        diagram.add_flow(SequenceFlow::new(task1_id, task2_id.clone()));
        diagram.add_flow(SequenceFlow::new(task2_id, end_id));

        diagram
    }

    #[test]
    fn test_execution_state_creation() {
        let state = ExecutionState::new();
        assert!(state.tokens.is_empty());
        assert_eq!(state.current_step, 0);
    }

    #[test]
    fn test_token_creation() {
        let token = Token::new("location_1");
        assert_eq!(token.location, "location_1");
    }

    #[test]
    fn test_add_token_to_state() {
        let mut state = ExecutionState::new();
        let token = Token::new("location_1");
        state.add_token(token.clone());

        assert_eq!(state.tokens.len(), 1);
        assert!(state.has_tokens_at("location_1"));
    }

    #[test]
    fn test_move_tokens() {
        let mut state = ExecutionState::new();
        let token = Token::new("location_1");
        state.add_token(token);

        state.move_tokens("location_1", "location_2");
        assert!(!state.has_tokens_at("location_1"));
        assert!(state.has_tokens_at("location_2"));
    }

    #[test]
    fn test_simple_execution() {
        let diagram = create_simple_diagram();
        let result = BPMNExecutor::execute(&diagram, &["Task1", "Task2"]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execution_invalid_activity() {
        let diagram = create_simple_diagram();
        let result = BPMNExecutor::execute(&diagram, &["InvalidTask"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_reachable_activities() {
        let diagram = create_simple_diagram();
        let reachable = BPMNExecutor::reachable_activities(&diagram);

        assert!(reachable.is_ok());
        let reachable = reachable.unwrap();
        assert!(reachable.contains("Task1"));
        assert!(reachable.contains("Task2"));
    }

    #[test]
    fn test_validate_sequence() {
        let diagram = create_simple_diagram();
        let valid = validate_sequence(&diagram, &["Task1", "Task2"]);

        assert!(valid.is_ok());
        assert!(valid.unwrap());
    }

    #[test]
    fn test_validate_sequence_invalid() {
        let diagram = create_simple_diagram();
        let valid = validate_sequence(&diagram, &["Task1", "InvalidTask"]);

        assert!(valid.is_ok());
        assert!(!valid.unwrap());
    }

    #[test]
    fn test_parallel_gateway_execution() {
        let mut diagram = BPMNDiagram::new("Parallel Test");
        let start = Event::new("Start", EventType::Start);
        let task1 = Task::new("Task1", TaskType::UserTask);
        let task2 = Task::new("Task2", TaskType::UserTask);
        let end = Event::new("End", EventType::End);

        let start_id = diagram.add_event(start);
        let task1_id = diagram.add_task(task1);
        let task2_id = diagram.add_task(task2);
        let end_id = diagram.add_event(end);

        diagram.add_flow(SequenceFlow::new(start_id, task1_id.clone()));
        diagram.add_flow(SequenceFlow::new(task1_id, task2_id.clone()));
        diagram.add_flow(SequenceFlow::new(task2_id, end_id));

        assert!(diagram.validate().is_ok());
    }

    #[test]
    fn test_complete_task_tracking() {
        let mut state = ExecutionState::new();
        state.complete_task("task_1".to_string());
        assert!(state.completed_tasks.contains("task_1"));
    }
}
