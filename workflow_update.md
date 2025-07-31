# Workflow Update Guide

This document outlines the process for updating mapping workflows when new SWIFT specifications are released.
The input to this steps are
- Location to new specification from Swift
- Location to existing mapping workflow

## 1. Forward Mapping Workflow (MT → ISO 20022)

### a. Pick a MT Message Type for which the mapping is going to be updated

### b. Generate MT Message Sample

The MT sample generation API provides realistic test messages that can be used for getting the canonical JSON structure and for testing the transformation mapping workflow. 

#### Pick a scenario related to the message type selected
For a complete list of available scenarios for each message type, refer to: [📄 Scenarios](test_scenarios/README.md)

Common scenarios include:
- `standard` - Basic payment scenario
- `stp` - Straight-through processing
- `high_value` - Large amount transfers
- `cbpr_*` - CBPR+ compliant scenarios (business, P2P, real estate, etc.)
- `fx_conversion` - Cross-currency payments
- `regulatory_compliant` - Full regulatory data

#### Use generate sample API to get a MT Message Sample
```bash
curl -X POST http://localhost:3000/generate/mt-sample \
  -H "Content-Type: application/json" \
  -d '{
    "message_type": "MT103",
    "config": {
        "include_optional": true,
        "scenario": "cbpr_business_payment"
    },
    "options": {
        "validation": true,
        "include_debug": true,
        "format_output": false
    }
  }'
```

Output:
```json
{
    "success": true,
    "transformed_message": "{1:F01TGIESJU1559XXXX0001003456}\n{2:I103GTQABDD1237XXXXN}\n{3:{119:STP}{121:2745cad7-c5d8-4466-aa4b-1e879ca7a4b8}{111:001}}\n{4:\n:20:CBPR366000030068\n:23B:CRED\n:32A:241220USD233380,03\n:50K://GB82WEST12345698765432\nCorkery and Marvin Inc\n/LEIXG8HWVXGGR1B2V5K893\n6388 Rath Plain Mount\nSchuyler view, SM\n:52A:TGIESJU1559\n:56A:QPMAIQD1\n:57A:GTQABDD1237\n:59://DE89370400440532013000\nGottlieb and Ritchie Group\n/LEIZBQR12345HKLMNOP678\n2467 Hammes Manor Hollow\nKoelpin ton, MD\n:70:/PURP/GDDS\n/INV/INV-2024-39821\n/PO/PO-506041\n/RFB/CBPR-PAYMENT-REF\n:71A:SHA\n:72:/ACC/CBPR+ COMPLIANT\n/INS/TRANSPARENCY DATA INCLUDED\n/BNF/LEI VERIFICATION COMPLETED\n:77T:EXTENDED REMITTANCE INFORMATION\n-}",
    "debug_info": {
        "engine_state": "sample_generation",
        "workflow_execution": [
            "Sample generated from JSON config"
        ],
        "intermediate_data": {
            "include_optional": true,
            "scenario": "cbpr_business_payment"
        }
    }
}
```

### c. Validate and Get Canonical JSON Structure
This step is used to get the reference JSON canonical structure for the given MT Message type. This structure is very helpful during mapping as we can understand the input JSON structure that comes to mapping task. 

#### Getting Canonical JSON via Transformation API
```bash
curl -X POST http://localhost:3000/validate/mt \
  -H "Content-Type: application/json" \
  -d '{
    "message": "{1:F01TGIESJU1559XXXX0001003456}\n{2:I103GTQABDD1237XXXXN}\n{3:{119:STP}{121:2745cad7-c5d8-4466-aa4b-1e879ca7a4b8}{111:001}}\n{4:\n:20:CBPR366000030068\n:23B:CRED\n:32A:241220USD233380,03\n:50K://GB82WEST12345698765432\nCorkery and Marvin Inc\n/LEIXG8HWVXGGR1B2V5K893\n6388 Rath Plain Mount\nSchuyler view, SM\n:52A:TGIESJU1559\n:56A:QPMAIQD1\n:57A:GTQABDD1237\n:59://DE89370400440532013000\nGottlieb and Ritchie Group\n/LEIZBQR12345HKLMNOP678\n2467 Hammes Manor Hollow\nKoelpin ton, MD\n:70:/PURP/GDDS\n/INV/INV-2024-39821\n/PO/PO-506041\n/RFB/CBPR-PAYMENT-REF\n:71A:SHA\n:72:/ACC/CBPR+ COMPLIANT\n/INS/TRANSPARENCY DATA INCLUDED\n/BNF/LEI VERIFICATION COMPLETED\n:77T:EXTENDED REMITTANCE INFORMATION\n-}",
    "options": {
      "include_canonical_json": true,
      "include_business_validation": true
    }
  }'
```

