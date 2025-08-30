# Reframe v3.0 Workflows - SR2025 Compliant

This directory contains the SR2025-compliant workflow definitions for Reframe's bidirectional SWIFT MT ↔ ISO 20022 transformation engine.

## Overview

Workflows are JSON-based transformation rules processed by the dataflow-rs engine. Each workflow defines a specific step in the transformation pipeline, enabling complete transparency and auditability of the transformation logic.

## Directory Structure

```
workflows/
├── forward/                 # MT → ISO 20022 transformations
│   ├── index.json          # Workflow registry and loading order
│   ├── parse-mt.json       # Common MT parser for all message types
│   ├── combine-xml.json    # Final XML assembly and formatting
│   └── [MT-Type]/          # Message-specific workflows
│       ├── bah-mapping.json      # Business Application Header v3 mapping
│       ├── document-mapping.json # Document body transformation
│       ├── precondition.json     # Pre-transformation validations
│       └── postcondition.json    # Post-transformation processing
│
└── reverse/                 # ISO 20022 → MT transformations
    ├── index.json          # Workflow registry and loading order
    ├── parse-mx.json       # Common MX parser for all message types
    └── [MX-Type]/          # Message-specific workflows
        ├── 01-variant-detection.json  # Detect message variant
        ├── 02-preconditions.json      # Validation rules
        ├── 03-field-mapping.json      # Core field transformations
        ├── 04-block-assembly.json     # MT block construction
        └── 05-mt-formatting.json      # Final MT formatting
```

## SR2025 Compliance Features

### Business Application Header v3
- Enhanced party identification with LEI support
- Improved service level codes (G001, G002, G003, G004)
- Extended priority options (HIGH, NORM, URGT)
- Mandatory UETR tracking

### Enhanced Data Quality
- Structured remittance information support
- Creditor reference information
- Document adjustment details
- Improved address structures with country subdivision

### New Message Types Supported
- **Cash Management**: camt.105, camt.106, camt.107, camt.108, camt.109
- **Payments**: Extended pacs and pain variants
- **Status Reports**: Enhanced camt.052, camt.053, camt.054
- **Administrative**: admi.024 for system notifications

## Supported Message Transformations

### Forward Transformations (MT → MX)

| MT Type | ISO 20022 Type | Description | SR2025 Status |
|---------|---------------|-------------|---------------|
| MT101 | pain.001 | Request for Transfer | ✅ Updated |
| MT103 | pacs.008 | Customer Credit Transfer | ✅ Updated |
| MT103REJT | pacs.002 | Payment Status (Rejection) | ✅ Updated |
| MT103RETN | pacs.004 | Payment Return | ✅ Updated |
| MT192 | pacs.002/camt.056 | Request for Cancellation | ✅ Updated |
| MT196 | camt.029/camt.056 | Cancellation Response | ✅ Updated |
| MT200 | pacs.009 | Financial Institution Transfer | ✅ Updated |
| MT202 | pacs.009 | General Financial Institution Transfer | ✅ Updated |
| MT202COVER | pacs.009 | Cover Payment | ✅ Updated |
| MT202REJT | pacs.002 | Transfer Rejection | ✅ Updated |
| MT202RETN | pacs.004 | Transfer Return | ✅ Updated |
| MT205 | pacs.009 | Financial Institution Transfer Execution | ✅ Updated |
| MT205COVER | pacs.009 | Cover Payment Execution | ✅ Updated |
| MT292 | camt.056 | Request for Cancellation | ✅ Updated |
| MT296 | camt.029 | Cancellation Status | ✅ Updated |
| MT900 | camt.054 | Confirmation of Debit | ✅ Updated |
| MT910 | camt.054 | Confirmation of Credit | ✅ Updated |
| MT940 | camt.053 | Customer Statement | ✅ Updated |
| MT942 | camt.052 | Interim Transaction Report | ✅ Updated |

### Reverse Transformations (MX → MT)

| ISO 20022 Type | MT Type | Description | SR2025 Status |
|---------------|---------|-------------|---------------|
| pacs.008 | MT103 | Customer Credit Transfer | ✅ Updated |
| pacs.009 | MT202/205 | Financial Institution Transfer | ✅ Updated |
| pacs.002 | MT103REJT/MT202REJT | Payment Status Report | ✅ Updated |
| pacs.003 | MT200 | Direct Debit | ✅ New |
| pacs.004 | MT103RETN/MT202RETN | Payment Return | ✅ Updated |
| camt.029 | MT296 | Resolution of Investigation | ✅ Updated |
| camt.052 | MT942 | Bank to Customer Account Report | ✅ Updated |
| camt.053 | MT940 | Bank to Customer Statement | ✅ Updated |
| camt.054 | MT900/910 | Bank to Customer Debit/Credit | ✅ Updated |
| camt.056 | MT192/292 | Payment Cancellation Request | ✅ Updated |
| camt.057 | MT210 | Notice to Receive | ✅ New |
| camt.058 | MT111 | Request for Stop Payment | ✅ New |
| camt.105 | MT196 | Billing Report | ✅ New |
| camt.106 | MT196 | Investigation Response | ✅ New |
| camt.107 | n/a | Non-deliverable Information | ✅ New |
| camt.108 | n/a | Identification Verification Request | ✅ New |
| camt.109 | n/a | Identification Verification Report | ✅ New |
| admi.024 | MT099 | System Event Notification | ✅ New |

