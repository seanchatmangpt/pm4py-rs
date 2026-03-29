//! OCEL → plain-text abstraction for RAG system messages.
//!
//! Implements Connection 4 from "No AI Without PI": GenAI/RAG interface grounded in real OCEL context.
//!
//! `abstract_ocel` produces a compact, structured text block that can be injected as
//! a system message so the LLM answers ONLY from real process data — not from hallucination.

use crate::ocpm::object_log::ObjectCentricEventLog;
use std::collections::BTreeMap;

/// Convert an `ObjectCentricEventLog` into a plain-text process context block.
///
/// The output format is:
/// ```text
/// === PROCESS CONTEXT ===
/// Object types: order (42 objects), item (180 objects)
/// Total events: 324
/// Time range: 2026-01-01T00:00:00Z → 2026-03-28T23:59:59Z
///
/// Activity frequencies (by object type):
///   [order] create_order: 42, approve_order: 40, ship_order: 38
///   [item]  pick_item: 180, pack_item: 175, dispatch_item: 170
///
/// Cross-object interactions: 312 events involve multiple object types
/// === END CONTEXT ===
/// ```
pub fn abstract_ocel(log: &ObjectCentricEventLog) -> String {
    let mut buf = String::new();

    buf.push_str("=== PROCESS CONTEXT ===\n");

    // Object types and counts
    let counts = log.count_objects_by_type();
    if counts.is_empty() {
        buf.push_str("Object types: (none)\n");
    } else {
        let types_line: Vec<String> = counts
            .iter()
            .map(|(t, c)| format!("{} ({} objects)", t, c))
            .collect();
        buf.push_str(&format!("Object types: {}\n", types_line.join(", ")));
    }

    buf.push_str(&format!("Total events: {}\n", log.num_events()));

    // Time range
    let timestamps: Vec<_> = log.events.values().map(|(_, ts, _)| *ts).collect();
    if !timestamps.is_empty() {
        let min_ts = timestamps.iter().min().unwrap();
        let max_ts = timestamps.iter().max().unwrap();
        buf.push_str(&format!(
            "Time range: {} → {}\n",
            min_ts.format("%Y-%m-%dT%H:%M:%SZ"),
            max_ts.format("%Y-%m-%dT%H:%M:%SZ")
        ));
    }

    buf.push('\n');

    // Activity frequencies by object type
    let activity_counts = compute_activity_counts_by_type(log);
    if !activity_counts.is_empty() {
        buf.push_str("Activity frequencies (by object type):\n");
        for (type_name, act_counts) in &activity_counts {
            // Sort by frequency descending, take top 10
            let mut sorted: Vec<(&String, &u64)> = act_counts.iter().collect();
            sorted.sort_by(|a, b| b.1.cmp(a.1));
            sorted.truncate(10);

            let line: Vec<String> = sorted
                .iter()
                .map(|(act, cnt)| format!("{}: {}", act, cnt))
                .collect();
            buf.push_str(&format!("  [{}] {}\n", type_name, line.join(", ")));
        }
        buf.push('\n');
    }

    // Cross-object interactions
    let multi_object_events = log
        .event_object_mappings
        .iter()
        .filter(|m| m.object_ids.len() > 1)
        .count();
    buf.push_str(&format!(
        "Cross-object interactions: {} events involve multiple object types\n",
        multi_object_events
    ));

    buf.push_str("=== END CONTEXT ===\n");

    buf
}

/// Compute activity frequency counts grouped by object type.
///
/// Returns `BTreeMap<object_type, BTreeMap<activity_name, frequency>>`.
fn compute_activity_counts_by_type(
    log: &ObjectCentricEventLog,
) -> BTreeMap<String, BTreeMap<String, u64>> {
    let mut result: BTreeMap<String, BTreeMap<String, u64>> = BTreeMap::new();

    for mapping in &log.event_object_mappings {
        if let Some((activity, _, _)) = log.events.get(&mapping.event_id) {
            for obj_id in &mapping.object_ids {
                if let Some(obj) = log.get_object(obj_id) {
                    *result
                        .entry(obj.object_type.name.clone())
                        .or_default()
                        .entry(activity.clone())
                        .or_insert(0) += 1;
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ocpm::object_log::{
        EventToObjectMapping, Object, ObjectCentricEventLog, ObjectType,
    };
    use chrono::Utc;
    use uuid::Uuid;

    fn make_simple_log() -> ObjectCentricEventLog {
        let mut log = ObjectCentricEventLog::new();
        let now = Utc::now();
        let order_type = ObjectType::new("order");
        log.add_object(Object::new("o1", order_type, now));

        let e1 = Uuid::new_v4();
        log.add_event(e1, "create_order", now, None);

        let mut m1 = EventToObjectMapping::new(e1);
        m1.add_object("o1");
        log.add_event_object_mapping(m1);

        log
    }

    #[test]
    fn test_abstract_ocel_produces_nonempty_string() {
        let log = make_simple_log();
        let result = abstract_ocel(&log);
        assert!(
            !result.is_empty(),
            "abstract_ocel must return non-empty string"
        );
    }

    #[test]
    fn test_abstract_ocel_contains_object_type() {
        let log = make_simple_log();
        let result = abstract_ocel(&log);
        assert!(
            result.contains("order"),
            "output must mention the 'order' object type"
        );
    }

    #[test]
    fn test_abstract_ocel_contains_event_count() {
        let log = make_simple_log();
        let result = abstract_ocel(&log);
        assert!(
            result.contains("Total events: 1"),
            "output must show event count"
        );
    }

    #[test]
    fn test_abstract_ocel_contains_activity() {
        let log = make_simple_log();
        let result = abstract_ocel(&log);
        assert!(
            result.contains("create_order"),
            "output must mention the activity name"
        );
    }

    #[test]
    fn test_abstract_ocel_has_context_markers() {
        let log = make_simple_log();
        let result = abstract_ocel(&log);
        assert!(result.contains("=== PROCESS CONTEXT ==="));
        assert!(result.contains("=== END CONTEXT ==="));
    }

    #[test]
    fn test_abstract_ocel_empty_log() {
        let log = ObjectCentricEventLog::new();
        let result = abstract_ocel(&log);
        assert!(
            result.contains("(none)"),
            "empty log should say no object types"
        );
        assert!(result.contains("Total events: 0"));
    }
}