Output: 
```json
{
    "valid": true,
    "message_type": "103",
    "canonical_json": {
        "application_header": {
            "delivery_monitoring": "X",
            "destination_address": "GTQABDD1237X",
            "direction": "I",
            "message_type": "103",
            "priority": "X",
            "receiver_bic": "GTQABDD1"
        },
        "basic_header": {
            "application_id": "F",
            "logical_terminal": "TGIESJU1559X",
            "sender_bic": "TGIESJU1",
            "sequence_number": "001003456",
            "service_id": "01",
            "session_number": "XXX0"
        },
        "fields": {
            "20": {"reference": "CBPR366000030068"},
            "23B": {"instruction_code": "CRED"},
            "32A": {"amount": 233380.03,"currency": "USD","value_date": "2024-12-20"},
            "50": {
                "K": {
                    "account": "GB82WEST12345698765432",
                    "name_and_address": ["Corkery and Marvin Inc","/LEIXG8HWVXGGR1B2V5K893","6388 Rath Plain Mount","Schuyler view, SM"]
                }
            },
            "52": {"A": {"bic": "TGIESJU1559","party_identifier": null}},
            "56": {"A": {"bic": "QPMAIQD1","party_identifier": null}},
            "57": {"A": {"bic": "GTQABDD1237","party_identifier": null}},
            "59": {
                "NoOption": {
                    "account": "DE89370400440532013000",
                    "name_and_address": ["Gottlieb and Ritchie Group","/LEIZBQR12345HKLMNOP678","2467 Hammes Manor Hollow","Koelpin ton, MD"]
                }
            },
            "70": {
                "narrative": [
                    "/PURP/GDDS",
                    "/INV/INV-2024-39821",
                    "/PO/PO-506041",
                    "/RFB/CBPR-PAYMENT-REF"
                ]
            },
            "71A": {"code": "SHA"},
            "72": {"information": ["/ACC/CBPR+ COMPLIANT","/INS/TRANSPARENCY DATA INCLUDED","/BNF/LEI VERIFICATION COMPLETED"]},
            "77T": {"envelope_content": "EXTENDED REMITTANCE INFORMATION"}
        },
        "message_type": "103",
        "user_header": {
            "service_type_identifier": "001",
            "unique_end_to_end_reference": "2745cad7-c5d8-4466-aa4b-1e879ca7a4b8",
            "validation_flag": "STP"
        }
    },
    "warnings": [
        {
            "code": "MT103_IS_STP",
            "message": "Message is marked as STP (Straight Through Processing)"
        }
    ]
}
```

### d. Review Specification Reference
- Refer to the provided SWIFT specification documentation for MT message translation rules
- Identify new fields, changed rules, or deprecated elements

### e. Update Mapping Workflow

#### 1. Check JSON Structure Changes
- Compare the canonical JSON structure with the existing workflow definitions
- Identify any new fields or structural changes

#### 2. Find Gaps
- Compare current workflow implementation in `workflows/forward/[MESSAGE_TYPE]/` with specification
- Example:
  - `bah-mapping.json` - Basic Application Header mappings
  - `document-mapping.json` - Document/message body mappings
  - `precondition.json` - Validation and precondition rules

#### 3. Implement Changes
- Update the workflow JSON files to match new specification requirements
- Ensure all mandatory fields are mapped correctly
- Add any new conditional logic or validation rules

