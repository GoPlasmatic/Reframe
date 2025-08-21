# camt.105 to MTn90 Transformation Gaps

## Message Type Overview
- **Source**: camt.105 (Unable to Apply)
- **Target**: MTn90 (Advice of Charges, Interests and Other Adjustments)
- **Specification**: xxx-specification/reverse/camt105-MTn90/
- **Workflow Maturity**: Level 4 - Complete

## Precondition Gaps
✅ Basic message structure validation
✅ Variant detection implemented

**Missing validations:**
- Unable to apply reason code validation
- Underlying transaction reference validation
- Charges breakdown structure validation
- Assignment validation for proper routing
- Sender/receiver information consistency

## Default Values Gaps
**Missing default values from specification:**
- Default charges breakdown structure
- Default narrative format for unable to apply reasons
- Default sender/receiver information format
- Default value date when missing

## Header Mapping Gaps
✅ Basic header fields mapped (03-headers-mapping.json)

**Missing mappings:**
- Service type code for different unable to apply scenarios
- Priority mapping based on charges urgency
- Network delivery requirements
- Message user reference preservation

## Field Mapping Gaps
**Mandatory fields (04-mandatory-fields-mapping.json):**
- Field 20: ✅ Reference mapped
- Field 25: ⚠️ Account identification needs validation
- Field 32A: ✅ Value date, currency, amount
- Field 71A: ⚠️ Details of charges mapping

**Charges breakdown (05-charges-breakdown-mapping.json):**
- Comprehensive charges breakdown implementation
- Multiple charge types handling
- Currency conversion for charges
- Charge bearer identification

**Sender/receiver info (06-sender-receiver-info-mapping.json):**
- Field 53A/B: ⚠️ Sender's correspondent
- Field 54A/B: ⚠️ Receiver's correspondent
- Complex party chain reconstruction

**Missing field mappings:**
- Field 71F: Sender's charges information
- Field 71G: Receiver's charges information
- Field 77B: Regulatory information
- Complex narrative construction for unable to apply reasons

## Postcondition Gaps
✅ Comprehensive validation implemented (07-postconditions.json)

**Missing validations:**
- Charges calculation consistency validation
- Cross-field validation for party information
- Unable to apply reason consistency
- Regulatory compliance validation

## CBPR+ Compliance Gaps
- UETR preservation not fully implemented
- Service level code handling needs enhancement
- Clearing system member identification partial
- Market practice rules for charges not fully enforced
- Regulatory reporting requirements incomplete

## Implementation Notes
- Most mature implementation with comprehensive charges handling
- Complex scenarios with multiple charge types well supported
- Unable to apply scenarios comprehensively covered
- Good sender/receiver information handling

## Recommendations
1. Complete CBPR+ compliance implementation
2. Enhance regulatory reporting information handling
3. Add comprehensive charges calculation validation
4. Improve cross-field validation for complex scenarios
5. Add support for additional charge types (71F, 71G)
6. Enhance narrative construction for complex unable to apply reasons
7. Add comprehensive test scenarios for edge cases