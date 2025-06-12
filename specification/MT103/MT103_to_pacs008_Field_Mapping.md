# MT103 to pacs.008 Field Mapping

## Overview
This document provides a comprehensive field-to-field mapping between MT103 Single Customer Credit Transfer and pacs.008.001.08 FIToFICustomerCreditTransferV08 messages.

## Group Header Mappings

| MT103 Field | Description | pacs.008 Path | Translation Notes |
|-------------|-------------|---------------|-------------------|
| Field 20 | Sender's Reference | GroupHeader/MessageIdentification | Copy - MessageIdentification is mandatory in MX as identification for group of transactions. MT103 is single transaction so field 20 serves as both instruction and message reference |
| Default | Creation Date/Time | GroupHeader/CreationDateTime | Use default value from system |
| Field 53A | Sender's Correspondent | GroupHeader/SettlementInformation/SettlementMethod | Entry point METAFCT001 for fields 53a, 54a, 55a translation |
| Field 53A (PartyId) | Sender's Correspondent | GroupHeader/SettlementInformation/InstructingReimbursementAgent | IF PartyIdentifier starts with "//" AND not "//CH" use MT_To_MXClearingIdentifier |
| Field 53A (BIC) | Sender's Correspondent | GroupHeader/SettlementInformation/InstructingReimbursementAgent | Use MT_To_MXBICFI |
| Field 53A (Account) | Sender's Correspondent | GroupHeader/SettlementInformation/InstructingReimbursementAgentAccount | IF PartyIdentifier not starts with "//" OR starts with "//CH" use MT_To_MXFinancialInstitutionAccount |
| Field 53B | Sender's Correspondent | GroupHeader/SettlementInformation/SettlementMethod | Entry point METAFCT001 - PartyIdentifier mandatory in STP |
| Field 53B (PartyId) | Sender's Correspondent | GroupHeader/SettlementInformation/SettlementAccount | IF PartyIdentifier not starts with "//" or starts with "//CH" use MT_To_MXFinancialInstitutionAccount - indicates account relationship between Sender and Receiver |
| Field 53B (PartyId) | Sender's Correspondent | GroupHeader/SettlementInformation/InstructingReimbursementAgent | IF PartyIdentifier starts with "//" AND not "//CH" THEN T20001 error - ClearingSystemIdentifier never translated per METAFCT001 |
| Field 53D | Sender's Correspondent | GroupHeader/SettlementInformation/SettlementMethod | Entry point METAFCT001 for fields 53a, 54a, 55a translation |
| Field 53D (PartyId) | Sender's Correspondent | GroupHeader/SettlementInformation/InstructingReimbursementAgent | IF PartyIdentifier starts with "//" AND not "//CH" use MT_To_MXClearingIdentifier |
| Field 53D (Account) | Sender's Correspondent | GroupHeader/SettlementInformation/InstructingReimbursementAgentAccount | IF PartyIdentifier not starts with "//" or starts with "//CH" use MT_To_MXFinancialInstitutionAccount |
| Field 54A | Receiver's Correspondent | GroupHeader/SettlementInformation/SettlementMethod | Entry point METAFCT001 |
| Field 54A (PartyId) | Receiver's Correspondent | GroupHeader/SettlementInformation/InstructedReimbursementAgent | IF PartyIdentifier starts with "//" AND not "//CH" use MT_To_MXClearingIdentifier |
| Field 54A (BIC) | Receiver's Correspondent | GroupHeader/SettlementInformation/InstructedReimbursementAgent | Use MT_To_MXBICFI |
| Field 54A (Account) | Receiver's Correspondent | GroupHeader/SettlementInformation/InstructedReimbursementAgentAccount | IF PartyIdentifier not starts with "//" or starts with "//CH" use MT_To_MXFinancialInstitutionAccount |
| Field 54B (PartyId) | Receiver's Correspondent | GroupHeader/SettlementInformation/InstructedReimbursementAgent | IF PartyIdentifier starts with "//" AND not "//CH" THEN TR010 |
| Field 54B (Account) | Receiver's Correspondent | GroupHeader/SettlementInformation/InstructedReimbursementAgentAccount | IF PartyIdentifier not starts with "//" or starts with "//CH" use MT_To_MXFinancialInstitutionAccount with dummy values |
| Field 54D | Receiver's Correspondent | GroupHeader/SettlementInformation/SettlementMethod | Entry point METAFCT001 |
| Field 54D (PartyId) | Receiver's Correspondent | GroupHeader/SettlementInformation/InstructedReimbursementAgent | IF PartyIdentifier starts with "//" AND not "//CH" use MT_To_MXClearingIdentifier |
| Field 54D (Account) | Receiver's Correspondent | GroupHeader/SettlementInformation/InstructedReimbursementAgentAccount | IF PartyIdentifier not starts with "//" or starts with "//CH" use MT_To_MXFinancialInstitutionAccount |
| Field 55A (PartyId) | Third Reimbursement Institution | GroupHeader/SettlementInformation/ThirdReimbursementAgent | IF PartyIdentifier starts with "//" AND not "//CH" use MT_To_MXClearingIdentifier |
| Field 55A (BIC) | Third Reimbursement Institution | GroupHeader/SettlementInformation/ThirdReimbursementAgent | Use MT_To_MXBICFI |
| Field 55A (Account) | Third Reimbursement Institution | GroupHeader/SettlementInformation/ThirdReimbursementAgentAccount | IF PartyIdentifier not starts with "//" or starts with "//CH" use MT_To_MXFinancialInstitutionAccount |
| Field 55B (PartyId) | Third Reimbursement Institution | GroupHeader/SettlementInformation/ThirdReimbursementAgent | IF PartyIdentifier starts with "//" AND not "//CH" THEN TR014 |
| Field 55B (Account) | Third Reimbursement Institution | GroupHeader/SettlementInformation/ThirdReimbursementAgentAccount | IF PartyIdentifier not starts with "//" or starts with "//CH" use MT_To_MXFinancialInstitutionAccount with dummy values |
| Field 55D (PartyId) | Third Reimbursement Institution | GroupHeader/SettlementInformation/ThirdReimbursementAgent | IF PartyIdentifier starts with "//" AND not "//CH" use MT_To_MXClearingIdentifier |
| Field 55D (Account) | Third Reimbursement Institution | GroupHeader/SettlementInformation/ThirdReimbursementAgentAccount | IF PartyIdentifier not starts with "//" or starts with "//CH" use MT_To_MXFinancialInstitutionAccount |

