/// Statistical analysis of process trees
///
/// This module provides metrics and analysis functions for process trees,
/// including complexity measures, operator frequencies, and structural analysis.
use crate::models::process_tree::{ProcessTree, ProcessTreeNode, TreeOperator};
use std::collections::HashMap;

/// Structure containing comprehensive tree statistics
#[derive(Debug, Clone)]
pub struct TreeStatistics {
    /// Total number of nodes in the tree
    pub node_count: usize,
    /// Number of leaf nodes (activities)
    pub leaf_count: usize,
    /// Number of operator nodes
    pub operator_count: usize,
    /// Depth of the tree
    pub depth: usize,
    /// Width of the tree (max nodes at any level)
    pub width: usize,
    /// Frequency of each operator type
    pub operator_frequencies: HashMap<String, usize>,
    /// Number of unique activities
    pub unique_activities: usize,
    /// Average branching factor
    pub avg_branching_factor: f64,
}

impl TreeStatistics {
    /// Calculate statistics for a process tree
    pub fn from_tree(tree: &ProcessTree) -> Self {
        let node_count = Self::count_nodes(&tree.root);
        let leaf_count = tree.leaf_count();
        let operator_count = tree.operator_count();
        let depth = tree.depth();
        let width = tree.width();
        let unique_activities = tree.activities().len();

        let operator_freq = tree.operator_frequencies();
        let operator_frequencies = operator_freq
            .iter()
            .map(|(op, count)| (op.as_str().to_string(), *count))
            .collect();

        let avg_branching_factor = Self::calculate_avg_branching(&tree.root);

        Self {
            node_count,
            leaf_count,
            operator_count,
            depth,
            width,
            operator_frequencies,
            unique_activities,
            avg_branching_factor,
        }
    }

    fn count_nodes(node: &ProcessTreeNode) -> usize {
        match node {
            ProcessTreeNode::Activity(_) => 1,
            ProcessTreeNode::Operator { children, .. } => {
                1 + children.iter().map(Self::count_nodes).sum::<usize>()
            }
        }
    }

    fn calculate_avg_branching(node: &ProcessTreeNode) -> f64 {
        let mut total_branching = 0.0;
        let mut operator_count = 0;

        Self::collect_branching(node, &mut total_branching, &mut operator_count);

        if operator_count > 0 {
            total_branching / operator_count as f64
        } else {
            0.0
        }
    }

    fn collect_branching(node: &ProcessTreeNode, total: &mut f64, count: &mut usize) {
        match node {
            ProcessTreeNode::Activity(_) => {}
            ProcessTreeNode::Operator { children, .. } => {
                *total += children.len() as f64;
                *count += 1;

                for child in children {
                    Self::collect_branching(child, total, count);
                }
            }
        }
    }

    /// Get a human-readable summary
    pub fn summary(&self) -> String {
        let mut operators = self
            .operator_frequencies
            .iter()
            .map(|(op, count)| format!("{}={}", op, count))
            .collect::<Vec<_>>();
        operators.sort();

        format!(
            "Nodes: {}, Leaves: {}, Operators: {}, Depth: {}, Width: {}, Activities: {}, Operators: {{{}}}",
            self.node_count,
            self.leaf_count,
            self.operator_count,
            self.depth,
            self.width,
            self.unique_activities,
            operators.join(", ")
        )
    }
}

/// Detailed process tree metrics
#[derive(Debug, Clone)]
pub struct TreeMetrics {
    /// Cyclicity ratio (0.0 to 1.0)
    pub cyclicity: f64,
    /// Coupling measure (how tightly connected)
    pub coupling: f64,
    /// Density of the tree
    pub density: f64,
    /// Complexity score based on multiple factors
    pub complexity_score: f64,
}

impl TreeMetrics {
    /// Calculate metrics for a process tree
    pub fn from_tree(tree: &ProcessTree) -> Self {
        let stats = TreeStatistics::from_tree(tree);

        // Cyclicity: presence of loop operators
        let loop_count = stats.operator_frequencies.get("∗").copied().unwrap_or(0);
        let cyclicity = if stats.operator_count > 0 {
            loop_count as f64 / stats.operator_count as f64
        } else {
            0.0
        };

        // Coupling: based on choice/parallel operators
        let choice_count = stats.operator_frequencies.get("×").copied().unwrap_or(0);
        let parallel_count = stats.operator_frequencies.get("∧").copied().unwrap_or(0);
        let coupling = if stats.operator_count > 0 {
            (choice_count + parallel_count) as f64 / stats.operator_count as f64
        } else {
            0.0
        };

        // Density: ratio of operators to total nodes
        let density = if stats.node_count > 0 {
            stats.operator_count as f64 / stats.node_count as f64
        } else {
            0.0
        };

        // Complexity: weighted combination of factors
        let complexity_score = (stats.depth as f64 / 10.0) * 0.3
            + (stats.avg_branching_factor / 5.0) * 0.3
            + coupling * 0.2
            + cyclicity * 0.2;

        Self {
            cyclicity,
            coupling,
            density,
            complexity_score: complexity_score.min(1.0),
        }
    }

