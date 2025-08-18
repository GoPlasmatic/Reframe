# MT196 Specification vs Implementation Gap Analysis

## Executive Summary
This document analyzes the gaps between the MT196/296 to camt.029 specification and the current Reframe implementation.

## 1. PRECONDITION GAPS

### PREC001: UETR Validation ❌ PARTIAL
**Specification:** 
- Must validate UETR format in field 77A: `/UETR/[a-f0-9]{8}-[a-f0-9]{4}-4[a-f0-9]{3}-[89ab][a-f0-9]{3}-[a-f0-9]{12}`
- If no UETR in Block3 or Field 77A, raise error T20087 and STOP translation
- UETR in Field 77A starts with "//" on the second line

**Current Implementation:**
- ✅ Checks for UETR in Block3 (user header)
- ✅ Extracts UETR from field 77A
- ❌ No UETR format validation
- ❌ No T20087 error raised when UETR is missing
- ❌ Doesn't handle "//" prefix on second line properly
- ❌ Translation continues even without UETR

### PREC002: Field 76 Validation ✅ IMPLEMENTED
**Specification:**
- Field 76 Line 1 must start with `/CNCL/`, `/PDCR/`, or `/RJCR/`
- Otherwise raise T20093 and STOP translation

**Current Implementation:**
- ✅ Validates Field 76 patterns
- ✅ Includes additional `/CUST/` pattern (acceptable extension)
- ❌ No T20093 error code raised (uses generic message)

### PREC003: BAH Translation ✅ IMPLEMENTED
**Specification:**
- MT Headers to BAH translation must be done before payload translation
- Map BAH From/To to Assigner/Assignee (TR002)

**Current Implementation:**
- ✅ BAH mapping is executed with priority 2 (before document mapping at priority 4)
- ✅ Correctly maps sender/receiver to Assigner/Assignee

## 2. TRANSLATION RULE GAPS

### TR001: Date Format Conversion ❌ PARTIAL
**Specification:**
- Convert Field 11R date (6!n) to ISO format: `YYMMDD` → `YYYY-MM-DD T00:00:00+00:00`

**Current Implementation:**
- ❌ Uses raw concatenation without proper date conversion
- ❌ No century conversion (YY to YYYY)
- ❌ Incorrect field reference (uses 11R.date_time instead of 11R.date)

### TR002: Sender/Receiver Extraction ✅ IMPLEMENTED
**Specification:**
- Extract sender/receiver BICs from BAH

**Current Implementation:**
- ✅ Correctly implemented

### TR003: MT to MX Type Mapping ❌ PARTIAL
**Specification:**
- Map MT types to MX equivalents:
  - 103 → pacs.008.001.08
  - 104 → pacs.003.001.08
  - 202/205 → pacs.009.001.08
  - 204 → pacs.010.001.03
  - Others → MT{type}

**Current Implementation:**
- ✅ Mappings for 103, 104, 202, 205 are correct
- ❌ 204 maps to pacs.010.001.03 (spec) but implementation uses pacs.010.001.03
- ❌ Incorrect field reference (11.R instead of 11R)

### TR004: UETR Extraction from Field 77A ❌ PARTIAL
**Specification:**
- Extract UETR from field 77A if present and absent from Block3
- UETR identifier starts with "//" on the second line in Field 77A
- Filter out UETR lines from 77A narrative

**Current Implementation:**
- ✅ Extracts UETR from field 77A
- ❌ Doesn't handle "//" prefix properly
- ❌ Filtering logic is incorrect
- ❌ Doesn't properly extract UETR value after /UETR/

### MT_To_MXField76: Field 76 Processing ❌ INCOMPLETE
**Specification:**
- Process Field 76 for cancellation status and reason codes
- Map to CancellationStatusReasonInformation
- Code ARPL removed from CBPR+ (should go to AdditionalInformation)

**Current Implementation:**
- ✅ Maps basic status codes
- ❌ No special handling for ARPL code
- ❌ Doesn't properly extract reason codes after status patterns
- ❌ AdditionalInformation hardcoded instead of extracting from Field 76

## 3. FIELD MAPPING GAPS

### Field 11R (MT and Date) ❌ ISSUES
**Specification:**
- Optional field (0..1)
- If absent, copy "NOTPROVIDED" to OriginalMessageNameIdentification
- Map message type using TR003
- Convert date using TR001

**Current Implementation:**
- ❌ Incorrect field access (uses 11.R and 11R interchangeably)
- ❌ Date conversion not properly implemented
- ✅ NOTPROVIDED default is correct

### Field 76 (Answers) ❌ PARTIAL
**Specification:**
- Mandatory (1..1)
- Map to Status/Confirmation and CancellationStatusReasonInformation
- Process using MT_To_MXField76

**Current Implementation:**
- ✅ Basic mapping works
- ❌ Doesn't extract actual reason codes from field content
- ❌ AdditionalInformation not properly extracted

### Field 77A (Narrative) ❌ PARTIAL
**Specification:**
- Optional (0..1)
- Extract UETR only using TR004
- Rest of content should NOT be translated

**Current Implementation:**
- ✅ UETR extraction attempted
- ❌ Currently maps to empty SupplementaryData (should not translate at all)
- ❌ UETR extraction logic incomplete

### Field 79 (Narrative Description) ❌ NOT IMPLEMENTED
**Specification:**
- Optional (0..1)
- No translation required

**Current Implementation:**
- ✅ Correctly not translated

## 4. ERROR HANDLING GAPS

The following error codes are not implemented:
- **T20087**: Missing UETR in Block3 and Field 77A
- **T20093**: Invalid status pattern in Field 76 Line 1

## 5. MISSING FEATURES

1. **Proper UETR extraction from Field 77A with "//" handling**
2. **Date conversion logic (YY to YYYY)**
3. **Field 76 detailed parsing for reason codes**
4. **ARPL code special handling**
5. **Error code implementation**

## 6. RECOMMENDATIONS

### Critical Fixes (Must Have)
1. Fix Field 11R access (use consistent field name)
2. Implement proper UETR extraction from Field 77A
3. Fix date conversion (TR001)
4. Extract actual content from Field 76 for AdditionalInformation

### Important Fixes (Should Have)
1. Implement error codes T20087 and T20093
2. Handle ARPL code properly
3. Fix "//" prefix handling in Field 77A

### Nice to Have
1. Add comprehensive logging
2. Improve Field 76 parsing for all patterns