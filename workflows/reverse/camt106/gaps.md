# camt.106 to MTn91 Transformation Gaps

## Message Type Overview
- **Source**: camt.106 (Unable to Apply)
- **Target**: MTn91 (Request for Payment of Charges, Interest and Other Expenses)
- **Specification**: xxx-specification/reverse/camt106-MTn91/
- **Workflow Maturity**: Level 4 - Complete

## Precondition Gaps
✅ Basic message structure validation
✅ Variant detection implemented

**Missing validations:**
- Unable to apply reason validation specific to MTn91
- Payment request structure validation
- Charges and expenses breakdown validation
- Agent institution validation
- Sender/receiver relationship validation

## Default Values Gaps
**Missing default values from specification:**
- Default payment request structure
- Default charges and agent information
- Default narrative format for payment requests
- Default settlement instructions

## Header Mapping Gaps
✅ Basic header fields mapped (03-headers-mapping.json)

**Missing mappings:**
- Service type code for payment requests
- Priority mapping based on payment urgency
- Network delivery requirements specific to MTn91
- Message user reference handling

## Field Mapping Gaps
**Mandatory fields (04-mandatory-fields-mapping.json):**
- Field 20: ✅ Reference mapped
- Field 25: ⚠️ Account identification validation
- Field 32A: ✅ Value date, currency, amount
- Field 71A: ⚠️ Details of charges needs enhancement

**Charges and agent mapping (05-charges-and-agent-mapping.json):**
- Comprehensive charges breakdown implementation
- Agent institution information handling
- Complex fee structure mapping
- Expense categorization

**Sender/receiver info (06-sender-receiver-info-mapping.json):**
- Field 53A/B: ⚠️ Sender's correspondent mapping
- Field 54A/B: ⚠️ Receiver's correspondent mapping
- Payment chain reconstruction

**Missing field mappings:**
- Field 71F: Sender's charges detailed breakdown
- Field 71G: Receiver's charges detailed breakdown
- Field 77B: Regulatory information for payment requests
- Complex settlement instructions

## Postcondition Gaps
✅ Comprehensive validation implemented (07-postconditions.json)

**Missing validations:**
- Payment request consistency validation
- Charges calculation accuracy validation
- Agent information consistency validation
- Cross-validation with underlying transaction

## CBPR+ Compliance Gaps
- UETR preservation in payment requests not implemented
- Service level code handling for payment scenarios
- Clearing system member identification needs enhancement
- Market practice rules for payment requests not fully enforced
- Regulatory compliance for cross-border payments

## Implementation Notes
- Mature implementation with sophisticated charges handling
- Payment request scenarios well covered
- Complex agent information handling implemented
- Good integration with underlying transaction data

## Recommendations
1. Complete CBPR+ compliance for payment scenarios
2. Enhance regulatory information handling for cross-border payments
3. Add comprehensive payment request validation
4. Improve agent information consistency validation
5. Add support for complex settlement instructions
6. Enhance charges breakdown accuracy validation
7. Add comprehensive test scenarios for different payment types