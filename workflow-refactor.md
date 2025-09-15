# Workflow Refactoring Guide

## Overview
This guide provides a systematic approach to refactor SWIFT MT/ISO 20022 transformation workflows for improved efficiency, reduced redundancy, and better maintainability.

## 📋 Pre-Refactoring Analysis Checklist

### 1. Identify Redundancies
- [ ] Check for multiple tasks that could be combined
- [ ] Look for temp_data fields that are only used once
- [ ] Find duplicate logic across tasks
- [ ] Identify unnecessary intermediate data storage
- [ ] Review validation rules that check the same conditions

### 2. Review Current Structure
- [ ] Count total number of tasks
- [ ] Map data flow between tasks
- [ ] Identify task dependencies
- [ ] Note conditional logic patterns
- [ ] Document which tasks can be merged

## 🔧 Refactoring Steps

### Step 1: Consolidate Related Tasks
**Pattern**: If multiple tasks are setting related fields, combine them into a single task.

**Example from MT103 bah-mapping.json**:
```json
// Before: 3 separate tasks
- prepare_bah_context (extracts sender/receiver)
- construct_business_application_header
- map_related_message_metadata

// After: 2 consolidated tasks
- construct_business_application_header (includes all BAH fields and related metadata)
- generate_header_xml (separate for conditional source_format)
```

**Key Changes**:
- Move sender/receiver extraction directly into BAH construction
- Embed related message metadata in the same mapping
- Eliminate intermediate temp_data storage

### Step 2: Eliminate Unnecessary temp_data
**Pattern**: If temp_data is only used in one place, calculate it inline.

**Examples of removed temp_data fields**:
- `PossibleDuplicateIndicator` → calculated directly in BAH
- Sender/Receiver BICs → extracted inline where needed
- Settlement method intermediate calculations → computed directly

**Before**:
```json
{
  "path": "temp_data.PossibleDuplicateIndicator",
  "logic": {"if": [{"var": "data.SwiftMT.trailer.PDE"}, true, false]}
}
// Later referenced as:
"PssblDplct": {"var": "temp_data.PossibleDuplicateIndicator"}
```

**After**:
```json
"PssblDplct": {"if": [{"var": "data.SwiftMT.trailer.PDE"}, true, false]}
```

### Step 3: Simplify Conditional Logic
**Pattern**: Remove redundant existence checks when the conditional already handles null values.

**Before**:
```json
{"and": [
  {"var": "data.SwiftMT.fields.72.information"},
  {"var": "data.SwiftMT.fields.72.information.0"},
  {"starts_with": [{"var": "data.SwiftMT.fields.72.information.0"}, "/REJT/"]}
]}
```

**After**:
```json
{"and": [
  {"var": "data.SwiftMT.fields.72.information.0"},
  {"starts_with": [{"var": "data.SwiftMT.fields.72.information.0"}, "/REJT/"]}
]}
```

### Step 4: Group Field Mappings
**Pattern**: Map multiple related fields in a single task to reduce overhead.

**Example from MT103 document-mapping.json**:
```json
// Before: 13 separate tasks
// After: 7 consolidated tasks

1. initialize_context_and_settlement (combines 3 tasks)
2. construct_group_header
3. map_credit_transfer_transaction_core
4. map_parties_and_accounts (combines debtor/creditor mappings)
5. map_intermediary_and_creditor_agents (combines agent mappings)
6. map_instructions_charges_and_remittance (combines multiple field mappings)
7. generate_document_xml (conditional per method)
```

### Step 5: Consolidate Validation Rules
**Pattern**: Group related validations in single task to reduce execution overhead.

**Example from MT103 postcondition.json**:
```json
// Before: 7 validation tasks
// After: 4 consolidated tasks

1. validate_mandatory_and_structural_fields (combines 3 validations)
2. validate_settlement_and_charges (combines 2 validations)
3. validate_cbpr_compliance (combines 2 validations)
4. validate_stp_restrictions (conditional, kept separate)
```

**Note**: Validation tasks don't need `"path"` as the entire context is provided by default.

### Step 6: Handle Conditional Tasks Properly
**Pattern**: Use conditions for method-specific tasks, but keep separate tasks for different source_format values.

**Example**:
```json
{
  "id": "generate_header_xml",
  "condition": {"==": [{"var": "metadata.SwiftMT.method"}, "normal"]},
  "function": {
    "name": "PublishMX",
    "input": {
      "source_format": "MT103.Header"  // Must be explicit string
    }
  }
},
{
  "id": "generate_stp_header_xml",
  "condition": {"==": [{"var": "metadata.SwiftMT.method"}, "stp"]},
  "function": {
    "name": "PublishMX",
    "input": {
      "source_format": "MT103_STP.Header"  // Different format string
    }
  }
}
```

