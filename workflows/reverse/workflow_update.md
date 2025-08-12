# Reverse Mapping Workflow Update Guide (ISO 20022 → MT)

This document outlines the process for updating reverse mapping workflows when new ISO 20022 or SWIFT specifications are released.

## Prerequisites
- Location to new ISO 20022 specification from SWIFT
- Location to existing mapping workflow in `workflows/reverse/[MESSAGE_TYPE]/`
- Understanding of CBPR+ reverse transformation rules and Translation Rules (TR)

## Workflow Update Process

### a. Pick an ISO 20022 Message Type for which the mapping is going to be updated

### b. Generate MX Message Sample

The unified sample generation API provides realistic test messages for getting the canonical JSON structure and testing the reverse transformation workflow.

```bash
curl -X POST http://localhost:3000/generate/sample \
  -H "Content-Type: application/json" \
  -d '{
    "message_type": "pacs.008",
    "config": {
        "scenario": "cbpr_business_payment"
    }
  }'
```

**API Status**: ✅ Working correctly - generates valid ISO 20022 XML messages for all supported scenarios.

Common scenarios: `standard`, `stp`, `high_value`, `cbpr_*`, `fx_conversion`, `regulatory_compliant`

### c. Validate and Get Canonical JSON Structure

**Note: The /validate/mx endpoint is now fully implemented and working.**

```bash
curl -X POST http://localhost:3000/validate/mx \
  -H "Content-Type: application/json" \
  -d '{
    "message": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><Document>...</Document>",
    "options": {
        "include_canonical_json": true,
        "include_business_validation": true
    }
  }'
```

**API Status**: ✅ Working correctly - validates MX messages and extracts canonical JSON structure.

**Important Notes about Canonical JSON Structure**:
- Top level contains: `header`, `document`, `message_type`
- Header preserves nested structure (e.g., `header.Fr.FIId.FinInstnId.BICFI`)
- Document contains message-specific elements at root level:
  - `document.GrpHdr` - Group header
  - `document.CdtTrfTxInf` - Credit transfer transaction info (for pacs.008)
- Amount fields use special notation:
  - `@Ccy` for currency attribute
  - `$value` for the numeric amount value

### d. Review Specification Reference
- Refer to the CBPR+ reverse transformation specification documentation
- Identify Translation Rules (TR001-TR020) that apply to the message type
- Review Preconditions (PREC001, PREC002) for validation requirements
- Check Postconditions (POSTC001-POSTC010) for output formatting rules

#### MX_To_MT Function Reference Documentation
- **Complete function reference**: `xxx-specification/reverse/MX_To_MT_Functions_Details.json`
- Contains detailed documentation for all 61 MX_To_MT transformation functions
- Each function entry includes:
  - **name**: Function identifier (e.g., MX_To_MTAnyBIC, MX_To_MTAgentGeneric)
  - **business_description**: What the function does and when to use it
  - **format**: Function signature and parameters
  - **input**: Detailed input parameter descriptions with types
  - **output**: Output format and structure
  - **preconditions**: Required conditions for function execution
  - **formal_description**: Complete algorithmic implementation details
- **Quick reference table**: `xxx-specification/reverse/MX_To_MT_Functions_Table.md`

### e. Update Mapping Workflow

#### 1. Adopt Modular Structure (Recommended Approach)

**Standard Modular Structure:**
```
workflows/reverse/[MESSAGE_TYPE]/
├── 02-preconditions.json          # PREC001-PREC002 validation
├── 03-headers-mapping.json        # SWIFT header construction
├── 04-mandatory-fields-mapping.json # Core MT fields (20, 23B, 32A)
├── 05-amount-fields-mapping.json  # Amount fields (33B, 36, 13C)
├── 06-charge-fields-mapping.json  # Charge fields (71A, 71F, 71G)
├── 07-party-fields-mapping.json   # Customer fields (50, 59)
├── 08-agent-fields-mapping.json   # Institution fields (52, 56, 57)
├── 09-remittance-fields-mapping.json # Field 70 mapping
├── 10-instruction-fields-mapping.json # Field 72 mapping
├── 11-postconditions.json         # POSTC validation and PublishMT
└── status.md                       # Implementation status tracking
```