    /// Get complexity level (Low, Medium, High)
    pub fn complexity_level(&self) -> &str {
        match self.complexity_score {
            c if c < 0.33 => "Low",
            c if c < 0.67 => "Medium",
            _ => "High",
        }
    }

    /// Get a human-readable summary
    pub fn summary(&self) -> String {
        format!(
            "Cyclicity: {:.2}, Coupling: {:.2}, Density: {:.2}, Complexity: {:.2} ({})",
            self.cyclicity,
            self.coupling,
            self.density,
            self.complexity_score,
            self.complexity_level()
        )
    }
}

/// Pattern analysis for process trees
#[derive(Debug, Clone)]
pub struct TreePattern {
    /// Whether tree is purely sequential
    pub is_sequential: bool,
    /// Whether tree has any parallelism
    pub has_parallel: bool,
    /// Whether tree has any branching/choice
    pub has_choice: bool,
    /// Whether tree has any loops
    pub has_loops: bool,
    /// Maximum nesting depth of operators
    pub max_operator_depth: usize,
}

impl TreePattern {
    /// Analyze patterns in a process tree
    pub fn from_tree(tree: &ProcessTree) -> Self {
        let mut pattern = Self {
            is_sequential: true,
            has_parallel: false,
            has_choice: false,
            has_loops: false,
            max_operator_depth: 0,
        };

        Self::analyze(&tree.root, &mut pattern, 0);

        pattern
    }

    fn analyze(node: &ProcessTreeNode, pattern: &mut TreePattern, depth: usize) {
        match node {
            ProcessTreeNode::Activity(_) => {}
            ProcessTreeNode::Operator {
                operator, children, ..
            } => {
                pattern.max_operator_depth = pattern.max_operator_depth.max(depth + 1);

                match operator {
                    TreeOperator::Sequence => {}
                    TreeOperator::Choice => {
                        pattern.is_sequential = false;
                        pattern.has_choice = true;
                    }
                    TreeOperator::Parallel => {
                        pattern.is_sequential = false;
                        pattern.has_parallel = true;
                    }
                    TreeOperator::Loop => {
                        pattern.has_loops = true;
                    }
                }

                for child in children {
                    Self::analyze(child, pattern, depth + 1);
                }
            }
        }
    }

    /// Get description of patterns found
    pub fn description(&self) -> String {
        let mut parts = Vec::new();

        if self.is_sequential {
            parts.push("purely sequential");
        }

        if self.has_choice {
            parts.push("contains choices");
        }

        if self.has_parallel {
            parts.push("contains parallelism");
        }

        if self.has_loops {
            parts.push("contains loops");
        }

        if parts.is_empty() {
            "simple single activity".to_string()
        } else {
            parts.join(", ")
        }
    }
}