## 📝 Key Refactoring Patterns

### Pattern 1: Merge Initialization Tasks
```json
// Before: Multiple initialization tasks
Task 1: Initialize temp_data with BICs
Task 2: Initialize clearing system codes
Task 3: Determine settlement method

// After: Single consolidated initialization
Task 1: Initialize context and settlement (all initialization in one task)
```

### Pattern 2: Inline Simple Calculations
```json
// Before: Store in temp_data then reference
"temp_data.value": {"if": [...]},
"field": {"var": "temp_data.value"}

// After: Calculate directly
"field": {"if": [...]}
```

### Pattern 3: Field Presence Checks
```json
// Only include field if source exists (omit entirely if null)
{"if": [
  {"var": "source.field"},
  {"mapped": "value"},
  null  // This omits the field entirely from output
]}
```

### Pattern 4: Array vs Object Handling
```json
// Ensure correct structure for fields
"PmtId": {
  "InstrId": "...",     // These are object properties
  "EndToEndId": "...",  // NOT array elements
  "TxId": "...",
  "UETR": "..."
}
```

## ⚠️ Common Pitfalls to Avoid

### 1. Don't Break Conditional source_format
- Keep separate tasks for different source_format values
- Each task needs explicit source_format string
- Cannot use conditional logic within source_format parameter

### 2. Preserve Field Structure
- Ensure mapped fields maintain correct nesting
- Watch for array vs object serialization
- Be careful with merge operations that might create arrays

### 3. Handle null vs Omission Correctly
```json
// To omit field entirely:
{"if": [condition, value, null]}

// Wrong - creates field with null value:
{"if": [condition, value, {"FinInstnId": null}]}
```

### 4. Validation Task Specifics
- Don't add `"path"` to validation rules - entire context is provided
- The `preserve: true` is not needed as it's default behavior
- Keep validation logic focused and clear

### 5. Test Incrementally
- Test after each major change
- Verify all scenarios still pass
- Check for unexpected null values or missing fields

## ✅ Post-Refactoring Verification

### 1. Reload Workflows
**CRITICAL**: Always reload workflows after making changes:
```bash
# Reload workflows via API (application must be running)
curl -X POST http://localhost:3000/admin/reload-workflows

# Alternative: Restart the application
# Kill existing process and restart
lsof -i :3000 | grep LISTEN | awk '{print $2}' | xargs kill -9 2>/dev/null
RUST_LOG=info cargo run
```

### 2. Test with Scenarios Script
Run comprehensive tests using the test_scenarios.py script:

```bash
# List available scenarios for a message type
python3 test/test_scenarios.py -m MT103 --list-scenarios

# Test all scenarios for a message type
python3 test/test_scenarios.py -m MT103

# Test specific scenario by ID (e.g., high_value, standard, cbpr_plus, etc.)
python3 test/test_scenarios.py -m MT103 -s high_value
python3 test/test_scenarios.py -m MT103 -s standard

# Test with debug output to identify issues
python3 test/test_scenarios.py -m MT103 -d
python3 test/test_scenarios.py -m MT103 -s high_value -d

# Test all MT messages
python3 test/test_scenarios.py --all-mt

# Test all MX messages
python3 test/test_scenarios.py --all-mx

# Export results for analysis
python3 test/test_scenarios.py -m MT103 -e
```

### 3. Debug Workflow Issues
When tests fail, use debug mode to identify problems:

```bash
# Step 1: Run with debug flag to see detailed output
python3 test/test_scenarios.py -m MT103 -d

# Step 2: Check specific workflow execution with engine debug logging
RUST_LOG=debug cargo run
# Then run the test again

# Step 3: Trace dataflow execution for deep debugging
RUST_LOG=debug,dataflow_rs=trace cargo run
# Then run the test again

# Step 4: Test individual API endpoints
# Generate sample
curl -X POST http://localhost:3000/generate/sample \
  -H "Content-Type: application/json" \
  -d '{"message_type": "MT103", "config": {"scenario": "standard"}}' | jq

# Validate MT
curl -X POST http://localhost:3000/validate/mt \
  -H "Content-Type: application/json" \
  -d @test_mt103.json | jq

# Transform with debug
curl -X POST http://localhost:3000/transform/mt-to-mx \
  -H "Content-Type: application/json" \
  -d '{"message": "...", "options": {"debug": true}}' | jq
```

### 4. Common Issues and Fixes

#### Issue: Workflow not loading
```bash
# Check workflow syntax
python3 -m json.tool workflows/forward/MT103/bah-mapping.json

# Verify index.json includes the workflow
cat workflows/forward/index.json | jq
```