### Additional Group Header Fields (Derived/Calculated)
| MT103 Field | Description | pacs.008 Path | Translation Notes |
|-------------|-------------|---------------|-------------------|
| Calculated | Number of Transactions | GroupHeader/NbOfTxs | Always "1" for MT103 (single transaction) |
| Field 32A (Currency+Amount) | Total Interbank Settlement Amount | GroupHeader/TtlIntrBkSttlmAmt | Currency and Amount from field 32A |
| Field 32A (Date) | Interbank Settlement Date | GroupHeader/IntrBkSttlmDt | Date component from field 32A using MT_To_MXDate |
| Temp~Sender | Instructing Agent | GroupHeader/InstgAgt/FinInstnId/BICFI | Sender BIC from basic header or field 52A |
| Temp~Receiver | Instructed Agent | GroupHeader/InstdAgt/FinInstnId/BICFI | Receiver BIC from basic header or field 57A |



## Credit Transfer Transaction Information

### Payment Identification
| MT103 Field | Description | pacs.008 Path | Translation Notes |
|-------------|-------------|---------------|-------------------|
| Field 20 | Sender's Reference | CreditTransferTransactionInformation/PaymentIdentification/InstructionIdentification | Copy |
| Block3 UETR | End-to-End Reference | CreditTransferTransactionInformation/PaymentIdentification/UETR | Copy UETR pattern |
| Field 70 | Remittance Information | CreditTransferTransactionInformation/PaymentIdentification/EndToEndIdentification | IF absent use "NOTPROVIDED", else TR013 |

### Payment Type Information
| MT103 Field | Description | pacs.008 Path | Translation Notes |
|-------------|-------------|---------------|-------------------|
| Block3 Service Type | Service Type Identifier | CreditTransferTransactionInformation/PaymentTypeInformation/ServiceLevel/Code | Use TR006 |
| Field 23E CORT | Instruction Code | CreditTransferTransactionInformation/PaymentTypeInformation/CategoryPurpose/Code | Code: CORT |
| Field 23E INTC | Instruction Code | CreditTransferTransactionInformation/PaymentTypeInformation/CategoryPurpose/Code | Code: INTC |
| Field 23E SDVA | Instruction Code | CreditTransferTransactionInformation/PaymentTypeInformation/ServiceLevel/Code | Code: SDVA |
| Field 26T | Transaction Type Code | CreditTransferTransactionInformation/Purpose/Proprietary | Concatenate ":26T:" + Transaction Type Code |
| Field 56A | Intermediary Institution | CreditTransferTransactionInformation/PaymentTypeInformation/ClearingChannel | Use TR009 |

