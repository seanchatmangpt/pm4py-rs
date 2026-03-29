//! Chicago TDD tests for boardchair Conway's Law and Little's Law analyzers.
//!
//! Tests assert exact numeric behaviour — every claim is independently verifiable.
//! No shared state; each test sets up its own inputs.

use pm4py::boardchair::{analyze_littles_law, check_conway};

// ── Conway's Law tests ────────────────────────────────────────────────────────

#[test]
fn test_conway_check_detects_violation_when_boundary_exceeds_40pct() {
    // boundary = 50ms, cycle = 100ms → score = 0.5 > 0.4 → violation
    let result = check_conway(50, 100);
    assert!(
        result.is_violation,
        "expected Conway violation when boundary_time/cycle_time = 0.5"
    );
    assert!(
        (result.conway_score - 0.5).abs() < 0.001,
        "conway_score should be 0.5, got {}",
        result.conway_score
    );
}

#[test]
fn test_conway_check_no_violation_when_boundary_within_normal() {
    // boundary = 30ms, cycle = 100ms → score = 0.3 < 0.4 → no violation
    let result = check_conway(30, 100);
    assert!(
        !result.is_violation,
        "expected no Conway violation when boundary_time/cycle_time = 0.3"
    );
    assert!(
        (result.conway_score - 0.3).abs() < 0.001,
        "conway_score should be 0.3, got {}",
        result.conway_score
    );
}

#[test]
fn test_conway_check_boundary_exactly_at_threshold_is_not_violation() {
    // score == 0.4 is NOT a violation (violation requires strictly > 0.4)
    let result = check_conway(40, 100);
    assert!(
        !result.is_violation,
        "conway_score == 0.4 should not be a violation"
    );
    assert!(
        (result.conway_score - 0.4).abs() < 0.001,
        "conway_score should be exactly 0.4, got {}",
        result.conway_score
    );
}

#[test]
fn test_conway_check_zero_cycle_time_returns_safe_default() {
    // Division by zero guard: cycle_time_ms == 0 → no violation, score == 0.0
    let result = check_conway(50, 0);
    assert!(
        !result.is_violation,
        "zero cycle_time should produce safe default (no violation)"
    );
    assert_eq!(
        result.conway_score, 0.0,
        "conway_score should be 0.0 for zero cycle_time"
    );
}

// ── Little's Law tests ────────────────────────────────────────────────────────

#[test]
fn test_littles_law_detects_overload_when_wip_exceeds_1_5x() {
    // arrival_rate=10/s, cycle=100ms=0.1s → expected_wip=1.0; actual=2.0 > 1.5 → violation
    let result = analyze_littles_law(10.0, 2.0, 100.0);
    assert!(
        result.is_violation,
        "expected Little's Law violation when actual_wip=2.0 > 1.5×expected=1.5"
    );
    assert!(
        (result.expected_wip - 1.0).abs() < 0.01,
        "expected_wip should be 1.0, got {}",
        result.expected_wip
    );
}

#[test]
fn test_littles_law_stable_when_wip_within_bounds() {
    // arrival_rate=10/s, cycle=100ms → expected_wip=1.0; actual=1.2 < 1.5 → no violation
    let result = analyze_littles_law(10.0, 1.2, 100.0);
    assert!(
        !result.is_violation,
        "expected no violation when actual_wip=1.2 < 1.5×expected=1.5"
    );
    assert!(
        (result.expected_wip - 1.0).abs() < 0.01,
        "expected_wip should be 1.0, got {}",
        result.expected_wip
    );
}

#[test]
fn test_littles_law_exactly_at_threshold_is_not_violation() {
    // actual_wip == 1.5 × expected_wip → not a violation (violation is strictly >)
    let result = analyze_littles_law(10.0, 1.5, 100.0);
    assert!(
        !result.is_violation,
        "wip == 1.5×expected should NOT be a violation (strictly >)"
    );
}

#[test]
fn test_littles_law_zero_arrival_rate_returns_safe_default() {
    // arrival_rate=0 → expected_wip=0 → safe default, no division by zero
    let result = analyze_littles_law(0.0, 5.0, 100.0);
    assert!(
        !result.is_violation,
        "zero arrival_rate should produce safe default (no violation)"
    );
    assert_eq!(result.expected_wip, 0.0);
}

#[test]
fn test_littles_law_stability_ratio_computed_correctly() {
    // arrival_rate=5/s, cycle=200ms=0.2s → expected_wip=1.0; actual=2.0 → ratio=2.0
    let result = analyze_littles_law(5.0, 2.0, 200.0);
    assert!(
        (result.stability_ratio - 2.0).abs() < 0.01,
        "stability_ratio should be 2.0, got {}",
        result.stability_ratio
    );
    assert!(
        result.is_violation,
        "stability_ratio=2.0 > 1.5 should be a violation"
    );
}
