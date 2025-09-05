#!/bin/bash

# Reframe Performance Testing Suite
# Comprehensive performance analysis with detailed metrics and visualizations

set -e

# Color codes for better output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
NC='\033[0m' # No Color
BOLD='\033[1m'

# Performance thresholds (for highlighting)
GOOD_LATENCY_P99=10  # ms
WARN_LATENCY_P99=20  # ms
GOOD_THROUGHPUT=2000 # req/s
WARN_THROUGHPUT=1000 # req/s

# Function to print colored output
print_header() {
    echo -e "\n${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${WHITE}${BOLD}$1${NC}"
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

print_subheader() {
    echo -e "\n${YELLOW}▶ $1${NC}"
    echo -e "${YELLOW}$(printf '─%.0s' {1..80})${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ ${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_metric() {
    local label="$1"
    local value="$2"
    local unit="$3"
    local threshold_good="$4"
    local threshold_warn="$5"
    
    # Format label to fixed width
    printf "${WHITE}%-25s${NC}" "$label:"
    
    # Color code based on thresholds
    if [[ -n "$threshold_good" && -n "$threshold_warn" ]]; then
        # Extract numeric value for comparison
        numeric_value=$(echo "$value" | grep -o '[0-9.]*' | head -1)
        if (( $(echo "$numeric_value < $threshold_good" | bc -l) )); then
            echo -e "${GREEN}${BOLD}$value${NC} $unit"
        elif (( $(echo "$numeric_value < $threshold_warn" | bc -l) )); then
            echo -e "${YELLOW}${BOLD}$value${NC} $unit"
        else
            echo -e "${RED}${BOLD}$value${NC} $unit"
        fi
    else
        echo -e "${WHITE}${BOLD}$value${NC} $unit"
    fi
}

# Check if required tools are installed
check_requirements() {
    local missing=()
    
    command -v curl >/dev/null 2>&1 || missing+=("curl")
    command -v jq >/dev/null 2>&1 || missing+=("jq")
    command -v bc >/dev/null 2>&1 || missing+=("bc")
    command -v python3 >/dev/null 2>&1 || missing+=("python3")
    
    if [ ${#missing[@]} -gt 0 ]; then
        print_error "Missing required tools: ${missing[*]}"
        echo "Please install them and try again."
        exit 1
    fi
}

# Get system information
get_system_info() {
    print_subheader "System Configuration"
    
    if [[ "$OSTYPE" == "darwin"* ]]; then
        local cpu_cores=$(sysctl -n hw.physicalcpu)
        local cpu_threads=$(sysctl -n hw.logicalcpu)
        local memory_gb=$(( $(sysctl -n hw.memsize) / 1024 / 1024 / 1024 ))
        local cpu_model=$(sysctl -n machdep.cpu.brand_string | sed 's/  */ /g')
        local os_version=$(sw_vers -productVersion)
        local os_name="macOS"
    else
        local cpu_cores=$(lscpu | grep "^Core(s) per socket:" | awk '{print $4}')
        local cpu_threads=$(nproc)
        local memory_gb=$(free -g | grep Mem | awk '{print $2}')
        local cpu_model=$(lscpu | grep "Model name:" | sed 's/Model name:[ ]*//')
        local os_version=$(lsb_release -r | awk '{print $2}')
        local os_name=$(lsb_release -i | awk '{print $3}')
    fi
    
    print_metric "Operating System" "$os_name $os_version" ""
    print_metric "CPU Model" "$cpu_model" ""
    print_metric "CPU Cores" "$cpu_cores" "physical"
    print_metric "CPU Threads" "$cpu_threads" "logical"
    print_metric "Memory" "$memory_gb" "GB"
    print_metric "Rust Version" "$(rustc --version | cut -d' ' -f2)" ""
    print_metric "Cargo Version" "$(cargo --version | cut -d' ' -f2)" ""
}

# Check Reframe health and get configuration
check_reframe() {
    print_subheader "Reframe Service Status"
    
    if ! curl -s http://localhost:3000/health > /dev/null 2>&1; then
        print_error "Reframe is not running on port 3000"
        echo ""
        echo "Please start Reframe first:"
        echo "  cargo build --release"
        echo "  RUST_LOG=info cargo run --release"
        echo ""
        exit 1
    fi
    
    local health=$(curl -s http://localhost:3000/health)
    local status=$(echo "$health" | jq -r '.status')
    local forward_engine=$(echo "$health" | jq -r '.engines.forward')
    local reverse_engine=$(echo "$health" | jq -r '.engines.reverse')
    
    print_success "Reframe is running"
    print_metric "Status" "$status" ""
    print_metric "Forward Engine" "$forward_engine" ""
    print_metric "Reverse Engine" "$reverse_engine" ""
    
    # Extract thread count
    local thread_count=$(echo "$forward_engine" | grep -o '[0-9]* workers' | cut -d' ' -f1)
    echo ""
    print_info "Detected configuration: $thread_count worker threads per engine"
}

# Run performance test and parse results
run_test() {
    local test_name="$1"
    local test_type="$2"
    local concurrency="$3"
    local requests="$4"
    local save_path="$5"
    
    echo ""
    echo -e "${MAGENTA}Running: $test_name${NC}"
    
    if [[ "$test_type" == "baseline" ]]; then
        python3 test/performance_test.py --baseline --save "$save_path" >/dev/null 2>&1
    else
        python3 test/performance_test.py --test load -c "$concurrency" -n "$requests" --save "$save_path" >/dev/null 2>&1
    fi
    
    # Parse results from JSON
    local json=$(cat "$save_path")
    
    # Extract metrics
    local throughput=$(echo "$json" | jq -r '.throughput_rps')
    local error_rate=$(echo "$json" | jq -r '.error_rate_percent')
    local latency_p50=$(echo "$json" | jq -r '.latency_p50_ms')
    local latency_p95=$(echo "$json" | jq -r '.latency_p95_ms')
    local latency_p99=$(echo "$json" | jq -r '.latency_p99_ms')
    local latency_min=$(echo "$json" | jq -r '.latency_min_ms')
    local latency_max=$(echo "$json" | jq -r '.latency_max_ms')
    local latency_mean=$(echo "$json" | jq -r '.latency_mean_ms')
    local latency_stddev=$(echo "$json" | jq -r '.latency_stdev_ms')
    local duration=$(echo "$json" | jq -r '.duration_seconds')
    local total_requests=$(echo "$json" | jq -r '.total_requests')
    local successful=$(echo "$json" | jq -r '.successful_requests')
    
    # Display results with color coding
    echo ""
    print_metric "Duration" "$duration" "seconds"
    print_metric "Total Requests" "$total_requests" ""
    print_metric "Successful" "$successful" ""
    
    # Throughput with color coding
    print_metric "Throughput" "$(printf '%.2f' $throughput)" "req/s" $GOOD_THROUGHPUT $WARN_THROUGHPUT
    
    # Error rate with color coding (inverse - lower is better)
    if (( $(echo "$error_rate == 0" | bc -l) )); then
        echo -e "${WHITE}%-25s${NC}${GREEN}${BOLD}0.00%%${NC}" "Error Rate:"
    else
        echo -e "${WHITE}%-25s${NC}${RED}${BOLD}${error_rate}%%${NC}" "Error Rate:"
    fi
    
    echo ""
    echo -e "${WHITE}Latency Distribution:${NC}"
    print_metric "  Min" "$(printf '%.2f' $latency_min)" "ms"
    print_metric "  P50 (median)" "$(printf '%.2f' $latency_p50)" "ms"
    print_metric "  P95" "$(printf '%.2f' $latency_p95)" "ms"
    print_metric "  P99" "$(printf '%.2f' $latency_p99)" "ms" $GOOD_LATENCY_P99 $WARN_LATENCY_P99
    print_metric "  Max" "$(printf '%.2f' $latency_max)" "ms"
    print_metric "  Mean" "$(printf '%.2f' $latency_mean)" "ms"
    print_metric "  StdDev" "$(printf '%.2f' $latency_stddev)" "ms"
    
    # Calculate and display percentile ratios
    if (( $(echo "$latency_p50 > 0" | bc -l) )); then
        local p99_p50_ratio=$(echo "scale=2; $latency_p99 / $latency_p50" | bc)
        local max_p99_ratio=$(echo "scale=2; $latency_max / $latency_p99" | bc)
        echo ""
        print_metric "  P99/P50 ratio" "$p99_p50_ratio" "(tail latency factor)"
        print_metric "  Max/P99 ratio" "$max_p99_ratio" "(outlier factor)"
    fi
}

# Generate performance summary
generate_summary() {
    local results_dir="$1"
    
    print_header "PERFORMANCE SUMMARY & ANALYSIS"
    
    # Load all test results
    local single=$(cat "$results_dir/single_thread.json" 2>/dev/null || echo "{}")
    local low=$(cat "$results_dir/load_c10.json" 2>/dev/null || echo "{}")
    local medium=$(cat "$results_dir/load_c50.json" 2>/dev/null || echo "{}")
    local high=$(cat "$results_dir/load_c100.json" 2>/dev/null || echo "{}")
    
    print_subheader "Throughput Scaling"
    
    if [[ "$single" != "{}" ]]; then
        local single_throughput=$(echo "$single" | jq -r '.throughput_rps')
        print_metric "Single-threaded" "$(printf '%.2f' $single_throughput)" "req/s"
    fi
    
    if [[ "$low" != "{}" ]]; then
        local low_throughput=$(echo "$low" | jq -r '.throughput_rps')
        local low_scaling=$(echo "scale=2; $low_throughput / $single_throughput" | bc)
        print_metric "Low concurrency (c=10)" "$(printf '%.2f' $low_throughput)" "req/s (${low_scaling}x scaling)"
    fi
    
    if [[ "$medium" != "{}" ]]; then
        local medium_throughput=$(echo "$medium" | jq -r '.throughput_rps')
        local medium_scaling=$(echo "scale=2; $medium_throughput / $single_throughput" | bc)
        print_metric "Medium concurrency (c=50)" "$(printf '%.2f' $medium_throughput)" "req/s (${medium_scaling}x scaling)"
    fi
    
    if [[ "$high" != "{}" ]]; then
        local high_throughput=$(echo "$high" | jq -r '.throughput_rps')
        local high_scaling=$(echo "scale=2; $high_throughput / $single_throughput" | bc)
        print_metric "High concurrency (c=100)" "$(printf '%.2f' $high_throughput)" "req/s (${high_scaling}x scaling)"
    fi
    
    print_subheader "Latency Under Load"
    
    echo -e "\n${WHITE}P99 Latency Progression:${NC}"
    [[ "$single" != "{}" ]] && print_metric "  Single-threaded" "$(echo "$single" | jq -r '.latency_p99_ms' | xargs printf '%.2f')" "ms"
    [[ "$low" != "{}" ]] && print_metric "  Low concurrency" "$(echo "$low" | jq -r '.latency_p99_ms' | xargs printf '%.2f')" "ms"
    [[ "$medium" != "{}" ]] && print_metric "  Medium concurrency" "$(echo "$medium" | jq -r '.latency_p99_ms' | xargs printf '%.2f')" "ms"
    [[ "$high" != "{}" ]] && print_metric "  High concurrency" "$(echo "$high" | jq -r '.latency_p99_ms' | xargs printf '%.2f')" "ms"
    
    # Calculate efficiency metrics
    print_subheader "Efficiency Analysis"
    
    if [[ "$high" != "{}" && "$single" != "{}" ]]; then
        local max_scaling=$(echo "scale=2; $high_throughput / $single_throughput" | bc)
        local efficiency=$(echo "scale=2; ($max_scaling / 10) * 100" | bc)  # Assuming 10 threads
        
        print_metric "Maximum scaling achieved" "${max_scaling}x" ""
        print_metric "Scaling efficiency" "${efficiency}%" "(of theoretical maximum)"
        
        # Determine bottleneck
        echo ""
        echo -e "${WHITE}Likely Bottleneck Analysis:${NC}"
        if (( $(echo "$efficiency < 50" | bc -l) )); then
            print_info "Low scaling efficiency suggests contention or synchronization issues"
        elif (( $(echo "$efficiency < 80" | bc -l) )); then
            print_info "Moderate scaling efficiency - room for optimization"
        else
            print_success "Good scaling efficiency - well-parallelized workload"
        fi
    fi
}

# Generate recommendations
generate_recommendations() {
    local results_dir="$1"
    
    print_header "OPTIMIZATION RECOMMENDATIONS"
    
    local high=$(cat "$results_dir/load_c100.json" 2>/dev/null || echo "{}")
    
    if [[ "$high" != "{}" ]]; then
        local high_p99=$(echo "$high" | jq -r '.latency_p99_ms')
        local high_throughput=$(echo "$high" | jq -r '.throughput_rps')
        
        echo -e "${WHITE}Based on the test results:${NC}\n"
        
        # Throughput recommendations
        if (( $(echo "$high_throughput < $WARN_THROUGHPUT" | bc -l) )); then
            print_info "❶ Throughput is below optimal levels. Consider:"
            echo "     • Increasing thread pool size"
            echo "     • Optimizing workflow processing"
            echo "     • Implementing request batching"
        elif (( $(echo "$high_throughput < $GOOD_THROUGHPUT" | bc -l) )); then
            print_info "❶ Throughput is moderate. Potential improvements:"
            echo "     • Fine-tune thread pool configuration"
            echo "     • Profile and optimize hot paths"
        else
            print_success "❶ Throughput is at good levels"
        fi
        
        echo ""
        
        # Latency recommendations
        if (( $(echo "$high_p99 > $WARN_LATENCY_P99" | bc -l) )); then
            print_info "❷ P99 latency is high. Consider:"
            echo "     • Implementing request prioritization"
            echo "     • Adding circuit breakers"
            echo "     • Optimizing slow operations"
        elif (( $(echo "$high_p99 > $GOOD_LATENCY_P99" | bc -l) )); then
            print_info "❷ P99 latency could be improved:"
            echo "     • Review garbage collection settings"
            echo "     • Consider connection pooling improvements"
        else
            print_success "❷ P99 latency is within acceptable range"
        fi
        
        echo ""
        
        # General recommendations
        print_info "❸ General optimization strategies:"
        echo "     • Enable release build optimizations (--release flag)"
        echo "     • Use CPU profiling to identify bottlenecks"
        echo "     • Monitor memory allocation patterns"
        echo "     • Consider async I/O optimizations"
    fi
}

# Main execution
main() {
    # Clear screen for better visibility
    clear
    
    print_header "REFRAME PERFORMANCE TESTING SUITE"
    echo -e "${WHITE}Comprehensive performance analysis and benchmarking${NC}"
    echo -e "${WHITE}$(date '+%Y-%m-%d %H:%M:%S %Z')${NC}"
    
    # Check requirements
    check_requirements
    
    # Get system information
    get_system_info
    
    # Check Reframe status
    check_reframe
    
    # Create results directory
    local timestamp=$(date +%Y%m%d_%H%M%S)
    local results_dir="test/results/baseline_${timestamp}"
    mkdir -p "$results_dir"
    
    print_info "Results will be saved to: $results_dir"
    
    # Run test suite
    print_header "PERFORMANCE TEST EXECUTION"
    
    # Test 1: Single-threaded baseline
    print_subheader "Test 1: Single-threaded Baseline"
    run_test "Establishing baseline performance" "baseline" 1 100 "$results_dir/single_thread.json"
    
    # Test 2: Low concurrency
    print_subheader "Test 2: Low Concurrency (c=10)"
    run_test "Testing with 10 concurrent connections" "load" 10 10000 "$results_dir/load_c10.json"
    
    # Test 3: Medium concurrency
    print_subheader "Test 3: Medium Concurrency (c=50)"
    run_test "Testing with 50 concurrent connections" "load" 50 10000 "$results_dir/load_c50.json"
    
    # Test 4: High concurrency
    print_subheader "Test 4: High Concurrency (c=100)"
    run_test "Testing with 100 concurrent connections" "load" 100 25000 "$results_dir/load_c100.json"
    
    # Generate summary and analysis
    generate_summary "$results_dir"
    
    # Generate recommendations
    generate_recommendations "$results_dir"
    
    # Final summary
    print_header "TEST COMPLETION"
    
    print_success "All performance tests completed successfully!"
    echo ""
    print_info "Results saved to: ${BOLD}$results_dir${NC}"
    echo ""
    echo -e "${WHITE}Additional commands:${NC}"
    echo -e "  ${CYAN}# View detailed JSON results${NC}"
    echo -e "  cat $results_dir/*.json | jq ."
    echo ""
    echo -e "  ${CYAN}# Generate HTML report${NC}"
    echo -e "  python3 test/performance_test.py --html-report"
    echo ""
    echo -e "  ${CYAN}# Compare with previous results${NC}"
    echo -e "  python3 test/compare_results.py $results_dir test/results/baseline_PREVIOUS"
    echo ""
    
    # Create a summary file
    cat > "$results_dir/SUMMARY.md" << EOF
# Performance Test Summary
**Date:** $(date '+%Y-%m-%d %H:%M:%S %Z')
**System:** $(uname -s) $(uname -r)
**CPU:** $(sysctl -n machdep.cpu.brand_string 2>/dev/null || lscpu | grep "Model name" | sed 's/Model name:[ ]*//')

## Results Overview
- Single-threaded: $(cat "$results_dir/single_thread.json" | jq -r '.throughput_rps' | xargs printf '%.2f') req/s
- Low concurrency: $(cat "$results_dir/load_c10.json" | jq -r '.throughput_rps' | xargs printf '%.2f') req/s
- Medium concurrency: $(cat "$results_dir/load_c50.json" | jq -r '.throughput_rps' | xargs printf '%.2f') req/s
- High concurrency: $(cat "$results_dir/load_c100.json" | jq -r '.throughput_rps' | xargs printf '%.2f') req/s

## Key Metrics
- Best P99 latency: $(cat "$results_dir/single_thread.json" | jq -r '.latency_p99_ms' | xargs printf '%.2f') ms
- Peak throughput: $(cat "$results_dir"/*.json | jq -r '.throughput_rps' | sort -rn | head -1 | xargs printf '%.2f') req/s
- Error rate: $(cat "$results_dir"/*.json | jq -r '.error_rate_percent' | sort -rn | head -1)%
EOF
    
    print_success "Summary file created: $results_dir/SUMMARY.md"
}

# Run the main function
main "$@"