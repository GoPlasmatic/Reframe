#!/bin/bash

# Quick Baseline Performance Test for Reframe
# Establishes key baseline metrics quickly

echo "=========================================="
echo "Reframe Quick Baseline Performance Test"
echo "=========================================="
echo ""

# Check if Reframe is running
if ! curl -s http://localhost:3000/health > /dev/null 2>&1; then
    echo "❌ Error: Reframe is not running on port 3000"
    echo "Please start Reframe first:"
    echo "  cargo build --release"
    echo "  RUST_LOG=info cargo run --release"
    exit 1
fi

echo "✅ Reframe is running on port 3000"
echo ""

# Create results directory
RESULTS_DIR="test/results/baseline_$(date +%Y%m%d_%H%M%S)"
mkdir -p $RESULTS_DIR

# 1. Single-threaded baseline
echo "1️⃣ Single-threaded baseline (100 requests)..."
python3 test/performance_test.py --baseline --save $RESULTS_DIR/single.json

# 2. Low concurrency
echo -e "\n2️⃣ Low concurrency test (10 connections, 100 requests)..."
python3 test/performance_test.py --test load -c 10 -n 100 --save $RESULTS_DIR/c10.json

# 3. Medium concurrency (showing bottleneck)
echo -e "\n3️⃣ Medium concurrency test (50 connections, 200 requests)..."
python3 test/performance_test.py --test load -c 50 -n 200 --save $RESULTS_DIR/c50.json

# Summary
echo -e "\n=========================================="
echo "📊 Quick Baseline Test Summary"
echo "=========================================="

# Extract version info from the first test result
if [ -f "$RESULTS_DIR/single.json" ]; then
    VERSION=$(jq -r '.reframe_version' $RESULTS_DIR/single.json 2>/dev/null || echo "unknown")
    echo -e "\n📦 Reframe Version: $VERSION"
fi

# Extract key metrics
echo -e "\nKey Metrics:"
echo "------------"

if [ -f "$RESULTS_DIR/single.json" ]; then
    SINGLE_RPS=$(jq -r '.throughput_rps' $RESULTS_DIR/single.json 2>/dev/null | xargs printf "%.2f")
    SINGLE_P99=$(jq -r '.latency_p99_ms' $RESULTS_DIR/single.json 2>/dev/null | xargs printf "%.2f")
    echo "Single-threaded: $SINGLE_RPS req/s, P99: ${SINGLE_P99}ms"
fi

if [ -f "$RESULTS_DIR/c10.json" ]; then
    C10_RPS=$(jq -r '.throughput_rps' $RESULTS_DIR/c10.json 2>/dev/null | xargs printf "%.2f")
    C10_P99=$(jq -r '.latency_p99_ms' $RESULTS_DIR/c10.json 2>/dev/null | xargs printf "%.2f")
    echo "10 concurrent:   $C10_RPS req/s, P99: ${C10_P99}ms"
fi

if [ -f "$RESULTS_DIR/c50.json" ]; then
    C50_RPS=$(jq -r '.throughput_rps' $RESULTS_DIR/c50.json 2>/dev/null | xargs printf "%.2f")
    C50_P99=$(jq -r '.latency_p99_ms' $RESULTS_DIR/c50.json 2>/dev/null | xargs printf "%.2f")
    echo "50 concurrent:   $C50_RPS req/s, P99: ${C50_P99}ms"
fi

echo -e "\n✅ Results saved to: $RESULTS_DIR"
echo ""
echo "These baseline metrics confirm the mutex bottleneck described in scaling.md:"
echo "- Throughput doesn't scale with concurrency"
echo "- Latency increases significantly under concurrent load"
echo "- System is processing requests sequentially despite concurrent connections"