/// Comprehensive analysis function
pub fn analyze_tree(tree: &ProcessTree) -> (TreeStatistics, TreeMetrics, TreePattern) {
    let stats = TreeStatistics::from_tree(tree);
    let metrics = TreeMetrics::from_tree(tree);
    let pattern = TreePattern::from_tree(tree);

    (stats, metrics, pattern)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_tree_statistics() {
        let tree = ProcessTree::new(ProcessTreeNode::activity("A"));
        let stats = TreeStatistics::from_tree(&tree);

        assert_eq!(stats.leaf_count, 1);
        assert_eq!(stats.operator_count, 0);
        assert_eq!(stats.unique_activities, 1);
    }

    #[test]
    fn test_sequence_tree_statistics() {
        let tree = ProcessTree::new(ProcessTreeNode::sequence(vec![
            ProcessTreeNode::activity("A"),
            ProcessTreeNode::activity("B"),
            ProcessTreeNode::activity("C"),
        ]));

        let stats = TreeStatistics::from_tree(&tree);

        assert_eq!(stats.leaf_count, 3);
        assert_eq!(stats.operator_count, 1);
        assert_eq!(stats.unique_activities, 3);
    }

    #[test]
    fn test_choice_tree_statistics() {
        let tree = ProcessTree::new(ProcessTreeNode::choice(vec![
            ProcessTreeNode::activity("A"),
            ProcessTreeNode::activity("B"),
        ]));

        let stats = TreeStatistics::from_tree(&tree);

        assert_eq!(stats.leaf_count, 2);
        assert_eq!(stats.operator_count, 1);
        assert_eq!(stats.operator_frequencies.get("×"), Some(&1));
    }

    #[test]
    fn test_parallel_tree_statistics() {
        let tree = ProcessTree::new(ProcessTreeNode::parallel(vec![
            ProcessTreeNode::activity("A"),
            ProcessTreeNode::activity("B"),
        ]));

        let stats = TreeStatistics::from_tree(&tree);

        assert_eq!(stats.operator_frequencies.get("∧"), Some(&1));
    }

    #[test]
    fn test_loop_tree_statistics() {
        let tree = ProcessTree::new(ProcessTreeNode::loop_node(
            ProcessTreeNode::activity("A"),
            ProcessTreeNode::activity("B"),
        ));

        let stats = TreeStatistics::from_tree(&tree);

        assert_eq!(stats.operator_frequencies.get("∗"), Some(&1));
    }

    #[test]
    fn test_complex_tree_statistics() {
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

        let stats = TreeStatistics::from_tree(&tree);

        assert!(stats.node_count > 0);
        assert_eq!(stats.unique_activities, 5);
        assert_eq!(stats.depth, 3);
    }

    #[test]
    fn test_tree_metrics_complexity() {
        let simple = ProcessTree::new(ProcessTreeNode::activity("A"));
        let simple_metrics = TreeMetrics::from_tree(&simple);

        let complex = ProcessTree::new(ProcessTreeNode::sequence(vec![
            ProcessTreeNode::choice(vec![
                ProcessTreeNode::activity("A"),
                ProcessTreeNode::activity("B"),
            ]),
            ProcessTreeNode::parallel(vec![
                ProcessTreeNode::activity("C"),
                ProcessTreeNode::activity("D"),
            ]),
        ]));
        let complex_metrics = TreeMetrics::from_tree(&complex);

        assert!(complex_metrics.complexity_score >= simple_metrics.complexity_score);
    }

    #[test]
    fn test_pattern_sequential() {
        let tree = ProcessTree::new(ProcessTreeNode::sequence(vec![
            ProcessTreeNode::activity("A"),
            ProcessTreeNode::activity("B"),
        ]));

        let pattern = TreePattern::from_tree(&tree);

        assert!(pattern.is_sequential);
        assert!(!pattern.has_choice);
        assert!(!pattern.has_parallel);
        assert!(!pattern.has_loops);
    }

    #[test]
    fn test_pattern_with_choice() {
        let tree = ProcessTree::new(ProcessTreeNode::choice(vec![
            ProcessTreeNode::activity("A"),
            ProcessTreeNode::activity("B"),
        ]));

        let pattern = TreePattern::from_tree(&tree);

        assert!(!pattern.is_sequential);
        assert!(pattern.has_choice);
    }

    #[test]
    fn test_pattern_with_loop() {
        let tree = ProcessTree::new(ProcessTreeNode::loop_node(
            ProcessTreeNode::activity("A"),
            ProcessTreeNode::activity("B"),
        ));

        let pattern = TreePattern::from_tree(&tree);

        assert!(pattern.has_loops);
    }

    #[test]
    fn test_tree_metrics_summary() {
        let tree = ProcessTree::new(ProcessTreeNode::sequence(vec![
            ProcessTreeNode::activity("A"),
            ProcessTreeNode::choice(vec![
                ProcessTreeNode::activity("B"),
                ProcessTreeNode::activity("C"),
            ]),
        ]));

        let metrics = TreeMetrics::from_tree(&tree);
        let summary = metrics.summary();

        assert!(summary.contains("Cyclicity"));
        assert!(summary.contains("Coupling"));
    }

    #[test]
    fn test_pattern_description() {
        let tree = ProcessTree::new(ProcessTreeNode::sequence(vec![
            ProcessTreeNode::activity("A"),
            ProcessTreeNode::choice(vec![
                ProcessTreeNode::activity("B"),
                ProcessTreeNode::activity("C"),
            ]),
        ]));

        let pattern = TreePattern::from_tree(&tree);
        let desc = pattern.description();

        assert!(desc.contains("choice"));
    }
}
