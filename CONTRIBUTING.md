# Contributing to PM4Py Rust

Thank you for your interest in contributing to PM4Py Rust! This document provides guidelines and instructions for contributing.

---

## Getting Started

### Prerequisites

- Rust 1.70 or later
- Git
- Basic familiarity with process mining concepts (helpful but not required)

### Setup Development Environment

```bash
git clone https://github.com/seanchatmangpt/pm4py-rust
cd pm4py-rust
cargo build
cargo test
```

---

## Development Workflow

### 1. Create a Feature Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/your-bug-fix
git checkout -b refactor/your-refactor
```

### 2. Make Your Changes

Follow the code standards:

- **Formatting**: `cargo fmt`
- **Linting**: `cargo clippy --all --all-targets -- -D warnings`
- **Testing**: Add tests for new functionality
- **Documentation**: Add doc comments for public APIs

### 3. Write Tests

All new features must include tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_your_feature() {
        // Arrange
        let input = create_test_data();

        // Act
        let result = your_function(input);

        // Assert
        assert_eq!(result, expected_value);
    }
}
```

### 4. Update Documentation

- Add doc comments to public items
- Include usage examples in doc comments
- Update README.md if adding major features
- Update CHANGELOG.md in Unreleased section

### 5. Run Quality Checks

```bash
# Format code
cargo fmt

# Run linter
cargo clippy --all --all-targets -- -D warnings

# Run tests
cargo test --all

# Check documentation
cargo doc --no-deps

# Run benchmarks (if applicable)
cargo bench --bench discovery
```

### 6. Commit Your Changes

Use conventional commit format:

```
type(scope): description

Body (optional):
- More detailed explanation
- Multiple bullet points okay
```

**Types**: `feat`, `fix`, `refactor`, `perf`, `docs`, `test`, `chore`

**Examples**:
```
feat(discovery): add ILP Miner algorithm
fix(conformance): correct precision calculation for loops
docs(api): improve module documentation
test(io): add edge case tests for CSV parsing
```

### 7. Push and Create Pull Request

```bash
git push origin feature/your-feature-name
```

Then create a PR on GitHub with:
- Clear title (use conventional format)
- Description of changes
- Link to any related issues
- Screenshots if UI-related

---

## Code Standards

### Rust Code Style