#### 2. Key Module Patterns

**Preconditions Example:**
```json
{
    "id": "validate_preconditions",
    "function": {
        "name": "validate",
        "input": {
            "rules": [
                {
                    "path": "data.ISO20022_MX.document.CdtTrfTxInf.IntrBkSttlmAmt.@Ccy",
                    "logic": {
                        "!": {
                            "in": [
                                {"var": "data.ISO20022_MX.document.CdtTrfTxInf.IntrBkSttlmAmt.@Ccy"},
                                ["XAU", "XAG", "XPD", "XPT"]
                            ]
                        }
                    },
                    "message": "PREC001: T20054 - Commodities currencies not allowed"
                }
            ]
        }
    }
}
```

**Field Mapping Pattern:**
```json
// For numeric-only field names:
{
    "path": "data.SwiftMT.fields.#20",
    "logic": {
        "reference": {"var": "data.ISO20022_MX.document.CdtTrfTxInf.PmtId.InstrId"}
    }
}

// For alphanumeric field names:
{
    "path": "data.SwiftMT.fields.32A",
    "logic": {
        "cat": [
            {"substr": [{"var": "data.ISO20022_MX.document.CdtTrfTxInf.IntrBkSttlmDt"}, 2, 6]},
            {"var": "data.ISO20022_MX.document.CdtTrfTxInf.IntrBkSttlmAmt.@Ccy"},
            {"var": "data.ISO20022_MX.document.CdtTrfTxInf.IntrBkSttlmAmt.$value"}
        ]
    }
}
```

**Agent Field with MX_To_MTAgentGeneric:**
```json
{
    "path": "data.SwiftMT.fields.#52",
    "logic": {
        "function": {
            "name": "MX_To_MTAgentGeneric",
            "input": {
                "mx_agent": {"var": "data.ISO20022_MX.document.CdtTrfTxInf.DbtrAgt"},
                "supported_options": ["A", "D"],
                "preferred_option": "A"
            }
        }
    }
}
```

#### Commonly Used MX_To_MT Functions

Refer to `xxx-specification/reverse/MX_To_MT_Functions_Details.json` for complete documentation of these functions:

| Function | Usage | Common Fields |
|----------|-------|---------------|
| **MX_To_MTAgentGeneric** | Financial institution fields | 52, 53, 54, 56, 57, 58 |
| **MX_To_MTDebtorGeneric** | Ordering customer | 50 (all options) |
| **MX_To_MTCreditorGeneric** | Beneficiary customer | 59 (all options) |
| **MX_To_MTCurrencyAmount** | Amount fields | 32A, 33B |
| **MX_To_MTDate** | Date conversions | Value dates, creation dates |
| **MX_To_MTRemittanceInformation** | Remittance info | 70 |
| **MX_To_MT72FullField** | Instructions | 72 |
| **MX_To_MTBICFI** | BIC extraction | All option A fields |
| **MX_To_MTAccount** | Account identification | Account subfields |
| **MX_To_MTFATFIdentification** | FATF compliance | 50F subfield 1 |
| **MX_To_MTFATFNameAndAddress** | FATF name/address | 50F subfield 2 |

#### 3. Module Development Best Practices

- **Always use 02-11 numbering** for consistency
- **Use temp_data** for intermediate calculations
- **Map to specific sub-paths** to avoid overwriting
- **Group related fields** in the same module
- **Test each module** individually before integration

### f. Test Transformation

**⚠️ Current Status (2025-08-08)**: Transformation is failing due to workflow bugs. See Section 8 for known issues.

```bash
# Generate sample
sample_response=$(curl -X POST http://localhost:3000/generate/sample \
  -H "Content-Type: application/json" \
  -d '{"message_type": "pacs.008", "config": {"scenario": "standard"}}')

# Transform
curl -X POST http://localhost:3000/transform/mx-to-mt \
  -H "Content-Type: application/json" \
  -d "{\"message\": $(echo "$sample_response" | jq -r '.transformed_message' | jq -Rs .)}"
```

### g. Apply Fixes and Retest

```bash
# Hot reload workflows
curl -X POST http://localhost:3000/admin/reload-workflows

# Retest transformation
```

