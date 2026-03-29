//! Prefix Tree (Variant) Discovery
//!
//! A prefix tree (also called a trie) organizes traces by their common prefixes,
//! making it useful for variant analysis and log compression.

use crate::log::EventLog;
use std::collections::{HashMap, HashSet};

/// A node in the prefix tree
#[derive(Debug, Clone)]
pub struct PrefixTreeNode {
    pub activity: Option<String>,
    pub children: HashMap<String, PrefixTreeNode>,
    pub trace_ids: Vec<usize>,
    pub count: usize,
}

impl PrefixTreeNode {
    pub fn new() -> Self {
        Self {
            activity: None,
            children: HashMap::new(),
            trace_ids: Vec::new(),
            count: 0,
        }
    }

    pub fn with_activity(activity: String) -> Self {
        Self {
            activity: Some(activity),
            children: HashMap::new(),
            trace_ids: Vec::new(),
            count: 0,
        }
    }
}

impl Default for PrefixTreeNode {
    fn default() -> Self {
        Self::new()
    }
}

/// Prefix tree representation
#[derive(Debug, Clone)]
pub struct PrefixTree {
    pub root: PrefixTreeNode,
    pub num_traces: usize,
    pub num_variants: usize,
}

impl PrefixTree {
    pub fn new() -> Self {
        Self {
            root: PrefixTreeNode::new(),
            num_traces: 0,
            num_variants: 0,
        }
    }

    /// Insert a trace into the prefix tree
    pub fn insert(&mut self, trace: &crate::log::Trace, trace_id: usize) {
        let mut current = &mut self.root;

        for event in &trace.events {
            let activity = event.activity.clone();
            current = current
                .children
                .entry(activity.clone())
                .or_insert_with(|| PrefixTreeNode::with_activity(activity));
        }

        current.trace_ids.push(trace_id);
        current.count += 1;
        self.num_traces += 1;
    }

    /// Get all variants (unique traces) from the tree
    pub fn get_variants(&self) -> Vec<(Vec<String>, usize)> {
        let mut variants = Vec::new();
        self.collect_variants(&self.root, Vec::new(), &mut variants);
        variants
    }

    fn collect_variants(
        &self,
        node: &PrefixTreeNode,
        mut current_path: Vec<String>,
        variants: &mut Vec<(Vec<String>, usize)>,
    ) {
        if node.children.is_empty() {
            // Leaf node - this is a complete variant
            variants.push((current_path.clone(), node.count));
            return;
        }

        for (activity, child) in &node.children {
            current_path.push(activity.clone());
            self.collect_variants(child, current_path.clone(), variants);
            current_path.pop();
        }
    }

    /// Get the number of unique variants
    pub fn count_variants(&self) -> usize {
        self.get_variants().len()
    }
}

impl Default for PrefixTree {
    fn default() -> Self {
        Self::new()
    }
}

/// Discover prefix tree from event log
pub fn discover_prefix_tree(log: &EventLog) -> PrefixTree {
    let mut tree = PrefixTree::new();

    for (idx, trace) in log.traces.iter().enumerate() {
        tree.insert(trace, idx);
    }

    tree.num_variants = tree.count_variants();
    tree
}

/// Get variants as a list of (trace, count) tuples
pub fn get_variants_from_log(log: &EventLog) -> Vec<(Vec<String>, usize)> {
    let mut variants: HashMap<Vec<String>, usize> = HashMap::new();

    for trace in &log.traces {
        let trace_activities: Vec<String> =
            trace.events.iter().map(|e| e.activity.clone()).collect();
        *variants.entry(trace_activities).or_insert(0) += 1;
    }

    let mut result: Vec<(Vec<String>, usize)> = variants.into_iter().collect();
    result.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by count descending
    result
}

/// Get the most common variants (top K)
pub fn get_variants_top_k(log: &EventLog, k: usize) -> Vec<(Vec<String>, usize)> {
    let mut variants = get_variants_from_log(log);
    variants.truncate(k);
    variants
}

/// Filter log to only keep traces matching specific variants
pub fn filter_log_by_variants(log: &EventLog, variants: &[Vec<String>]) -> EventLog {
    let variant_set: HashSet<Vec<String>> = variants.iter().cloned().collect();
    let mut filtered = log.clone();

    filtered.traces.retain(|trace| {
        let activities: Vec<String> = trace.events.iter().map(|e| e.activity.clone()).collect();
        variant_set.contains(&activities)
    });

    filtered
}
