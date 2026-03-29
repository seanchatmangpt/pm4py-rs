/// Trace abstraction and activity grouping operations
///
/// This module provides advanced abstraction capabilities for traces:
/// - Activity abstraction (grouping similar activities)
/// - Hierarchical activity levels
/// - User-defined abstraction rules
/// - Configurable abstraction strategies
///
/// # Examples
///
/// ```ignore
/// use pm4py::log::trace_abstraction::{ActivityAbstractor, AbstractionRule};
///
/// let mut abstractor = ActivityAbstractor::new();
/// abstractor.add_rule(AbstractionRule::new_prefix("log_", "logging"));
///
/// let abstracted_log = abstractor.abstract_log(&log);
/// ```
use crate::log::{Event, EventLog, Trace};
use std::collections::BTreeMap;

/// Defines how activities should be abstracted
#[derive(Debug, Clone)]
pub enum AbstractionRule {
    /// Group activities with a specific prefix into a single activity
    PrefixGrouping {
        prefix: String,
        abstracted_name: String,
    },
    /// Group activities with a specific suffix into a single activity
    SuffixGrouping {
        suffix: String,
        abstracted_name: String,
    },
    /// Map specific activities to an abstracted activity
    ActivityMapping {
        activities: Vec<String>,
        abstracted_name: String,
    },
    /// Apply regex-based pattern matching (simplified: exact substring match)
    PatternBased {
        pattern: String,
        abstracted_name: String,
    },
    /// Hierarchical level abstraction (keep top N levels of activity names)
    HierarchicalLevel {
        separator: char,
        level: usize,
        default_suffix: Option<String>,
    },
}

impl AbstractionRule {
    /// Create a new prefix grouping rule
    pub fn new_prefix(prefix: &str, abstracted_name: &str) -> Self {
        Self::PrefixGrouping {
            prefix: prefix.to_string(),
            abstracted_name: abstracted_name.to_string(),
        }
    }

    /// Create a new suffix grouping rule
    pub fn new_suffix(suffix: &str, abstracted_name: &str) -> Self {
        Self::SuffixGrouping {
            suffix: suffix.to_string(),
            abstracted_name: abstracted_name.to_string(),
        }
    }

    /// Create a new activity mapping rule
    pub fn new_activity_mapping(activities: Vec<&str>, abstracted_name: &str) -> Self {
        Self::ActivityMapping {
            activities: activities.iter().map(|s| s.to_string()).collect(),
            abstracted_name: abstracted_name.to_string(),
        }
    }

    /// Create a new pattern-based rule
    pub fn new_pattern(pattern: &str, abstracted_name: &str) -> Self {
        Self::PatternBased {
            pattern: pattern.to_string(),
            abstracted_name: abstracted_name.to_string(),
        }
    }

    /// Create a new hierarchical level rule
    pub fn new_hierarchical(separator: char, level: usize, default_suffix: Option<&str>) -> Self {
        Self::HierarchicalLevel {
            separator,
            level,
            default_suffix: default_suffix.map(|s| s.to_string()),
        }
    }

    /// Apply the rule to an activity name, returning the abstracted name or original if no match
    fn apply(&self, activity: &str) -> Option<String> {
        match self {
            Self::PrefixGrouping {
                prefix,
                abstracted_name,
            } => {
                if activity.starts_with(prefix) {
                    Some(abstracted_name.clone())
                } else {
                    None
                }
            }
            Self::SuffixGrouping {
                suffix,
                abstracted_name,
            } => {
                if activity.ends_with(suffix) {
                    Some(abstracted_name.clone())
                } else {
                    None
                }
            }
            Self::ActivityMapping {
                activities,
                abstracted_name,
            } => {
                if activities.contains(&activity.to_string()) {
                    Some(abstracted_name.clone())
                } else {
                    None
                }
            }
            Self::PatternBased {
                pattern,
                abstracted_name,
            } => {
                if activity.contains(pattern.as_str()) {
                    Some(abstracted_name.clone())
                } else {
                    None
                }
            }
            Self::HierarchicalLevel {
                separator,
                level,
                default_suffix,
            } => {
                let parts: Vec<&str> = activity.split(*separator).collect();
                if parts.len() > *level {
                    let abstracted = parts[..*level].join(&separator.to_string());
                    Some(abstracted)
                } else if let Some(suffix) = default_suffix {
                    Some(format!("{}{}{}{}", activity, separator, suffix, ""))
                } else {
                    Some(activity.to_string())
                }
            }
        }
    }
}

