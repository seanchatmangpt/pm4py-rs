/// Process Tree representation and operations
///
/// A process tree is a hierarchical representation of a process model
/// using four basic operators:
/// - Sequence (→): Activities must occur in order
/// - Choice (×): One of several branches is taken
/// - Parallel (∧): All branches must occur (in any order)
/// - Loop (∗): An activity or subtree can be repeated
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents an operator node in a process tree
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TreeOperator {
    /// Sequence operator (→): All children must execute in order
    Sequence,
    /// Choice operator (×): Exactly one child executes
    Choice,
    /// Parallel operator (∧): All children execute concurrently (any order)
    Parallel,
    /// Loop operator (∗): First child repeats, second is exit condition
    Loop,
}

impl TreeOperator {
    /// Get string representation of the operator
    pub fn as_str(&self) -> &str {
        match self {
            TreeOperator::Sequence => "→",
            TreeOperator::Choice => "×",
            TreeOperator::Parallel => "∧",
            TreeOperator::Loop => "∗",
        }
    }
}

/// Represents a node in a process tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessTreeNode {
    /// Leaf node representing an activity
    Activity(String),
    /// Operator node with children
    Operator {
        operator: TreeOperator,
        children: Vec<ProcessTreeNode>,
        id: String,
    },
}

impl ProcessTreeNode {
    /// Create a new activity leaf node
    pub fn activity(name: impl Into<String>) -> Self {
        ProcessTreeNode::Activity(name.into())
    }

    /// Create a new operator node
    pub fn operator(op: TreeOperator, children: Vec<ProcessTreeNode>) -> Self {
        ProcessTreeNode::Operator {
            operator: op,
            children,
            id: Uuid::new_v4().to_string(),
        }
    }

    /// Create a sequence operator with children
    pub fn sequence(children: Vec<ProcessTreeNode>) -> Self {
        Self::operator(TreeOperator::Sequence, children)
    }

    /// Create a choice operator with children
    pub fn choice(children: Vec<ProcessTreeNode>) -> Self {
        Self::operator(TreeOperator::Choice, children)
    }

    /// Create a parallel operator with children
    pub fn parallel(children: Vec<ProcessTreeNode>) -> Self {
        Self::operator(TreeOperator::Parallel, children)
    }

    /// Create a loop operator with body and exit
    pub fn loop_node(body: ProcessTreeNode, exit: ProcessTreeNode) -> Self {
        Self::operator(TreeOperator::Loop, vec![body, exit])
    }

    /// Check if this is a leaf node (activity)
    pub fn is_activity(&self) -> bool {
        matches!(self, ProcessTreeNode::Activity(_))
    }

    /// Check if this is an operator node
    pub fn is_operator(&self) -> bool {
        matches!(self, ProcessTreeNode::Operator { .. })
    }

    /// Get all activities in this subtree
    pub fn activities(&self) -> Vec<String> {
        let mut activities = Vec::new();
        self.collect_activities(&mut activities);
        activities.sort();
        activities.dedup();
        activities
    }

    fn collect_activities(&self, activities: &mut Vec<String>) {
        match self {
            ProcessTreeNode::Activity(name) => {
                activities.push(name.clone());
            }
            ProcessTreeNode::Operator { children, .. } => {
                for child in children {
                    child.collect_activities(activities);
                }
            }
        }
    }

    /// Get number of operators in the tree
    pub fn operator_count(&self) -> usize {
        match self {
            ProcessTreeNode::Activity(_) => 0,
            ProcessTreeNode::Operator { children, .. } => {
                1 + children.iter().map(|c| c.operator_count()).sum::<usize>()
            }
        }
    }

    /// Get number of leaf nodes (activities)
    pub fn leaf_count(&self) -> usize {
        match self {
            ProcessTreeNode::Activity(_) => 1,
            ProcessTreeNode::Operator { children, .. } => {
                children.iter().map(|c| c.leaf_count()).sum()
            }
        }
    }

    /// Get depth of the tree
    pub fn depth(&self) -> usize {
        match self {
            ProcessTreeNode::Activity(_) => 1,
            ProcessTreeNode::Operator { children, .. } => {
                1 + children.iter().map(|c| c.depth()).max().unwrap_or(0)
            }
        }
    }

    /// Get width (maximum number of children at any level)
    pub fn width(&self) -> usize {
        self.width_recursive().0
    }