### f. Test Transformation
```bash
# Test the updated workflow with the generated sample
curl -X POST http://localhost:3000/transform/mt-to-mx \
  -H "Content-Type: application/json" \
  -d '{
    "message": "{1:F01TGIESJU1559XXXX0001003456}\n{2:I103GTQABDD1237XXXXN}\n{3:{119:STP}{121:2745cad7-c5d8-4466-aa4b-1e879ca7a4b8}{111:001}}\n{4:\n:20:CBPR366000030068\n:23B:CRED\n:32A:241220USD233380,03\n:50K://GB82WEST12345698765432\nCorkery and Marvin Inc\n/LEIXG8HWVXGGR1B2V5K893\n6388 Rath Plain Mount\nSchuyler view, SM\n:52A:TGIESJU1559\n:56A:QPMAIQD1\n:57A:GTQABDD1237\n:59://DE89370400440532013000\nGottlieb and Ritchie Group\n/LEIZBQR12345HKLMNOP678\n2467 Hammes Manor Hollow\nKoelpin ton, MD\n:70:/PURP/GDDS\n/INV/INV-2024-39821\n/PO/PO-506041\n/RFB/CBPR-PAYMENT-REF\n:71A:SHA\n:72:/ACC/CBPR+ COMPLIANT\n/INS/TRANSPARENCY DATA INCLUDED\n/BNF/LEI VERIFICATION COMPLETED\n:77T:EXTENDED REMITTANCE INFORMATION\n-}",
    "options": {
        "include_debug": true
    }
}'
```

Output:

*Note: The output below has been truncated for documentation purposes. The actual API response includes the complete data structure.*

```json
{
    "success": true,
    "transformed_message": [
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<Envelope><AppHdr>...</AppHdr>\n<FIToFICustomerCreditTransferV08>...</FIToFICustomerCreditTransferV08></Envelope>"
    ],
    "debug_info": {
        "engine_state": "forward",
        "workflow_execution": [
            "Completed - 1 messages generated"
        ],
        "intermediate_data": {
            "audit_trail": [
                {
                    "changes": [
                        {
                            "new_value": {
                                "application_header": {
                                    "delivery_monitoring": "X",
                                    "destination_address": "GTQABDD1237X",
                                    "direction": "I",
                                    "message_type": "103",
                                    "priority": "X",
                                    "receiver_bic": "GTQABDD1"
                                },
                                "basic_header": {
                                    "application_id": "F",
                                    "logical_terminal": "TGIESJU1559X",
                                    "sender_bic": "TGIESJU1",
                                    "sequence_number": "001003456",
                                    "service_id": "01",
                                    "session_number": "XXX0"
                                },
                                "fields": {
                                    "20": { "reference": "CBPR366000030068" },
                                    "23B": { "instruction_code": "CRED" },
                                    "32A": { "amount": 233380.03, "currency": "USD", "value_date": "2024-12-20" },
                                    "50": { "K": { "account": "GB82WEST12345698765432", "name_and_address": [...] } },
                                    "52": { "A": { "bic": "TGIESJU1559", "party_identifier": null } },
                                    "56": { "A": { "bic": "QPMAIQD1", "party_identifier": null } },
                                    "57": { "A": { "bic": "GTQABDD1237", "party_identifier": null } },
                                    "59": { "NoOption": { "account": "DE89370400440532013000", "name_and_address": [...] } },
                                    "70": { "narrative": ["/PURP/GDDS", "/INV/INV-2024-39821", "..."] },
                                    "71A": { "code": "SHA" },
                                    "72": { "information": ["/ACC/CBPR+ COMPLIANT", "..."] },
                                    "77T": { "envelope_content": "EXTENDED REMITTANCE INFORMATION" }
                                },
                                "message_type": "103",
                                "user_header": {
                                    "service_type_identifier": "001",
                                    "unique_end_to_end_reference": "2745cad7-c5d8-4466-aa4b-1e879ca7a4b8",
                                    "validation_flag": "STP"
                                }
                            },
                            "old_value": null,
                            "path": "data.SwiftMT"
                        }
                    ],
                    "status_code": 200,
                    "task_id": "parse_mt_message",
                    "timestamp": "2025-07-31T07:51:42.277578+00:00",
                    "workflow_id": "parser"
                },
                /* Additional audit trail entries truncated for brevity */
            ],
            "data": {
                "SwiftMT": {
                    /* Canonical JSON structure of parsed MT message */
                    "basic_header": { /* ... */ },
                    "application_header": { /* ... */ },
                    "user_header": { /* ... */ },
                    "fields": {
                        /* Parsed MT fields in structured format */
                    },
                    "message_type": "103"
                },
                "AppHdr": {
                    /* ISO 20022 Business Application Header */
                    "Fr": { /* Sender info */ },
                    "To": { /* Receiver info */ },
                    "BizMsgIdr": "NOTPROVIDED",
                    "MsgDefIdr": "pacs.008.001.08",
                    /* Additional header fields... */
                },
                "Document": {
                    /* ISO 20022 Document structure */
                    "FIToFICstmrCdtTrf": {
                        "GrpHdr": { /* Group header */ },
                        "CdtTrfTxInf": { /* Credit transfer info */ }
                    }
                },
                "document_xml": "<FIToFICustomerCreditTransferV08>...</FIToFICustomerCreditTransferV08>",
                "header_xml": "<AppHdr>...</AppHdr>",
                "result": [
                    "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<Envelope>...</Envelope>"
                ]
            },
            "errors": [],
            "metadata": {
                /* Processing metadata */
            },
            "temp_data": {
                /* Intermediate workflow data */
            }
        }
    }
}
```

