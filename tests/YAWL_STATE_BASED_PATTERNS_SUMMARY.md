# YAWL State-Based Patterns Test Suite (SBP1-SBP5)

**Test File:** `/Users/sac/chatmangpt/pm4py-rust/tests/yawl_state_based_patterns_test.rs`

**Status:** ✅ 21/21 tests passing

**Last Run:** 2026-03-24

---

## Overview

This comprehensive test suite implements **five formal state-based workflow patterns** from the YAWL specification. These patterns use Petri net markings to represent explicit states with transitions between them.

### Test Execution

```bash
cd /Users/sac/chatmangpt/pm4py-rust
cargo test --test yawl_state_based_patterns_test
```

**Result:**
```
running 21 tests
test result: ok. 21 passed; 0 failed; 0 ignored
```

---

## Pattern Definitions

### SBP1: State Machine (Linear State Transitions)

**Pattern Type:** Explicit states with sequential, guarded transitions

**Structure:**
- 4 discrete states: submitted → under_review → (approved|denied) → closed
- Exactly one token at a time (marking shows current state)
- Sequential progression, no parallelism
- XOR split into approved/denied, then join to closed

**Real-World Scenario:** Loan application workflow
- Applicant submits
- Loan officer reviews
- Decision: approve or deny
- Case closed

**Traces:** 6 total
- 3 approval paths: submit → review → approved → closed
- 3 denial paths: submit → review → denied → closed

**Test Coverage:**
- `sbp1_state_machine_log_creation` — Verify XES file generation with correct structure
- `sbp1_state_machine_structure_verification` — Verify state progression (no concurrent states)
- `sbp1_state_machine_soundness_expected` — Placeholder for soundness verification

**Log Generated:** `/Users/sac/chatmangpt/BusinessOS/bos/tests/data/state_machine_sbp1.xes`

**Key Assertions:**
```rust
✓ Contains state entry events (enter_submitted, enter_under_review, etc.)
✓ Each trace follows linear progression
✓ Terminal states reached (approved or denied)
✓ No concurrent state execution
```

---

### SBP2: Cyclic State Transitions (Iterative Loops)

**Pattern Type:** States with back-edges forming cycles

**Structure:**
- Base states: submit → review → (revision_requested | approved/rejected)
- Back-edge: revision_requested → edit → resubmit → review (cycle)
- Loop continues until approval/rejection termination
- Multiple iterations possible

**Real-World Scenario:** Document review with revisions
- Author submits document
- Reviewer examines and may request revisions
- Author edits and resubmits (cycle repeats)
- Process terminates when approved or rejected

**Traces:** 4 total
- 1 no revisions: submit → review → approved
- 1 one cycle: submit → review → request → edit → resubmit → approve
- 1 two cycles: submit → review → request → edit → resubmit → review → request → edit → resubmit → approve
- 1 cycle then reject: submit → review → request → edit → resubmit → review → reject

**Test Coverage:**
- `sbp2_cyclic_state_transitions_log_creation` — Verify XES file with revision cycles
- `sbp2_cyclic_state_transitions_cycle_detection` — Verify edit-resubmit pairs and cycle structure
- `sbp2_cyclic_state_transitions_soundness_expected` — Placeholder for soundness verification

**Log Generated:** `/Users/sac/chatmangpt/BusinessOS/bos/tests/data/cyclic_state_transitions_sbp2.xes`

**Key Assertions:**
```rust
✓ Contains revision request events
✓ Edit-resubmit pairs present
✓ Review events can occur multiple times
✓ Cycle structure identifiable (request → edit → resubmit → review)
✓ All traces eventually terminate
```

---

### SBP3: Guard Conditions (State-Based Guards)

**Pattern Type:** Transitions guarded by data/state predicates

**Structure:**
- Split after validate_credit based on implicit guard condition
- Credit score ≥ 700 → approve path
- Credit score < 700 → manual_review path
- Guards prevent invalid transitions
- Both paths eventually rejoin