#### Issue: Field mapping errors
```bash
# Debug specific field mappings
# Add debug print in workflow:
{
  "id": "debug_field",
  "function": {
    "name": "Debug",
    "input": {"message": "Field value", "data": {"var": "path.to.field"}}
  }
}
```

#### Issue: Validation failures
```bash
# Check precondition/postcondition rules
# Temporarily disable specific validations to isolate issue
```

#### Issue: Transformation produces wrong output
```bash
# Compare before/after workflow output
# Save transformation results before refactoring
python3 test/test_scenarios.py -m MT103 -e
# Compare with results after refactoring
diff logs/test_results_*.json
```

### 5. Incremental Testing Strategy

#### Phase 1: Test After Each Change
```bash
# Make one workflow change
vim workflows/forward/MT103/bah-mapping.json

# Reload workflows
curl -X POST http://localhost:3000/admin/reload-workflows

# Test immediately with specific scenario
python3 test/test_scenarios.py -m MT103 -s high_value

# Test with debug if it fails
python3 test/test_scenarios.py -m MT103 -s high_value -d

# If it fails, revert and try smaller change
```

#### Phase 2: Test All Scenarios
```bash
# After refactoring a message type, test all its scenarios
python3 test/test_scenarios.py -m MT103 --list-scenarios
python3 test/test_scenarios.py -m MT103  # Tests all scenarios

# Check success rates in output
```

#### Phase 3: Regression Testing
```bash
# Test related message types that might be affected
# For MT103, also test MT103REJT, MT103RETN
python3 test/test_scenarios.py -m MT103REJT
python3 test/test_scenarios.py -m MT103RETN
```

### 6. Performance Validation
```bash
# Run performance benchmark before and after
python3 test/simple_benchmark.py

# Compare task execution times in debug logs
RUST_LOG=debug cargo run
# Look for "Task completed in X ms" messages
```

### 7. Validation Checklist
- [ ] All workflows reload without errors
- [ ] All scenarios pass for the refactored message type
- [ ] No regression in related message types
- [ ] Debug output shows expected data flow
- [ ] Performance metrics improved or unchanged
- [ ] No unexpected null values in output
- [ ] All conditional logic works correctly
- [ ] Field structure matches expected format

## 📊 Expected Improvements

| Metric | Target | MT103 Achievement |
|--------|--------|-------------------|
| Task Reduction | 30-50% fewer tasks | 44% reduction (16→9 tasks) |
| Code Lines | 20-30% reduction | ~25% reduction |
| Execution Time | 10-20% faster | Improved due to fewer tasks |
| Maintainability | Significantly improved | ✓ Achieved |

## 🔄 Workflow-Specific Considerations

### For BAH Mapping Workflows
- Consolidate sender/receiver extraction into main BAH construction
- Merge related message metadata into single task
- Inline duplicate detection logic
- Keep separate tasks for different source_format values

### For Document Mapping Workflows
- Group party mappings (Dbtr, DbtrAcct, Cdtr, CdtrAcct)
- Consolidate agent mappings (IntrmyAgt, CdtrAgt)
- Merge settlement method determination with initialization
- Combine instruction, charges, and remittance mappings

### For Validation Workflows (Precondition/Postcondition)
- Combine related validation rules by type
- Group mandatory field checks together
- Consolidate business rule validations
- Keep STP-specific validations conditional

### For Method Detection Workflows
- Simplify conditional checks
- Remove redundant field existence validations
- Keep logic clear and maintainable

## 🚀 Implementation Approach

1. **Start with Analysis**
   - Review current workflow structure
   - Identify consolidation opportunities
   - Plan the refactoring approach

2. **Refactor Incrementally**
   - Start with simple consolidations
   - Test after each change
   - Gradually merge more complex tasks

3. **Validate Thoroughly**
   - Run all test scenarios
   - Compare output with original
   - Ensure no data loss or corruption

4. **Document Changes**
   - Note what was consolidated
   - Explain complex logic
   - Update any related documentation

## 📝 Refactoring Checklist Template

```markdown
### Workflow: [Workflow Name]
- [ ] Analyzed current structure (__ tasks)
- [ ] Identified consolidation opportunities
- [ ] Merged initialization tasks
- [ ] Eliminated unnecessary temp_data
- [ ] Simplified conditional logic
- [ ] Grouped related field mappings
- [ ] Consolidated validation rules
- [ ] Tested all scenarios
- [ ] Verified output correctness
- [ ] Final task count: __ tasks
- [ ] Reduction achieved: __%
```

## Summary

The key to successful workflow refactoring is to:
1. **Consolidate** related operations
2. **Eliminate** unnecessary intermediate storage
3. **Simplify** conditional logic
4. **Maintain** functionality and correctness
5. **Test** thoroughly at each step

Following this guide will result in cleaner, more efficient workflows that are easier to maintain and understand while preserving all original functionality.