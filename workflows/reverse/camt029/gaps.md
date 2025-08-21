# camt.029 to MTx96 Transformation Gaps

## Message Type Overview
- **Source**: camt.029 (Resolution of Investigation)
- **Target**: MT196/MT296 (Answer/Query Messages)
- **Specification**: xxx-specification/reverse/camt029-MTx96/
- **Workflow Maturity**: Level 3 - Advanced

## Precondition Gaps
✅ Basic message structure validation
✅ Variant detection for MT196 vs MT296

**Missing validations:**
- Case identification format validation
- Status reason code validation
- Resolution details structure validation
- Assignment validation for routing

## Default Values Gaps
**Missing default values from specification:**
- Field 11S: Default date/time when not provided
- Field 76: Default narrative structure
- Field 77A: Default narrative format

## Header Mapping Gaps
✅ Basic header fields mapped
✅ Message type determination (196/296)

**Missing mappings:**
- Service type code for CBPR+
- Priority mapping from urgency
- Possible duplicate indication

## Field Mapping Gaps
**Mandatory fields (04-mandatory-fields-mapping.json):**
- Field 20: ✅ Transaction reference
- Field 21: ✅ Related reference
- Field 76: ⚠️ Answers/narrative mapping needs enhancement
- Field 77A: ⚠️ Narrative continuation logic

**Narrative fields (05-narrative-fields-mapping.json):**
- Basic narrative construction implemented
- Complex resolution details may be truncated
- Missing structured data preservation

**Missing field mappings:**
- Field 11S: MT and Date of Original Message
- Field 79: Additional narrative for complex cases
- Compensation amount details when present
- Charges information mapping

## Postcondition Gaps
✅ Basic validation implemented
✅ Field length checks

**Missing validations:**
- Narrative field coherence validation
- Cross-reference validation
- Total message length validation
- SWIFT character set compliance

## CBPR+ Compliance Gaps
- UETR handling not implemented
- Service level code mapping missing
- Clearing system identification not handled
- Market practice rules not enforced

## Implementation Notes
- Current implementation handles basic resolution scenarios
- Complex investigation cases with multiple statuses need review
- Cancellation request responses may need special handling

## Recommendations
1. Enhance narrative building logic for field 76 and 77A
2. Implement comprehensive status reason code mapping
3. Add support for compensation and charges information
4. Implement CBPR+ specific requirements
5. Add test scenarios for different resolution types