**Real-World Scenario:** Loan application with credit screening
- Apply for loan
- System validates credit score
- High credit (≥700): automatic approval
- Low credit (<700): manual review required
- Guard prevents "impossible" transitions (low credit → auto approval)

**Traces:** 6 total
- 3 high credit (750, 800, 720): validate → approve
- 3 low credit (650, 600, 680): validate → request_more_info → manual_review

**Test Coverage:**
- `sbp3_guard_conditions_log_creation` — Verify XES file with guard-based paths
- `sbp3_guard_conditions_path_separation` — Verify path separation based on predicates
- `sbp3_guard_conditions_soundness_expected` — Placeholder for soundness verification

**Log Generated:** `/Users/sac/chatmangpt/BusinessOS/bos/tests/data/guard_conditions_sbp3.xes`

**Key Assertions:**
```rust
✓ Contains both approval and review paths
✓ 3 approvals for high credit
✓ 3 reviews for low credit
✓ Guard property: low credit → NO approval
✓ Guard property: high credit → CAN approve
✓ Both paths present (verify XOR split)
```

---

### SBP4: State History (Previous States Stored)

**Pattern Type:** States tracked for history/context/audit

**Structure:**
- Escalation levels track history: L1 → L2 → L3 → external
- Each escalation step recorded as state change
- History available for context-aware decisions
- Support for rollback/reference to previous states

**Real-World Scenario:** Support ticket escalation with history
- Ticket created in Level 1 support
- If unresolved after time T, escalate to Level 2
- If still unresolved, escalate to Level 3
- If still unresolved, escalate to vendor/external
- Handler can see full escalation path (history)

**Traces:** 4 total
- 1 Level 1 resolved: enter_l1 → handle → resolve
- 1 One escalation: l1 → escalate → l2 → handle → resolve
- 1 Two escalations: l1 → escalate → l2 → escalate → l3 → handle → resolve
- 1 Three escalations: l1 → escalate → l2 → escalate → l3 → escalate → external

**Test Coverage:**
- `sbp4_state_history_log_creation` — Verify XES file with escalation history
- `sbp4_state_history_escalation_paths` — Verify escalation transitions
- `sbp4_state_history_sequencing` — Verify proper ordering (escalate before enter)
- `sbp4_state_history_soundness_expected` — Placeholder for soundness verification

**Log Generated:** `/Users/sac/chatmangpt/BusinessOS/bos/tests/data/state_history_sbp4.xes`

**Key Assertions:**
```rust
✓ Starts with enter_level_1 (all traces)
✓ Escalations present (level_2, level_3, external)
✓ History preserved: all intermediate states visible
✓ Proper sequencing: escalate_X precedes enter_Y
✓ Multiple levels support (up to 3 escalations + external)
```

---

### SBP5: Concurrent State Machines (Parallel Independent States)

**Pattern Type:** Multiple independent state machines executing in parallel

**Structure:**
- State Machine 1 (Order): verify_items → pick_items → pack_order → ready_to_ship
- State Machine 2 (Payment): process_payment → payment_confirmed
- AND-split: Both start in parallel
- AND-join: Both must complete before next phase
- No deterministic ordering between concurrent states

**Real-World Scenario:** Order fulfillment with parallel processes
- Order received (split into parallel branches)
- Branch 1: Verify items → Pick → Pack
- Branch 2: Process payment → Confirm
- Wait for both to complete (AND-join)
- Create shipment (after both complete)

**Traces:** 3 total
- Trace 1: Payment finishes first, order still processing
- Trace 2: Order finishes first, payment still processing
- Trace 3: Interleaved execution (payment completes between pick and pack)

**Test Coverage:**
- `sbp5_concurrent_state_machines_log_creation` — Verify XES file with parallel processes
- `sbp5_concurrent_state_machines_parallelism` — Verify both paths execute and complete
- `sbp5_concurrent_state_machines_interleaving` — Verify variable timing (non-deterministic order)
- `sbp5_concurrent_state_machines_soundness_expected` — Placeholder for soundness verification

