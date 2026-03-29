#!/bin/bash

###############################################################################
# Comprehensive Benchmark Suite Runner
#
# Runs both Rust and Python benchmarks, then compares results
#
# Usage:
#   ./scripts/run_benchmarks.sh [--rust-only] [--python-only] [--output DIR]
###############################################################################

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
OUTPUT_DIR="${OUTPUT_DIR:-.}"
RUN_RUST=true
RUN_PYTHON=true

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --rust-only)
            RUN_PYTHON=false
            shift
            ;;
        --python-only)
            RUN_RUST=false
            shift
            ;;
        --output)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Create output directory
mkdir -p "$OUTPUT_DIR"

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}  PM4PY-RUST COMPREHENSIVE BENCHMARK SUITE${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# ============================================================================
# RUST BENCHMARKS
# ============================================================================

if [ "$RUN_RUST" = true ]; then
    echo -e "${GREEN}🦀 Running Rust Benchmarks${NC}"
    echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""

    cd "$PROJECT_ROOT"

    RUST_TIMESTAMP=$(date +%Y%m%d_%H%M%S)
    RUST_RESULTS="$OUTPUT_DIR/rust_benchmarks_${RUST_TIMESTAMP}.json"

    echo "Building release version..."
    if ! cargo build --release 2>/dev/null; then
        echo -e "${RED}✗ Build failed${NC}"
        exit 1
    fi
    echo -e "${GREEN}✓ Build complete${NC}"
    echo ""

    echo "Running scale benchmarks..."
    if cargo bench --bench scale_benchmarks -- --output-format bencher \
        | tee "$OUTPUT_DIR/rust_scale_output.txt"; then
        echo -e "${GREEN}✓ Scale benchmarks complete${NC}"
    else
        echo -e "${YELLOW}⚠ Scale benchmarks had some issues${NC}"
    fi
    echo ""

    echo "Running discovery benchmarks..."
    if cargo bench --bench discovery_bench -- --output-format bencher \
        | tee "$OUTPUT_DIR/rust_discovery_output.txt"; then
        echo -e "${GREEN}✓ Discovery benchmarks complete${NC}"
    else
        echo -e "${YELLOW}⚠ Discovery benchmarks had some issues${NC}"
    fi
    echo ""

    echo "Running conformance benchmarks..."
    if cargo bench --bench comprehensive_conformance_bench -- --output-format bencher \
        | tee "$OUTPUT_DIR/rust_conformance_output.txt"; then
        echo -e "${GREEN}✓ Conformance benchmarks complete${NC}"
    else
        echo -e "${YELLOW}⚠ Conformance benchmarks had some issues${NC}"
    fi
    echo ""

    echo "Running statistics benchmarks..."
    if cargo bench --bench comprehensive_statistics_bench -- --output-format bencher \
        | tee "$OUTPUT_DIR/rust_statistics_output.txt"; then
        echo -e "${GREEN}✓ Statistics benchmarks complete${NC}"
    else
        echo -e "${YELLOW}⚠ Statistics benchmarks had some issues${NC}"
    fi
    echo ""

    echo -e "${GREEN}✓ All Rust benchmarks complete${NC}"
    echo "Results: $OUTPUT_DIR/rust_*"
    echo ""
fi

# ============================================================================
# PYTHON BENCHMARKS
# ============================================================================

if [ "$RUN_PYTHON" = true ]; then
    echo -e "${GREEN}🐍 Running Python pm4py Benchmarks${NC}"
    echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""

    # Check Python availability
    if ! command_exists python3; then
        echo -e "${RED}✗ Python3 not found${NC}"
        exit 1
    fi

    # Check pm4py availability
    if ! python3 -c "import pm4py" 2>/dev/null; then
        echo -e "${YELLOW}⚠ pm4py not installed${NC}"
        echo "  Install with: pip install pm4py psutil pandas"
        echo "  Skipping Python benchmarks"
    else
        PYTHON_TIMESTAMP=$(date +%Y%m%d_%H%M%S)
        PYTHON_RESULTS="$OUTPUT_DIR/python_benchmarks_${PYTHON_TIMESTAMP}.json"

        echo "Running Python benchmarks..."
        echo "Output: $PYTHON_RESULTS"
        echo ""

        if python3 "$SCRIPT_DIR/python_benchmark.py" \
            --output "$PYTHON_RESULTS" \
            --warmup 1; then
            echo ""
            echo -e "${GREEN}✓ Python benchmarks complete${NC}"
            echo "Results: $PYTHON_RESULTS"
        else
            echo -e "${YELLOW}⚠ Python benchmarks had some issues${NC}"
        fi
    fi
    echo ""
fi

# ============================================================================
# COMPARISON
# ============================================================================

if [ "$RUN_RUST" = true ] && [ "$RUN_PYTHON" = true ]; then
    echo ""
    echo -e "${GREEN}📊 Generating Comparison Report${NC}"
    echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""

    if command_exists python3 && [ -f "$SCRIPT_DIR/compare_benchmarks.py" ]; then
        # Find the latest results files
        LATEST_RUST=$(ls -t "$OUTPUT_DIR"/rust_benchmarks_*.json 2>/dev/null | head -1)
        LATEST_PYTHON=$(ls -t "$OUTPUT_DIR"/python_benchmarks_*.json 2>/dev/null | head -1)

        if [ -n "$LATEST_RUST" ] && [ -n "$LATEST_PYTHON" ]; then
            COMPARISON_OUTPUT="$OUTPUT_DIR/performance_comparison_${RUST_TIMESTAMP}.json"
            MARKDOWN_OUTPUT="$OUTPUT_DIR/PERFORMANCE_COMPARISON_${RUST_TIMESTAMP}.md"

            echo "Comparing results..."
            python3 "$SCRIPT_DIR/compare_benchmarks.py" \
                "$LATEST_RUST" \
                "$LATEST_PYTHON" \
                --output-json "$COMPARISON_OUTPUT" \
                --output-md "$MARKDOWN_OUTPUT"

            echo ""
            echo -e "${GREEN}✓ Comparison complete${NC}"
            echo "JSON Report: $COMPARISON_OUTPUT"
            echo "Markdown Report: $MARKDOWN_OUTPUT"
        else
            echo -e "${YELLOW}⚠ Could not find benchmark result files for comparison${NC}"
        fi
    else
        echo -e "${YELLOW}⚠ Comparison script not available${NC}"
    fi
    echo ""
fi

# ============================================================================
# SUMMARY
# ============================================================================

echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}  BENCHMARK SUITE COMPLETE${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "Output directory: $OUTPUT_DIR"
echo ""
echo "Files generated:"
ls -lh "$OUTPUT_DIR"/ | grep -E "\.(json|txt|md)$" | awk '{print "  " $9 " (" $5 ")"}'
echo ""
echo -e "${GREEN}✓ All benchmarks complete${NC}"
echo ""
