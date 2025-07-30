# Reframe Test Suite

This directory contains the test suite for the Reframe SWIFT MT ↔ ISO 20022 transformation service.

## Overview

The test suite now uses **dynamic sample generation** powered by the swift-mt-message v3 library's scenario-based system. This approach:
- Eliminates the need for static sample files
- Provides more realistic and varied test data
- Uses datafake-rs for dynamic value generation
- Supports multiple scenarios per message type

## Test Scripts

### 1. `test_api.py` - API Transformation Tests
Tests individual transformations using dynamically generated samples.

```bash
# Run all default test scenarios
python test_api.py

# Test specific message type
python test_api.py --message-type MT103

# Test specific scenario
python test_api.py --scenario high_value

# Test specific combination
python test_api.py --message-type MT103 --scenario cbpr_stp_compliant
```

### 2. `test_round_trip.py` - Round-Trip Tests
Tests complete transformation cycles: Generate MT → Transform to MX → Transform back to MT → Compare

```bash
# Run default round-trip tests
python test_round_trip.py

# Test specific message type
python test_round_trip.py --message-type MT103

# Test with specific scenario
python test_round_trip.py --message-type MT103 --scenario high_value

# Run all message types
python test_round_trip.py --all

# Use configuration file
python test_round_trip.py --config round_trip_config.json
```

## Configuration

### `round_trip_config.json`
Defines test scenarios for round-trip testing:

```json
[
  {
    "test_name": "MT103_standard",
    "message_type": "MT103",
    "scenario": "standard"
  },
  {
    "test_name": "MT103_high_value",
    "message_type": "MT103",
    "scenario": "high_value"
  }
]
```

## Available Scenarios

Scenarios are defined in the swift-mt-message library at `test_scenarios/[message_type]/`:

### MT103 Scenarios
- `standard` - Default customer credit transfer
- `high_value` - High-value payment with enhanced details
- `cbpr_stp_compliant` - Cross-Border Payments Regulation compliant
- `remittance_enhanced` - Enhanced remittance information
- `regulatory_compliant` - Regulatory compliance focused
- Many more CBPR scenarios...

### MT202 Scenarios
- `standard` - Default financial institution transfer
- `cbpr_cov_standard` - CBPR cover payment
- `fi_to_fi_transparency` - FI to FI with transparency

### Other Message Types
Most other message types have at least a `standard` scenario, with some having specialized scenarios.

## Directory Structure

```
test/
├── README.md              # This file
├── test_api.py           # API transformation tests
├── test_round_trip.py    # Round-trip transformation tests
├── round_trip_config.json # Test configuration
├── requirements.txt      # Python dependencies
└── logs/                 # Test results and debug logs
    ├── test_summary_latest.json
    ├── test_details_latest.log
    ├── debug/            # Individual debug files per test
    └── round_trip/       # Round-trip test logs
        ├── summary_latest.json
        ├── details_latest.log
        └── debug/        # Round-trip debug files
```

## Requirements

Install Python dependencies:
```bash
pip install -r requirements.txt
```

## Running Tests

1. **Start the Reframe server:**
   ```bash
   cd ..
   cargo run
   ```

2. **In another terminal, run tests:**
   ```bash
   cd test
   python test_api.py
   python test_round_trip.py
   ```

## Understanding Test Results

### Success Indicators
- ✓ Green checkmarks indicate passed tests
- All stages (generate, transform, compare) should pass
- Round-trip tests verify message integrity through full transformation cycle

### Failure Analysis
- ✗ Red X marks indicate failures
- Check `logs/` directory for detailed error information
- Debug files in `logs/debug/` contain transformation details

### Log Files
- `test_summary_latest.json` - High-level test results
- `test_details_latest.log` - Detailed execution logs
- `debug/*.json` - Individual test debug information

## Adding New Test Scenarios

To test new scenarios:

1. Check available scenarios in the swift-mt-message library:
   ```bash
   ls ../test_scenarios/[message_type]/
   ```

2. Add to configuration or use command-line:
   ```bash
   python test_round_trip.py --message-type MT103 --scenario your_scenario
   ```

## Troubleshooting

### Common Issues

1. **"No test scenarios found" error**
   - Ensure the `test_scenarios` symlink exists in the project root
   - Check that swift-mt-message test scenarios are available

2. **Server connection errors**
   - Verify Reframe server is running on http://localhost:3000
   - Use `--url` flag for different server location

3. **Transformation failures**
   - Check server logs for detailed error messages
   - Review debug files in `logs/debug/` for transformation details
   - Ensure workflows are properly configured for the message type

## Migration from Static Files

This test suite previously used static sample files in `data/`. The new approach:
- No longer requires maintaining static `.txt` and `.xml` files
- Generates fresh samples for each test run
- Provides more comprehensive test coverage through scenarios
- Reduces repository size and maintenance burden