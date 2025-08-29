#!/bin/bash

# Test script for camt.054 scenarios
# Usage: ./test_camt054.sh [all|mt103|mt202|mt900|mt910|debug]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Base test command
BASE_CMD="python3 test/test_scenarios.py -m camt.054"

# Function to print colored messages
print_header() {
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}========================================${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${YELLOW}ℹ️  $1${NC}"
}

# Check if server is running
check_server() {
    if curl -s http://localhost:3000/health > /dev/null 2>&1; then
        print_success "Server is running"
        return 0
    else
        print_error "Server is not running"
        print_info "Starting server..."
        lsof -i :3000 | grep LISTEN | awk '{print $2}' | xargs kill -9 2>/dev/null || true
        RUST_LOG=info cargo run &
        sleep 10
        if curl -s http://localhost:3000/health > /dev/null 2>&1; then
            print_success "Server started successfully"
            return 0
        else
            print_error "Failed to start server"
            exit 1
        fi
    fi
}

# Test all scenarios
test_all() {
    print_header "Testing All camt.054 Scenarios"
    $BASE_CMD
    echo ""
    print_info "Run with debug flag for detailed output: ./test_camt054.sh debug"
}

# Test MT103 Advice
test_mt103() {
    print_header "Testing MT103 Advice (Customer Notification)"
    $BASE_CMD -s customer_notification $1
}

# Test MT202 Advice
test_mt202() {
    print_header "Testing MT202 Advice (Bank Notification)"
    $BASE_CMD -s bank_notification $1
}

# Test MT900
test_mt900() {
    print_header "Testing MT900 (Debit Confirmation)"
    $BASE_CMD -s debit_confirmation $1
}

# Test MT910
test_mt910() {
    print_header "Testing MT910 (Credit Confirmation)"
    $BASE_CMD -s credit_confirmation $1
}

# Test all with debug
test_debug() {
    print_header "Testing All camt.054 Scenarios (Debug Mode)"
    $BASE_CMD -d
}

# Test individual scenario with debug
test_scenario_debug() {
    print_header "Testing $1 (Debug Mode)"
    case $1 in
        mt103)
            $BASE_CMD -s customer_notification -d
            ;;
        mt202)
            $BASE_CMD -s bank_notification -d
            ;;
        mt900)
            $BASE_CMD -s debit_confirmation -d
            ;;
        mt910)
            $BASE_CMD -s credit_confirmation -d
            ;;
        *)
            print_error "Unknown scenario: $1"
            exit 1
            ;;
    esac
}

# Reload workflows
reload_workflows() {
    print_info "Reloading workflows..."
    if curl -X POST http://localhost:3000/admin/reload-workflows -s > /dev/null 2>&1; then
        print_success "Workflows reloaded successfully"
    else
        print_error "Failed to reload workflows"
    fi
}

# Show usage
usage() {
    echo "Usage: $0 [command] [options]"
    echo ""
    echo "Commands:"
    echo "  all       - Test all camt.054 scenarios"
    echo "  mt103     - Test MT103 Advice scenario"
    echo "  mt202     - Test MT202 Advice scenario"
    echo "  mt900     - Test MT900 scenario"
    echo "  mt910     - Test MT910 scenario"
    echo "  debug     - Test all scenarios with debug output"
    echo "  reload    - Reload workflows"
    echo "  help      - Show this help message"
    echo ""
    echo "Debug options:"
    echo "  mt103-debug  - Test MT103 with debug output"
    echo "  mt202-debug  - Test MT202 with debug output"
    echo "  mt900-debug  - Test MT900 with debug output"
    echo "  mt910-debug  - Test MT910 with debug output"
    echo ""
    echo "Examples:"
    echo "  $0 all           # Test all scenarios"
    echo "  $0 mt103         # Test MT103 scenario"
    echo "  $0 mt103-debug   # Test MT103 with debug output"
    echo "  $0 debug         # Test all with debug output"
}

# Main script
print_header "camt.054 Test Suite"

# Check server first
check_server

# Parse command line arguments
case "${1:-all}" in
    all)
        test_all
        ;;
    mt103)
        test_mt103
        ;;
    mt202)
        test_mt202
        ;;
    mt900)
        test_mt900
        ;;
    mt910)
        test_mt910
        ;;
    debug)
        test_debug
        ;;
    mt103-debug)
        test_scenario_debug mt103
        ;;
    mt202-debug)
        test_scenario_debug mt202
        ;;
    mt900-debug)
        test_scenario_debug mt900
        ;;
    mt910-debug)
        test_scenario_debug mt910
        ;;
    reload)
        reload_workflows
        ;;
    help|--help|-h)
        usage
        exit 0
        ;;
    *)
        print_error "Unknown command: $1"
        usage
        exit 1
        ;;
esac

# Print summary
echo ""
print_header "Test Summary"
print_info "Test completed. Check the output above for results."
print_info "For more detailed analysis, use debug mode: $0 debug"