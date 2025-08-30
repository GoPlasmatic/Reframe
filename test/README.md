# Reframe Test Suite

Comprehensive testing framework for the Reframe SWIFT MT ↔ ISO 20022 transformation service, including functional validation, scenario testing, and performance benchmarking.

## 📋 Table of Contents

- [Overview](#overview)
- [Test Components](#test-components)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Functional Testing](#functional-testing)
- [Performance Testing](#performance-testing)
- [Test Scripts Reference](#test-scripts-reference)
- [CI/CD Integration](#cicd-integration)
- [Troubleshooting](#troubleshooting)

## Overview

The Reframe test suite provides comprehensive testing capabilities across three key areas:

1. **Functional Testing** - Validates message transformations, generation, and validation
2. **Scenario Testing** - Tests real-world transformation scenarios with various message types
3. **Performance Testing** - Measures throughput, latency, and resource utilization to validate vertical scaling improvements

## Test Components

### 🧪 Functional Test Scripts

| Script | Purpose | Key Features |
|--------|---------|--------------|
| `test_scenarios.py` | End-to-end transformation testing | 8-step validation flow, roundtrip testing |
| `generate_sample.py` | Sample message generation | Support for MT and MX messages with scenarios |
| `validate_sample.py` | Message validation testing | Combined generation and validation workflow |

### 📊 Performance Test Scripts

| Script | Purpose | Key Features |
|--------|---------|--------------|
| `performance_test.py` | Comprehensive performance testing | Load, stress, spike, endurance tests |
| `run_baseline_test.sh` | Full baseline test suite | Automated baseline metrics collection |
| `quick_baseline_test.sh` | Quick performance check | Fast validation of key metrics |

## Installation

### Prerequisites

- Python 3.6+ 
- Rust and Cargo (for running Reframe)
- Unix-like environment (macOS/Linux)

### Python Dependencies

```bash
# Required dependencies
pip3 install requests tabulate

# Optional performance monitoring dependencies
pip3 install psutil numpy  # For enhanced metrics

# Optional for Apache Bench testing
# macOS
brew install ab
# Ubuntu/Debian
apt-get install apache2-utils
```

## Quick Start

### 1. Start Reframe Server

```bash
# Build and run in release mode for accurate performance testing
cargo build --release
RUST_LOG=info cargo run --release
```

### 2. Run Basic Tests

```bash
# Functional test - validate MT103 transformation
python3 test/test_scenarios.py -m MT103

# Generate sample message
python3 test/generate_sample.py MT103 -s standard

# Quick performance check
./test/quick_baseline_test.sh
```

## Functional Testing

### test_scenarios.py - Transformation Testing

Implements an 8-step validation flow for comprehensive transformation testing:

1. **List scenarios** - Discovers applicable scenarios
2. **Generate sample** - Creates test messages
3. **Validate source** - Validates generated message
4. **Transform** - Performs transformation
5. **Extract result** - Extracts transformed data
6. **Validate transformed** - Validates result
7. **Reverse transform** - Tests roundtrip
8. **Compare roundtrip** - Verifies consistency

#### Usage Examples

```bash
# List all available message types
python3 test_scenarios.py --list-types

# Test specific message type
python3 test_scenarios.py -m MT103

# Test with specific scenarios
python3 test_scenarios.py -m MT103 -s standard high_value

# Test with multiple samples per scenario
python3 test_scenarios.py -m pacs.008 --sample-count 5

# Enable debug output
python3 test_scenarios.py -m MT202 --debug

# Export results to JSON
python3 test_scenarios.py -m camt.054 --export
```

#### Command-line Options

| Option | Short | Description |
|--------|-------|-------------|
| `--message-type` | `-m` | Message type to test (e.g., MT103, pacs.008) |
| `--scenario` | `-s` | Specific scenario(s) to test |
| `--sample-count` | `-c` | Number of samples per scenario (default: 1) |
| `--debug` | `-d` | Enable debug output |
| `--export` | `-e` | Export results to JSON file |
| `--base-url` | `-u` | Base URL of service (default: http://localhost:3000) |
| `--list-types` | `-l` | List all available message types |
| `--list-scenarios` | | List scenarios for a message type |

### generate_sample.py - Sample Generation

Generates sample SWIFT MT or ISO 20022 messages using the Reframe API.

#### Usage Examples

```bash
# Generate MT103 with standard scenario
python3 generate_sample.py MT103 -s standard

# Generate pacs.008 with pretty XML formatting
python3 generate_sample.py pacs.008 -s cbpr_standard -p

# Save generated message to file
python3 generate_sample.py camt.052 -o sample.xml

# Generate with debug output
python3 generate_sample.py MT202 -s correspondent -d
```

#### Command-line Options

| Option | Short | Description |
|--------|-------|-------------|
| `message_type` | | Message type to generate (positional) |
| `--scenario` | `-s` | Scenario to use for generation |
| `--host` | `-H` | API host URL (default: http://localhost:3000) |
| `--debug` | `-d` | Enable debug output |
| `--validation` | `-v` | Enable validation |
| `--output` | `-o` | Output file (default: stdout) |
| `--pretty` | `-p` | Pretty print XML output |

### validate_sample.py - Validation Testing

Combines generation and validation in a single workflow.

#### Usage Examples

```bash
# Generate and validate MT101
python3 validate_sample.py MT101 -s single_payment

# Validate with business rules
python3 validate_sample.py pacs.008 -s cbpr_standard -b

# Verbose mode with generated message display
python3 validate_sample.py MT103 -s standard -v

# Full validation with all options
python3 validate_sample.py camt.052 -s cbpr -v -b -f
```

#### Command-line Options

| Option | Short | Description |
|--------|-------|-------------|
| `message_type` | | Message type to validate (positional) |
| `--scenario` | `-s` | Scenario to use |
| `--host` | `-H` | API host URL |
| `--debug` | `-d` | Enable debug output |
| `--business-validation` | `-b` | Enable business rules |
| `--no-canonical` | `-nc` | Disable canonical format |
| `--fail-fast` | `-f` | Stop on first error |
| `--json` | `-j` | Output raw JSON response |
| `--verbose` | `-v` | Show generated message |

## Performance Testing

### performance_test.py - Comprehensive Performance Testing

Advanced performance testing framework designed to measure and validate vertical scaling improvements as documented in `scaling.md`.

#### Key Metrics Measured

- **Throughput**: Requests per second (RPS)
- **Latency**: P50, P95, P99 percentiles
- **CPU Usage**: Reframe process CPU utilization
- **Memory Usage**: RSS (Resident Set Size)
- **Thread Count**: Number of active threads
- **Error Rates**: Failed request percentage

#### Test Types

##### 1. Baseline Test
Single-threaded sequential test to establish baseline performance.

```bash
python3 test/performance_test.py --baseline
```

##### 2. Load Test
Fixed load with configurable concurrency.

```bash
# 100 concurrent connections, 1000 total requests
python3 test/performance_test.py --test load -c 100 -n 1000
```

##### 3. Stress Test
Gradually increasing load to find breaking point.

```bash
# Increase from 20 to 200 connections in steps
python3 test/performance_test.py --test stress --max-concurrency 200
```

##### 4. Spike Test
Sudden burst of traffic to test resilience.

```bash
# 500 concurrent connections for 10 seconds
python3 test/performance_test.py --test spike -c 500
```

##### 5. Endurance Test
Sustained load over extended period.

```bash
# 50 concurrent connections for 5 minutes
python3 test/performance_test.py --test endurance -c 50 --duration 5
```

##### 6. Apache Bench Integration
Industry-standard HTTP benchmarking.

```bash
# Requires ab installation
python3 test/performance_test.py --test ab -c 100 -n 10000
```

#### Command-line Options

| Option | Description | Default |
|--------|-------------|---------|
| `--url` | Base URL of Reframe service | http://localhost:3000 |
| `--test` | Test type: baseline, load, stress, spike, endurance, ab, all | - |
| `--baseline` | Run baseline test and save results | - |
| `-c, --concurrency` | Number of concurrent connections | 10 |
| `-n, --requests` | Total number of requests | 1000 |
| `--max-concurrency` | Maximum concurrency for stress test | 200 |
| `--duration` | Duration in minutes for endurance test | 5 |
| `--save` | Save results to JSON file | - |
| `--compare` | Compare two result files | - |
| `--html-report` | Generate HTML report with charts | - |
| `--debug` | Enable debug output | - |

#### Performance Comparison

```bash
# Compare baseline vs optimized results
python3 test/performance_test.py --compare baseline.json optimized.json

# Generate visual HTML report
python3 test/performance_test.py --test stress --html-report
```

### Automated Performance Test Scripts

#### run_baseline_test.sh
Comprehensive baseline test suite that runs multiple test scenarios and saves results.

```bash
./test/run_baseline_test.sh
```

Runs:
1. Single-threaded baseline (100 requests)
2. Low concurrency (10 connections, 500 requests)
3. Medium concurrency (50 connections, 1000 requests)
4. High concurrency (100 connections, 1000 requests)
5. Stress test (up to 100 connections)

#### quick_baseline_test.sh
Fast performance check for quick validation.

```bash
./test/quick_baseline_test.sh
```

Runs:
1. Single-threaded baseline (100 requests)
2. Low concurrency (10 connections, 100 requests)
3. Medium concurrency (50 connections, 200 requests)

### Expected Performance Metrics

#### Current Architecture (Mutex-based)
- **Throughput**: ~50-100 req/s
- **CPU Usage**: 10-15% (single core on multi-core machine)
- **P99 Latency**: 500ms-2s under load
- **Concurrency**: 1 request at a time

#### After Optimization (Target)
- **Throughput**: ~2,000-5,000 req/s (20-50x improvement)
- **CPU Usage**: 70-90% (all cores utilized)
- **P99 Latency**: 50-200ms under load
- **Concurrency**: 64+ simultaneous requests

### Success Criteria Validation

The performance tests validate the following criteria from `scaling.md`:

- [ ] 10x throughput improvement
- [ ] >70% CPU utilization under load
- [ ] P99 latency <200ms at 80% capacity
- [ ] Zero request drops under normal load
- [ ] Graceful degradation under overload

## Test Result Formats

### Functional Test Results

```
+----------------+----------+----------+-------------+-------------+-------------+--------------+
| Message Type   | Scenario | Sample   | Generator   | Validator   | Transform   | Round Trip   |
+================+==========+==========+=============+=============+=============+==============+
| MT103          | standard | 1        | ✅          | ✅          | ✅          | ✅           |
+----------------+----------+----------+-------------+-------------+-------------+--------------+
```

### Performance Test Results

```
============================================================
Performance Test Results: Load Test (c=50)
============================================================
| Metric                 | Value         |
|========================|===============|
| Throughput             | 17.96 req/s   |
| Latency P99            | 1526.52 ms    |
| Reframe CPU Usage      | 13.5%         |
| Reframe Memory (RSS)   | 705.33 MB     |
| Reframe Threads        | 16            |
| Concurrent Connections | 50            |
```

### JSON Export Format

```json
{
  "test_name": "Load Test (c=50)",
  "timestamp": "2025-08-30T17:30:00",
  "duration_seconds": 28.15,
  "total_requests": 200,
  "successful_requests": 200,
  "throughput_rps": 7.10,
  "latency_p99_ms": 7766.74,
  "cpu_usage_percent": 13.5,
  "memory_usage_mb": 705.33,
  "error_rate_percent": 0.0
}
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Reframe Tests
on: [push, pull_request]

jobs:
  functional-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Build Reframe
        run: cargo build --release
      - name: Start Reframe
        run: |
          RUST_LOG=info cargo run --release &
          sleep 5
      - name: Run functional tests
        run: |
          python3 test/test_scenarios.py -m MT103 --export
      - name: Upload results
        uses: actions/upload-artifact@v2
        with:
          name: test-results
          path: test/logs/*.json

  performance-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Build Reframe
        run: cargo build --release
      - name: Run performance baseline
        run: |
          RUST_LOG=info cargo run --release &
          sleep 5
          python3 test/performance_test.py --baseline --save baseline.json
      - name: Check performance criteria
        run: |
          python3 -c "
          import json
          with open('baseline.json') as f:
              data = json.load(f)
              assert data['throughput_rps'] > 50, 'Throughput too low'
              assert data['latency_p99_ms'] < 2000, 'Latency too high'
          "
```

### Exit Codes

- **0**: Success (tests passed)
- **1**: Failure (tests failed or criteria not met)

## Directory Structure

```
test/
├── README.md                    # This file
├── test_scenarios.py            # Functional transformation testing
├── generate_sample.py           # Sample message generator
├── validate_sample.py           # Validation testing
├── performance_test.py          # Performance testing framework
├── run_baseline_test.sh         # Full baseline test suite
├── quick_baseline_test.sh       # Quick performance check
├── results/                     # Test results (created automatically)
│   └── baseline_*/              # Timestamped test results
└── logs/                        # Functional test logs
    └── test_results_*.json      # Timestamped test results
```

## Troubleshooting

### Common Issues

#### 1. Module Import Errors
```bash
# Install required Python packages
pip3 install requests tabulate

# Optional for enhanced monitoring
pip3 install psutil numpy
```

#### 2. Connection Refused
- Verify Reframe is running: `curl http://localhost:3000/health`
- Check correct port: `lsof -i :3000`
- Use `--url` flag for different server location

#### 3. Low Performance Metrics
- Ensure using release build: `cargo build --release`
- Check system resources: `top` or `htop`
- Verify no other processes consuming CPU

#### 4. CPU/Memory Showing 0
- Install psutil for accurate metrics: `pip3 install psutil`
- Script has fallback methods but psutil is recommended

#### 5. Apache Bench Not Found
```bash
# macOS
brew install ab

# Ubuntu/Debian
apt-get install apache2-utils
```

### Debug Mode

Enable debug output for detailed troubleshooting:

```bash
# Functional tests
python3 test_scenarios.py -m MT103 --debug

# Performance tests
python3 performance_test.py --test load --debug
```

### Performance Monitoring

Monitor Reframe during tests:

```bash
# Watch process stats
watch -n 1 'ps aux | grep -E "(cargo|reframe)" | grep -v grep'

# Monitor port connections
watch -n 1 'netstat -an | grep :3000 | grep ESTABLISHED | wc -l'

# System resources
top -o cpu  # macOS
htop        # Linux
```

## Best Practices

1. **Always use release builds** for performance testing
2. **Run baseline tests** before making changes
3. **Save test results** for comparison and tracking
4. **Monitor system resources** during tests
5. **Use appropriate test types** for different scenarios
6. **Export results** for CI/CD and reporting

## Contributing

When adding new tests:

1. Follow existing naming conventions
2. Add documentation to this README
3. Include appropriate error handling
4. Support both debug and normal modes
5. Provide example usage
6. Update CI/CD configurations if needed

## Support

For issues or questions:
- Check the [Troubleshooting](#troubleshooting) section
- Review debug output with `--debug` flag
- Check Reframe logs with `RUST_LOG=debug`
- Report issues at https://github.com/anthropics/reframe/issues