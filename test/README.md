# Reframe Test Suite

This directory contains the test suite for the Reframe SWIFT MT ↔ ISO 20022 transformation service.

## Overview

The test suite uses **dynamic sample generation** powered by the swift-mt-message v3 library's scenario-based system. This approach:
- Eliminates the need for static sample files
- Provides more realistic and varied test data
- Uses datafake-rs for dynamic value generation
- Supports multiple scenarios per message type

## Test Script

### `test_scenarios.py` - Comprehensive Scenario Testing
Tests MT message generation and transformation to ISO 20022 for all available scenarios.

```bash
# Test all scenarios for a message type
python test_scenarios.py --message-type MT103

# Test specific scenarios
python test_scenarios.py --message-type MT103 --scenarios standard high_value

# Use custom server URL
python test_scenarios.py --message-type MT103 --base-url http://localhost:8080

# Export results to JSON file
python test_scenarios.py --message-type MT103 --export
```

**Command-line options:**
- `--message-type`, `-m`: Message type to test (e.g., MT103, MT202, MT292)
- `--scenarios`, `-s`: Specific scenarios to test (otherwise uses all from index.json)
- `--base-url`, `-u`: Base URL of the transformation service (default: http://localhost:3000)
- `--export`, `-e`: Export results to JSON file in test/logs/

## Scenario Discovery

The test script automatically discovers available scenarios from `test_scenarios/[message_type]/index.json` files. If no index.json is found, it falls back to common scenario names like standard, high_value, remittance_enhanced, etc.

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
├── test_scenarios.py      # Main test script for scenario testing
└── logs/                  # Test results and logs (created automatically)
    └── scenario_test_results_*.json  # Timestamped test results
```

## Requirements

The test script requires Python 3.6+ with the following standard library modules:
- `json`
- `requests`
- `argparse`
- `pathlib`
- `datetime`
- `time`
- `collections`
- `re`

No additional dependencies are required.

## Running Tests

1. **Start the Reframe server:**
   ```bash
   cd ..
   cargo run
   ```

2. **In another terminal, run tests:**
   ```bash
   cd test
   
   # Test all scenarios for MT103
   python test_scenarios.py --message-type MT103
   
   # Test specific scenarios
   python test_scenarios.py --message-type MT103 --scenarios standard high_value
   
   # Test MT292 (or any other message type)
   python test_scenarios.py --message-type MT292
   
   # Export results
   python test_scenarios.py --message-type MT103 --export
   ```

## Understanding Test Results

### Test Output
The script displays real-time progress for each scenario:
```
Testing MT103 with 5 scenarios...
================================================================================

[1/5] Testing scenario: standard
  Generation: ✅
  Transformation: ✅
  Validation: ✅
  Document Type: pacs.008
  Business Service: swift.cbprplus.01
```

### Success Indicators
- ✅ Generation: MT message was successfully generated
- ✅ Transformation: MT to MX transformation succeeded
- ✅ Validation: MX output is valid and contains expected elements

### Summary Statistics
After all tests, a summary is displayed:
```
================================================================================
TEST SUMMARY
================================================================================
Total scenarios tested: 5
Generation success: 5/5 (100.0%)
Transformation success: 5/5 (100.0%)
Validation success: 5/5 (100.0%)

By Document Type:
  pacs.002: 1
  pacs.004: 1
  pacs.008: 3

By Method:
  cover: 1
  normal: 3
  rejection: 1
```

### Export Results
Use `--export` to save detailed results to `test/logs/scenario_test_results_[timestamp].json`

## Adding New Test Scenarios

To test new scenarios:

1. Check available scenarios in the swift-mt-message library:
   ```bash
   ls test_scenarios/[message_type]/
   ```

2. Use the specific scenario name with the command-line option:
   ```bash
   python test_scenarios.py --message-type MT103 --scenarios your_scenario
   ```

3. Scenarios are defined in JSON files within the swift-mt-message library and use datafake-rs for dynamic value generation.

## Troubleshooting

### Common Issues

1. **"No index.json found" message**
   - This is normal - the script will use default scenario names
   - Check that test_scenarios directory exists in the swift-mt-message library

2. **Server connection errors**
   - Verify Reframe server is running on http://localhost:3000
   - Use `--base-url` flag for different server location

3. **Generation failures**
   - Check if the message type is supported
   - Verify the scenario name is valid for that message type
   - Some scenarios may require specific swift-mt-message library versions

4. **Transformation failures**
   - Check server logs for detailed error messages
   - Ensure workflows are properly configured for the message type
   - Validation errors will be shown in the test output

### Exit Codes
- Exit code 0: Success (≥95% validation success rate)
- Exit code 1: Failure (<95% validation success rate)

## Notes

- The test suite uses the swift-mt-message v3 library's scenario-based generation system
- Each test run generates fresh, realistic MT messages using datafake-rs
- Results can be exported to JSON for further analysis or CI/CD integration
- The 0.1 second delay between tests prevents overwhelming the service