# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added
- **Board KPIs Feature** — Business intelligence metrics for process mining
  - `cycle_time_avg_ms` — Average case duration
  - `conformance_score` — 0.0-1.0 fitness via Alpha miner + Token Replay
  - `bottleneck_count` — Activities exceeding 1000ms threshold
  - `variant_count` — Unique process execution paths
  - 16 tests passing (8 integration + 8 standalone)
  - OTEL spans verified with schema conformance

- **Weaver Automation Pipeline** — Automated schema validation
  - CI/CD integration with 3 workflows
  - MCP server for Claude Code integration
  - Make targets for local development (`make weaver-check`, `make weaver-generate`)
  - Live schema checking and drift detection

- **Comprehensive API Documentation**
  - Quick start guide in `src/lib.rs`
  - Algorithm comparison tables
  - Usage examples for all major modules
  - Error handling documentation

### Changed
- Enhanced documentation across all modules (discovery, conformance, io, statistics)
- Improved test organization and deferred test analysis
- Synchronized semconv bindings across all projects (BusinessOS, OSA, Canopy)

### Fixed
- Type annotation error in `src/middleware/idempotency.rs`
- All 669 library tests passing
- Zero compilation errors

### Security
- XES security verification documentation added

### Performance
- Board KPIs calculation is O(n log n) for n traces
- Minimal memory overhead (<10MB for 10K events)

## [0.1.0] - 2024-03-24

### Added
- Initial release of PM4Py Rust
- **Core Data Structures**
  - Event log representation with traces and events
  - Petri net modeling with places, transitions, and arcs
  - Directly-follows graph (DFG) support
  - Transition system representation

- **Discovery Algorithms**
  - Alpha Miner algorithm
  - Inductive Miner algorithm
  - Heuristic Miner algorithm
  - DFG Miner for direct follows graph extraction

- **Conformance Checking**
  - Token replay conformance checking
  - Basic alignment analysis

- **Performance Analysis**
  - Case duration analysis (min, max, average, median)
  - Activity processing times
  - Waiting time analysis
  - Throughput calculation
  - Rework detection and metrics

- **Statistics**
  - Log statistics and summary
  - Trace variant analysis
  - Activity frequency and resource mapping

- **Utilities**
  - Log merging and splitting
  - Trace filtering and sampling
  - Outlier removal
  - Encoding support (one-hot, frequency-based, sequence)
  - Feature matrix generation

- **Input/Output**
  - XES (eXtensible Event Stream) format support
  - CSV format with flexible column mapping
  - JSON serialization via serde

- **Build & Tooling**
  - Production Makefile with comprehensive targets
  - GitHub Actions CI/CD workflows
  - Code coverage reporting
  - Security auditing
  - Benchmark framework with criterion

- **Documentation**
  - Comprehensive README
  - API documentation with examples
  - Release process documentation
  - Changelog

### Infrastructure
- GitHub Actions workflows:
  - Automated testing on stable, beta, nightly
  - Clippy and formatting checks
  - Documentation building
  - Code coverage with codecov
  - Security vulnerability scanning
  - Dependency auditing
- Cargo.toml with all metadata:
  - Author information
  - License (AGPL-3.0-or-later)
  - Repository and documentation URLs
  - Keywords and categories
  - Feature flags
  - Build optimizations

---

## Guidelines for Changelog Updates

When adding entries to the Unreleased section:

1. **Added** - For new features. Use present tense.
2. **Changed** - For changes in existing functionality.
3. **Fixed** - For bug fixes.
4. **Deprecated** - For soon-to-be removed features. Include migration guidance.
5. **Removed** - For now-removed features.
6. **Security** - For vulnerability fixes.

### Entry Format

Use present tense and provide context:

- Brief description of change with context
- Second feature that was added in this PR
- Fixed issue where X behavior occurred (fixes #123)

### Examples

**Good entries:**
- Added support for XES format parsing (#45)
- Improved Alpha Miner performance by 25% through caching (#67)
- Fixed bug where empty traces caused panic (#89)
- Deprecated `old_function()`, use `new_function()` instead (will be removed in 0.3.0)

### Release Checklist

When releasing:

1. Copy "Unreleased" section to new version heading
2. Add date in format `[0.X.Y] - YYYY-MM-DD`
3. Verify all entries are present and well-formatted
4. Create new empty "Unreleased" section for next changes
5. Verify links work correctly

## Historical Context

### Project Goals

PM4Py Rust aims to:
- Provide a one-for-one implementation of pm4py in Rust
- Maintain API parity where practical (adapted for Rust idioms)
- Offer native performance benefits for process mining
- Support both library and CLI use cases

### Design Principles

1. **API Compatibility**: Follow pm4py naming conventions
2. **Rust Idioms**: Use Rust best practices (Result types, ownership, etc.)
3. **Performance**: Native execution speed without GIL
4. **Correctness**: Comprehensive test coverage and validation
5. **Maintainability**: Clear code with excellent documentation

## References

- [Keep a Changelog](https://keepachangelog.com/)
- [Semantic Versioning](https://semver.org/)
- [PM4Py Documentation](https://pm4py.fit.fraunhofer.de/)