    fn width_recursive(&self) -> (usize, usize) {
        match self {
            ProcessTreeNode::Activity(_) => (1, 1),
            ProcessTreeNode::Operator { children, .. } => {
                let child_count = children.len();
                let max_child_width = children
                    .iter()
                    .map(|c| c.width_recursive().1)
                    .max()
                    .unwrap_or(0);
                (
                    max(child_count, max_child_width),
                    max(child_count, max_child_width),
                )
            }
        }
    }

    /// Validate the tree structure
    /// Returns true if valid, false otherwise
    pub fn is_valid(&self) -> bool {
        self.is_valid_recursive().0
    }

    fn is_valid_recursive(&self) -> (bool, usize) {
        match self {
            ProcessTreeNode::Activity(_) => (true, 1),
            ProcessTreeNode::Operator {
                operator, children, ..
            } => {
                // Operators must have children
                if children.is_empty() {
                    return (false, 0);
                }

                // Loop operator must have exactly 2 children
                if matches!(operator, TreeOperator::Loop) && children.len() != 2 {
                    return (false, 0);
                }

                // Check all children
                for child in children {
                    let (is_valid, _) = child.is_valid_recursive();
                    if !is_valid {
                        return (false, 0);
                    }
                }

                (true, children.len())
            }
        }
    }

    /// Simplify the tree by removing trivial operators
    /// (operators with single children or redundant nesting)
    pub fn simplify(&mut self) -> bool {
        match self {
            ProcessTreeNode::Activity(_) => false,
            ProcessTreeNode::Operator { children, .. } => {
                let mut modified = false;

                // Simplify children first
                for child in children.iter_mut() {
                    if child.simplify() {
                        modified = true;
                    }
                }

                // Remove single-child operators (replace with child)
                if children.len() == 1 {
                    let child = children.remove(0);
                    *self = child;
                    modified = true;
                }

                modified
            }
        }
    }

    /// Get a string representation for debugging
    pub fn to_string_recursive(&self) -> String {
        match self {
            ProcessTreeNode::Activity(name) => name.clone(),
            ProcessTreeNode::Operator {
                operator, children, ..
            } => {
                let child_strs: Vec<_> = children.iter().map(|c| c.to_string_recursive()).collect();
                format!("{}({})", operator.as_str(), child_strs.join(", "))
            }
        }
    }
}

impl PartialEq for ProcessTreeNode {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ProcessTreeNode::Activity(a), ProcessTreeNode::Activity(b)) => a == b,
            (
                ProcessTreeNode::Operator {
                    operator: op1,
                    children: ch1,
                    ..
                },
                ProcessTreeNode::Operator {
                    operator: op2,
                    children: ch2,
                    ..
                },
            ) => op1 == op2 && ch1 == ch2,
            _ => false,
        }
    }
}

/// Represents a complete process tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessTree {
    /// Root node of the tree
    pub root: ProcessTreeNode,
    /// Optional tree name/label
    pub name: Option<String>,
}

impl ProcessTree {
    /// Create a new process tree with the given root node
    pub fn new(root: ProcessTreeNode) -> Self {
        Self { root, name: None }
    }

    /// Create a new process tree with a name
    pub fn with_name(root: ProcessTreeNode, name: impl Into<String>) -> Self {
        Self {
            root,
            name: Some(name.into()),
        }
    }

    /// Get all activities in the tree
    pub fn activities(&self) -> Vec<String> {
        self.root.activities()
    }

    /// Get the depth of the tree
    pub fn depth(&self) -> usize {
        self.root.depth()
    }

    /// Get the width of the tree
    pub fn width(&self) -> usize {
        self.root.width()
    }

    /// Get the number of leaf nodes
    pub fn leaf_count(&self) -> usize {
        self.root.leaf_count()
    }

    /// Get the total number of operators
    pub fn operator_count(&self) -> usize {
        self.root.operator_count()
    }

    /// Count operators by type
    pub fn operator_frequencies(&self) -> std::collections::HashMap<TreeOperator, usize> {
        let mut counts = std::collections::HashMap::new();
        self.count_operators_recursive(&self.root, &mut counts);
        counts
    }

    fn count_operators_recursive(
        &self,
        node: &ProcessTreeNode,
        counts: &mut std::collections::HashMap<TreeOperator, usize>,
    ) {
        match node {
            ProcessTreeNode::Activity(_) => {}
            ProcessTreeNode::Operator {
                operator, children, ..
            } => {
                *counts.entry(*operator).or_insert(0) += 1;
                for child in children {
                    self.count_operators_recursive(child, counts);
                }
            }
        }
    }

