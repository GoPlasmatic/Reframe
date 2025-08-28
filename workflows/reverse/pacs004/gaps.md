# pacs.004 to MT103RETN/MT202RETN/MT205RETN Transformation Gaps

## Message Type Overview
- **Source**: pacs.004 (Payment Return)
- **Target**: MT103RETN/MT202RETN/MT205RETN (Return Messages)
- **Specification**: xxx-specification/reverse/pacs004-MTxxxRETN/
- **Workflow Maturity**: Level 4 - Complete

## Implementation Status
✅ **COMPLETE** - All major gaps have been addressed

### Recent Updates (2025-08-27)
1. ✅ Created 01-variant-detection.json for message type determination
2. ✅ Added Field 79 mapping for return reason narrative (critical for RETN)
3. ✅ Updated charge handling for TR008, TR009, TR010
4. ✅ Fixed agent mappings for TR016, TR019, TR021
5. ✅ Enhanced return chain validation
6. ✅ Updated workflow chain dependencies
7. ✅ Updated index.json with new variant detection

## Precondition Status
✅ Basic message structure validation
✅ Variant detection file (01-variant-detection.json)
✅ Return reason code validation
✅ Original transaction reference validation
✅ Return chain validation
✅ CBPR+ compliance validation for returns

## Field Mapping Status

### Mandatory Fields
- ✅ Field 20: Return reference mapping
- ✅ Field 21: Original transaction reference (MT202/205 only)
- ✅ Field 23B: Bank operation code RETN (MT103 only)
- ✅ Field 32A: Value date, currency, amount
- ✅ **Field 79: Return reason narrative (CRITICAL - now implemented)**

### Amount Fields
- ✅ Field 33B: Instructed amount (MT103 or EU rules)
- ✅ Field 36: Exchange rate (MT103 only)
- ✅ Field 13C: Time indications (SNDTIME/RNCTIME)
- ✅ Field 53B: Sender's correspondent

### Charge Fields
- ✅ Field 71A: Charge bearer mapping
- ✅ Field 71F: Individual charges (CRED/SHAR)
- ✅ Field 71G: Aggregated charges (DEBT)
- ✅ Field 72: Charges for MT202/205

### Party Fields
- ✅ Field 50: Ordering customer/institution
- ✅ Field 59: Beneficiary customer/institution
- ✅ Field 77B: Country of residence

### Agent Fields
- ✅ Field 52: Ordering institution (Debtor Agent)
- ✅ Field 56: Intermediary institution
- ✅ Field 57: Account with institution (Creditor Agent)
- ✅ Field 58: Beneficiary institution (MT202/205)

### Remittance Fields
- ✅ Field 70: Ultimate debtor/creditor information
- ✅ Field 79: Return reason and additional information
- ✅ Field 72: Sender to receiver information

## CBPR+ Compliance Status
✅ UETR preservation from original transaction
✅ Service level code handling (G00x codes)
✅ Clearing system identification
✅ Market practice rules for returns
✅ Regulatory compliance for cross-border returns

## Testing Recommendations
1. Test MT103 RETN with pacs.008 original message
2. Test MT202 RETN with pacs.009 original message
3. Test MT205 RETN variant
4. Test variant detection based on agent presence
5. Test Field 79 with various return reason codes
6. Test charge aggregation for DEBT bearer
7. Test EU country validation for Field 33B

## Notes
- Implementation follows CBPR+ User Guide specifications
- All translation rules (TR001-TR023) from specification implemented
- Complex return chain logic properly handled
- Ready for comprehensive testing