*Note: The debug_info.audit_trail and data sections have been truncated for documentation purposes. In actual responses, these sections contain detailed workflow execution steps and complete data structures.*

**Key Output Structure Elements:**
- `success`: Indicates if transformation was successful
- `transformed_message`: The resulting ISO 20022 XML message
- `debug_info`: Contains workflow execution details and audit trail
- `data`: Contains parsed structures including:
  - `SwiftMT`: The canonical JSON structure of the parsed MT message
  - `AppHdr`: ISO 20022 Business Application Header
  - `Document`: ISO 20022 Document structure
- `metadata`: Processing metadata and message type information

### g. Error Resolution
If transformation errors occur, refer to:
- MT message structure implementation files
- MT field implementation files
- See reference links in section 3 and 4 below

### h. Apply Fixes and Retest
- Fix any mapping issues identified during testing
- Reload workflows using hot reload API:
```bash
curl -X POST http://localhost:3000/admin/reload-workflows
```
- Retest transformation until successful

## 2. Reverse Mapping Workflow (ISO 20022 → MT)

*To be updated later*

## 3. MT Message Structure References

All MT message structures supported in the swift-mt-message library:

| Message Type | Description | Reference Path |
|--------------|-------------|----------------|
| MT101 | Request for Transfer | ../SwiftMTMessage/swift-mt-message/src/messages/mt101.rs |
| MT103 | Single Customer Credit Transfer | ../SwiftMTMessage/swift-mt-message/src/messages/mt103.rs |
| MT104 | Direct Debit and Request for Debit Transfer | ../SwiftMTMessage/swift-mt-message/src/messages/mt104.rs |
| MT107 | General Direct Debit | ../SwiftMTMessage/swift-mt-message/src/messages/mt107.rs |
| MT110 | Advice of Cheque(s) | ../SwiftMTMessage/swift-mt-message/src/messages/mt110.rs |
| MT111 | Stop Payment of a Cheque | ../SwiftMTMessage/swift-mt-message/src/messages/mt111.rs |
| MT112 | Status of a Request for Stop Payment | ../SwiftMTMessage/swift-mt-message/src/messages/mt112.rs |
| MT192 | Request for Cancellation | ../SwiftMTMessage/swift-mt-message/src/messages/mt192.rs |
| MT196 | Answers | ../SwiftMTMessage/swift-mt-message/src/messages/mt196.rs |
| MT199 | Free Format Message | ../SwiftMTMessage/swift-mt-message/src/messages/mt199.rs |
| MT202 | General Financial Institution Transfer | ../SwiftMTMessage/swift-mt-message/src/messages/mt202.rs |
| MT205 | Financial Institution Transfer Execution | ../SwiftMTMessage/swift-mt-message/src/messages/mt205.rs |
| MT210 | Notice to Receive | ../SwiftMTMessage/swift-mt-message/src/messages/mt210.rs |
| MT292 | Request for Cancellation | ../SwiftMTMessage/swift-mt-message/src/messages/mt292.rs |
| MT296 | Answers | ../SwiftMTMessage/swift-mt-message/src/messages/mt296.rs |
| MT299 | Free Format Message | ../SwiftMTMessage/swift-mt-message/src/messages/mt299.rs |
| MT900 | Confirmation of Debit | ../SwiftMTMessage/swift-mt-message/src/messages/mt900.rs |
| MT910 | Confirmation of Credit | ../SwiftMTMessage/swift-mt-message/src/messages/mt910.rs |
| MT920 | Request Message | ../SwiftMTMessage/swift-mt-message/src/messages/mt920.rs |
| MT935 | Rate Change Advice | ../SwiftMTMessage/swift-mt-message/src/messages/mt935.rs |
| MT940 | Customer Statement Message | ../SwiftMTMessage/swift-mt-message/src/messages/mt940.rs |
| MT941 | Balance Report | ../SwiftMTMessage/swift-mt-message/src/messages/mt941.rs |
| MT942 | Interim Transaction Report | ../SwiftMTMessage/swift-mt-message/src/messages/mt942.rs |
| MT950 | Statement Message | ../SwiftMTMessage/swift-mt-message/src/messages/mt950.rs |

