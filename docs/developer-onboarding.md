# Developer Onboarding Guide — pm4py-rust

> **Get from zero to first PR in under 1 hour.**

This guide walks you through everything you need to contribute to pm4py-rust: setup, workflow, testing, and your first contribution.

---

## Table of Contents

1. [Quick Start (5 minutes)](#quick-start-5-minutes)
2. [Prerequisites](#prerequisites)
3. [Project Structure](#project-structure)
4. [Development Workflow](#development-workflow)
5. [Testing Guidelines (Chicago TDD)](#testing-guidelines-chicago-tdd)
6. [Code Style Standards](#code-style-standards)
7. [Adding New Features](#adding-new-features)
8. [Adding New Tests](#adding-new-tests)
9. [Common Pitfalls](#common-pitfalls)
10. [Resources](#resources)

---

## Quick Start (5 minutes)

### Step 1: Clone and Build

```bash
# Clone the repository
git clone https://github.com/seanchatmangpt/pm4py-rust.git
cd pm4py-rust

# Build in debug mode
cargo build

# Run tests to verify everything works
cargo test --lib

# Check formatting and linting
cargo fmt
cargo clippy --all-targets -- -D warnings
```

**Expected output:** All tests pass, no warnings.

### Step 2: Run an Example

```bash
# Create a simple event log and discover a process model
cargo run --example discovery
```

### Step 3: Make Your First Change

Edit `src/lib.rs` and add a comment:

```rust
//! PM4Py - A Rust Process Mining Library
//!
//! Added by <your-name> on <date>
```

Verify:

```bash
cargo build
cargo test
```

Commit:

```bash
git add src/lib.rs
git commit -m "docs(lib): add onboarding comment"
```

**Congratulations!** You've made your first contribution. Now let's dive deeper.

---

## Prerequisites

### Required Tools

| Tool | Version | Purpose | Install |
|------|---------|---------|---------|
| **Rust** | 1.70+ | Language | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| **Git** | Any | Version control | `brew install git` (macOS) or `sudo apt install git` (Linux) |
| **Cargo** | Bundled with Rust | Build tool | Installed with rustup |

### Optional but Recommended

| Tool | Purpose | Install |
|------|---------|---------|
| **cargo-watch** | Auto-rebuild on changes | `cargo install cargo-watch` |
| **cargo-tarpaulin** | Code coverage | `cargo install cargo-tarpaulin` |
| **cargo-audit** | Security audits | `cargo install cargo-audit` |

### Verify Installation

```bash
rustc --version    # Should be 1.70+
cargo --version    # Should match rustc
git --version      # Any 2.x version
```

---

## Project Structure

### Directory Layout

```
pm4py-rust/
├── src/                      # All library source code
│   ├── lib.rs               # Library root (exports public API)
│   ├── version.rs           # Version constants
│   ├── log/                 # Event log structures
│   │   ├── mod.rs
│   │   ├── operations.rs    # Log filtering, merging, sampling
│   │   └── advanced_filters.rs
│   ├── discovery/           # Process discovery algorithms
│   │   ├── mod.rs
│   │   ├── alpha_miner.rs
│   │   ├── inductive_miner.rs
│   │   ├── heuristic_miner.rs
│   │   └── ...
│   ├── conformance/         # Conformance checking
│   │   ├── mod.rs
│   │   ├── token_replay.rs
│   │   └── alignment.rs
│   ├── models/              # Process model representations
│   │   ├── mod.rs
│   │   ├── petri_net.rs
│   │   ├── process_tree.rs
│   │   └── dfg.rs
│   ├── io/                  # File I/O (XES, CSV, Parquet)
│   │   ├── mod.rs
│   │   └── xes.rs
│   ├── statistics/          # Log and trace statistics
│   ├── performance/         # Performance analysis
│   ├── visualization/       # SVG generation
│   ├── semconv/             # OpenTelemetry semantic conventions
│   └── http/                # HTTP API endpoints
├── tests/                   # Integration tests
│   ├── semconv_chicago_tdd_test.rs
│   ├── wvda_soundness_test.rs
│   └── ...
├── benches/                 # Criterion benchmarks
│   ├── discovery.rs
│   └── conformance.rs
├── examples/                # Runnable examples
│   └── disabled/            # Legacy examples (not actively maintained)
├── docs/                    # Documentation
│   ├── DEVELOPER_GUIDE.md   # API usage guide
│   └── diataxis/            # Structured documentation
├── Cargo.toml               # Project manifest
├── Makefile                 # Build automation (run `make help`)
└── CONTRIBUTING.md          # Contribution guidelines
```

### Key Files to Know

| File | Purpose | When to Edit |
|------|---------|--------------|
| `Cargo.toml` | Dependencies, features, version | Adding deps, bumping version |
| `src/lib.rs` | Public API exports | Adding new top-level exports |
| `Makefile` | Build automation targets | Adding new build commands |
| `CONTRIBUTING.md` | Contribution guidelines | Updating process |

---

## Development Workflow

### Daily Workflow

```bash
# 1. Pull latest changes
git pull origin main

# 2. Create feature branch
git checkout -b feat/your-feature-name

# 3. Watch for changes and auto-rebuild (optional)
cargo watch -x build

# 4. Run tests in watch mode (optional)
cargo watch -x test

# 5. Format code
cargo fmt

# 6. Check for warnings
cargo clippy --all-targets -- -D warnings

# 7. Run full test suite
cargo test --all

# 8. Commit changes
git add .
git commit -m "feat(scope): description"

# 9. Push and create PR
git push origin feat/your-feature-name
```

### Make Targets (Faster Development)

The Makefile provides shortcuts for common operations:

```bash
make help              # Show all available targets
make build             # Build debug binary
make release           # Build optimized release binary
make test              # Run all tests
make test-unit         # Run unit tests only
make test-integration  # Run integration tests only
make fmt               # Format code
make clippy            # Run linter
make check             # Run all checks (fmt + clippy + doc)
make doc               # Generate documentation
make doc-open          # Generate and open docs in browser
make coverage          # Generate coverage report
make ci                # Run full CI pipeline locally
```

**Example: Pre-commit workflow**

```bash
make pre-commit        # Runs fmt + clippy + test
```

### Git Workflow

**Branch Naming:**

- `feat/` — New features
- `fix/` — Bug fixes
- `refactor/` — Code refactoring
- `docs/` — Documentation changes
- `test/` — Test additions/changes
- `chore/` — Maintenance tasks

**Commit Format:**

```
type(scope): description

Examples:
feat(discovery): add ILP Miner algorithm
fix(conformance): correct precision calculation
docs(api): improve module documentation
test(io): add edge case tests for CSV parsing
refactor(log): simplify trace filtering logic
```

**Never:**

- `git reset --hard` (forbidden by project rules)
- `git rebase` (use merge commits only)
- Force-push to `main` (allowed on feature branches only)

---

## Testing Guidelines (Chicago TDD)

pm4py-rust follows **Chicago School TDD**: behavior verification via black-box tests with real implementations.

### The Red-Green-Refactor Cycle

#### Phase 1: RED (Write Failing Test First)

**Example:** Adding a new statistics function

```rust
// In src/statistics/log_stats.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_median_duration_calculates_correctly() {
        // Arrange: Create test log with known durations
        let mut log = EventLog::new();
        let trace1 = create_trace_with_duration(Duration::hours(1));
        let trace2 = create_trace_with_duration(Duration::hours(2));
        let trace3 = create_trace_with_duration(Duration::hours(3));
        log.add_trace(trace1);
        log.add_trace(trace2);
        log.add_trace(trace3);

        // Act: Call the function
        let result = median_duration(&log);

        // Assert: Verify correct behavior
        assert_eq!(result, Duration::hours(2));
    }
}
```

Run test — it should **FAIL** because function doesn't exist yet:

```bash
cargo test median_duration
```

#### Phase 2: GREEN (Implement Minimum to Pass)

```rust
pub fn median_duration(log: &EventLog) -> Duration {
    let mut durations: Vec<_> = log.traces()
        .iter()
        .map(|t| t.case_duration())
        .collect();
    durations.sort();
    durations[durations.len() / 2]
}
```

Run test — it should **PASS**:

```bash
cargo test median_duration
```

#### Phase 3: REFACTOR (Improve Without Changing Behavior)

```rust
pub fn median_duration(log: &EventLog) -> Duration {
    let mut durations: Vec<_> = log.traces()
        .iter()
        .map(|t| t.case_duration())
        .collect();

    if durations.is_empty() {
        return Duration::zero();
    }

    durations.sort();
    let mid = durations.len() / 2;

    if durations.len() % 2 == 0 {
        (durations[mid - 1] + durations[mid]) / 2
    } else {
        durations[mid]
    }
}
```

Run test again — still **PASS** (behavior unchanged, code improved).

### Test Characteristics (FIRST)

| Principle | Meaning | Example |
|-----------|---------|---------|
| **F**ast | Tests run in milliseconds | No external API calls, no database queries |
| **I**ndependent | No test depends on another | Each test creates its own fixtures |
| **R**epeatable | Same result every run | Use fake clocks, not real time |
| **S**elf-Checking | Clear PASS/FAIL | Specific assertions, not manual inspection |
| **T**imely | Written with implementation | Test written first (RED), then code |

### Test Organization

#### Unit Tests (In-file)

```rust
// In src/statistics/log_stats.rs

impl LogStatistics {
    pub fn median_duration(&self) -> Duration {
        // implementation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_median_duration_empty_log() {
        let stats = LogStatistics::new(&EventLog::new());
        assert_eq!(stats.median_duration(), Duration::zero());
    }

    #[test]
    fn test_median_duration_single_trace() {
        let log = create_log_with_one_trace();
        let stats = LogStatistics::new(&log);
        assert_eq!(stats.median_duration(), Duration::hours(1));
    }
}
```

#### Integration Tests (Separate file)

```rust
// In tests/statistics_integration_test.rs

use pm4py::{EventLog, LogStatistics};

#[test]
fn test_full_statistics_workflow() {
    // Create log from CSV
    let log = EventLog::from_csv("tests/data/simple_log.csv").unwrap();

    // Calculate statistics
    let stats = LogStatistics::new(&log);

    // Verify results
    assert_eq!(stats.num_traces(), 10);
    assert!(stats.median_duration() > Duration::zero());
}
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_median_duration

# Run tests in a module
cargo test statistics::

# Run with output (for debugging)
cargo test -- --nocapture

# Run with backtrace on failure
RUST_BACKTRACE=1 cargo test

# Run integration tests only
cargo test --test '*'

# Run unit tests only
cargo test --lib
```

---

## Code Style Standards

### Rust Code Style

#### 1. Formatting (Required)

```bash
cargo fmt
```

**Before every commit.** No exceptions.

#### 2. Clippy (Required)

```bash
cargo clippy --all-targets -- -D warnings
```

**Zero warnings allowed.** Fix all warnings before committing.

#### 3. Documentation (Public APIs)

```rust
/// Calculates the median case duration from an event log.
///
/// This function computes the median duration across all traces in the log.
/// Empty logs return a zero duration.
///
/// # Examples
///
/// ```
/// use pm4py::{EventLog, LogStatistics};
///
/// let log = EventLog::from_csv("data.csv")?;
/// let stats = LogStatistics::new(&log);
/// let median = stats.median_duration();
///
/// println!("Median case duration: {:?}", median);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Arguments
///
/// * `log` - The event log to analyze
///
/// # Returns
///
/// The median duration as a `chrono::Duration`. Returns `Duration::zero()` for empty logs.
///
/// # Errors
///
/// This function does not return errors. It handles empty logs gracefully.
pub fn median_duration(log: &EventLog) -> Duration {
    // implementation
}
```

#### 4. Error Handling

**Use Result, not panics:**

```rust
// ✓ Good: Returns Result
pub fn discover(log: &EventLog) -> Result<PetriNet, DiscoveryError> {
    if log.traces.is_empty() {
        return Err(DiscoveryError::EmptyLog);
    }
    // ...
}

// ✗ Bad: Panics
pub fn discover(log: &EventLog) -> PetriNet {
    assert!(!log.traces.is_empty(), "Log cannot be empty");
    // ...
}
```

**Define error types:**

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DiscoveryError {
    #[error("Event log is empty")]
    EmptyLog,

    #[error("Invalid log format: {0}")]
    InvalidFormat(String),

    #[error("Discovery algorithm failed: {0}")]
    AlgorithmFailed(String),
}
```

#### 5. No Unsafe Code Without Documentation

```rust
// ✓ Good: Documented unsafe
/// # Safety
///
/// This function uses unsafe code to transmute a `Vec<u8>` to `Vec<T>`.
/// Caller must ensure that the byte slice represents a valid T.
pub unsafe fn from_raw_bytes<T>(bytes: Vec<u8>) -> Vec<T> {
    std::mem::transmute(bytes)
}

// ✗ Bad: Undocumented unsafe
pub unsafe fn from_raw_bytes_bad<T>(bytes: Vec<u8>) -> Vec<T> {
    std::mem::transmute(bytes)
}
```

#### 6. Performance Best Practices

```rust
// ✓ Good: Uses iterators
let count: usize = log.traces()
    .iter()
    .filter(|t| t.events.len() > 5)
    .count();

// ✗ Bad: Allocates intermediate Vec
let filtered: Vec<_> = log.traces()
    .iter()
    .filter(|t| t.events.len() > 5)
    .collect();
let count = filtered.len();
```

---

## Adding New Features

### Step-by-Step Example: Adding a New Discovery Algorithm

#### Step 1: Create Module File

```bash
touch src/discovery/my_miner.rs
```

#### Step 2: Implement Algorithm

```rust
// src/discovery/my_miner.rs

use crate::log::EventLog;
use crate::models::PetriNet;
use crate::discovery::DiscoveryError;

/// My custom process discovery algorithm.
///
/// This algorithm discovers a Petri net from an event log using
/// a novel approach that balances precision and generalization.
pub struct MyMiner;

impl MyMiner {
    /// Creates a new MyMiner instance.
    pub fn new() -> Self {
        MyMiner
    }

    /// Discovers a Petri net from the event log.
    ///
    /// # Errors
    ///
    /// Returns `DiscoveryError::EmptyLog` if the log contains no traces.
    pub fn discover(&self, log: &EventLog) -> Result<PetriNet, DiscoveryError> {
        if log.traces.is_empty() {
            return Err(DiscoveryError::EmptyLog);
        }

        // Algorithm implementation here
        let mut petri_net = PetriNet::new();

        // ... discovery logic ...

        Ok(petri_net)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::{Trace, Event};
    use chrono::Utc;

    fn create_simple_log() -> EventLog {
        let mut log = EventLog::new();
        let mut trace = Trace::new("case1");
        trace.add_event(Event::new("A", Utc::now()));
        trace.add_event(Event::new("B", Utc::now()));
        log.add_trace(trace);
        log
    }

    #[test]
    fn test_my_miner_simple_log() {
        let log = create_simple_log();
        let miner = MyMiner::new();
        let result = miner.discover(&log);

        assert!(result.is_ok());
        let petri_net = result.unwrap();
        assert!(!petri_net.transitions.is_empty());
    }

    #[test]
    fn test_my_miner_empty_log() {
        let log = EventLog::new();
        let miner = MyMiner::new();
        let result = miner.discover(&log);

        assert!(matches!(result, Err(DiscoveryError::EmptyLog)));
    }
}
```

#### Step 3: Export from Module

```rust
// src/discovery/mod.rs

pub mod my_miner;
pub use my_miner::MyMiner;
```

#### Step 4: Export from Library Root

```rust
// src/lib.rs

pub use discovery::MyMiner;
```

#### Step 5: Run Tests

```bash
cargo test my_miner
```

#### Step 6: Format and Lint

```bash
cargo fmt
cargo clippy --all-targets -- -D warnings
```

#### Step 7: Update Documentation

Add to `README.md` features list:

```markdown
### Process Discovery

| Algorithm | Status | Description |
|-----------|--------|-------------|
| **My Miner** | ✅ | Custom algorithm balancing precision and generalization |
```

Add to `CHANGELOG.md`:

```markdown
## [Unreleased]

### Added
- MyMiner discovery algorithm by @yourname
```

#### Step 8: Create Example (Optional)

```rust
// examples/my_miner_example.rs

use pm4py::{EventLog, MyMiner};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let log = EventLog::from_csv("examples/data/simple_log.csv")?;
    let miner = MyMiner::new();
    let petri_net = miner.discover(&log)?;

    println!("Discovered Petri net:");
    println!("  Places: {}", petri_net.places.len());
    println!("  Transitions: {}", petri_net.transitions.len());

    Ok(())
}
```

Run example:

```bash
cargo run --example my_miner_example
```

#### Step 9: Commit

```bash
git add src/discovery/my_miner.rs src/discovery/mod.rs src/lib.rs README.md CHANGELOG.md
git commit -m "feat(discovery): add MyMiner algorithm"
```

---

## Adding New Tests

### Unit Tests

Add tests in the same file as the code:

```rust
// In src/statistics/log_stats.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_name() {
        // Arrange
        let input = create_test_data();

        // Act
        let result = function_under_test(input);

        // Assert
        assert_eq!(result, expected_value);
    }
}
```

### Integration Tests

Create a new file in `tests/`:

```rust
// tests/my_feature_test.rs

use pm4py::{EventLog, MyMiner};

#[test]
fn test_my_feature_integration() {
    // Load real test data
    let log = EventLog::from_csv("tests/data/test_log.csv").unwrap();

    // Test the feature
    let miner = MyMiner::new();
    let result = miner.discover(&log).unwrap();

    // Verify
    assert!(!result.transitions.is_empty());
}
```

### Property-Based Tests (Proptest)

For complex logic, use property-based testing:

```rust
// In src/statistics/log_stats.rs

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_median_in_range(durations in prop::collection::vec(
            0..1000i64,
            1..100
        )) {
            let log = create_log_from_durations(durations);
            let stats = LogStatistics::new(&log);
            let median = stats.median_duration();

            // Property: median should be within range of values
            assert!(median.num_seconds() >= 0);
            assert!(median.num_seconds() <= 1000);
        }
    }
}
```

### OpenTelemetry Span Tests (Required for Features)

Every feature that emits OTEL spans must have schema conformance tests:

```rust
// In tests/semconv_my_feature_test.rs

use pm4py::semconv::my_feature_attributes::*;

#[test]
fn my_feature_attribute_key_matches_schema() {
    assert_eq!(MY_FEATURE_ATTRIBUTE, "my.feature.attribute");
}

#[test]
fn my_feature_enum_value_matches_schema() {
    assert_eq!(my_feature_status::SUCCESS, "success");
}
```

---

## Common Pitfalls

### Pitfall 1: Not Following TDD

**Wrong:** Write code first, add tests later.

**Right:** Write failing test first (RED), implement until it passes (GREEN), refactor.

**Why:** Tests prove behavior works. Code-first tests often test implementation, not behavior.

### Pitfall 2: Ignoring Clippy Warnings

**Wrong:** Push code with clippy warnings.

**Right:** Fix all warnings before committing.

**Why:** Warnings often indicate real bugs or performance issues.

### Pitfall 3: Not Formatting Code

**Wrong:** Inconsistent formatting.

**Right:** Run `cargo fmt` before every commit.

**Why:** Consistent formatting reduces cognitive load in code reviews.

### Pitfall 4: Testing Implementation, Not Behavior

**Wrong:**

```rust
#[test]
fn test_internal_state() {
    let obj = MyStruct::new();
    assert_eq!(obj.internal_counter, 0); // Tests implementation detail
}
```

**Right:**

```rust
#[test]
fn test_public_api() {
    let obj = MyStruct::new();
    let result = obj.process();
    assert_eq!(result, expected_value); // Tests observable behavior
}
```

### Pitfall 5: Using Panic in Library Code

**Wrong:**

```rust
pub fn process(log: &EventLog) -> PetriNet {
    assert!(!log.traces.is_empty(), "Log cannot be empty");
    // ...
}
```

**Right:**

```rust
pub fn process(log: &EventLog) -> Result<PetriNet, Error> {
    if log.traces.is_empty() {
        return Err(Error::EmptyLog);
    }
    Ok(/* ... */)
}
```

### Pitfall 6: Forgetting to Update Documentation

**Wrong:** Add public API without doc comments.

**Right:** Document all public APIs with examples.

**Why:** Documentation is the first thing users see.

### Pitfall 7: Not Running Full Test Suite

**Wrong:** Only run tests for the module you changed.

**Right:** Run `cargo test --all` before committing.

**Why:** Changes can have unexpected effects on other modules.

### Pitfall 8: Committing Too Much at Once

**Wrong:** One commit with 10 different features.

**Right:** One feature per commit, logical atomic changes.

**Why:** Easier to review, easier to revert if needed.

### Pitfall 9: Using `unwrap()` in Library Code

**Wrong:**

```rust
pub fn load_log(path: &str) -> EventLog {
    let content = std::fs::read_to_string(path).unwrap(); // Crashes on error
    EventLog::from_json(&content).unwrap()
}
```

**Right:**

```rust
pub fn load_log(path: &str) -> Result<EventLog, LoadError> {
    let content = std::fs::read_to_string(path)?;
    EventLog::from_json(&content).map_err(LoadError::InvalidFormat)
}
```

### Pitfall 10: Forgetting OTEL Spans for Features

**Wrong:** Implement feature without emitting OTEL spans.

**Right:** Every feature emits spans with schema-enforced attributes.

**Why:** Verification standard requires OTEL span + test assertion + schema check.

---

## Resources

### Internal Documentation

| Document | Purpose | Location |
|----------|---------|----------|
| **DEVELOPER_GUIDE.md** | API usage and workflows | `docs/DEVELOPER_GUIDE.md` |
| **CONTRIBUTING.md** | Contribution guidelines | `CONTRIBUTING.md` |
| **README.md** | Project overview | `README.md` |
| **ARCHITECTURE.md** | System design | `docs/ARCHITECTURE.md` |

### External Resources

| Resource | Purpose | Link |
|----------|---------|------|
| **Rust Book** | Learn Rust | https://doc.rust-lang.org/book/ |
| **Rust API Guidelines** | API design | https://rust-lang.github.io/api-guidelines/ |
| **pm4py Documentation** | Process mining concepts | https://pm4py.fit.fraunhofer.de/ |
| **Process Mining Book** | Theory | https://www.springer.com/gp/book/9783662493458 |

### Getting Help

1. **Check documentation first:** Most questions answered in `docs/`
2. **Search existing issues:** Someone may have asked before
3. **Open an issue:** Include minimal reproducible example
4. **Email:** info@chatmangpt.com for confidential issues

### Verification Standards

pm4py-rust follows the **Evidence-Based Verification Standard**:

Every claim requires three proofs:
1. **OTEL Span:** Execution proof in Jaeger UI
2. **Test Assertion:** Behavior proof in test suite
3. **Schema Conformance:** Weaver registry check exits 0

See `.claude/rules/verification.md` for details.

---

## Summary: Your First Hour Checklist

- [ ] Clone repository and build: `cargo build`
- [ ] Run tests: `cargo test --lib`
- [ ] Format code: `cargo fmt`
- [ ] Check linting: `cargo clippy --all-targets -- -D warnings`
- [ ] Read `CONTRIBUTING.md` for full guidelines
- [ ] Read `docs/DEVELOPER_GUIDE.md` for API usage
- [ ] Make a small change (add a comment)
- [ ] Commit with conventional format: `docs(lib): add onboarding comment`
- [ ] Create feature branch: `git checkout -b feat/your-feature`
- [ ] Write a failing test (RED phase)
- [ ] Implement until test passes (GREEN phase)
- [ ] Refactor without breaking test (REFACTOR phase)
- [ ] Run full test suite: `cargo test --all`
- [ ] Push and create PR

**You're now ready to contribute!** 🚀

---

**Last Updated:** 2026-03-28
**Maintained By:** Sean Chatman <info@chatmangpt.com>
