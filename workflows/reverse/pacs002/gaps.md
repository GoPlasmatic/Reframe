# pacs.002 to MT199/MT299 REJT Transformation Gaps

## Message Type Overview
- **Source**: pacs.002.001.10 (FI to FI Payment Status Report)
- **Target**: MT199/MT299 REJT (Rejection Messages)
- **Specification**: CBPR+ xxx-specification/reverse/pacs002/
- **Workflow Maturity**: Level 3 - CBPR+ Compliant

## Implementation Status

### Completed Features
✅ Variant detection file (01-variant-detection.json)
✅ MT message type determination based on original message
✅ Preconditions validation (02-preconditions.json)
✅ Headers mapping (03-headers-mapping.json)
✅ Mandatory fields mapping (04-mandatory-fields-mapping.json)
✅ Status fields mapping (05-status-fields-mapping.json)
✅ Postconditions (06-postconditions.json)

### Precondition Validations
✅ Status report type validation (RJCT only)
✅ Original transaction reference validation
✅ Status reason code validation
✅ BAH sender/receiver BIC validation
✅ Message identification validation

### Header Mappings
✅ Basic header (Block 1) with sender BIC
✅ Application header (Block 2) with receiver BIC and message type
✅ User header (Block 3) with UETR from original transaction
✅ Service type code differentiation between MT199/MT299
✅ Logical terminal construction based on BIC length

### Field Mappings
✅ Field 20: Transaction reference (with truncation to 16 chars)
✅ Field 21: Related reference (OriginalInstructionIdentification)
✅ Field 79: Rejection details with structured format:
  - Line 1: /REJT/
  - Line 2: /[ReasonCode]/[AdditionalInfo up to 44 chars]
  - Line 3: /MREF/[OriginalMessageId]
  - Line 4: /TREF/[OriginalEndToEndId]
  - Line 5: /UETR/[OriginalUETR]
  - Line 6+: /TEXT/[Additional information lines]

### Character Set and Formatting
✅ SWIFT character set compliance
✅ Field length validation and truncation
✅ Multiline field formatting
✅ Empty line handling

## Remaining Minor Gaps

### Optional Enhancements
⚠️ Field 11S: Original message identification (not in specification)
⚠️ Field 77A: Additional narrative for complex rejections (optional)
⚠️ Cross-validation with original transaction data
⚠️ Regulatory compliance indicators (market-specific)

## CBPR+ Compliance
✅ UETR preservation from original transaction
✅ Service level code handling via variant detection
✅ Clearing system identification (when present)
✅ Market practice rules for rejections
✅ Error code mapping per specification

## Testing Recommendations
1. Test with different original message types (pacs.008, pacs.009, MT10x, MT20x)
2. Validate rejection reason code handling
3. Test field truncation for long values
4. Verify UETR preservation in Block 3 and Field 79
5. Test with various AdditionalInformation scenarios
6. Validate character set compliance