    /// Check if the tree is valid
    pub fn is_valid(&self) -> bool {
        self.root.is_valid()
    }

    /// Simplify the tree
    pub fn simplify(&mut self) {
        self.root.simplify();
    }

    /// Get all possible traces (simplified - returns first traces only for large trees)
    pub fn all_traces(&self) -> Vec<Vec<String>> {
        self.traces_recursive(&self.root, 100)
    }

    fn traces_recursive(&self, node: &ProcessTreeNode, max_traces: usize) -> Vec<Vec<String>> {
        match node {
            ProcessTreeNode::Activity(name) => {
                vec![vec![name.clone()]]
            }
            ProcessTreeNode::Operator {
                operator, children, ..
            } => {
                match operator {
                    TreeOperator::Sequence => {
                        // All children in sequence
                        let mut all_traces = vec![vec![]];
                        for child in children {
                            let child_traces = self.traces_recursive(child, max_traces);
                            let mut new_traces = Vec::new();
                            for trace in all_traces {
                                for child_trace in &child_traces {
                                    let mut new_trace = trace.clone();
                                    new_trace.extend(child_trace.clone());
                                    new_traces.push(new_trace);
                                    if new_traces.len() >= max_traces {
                                        return new_traces;
                                    }
                                }
                            }
                            all_traces = new_traces;
                        }
                        all_traces
                    }
                    TreeOperator::Choice => {
                        // Pick one child
                        let mut all_traces = Vec::new();
                        for child in children {
                            let child_traces = self.traces_recursive(child, max_traces);
                            all_traces.extend(child_traces);
                            if all_traces.len() >= max_traces {
                                return all_traces.into_iter().take(max_traces).collect();
                            }
                        }
                        all_traces
                    }
                    TreeOperator::Parallel => {
                        // All children in any order (simplified: just return sequence)
                        let mut all_traces = vec![vec![]];
                        for child in children {
                            let child_traces = self.traces_recursive(child, max_traces);
                            let mut new_traces = Vec::new();
                            for trace in all_traces {
                                for child_trace in &child_traces {
                                    let mut new_trace = trace.clone();
                                    new_trace.extend(child_trace.clone());
                                    new_traces.push(new_trace);
                                    if new_traces.len() >= max_traces {
                                        return new_traces;
                                    }
                                }
                            }
                            all_traces = new_traces;
                        }
                        all_traces
                    }
                    TreeOperator::Loop => {
                        // Body followed by exit (simplified: return both)
                        if children.len() == 2 {
                            let body_traces = self.traces_recursive(&children[0], max_traces);
                            let exit_traces = self.traces_recursive(&children[1], max_traces);

                            let mut all_traces = Vec::new();
                            // Just body
                            all_traces.extend(body_traces.iter().cloned());

                            // Body + Exit
                            for body_trace in &body_traces {
                                for exit_trace in &exit_traces {
                                    let mut combined = body_trace.clone();
                                    combined.extend(exit_trace.clone());
                                    all_traces.push(combined);
                                    if all_traces.len() >= max_traces {
                                        return all_traces;
                                    }
                                }
                            }
                            all_traces
                        } else {
                            vec![vec![]]
                        }
                    }
                }
            }
        }
    }
}

impl Default for ProcessTree {
    fn default() -> Self {
        Self {
            root: ProcessTreeNode::activity("SKIP"),
            name: None,
        }
    }
}