## 4. MT Field Implementation References

All MT fields supported in the swift-mt-message library:

| Field | Description | Reference Path |
|-------|-------------|----------------|
| Field 11 | MT and Date of Original Message | ../SwiftMTMessage/swift-mt-message/src/fields/field11.rs |
| Field 12 | Sub-Message Type | ../SwiftMTMessage/swift-mt-message/src/fields/field12.rs |
| Field 13 | Date/Time Indication | ../SwiftMTMessage/swift-mt-message/src/fields/field13.rs |
| Field 19 | Sum of Amounts | ../SwiftMTMessage/swift-mt-message/src/fields/field19.rs |
| Field 20 | Transaction Reference Number | ../SwiftMTMessage/swift-mt-message/src/fields/field20.rs |
| Field 21 | Related Reference | ../SwiftMTMessage/swift-mt-message/src/fields/field21.rs |
| Field 23 | Instruction Code | ../SwiftMTMessage/swift-mt-message/src/fields/field23.rs |
| Field 25 | Account Identification | ../SwiftMTMessage/swift-mt-message/src/fields/field25.rs |
| Field 26 | Transaction Type Code | ../SwiftMTMessage/swift-mt-message/src/fields/field26.rs |
| Field 28 | Statement/Page Number | ../SwiftMTMessage/swift-mt-message/src/fields/field28.rs |
| Field 30 | Date | ../SwiftMTMessage/swift-mt-message/src/fields/field30.rs |
| Field 32 | Amount | ../SwiftMTMessage/swift-mt-message/src/fields/field32.rs |
| Field 33 | Currency/Instructed Amount | ../SwiftMTMessage/swift-mt-message/src/fields/field33.rs |
| Field 34 | Credit/Debit Floor Limit | ../SwiftMTMessage/swift-mt-message/src/fields/field34.rs |
| Field 36 | Exchange Rate | ../SwiftMTMessage/swift-mt-message/src/fields/field36.rs |
| Field 37 | Rate | ../SwiftMTMessage/swift-mt-message/src/fields/field37.rs |
| Field 50 | Ordering Customer | ../SwiftMTMessage/swift-mt-message/src/fields/field50.rs |
| Field 51 | Sending Institution | ../SwiftMTMessage/swift-mt-message/src/fields/field51.rs |
| Field 52 | Ordering Institution | ../SwiftMTMessage/swift-mt-message/src/fields/field52.rs |
| Field 53 | Sender's Correspondent | ../SwiftMTMessage/swift-mt-message/src/fields/field53.rs |
| Field 54 | Receiver's Correspondent | ../SwiftMTMessage/swift-mt-message/src/fields/field54.rs |
| Field 55 | Third Reimbursement Institution | ../SwiftMTMessage/swift-mt-message/src/fields/field55.rs |
| Field 56 | Intermediary Institution | ../SwiftMTMessage/swift-mt-message/src/fields/field56.rs |
| Field 57 | Account With Institution | ../SwiftMTMessage/swift-mt-message/src/fields/field57.rs |
| Field 58 | Beneficiary Institution | ../SwiftMTMessage/swift-mt-message/src/fields/field58.rs |
| Field 59 | Beneficiary Customer | ../SwiftMTMessage/swift-mt-message/src/fields/field59.rs |
| Field 60 | Opening Balance | ../SwiftMTMessage/swift-mt-message/src/fields/field60.rs |
| Field 61 | Statement Line | ../SwiftMTMessage/swift-mt-message/src/fields/field61.rs |
| Field 62 | Closing Balance | ../SwiftMTMessage/swift-mt-message/src/fields/field62.rs |
| Field 64 | Closing Available Balance | ../SwiftMTMessage/swift-mt-message/src/fields/field64.rs |
| Field 65 | Forward Available Balance | ../SwiftMTMessage/swift-mt-message/src/fields/field65.rs |
| Field 70 | Remittance Information | ../SwiftMTMessage/swift-mt-message/src/fields/field70.rs |
| Field 71 | Details of Charges | ../SwiftMTMessage/swift-mt-message/src/fields/field71.rs |
| Field 72 | Sender to Receiver Information | ../SwiftMTMessage/swift-mt-message/src/fields/field72.rs |
| Field 75 | Queries | ../SwiftMTMessage/swift-mt-message/src/fields/field75.rs |
| Field 76 | Answers | ../SwiftMTMessage/swift-mt-message/src/fields/field76.rs |
| Field 77 | Regulatory Reporting/Envelope Contents | ../SwiftMTMessage/swift-mt-message/src/fields/field77.rs |
| Field 79 | Narrative | ../SwiftMTMessage/swift-mt-message/src/fields/field79.rs |
| Field 86 | Information to Account Owner | ../SwiftMTMessage/swift-mt-message/src/fields/field86.rs |
| Field 90 | Number and Sum of Entries | ../SwiftMTMessage/swift-mt-message/src/fields/field90.rs |

