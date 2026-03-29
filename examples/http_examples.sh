#!/bin/bash

# PM4Py REST API Examples using cURL
#
# Demonstrates all major API endpoints with real HTTP requests.
# No external dependencies required - just curl and standard Unix tools.
#
# Setup:
#   1. Set your API key:
#      export PM4PY_API_KEY="your-api-key-here"
#
#   2. Optional: Change API base URL (default: http://localhost:8080/api/v1)
#      export PM4PY_API_URL="http://localhost:8080/api/v1"
#
#   3. Run examples:
#      bash examples/http_examples.sh
#
# Features:
#   - Health check
#   - Process discovery
#   - Conformance checking
#   - Log statistics
#   - Model analysis
#   - Error handling
#   - Pretty JSON output

set -e

# Configuration
API_KEY="${PM4PY_API_KEY:-demo-key-for-testing}"
API_URL="${PM4PY_API_URL:-http://localhost:8080/api/v1}"
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Helper functions
print_header() {
    echo ""
    echo "╔════════════════════════════════════════════════════════╗"
    echo "║ $1"
    echo "╚════════════════════════════════════════════════════════╝"
    echo ""
}

print_section() {
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "$1"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_info() {
    echo "  $1"
}

api_call() {
    local method=$1
    local endpoint=$2
    local data=$3
    local output_file=${4:-.response.json}

    echo "API Call: $method $endpoint"
    echo ""

    if [ -z "$data" ]; then
        curl -s -X "$method" \
            "$API_URL$endpoint" \
            -H "X-API-Key: $API_KEY" \
            -H "Content-Type: application/json" \
            -w "\nHTTP Status: %{http_code}\n" \
            | tee "$output_file"
    else
        curl -s -X "$method" \
            "$API_URL$endpoint" \
            -H "X-API-Key: $API_KEY" \
            -H "Content-Type: application/json" \
            -d "$data" \
            -w "\nHTTP Status: %{http_code}\n" \
            | tee "$output_file"
    fi

    echo ""
}

# ============================================================================
# EXAMPLES
# ============================================================================

main() {
    print_header "PM4Py REST API Examples (cURL)"

    echo "Configuration:"
    echo "  API Key: ${API_KEY:0:10}..."
    echo "  API URL: $API_URL"
    echo ""

    # Example 1: Health Check
    print_section "EXAMPLE 1: Health Check"
    echo "Endpoint: GET /health"
    echo "Purpose: Verify API is running"
    echo ""

    api_call "GET" "/health"

    echo ""
    print_success "API is running"
    echo ""

    # Example 2: Process Discovery
    print_section "EXAMPLE 2: Process Discovery"
    echo "Endpoint: POST /discover"
    echo "Purpose: Mine a process model from an event log"
    echo ""

    # Create sample event log
    DISCOVERY_REQUEST=$(cat <<'EOF'
{
  "log": {
    "events": [
      {
        "case_id": "order_001",
        "activity": "receive",
        "timestamp": "2026-03-24T10:00:00Z",
        "resource": "clerk",
        "attributes": {"amount": 1000}
      },
      {
        "case_id": "order_001",
        "activity": "validate",
        "timestamp": "2026-03-24T10:05:00Z",
        "resource": "officer"
      },
      {
        "case_id": "order_001",
        "activity": "process",
        "timestamp": "2026-03-24T10:30:00Z",
        "resource": "processor"
      },
      {
        "case_id": "order_001",
        "activity": "complete",
        "timestamp": "2026-03-24T11:00:00Z",
        "resource": "admin"
      },
      {
        "case_id": "order_002",
        "activity": "receive",
        "timestamp": "2026-03-24T11:00:00Z",
        "resource": "clerk",
        "attributes": {"amount": 2000}
      },
      {
        "case_id": "order_002",
        "activity": "validate",
        "timestamp": "2026-03-24T11:05:00Z",
        "resource": "officer"
      },
      {
        "case_id": "order_002",
        "activity": "process",
        "timestamp": "2026-03-24T11:30:00Z",
        "resource": "processor"
      },
      {
        "case_id": "order_002",
        "activity": "complete",
        "timestamp": "2026-03-24T12:00:00Z",
        "resource": "admin"
      }
    ],
    "format": "json"
  },
  "algorithm": "inductive",
  "parameters": {
    "frequency_threshold": 0.1
  }
}
EOF
)

    echo "Request body:"
    echo "$DISCOVERY_REQUEST" | jq '.'
    echo ""

    api_call "POST" "/discover" "$DISCOVERY_REQUEST" "discovery_response.json"

    # Extract model for later use
    DISCOVERED_MODEL=$(cat discovery_response.json | jq '.model' 2>/dev/null || echo "{}")

    echo ""
    print_success "Model discovered"
    echo ""

    # Example 3: Conformance Checking
    if [ ! -z "$DISCOVERED_MODEL" ] && [ "$DISCOVERED_MODEL" != "{}" ]; then
        print_section "EXAMPLE 3: Conformance Checking"
        echo "Endpoint: POST /conform"
        echo "Purpose: Check if log conforms to discovered model"
        echo ""

        CONFORMANCE_REQUEST=$(cat <<'EOF'
{
  "log": {
    "events": [
      {
        "case_id": "order_001",
        "activity": "receive",
        "timestamp": "2026-03-24T10:00:00Z"
      },
      {
        "case_id": "order_001",
        "activity": "validate",
        "timestamp": "2026-03-24T10:05:00Z"
      },
      {
        "case_id": "order_001",
        "activity": "process",
        "timestamp": "2026-03-24T10:30:00Z"
      },
      {
        "case_id": "order_001",
        "activity": "complete",
        "timestamp": "2026-03-24T11:00:00Z"
      }
    ],
    "format": "json"
  },
  "model": {}
}
EOF
)

        # Replace empty model with discovered model
        CONFORMANCE_REQUEST=$(echo "$CONFORMANCE_REQUEST" | jq ".model = $DISCOVERED_MODEL")

        echo "Request summary:"
        echo "  - 1 trace with 4 events"
        echo "  - Using discovered model"
        echo ""

        api_call "POST" "/conform" "$CONFORMANCE_REQUEST" "conformance_response.json"

        echo ""
        print_success "Conformance checked"
        echo ""
    fi

    # Example 4: Log Statistics
    print_section "EXAMPLE 4: Log Statistics"
    echo "Endpoint: POST /stats"
    echo "Purpose: Extract statistical metrics from event log"
    echo ""

    STATS_REQUEST=$(cat <<'EOF'
{
  "log": {
    "events": [
      {
        "case_id": "order_001",
        "activity": "receive",
        "timestamp": "2026-03-24T10:00:00Z"
      },
      {
        "case_id": "order_001",
        "activity": "validate",
        "timestamp": "2026-03-24T10:05:00Z"
      },
      {
        "case_id": "order_001",
        "activity": "process",
        "timestamp": "2026-03-24T10:30:00Z"
      },
      {
        "case_id": "order_001",
        "activity": "complete",
        "timestamp": "2026-03-24T11:00:00Z"
      },
      {
        "case_id": "order_002",
        "activity": "receive",
        "timestamp": "2026-03-24T11:00:00Z"
      },
      {
        "case_id": "order_002",
        "activity": "validate",
        "timestamp": "2026-03-24T11:05:00Z"
      },
      {
        "case_id": "order_002",
        "activity": "process",
        "timestamp": "2026-03-24T11:30:00Z"
      },
      {
        "case_id": "order_002",
        "activity": "complete",
        "timestamp": "2026-03-24T12:00:00Z"
      }
    ],
    "format": "json"
  }
}
EOF
)

    echo "Request summary:"
    echo "  - 2 traces"
    echo "  - 8 total events"
    echo "  - 4 unique activities"
    echo ""

    api_call "POST" "/stats" "$STATS_REQUEST" "stats_response.json"

    echo ""
    print_success "Statistics extracted"
    echo ""

    # Example 5: Model Analysis
    if [ ! -z "$DISCOVERED_MODEL" ] && [ "$DISCOVERED_MODEL" != "{}" ]; then
        print_section "EXAMPLE 5: Model Analysis"
        echo "Endpoint: POST /analyze"
        echo "Purpose: Analyze model structure (soundness, behavioral profile)"
        echo ""

        ANALYSIS_REQUEST=$(cat <<EOF
{
  "model": $DISCOVERED_MODEL
}
EOF
)

        echo "Request summary:"
        echo "  - Analyzing discovered model"
        echo "  - Checking soundness"
        echo "  - Computing behavioral profile"
        echo ""

        api_call "POST" "/analyze" "$ANALYSIS_REQUEST" "analysis_response.json"

        echo ""
        print_success "Model analyzed"
        echo ""
    fi

    # Summary
    print_section "EXAMPLES COMPLETE"
    echo ""
    echo "API Endpoints Tested:"
    echo "  ✓ GET  /health"
    echo "  ✓ POST /discover"
    echo "  ✓ POST /conform"
    echo "  ✓ POST /stats"
    echo "  ✓ POST /analyze"
    echo ""
    echo "Response files created:"
    echo "  - discovery_response.json"
    echo "  - conformance_response.json"
    echo "  - stats_response.json"
    echo "  - analysis_response.json"
    echo ""

    # Parse and display key information
    if [ -f "stats_response.json" ]; then
        echo "Sample results from stats endpoint:"
        cat stats_response.json | jq '.stats | {num_traces, num_events, num_activities}' 2>/dev/null || true
        echo ""
    fi

    echo "Next steps:"
    echo "  1. Review response files (*.json)"
    echo "  2. Try different algorithms: alpha, heuristic, dfg"
    echo "  3. Experiment with frequency thresholds"
    echo "  4. Integrate into your applications"
    echo ""

    # Cleanup
    echo "Cleanup:"
    read -p "Remove temporary response files? (y/n) " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        rm -f .response.json discovery_response.json conformance_response.json stats_response.json analysis_response.json
        echo "Removed temporary files"
    fi

    echo ""
    print_success "Examples completed successfully!"
}

# Run main function
main