fn max(a: usize, b: usize) -> usize {
    if a > b {
        a
    } else {
        b
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_activity_node_creation() {
        let node = ProcessTreeNode::activity("A");
        assert!(node.is_activity());
        assert!(!node.is_operator());
        assert_eq!(node.activities(), vec!["A"]);
    }

    #[test]
    fn test_sequence_node() {
        let node = ProcessTreeNode::sequence(vec![
            ProcessTreeNode::activity("A"),
            ProcessTreeNode::activity("B"),
            ProcessTreeNode::activity("C"),
        ]);

        assert!(node.is_operator());
        assert_eq!(node.activities(), vec!["A", "B", "C"]);
        assert_eq!(node.leaf_count(), 3);
        assert_eq!(node.depth(), 2);
    }

    #[test]
    fn test_choice_node() {
        let node = ProcessTreeNode::choice(vec![
            ProcessTreeNode::activity("A"),
            ProcessTreeNode::activity("B"),
        ]);

        assert!(node.is_operator());
        assert_eq!(node.activities(), vec!["A", "B"]);
        assert_eq!(node.leaf_count(), 2);
    }

    #[test]
    fn test_parallel_node() {
        let node = ProcessTreeNode::parallel(vec![
            ProcessTreeNode::activity("A"),
            ProcessTreeNode::activity("B"),
        ]);

        assert!(node.is_operator());
        assert_eq!(node.leaf_count(), 2);
    }

    #[test]
    fn test_loop_node() {
        let node = ProcessTreeNode::loop_node(
            ProcessTreeNode::activity("A"),
            ProcessTreeNode::activity("B"),
        );

        assert!(node.is_operator());
        assert_eq!(node.activities(), vec!["A", "B"]);
    }

    #[test]
    fn test_nested_tree() {
        let tree = ProcessTree::new(ProcessTreeNode::sequence(vec![
            ProcessTreeNode::activity("A"),
            ProcessTreeNode::choice(vec![
                ProcessTreeNode::activity("B"),
                ProcessTreeNode::activity("C"),
            ]),
            ProcessTreeNode::activity("D"),
        ]));

        assert_eq!(tree.activities(), vec!["A", "B", "C", "D"]);
        assert_eq!(tree.leaf_count(), 4);
        assert_eq!(tree.depth(), 3);
        assert!(tree.is_valid());
    }

    #[test]
    fn test_tree_validation() {
        // Valid tree
        let tree = ProcessTree::new(ProcessTreeNode::sequence(vec![
            ProcessTreeNode::activity("A"),
            ProcessTreeNode::activity("B"),
        ]));
        assert!(tree.is_valid());

        // Invalid: loop with wrong number of children
        let invalid_loop =
            ProcessTreeNode::operator(TreeOperator::Loop, vec![ProcessTreeNode::activity("A")]);
        assert!(!invalid_loop.is_valid());
    }

    #[test]
    fn test_operator_frequencies() {
        let tree = ProcessTree::new(ProcessTreeNode::sequence(vec![
            ProcessTreeNode::activity("A"),
            ProcessTreeNode::choice(vec![
                ProcessTreeNode::activity("B"),
                ProcessTreeNode::activity("C"),
            ]),
            ProcessTreeNode::parallel(vec![
                ProcessTreeNode::activity("D"),
                ProcessTreeNode::activity("E"),
            ]),
        ]));

        let freqs = tree.operator_frequencies();
        assert_eq!(freqs.get(&TreeOperator::Sequence), Some(&1));
        assert_eq!(freqs.get(&TreeOperator::Choice), Some(&1));
        assert_eq!(freqs.get(&TreeOperator::Parallel), Some(&1));
    }

    #[test]
    fn test_simplify() {
        // Single child operator should be removed
        let mut node = ProcessTreeNode::sequence(vec![ProcessTreeNode::activity("A")]);
        node.simplify();
        assert!(node.is_activity());
    }

    #[test]
    fn test_depth_and_width() {
        let tree = ProcessTree::new(ProcessTreeNode::sequence(vec![
            ProcessTreeNode::parallel(vec![
                ProcessTreeNode::activity("A"),
                ProcessTreeNode::activity("B"),
                ProcessTreeNode::activity("C"),
            ]),
            ProcessTreeNode::activity("D"),
        ]));

        assert_eq!(tree.depth(), 3);
        assert_eq!(tree.width(), 3);
    }

    #[test]
    fn test_traces_sequence() {
        let tree = ProcessTree::new(ProcessTreeNode::sequence(vec![
            ProcessTreeNode::activity("A"),
            ProcessTreeNode::activity("B"),
        ]));

        let traces = tree.all_traces();
        assert_eq!(traces.len(), 1);
        assert_eq!(traces[0], vec!["A", "B"]);
    }

    #[test]
    fn test_traces_choice() {
        let tree = ProcessTree::new(ProcessTreeNode::choice(vec![
            ProcessTreeNode::activity("A"),
            ProcessTreeNode::activity("B"),
        ]));

        let traces = tree.all_traces();
        assert_eq!(traces.len(), 2);
        assert!(traces.contains(&vec!["A".to_string()]) || traces.contains(&vec!["B".to_string()]));
    }

    #[test]
    fn test_to_string_recursive() {
        let tree = ProcessTree::new(ProcessTreeNode::sequence(vec![
            ProcessTreeNode::activity("A"),
            ProcessTreeNode::activity("B"),
        ]));

        let s = tree.root.to_string_recursive();
        assert!(s.contains("→"));
        assert!(s.contains("A"));
        assert!(s.contains("B"));
    }
}