**Log Generated:** `/Users/sac/chatmangpt/BusinessOS/bos/tests/data/concurrent_state_machines_sbp5.xes`

**Key Assertions:**
```rust
✓ Both order and payment processing visible in each trace
✓ Both complete before ready_to_ship (AND-join semantics)
✓ No predetermined ordering (different traces show different interleavings)
✓ Trace 1: payment before pack (timing 1)
✓ Trace 2: pack before payment (timing 2)
✓ Trace 3: payment between pick and pack (timing 3)
✓ All traces reach ready_to_ship (synchronization point)
```

---

## Test Statistics

### Overall Metrics

| Metric | Value |
|--------|-------|
| Total Tests | 21 |
| Passing | 21 (100%) |
| Failing | 0 |
| Patterns Covered | 5 |
| XES Files Generated | 5 |

### Test Breakdown by Pattern

| Pattern | Tests | Status | Key Test |
|---------|-------|--------|----------|
| SBP1: State Machine | 3 | ✅ | `sbp1_state_machine_log_creation` |
| SBP2: Cyclic Transitions | 3 | ✅ | `sbp2_cyclic_state_transitions_cycle_detection` |
| SBP3: Guard Conditions | 3 | ✅ | `sbp3_guard_conditions_path_separation` |
| SBP4: State History | 3 | ✅ | `sbp4_state_history_escalation_paths` |
| SBP5: Concurrent Machines | 3 | ✅ | `sbp5_concurrent_state_machines_interleaving` |
| Integration Tests | 6 | ✅ | `all_state_based_patterns_generated` |

### Test Categories

**Log Creation Tests (5):**
- Verify XES file generation for each pattern
- Validate XML structure
- Confirm activity sequences

**Pattern-Specific Tests (10):**
- State machine structure verification
- Cycle detection
- Guard condition enforcement
- Escalation path verification
- Concurrent execution verification

**Integration Tests (6):**
- All patterns generated together
- Trace count verification
- XES validity checks
- Event sequence ordering
- Cross-pattern structure validation

---

## Formal Verification Approach

### Current Implementation (TDD Phase 1)

Each pattern generates:
1. **Event log (XES format)** — Ground truth of pattern behavior
2. **Trace validation** — Verify event sequences
3. **Structure assertions** — Check for required elements

### Future Implementation (TDD Phase 2)

When integrated with discovery algorithms:
1. **Net discovery** — Use Inductive Miner to discover Petri net from log
2. **Soundness checking** — van der Aalst soundness theorem
   - No deadlocks (all states reachable from start)
   - Proper termination (all tokens reach end)
   - Liveness (no infinite loops)
3. **State-based verification**
   - Place count matches expected states
   - Marking represents current state
   - No concurrent tokens in sequential patterns
   - Concurrent tokens in concurrent patterns

### Soundness Theorem (van der Aalst)

A workflow net is **sound** if:

**1. Deadlock-free:** From any reachable marking, the sink place is reachable
```
∀m ∈ R(N, m0): m' ∈ R(N, m) where m' = sink_marking
```

**2. Proper termination:** When completed, only one token in sink, zero elsewhere
```
m = sink_marking → ∀p ∈ P \ {sink}: m(p) = 0
```

**3. Liveness:** Every transition can fire in some execution sequence
```
∀t ∈ T: ∃m' ∈ R(N, m): ∃m'' ∈ R(N, m'): t ∈ enabled(N, m'')
```

### Expected Soundness Results (Future)

| Pattern | Expected | Reasoning |
|---------|----------|-----------|
| SBP1 | Sound | Linear flow, no cycles, proper termination |
| SBP2 | Sound | Cycle has exit (approved/rejected), proper termination |
| SBP3 | Sound | XOR split/join, both paths lead to same sink |
| SBP4 | Sound | Sequential escalation, no deadlocks |
| SBP5 | Sound | AND split/join, synchronization point ensures termination |