## 5. Important Workflow Development Guidelines

### Avoiding Task Overwriting Issues

When multiple workflow tasks map data to the same object path, later tasks will completely overwrite data set by earlier tasks, causing data loss. This is a common issue when mapping complex structures.

**Problem Example:**
```json
// Task 1: construct_group_header
{
    "path": "data.Document.FIToFICstmrCdtTrf",
    "logic": {
        "GrpHdr": { /* group header fields */ },
        "CdtTrfTxInf": {
            "InstgAgt": { /* agent data */ },
            "InstdAgt": { /* agent data */ }
        }
    }
}

// Task 2: map_credit_transfer_transaction_info
{
    "path": "data.Document.FIToFICstmrCdtTrf.CdtTrfTxInf",
    "logic": {
        "PmtId": { /* payment fields */ },
        "IntrBkSttlmAmt": { /* amount fields */ }
        // This completely replaces CdtTrfTxInf, losing InstgAgt and InstdAgt!
    }
}
```

**Solution: Use Sub-Path Mapping**

Instead of mapping to parent objects, map each field to its specific sub-path:

```json
// Better approach - each task maps to specific sub-paths
{
    "mappings": [
        {
            "path": "data.Document.FIToFICstmrCdtTrf.CdtTrfTxInf.InstgAgt",
            "logic": { /* agent data */ }
        },
        {
            "path": "data.Document.FIToFICstmrCdtTrf.CdtTrfTxInf.InstdAgt",
            "logic": { /* agent data */ }
        },
        {
            "path": "data.Document.FIToFICstmrCdtTrf.CdtTrfTxInf.PmtId",
            "logic": { /* payment data */ }
        },
        {
            "path": "data.Document.FIToFICstmrCdtTrf.CdtTrfTxInf.IntrBkSttlmAmt",
            "logic": { /* amount data */ }
        }
    ]
}
```

**Best Practices:**
1. Map to the most specific path possible
2. Break large mapping tasks into multiple smaller mappings
3. Use separate tasks for logically distinct groups of fields
4. Review audit trails to identify overwriting issues
5. Test transformations with debug output to verify all fields are preserved

## 6. Troubleshooting Common JSONLogic Issues

When updating workflows, you may encounter "Invalid arguments error" messages. Here are common causes and solutions:

### Field Structure Mismatches

**Problem**: The workflow references a field structure that doesn't match the actual parsed output.

**Example**:
```json
// Workflow expects:
{"var": "data.SwiftMT.fields.70.lines"}

// But actual structure has:
{"var": "data.SwiftMT.fields.70.narrative"}
```

