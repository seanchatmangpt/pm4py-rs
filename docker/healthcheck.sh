#!/bin/bash
# Health check script for pm4py-rust container
# Verifies the axum HTTP server is responding on port 8090

set -e

curl -sf http://localhost:8090/api/health || {
    echo "Health check failed: pm4py-rust HTTP server not responding on port 8090"
    exit 1
}
