.PHONY: help build test test-unit test-integration bench check coverage doc clean release fmt clippy logs debug profile status info doctor watch watch-test size outdated tree deny audit update-deps ci ci-all pre-commit

# Colors for output
BLUE := \033[0;34m
GREEN := \033[0;32m
RED := \033[0;31m
NC := \033[0m # No Color

help: ## Show this help message
	@echo "$(BLUE)PM4Py Rust - Build System$(NC)"
	@echo ""
	@echo "$(GREEN)Build:$(NC)"
	@grep -E '^(build|release|check):.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(BLUE)%-20s$(NC) %s\n", $$1, $$2}'
	@echo ""
	@echo "$(GREEN)Testing:$(NC)"
	@grep -E '^test.*:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(BLUE)%-20s$(NC) %s\n", $$1, $$2}'
	@echo ""
	@echo "$(GREEN)Code Quality:$(NC)"
	@grep -E '^(fmt|clippy|doc|audit):.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(BLUE)%-20s$(NC) %s\n", $$1, $$2}'
	@echo ""
	@echo "$(GREEN)Benchmarks & Profiling:$(NC)"
	@grep -E '^(bench|size|profile):.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(BLUE)%-20s$(NC) %s\n", $$1, $$2}'
	@echo ""
	@echo "$(GREEN)Maintenance:$(NC)"
	@grep -E '^(coverage|audit|deny|clean):.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(BLUE)%-20s$(NC) %s\n", $$1, $$2}'
	@echo ""
	@echo "$(GREEN)CI/CD:$(NC)"
	@grep -E '^(ci|pre-commit):.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(BLUE)%-20s$(NC) %s\n", $$1, $$2}'
	@echo ""
	@echo "$(GREEN)Helpers:$(NC)"
	@grep -E '^(watch|info|doctor):.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(BLUE)%-20s$(NC) %s\n", $$1, $$2}'
	@echo ""

# ============================================================================
# Build Targets
# ============================================================================

build: ## Build the project in debug mode
	@echo "$(BLUE)Building project...$(NC)"
	cargo build

release: ## Build optimized release binary
	@echo "$(BLUE)Building release...$(NC)"
	cargo build --release

check: fmt-check clippy doc-check ## Run all checks (format, clippy, docs)
	@echo "$(GREEN)✓ All checks passed$(NC)"

# ============================================================================
# Testing Targets
# ============================================================================

test: test-unit test-integration ## Run all tests (unit + integration)
	@echo "$(GREEN)✓ All tests passed$(NC)"

test-unit: ## Run unit tests only
	@echo "$(BLUE)Running unit tests...$(NC)"
	cargo test --lib

test-integration: ## Run integration tests only
	@echo "$(BLUE)Running integration tests...$(NC)"
	cargo test --test '*'

test-all-features: ## Run tests with all features enabled
	@echo "$(BLUE)Running tests with all features...$(NC)"
	cargo test --all-features

test-verbose: ## Run tests with verbose output
	@echo "$(BLUE)Running tests (verbose)...$(NC)"
	cargo test -- --nocapture

# ============================================================================
# Code Quality Targets
# ============================================================================

fmt: ## Format code with rustfmt
	@echo "$(BLUE)Formatting code...$(NC)"
	cargo fmt

fmt-check: ## Check code formatting without modifying
	@echo "$(BLUE)Checking code format...$(NC)"
	cargo fmt -- --check

clippy: ## Run clippy linter
	@echo "$(BLUE)Running clippy...$(NC)"
	cargo clippy --all-targets --all-features -- -D warnings

clippy-fix: ## Apply clippy suggestions automatically
	@echo "$(BLUE)Applying clippy fixes...$(NC)"
	cargo clippy --all-targets --all-features --fix --allow-dirty

# ============================================================================
# Documentation Targets
# ============================================================================

doc: ## Generate documentation
	@echo "$(BLUE)Generating documentation...$(NC)"
	cargo doc --no-deps --all-features

doc-open: doc ## Generate and open documentation in browser
	@echo "$(BLUE)Opening documentation...$(NC)"
	cargo doc --no-deps --all-features --open

doc-check: ## Check documentation without generating
	@echo "$(BLUE)Checking documentation...$(NC)"
	cargo doc --no-deps --all-features --document-private-items 2>&1 | grep -i warning && exit 1 || true

# ============================================================================
# Benchmark Targets
# ============================================================================

bench: ## Run all benchmarks
	@echo "$(BLUE)Running benchmarks...$(NC)"
	cargo bench --all-features

bench-discovery: ## Run discovery algorithm benchmarks
	@echo "$(BLUE)Running discovery benchmarks...$(NC)"
	cargo bench --bench discovery

bench-conformance: ## Run conformance checking benchmarks
	@echo "$(BLUE)Running conformance benchmarks...$(NC)"
	cargo bench --bench conformance

bench-io: ## Run I/O benchmarks
	@echo "$(BLUE)Running I/O benchmarks...$(NC)"
	cargo bench --bench io

bench-analysis: ## Run analysis benchmarks
	@echo "$(BLUE)Running analysis benchmarks...$(NC)"
	cargo bench --bench analysis

# ============================================================================
# Coverage Targets
# ============================================================================

coverage: ## Generate code coverage report
	@echo "$(BLUE)Generating coverage report...$(NC)"
	@command -v tarpaulin >/dev/null 2>&1 || (echo "$(RED)Installing cargo-tarpaulin...$(NC)" && cargo install cargo-tarpaulin)
	cargo tarpaulin --all-features --out Html --output-dir coverage
	@echo "$(GREEN)✓ Coverage report generated in coverage/tarpaulin-report.html$(NC)"

coverage-text: ## Generate text coverage report
	@echo "$(BLUE)Generating coverage report...$(NC)"
	@command -v tarpaulin >/dev/null 2>&1 || (echo "$(RED)Installing cargo-tarpaulin...$(NC)" && cargo install cargo-tarpaulin)
	cargo tarpaulin --all-features --out Stdout

# ============================================================================
# Maintenance Targets
# ============================================================================

clean: ## Clean build artifacts
	@echo "$(BLUE)Cleaning build artifacts...$(NC)"
	cargo clean

audit: ## Run security audit
	@echo "$(BLUE)Running security audit...$(NC)"
	@command -v cargo-audit >/dev/null 2>&1 || (echo "$(RED)Installing cargo-audit...$(NC)" && cargo install cargo-audit)
	cargo audit

deny: ## Check dependencies with cargo-deny
	@echo "$(BLUE)Checking dependencies...$(NC)"
	@command -v cargo-deny >/dev/null 2>&1 || (echo "$(RED)Installing cargo-deny...$(NC)" && cargo install cargo-deny)
	cargo deny check

update-deps: ## Update dependencies to latest versions
	@echo "$(BLUE)Updating dependencies...$(NC)"
	cargo update

outdated: ## Show outdated dependencies
	@echo "$(BLUE)Checking outdated dependencies...$(NC)"
	@command -v cargo-outdated >/dev/null 2>&1 || (echo "$(RED)Installing cargo-outdated...$(NC)" && cargo install cargo-outdated)
	cargo outdated

tree: ## Display dependency tree
	@echo "$(BLUE)Displaying dependency tree...$(NC)"
	cargo tree

# ============================================================================
# CI/CD-style Targets
# ============================================================================

ci: clean fmt-check clippy test-unit test-integration coverage ## Run all CI checks
	@echo "$(GREEN)✓ All CI checks passed$(NC)"

ci-all: clean fmt-check clippy test-all-features coverage doc-check ## Run all CI checks with all features
	@echo "$(GREEN)✓ All CI checks passed$(NC)"

pre-commit: fmt clippy test ## Run pre-commit checks
	@echo "$(GREEN)✓ Ready to commit$(NC)"

# ============================================================================
# Development Helpers
# ============================================================================

watch: ## Watch for changes and rebuild (requires cargo-watch)
	@command -v cargo-watch >/dev/null 2>&1 || (echo "$(RED)Installing cargo-watch...$(NC)" && cargo install cargo-watch)
	cargo watch -x build

watch-test: ## Watch for changes and run tests
	@command -v cargo-watch >/dev/null 2>&1 || (echo "$(RED)Installing cargo-watch...$(NC)" && cargo install cargo-watch)
	cargo watch -x test

size: ## Show binary size
	@echo "$(BLUE)Build sizes:$(NC)"
	@ls -lh target/release/pm4py 2>/dev/null || echo "Release binary not found. Run 'make release' first."
	@ls -lh target/debug/pm4py 2>/dev/null || echo "Debug binary not found. Run 'make build' first."

info: ## Show project information
	@echo "$(BLUE)PM4Py Rust Project Information$(NC)"
	@echo ""
	@echo "$(GREEN)Rust Version:$(NC)"
	@rustc --version
	@echo "$(GREEN)Cargo Version:$(NC)"
	@cargo --version
	@echo "$(GREEN)Dependencies:$(NC)"
	@cargo tree --depth 1
	@echo "$(GREEN)Package Info:$(NC)"
	@cargo pkgid

logs: ## Show build output logs (tail from last build)
	@echo "$(BLUE)Last build log:$(NC)"
	@ls -t target/debug/*pm4py 2>/dev/null | head -1 | xargs -r cat || echo "No build artifacts found"

debug: ## Show debug symbols info
	@echo "$(BLUE)Debug symbol information:$(NC)"
	@ls -lh target/debug/pm4py 2>/dev/null || echo "Debug binary not found. Run 'make build' first."
	@cargo build --message-format=json 2>&1 | jq '.reason' | sort | uniq -c

profile: ## Show Cargo.lock and dependency versions
	@echo "$(BLUE)Project Dependencies (top-level):$(NC)"
	@cargo tree --depth 0

status: ## Show build status
	@echo "$(BLUE)Build Status:$(NC)"
	@echo "  Debug binary: $$([ -f target/debug/pm4py ] && echo 'EXISTS' || echo 'MISSING (run: make build)')"
	@echo "  Release binary: $$([ -f target/release/pm4py ] && echo 'EXISTS' || echo 'MISSING (run: make release)')"

doctor: ## Check Rust environment and toolchain
	@echo "$(BLUE)Rust Environment Check$(NC)"
	@echo "$(GREEN)Rust Version:$(NC)"
	@rustc --version
	@echo "$(GREEN)Cargo Version:$(NC)"
	@cargo --version
	@echo "$(GREEN)Cargo features:$(NC)"
	@grep -A 100 '^\[features\]' Cargo.toml | head -20
	@echo "$(GREEN)Installed tools:$(NC)"
	@cargo install --list 2>/dev/null | head -10

# ============================================================================
# Weaver Live-Check Targets
# ============================================================================

weaver-live-check: ## Run tests with Weaver live-check OTEL export
	@echo "$(BLUE)Running Weaver live-check...$(NC)"
	WEAVER_LIVE_CHECK=true WEAVER_OTLP_ENDPOINT=http://localhost:4317 cargo test --test weaver_live_check_smoke -- --nocapture