## Workflow Components

### 1. Parser Workflows
- **parse-mt.json**: Parses SWIFT MT message blocks (1-5) into structured data
- **parse-mx.json**: Parses ISO 20022 XML into JSON structure

### 2. Mapping Workflows
- **bah-mapping.json**: Maps header information to Business Application Header v3
- **document-mapping.json**: Core business data transformation
- **field-mapping.json**: Individual field-level transformations

### 3. Validation Workflows
- **precondition.json**: Pre-transformation business rule validation
- **postcondition.json**: Post-transformation compliance checks
- **variant-detection.json**: Identifies message variants and routing

### 4. Assembly Workflows
- **combine-xml.json**: Assembles final ISO 20022 XML document
- **block-assembly.json**: Constructs SWIFT MT blocks
- **mt-formatting.json**: Applies MT-specific formatting rules

## Workflow Execution Flow

### Forward Flow (MT → MX)
```
1. parse-mt.json         → Parse incoming MT message
2. precondition.json     → Validate business rules
3. bah-mapping.json      → Create BAH v3 header
4. document-mapping.json → Transform business data
5. postcondition.json    → Apply final validations
6. combine-xml.json      → Generate ISO 20022 XML
```

### Reverse Flow (MX → MT)
```
1. parse-mx.json              → Parse incoming XML
2. 01-variant-detection.json  → Identify MT variant
3. 02-preconditions.json      → Validate transformation rules
4. 03-field-mapping.json      → Map MX fields to MT
5. 04-block-assembly.json     → Construct MT blocks
6. 05-mt-formatting.json      → Apply MT formatting
```

## JSONLogic Operations

Workflows use JSONLogic for conditional processing. Common operations include:

### Variable Access
```json
{"var": "SwiftMT.message_type"}
{"var": "Document.FIToFICstmrCdtTrf.CdtTrfTxInf.0.Amt.InstdAmt.#text"}
```

### Conditional Logic
```json
{"if": [
    {"==": [{"var": "message_type"}, "103"]},
    "MT103",
    "OTHER"
]}
```

### String Operations
```json
{"substr": [{"var": "field_20"}, 0, 16]}
{"cat": ["MT", {"var": "type"}]}
```

### Array Operations
```json
{"map": [
    {"var": "transactions"},
    {"cat": [{"var": "id"}, "-", {"var": "amount"}]}
]}
```

## Configuration Management

### Hot Reload
Workflows can be reloaded without service restart:
```bash
curl -X POST http://localhost:3000/admin/reload-workflows
```

### Validation
Validate workflow syntax before deployment:
```bash
python3 tools/validate_workflows.py
```

### Testing
Test specific workflow transformations:
```bash
python3 test/test_workflow.py MT103 forward
```

## Best Practices

### 1. Workflow Design
- Keep workflows focused on single responsibility
- Use descriptive IDs and clear descriptions
- Document complex transformations with comments
- Version control all workflow changes

### 2. Performance
- Minimize nested conditions
- Use early returns for validation failures
- Cache frequently accessed data in variables
- Avoid unnecessary array iterations

### 3. SR2025 Compliance
- Always include UETR in payment messages
- Validate LEI format when present
- Ensure BAH v3 compliance for all messages
- Apply proper service level codes

### 4. Error Handling
- Provide clear error messages
- Include field references in validation errors
- Log transformation failures with context
- Implement graceful fallbacks where appropriate

## Development Tools

### JSON Semi-Beautifier
Format workflow files for optimal readability:
```bash
../tools/json_semi_beautifier.py --in-place forward/MT103/*.json
```

### Workflow Generator
Generate workflow templates for new message types:
```bash
python3 tools/generate_workflow.py MT104 pacs.008
```

### Diff Tool
Compare workflow changes:
```bash
python3 tools/workflow_diff.py v2.0/MT103 v3.0/MT103
```

## Troubleshooting

### Common Issues

1. **Workflow Loading Errors**
   - Check JSON syntax with `jq` or online validators
   - Verify all referenced workflows exist in index.json
   - Ensure unique workflow IDs

2. **Transformation Failures**
   - Enable debug logging: `RUST_LOG=debug`
   - Check precondition violations in logs
   - Verify field paths match actual message structure

3. **SR2025 Validation Errors**
   - Ensure UETR is properly formatted
   - Validate service level codes against SR2025 spec
   - Check BAH v3 required fields

### Debug Mode
Enable detailed workflow execution tracing:
```bash
RUST_LOG=debug,dataflow_rs=trace cargo run
```

## Contributing

When adding new workflows:
1. Follow existing naming conventions
2. Update index.json with new workflow entries
3. Add comprehensive test scenarios
4. Document SR2025 specific changes
5. Run validation suite before committing

## References

- [SWIFT SR2025 Standards](https://www.swift.com/standards/release-guide/sr2025)
- [ISO 20022 Message Definitions](https://www.iso20022.org/catalogue-messages)
- [JSONLogic Documentation](https://jsonlogic.com/)
- [Dataflow-rs Documentation](https://github.com/GoPlasmatic/dataflow-rs)