## 2. ISO 20022 Message Structure References

| Message Type | Description | Reference Path |
|--------------|-------------|----------------|
| pacs.002 | Payment Status Report | ../MXMessage/mx-message/src/messages/pacs002.rs |
| pacs.004 | Payment Return | ../MXMessage/mx-message/src/messages/pacs004.rs |
| pacs.008 | FI To FI Customer Credit Transfer | ../MXMessage/mx-message/src/messages/pacs008.rs |
| pacs.009 | Financial Institution Credit Transfer | ../MXMessage/mx-message/src/messages/pacs009.rs |
| camt.052 | Bank To Customer Account Report | ../MXMessage/mx-message/src/messages/camt052.rs |
| camt.053 | Bank To Customer Statement | ../MXMessage/mx-message/src/messages/camt053.rs |

## 3. Common Transformation Patterns

### Settlement Method Awareness
```json
{
    "path": "temp_data.use_clearing_codes",
    "logic": {
        "or": [
            {"==": [{"var": "data.ISO20022_MX.document.GrpHdr.SttlmInf.SttlmMtd"}, "INDA"]},
            {"==": [{"var": "data.ISO20022_MX.document.GrpHdr.SttlmInf.SttlmMtd"}, "INGA"]}
        ]
    }
}
```

### EU Validation Pattern (Field 33B)
```json
{
    "path": "data.SwiftMT.fields.33B",  // No # prefix - contains letter 'B'
    "logic": {
        "if": [
            {"and": [
                {"in": [{"substr": [{"var": "temp_data.Sender"}, 4, 2]}, 
                       ["AT", "BE", "BG", "HR", "CY", "CZ", "DK", "EE", "FI", "FR", 
                        "DE", "GR", "HU", "IE", "IT", "LV", "LT", "LU", "MT", "NL", 
                        "PL", "PT", "RO", "SK", "SI", "ES", "SE"]]},
                {"in": [{"substr": [{"var": "temp_data.Receiver"}, 4, 2]}, 
                       [/* same EU countries */]]}
            ]},
            {
                "currency": {"var": "data.ISO20022_MX.document.CdtTrfTxInf.InstdAmt.@Ccy"},
                "amount": {"var": "data.ISO20022_MX.document.CdtTrfTxInf.InstdAmt.$value"}
            },
            null
        ]
    }
}
```

## 4. Troubleshooting Common Issues

### JSONLogic Function Issues

| Invalid | Correct | Note |
|---------|---------|------|
| `index_of` | `starts_with` | Use for prefix checking |
| `matches` | Simple logic | No regex support |
| `count` | `length` | For arrays |
| `concat` | `cat` | String concatenation |

### Available Operators
- **Array**: `map`, `filter`, `reduce`, `all`, `some`, `none`, `merge`, `in`, `length`
- **String**: `cat`, `substr`, `starts_with`
- **Logic**: `if`, `and`, `or`, `not`, `==`, `!=`, `>`, `<`, `>=`, `<=`
- **Math**: `+`, `-`, `*`, `/`, `%`, `min`, `max`

### Amount Field Access
```json
// ❌ Wrong:
{"var": "data.ISO20022_MX.document.CdtTrfTxInf.IntrBkSttlmAmt"}

// ✅ Correct:
{
    "currency": {"var": "data.ISO20022_MX.document.CdtTrfTxInf.IntrBkSttlmAmt.@Ccy"},
    "amount": {"var": "data.ISO20022_MX.document.CdtTrfTxInf.IntrBkSttlmAmt.$value"}
}
```

## 5. Validation Requirements

### Preconditions (PREC)
- **PREC001**: Commodity currencies (XAU, XAG, XPD, XPT) not allowed
- **PREC002**: Only single transaction supported

### Translation Rules (TR)
- **TR001-TR005**: Header construction
- **TR006-TR010**: Field mappings
- **TR011-TR015**: Special cases (EU validation, time indication)
- **TR016-TR020**: Remittance & instructions

### Postconditions (POSTC)
- **POSTC001-005**: Character set validation
- **POSTC006-010**: Formatting rules

## 6. Known Issues and Fixes (Updated 2025-08-08)

### Critical Workflow Issues Found

