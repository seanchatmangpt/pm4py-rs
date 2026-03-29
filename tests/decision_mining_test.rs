/// Chicago TDD — Decision Mining gap-fill tests.
///
/// T1: mine_decision_rules identifies split points in a branching log
/// T2: returned rules have confidence values in [0.0, 1.0]
use chrono::Utc;
use pm4py::discovery::decision_mining::mine_decision_rules;
use pm4py::log::{Event, EventLog, Trace};

/// Build a log where activity "A" is always followed by either "B" or "C"
/// (an XOR split), giving a clear branching point.
fn make_branching_log() -> EventLog {
    let mut log = EventLog::new();
    let now = Utc::now();

    // Traces that go A → B → D
    for i in 0..3_u32 {
        let mut t = Trace::new(format!("case_ab_{}", i));
        t.add_event(Event::new("A", now));
        t.add_event(Event::new("B", now));
        t.add_event(Event::new("D", now));
        log.add_trace(t);
    }

    // Traces that go A → C → D
    for i in 0..2_u32 {
        let mut t = Trace::new(format!("case_ac_{}", i));
        t.add_event(Event::new("A", now));
        t.add_event(Event::new("C", now));
        t.add_event(Event::new("D", now));
        log.add_trace(t);
    }

    log
}

// ── T1 ────────────────────────────────────────────────────────────────────────

#[test]
fn decision_mining_identifies_split_points() {
    let log = make_branching_log();
    let model = mine_decision_rules(&log);

    // "A" has two successors (B and C) so it must be a split point
    assert!(
        !model.rules.is_empty(),
        "mine_decision_rules must return at least one rule for a branching log"
    );

    let split_activities: Vec<&str> = model
        .rules
        .iter()
        .map(|r| r.split_activity.as_str())
        .collect();

    assert!(
        split_activities.contains(&"A"),
        "Activity 'A' must be identified as a split point; got splits: {:?}",
        split_activities
    );
}

// ── T2 ────────────────────────────────────────────────────────────────────────

#[test]
fn decision_mining_returns_rules_with_confidence_in_unit_interval() {
    let log = make_branching_log();
    let model = mine_decision_rules(&log);

    // Every rule must have confidence in [0.0, 1.0]
    for rule in &model.rules {
        assert!(
            rule.confidence >= 0.0 && rule.confidence <= 1.0,
            "Rule confidence must be in [0.0, 1.0]; got {} for rule '{}'",
            rule.confidence,
            rule.condition
        );
        assert!(
            rule.support > 0,
            "Rule support must be > 0; got {} for rule '{}'",
            rule.support,
            rule.condition
        );
    }

    // Confidence for rules at "A" must sum to 1.0 (they cover all 5 cases)
    let a_confidences: Vec<f64> = model
        .rules
        .iter()
        .filter(|r| r.split_activity == "A")
        .map(|r| r.confidence)
        .collect();

    let total: f64 = a_confidences.iter().sum();
    assert!(
        (total - 1.0).abs() < 1e-9,
        "Confidence values at split 'A' must sum to 1.0; got {:.6}",
        total
    );
}
