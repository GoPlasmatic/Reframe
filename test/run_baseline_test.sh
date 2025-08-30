#!/bin/bash

# Run Baseline Performance Test for Reframe
# This script establishes baseline metrics for the current (unoptimized) architecture

echo "=========================================="
echo "Reframe Baseline Performance Test"
echo "=========================================="
echo ""

# Check if Reframe is running
if ! curl -s http://localhost:3000/health > /dev/null 2>&1; then
    echo "❌ Error: Reframe is not running on port 3000"
    echo ""
    echo "Please start Reframe first:"
    echo "  cargo build --release"
    echo "  RUST_LOG=info cargo run --release"
    echo ""
    exit 1
fi

echo "✅ Reframe is running on port 3000"
echo ""

# Create results directory
RESULTS_DIR="test/results/baseline_$(date +%Y%m%d_%H%M%S)"
mkdir -p $RESULTS_DIR

echo "📁 Results will be saved to: $RESULTS_DIR"
echo ""

# System info
echo "📊 System Information:"
echo "------------------------"
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "CPU Cores: $(sysctl -n hw.physicalcpu)"
    echo "Memory: $(( $(sysctl -n hw.memsize) / 1024 / 1024 / 1024 )) GB"
else
    echo "CPU Cores: $(nproc)"
    echo "Memory: $(free -h | grep Mem | awk '{print $2}')"
fi
echo "Rust: $(rustc --version)"
echo "Cargo: $(cargo --version)"
echo ""

# Run tests
echo "🚀 Starting Performance Tests..."
echo "================================="
echo ""

# 1. Single-threaded baseline
echo "1️⃣ Running Single-threaded Baseline Test..."
python3 test/performance_test.py --baseline --save $RESULTS_DIR/single_thread.json
echo ""

# 2. Low concurrency test
echo "2️⃣ Running Load Test (10 concurrent connections)..."
python3 test/performance_test.py --test load -c 10 -n 500 --save $RESULTS_DIR/load_c10.json
echo ""

# 3. Medium concurrency test
echo "3️⃣ Running Load Test (50 concurrent connections)..."
python3 test/performance_test.py --test load -c 50 -n 1000 --save $RESULTS_DIR/load_c50.json
echo ""

# 4. High concurrency test (expected to show bottleneck)
echo "4️⃣ Running Load Test (100 concurrent connections)..."
python3 test/performance_test.py --test load -c 100 -n 1000 --save $RESULTS_DIR/load_c100.json
echo ""

# 5. Quick stress test
echo "5️⃣ Running Stress Test (up to 100 connections)..."
python3 test/performance_test.py --test stress --max-concurrency 100 --save $RESULTS_DIR/stress.json
echo ""

# Summary
echo "=========================================="
echo "✅ Baseline Testing Complete!"
echo "=========================================="

# Extract and display version info
if [ -f "$RESULTS_DIR/single_thread.json" ]; then
    VERSION=$(jq -r '.reframe_version' $RESULTS_DIR/single_thread.json 2>/dev/null || echo "unknown")
    echo ""
    echo "📦 Reframe Version: $VERSION"
fi

echo ""
echo "Results saved to: $RESULTS_DIR"
echo ""
echo "Key findings to note:"
echo "- Single-threaded throughput (baseline)"
echo "- Point where performance degrades (concurrency limit)"
echo "- CPU utilization (should be low, ~6-12%)"
echo "- Error rate at high concurrency"
echo ""
echo "These metrics will be compared after implementing optimizations from scaling.md"
echo ""
echo "To view detailed results:"
echo "  cat $RESULTS_DIR/*.json | jq ."
echo ""
echo "To generate HTML report:"
echo "  python3 test/performance_test.py --html-report"