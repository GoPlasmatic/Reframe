# Reframe Test Suite

This directory contains the test suite for the Reframe SWIFT MT ↔ ISO 20022 transformation service.

## Overview

The test suite provides comprehensive testing for both:
- **SWIFT MT Messages**: Traditional MT101-MT950 format messages
- **ISO 20022 MX Messages**: Modern XML-based pacs, pain, and camt messages

The test suite uses **dynamic sample generation** powered by:
- swift-mt-message v3 library for MT messages (scenario-based system)
- mx-message library for MX messages (scenario-based system)
- datafake-rs for dynamic value generation
- Unified API endpoint `/generate/sample` for both message types

## Requirements

```bash
# Install Python dependencies
pip install tabulate
```

The test script requires Python 3.6+ with the following modules:
- `json`, `requests`, `argparse`, `pathlib`, `datetime`, `time`, `collections`, `re` (standard library)
- `tabulate` (for table formatting)

## Test Script Features

### `test_scenarios.py` - Unified MT and MX Testing

The script provides comprehensive testing with the following features:

1. **Dynamic Message Type Discovery**: Automatically discovers available message types from scenario directories
2. **Scenario Loading**: Loads scenarios from index.json files in each message type folder
3. **Multiple Sample Generation**: Can generate multiple samples per scenario for thorough testing
4. **Table Output**: Displays results in a formatted table with status indicators
5. **Debug Mode**: Optional verbose output for troubleshooting
6. **Export Capability**: Export test results to JSON for CI/CD integration
7. **Round-trip Testing**: Tests bidirectional transformation capabilities

## Usage

### Basic Commands

```bash
# List all available message types
python test_scenarios.py --list-types

# List scenarios for a specific message type
python test_scenarios.py --list-scenarios --message-type MT103

# Test a specific message type with all scenarios
python test_scenarios.py --message-type MT103

# Test specific scenarios
python test_scenarios.py -m MT103 -s standard high_value

# Test with multiple samples per scenario
python test_scenarios.py -m pacs.008 --sample-count 3

# Enable debug output
python test_scenarios.py -m MT202 --debug

# Export results to JSON
python test_scenarios.py -m camt.054 --export
```

### Command-line Options

