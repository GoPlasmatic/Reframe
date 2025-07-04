# 📋 Message Formats

Complete reference guide for all supported SWIFT MT and ISO 20022 message formats in Reframe.

## Table of Contents
- [Overview](#overview)
- [Forward Transformations](#forward-transformations)
- [Reverse Transformations](#reverse-transformations)
- [Message Coverage Matrix](#message-coverage-matrix)
- [Field Mapping References](#field-mapping-references)
- [Business Rules](#business-rules)
- [Message Examples](#message-examples)

---

## Overview

Reframe supports comprehensive bidirectional transformation between SWIFT MT and ISO 20022 message formats, covering the complete lifecycle of financial messaging including payments, cancellations, investigations, cash management, and cheque processing.

### Supported Message Families

- **Payment Messages**: Customer and financial institution credit transfers
- **Cancellation Messages**: Request and response for cancellation
- **Investigation Messages**: Query and response messages
- **Cash Management**: Account notifications and statements
- **Cheque Processing**: Presentment, cancellation, and status messages
- **Status Reporting**: Payment status and rejection notifications

### Coverage Statistics

| Category | MT Messages | ISO 20022 Schemas | Total Variants |
|----------|-------------|-------------------|----------------|
| **Payments** | 3 families | 3 schemas | 12 variants |
| **Cancellations** | 4 families | 1 schema | 8 variants |
| **Cash Management** | 4 families | 2 schemas | 6 variants |
| **Cheque Processing** | 3 families | 3 schemas | 6 variants |
| **Status Reporting** | 2 families | 1 schema | 4 variants |
| **Total** | **16 families** | **10 schemas** | **36 variants** |

---

## Forward Transformations

### Payment Messages

#### MT103 → pacs.008/pacs.002/pacs.004

**MT103 - Single Customer Credit Transfer**

| Processing Method | Target Schema | Description |
|------------------|---------------|-------------|
| **Normal** | pacs.008.001.08 | Standard customer credit transfer |
| **STP** | pacs.008.001.08 | Straight-through processing |
| **Urgent** | pacs.008.001.08 | High priority processing |
| **Rejection** | pacs.002.001.10 | Payment status rejection |
| **Return** | pacs.004.001.09 | Payment return notification |

**Key Fields Mapped**:
- Tag 20 → Payment identification
- Tag 32A → Settlement amount and date
- Tag 50K → Debtor information
- Tag 52A/52D → Debtor agent
- Tag 56A/57A → Intermediary agents
- Tag 59 → Creditor information
- Tag 70 → Remittance information
- Tag 71A → Charge bearer

**Business Rules**:
- Service level determination based on tag 23B
- Settlement method based on intermediary agent fields (53A/53B/53D)
- Charge bearer mapping (OUR/BEN/SHA)
- Currency and amount validation

#### MT202 → pacs.009/pacs.002/pacs.004

**MT202 - General Financial Institution Transfer**

| Processing Method | Target Schema | Description |
|------------------|---------------|-------------|
| **Normal** | pacs.009.001.08 | Standard FI-to-FI transfer |
| **Cover** | pacs.009.001.08 | Cover payment method |
| **Rejection** | pacs.002.001.10 | Payment status rejection |
| **Return** | pacs.004.001.09 | Payment return notification |

**Key Fields Mapped**:
- Tag 20 → Transaction reference
- Tag 21 → Related reference
- Tag 32A → Value date, currency, amount
- Tag 52A/52D → Ordering institution
- Tag 53A/53B/53D → Sender's correspondent
- Tag 54A/54B/54D → Receiver's correspondent
- Tag 56A/56D → Intermediary institution
- Tag 57A/57D → Account with institution
- Tag 58A/58D → Beneficiary institution
- Tag 72 → Sender to receiver information

#### MT205 → pacs.009/pacs.002/pacs.004

**MT205 - Corporate Trade**

| Processing Method | Target Schema | Description |
|------------------|---------------|-------------|
| **Normal** | pacs.009.001.08 | Corporate trade transfer |
| **Cover** | pacs.009.001.08 | Cover payment method |
| **Rejection** | pacs.002.001.10 | Payment status rejection |
| **Return** | pacs.004.001.09 | Payment return notification |

**Key Fields Mapped**:
- Similar to MT202 with additional corporate-specific fields
- Tag 77B → Regulatory reporting information
- Enhanced party identification requirements

### Cancellation Messages

#### MT192 → camt.056

**MT192 - Request for Cancellation (Customer Credit Transfer)**

| Field | ISO 20022 Mapping | Description |
|-------|-------------------|-------------|
| Tag 20 | CxlId | Cancellation identification |
| Tag 21 | OrgnlInstrId | Original instruction identification |
| Tag 11S | OrgnlTxId | Original transaction identification |
| Tag 79 | CxlRsnInf | Cancellation reason |

**Business Logic**:
- UETR (Unique End-to-End Transaction Reference) handling
- Original message reference validation
- Cancellation reason code mapping

#### MT292 → camt.056

**MT292 - Request for Cancellation (Financial Institution Transfer)**

Similar structure to MT192 but for institutional transfers.

#### MT196 → camt.056

**MT196 - Client Notification (Customer Credit Transfer)**

Response to cancellation request with status information.

#### MT296 → camt.056

**MT296 - Client Notification (Financial Institution Transfer)**

Response to cancellation request for institutional transfers.

### Cash Management Messages

#### MT900 → camt.054

**MT900 - Confirmation of Debit**

| Field | ISO 20022 Mapping | Description |
|-------|-------------------|-------------|
| Tag 20 | NtfctnId | Notification identification |
| Tag 21 | AcctSvcrRef | Account servicer reference |
| Tag 25 | Acct | Account identification |
| Tag 32A | Amt | Transaction amount |
| Tag 50K/50F | RltdPties | Related parties |

#### MT910 → camt.054

**MT910 - Confirmation of Credit**

Similar structure to MT900 for credit notifications.

#### MT940 → camt.053

**MT940 - Customer Statement**

Account statement with transaction details and balances.

#### MT942 → camt.052

**MT942 - Interim Transaction Report**

Intraday account report with transaction notifications.

### Cheque Processing Messages

#### MT110 → camt.107

**MT110 - Advice of Cheque(s)**

Cheque presentment notification with cheque details.

#### MT111 → camt.108

**MT111 - Request for Stop Payment of a Cheque**

Cheque cancellation or stop payment request.

#### MT112 → camt.109

**MT112 - Status of Request for Stop Payment of a Cheque**

Response to cheque stop payment request.

---

## Reverse Transformations

### Payment Messages

#### pacs.008 → MT103

**pacs.008 - Customer Credit Transfer Initiation**

| ISO 20022 Element | MT Field | Description |
|-------------------|----------|-------------|
| GrpHdr.MsgId | Tag 20 | Message identification |
| GrpHdr.CreDtTm | App Header | Creation date time |
| CdtTrfTxInf.PmtId.InstrId | Tag 20 | Instruction identification |
| CdtTrfTxInf.IntrBkSttlmAmt | Tag 32A | Settlement amount |
| CdtTrfTxInf.Dbtr | Tag 50K | Ordering customer |
| CdtTrfTxInf.DbtrAgt | Tag 52A | Ordering institution |
| CdtTrfTxInf.CdtrAgt | Tag 57A | Account with institution |
| CdtTrfTxInf.Cdtr | Tag 59 | Beneficiary customer |
| CdtTrfTxInf.RmtInf | Tag 70 | Remittance information |

**Transformation Logic**:
- Service level mapping from SvcLvl.Cd to tag 23B
- Charge bearer conversion (DEBT/CRED/SHAR → BEN/OUR/SHA)
- Party identification hierarchy resolution
- Amount and currency formatting

#### pacs.009 → MT202/MT205

**pacs.009 - Financial Institution Credit Transfer**

| ISO 20022 Element | MT Field | Description |
|-------------------|----------|-------------|
| GrpHdr.InstgAgt | Tag 52A/52D | Instructing agent |
| GrpHdr.InstdAgt | Tag 58A/58D | Instructed agent |
| CdtTrfTxInf.IntrmyAgt1 | Tag 56A/56D | Intermediary agent 1 |
| CdtTrfTxInf.IntrmyAgt2 | Tag 57A/57D | Intermediary agent 2 |
| CdtTrfTxInf.InstrForNxtAgt | Tag 72 | Instructions |

**Business Rules**:
- Message type determination (MT202 vs MT205) based on business context
- Correspondent banking chain reconstruction
- Cover payment method detection

#### pacs.004 → MT103RETN/MT202RETN/MT205RETN

**pacs.004 - Payment Return**

Return message transformation with original payment reference and return reason.

#### pacs.002 → MT199/MT299

**pacs.002 - Payment Status Report**

Status report transformation for payment rejections and confirmations.

### Cash Management Messages

#### camt.052 → MT942

**camt.052 - Bank to Customer Account Report**

Interim account report with transaction entries.

#### camt.053 → MT940

**camt.053 - Bank to Customer Statement**

Account statement with opening/closing balances and transaction details.

#### camt.054 → MT900/MT910

**camt.054 - Bank to Customer Debit/Credit Notification**

Individual transaction notifications.

### Cancellation Messages

#### camt.056 → MT192/MT292/MT196/MT296

**camt.056 - Cancellation Request**

Cancellation request/response transformation with UETR support.

### Cheque Processing Messages

#### camt.107 → MT110

**camt.107 - Cheque Presentment Notification**

Cheque presentment details transformation.

#### camt.108 → MT111

**camt.108 - Cheque Cancellation/Stop Request**

Cheque stop payment request transformation.

#### camt.109 → MT112

**camt.109 - Cheque Cancellation/Stop Report**

Cheque stop payment status transformation.

---

## Message Coverage Matrix

### Complete Transformation Matrix

| SWIFT MT | Description | ISO 20022 | Description | Status |
|----------|-------------|-----------|-------------|--------|
| **MT103** | Customer Credit Transfer | **pacs.008** | Customer Credit Transfer | ✅ Full |
| **MT103** | Credit Transfer Rejection | **pacs.002** | Payment Status Report | ✅ Full |
| **MT103** | Credit Transfer Return | **pacs.004** | Payment Return | ✅ Full |
| **MT202** | FI Credit Transfer | **pacs.009** | FI Credit Transfer | ✅ Full |
| **MT202** | FI Transfer Rejection | **pacs.002** | Payment Status Report | ✅ Full |
| **MT202** | FI Transfer Return | **pacs.004** | Payment Return | ✅ Full |
| **MT205** | Corporate Trade | **pacs.009** | FI Credit Transfer | ✅ Full |
| **MT205** | Corporate Rejection | **pacs.002** | Payment Status Report | ✅ Full |
| **MT205** | Corporate Return | **pacs.004** | Payment Return | ✅ Full |
| **MT192** | Cancellation Request (Customer) | **camt.056** | Cancellation Request | ✅ Full |
| **MT292** | Cancellation Request (FI) | **camt.056** | Cancellation Request | ✅ Full |
| **MT196** | Cancellation Response (Customer) | **camt.056** | Cancellation Request | ✅ Full |
| **MT296** | Cancellation Response (FI) | **camt.056** | Cancellation Request | ✅ Full |
| **MT900** | Confirmation of Debit | **camt.054** | Debit/Credit Notification | ✅ Full |
| **MT910** | Confirmation of Credit | **camt.054** | Debit/Credit Notification | ✅ Full |
| **MT940** | Customer Statement | **camt.053** | Customer Statement | ✅ Full |
| **MT942** | Interim Transaction Report | **camt.052** | Account Report | ✅ Full |
| **MT110** | Advice of Cheque(s) | **camt.107** | Cheque Presentment | ✅ Full |
| **MT111** | Stop Payment Request | **camt.108** | Cheque Cancellation | ✅ Full |
| **MT112** | Stop Payment Status | **camt.109** | Cheque Stop Report | ✅ Full |

---

## Field Mapping References

### Common Field Mappings

#### Amount Fields

| SWIFT MT | ISO 20022 | Format | Example |
|----------|-----------|--------|---------|
| Tag 32A | IntrBkSttlmAmt | YYMMDDCCCNNNNN,NN | 240115USD1000,50 |
| Tag 33B | InstructedAmount | CCCNNNNN,NN | USD1000,50 |
| Tag 71A | ChrgBr | Code | OUR/BEN/SHA → CRED/DEBT/SHAR |

#### Party Fields

| SWIFT MT | ISO 20022 | Format | Description |
|----------|-----------|--------|-------------|
| Tag 50K | Dbtr | Name and Address | Debtor/Ordering Customer |
| Tag 52A/52D | DbtrAgt | BIC or Name/Address | Debtor Agent |
| Tag 56A/56D | IntrmyAgt1 | BIC or Name/Address | Intermediary Agent 1 |
| Tag 57A/57D | CdtrAgt | BIC or Name/Address | Creditor Agent |
| Tag 59 | Cdtr | Name and Address | Creditor/Beneficiary |

#### Reference Fields

| SWIFT MT | ISO 20022 | Description |
|----------|-----------|-------------|
| Tag 20 | InstrId/MsgId | Instruction/Message ID |
| Tag 21 | EndToEndId | End-to-end identification |
| Tag 103 | SvcLvl | Service level |
| Tag 121 | UETR | Unique End-to-End Transaction Reference |

---

## Business Rules

### Validation Rules

#### Message Structure Validation

```json
{
  "mt103_validation": {
    "required_fields": ["20", "32A", "50K", "59"],
    "optional_fields": ["21", "23B", "52A", "53A", "56A", "57A", "70", "71A"],
    "conditional_fields": {
      "53A": "requires_correspondent_banking",
      "53B": "alternative_to_53A",
      "53D": "alternative_to_53A_53B"
    }
  }
}
```

#### Amount Validation

```json
{
  "amount_validation": {
    "min_amount": 0.01,
    "max_amount": 999999999.99,
    "decimal_places": 2,
    "supported_currencies": ["USD", "EUR", "GBP", "JPY", "CHF", "CAD", "AUD"]
  }
}
```

#### BIC Validation

```json
{
  "bic_validation": {
    "format": "^[A-Z]{6}[A-Z0-9]{2}([A-Z0-9]{3})?$",
    "length": [8, 11],
    "examples": ["BANKUS33XXX", "DEUTDEFFXXX"]
  }
}
```

### Business Logic Rules

#### Settlement Method Determination

1. **Priority Order**:
   - If Tag 53A present → CLRG (Clearing)
   - Else if Tag 53B present → INDA (Indirect)
   - Else if Tag 53D present → INGA (Indirect)
   - Else → COVE (Cover)

2. **Validation**:
   - Only one 53 tag variant allowed
   - 53A must contain valid BIC
   - 53B must contain valid location code

#### Charge Bearer Logic

```json
{
  "charge_bearer_rules": {
    "OUR": {
      "description": "All charges borne by ordering customer",
      "iso20022": "CRED"
    },
    "BEN": {
      "description": "All charges borne by beneficiary",
      "iso20022": "DEBT"
    },
    "SHA": {
      "description": "Charges shared",
      "iso20022": "SHAR"
    }
  }
}
```

#### Message Type Routing

```json
{
  "routing_rules": {
    "mt103_normal": {
      "condition": "tag_23B == 'CRED'",
      "target_schema": "pacs.008.001.08",
      "processing_method": "normal"
    },
    "mt103_stp": {
      "condition": "tag_23B == 'SPAY'",
      "target_schema": "pacs.008.001.08",
      "processing_method": "stp"
    },
    "mt103_return": {
      "condition": "tag_23B == 'RETN'",
      "target_schema": "pacs.004.001.09",
      "processing_method": "return"
    }
  }
}
```

---

## Next Steps

1. **[Installation Guide](installation.md)** - Setup and deployment instructions
2. **[Workflow Guide](workflow-guide.md)** - Configuration and customization
3. **[Mapping Guide](mapping-guide.md)** - Field mapping and business rules
4. **[Architecture Guide](architecture.md)** - Technical architecture details

---
