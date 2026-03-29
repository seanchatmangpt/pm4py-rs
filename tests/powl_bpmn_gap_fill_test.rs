/// Chicago TDD — POWL model + BPMN conversion gap-fill tests.
///
/// T1: discover_bpmn_inductive preserves Sequence operator (3 tasks)
/// T2: discover_bpmn_inductive preserves Choice gateway (XOR-split + XOR-join)
/// T3: discover_bpmn_inductive produces valid BPMN (start event + tasks)
/// T4: discover_powl returns POWLModel (not ProcessTree) with all log activities
use chrono::Utc;
use pm4py::discovery::extended_discovery::discover_powl;
use pm4py::log::{Event, EventLog, Trace};
use pm4py::models::{ProcessTree, ProcessTreeNode};
use pm4py::statistics::discover_bpmn_inductive;

fn make_abc_log() -> EventLog {
    let mut log = EventLog::new();
    let now = Utc::now();
    for i in 0..3_u32 {
        let mut t = Trace::new(format!("case_{}", i));
        t.add_event(Event::new("A", now));
        t.add_event(Event::new("B", now));
        t.add_event(Event::new("C", now));
        log.add_trace(t);
    }
    log
}

// ── T1 ────────────────────────────────────────────────────────────────────────

#[test]
fn bpmn_conversion_preserves_sequence_operator() {
    let tree = ProcessTree::new(ProcessTreeNode::sequence(vec![
        ProcessTreeNode::activity("A"),
        ProcessTreeNode::activity("B"),
        ProcessTreeNode::activity("C"),
    ]));

    let bpmn = discover_bpmn_inductive(&tree);

    // Sequence of 3 activities → 3 task nodes (no gateways needed)
    assert_eq!(
        bpmn.tasks.len(),
        3,
        "Sequence(A,B,C) must produce exactly 3 task nodes, got {}",
        bpmn.tasks.len()
    );
    assert_eq!(
        bpmn.gateways.len(),
        0,
        "Sequence operator must not produce any gateway nodes"
    );
}

// ── T2 ────────────────────────────────────────────────────────────────────────

#[test]
fn bpmn_conversion_preserves_choice_gateway() {
    let tree = ProcessTree::new(ProcessTreeNode::choice(vec![
        ProcessTreeNode::activity("X"),
        ProcessTreeNode::activity("Y"),
    ]));

    let bpmn = discover_bpmn_inductive(&tree);

    assert_eq!(bpmn.tasks.len(), 2, "Choice(X,Y) must produce 2 task nodes");
    assert_eq!(
        bpmn.gateways.len(),
        2,
        "Choice must produce XOR-split + XOR-join gateways (2 total)"
    );
}

// ── T3 ────────────────────────────────────────────────────────────────────────

#[test]
fn bpmn_conversion_roundtrip_produces_valid_structure() {
    let tree = ProcessTree::new(ProcessTreeNode::sequence(vec![
        ProcessTreeNode::activity("Submit"),
        ProcessTreeNode::activity("Approve"),
    ]));

    let bpmn = discover_bpmn_inductive(&tree);

    // Must have a start event
    assert!(
        bpmn.start_event_id.is_some(),
        "BPMN diagram must have a start event"
    );
    // Must have at least one end event
    assert!(
        !bpmn.end_event_ids.is_empty(),
        "BPMN diagram must have at least one end event"
    );
    // Tasks must be present
    assert!(
        !bpmn.tasks.is_empty(),
        "BPMN diagram must contain task nodes"
    );
    // Flows must connect everything
    assert!(
        !bpmn.flows.is_empty(),
        "BPMN diagram must contain sequence flows"
    );
}

// ── T4 ────────────────────────────────────────────────────────────────────────

#[test]
fn discover_powl_returns_powl_model_with_all_activities() {
    let log = make_abc_log();
    let model = discover_powl(&log);

    // Must expose activities field (not leaf_count like ProcessTree)
    assert!(
        model.activities.len() >= 3,
        "POWL model must contain all 3 activities from the log, got {}",
        model.activities.len()
    );
    assert!(
        model.activities.contains(&"A".to_string()),
        "POWL model must include activity 'A'"
    );
    assert!(
        model.activities.contains(&"B".to_string()),
        "POWL model must include activity 'B'"
    );
    assert!(
        model.activities.contains(&"C".to_string()),
        "POWL model must include activity 'C'"
    );
    // Partial order must have edges (A→B and B→C at minimum)
    assert!(
        !model.partial_order.is_empty(),
        "POWL model must have at least one partial order edge"
    );
}
