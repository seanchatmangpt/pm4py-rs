use serde::{Deserialize, Serialize};
/// BPMN 2.0 Process Model
///
/// Complete representation of Business Process Model and Notation 2.0
/// including tasks, gateways, events, and flows.
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// BPMN Gateway types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GatewayType {
    /// Exclusive choice - exactly one outgoing sequence flow
    ExclusiveXor,
    /// Parallel split/join - all outgoing flows execute simultaneously
    Parallel,
    /// Inclusive choice - one or more outgoing flows
    Inclusive,
    /// Event-based gateway - choice based on events
    EventBased,
}

/// BPMN Event types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    /// Process start event
    Start,
    /// Process end event
    End,
    /// Intermediate event (can trigger or be triggered)
    Intermediate,
    /// Boundary event (attached to activity)
    Boundary,
}

/// BPMN Flow Object - base trait for all flowable objects
pub trait FlowObject {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
}

/// BPMN Task - a unit of work
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub task_type: TaskType,
    pub incoming: Vec<String>, // IDs of incoming flows
    pub outgoing: Vec<String>, // IDs of outgoing flows
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskType {
    UserTask,
    ServiceTask,
    AutomaticTask,
    ManualTask,
    SendTask,
    ReceiveTask,
    ScriptTask,
}

impl Task {
    pub fn new(name: impl Into<String>, task_type: TaskType) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            task_type,
            incoming: Vec::new(),
            outgoing: Vec::new(),
            properties: HashMap::new(),
        }
    }

    pub fn with_property(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.properties.insert(key.into(), value.into());
        self
    }
}

impl FlowObject for Task {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// BPMN Gateway - controls flow splitting/joining
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gateway {
    pub id: String,
    pub name: String,
    pub gateway_type: GatewayType,
    pub incoming: Vec<String>,
    pub outgoing: Vec<String>,
}

impl Gateway {
    pub fn new(name: impl Into<String>, gateway_type: GatewayType) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            gateway_type,
            incoming: Vec::new(),
            outgoing: Vec::new(),
        }
    }
}

impl FlowObject for Gateway {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// BPMN Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub name: String,
    pub event_type: EventType,
    pub incoming: Vec<String>,
    pub outgoing: Vec<String>,
}

impl Event {
    pub fn new(name: impl Into<String>, event_type: EventType) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            event_type,
            incoming: Vec::new(),
            outgoing: Vec::new(),
        }
    }
}

impl FlowObject for Event {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// BPMN Sequence Flow - connects flow objects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequenceFlow {
    pub id: String,
    pub name: Option<String>,
    pub source_id: String,
    pub target_id: String,
    pub condition: Option<String>,
}