### Amount and Currency
| MT103 Field | Description | pacs.008 Path | Translation Notes |
|-------------|-------------|---------------|-------------------|
| Field 32A (Currency) | Interbank Settled Amount | CreditTransferTransactionInformation/InterbankSettlementAmount/Currency | Use MT_To_MXCurrencyAmount |
| Field 32A (Amount) | Interbank Settled Amount | CreditTransferTransactionInformation/InterbankSettlementAmount/Amount | Copy |
| Field 33B | Instructed Amount | CreditTransferTransactionInformation/InstructedAmount | Use MT_To_MXCurrencyAmount |
| Field 36 | Exchange Rate | CreditTransferTransactionInformation/ExchangeRate | Use MT_To_MXRate |

### Party Information

#### Debtor (Ordering Customer)
| MT103 Field | Description | pacs.008 Path | Translation Notes |
|-------------|-------------|---------------|-------------------|
| Field 50A (Account) | Ordering Customer | CreditTransferTransactionInformation/DebtorAccount | IF absent use "NOTPROVIDED" + "TXID", else MT_To_MXPartyAccount |
| Field 50A (BIC) | Ordering Customer | CreditTransferTransactionInformation/Debtor | Use MT_To_MXAnyBIC |
| Field 50F | Ordering Customer | CreditTransferTransactionInformation/Debtor | Use MT_To_MXFATFIdentification if not starts with "/" |
| Field 50F (Account) | Ordering Customer | CreditTransferTransactionInformation/DebtorAccount | IF starts with "/" use MT_To_MXPartyAccount |
| Field 50K (Account) | Ordering Customer | CreditTransferTransactionInformation/DebtorAccount | IF absent use "NOTPROVIDED" + "TXID", else MT_To_MXPartyAccount |
| Field 50K (Name/Address) | Ordering Customer | CreditTransferTransactionInformation/Debtor | Use MT_To_MXPartyNameAndAddress |

#### Debtor Agent (Ordering Institution)
| MT103 Field | Description | pacs.008 Path | Translation Notes |
|-------------|-------------|---------------|-------------------|
| Field 52A | Ordering Institution | CreditTransferTransactionInformation/DebtorAgent | Use clearing identifier or BICFI |
| Field 52A (Account) | Ordering Institution | CreditTransferTransactionInformation/DebtorAgentAccount | Use MT_To_MXFinancialInstitutionAccount |
| Temp~Sender | Default Debtor Agent | CreditTransferTransactionInformation/DebtorAgent/FinancialInstitutionIdentification/BICFI | IF OrderingInstitution absent, copy BIC of Sender |

#### Intermediary Agent
| MT103 Field | Description | pacs.008 Path | Translation Notes |
|-------------|-------------|---------------|-------------------|
| Field 56A | Intermediary Institution | CreditTransferTransactionInformation/IntermediaryAgent1 | Use clearing identifier or BICFI |
| Field 56A (Account) | Intermediary Institution | CreditTransferTransactionInformation/IntermediaryAgent1Account | Use MT_To_MXFinancialInstitutionAccount |

#### Creditor Agent (Account With Institution)
| MT103 Field | Description | pacs.008 Path | Translation Notes |
|-------------|-------------|---------------|-------------------|
| Field 57A | Account With Institution | CreditTransferTransactionInformation/CreditorAgent | Use clearing identifier or BICFI |
| Field 57A (Account) | Account With Institution | CreditTransferTransactionInformation/CreditorAgentAccount | Use MT_To_MXFinancialInstitutionAccount |
| Temp~Receiver | Default Creditor Agent | CreditTransferTransactionInformation/CreditorAgent/FinancialInstitutionIdentification/BICFI | IF AccountWithInstitution absent, copy BIC of Receiver |

#### Creditor (Beneficiary Customer)
| MT103 Field | Description | pacs.008 Path | Translation Notes |
|-------------|-------------|---------------|-------------------|
| Field 59 (Account) | Beneficiary Customer | CreditTransferTransactionInformation/CreditorAccount | IF absent use "NOTPROVIDED", else MT_To_MXPartyAccount |
| Field 59 (Name/Address) | Beneficiary Customer | CreditTransferTransactionInformation/Creditor | Use MT_To_MXPartyNameAndAddress |
| Field 59A (Account) | Beneficiary Customer | CreditTransferTransactionInformation/CreditorAccount | IF absent use "NOTPROVIDED", else MT_To_MXPartyAccount |
| Field 59A (BIC) | Beneficiary Customer | CreditTransferTransactionInformation/Creditor | Use MT_To_MXAnyBIC |
| Field 59F (Account) | Beneficiary Customer | CreditTransferTransactionInformation/CreditorAccount | IF absent use "NOTPROVIDED", else MT_To_MXPartyAccount |
| Field 59F (Name/Address) | Beneficiary Customer | CreditTransferTransactionInformation/Creditor | Use MT_To_MXPartyNameAndStructuredAddress |