- Use `cargo fmt` for formatting
- Follow Rust API guidelines (https://rust-lang.github.io/api-guidelines/)
- No `unsafe` code unless absolutely necessary (and document why)
- Use meaningful variable names
- Keep functions small and focused

### Documentation

```rust
/// Brief one-liner description.
///
/// More detailed explanation if needed.
///
/// # Examples
///
/// ```
/// use pm4py::discovery::TreeMiner;
/// let tree = TreeMiner::discover(&log)?;
/// ```
///
/// # Errors
///
/// Returns error if log is empty.
pub fn your_function(input: &EventLog) -> Result<ProcessTree> {
    // implementation
}
```

### Testing

- Minimum 80% code coverage
- Unit tests for functions
- Integration tests for workflows
- Property-based tests with `proptest` for complex logic
- Test edge cases and error conditions

```rust
#[test]
fn test_empty_log() {
    let empty_log = EventLog::new();
    // Should handle gracefully or return appropriate error
}

#[test]
fn test_large_log_performance() {
    let large_log = create_large_test_log(100_000);
    let start = std::time::Instant::now();
    let _ = your_function(&large_log);
    let elapsed = start.elapsed();
    assert!(elapsed.as_secs() < 5, "Should complete in < 5 seconds");
}
```

### Error Handling

Use proper error types:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum YourError {
    #[error("Empty log")]
    EmptyLog,
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
}

pub fn your_function(log: &EventLog) -> Result<ProcessTree, YourError> {
    if log.traces.is_empty() {
        return Err(YourError::EmptyLog);
    }
    Ok(/* result */)
}
```

### Performance

- Use benchmarks for critical paths
- Avoid unnecessary allocations
- Prefer iterators over collecting to vec
- Document complexity expectations

```rust
// Good: Uses iterators
log.traces.iter()
    .filter(|t| t.events.len() > 0)
    .map(|t| t.name.clone())
    .collect::<Vec<_>>()

// Avoid: Multiple allocations
let mut result = vec![];
for trace in &log.traces {
    if trace.events.len() > 0 {
        result.push(trace.name.clone());
    }
}
```

---

## Project Structure

```
pm4py-rust/
├── src/
│   ├── lib.rs                    # Library root
│   ├── version.rs                # Version information
│   ├── log/                      # Event log structures
│   ├── discovery/                # Discovery algorithms
│   ├── conformance/              # Conformance checking
│   ├── models/                   # Process models (Petri nets, etc.)
│   ├── visualization/            # SVG and layout algorithms
│   ├── io/                       # I/O formats (XES, CSV, etc.)
│   ├── statistics/               # Statistical analysis
│   ├── performance/              # Performance analysis
│   └── utils/                    # Utilities
├── tests/                        # Integration tests
├── examples/                     # Runnable examples
├── benches/                      # Benchmarks
├── docs/                         # Documentation
└── Cargo.toml                    # Project manifest
```

---

## Adding a New Feature

### 1. Create Module Structure

```
src/discovery/your_algorithm.rs
```

### 2. Implement Algorithm

```rust
pub struct YourMiner;

impl YourMiner {
    pub fn discover(log: &EventLog) -> Result<PetriNet> {
        // Implementation
    }
}
```

### 3. Add to Module Exports

```rust
// In src/discovery/mod.rs
pub mod your_algorithm;
pub use your_algorithm::YourMiner;

// In src/lib.rs
pub use discovery::YourMiner;
```

### 4. Write Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_log() {
        let log = create_test_log();
        let result = YourMiner::discover(&log);
        assert!(result.is_ok());
    }
}
```

### 5. Add Examples

```rust
// examples/your_example.rs
fn main() -> Result<()> {
    let log = EventLog::from_csv("examples/data/event_log.csv")?;
    let model = YourMiner::discover(&log)?;
    model.to_svg("output.svg")?;
    Ok(())
}
```

### 6. Document

- Add to README.md features list
- Add entry to CHANGELOG.md
- Write doc comments with examples
- Consider a detailed doc in docs/

---

## Pull Request Process

### Before Submitting

- [ ] Code follows style guidelines (run `cargo fmt`)
- [ ] No clippy warnings (`cargo clippy -- -D warnings`)
- [ ] Tests pass (`cargo test --all`)
- [ ] New public APIs have doc comments
- [ ] CHANGELOG.md updated
- [ ] No unnecessary dependencies added

### PR Description Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] New feature
- [ ] Bug fix
- [ ] Performance improvement
- [ ] Documentation
- [ ] Refactoring

## Related Issues
Fixes #123

## Testing
Describe testing done

## Performance Impact
If applicable, describe performance changes

## Checklist
- [ ] Tests added/updated
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] No breaking changes (or documented)
```

---

## Reporting Issues

### Bug Reports

Include:
- Clear title
- Steps to reproduce
- Expected behavior
- Actual behavior
- Rust version (`rustc --version`)
- Minimal reproducible example

```markdown
## Description
Brief description

## Steps to Reproduce
1. Load event log with...
2. Call algorithm...
3. Expected result...

## Actual Result
What happened instead

## Environment
- Rust: 1.75.0
- OS: macOS 14.0
```

### Feature Requests

Include:
- Clear title
- Use case description
- Proposed API (if applicable)
- Any relevant research or examples

---

## Code Review Process

All PRs require review before merging:

1. **Automated Checks**
   - Tests pass
   - Formatting correct
   - No clippy warnings

2. **Code Review**
   - Design and architecture
   - Performance implications
   - Documentation quality
   - Test coverage

3. **Approval**
   - At least one approval needed
   - All feedback addressed

---

## Communication

- **Issues**: Use GitHub Issues for bugs and features
- **Discussions**: Use GitHub Discussions for questions
- **Email**: info@chatmangpt.com for confidential issues
- **Slack/Discord**: None currently (use issues/email)

---

## Code of Conduct

This project adheres to the Rust Code of Conduct:
- Be respectful
- Be inclusive
- Be collaborative
- Assume good intent
- Address issues directly and professionally

---

## License

By contributing, you agree that your contributions will be licensed under the project's license (AGPL-3.0-or-later OR MIT).

---

## Recognition

Contributors will be recognized in:
- Git commit history
- GitHub contributors page
- CHANGELOG.md release notes
- README.md acknowledgments (for significant contributions)

---

Thank you for contributing to PM4Py Rust! 🙏

Questions? Open an issue or email info@chatmangpt.com
