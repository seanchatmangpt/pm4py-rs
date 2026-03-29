#!/bin/bash
# Simple test script to verify AlphaMiner fix

cd "$(dirname "$0")"

echo "Building pm4py library..."
cargo build --lib 2>&1 | grep -E "^error" || echo "Build successful"

echo ""
echo "Running AlphaMiner discovery tests..."
cargo test --lib discovery::alpha_miner -- --nocapture 2>&1 | tail -100