| Option | Short | Description |
|--------|-------|-------------|
| `--message-type` | `-m` | Message type to test (e.g., MT103, pacs.008) |
| `--scenario` | `-s` | Specific scenario(s) to test |
| `--sample-count` | `-c` | Number of samples per scenario (default: 1) |
| `--debug` | `-d` | Enable debug output |
| `--export` | `-e` | Export results to JSON file |
| `--base-url` | `-u` | Base URL of the service (default: http://localhost:3000) |
| `--list-types` | `-l` | List all available message types |
| `--list-scenarios` | | List scenarios for a message type |

## Understanding Test Results

### Result Table

The test results are displayed in a formatted table with the following columns:

| Column | Description |
|--------|-------------|
| Message Type | The message type being tested (e.g., MT103, pacs.008) |
| Scenario | The scenario name (truncated if too long) |
| Sample | Sample number (when multiple samples are generated) |
| Generator | ✅ Success / ❌ Failed - Message generation status |
| Validator | ✅ Success / ❌ Failed - Message validation status |
| Transform | ✅ Success / ❌ Failed / N/A - Transformation status |
| Round Trip | ✅ Success / ⚠️ Warning / ❌ Failed - Round-trip test status |
| Errors | ⚠️ if errors occurred (details in debug mode) |

### Status Indicators

- ✅ **Success**: Operation completed successfully
- ❌ **Failed**: Operation failed
- ⚠️ **Warning**: Partial success or known limitation
- **N/A**: Not applicable or not yet supported

### Example Output

```
Testing MT103 with 2 scenario(s), 1 sample(s) each...

+---------------+----------------------+--------+-----------+-----------+-----------+------------+--------+
| Message Type  | Scenario             | Sample | Generator | Validator | Transform | Round Trip | Errors |
+===============+======================+========+===========+===========+===========+============+========+
| MT103         | standard             |      1 | ✅        | ✅        | ✅        | ✅         |        |
+---------------+----------------------+--------+-----------+-----------+-----------+------------+--------+
| MT103         | high_value           |      1 | ✅        | ✅        | ✅        | ⚠️         |        |
+---------------+----------------------+--------+-----------+-----------+-----------+------------+--------+

================================================================================
TEST SUMMARY
================================================================================
Total tests: 2
Generation success: 2/2 (100.0%)
Validation success: 2/2 (100.0%)
Transformation success: 2/2 (100.0%)
Round trip success: 1/2 (50.0%)

Tests by Message Type:
  MT103: 2
```

## Message Type Discovery

The script automatically discovers available message types from:
- `scenarios/SwiftMTMessage/*/index.json` for MT messages
- `scenarios/MXMessage/*/index.json` for MX messages

Directory names are automatically converted to proper format:
- MT: `mt103` → `MT103`
- MX: `pacs008` → `pacs.008`

## Scenario Discovery

Scenarios are loaded from `index.json` files in each message type directory. The index.json format supports:

```json
{
  "scenarios": [
    {
      "file": "standard.json",
      "description": "Standard payment"
    },
    {
      "file": "high_value.json",
      "description": "High value payment"
    }
  ]
}
```

Or simple format:
```json
{
  "scenarios": ["standard.json", "high_value.json"]
}
```

## Directory Structure

```
test/
├── README.md              # This file
├── test_scenarios.py      # Main test script
└── logs/                  # Test results (created automatically)
    └── test_results_*.json  # Timestamped test results
```

## Running Tests

### 1. Start the Reframe Server

```bash
cd ..
cargo run
# or with logging
RUST_LOG=info cargo run
```

### 2. Run Tests

```bash
cd test

# Quick test of one message type
python test_scenarios.py -m MT103

# Test with specific scenarios
python test_scenarios.py -m MT103 -s standard high_value

# Test with multiple samples
python test_scenarios.py -m pacs.008 -c 5

# Full test with debug and export
python test_scenarios.py -m MT202 -d -e

# Discover available types
python test_scenarios.py --list-types

# List scenarios for a type
python test_scenarios.py --list-scenarios -m MT103
```

## CI/CD Integration

The script returns appropriate exit codes for CI/CD:
- Exit code 0: Success (≥95% validation success rate)
- Exit code 1: Failure (<95% validation success rate)

Example CI/CD usage:
```bash
# Run tests and check exit code
python test_scenarios.py -m MT103 -e
if [ $? -eq 0 ]; then
    echo "Tests passed"
else
    echo "Tests failed"
    exit 1
fi
```

## Export Format

Test results can be exported to JSON with the `--export` flag:

```json
{
  "timestamp": "2025-01-08T10:30:00",
  "base_url": "http://localhost:3000",
  "statistics": {
    "total": 10,
    "generation_success": 10,
    "transformation_success": 8,
    "validation_success": 10,
    "roundtrip_success": 7,
    "by_message_type": {
      "MT103": 10
    }
  },
  "results": [
    {
      "message_type": "MT103",
      "scenario": "standard",
      "sample": 1,
      "generation": "✅",
      "validation": "✅",
      "transformation": "✅",
      "roundtrip": "✅",
      "errors": []
    }
  ]
}
```

## Troubleshooting

### Common Issues

1. **ModuleNotFoundError: No module named 'tabulate'**
   ```bash
   pip install tabulate
   ```

2. **Connection refused errors**
   - Verify Reframe server is running on http://localhost:3000
   - Use `--base-url` flag for different server location

3. **No scenarios found**
   - Check that index.json exists in the message type directory
   - Verify scenario directory structure

4. **Generation failures**
   - Check server logs for detailed error messages
   - Verify message type is supported
   - Use `--debug` flag for verbose output

5. **Transformation N/A**
   - Some transformations (especially MX→MT) are still in development
   - This is expected behavior for certain message types

## Performance Considerations

- The script includes a 0.1 second delay between tests to avoid overwhelming the service
- For large-scale testing, consider using `--sample-count` with smaller values
- Export results for later analysis rather than running all tests interactively

## Notes

- Message types are dynamically discovered from scenario directories
- Scenarios are loaded from index.json files in each message type folder
- Results are displayed in a table format for easy readability
- The script supports both MT and MX message types through the unified API
- Round-trip testing validates bidirectional transformation capabilities