### Charges Information
| MT103 Field | Description | pacs.008 Path | Translation Notes |
|-------------|-------------|---------------|-------------------|
| Field 71A | Details of Charges | CreditTransferTransactionInformation/ChargeBearer | BEN=>CRED, OUR=>DEBT, SHA=>SHAR |
| Field 71F | Sender's Charges | CreditTransferTransactionInformation/ChargesInformation[71F]/Amount | Use TR004 |
| Field 71G | Receiver's Charges | CreditTransferTransactionInformation/ChargesInformation[71G]/Amount | Use TR005 |

### Instructions and Additional Information
| MT103 Field | Description | pacs.008 Path | Translation Notes |
|-------------|-------------|---------------|-------------------|
| Field 23E CHQB | Instruction Code | CreditTransferTransactionInformation/InstructionForCreditorAgent/Code | Code: CHQB |
| Field 23E HOLD | Instruction Code | CreditTransferTransactionInformation/InstructionForCreditorAgent/Code | Code: HOLD |
| Field 23E PHOB | Instruction Code | CreditTransferTransactionInformation/InstructionForCreditorAgent/Code | Code: PHOB |
| Field 23E TELB | Instruction Code | CreditTransferTransactionInformation/InstructionForCreditorAgent/Code | Code: TELB |
| Field 72 /INS/ | Sender to Receiver Info | CreditTransferTransactionInformation/PreviousInstructingAgent | Use MT72INS_To_MXAgent |
| Field 72 /ACC/ | Sender to Receiver Info | CreditTransferTransactionInformation/InstructionForCreditorAgent[ACC] | Use MT_To_MXInstructionForCreditorAgent |
| Field 72 (Other codes) | Sender to Receiver Info | CreditTransferTransactionInformation/InstructionForNextAgent | IF code not in specific list use TR011 |

### Remittance Information
| MT103 Field | Description | pacs.008 Path | Translation Notes |
|-------------|-------------|---------------|-------------------|
| Field 70 | Remittance Information | CreditTransferTransactionInformation/RemittanceInformation/Unstructured | Use MT_To_MXRemittanceInformation |

### Settlement Time Indications
| MT103 Field | Description | pacs.008 Path | Translation Notes |
|-------------|-------------|---------------|-------------------|
| Field 13C SNDTIME | Time Indication | CreditTransferTransactionInformation/SettlementTimeIndication/DebitDateTime | Use MT_To_MXTimeOffset with dummy date "0001-01-01" |
| Field 13C RNCTIME | Time Indication | CreditTransferTransactionInformation/SettlementTimeIndication/CreditDateTime | Use MT_To_MXTimeOffset |
| Field 13C CLSTIME | Time Indication | CreditTransferTransactionInformation/SettlementTimeRequest/CLSTime | Use MT_To_MXTimeOffset |
| Field 13C TILTIME | Time Indication | CreditTransferTransactionInformation/SettlementTimeRequest/TillTime | Use MT_To_MXTimeOffset |
| Field 13C FROTIME | Time Indication | CreditTransferTransactionInformation/SettlementTimeRequest/FromTime | Use MT_To_MXTimeOffset |
| Field 13C REJTIME | Time Indication | CreditTransferTransactionInformation/SettlementTimeRequest/RejectTime | Use MT_To_MXTimeOffset |

### Regulatory Reporting
| MT103 Field | Description | pacs.008 Path | Translation Notes |
|-------------|-------------|---------------|-------------------|
| Field 77B | Regulatory Reporting | CreditTransferTransactionInformation/RegulatoryReporting/Details/Information | Use MT_To_MXRegulatoryReporting |
| Field 77B /BENEFRES/ | Beneficiary Residence | CreditTransferTransactionInformation/Creditor/CountryOfResidence | Extract country code |
| Field 77B /ORDERRES/ | Ordering Residence | CreditTransferTransactionInformation/Debtor/CountryOfResidence | Extract country code |

## Fields Not Translated
| MT103 Field | Description | Reason |
|-------------|-------------|---------|
| Field 23B | Bank Operation Code | No equivalent in pacs.008 |
| Field 23E PHOI/PHON/REPA/TELE/TELI | Instruction Codes | No translation defined |
| CreditTransferTransactionInformation/PaymentIdentification/TransactionIdentification | N/A | No translation from MT103 |