impl SequenceFlow {
    pub fn new(source_id: impl Into<String>, target_id: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: None,
            source_id: source_id.into(),
            target_id: target_id.into(),
            condition: None,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_condition(mut self, condition: impl Into<String>) -> Self {
        self.condition = Some(condition.into());
        self
    }
}

/// BPMN Process diagram
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BPMNDiagram {
    pub id: String,
    pub name: String,
    pub tasks: HashMap<String, Task>,
    pub gateways: HashMap<String, Gateway>,
    pub events: HashMap<String, Event>,
    pub flows: HashMap<String, SequenceFlow>,
    pub start_event_id: Option<String>,
    pub end_event_ids: HashSet<String>,
}

impl BPMNDiagram {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            tasks: HashMap::new(),
            gateways: HashMap::new(),
            events: HashMap::new(),
            flows: HashMap::new(),
            start_event_id: None,
            end_event_ids: HashSet::new(),
        }
    }

    /// Add a task to the diagram
    pub fn add_task(&mut self, task: Task) -> String {
        let id = task.id.clone();
        self.tasks.insert(id.clone(), task);
        id
    }

    /// Add a gateway to the diagram
    pub fn add_gateway(&mut self, gateway: Gateway) -> String {
        let id = gateway.id.clone();
        self.gateways.insert(id.clone(), gateway);
        id
    }

    /// Add an event to the diagram
    pub fn add_event(&mut self, event: Event) -> String {
        let id = event.id.clone();
        if event.event_type == EventType::Start {
            self.start_event_id = Some(id.clone());
        }
        if event.event_type == EventType::End {
            self.end_event_ids.insert(id.clone());
        }
        self.events.insert(id.clone(), event);
        id
    }

    /// Add a sequence flow
    pub fn add_flow(&mut self, flow: SequenceFlow) -> String {
        let id = flow.id.clone();

        // Update incoming/outgoing references
        if let Some(task) = self.tasks.get_mut(&flow.source_id) {
            task.outgoing.push(id.clone());
        }
        if let Some(gateway) = self.gateways.get_mut(&flow.source_id) {
            gateway.outgoing.push(id.clone());
        }
        if let Some(event) = self.events.get_mut(&flow.source_id) {
            event.outgoing.push(id.clone());
        }

        if let Some(task) = self.tasks.get_mut(&flow.target_id) {
            task.incoming.push(id.clone());
        }
        if let Some(gateway) = self.gateways.get_mut(&flow.target_id) {
            gateway.incoming.push(id.clone());
        }
        if let Some(event) = self.events.get_mut(&flow.target_id) {
            event.incoming.push(id.clone());
        }

        self.flows.insert(id.clone(), flow);
        id
    }

    /// Get all activity names (tasks)
    pub fn activities(&self) -> Vec<String> {
        self.tasks.values().map(|t| t.name.clone()).collect()
    }

    /// Check if diagram is well-formed
    pub fn validate(&self) -> Result<(), String> {
        // Check start event exists
        if self.start_event_id.is_none() {
            return Err("No start event defined".to_string());
        }

        // Check end events exist
        if self.end_event_ids.is_empty() {
            return Err("No end events defined".to_string());
        }

        // Check all flows reference valid elements
        for flow in self.flows.values() {
            let source_exists = self.tasks.contains_key(&flow.source_id)
                || self.gateways.contains_key(&flow.source_id)
                || self.events.contains_key(&flow.source_id);

            let target_exists = self.tasks.contains_key(&flow.target_id)
                || self.gateways.contains_key(&flow.target_id)
                || self.events.contains_key(&flow.target_id);

            if !source_exists {
                return Err(format!("Flow source {} not found", flow.source_id));
            }
            if !target_exists {
                return Err(format!("Flow target {} not found", flow.target_id));
            }
        }

        Ok(())
    }

    /// Get all elements in topological order (best effort)
    pub fn topological_sort(&self) -> Result<Vec<String>, String> {
        let mut sorted = Vec::new();
        let mut visited = HashSet::new();
        let mut visiting = HashSet::new();

        let mut all_ids = Vec::new();
        if let Some(id) = &self.start_event_id {
            all_ids.push(id.clone());
        }
        all_ids.extend(self.tasks.keys().cloned());
        all_ids.extend(self.gateways.keys().cloned());
        all_ids.extend(self.events.keys().cloned());

        for id in all_ids {
            if !visited.contains(&id) {
                self._dfs_sort(&id, &mut visited, &mut visiting, &mut sorted)?;
            }
        }

        Ok(sorted)
    }

    fn _dfs_sort(
        &self,
        id: &str,
        visited: &mut HashSet<String>,
        visiting: &mut HashSet<String>,
        sorted: &mut Vec<String>,
    ) -> Result<(), String> {
        if visited.contains(id) {
            return Ok(());
        }
        if visiting.contains(id) {
            return Err("Cycle detected in flow".to_string());
        }

        visiting.insert(id.to_string());

        // Get outgoing flows
        let outgoing: Vec<_> = self
            .flows
            .values()
            .filter(|f| f.source_id == id)
            .map(|f| f.target_id.clone())
            .collect();

        for target in outgoing {
            self._dfs_sort(&target, visited, visiting, sorted)?;
        }

        visiting.remove(id);
        visited.insert(id.to_string());
        sorted.push(id.to_string());

        Ok(())
    }
}