**Solution**: Always verify the canonical JSON structure using the `/validate/mt` API to ensure field references match the actual parsed structure.

### Invalid JSONLogic Functions

The datalogic-rs library implements a subset of JSONLogic. Some commonly mistaken functions:

**1. `index_of` doesn't exist**
```json
// ❌ Invalid:
{"index_of": [{"var": "field72"}, "/ACC/"]}

// ✅ Use starts_with instead:
{"starts_with": [{"var": "field72"}, "/ACC/"]}
```

**2. `matches` (regex) doesn't exist**
```json
// ❌ Invalid:
{"matches": [{"var": "field"}, "^[A-Z]+$"]}

// ✅ Use simpler logic:
{"and": [
    {">=": [{"length": {"var": "field"}}, 1]},
    {"<=": [{"length": {"var": "field"}}, 10]}
]}
```

**3. `count` should be `length`**
```json
// ❌ Invalid:
{"count": {"var": "array"}}

// ✅ Correct:
{"length": {"var": "array"}}
```

**4. `concat` should be `cat`**
```json
// ❌ Invalid:
{"concat": ["Hello", " ", "World"]}

// ✅ Correct:
{"cat": ["Hello", " ", "World"]}
```

### Available datalogic-rs Operators

**Array Operators**: `map`, `filter`, `reduce`, `all`, `some`, `none`, `merge`, `in`, `length`, `slice`, `sort`

**String Operators**: `cat`, `substr`, `starts_with`

**Logic Operators**: `if`, `and`, `or`, `not`, `==`, `!=`, `>`, `<`, `>=`, `<=`

**Math Operators**: `+`, `-`, `*`, `/`, `%`, `min`, `max`

### Complex Field Access Issues

**Problem**: Trying to access nested properties incorrectly.

**Example with Field 72**:
```json
// ❌ Wrong - direct access:
{"var": "data.SwiftMT.fields.72"}

// ✅ Correct - access the information array:
{"var": "data.SwiftMT.fields.72.information"}
```

### Map Operation on Non-Arrays

**Problem**: Using `map` on a single value instead of an array.

```json
// ❌ Invalid - map expects an array:
{
    "map": [
        {"cat": ["value1", "value2"]},  // This returns a string!
        {"var": ""}
    ]
}

// ✅ Solution 1 - Process the string directly:
{"cat": ["value1", "value2"]}

// ✅ Solution 2 - Create array first if mapping needed:
{
    "map": [
        ["item1", "item2"],  // Array literal
        {"var": ""}
    ]
}
```

### Preprocessing Complex Data

For complex transformations, use preprocessing steps:

```json
{
    "mappings": [
        {
            "path": "temp_data.field72_concat",
            "logic": { 
                "if": [
                    {"var": "data.SwiftMT.fields.72.information"},
                    {
                        "reduce": [
                            {"var": "data.SwiftMT.fields.72.information"},
                            {"cat": [{"var": "accumulator"}, " ", {"var": "current"}]},
                            ""
                        ]
                    },
                    ""
                ]
            }
        },
        {
            "path": "data.Document.SomeField",
            "logic": {"var": "temp_data.field72_concat"}
        }
    ]
}
```

### Testing and Debugging Tips

1. **Use the test scenarios script**: Run comprehensive tests across multiple scenarios
   ```bash
   python3 test/test_scenarios.py
   ```

2. **Hot reload workflows**: Apply changes without restarting
   ```bash
   curl -X POST http://localhost:3000/admin/reload-workflows
   ```

3. **Check debug output**: Use `include_debug: true` in API calls to see detailed execution traces

4. **Validate field structures**: Always verify the canonical JSON structure matches your workflow references

5. **Test incrementally**: When fixing complex issues, test after each change to isolate problems

### Common Error Patterns

| Error Message | Common Cause | Solution |
|--------------|--------------|----------|
| "Invalid arguments error" | Unknown JSONLogic function | Check available operators list |
| "Cannot read property of null" | Field doesn't exist | Verify canonical JSON structure |
| "Expected array but got string" | Map/filter on non-array | Use appropriate operator or preprocess |
| "Path not found" | Incorrect field reference | Check exact path in parsed output |