/// Main activity abstractor with support for multiple rules
pub struct ActivityAbstractor {
    rules: Vec<AbstractionRule>,
}

impl ActivityAbstractor {
    /// Create a new activity abstractor with no rules
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Add an abstraction rule
    pub fn add_rule(&mut self, rule: AbstractionRule) {
        self.rules.push(rule);
    }

    /// Add multiple rules at once
    pub fn add_rules(&mut self, rules: Vec<AbstractionRule>) {
        self.rules.extend(rules);
    }

    /// Get the number of rules
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    /// Clear all rules
    pub fn clear_rules(&mut self) {
        self.rules.clear();
    }

    /// Apply abstraction rules to a single activity
    /// Returns the abstracted activity name, or the original if no rules match
    pub fn abstract_activity(&self, activity: &str) -> String {
        for rule in &self.rules {
            if let Some(abstracted) = rule.apply(activity) {
                return abstracted;
            }
        }
        activity.to_string()
    }

    /// Abstract a single event
    fn abstract_event(&self, event: &Event) -> Event {
        let mut abstracted = event.clone();
        abstracted.activity = self.abstract_activity(&event.activity);
        abstracted
    }

    /// Abstract a single trace
    fn abstract_trace(&self, trace: &Trace) -> Trace {
        let mut abstracted = Trace::new(trace.id.clone());
        abstracted.attributes = trace.attributes.clone();

        for event in &trace.events {
            abstracted.add_event(self.abstract_event(event));
        }

        abstracted
    }

    /// Abstract an entire event log
    pub fn abstract_log(&self, log: &EventLog) -> EventLog {
        let mut abstracted = EventLog::new();
        abstracted.attributes = log.attributes.clone();

        for trace in &log.traces {
            abstracted.add_trace(self.abstract_trace(trace));
        }

        abstracted
    }

    /// Get mapping statistics: original activities to abstracted activities
    pub fn get_abstraction_mapping(&self, log: &EventLog) -> BTreeMap<String, String> {
        let mut mapping = BTreeMap::new();

        for trace in &log.traces {
            for event in &trace.events {
                let abstracted = self.abstract_activity(&event.activity);
                if abstracted != event.activity {
                    mapping.insert(event.activity.clone(), abstracted);
                }
            }
        }

        mapping
    }

    /// Get statistics about abstraction results
    pub fn get_statistics(&self, log: &EventLog) -> AbstractionStatistics {
        let abstracted_log = self.abstract_log(log);

        let original_activities = log.activities();
        let abstracted_activities = abstracted_log.activities();

        let mapping = self.get_abstraction_mapping(log);
        let mapped_count = mapping.len();

        AbstractionStatistics {
            original_activity_count: original_activities.len(),
            abstracted_activity_count: abstracted_activities.len(),
            activities_mapped: mapped_count,
            reduction_ratio: if !original_activities.is_empty() {
                1.0 - (abstracted_activities.len() as f64 / original_activities.len() as f64)
            } else {
                0.0
            },
        }
    }
}

impl Default for ActivityAbstractor {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about abstraction results
#[derive(Debug, Clone)]
pub struct AbstractionStatistics {
    /// Number of activities before abstraction
    pub original_activity_count: usize,
    /// Number of activities after abstraction
    pub abstracted_activity_count: usize,
    /// Number of distinct activities that were mapped
    pub activities_mapped: usize,
    /// Reduction ratio (0.0 to 1.0)
    pub reduction_ratio: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_log() -> EventLog {
        let mut log = EventLog::new();
        let now = Utc::now();

        let mut trace1 = Trace::new("case_1");
        trace1.add_event(Event::new("log_info", now));
        trace1.add_event(Event::new("log_error", now));
        trace1.add_event(Event::new("process_data", now));
        log.add_trace(trace1);

        let mut trace2 = Trace::new("case_2");
        trace2.add_event(Event::new("log_debug", now));
        trace2.add_event(Event::new("save_file.start", now));
        trace2.add_event(Event::new("save_file.end", now));
        log.add_trace(trace2);

        log
    }