#### Issue 1: Field Path Notation for Numeric Fields
**Important**: The dataflow engine requires special notation for purely numeric field names.

```json
// For purely numeric fields (interpreted as array indices without prefix):
"path": "data.SwiftMT.fields.#20"    // ✅ Correct - # prefix required
"path": "data.SwiftMT.fields.#21"    // ✅ Correct - # prefix required
"path": "data.SwiftMT.fields.#52"    // ✅ Correct - # prefix required
"path": "data.SwiftMT.fields.#72"    // ✅ Correct - # prefix required

// For alphanumeric fields (contain letters, no prefix needed):
"path": "data.SwiftMT.fields.13C"    // ✅ Correct - no # prefix
"path": "data.SwiftMT.fields.32A"    // ✅ Correct - no # prefix
"path": "data.SwiftMT.fields.52A"    // ✅ Correct - no # prefix
"path": "data.SwiftMT.fields.53B"    // ✅ Correct - no # prefix

// ❌ Wrong - will cause array index interpretation:
"path": "data.SwiftMT.fields.20"     // Field "20" interpreted as array[20]
"path": "data.SwiftMT.fields.72"     // Field "72" interpreted as array[72]
```

**Rule**: Use `#` prefix ONLY for purely numeric field names (e.g., 20, 21, 52, 72). Fields with letters (e.g., 13C, 32A, 52A) do NOT need the prefix.

#### Issue 2: Field Value Structure Mismatch
**Problem**: PublishMT expects different field structures than workflows create.

```json
// ❌ Current (wrong):
{"20": {"value": "INSTR123"}}

// ✅ Expected:
{"20": "INSTR123"}
```

#### Issue 3: MX Validation API Status
**Update**: The `/validate/mx` endpoint is fully implemented and working correctly.

### Testing Checklist

1. **Validate Generated Samples**
2. **Test Transformation End-to-End**
3. **Verify Canonical JSON Structure**
4. **Check Field Path Notation** - Ensure numeric-only fields use `#` prefix

## 7. Field Naming Convention Summary

### When to Use `#` Prefix

| Field Type | Example Fields | Path Notation | Reason |
|------------|---------------|---------------|--------|
| **Numeric Only** | 20, 21, 50, 52, 53, 54, 56, 57, 58, 59, 70, 71, 72 | `data.SwiftMT.fields.#20` | Without `#`, interpreted as array index |
| **Alphanumeric** | 13C, 23B, 32A, 33B, 52A, 53B, 56D, 57A, 58D, 71F, 71G | `data.SwiftMT.fields.32A` | Contains letters, no ambiguity |
| **With Option** | 52A, 52D, 53A, 53B, 56A, 56D, 57A, 57B, 57D | `data.SwiftMT.fields.52A` | Letter suffix prevents array interpretation |

### Quick Reference
```json
// ✅ Correct usage:
"data.SwiftMT.fields.#20"     // Sender's Reference
"data.SwiftMT.fields.#21"     // Related Reference  
"data.SwiftMT.fields.32A"     // Value Date/Currency/Amount
"data.SwiftMT.fields.#50"     // Ordering Customer
"data.SwiftMT.fields.#52"     // Ordering Institution (base)
"data.SwiftMT.fields.52A"     // Ordering Institution Option A
"data.SwiftMT.fields.#59"     // Beneficiary Customer
"data.SwiftMT.fields.#70"     // Remittance Information
"data.SwiftMT.fields.#72"     // Sender to Receiver Information
"data.SwiftMT.fields.13C"     // Time Indication
"data.SwiftMT.fields.71A"     // Details of Charges
```

## 8. Critical Implementation Guidelines (MUST READ)

### Workflow Priority Management
**Issue**: Multiple workflows with same priority can cause execution order issues.

**Solution**: Ensure proper priority sequencing:
```json
// Example priority chain for pacs.009:
Priority 1: parse-mx.json (global parser)
Priority 2: 01-variant-detection.json (message-specific)
Priority 3: 02-preconditions.json
Priority 4: 03-headers-mapping.json
Priority 5-10: Field mappings
Priority 11: Postconditions
```

