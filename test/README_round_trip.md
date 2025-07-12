# Reframe Round-Trip Test Script

This script tests the complete bidirectional transformation flow of the Reframe service:

1. **Generate MT Sample** - Use the `/generate/mt-sample` API to create an MT message
2. **Transform MT → MX** - Convert the MT message to ISO 20022 format
3. **Transform MX → MT** - Convert the MX message back to MT format  
4. **Compare Messages** - Verify that the original and final MT messages match

## Prerequisites

1. **Reframe Server Running**: Make sure the Reframe server is running on `http://localhost:3000` (or specify different URL with `--url`)
2. **Python Dependencies**: Install required packages:
   ```bash
   pip install -r requirements.txt
   ```

## Usage

### Quick Test (3 common message types)
```bash
python3 test_round_trip.py
```

### Test Specific Message Type
```bash
python3 test_round_trip.py --message-type MT103
python3 test_round_trip.py --message-type MT202 --include-optional
```

### Test All Supported Message Types
```bash
python3 test_round_trip.py --all
```

### Test with Configuration File
```bash
python3 test_round_trip.py --config round_trip_config.json
```

### Test with Different Server URL
```bash
python3 test_round_trip.py --url http://localhost:8080 --message-type MT103
```

## Configuration File Format

Create a JSON file with test configurations:

```json
[
  {
    "test_name": "MT103_basic",
    "message_type": "MT103",
    "config": {},
    "options": {
      "validation": true,
      "include_debug": true
    }
  },
  {
    "test_name": "MT103_with_optional",
    "message_type": "MT103",
    "config": {
      "include_optional": true,
      "scenario": "standard",
      "field_configs": {}
    },
    "options": {
      "validation": true,
      "include_debug": true
    }
  }
]
```

### Configuration Parameters

- **test_name**: Unique identifier for the test
- **message_type**: One of the supported MT message types (MT101, MT103, etc.)
- **config**: Configuration passed to the generate API:
  - `include_optional`: Include optional fields in generated message
  - `scenario`: Generation scenario (e.g., "standard", "cover")
  - `field_configs`: Field-specific configurations
- **options**: API options:
  - `validation`: Enable message validation
  - `include_debug`: Include debug information in responses

## Supported Message Types

The script supports all 24 message types that Reframe supports:

- **Customer Payments**: MT101, MT103, MT104, MT107
- **Bank-to-Bank**: MT202, MT205, MT210
- **Confirmations**: MT900, MT910, MT920
- **Account Statements**: MT940, MT941, MT942, MT950
- **Treasury**: MT110, MT111, MT112
- **Investigations**: MT192, MT196, MT199, MT292, MT296, MT299
- **System**: MT935

## Output

### Console Output
- Colored progress indicators for each test
- Stage-by-stage results (Generate → MT→MX → MX→MT → Compare)
- Summary with pass/fail counts

### Log Files
All logs are saved to `test/data/logs/round_trip/`:

- **summary_latest.json**: Complete test results in JSON format
- **details_latest.log**: Detailed execution log with timestamps
- **debug/**: Individual debug files for each test stage

### Debug Files
For each test, separate debug files are created:
- `{test_name}_generate.json`: Debug info from MT generation
- `{test_name}_mt_to_mx.json`: Debug info from MT→MX transformation
- `{test_name}_mx_to_mt.json`: Debug info from MX→MT transformation
- `{test_name}_comparison.json`: Message comparison details (if messages differ)

## Understanding Results

### Success Criteria
A test passes if:
1. MT message is successfully generated
2. MT→MX transformation succeeds
3. MX→MT transformation succeeds
4. Original and final MT messages match exactly (after normalization)

### Message Normalization
Before comparison, messages are normalized by:
- Removing trailing whitespace from lines
- Removing empty lines at the end
- Normalizing line endings to `\n`

### Common Failure Scenarios
- **Generation fails**: Invalid configuration or unsupported message type
- **Transformation fails**: Message format issues or workflow errors
- **Messages differ**: Round-trip transformation is not perfect (data loss/modification)

## Examples

### Example 1: Test MT103 with basic configuration
```bash
python3 test_round_trip.py --message-type MT103
```

### Example 2: Test multiple message types with optional fields
```bash
python3 test_round_trip.py --config custom_config.json
```

### Example 3: Test all message types (comprehensive test)
```bash
python3 test_round_trip.py --all
```

## Troubleshooting

1. **Server not responding**: Make sure Reframe is running with `cargo run`
2. **Import errors**: Install dependencies with `pip install -r requirements.txt`
3. **Permission denied**: Make script executable with `chmod +x test_round_trip.py`
4. **API errors**: Check server logs and debug files for detailed error information

## Integration with CI/CD

The script returns appropriate exit codes:
- **0**: All tests passed
- **1**: Some tests failed or server unavailable

Example usage in CI:
```bash
python3 test_round_trip.py --all
if [ $? -eq 0 ]; then
    echo "All round-trip tests passed!"
else
    echo "Some tests failed. Check logs for details."
    exit 1
fi
```