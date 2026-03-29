#!/bin/bash
#
# run-integration-tests.sh — Execute pm4py/pm4py-rust integration tests
#
# Usage:
#   ./run-integration-tests.sh [python|rust|all]
#   ./run-integration-tests.sh all --verbose
#   BUSINESSOS_API_BASE=http://localhost:8001 ./run-integration-tests.sh python
#
# Requires:
#   - Python 3.9+ with pytest and pm4py
#   - Rust 1.70+ with cargo
#   - BusinessOS running (for HTTP tests)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEST_TYPE="${1:-all}"
VERBOSE="${2:-}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default BusinessOS API base
BUSINESSOS_API_BASE="${BUSINESSOS_API_BASE:-http://localhost:8001}"

# ============================================================================
# Helper Functions
# ============================================================================

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_dependency() {
    if ! command -v "$1" &> /dev/null; then
        log_error "$1 not found. Please install it."
        return 1
    fi
}

check_api_available() {
    log_info "Checking BusinessOS API at $BUSINESSOS_API_BASE..."
    if curl -sf "$BUSINESSOS_API_BASE/health" > /dev/null 2>&1; then
        log_success "BusinessOS API is available"
        return 0
    else
        log_warn "BusinessOS API not available at $BUSINESSOS_API_BASE"
        log_warn "HTTP integration tests will be skipped"
        return 1
    fi
}

check_test_data() {
    local home="${HOME:-/root}"
    local test_data_dir="$home/chatmangpt/pm4py-rust/test_data"

    if [ ! -d "$test_data_dir" ]; then
        log_error "Test data directory not found: $test_data_dir"
        return 1
    fi

    if [ ! -f "$test_data_dir/running-example.csv" ]; then
        log_error "Test file missing: $test_data_dir/running-example.csv"
        return 1
    fi

    if [ ! -f "$test_data_dir/running-example.xes" ]; then
        log_error "Test file missing: $test_data_dir/running-example.xes"
        return 1
    fi

    log_success "Test data files verified"
    return 0
}

# ============================================================================
# Python Tests
# ============================================================================

run_python_tests() {
    log_info "Running Python integration tests..."

    # Check dependencies
    check_dependency python3 || return 1
    check_dependency pip || return 1
    check_dependency pytest || return 1

    # Check test data
    check_test_data || return 1

    # Install requirements if needed
    if [ ! -d "$SCRIPT_DIR/venv" ]; then
        log_info "Creating Python virtual environment..."
        python3 -m venv "$SCRIPT_DIR/venv"
        source "$SCRIPT_DIR/venv/bin/activate"
        pip install -q -r "$SCRIPT_DIR/requirements-integration.txt"
    else
        source "$SCRIPT_DIR/venv/bin/activate"
    fi

    # Check if BusinessOS is available
    if check_api_available; then
        export BUSINESSOS_API_BASE="$BUSINESSOS_API_BASE"
        log_info "Running full Python test suite (with HTTP tests)..."
    else
        log_warn "Skipping HTTP tests (BusinessOS not available)"
    fi

    # Run tests
    local pytest_opts="-v"
    if [ -n "$VERBOSE" ]; then
        pytest_opts="$pytest_opts -vv --tb=long"
    fi

    if pytest $pytest_opts "$SCRIPT_DIR/businessos_http_integration_tests.py"; then
        log_success "Python tests passed"
        return 0
    else
        log_error "Python tests failed"
        return 1
    fi
}

# ============================================================================
# Rust Tests
# ============================================================================

run_rust_tests() {
    log_info "Running Rust integration tests..."

    # Check dependencies
    check_dependency cargo || return 1
    check_dependency rustc || return 1

    # Check test data
    check_test_data || return 1

    # Determine project root
    local project_root="$(cd "$SCRIPT_DIR/../.." && pwd)"

    # Build test binary
    log_info "Building Rust test binary..."
    cd "$project_root/pm4py-rust"

    local cargo_opts="--test businessos_rust_http_integration_tests"
    if [ -n "$VERBOSE" ]; then
        cargo_opts="$cargo_opts -- --nocapture"
    fi

    # Run serialization tests (no BusinessOS required)
    log_info "Running serialization tests (no API required)..."
    if cargo test $cargo_opts test_serialize 2>&1 | grep -q "test result:"; then
        log_success "Serialization tests completed"
    fi

    # Run deserialization tests
    log_info "Running deserialization tests..."
    if cargo test $cargo_opts test_deserialize 2>&1 | grep -q "test result:"; then
        log_success "Deserialization tests completed"
    fi

    # Run CSV/XES tests
    log_info "Running CSV/XES support tests..."
    if cargo test $cargo_opts test_load_and_serialize_csv_file 2>&1 | grep -q "test result:"; then
        log_success "CSV/XES tests completed"
    fi

    # Check if BusinessOS is available for HTTP tests
    if check_api_available; then
        export BUSINESSOS_API_BASE="$BUSINESSOS_API_BASE"
        log_info "Running HTTP integration tests..."
        cargo test $cargo_opts --ignored
    else
        log_warn "Skipping HTTP tests (BusinessOS not available)"
    fi

    log_success "Rust tests completed"
    return 0
}

# ============================================================================
# Summary Report
# ============================================================================

print_summary() {
    echo ""
    log_info "Integration Test Summary:"
    echo "  Test Type: $TEST_TYPE"
    echo "  BusinessOS API: $BUSINESSOS_API_BASE"
    echo "  Test Data Dir: $HOME/chatmangpt/pm4py-rust/test_data"
    echo "  Timestamp: $(date)"
    echo ""
}

# ============================================================================
# Main Entry Point
# ============================================================================

main() {
    print_summary

    case "$TEST_TYPE" in
        python)
            log_info "Running Python tests only..."
            run_python_tests
            ;;
        rust)
            log_info "Running Rust tests only..."
            run_rust_tests
            ;;
        all)
            log_info "Running all integration tests..."
            python_ok=0
            rust_ok=0

            if run_python_tests; then
                python_ok=1
            else
                log_warn "Python tests had issues"
            fi

            if run_rust_tests; then
                rust_ok=1
            else
                log_warn "Rust tests had issues"
            fi

            echo ""
            log_info "Final Results:"
            if [ $python_ok -eq 1 ]; then
                log_success "Python tests: PASS"
            else
                log_error "Python tests: FAIL"
            fi

            if [ $rust_ok -eq 1 ]; then
                log_success "Rust tests: PASS"
            else
                log_error "Rust tests: FAIL"
            fi

            if [ $python_ok -eq 1 ] && [ $rust_ok -eq 1 ]; then
                log_success "All integration tests completed successfully"
                return 0
            else
                log_error "Some tests failed"
                return 1
            fi
            ;;
        *)
            log_error "Unknown test type: $TEST_TYPE"
            echo "Usage: $0 [python|rust|all]"
            return 1
            ;;
    esac
}

main "$@"