**Rules**:
1. Global parsers (parse-mx.json) should have lowest priority (1)
2. Message-specific workflows start from priority 2+
3. Each workflow should depend on previous via `progress.workflow_id` checks
4. Never have two workflows with same priority in same message type

### SWIFT Block 2 (Application Header) Requirements
**Issue**: "Block 2 too short: expected at least 18 characters, got 16"

**Root Cause**: Missing required fields in application header.

**Solution**: Include all required Block 2 fields:
```json
{
    "path": "data.SwiftMT.application_header.direction",
    "logic": "I"  // or "O" for output
},
{
    "path": "data.SwiftMT.application_header.message_type",
    "logic": "202"
},
{
    "path": "data.SwiftMT.application_header.receiver_bic",
    "logic": {"var": "temp_data.Receiver"}
},
{
    "path": "data.SwiftMT.application_header.priority",
    "logic": "N"  // Normal priority
},
{
    "path": "data.SwiftMT.application_header.delivery_monitoring",
    "logic": "2"  // Required for proper Block 2 length
},
{
    "path": "data.SwiftMT.application_header.obsolescence_period",
    "logic": "003"  // Required for proper Block 2 length
}
```

### Field Structure Requirements for PublishMT
**Issue**: PublishMT expects specific field structures, not nested objects.

**Wrong**:
```json
{
    "path": "data.SwiftMT.fields.#20",
    "logic": {
        "value": {"var": "data.ISO20022_MX.document.CdtTrfTxInf.PmtId.InstrId"}
    }
}
```

**Correct**:
```json
{
    "path": "data.SwiftMT.fields.#20",
    "logic": {
        "reference": {"var": "data.ISO20022_MX.document.CdtTrfTxInf.PmtId.InstrId"}
    }
}
```

For fields with subfields (like option fields), use proper structure:
```json
{
    "path": "data.SwiftMT.fields.#52.A.bic",
    "logic": {"var": "data.ISO20022_MX.document.CdtTrfTxInf.DbtrAgt.FinInstnId.BICFI"}
}
```

### Variant Detection Pattern
For messages with multiple variants (e.g., pacs.009 CORE/COVE/ADV):

1. **Create variant detection workflow** (priority 2):
```json
{
    "id": "pacs009-variant-detection",
    "priority": 2,
    "tasks": [{
        "id": "detect_variant",
        "function": {
            "name": "map",
            "input": {
                "mappings": [{
                    "path": "temp_data.pacs009_variant",
                    "logic": { "if": [
                        {"var": "temp_data.has_underlying_customer"},
                        "COVE",
                        { "if": [
                            {"var": "temp_data.has_reimbursement_agents"},
                            "ADV",
                            "CORE"
                        ]}
                    ]}
                }]
            }
        }
    }]
}
```

2. **Use variant in conditional workflows**:
```json
"condition": { "and": [
    {"==": [{"var": "temp_data.pacs009_variant"}, "COVE"]},
    // other conditions
]}
```

### Testing Transformation with Debug Output
```bash
# Generate sample with specific scenario
curl -X POST http://localhost:3000/generate/sample \
  -H "Content-Type: application/json" \
  -d '{"message_type": "pacs.009", "config": {"scenario": "standard"}}' \
  | jq -r '.result' > /tmp/test.xml

# Create JSON request
cat /tmp/test.xml | jq -Rs '{"message": .}' > /tmp/request.json

# Transform with full response
curl -X POST http://localhost:3000/transform/mx-to-mt \
  -H "Content-Type: application/json" \
  -d @/tmp/request.json

# Run with debug logging for detailed errors
RUST_LOG=debug cargo run
```

## 9. Performance Considerations

### Optimization Tips
1. Use `temp_data` for intermediate calculations
2. Minimize nested `if` statements
3. Cache frequently accessed paths
4. Use MX_To_MTAgentGeneric for all agent fields
5. Batch related field mappings
6. Set proper workflow priorities to avoid unnecessary condition checks

### Hot Reload Best Practices
- Test changes incrementally with reload API
- Keep backup of working workflows
- Document changes in status.md
- Use version control for workflow files
- Always reload workflows after making changes:
  ```bash
  curl -X POST http://localhost:3000/admin/reload-workflows
  ```