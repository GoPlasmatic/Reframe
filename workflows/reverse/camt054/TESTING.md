# camt.054 Testing Guide

## Quick Start

### Prerequisites
```bash
# Ensure server is running
curl -s http://localhost:3000/health | jq

# If not running, start it
RUST_LOG=info cargo run
```

### Test All Scenarios
```bash
# Using the convenience script (recommended)
./test_camt054.sh all

# Using Python directly
python3 test/test_scenarios.py -m camt.054
```

### Test Individual Scenarios

| Scenario | MT Type | Command | Debug Command |
|----------|---------|---------|---------------|
| Customer Notification | MT103 | `./test_camt054.sh mt103` | `./test_camt054.sh mt103-debug` |
| Bank Notification | MT202 | `./test_camt054.sh mt202` | `./test_camt054.sh mt202-debug` |
| Debit Confirmation | MT900 | `./test_camt054.sh mt900` | `./test_camt054.sh mt900-debug` |
| Credit Confirmation | MT910 | `./test_camt054.sh mt910` | `./test_camt054.sh mt910-debug` |

## Current Test Results

### ✅ Working Features
- **Generation**: 100% success (4/4 scenarios)
- **Validation**: 100% success (4/4 scenarios)
- All scenarios generate valid camt.054 XML
- All generated messages pass ISO 20022 validation

### ❌ Known Issues
- **Transformation**: 0% success (0/4 scenarios)
  - **Root Cause**: ParseMX function doesn't recognize camt.054 message type
  - **Error**: "Unknown message type"
  - **Solution**: Requires Rust codebase update to add camt.054 support

## Detailed Testing Commands

### Python Test Script Options
```bash
# Basic test
python3 test/test_scenarios.py -m camt.054

# Debug mode (verbose output)
python3 test/test_scenarios.py -m camt.054 -d

# Test specific scenario
python3 test/test_scenarios.py -m camt.054 -s customer_notification

# Test with multiple samples
python3 test/test_scenarios.py -m camt.054 --sample-count 10

# Export results to JSON
python3 test/test_scenarios.py -m camt.054 --export > results.json
```

### Validation Testing
```bash
# Validate individual scenarios
python3 test/validate_sample.py camt.054 -s customer_notification -d
python3 test/validate_sample.py camt.054 -s bank_notification -d
python3 test/validate_sample.py camt.054 -s debit_confirmation -d
python3 test/validate_sample.py camt.054 -s credit_confirmation -d
```

### Direct API Testing
```bash
# Generate MT103 Advice sample
curl -X POST http://localhost:3000/generate/sample \
  -H "Content-Type: application/json" \
  -d '{"message_type": "camt.054", "config": {"scenario": "customer_notification"}}' | jq

# Generate MT202 Advice sample
curl -X POST http://localhost:3000/generate/sample \
  -H "Content-Type: application/json" \
  -d '{"message_type": "camt.054", "config": {"scenario": "bank_notification"}}' | jq

# Generate MT900 sample
curl -X POST http://localhost:3000/generate/sample \
  -H "Content-Type: application/json" \
  -d '{"message_type": "camt.054", "config": {"scenario": "debit_confirmation"}}' | jq

# Generate MT910 sample
curl -X POST http://localhost:3000/generate/sample \
  -H "Content-Type: application/json" \
  -d '{"message_type": "camt.054", "config": {"scenario": "credit_confirmation"}}' | jq
```

## Workflow Testing

### After Making Workflow Changes
```bash
# 1. Reload workflows
curl -X POST http://localhost:3000/admin/reload-workflows
# or
./test_camt054.sh reload

# 2. Test the specific scenario affected
./test_camt054.sh mt103-debug  # For MT103 changes
./test_camt054.sh mt202-debug  # For MT202 changes
./test_camt054.sh mt900-debug  # For MT900 changes
./test_camt054.sh mt910-debug  # For MT910 changes

# 3. Run full test suite
./test_camt054.sh all
```

## Performance Testing
```bash
# Test with high volume (100 samples per scenario)
python3 test/test_scenarios.py -m camt.054 --sample-count 100

# Measure transformation time
time python3 test/test_scenarios.py -m camt.054 -s customer_notification
```

## Troubleshooting

### Common Issues and Solutions

| Issue | Solution |
|-------|----------|
| Server not running | `RUST_LOG=info cargo run` |
| Port 3000 in use | `lsof -i :3000 \| grep LISTEN \| awk '{print $2}' \| xargs kill -9` |
| Workflows not updating | `curl -X POST http://localhost:3000/admin/reload-workflows` |
| Generation fails | Check scenario JSON syntax in `/scenarios/reverse/` |
| Validation fails | Verify XML structure matches ISO 20022 schema |
| Transformation fails | Known issue - ParseMX doesn't support camt.054 yet |

### Debug Logging
```bash
# Start server with debug logging
RUST_LOG=debug cargo run

# Start with trace logging (very verbose)
RUST_LOG=trace cargo run

# Filter logs by component
RUST_LOG=reframe=debug,dataflow_rs=trace cargo run
```

## Test Coverage Summary

| Component | Status | Coverage |
|-----------|--------|----------|
| Variant Detection | ✅ | 100% - All 4 variants detected correctly |
| Scenario Generation | ✅ | 100% - All scenarios generate valid XML |
| XML Validation | ✅ | 100% - All generated XML passes validation |
| Field Mappings | ⚠️ | 75% - Some complex fields need enhancement |
| Postconditions | ⚠️ | 60% - Missing business logic transformations |
| Transformation | ❌ | 0% - Blocked by ParseMX limitation |

## Next Steps

1. **Immediate**: Fix ParseMX to recognize camt.054
2. **Short-term**: Implement missing field mappings (see gaps.md)
3. **Medium-term**: Add postcondition business logic
4. **Long-term**: Achieve 100% CBPR+ compliance