## Translation Functions Reference
- **TR001**: Temporary sender/receiver handling
- **TR002**: Intermediary institution clearing code handling
- **TR003**: Account with institution clearing code handling  
- **TR004**: Sender's charges processing
- **TR005**: Receiver's charges processing
- **TR006**: Service type identifier processing
- **TR009**: Clearing channel processing
- **TR010**: Receiver's correspondent processing
- **TR011**: Instruction for next agent processing
- **TR012**: Category purpose processing
- **TR013**: End-to-end identification processing
- **TR014**: Third reimbursement agent processing
- **TR015**: Account with institution processing
- **METAFCT001**: Meta function for correspondent bank translation

## Important Notes
1. **Mandatory Fields**: Some pacs.008 fields are mandatory even when optional in MT103 - dummy values like "NOTPROVIDED" are used
2. **BIC Validation**: MX Agent only allows FI BIC codes - non-FI BICs are replaced with names
3. **Time Handling**: Dummy date "0001-01-01" is used for time-only fields to create valid DateTime
4. **STP Restrictions**: MT103 STP has restrictions vs. standard MT103 - column Q shows the deltas
5. **Address Requirements**: CBPR+ rules require Name and PostalAddress for agents in international payments when BIC is absent

## METAFCT001 - Meta Function for Settlement Method and Correspondent Banks

### Overview
This Meta function implements the 4 tables translation rules and handles the translation of fields 53a, 54a and 55a for settlement information and correspondent bank processing.

### Input Parameters
- **Field 53a**: Sender's Correspondent (variants A, B, C, D)
- **Field 54a**: Receiver's Correspondent (variants A, B, C, D) 
- **Field 55a**: Third Reimbursement Institution (variants A, B, C, D)
- **Temp~Sender**: Sender BIC from TR001
- **Temp~Receiver**: Receiver BIC from TR001

### Output Parameters
The function generates the following output elements (presence depends on scenario):

#### Settlement Information
- **SettlementMethod**: Method of settlement (CLRG, INDA, INGA, etc.)
- **SettlementAccount**: Account used for settlement

#### Reimbursement Agents
- **InstructingReimbursementAgent**: Agent instructing reimbursement (from field 53a)
- **InstructedReimbursementAgent**: Agent instructed for reimbursement (from field 54a)
- **ThirdReimbursementAgent**: Third reimbursement institution (from field 55a)

#### Reimbursement Agent Accounts
- **InstructingReimbursementAgentAccount**: Account at instructing reimbursement agent
- **InstructedReimbursementAgentAccount**: Account at instructed reimbursement agent
- **ThirdReimbursementAgentAccount**: Account at third reimbursement agent

#### Special Output Parameter
- **InstructionForNextAgentFIN53**: Special instruction parameter with format "/FIN53/BIC(8 or 11)" where the BIC is the BIC of the Institution to which the Receiver will claim the money

### Processing Rules

#### InstructionForNextAgentFIN53 Handling
- **Format**: "/FIN53/BIC(8 or 11)" where BIC is 8 or 11 characters
- **Purpose**: Identifies the institution to which the receiver will claim the money
- **Priority**: Has first priority for translation to pacs.008.InstructionForNextAgent (before codes from Field 72)
- **Initial Value**: Empty string when starting the function
- **Usage**: Must be defined as temporary output parameter from METAFCT001 execution

#### Translation Reference
When METAFCT001 rules reference "translated to [Element] (see sheet MT103 to pacs.008)", it means:
- Follow the specific translation rule defined in the main MT103 to pacs.008 mapping sheet
- Apply the corresponding field-to-field mapping logic as documented in the main specification
- Use the same validation and transformation rules as specified for that element

### Function Execution Flow
1. **Initialize**: Set InstructionForNextAgentFIN53 to empty string
2. **Process Fields**: Analyze fields 53a, 54a, 55a variants and content
3. **Apply Rules**: Execute the 4 tables translation rules based on field combinations
4. **Generate Output**: Populate appropriate output parameters based on scenario
5. **Set Priority**: Ensure InstructionForNextAgentFIN53 takes precedence in instruction processing

### Integration with Main Translation
- METAFCT001 must be executed before TR011 (Field 72 processing)
- Output parameters are used as input for subsequent translation steps
- InstructionForNextAgentFIN53 has priority over Field 72 codes for InstructionForNextAgent mapping 