# Critical Bugs Found in pm4py-rust

## Compilation Errors (Blocking)

### BUG #1: Missing `events_iter()` method
**Files:** `src/statistics/additional.rs` (lines 72, 103, 152, 518, 940)
**Root Cause:** Code references `log.events_iter()` which doesn't exist in EventLog struct
**Fix:** Replace with direct iteration over traces and their events
**Severity:** CRITICAL - Prevents compilation

### BUG #2: Missing `weekday()` import
**File:** `src/statistics/additional.rs:551`
**Root Cause:** Uses `event.timestamp.weekday()` but `Datelike` trait not imported
**Fix:** Add `use chrono::Datelike;` to imports
**Severity:** CRITICAL - Prevents compilation

### BUG #3: Type inference failures in string operations
**File:** `src/statistics/additional.rs` (lines 54-58, 74-78, 104-107, 154-157)
**Root Cause:** `event.attributes` is `BTreeMap`, not `Option<BTreeMap>`. Code tries Option pattern matching.
**Fix:** Remove `if let Some()` and iterate directly over BTreeMap
**Severity:** CRITICAL - Prevents compilation

### BUG #4: Type mismatches in conformance/advanced.rs
**File:** `src/conformance/advanced.rs` (lines 408, 482, 704)
**Root Cause:** String reference vs owned string comparison: `&String == String` fails
**Fix:** Use `.as_str()` for comparisons or deref properly
**Severity:** CRITICAL - Prevents compilation

### BUG #5: DFG edge set contains check
**File:** `src/models/dfg.rs:178`
**Root Cause:** `edge_set.contains()` expects `&(&str, &str)` but receives `(&str, &str)`
**Fix:** Pass reference correctly or change contains() call
**Severity:** CRITICAL - Prevents compilation

### BUG #6: Type annotation needed in discovery/variants.rs
**File:** `src/discovery/variants.rs:221`
**Root Cause:** Type inference failure on `.collect()`
**Fix:** Specify target type explicitly
**Severity:** CRITICAL - Prevents compilation

## Runtime Bugs (Found by Edge Cases)

### BUG #7: Empty log panic risk in token replay
**File:** `src/conformance/token_replay.rs:38`
**Root Cause:** `net.fire_transition()` may panic on empty net
**Impact:** Crash when discovering/checking empty logs
**Fix:** Add guard check for empty nets
**Severity:** HIGH - Panic on edge case

### BUG #8: Uninitialized marking state
**File:** `src/conformance/soundness_checker.rs:152`
**Root Cause:** Reachability computation doesn't initialize properly
**Impact:** Infinite loops or missed markings
**Fix:** Ensure proper BFS initialization
**Severity:** HIGH - Logic error

### BUG #9: Division by zero in statistics
**File:** `src/statistics/additional.rs` multiple locations
**Root Cause:** No guard checks before `/ count as f64`
**Impact:** NaN values in results
**Fix:** Add `if count > 0` checks
**Severity:** MEDIUM - Silent data corruption

## Test Results

- **Compilation:** 17 errors, 40+ warnings
- **Tests:** Cannot run due to compilation failures
- **Critical Path:** All discovery/conformance modules blocked

## Estimated Fix Effort

- Fixes for compilation errors: 2-3 hours
- Fixes for runtime bugs: 3-4 hours
- Complete test coverage: 4-5 hours
- **Total: 9-12 hours**

## Priority Order

1. Fix string comparison bugs (advanced.rs)
2. Fix missing `events_iter()` pattern (additional.rs)
3. Fix `weekday()` import (additional.rs)
4. Fix DFG edge set operations (dfg.rs)
5. Add empty log guards (token_replay.rs, soundness_checker.rs)
6. Add division-by-zero guards (statistics)