impl Default for BPMNDiagram {
    fn default() -> Self {
        Self::new("Process")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bpmn_creation() {
        let diagram = BPMNDiagram::new("Test Process");
        assert_eq!(diagram.name, "Test Process");
        assert!(diagram.tasks.is_empty());
    }

    #[test]
    fn test_add_task() {
        let mut diagram = BPMNDiagram::new("Test");
        let task = Task::new("Do Something", TaskType::UserTask);
        let id = diagram.add_task(task);

        assert!(diagram.tasks.contains_key(&id));
    }

    #[test]
    fn test_add_gateway() {
        let mut diagram = BPMNDiagram::new("Test");
        let gateway = Gateway::new("Decision", GatewayType::ExclusiveXor);
        let id = diagram.add_gateway(gateway);

        assert!(diagram.gateways.contains_key(&id));
    }

    #[test]
    fn test_add_events() {
        let mut diagram = BPMNDiagram::new("Test");
        let start = Event::new("Start", EventType::Start);
        let end = Event::new("End", EventType::End);

        let start_id = diagram.add_event(start);
        let end_id = diagram.add_event(end);

        assert_eq!(diagram.start_event_id, Some(start_id));
        assert!(diagram.end_event_ids.contains(&end_id));
    }

    #[test]
    fn test_add_flow() {
        let mut diagram = BPMNDiagram::new("Test");
        let task1 = Task::new("Task 1", TaskType::UserTask);
        let task2 = Task::new("Task 2", TaskType::UserTask);

        let id1 = diagram.add_task(task1);
        let id2 = diagram.add_task(task2);

        let flow = SequenceFlow::new(id1.clone(), id2.clone());
        diagram.add_flow(flow);

        assert_eq!(diagram.tasks.get(&id1).unwrap().outgoing.len(), 1);
        assert_eq!(diagram.tasks.get(&id2).unwrap().incoming.len(), 1);
    }

    #[test]
    fn test_activities() {
        let mut diagram = BPMNDiagram::new("Test");
        let task1 = Task::new("Activity A", TaskType::UserTask);
        let task2 = Task::new("Activity B", TaskType::UserTask);

        diagram.add_task(task1);
        diagram.add_task(task2);

        let activities = diagram.activities();
        assert_eq!(activities.len(), 2);
    }

    #[test]
    fn test_validate_missing_start() {
        let diagram = BPMNDiagram::new("Test");
        assert!(diagram.validate().is_err());
    }

    #[test]
    fn test_validate_success() {
        let mut diagram = BPMNDiagram::new("Test");
        let start = Event::new("Start", EventType::Start);
        let end = Event::new("End", EventType::End);
        let task = Task::new("Task", TaskType::UserTask);

        let start_id = diagram.add_event(start);
        let task_id = diagram.add_task(task);
        let end_id = diagram.add_event(end);

        diagram.add_flow(SequenceFlow::new(start_id, task_id.clone()));
        diagram.add_flow(SequenceFlow::new(task_id, end_id));

        assert!(diagram.validate().is_ok());
    }

    #[test]
    fn test_gateway_type_exclusive() {
        let gateway = Gateway::new("Decision", GatewayType::ExclusiveXor);
        assert_eq!(gateway.gateway_type, GatewayType::ExclusiveXor);
    }

    #[test]
    fn test_gateway_type_parallel() {
        let gateway = Gateway::new("Fork", GatewayType::Parallel);
        assert_eq!(gateway.gateway_type, GatewayType::Parallel);
    }

    #[test]
    fn test_task_with_property() {
        let task = Task::new("Task", TaskType::ServiceTask).with_property("handler", "my_service");
        assert_eq!(
            task.properties.get("handler").map(|s| s.as_str()),
            Some("my_service")
        );
    }

    #[test]
    fn test_sequence_flow_with_condition() {
        let flow = SequenceFlow::new("source", "target").with_condition("status == 'approved'");
        assert_eq!(flow.condition, Some("status == 'approved'".to_string()));
    }
}