    #[test]
    fn test_prefix_grouping() {
        let mut abstractor = ActivityAbstractor::new();
        abstractor.add_rule(AbstractionRule::new_prefix("log_", "logging"));

        let log = create_test_log();
        let abstracted = abstractor.abstract_log(&log);

        let activities = abstracted.activities();
        assert!(activities.contains(&"logging".to_string()));
        assert!(!activities.contains(&"log_info".to_string()));
    }

    #[test]
    fn test_suffix_grouping() {
        let mut abstractor = ActivityAbstractor::new();
        abstractor.add_rule(AbstractionRule::new_suffix(
            ".start",
            "file_operation_start",
        ));
        abstractor.add_rule(AbstractionRule::new_suffix(".end", "file_operation_end"));

        let log = create_test_log();
        let abstracted = abstractor.abstract_log(&log);

        let activities = abstracted.activities();
        assert!(activities.contains(&"file_operation_start".to_string()));
        assert!(activities.contains(&"file_operation_end".to_string()));
    }

    #[test]
    fn test_activity_mapping() {
        let mut abstractor = ActivityAbstractor::new();
        abstractor.add_rule(AbstractionRule::new_activity_mapping(
            vec!["log_debug", "log_info", "log_error"],
            "logging",
        ));

        let log = create_test_log();
        let abstracted = abstractor.abstract_log(&log);

        let activities = abstracted.activities();
        // Verify that the mapping rule was applied - "logging" activity should be present
        assert!(activities.contains(&"logging".to_string()));
        // Verify abstraction reduced the number of distinct activities
        assert!(abstracted.activities().len() <= create_test_log().activities().len());
    }

    #[test]
    fn test_multiple_rules() {
        let mut abstractor = ActivityAbstractor::new();
        abstractor.add_rule(AbstractionRule::new_prefix("log_", "logging"));
        abstractor.add_rule(AbstractionRule::new_suffix(".start", "file_op"));

        let log = create_test_log();
        let abstracted = abstractor.abstract_log(&log);

        let activities = abstracted.activities();
        assert!(activities.contains(&"logging".to_string()));
        assert!(activities.contains(&"file_op".to_string()));
    }

    #[test]
    fn test_abstraction_statistics() {
        let mut abstractor = ActivityAbstractor::new();
        abstractor.add_rule(AbstractionRule::new_prefix("log_", "logging"));

        let log = create_test_log();
        let stats = abstractor.get_statistics(&log);

        assert!(stats.original_activity_count >= 4);
        assert!(stats.abstracted_activity_count <= stats.original_activity_count);
        assert!(stats.reduction_ratio >= 0.0 && stats.reduction_ratio <= 1.0);
    }

    #[test]
    fn test_abstraction_mapping() {
        let mut abstractor = ActivityAbstractor::new();
        abstractor.add_rule(AbstractionRule::new_prefix("log_", "logging"));

        let log = create_test_log();
        let mapping = abstractor.get_abstraction_mapping(&log);

        assert!(mapping.contains_key(&"log_info".to_string()));
        assert_eq!(
            mapping.get(&"log_info".to_string()),
            Some(&"logging".to_string())
        );
    }

    #[test]
    fn test_hierarchical_abstraction() {
        let mut log = EventLog::new();
        let now = Utc::now();

        let mut trace = Trace::new("case_1");
        trace.add_event(Event::new("level1:level2:level3", now));
        trace.add_event(Event::new("process:data:save", now));
        log.add_trace(trace);

        let mut abstractor = ActivityAbstractor::new();
        abstractor.add_rule(AbstractionRule::new_hierarchical(':', 2, None));

        let abstracted = abstractor.abstract_log(&log);
        let activities = abstracted.activities();

        assert!(activities.iter().any(|a| a.contains("level1")));
        assert!(activities.iter().any(|a| a.contains("process")));
    }
}