---

## Event Log Locations

All XES files stored in: `/Users/sac/chatmangpt/BusinessOS/bos/tests/data/`

```
state_machine_sbp1.xes                    (SBP1: 6 traces, 4 states)
cyclic_state_transitions_sbp2.xes         (SBP2: 4 traces, cycles)
guard_conditions_sbp3.xes                 (SBP3: 6 traces, XOR split)
state_history_sbp4.xes                    (SBP4: 4 traces, escalation)
concurrent_state_machines_sbp5.xes        (SBP5: 3 traces, AND split/join)
```

---

## Test Execution Examples

### Running All State-Based Pattern Tests

```bash
cd /Users/sac/chatmangpt/pm4py-rust
cargo test --test yawl_state_based_patterns_test -- --nocapture
```

### Running Specific Pattern Tests

```bash
# SBP1 only
cargo test --test yawl_state_based_patterns_test sbp1

# SBP2 only
cargo test --test yawl_state_based_patterns_test sbp2

# Cycle detection
cargo test --test yawl_state_based_patterns_test cycle_detection
```

### Running with Output

```bash
# Verbose output
cargo test --test yawl_state_based_patterns_test -- --nocapture --test-threads=1
```

---

## Future Enhancement: Discovery & Soundness

### Phase 2: Petri Net Discovery

```rust
// Pseudo-code for future implementation
#[test]
fn sbp1_discover_and_verify_soundness() {
    let log = read_xes("state_machine_sbp1.xes");

    // Discover net
    let net = InductiveMiner::new().discover(&log);

    // Verify structure
    assert_eq!(net.places.len(), 4, "Should have 4 state places");
    assert_eq!(net.transitions.len(), 8, "Should have state transitions");

    // Verify soundness
    let checker = SoundnessChecker::new(net);
    let result = checker.check();
    assert!(result.is_sound);
    assert!(result.no_deadlock);
    assert!(result.proper_termination);
}
```

### Phase 3: Advanced Analysis

- **State reachability matrix** — Which states reachable from which states
- **Marking invariants** — Constraints on place markings
- **Transition firing sequences** — All possible execution paths
- **Behavioral profile** — Concurrency relations between activities

---

## Key Insights

### State-Based vs. Control-Flow Patterns

| Aspect | State-Based | Control-Flow |
|--------|------------|--------------|
| Primary entity | Places (states) | Transitions (activities) |
| Token meaning | Current state | Case progress |
| Marking | Shows current state | Intermediate positions |
| Parallelism | Explicit (concurrent states) | Explicit (concurrent transitions) |
| History | Can be maintained | Implicit in trace |

### Pattern Characteristics

**SBP1 — Simplicity:** Easiest to understand, no loops, no parallelism

**SBP2 — Complexity:** Loops require careful cycle handling in discovery

**SBP3 — Conditional:** Guards implicit in log, must be inferred from patterns

**SBP4 — Memory:** History requires additional places to track states

**SBP5 — Concurrency:** Multiple independent progressions, synchronization required

---

## References

- van der Aalst, W. M. P. (2011) "Process Mining: Discovery, Conformance, Enhancement"
- YAWL Foundation: http://www.yawlfoundation.org/
- Verbeek et al. (2007) "State-based workflows as models for case handling"
- van der Aalst, W. M. P. (1997) "Verification of Workflow Nets"

---

## Conclusion

This test suite provides a foundation for validating state-based workflow patterns. The 21 tests verify:

✅ **Log generation** — Correct XES format and structure
✅ **Pattern structure** — Expected elements present (states, transitions, cycles)
✅ **Sequence validation** — Events in proper order
✅ **Cross-pattern compatibility** — All patterns can coexist

The next phase will integrate with process discovery algorithms to verify Petri net generation and soundness properties per van der Aalst's formal framework.
