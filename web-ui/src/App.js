import React, { useState, useEffect } from 'react';
import {
  Container,
  Grid,
  Card,
  Text,
  Title,
  Button,
  Textarea,
  Stack,
  Group,
  Alert,
  Loader,
  Badge,
  Box,
  Progress,
  Transition,
  Paper,
  Tabs,
  Select,
} from '@mantine/core';
import {
  IconTransform,
  IconCode,
  IconCheck,
  IconAlertCircle,
  IconPlayerPlay,
  IconRefresh,
  IconArrowRight,
  IconArrowLeft,
  IconExclamationMark,
  IconCopy,
} from '@tabler/icons-react';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { vscDarkPlus } from 'react-syntax-highlighter/dist/esm/styles/prism';

// API Configuration - using relative URL since we're serving from the same origin
const API_ENDPOINTS = {
  forward: '/transform/mt-to-mx',
  reverse: '/transform/mx-to-mt',
  legacy: '/reframe'
};

// Forward transformation configurations (MT to MX)
const FORWARD_TRANSFORMATIONS = {
  'MT103': {
    name: 'MT103 → ISO 20022 pacs.008.001.08',
    description: 'Customer Credit Transfer',
    targetFormat: 'ISO 20022 pacs.008.001.08 XML',
    sample: `{1:F01BANKBEBBAXXX0000000000}
{2:I103BANKDEFFXXXXN}
{3:{108:MT103}{121:180f1e65-90e0-44d5-a49a-92b55eb3025f}}
{4:
:20:REF123456789
:23B:CRED
:32A:250615EUR123456,78
:50K:/1234567890
John Doe
123 Street
City, Country
:52D:ORDERING BANK NAME
123 BANK STREET
BRUSSELS BELGIUM
:56D:INTERMEDIARY BANK
456 FINANCE AVENUE  
FRANKFURT GERMANY
:59:Jane Smith
456 Avenue
Another City, Country
:70:INVOICE 45678
PAYMENT FOR SERVICES
RENDERED IN DECEMBER
WITH ADDITIONAL NOTES
:71A:OUR
:72:/ACC/MANUAL PROCESSING REQUIRED
/INS/SPECIAL HANDLING NEEDED
REQUIRES COMPLIANCE REVIEW
-}`
  },
  'MT103REJT': {
    name: 'MT103REJT → ISO 20022 pacs.002.001.10',
    description: 'Customer Credit Transfer Rejection',
    targetFormat: 'ISO 20022 pacs.002.001.10 XML',
    sample: `{1:F01DEUTDEFFAXXX0000000000}{2:I103CHASUS33XXXXN}{3:{108:MT103REJT001}{119:STP}{121:12345678-1234-4123-8123-123456789012}}{4:
:20:FT23001234567890
:23B:CRED
:32A:231201USD1000000,00
:50K:/1234567890
ACME CORPORATION
123 MAIN STREET
NEW YORK NY 10001 US
:52A:DEUTDEFFXXX
:57A:CHASUS33XXX
:59:/9876543210
BENEFICIARY COMPANY INC
456 BROADWAY
NEW YORK NY 10013 US
:70:INVOICE PAYMENT REF 2023-INV-001
:71A:OUR
:72:/REJT/
/MREF/FT23001234567890
/TREF/E2E-REF-2023-001
/ReasonCode/AC01
/TEXT/ACCOUNT IDENTIFIER INCORRECT
-}{5:{MAC:12345678}{CHK:123456789ABC}}`
  },
  'MT103RETN': {
    name: 'MT103RETN → ISO 20022 pacs.004.001.09',
    description: 'Customer Credit Transfer Return',
    targetFormat: 'ISO 20022 pacs.004.001.09 XML',
    sample: `{1:F01BANKBEBBAXXX0000000000}
{2:I103BANKDEFFXXXXN}
{3:{113:SEPA}{121:180f1e65-90e0-44d5-a49a-92b55eb3025f}}
{4:
:20:RET123456
:21:REF987654321
:23B:CRED
:32A:250615EUR123456,78
:50K:/1234567890
John Doe
123 Street
City, Country
:59:/9876543210
Jane Smith
456 Avenue
Another City, Country
:70:RETURN OF FUNDS
:71A:OUR
:72:/RETN/INVALID ACCOUNT
-}`
  },
  'MT103STP': {
    name: 'MT103 STP → ISO 20022 pacs.008.001.08',
    description: 'Customer Credit Transfer (STP)',
    targetFormat: 'ISO 20022 pacs.008.001.08 XML',
    sample: `{1:F01CHASUS33AXXX0000000000}{2:I103DEUTDEFFAXXXN}{3:{113:SEPA}{121:180f1e65-90e0-44d5-a49a-92b55eb3025f}}{4:
:13C:/123045+0/+0100/-0500
:20:STP2024123456
:23B:CRED
:23E:INTC/COMPLIANCE
:26T:A01
:32A:241231USD1500000,00
:33B:EUR1375000,00
:36:1,0909
:50K:/1234567890
GLOBAL TECH CORPORATION
456 INNOVATION DRIVE
SAN FRANCISCO CA 94105 US
:52A:CHASUS33
:53A:BNPAFRPP
:54A:DEUTDEFF
:57A:DEUTDEFF
:59A:/DE89370400440532013000
DEUTDEFF
:70:/INV/INVOICE-2024-Q4-789
/RFB/SOFTWARE LICENSE PAYMENT
ENTERPRISE SOFTWARE LICENSES
ANNUAL SUBSCRIPTION RENEWAL
:71A:SHA
:71F:USD50,00
:72:/ACC/STANDARD PROCESSING
/INS/COMPLY WITH LOCAL REGS
AUTOMATED STP PROCESSING
:77B:/ORDERRES/DE//REGULATORY INFO
SOFTWARE LICENSE COMPLIANCE
TRADE RELATED TRANSACTION
-}`
  },
  'MT192': {
    name: 'MT192 → ISO 20022 camt.056.001.08',
    description: 'Request for Cancellation',
    targetFormat: 'ISO 20022 camt.056.001.08 XML',
    sample: `{1:F01BANKBEBBAXXX0000000000}
{2:I192BANKDEFFXXXXN}
{3:{113:CBPR}{121:123e4567-e89b-12d3-a456-426614174000}}
{4:
:20:CANC123456789
:21:ORIG987654321
:11R:pacs.008.001.08,20220625140530
:11S:10324062112341234
:11T:ETOEID20220625001
:13C:/RNCTIME/1405+0200
:32A:220625USD25000,00
:52A:BANKUS33XXX
:57A:BANKGB2LXXX
:79:Customer requested cancellation due to 
duplicate payment instruction. Please 
process immediate cancellation.
-}`
  },
  'MT196': {
    name: 'MT196 → ISO 20022 pacs.002.001.10',
    description: 'Client Side Rejection',
    targetFormat: 'ISO 20022 pacs.002.001.10 XML',
    sample: `{1:F01BANKGB2LXXX0000000000}{2:I196BANKUS33XXXXN}{3:{103:EBA}{108:TRADEREF196}{111:001}{119:STP}{121:87654321-5678-9012-3456-789012345678}}{4:
:20:TR20240101196
:21:TR20240101095
:11R:240101BANKGB2L130800/pacs.008.001.08
:11S:20324062112341234
:11T:E2E196123456789
:13C:/RNCTIME/1455+0000
:32A:240101EUR25000,00
:52A:BANKGB2LXXX
:57A:BANKUS33XXX
:76:ORIGINAL TRANSACTION DETAILS:
//MT202 DATED 240101
//REF: TR20240101095
//AMOUNT: EUR 25000.00
//DEBIT: BANKGB2LXXX
//CREDIT: BANKUS33XXX
:79:INTERBANK TRANSFER CANCELLATION
//REQUEST DUE TO REGULATORY COMPLIANCE
//CUSTOMER REQUESTED IMMEDIATE STOP
//PLEASE CANCEL TRANSACTION
-}`
  },
  'MT202': {
    name: 'MT202 → ISO 20022 pacs.009.001.08',
    description: 'Financial Institution Transfer',
    targetFormat: 'ISO 20022 pacs.009.001.08 XML',
    sample: `{1:F01BANKBEBBXXX0000000000}
{2:I202BANKDEFFXXXN01}
{3:{108:MT202}{121:550e8400-e29b-41d4-a716-446655440000}}
{4:
:20:FI2024123456789
:21:REL2024987654321
:32A:241215USD2500000,00
:52A:BANKBEBBXXX
:53A:BNPAFRPPXXX
:56A:DEUTDEFFXXX
:57A:CHASUS33XXX
:58A:CHASUS33XXX
:72:/ACC/PRIORITY PROCESSING
/INS/BNPAFRPPXXX
INTERBANK SETTLEMENT
HIGH VALUE PAYMENT
-}`
  },
  'MT202COV': {
    name: 'MT202COV → ISO 20022 pacs.009.001.08',
    description: 'Cover Payment',
    targetFormat: 'ISO 20022 pacs.009.001.08 XML',
    sample: `{1:F01BANKBEBBAXXX0000000000}
{2:O2021530080607BANKUS33XXXX00000000000806071530N}
{3:{113:CBPR}{108:COV20240607-001}{121:550e8400-e29b-41d4-a716-446655440000}}
{4:
:20:COV20240607-001
:21:123456789REF
:13C:/RNCTIME/1500+0000
:32A:240607USD5000000,00
:52A:BANKBEBBXXX
:53A:BANKDEFFXXX
:54A:BANKGB22XXX
:56A:BOFAUS3NXXX
:57A:CHASUS33XXX
:58A:BKTRUS33XXX
:50A:/1234567890
ABC Corp
:59A:/0987654321
XYZ Holdings Inc
:72:/INS/BANKBEBBXXX
-}`
  },
  'MT202REJT': {
    name: 'MT202REJT → ISO 20022 pacs.002.001.10',
    description: 'Financial Institution Transfer Rejection',
    targetFormat: 'ISO 20022 pacs.002.001.10 XML',
    sample: `{1:F01DEUTDEFFAXXX0000000000}
{2:I202BANKBEBBXXXXN}
{3:{108:MT202REJT}{121:987fcdeb-51a2-34b5-6789-426614174abc}}
{4:
:20:REJ2024123789456
:21:FI2024987654321
:32A:241218GBP750000,00
:52A:DEUTDEFFXXX
:53D:CORRESPONDENT BANK NAME
MAIN FINANCIAL DISTRICT
FRANKFURT AM MAIN GERMANY
:56D:INTERMEDIARY BANK LTD
456 BANKING STREET
LONDON UNITED KINGDOM
:57D:ACCOUNT WITH INSTITUTION
789 FINANCE AVENUE
BRUSSELS BELGIUM
:58A:BANKBEBBXXX
:72:/REJT/
/MREF/FI2024987654321
/RREF/REJ2024123789456
/AC01/ACCOUNT IDENTIFIER INCORRECT
/TEXT/INVALID CORRESPONDENT ACCOUNT
FAILED DUE TO INCORRECT ACCOUNT
-}`
  },
  'MT202RETN': {
    name: 'MT202RETN → ISO 20022 pacs.004.001.09',
    description: 'Financial Institution Transfer Return',
    targetFormat: 'ISO 20022 pacs.004.001.09 XML',
    sample: `{1:F01BANKBEBBAXXX0000000000}
{2:I202CHASUS33XXXXN}
{3:{108:MT202RETN}{121:456a789b-12c3-45d6-e789-012345678def}}
{4:
:20:RET2024567891234
:21:FI2024123456789
:32A:241219USD950000,00
:52D:ORDERING INSTITUTION NAME
123 BANKING PLAZA
BRUSSELS BELGIUM
:53B:/CH/1234567890
CORRESPONDENT LOCATION INFO
:56A:BNPAFRPPXXX
:57B:/IBAN/GB29BARC20001234567890
BARCLAYS BANK LOCATION
:58A:CHASUS33
:72:/RETN/
/RTRN/INSUFFICIENT FUNDS
/MREF/FI2024123456789
/RREF/RET2024567891234
/RC/AG01/TRANSACTION FORBIDDEN
/TEXT/RETURN DUE TO INSUFFICIENT
NOSTRO ACCOUNT BALANCE
PLEASE ENSURE ADEQUATE FUNDING
-}`
  },
  'MT205': {
    name: 'MT205 → ISO 20022 pacs.010.001.03',
    description: 'Customer Transfer',
    targetFormat: 'ISO 20022 pacs.010.001.03 XML',
    sample: `{1:F01CORPBEBBAXXX0000000000}
{2:I205CORPDEFAXXXXN}
{3:{108:MT205}{121:660e8400-e29b-41d4-a716-446655440001}}
{4:
:20:CP2024123456789
:21:REL2024987654321
:32A:241215USD1750000,00
:52A:CORPBEBBXXX
:58A:CORPDEFAXXX
:72:/INS/DIRECT CORPORATE PAYMENT
/PUR/CORPORATE SERVICES PAYMENT
STANDARD CORPORATE SETTLEMENT
-}`
  },
  'MT205COV': {
    name: 'MT205COV → ISO 20022 pacs.010.001.03',
    description: 'Customer Transfer Cover',
    targetFormat: 'ISO 20022 pacs.010.001.03 XML',
    sample: `{1:F01CORPUS33AXXX0000000000}{2:I205CORPDE55XXXXN}{3:{113:NOMF}{108:COVER002}{119:COV}{121:660e8400-e29b-41d4-a716-446655440002}}{4:
:20:CT220315002
:21:REL220315002
:32A:220315USD2000000,00
:52A:DEUTDEFFXXX
:53A:CORPUS33XXX
:54A:RBOSGGSGXXX
:56A:CITIUS33XXX
:57A:HSBCSGSGXXX
:58A:HSBCSGSGXXX
:50K:/US1234567890123457
GLOBAL CORP SOLUTIONS INC
789 CORPORATE PLAZA
NEW YORK NY 10001
US
:59:/SG56HSBC000012345679
ASIA PACIFIC ENTERPRISES PTE
RAFFLES PLACE TOWER
SINGAPORE 048624
:70:/INV/CORP-INV-2022-0315
/RFB/CORPORATE SERVICE FEE
:72:/BNF/CORPORATE PAYMENT MARCH 2022
/ACC/CORPORATE OPERATIONAL ACCOUNT
:33B:USD2000000,00
-}{5:{CHK:987654321DEF}{TNG:}}`
  },
  'MT292': {
    name: 'MT292 → ISO 20022 camt.056.001.08',
    description: 'Request for Cancellation of Payment',
    targetFormat: 'ISO 20022 camt.056.001.08 XML',
    sample: `{1:F01BANKGB2LXXX0000000000}{2:I292BANKUS33XXXXXN}{3:{103:EBA}{108:TRADEREF123}{111:001}{119:STP}{121:12345678-1234-4567-8901-123456789012}}{4:
:20:TR20240101001
:21:TR20240101000
:11R:240101BANKGB2L130800/pacs.008.001.08
:11S:20224062112341234
:11T:E2E123456789012
:13C:/RNCTIME/2359+0000
:32A:240101USD10000,00
:52A:BANKGB2LXXX
:57A:BANKUS33XXX
:79:REQUEST FOR CANCELLATION DUE TO
//CUSTOMER INSTRUCTION
//DUPLICATE PAYMENT IDENTIFIED
//PLEASE CANCEL IMMEDIATELY
-}`
  },
  'MT296': {
    name: 'MT296 → ISO 20022 camt.056.001.08',
    description: 'Customer Transfer Cancellation Request',
    targetFormat: 'ISO 20022 camt.056.001.08 XML',
    sample: `{1:F01BANKGB2LXXX0000000000}{2:I296BANKUS33XXXXN}{3:{103:EBA}{108:TRADEREF296}{111:001}{119:STP}{121:96543210-8765-4321-9876-543210987654}}{4:
:20:TR20240101296
:21:TR20240101096
:11R:240101BANKGB2L130800/pacs.008.001.08
:11S:20524062112341234
:11T:E2E296987654321
:13C:/RNCTIME/1630+0000
:32A:240101GBP50000,00
:52A:BANKGB2LXXX
:57A:BANKUS33XXX
:76:ORIGINAL TRANSACTION DETAILS:
//MT205 DATED 240101
//REF: TR20240101096
//AMOUNT: GBP 50000.00
//DEBIT: BANKGB2LXXX
//CREDIT: BANKUS33XXX
:79:CUSTOMER TRANSFER CANCELLATION
//REQUEST FROM BENEFICIARY CUSTOMER
//DUPLICATE TRANSACTION DETECTED
//PLEASE CANCEL IMMEDIATELY
-}`
  },
  'MT900': {
    name: 'MT900 → ISO 20022 camt.054.001.08',
    description: 'Confirmation of Debit',
    targetFormat: 'ISO 20022 camt.054.001.08 XML',
    sample: `{1:F01BANKBEBBAXXX0000000000}
{2:I900BANKDEFFXXXXN}
{3:{113:CBPR}{121:123e4567-e89b-12d3-a456-426614174000}}
{4:
:20:C11126A1378
:21:MT10345678901
:25:/1234567890123456
:32A:250622USD12500,00
:13D:2506221015+0530
:52A:BANKUS33XXX
:72:/INS/DEUTDEFFXXX
/ACC/US123456789
-}`
  },
  'MT910': {
    name: 'MT910 → ISO 20022 camt.054.001.08',
    description: 'Confirmation of Credit',
    targetFormat: 'ISO 20022 camt.054.001.08 XML',
    sample: `{1:F01BANKBEBBAXXX0000000000}
{2:I910BANKDEFFXXXXN}
{3:{113:CBPR}{121:123e4567-e89b-12d3-a456-426614174000}}
{4:
:20:C11126A1379
:21:MT10345678902
:25:/1234567890123456
:32A:250622USD12500,00
:13D:2506221015+0530
:52A:BANKUS33XXX
:72:/INS/DEUTDEFFXXX
/ACC/US123456789
-}`
  }
};

// Reverse transformation configurations (MX to MT)
const REVERSE_TRANSFORMATIONS = {
  'pacs.002': {
    name: 'ISO 20022 pacs.002 → MT199/MT299',
    description: 'Payment Status Report (Rejection)',
    targetFormat: 'SWIFT MT199/MT299',
    sample: `<?xml version="1.0" encoding="UTF-8"?>
<Envelope xmlns="urn:swift:xsd:$ahV10">
  <AppHdr>
    <Fr>
      <FIId>
        <FinInstnId>
          <BICFI>DEUTDEFFXXX</BICFI>
        </FinInstnId>
      </FIId>
    </Fr>
    <To>
      <FIId>
        <FinInstnId>
          <BICFI>BANKUS33XXX</BICFI>
        </FinInstnId>
      </FIId>
    </To>
    <BizMsgIdr>20250630-REJT-001</BizMsgIdr>
    <MsgDefIdr>pacs.002.001.10</MsgDefIdr>
    <CreDt>2025-06-30T14:30:00Z</CreDt>
    <BizSvc>CBPR</BizSvc>
  </AppHdr>
  <Document xmlns="urn:iso:std:iso:20022:tech:xsd:pacs.002.001.10">
    <FIToFIPmtStsRpt>
      <GrpHdr>
        <MsgId>REJT20250630001</MsgId>
        <CreDtTm>2025-06-30T14:30:00Z</CreDtTm>
        <InstgAgt>
          <FinInstnId>
            <BICFI>DEUTDEFFXXX</BICFI>
          </FinInstnId>
        </InstgAgt>
        <InstdAgt>
          <FinInstnId>
            <BICFI>BANKUS33XXX</BICFI>
          </FinInstnId>
        </InstdAgt>
        <SttlmInf>
          <SttlmMtd>INDA</SttlmMtd>
          <SttlmAcct>
            <Id>
              <Othr>
                <Id>DE89370400440532013000</Id>
              </Othr>
            </Id>
          </SttlmAcct>
        </SttlmInf>
      </GrpHdr>
      <TxInfAndSts>
        <StsId>STATUS001</StsId>
        <OrgnlGrpInf>
          <OrgnlMsgId>20250627-12345</OrgnlMsgId>
          <OrgnlMsgNmId>pacs.008.001.08</OrgnlMsgNmId>
          <OrgnlCreDtTm>2025-06-27T12:00:00Z</OrgnlCreDtTm>
        </OrgnlGrpInf>
        <OrgnlInstrId>FT21001234567890</OrgnlInstrId>
        <OrgnlEndToEndId>E2E-REF-2025-001</OrgnlEndToEndId>
        <OrgnlTxId>TXN-2025-001234</OrgnlTxId>
        <OrgnlUETR>12345678-1234-4567-8901-123456789012</OrgnlUETR>
        <TxSts>RJCT</TxSts>
        <StsRsnInf>
          <Rsn>
            <Cd>AC01</Cd>
          </Rsn>
          <AddtlInf>ACCOUNT IDENTIFIER INCORRECT</AddtlInf>
          <AddtlInf>BENEFICIARY ACCOUNT NUMBER INVALID</AddtlInf>
        </StsRsnInf>
        <OrgnlTxRef>
          <IntrBkSttlmAmt Ccy="USD">1000000.00</IntrBkSttlmAmt>
          <IntrBkSttlmDt>2025-06-27</IntrBkSttlmDt>
          <InstgAgt>
            <FinInstnId>
              <BICFI>BANKUS33XXX</BICFI>
            </FinInstnId>
          </InstgAgt>
          <InstdAgt>
            <FinInstnId>
              <BICFI>DEUTDEFFXXX</BICFI>
            </FinInstnId>
          </InstdAgt>
          <DbtrAgt>
            <FinInstnId>
              <BICFI>BANKUS33XXX</BICFI>
            </FinInstnId>
          </DbtrAgt>
          <CdtrAgt>
            <FinInstnId>
              <BICFI>DEUTDEFFXXX</BICFI>
            </FinInstnId>
          </CdtrAgt>
          <Dbtr>
            <Nm>ACME CORPORATION</Nm>
            <PstlAdr>
              <StrtNm>123 MAIN STREET</StrtNm>
              <TwnNm>NEW YORK</TwnNm>
              <Ctry>US</Ctry>
            </PstlAdr>
          </Dbtr>
          <DbtrAcct>
            <Id>
              <Othr>
                <Id>1234567890</Id>
              </Othr>
            </Id>
          </DbtrAcct>
          <Cdtr>
            <Nm>MUELLER GMBH</Nm>
            <PstlAdr>
              <StrtNm>HAUPTSTRASSE 1</StrtNm>
              <TwnNm>BERLIN</TwnNm>
              <Ctry>DE</Ctry>
            </PstlAdr>
          </Cdtr>
          <CdtrAcct>
            <Id>
              <IBAN>DE89370400440532013000</IBAN>
            </Id>
          </CdtrAcct>
          <RmtInf>
            <Ustrd>PAYMENT FOR INVOICE 12345</Ustrd>
          </RmtInf>
          <PmtTpInf>
            <SvcLvl>
              <Cd>NURG</Cd>
            </SvcLvl>
            <LclInstrm>
              <Cd>SWIFT</Cd>
            </LclInstrm>
            <CtgyPurp>
              <Cd>SUPP</Cd>
            </CtgyPurp>
          </PmtTpInf>
        </OrgnlTxRef>
        <AccptncDtTm>2025-06-30T14:25:00Z</AccptncDtTm>
        <InstgAgt>
          <FinInstnId>
            <BICFI>DEUTDEFFXXX</BICFI>
          </FinInstnId>
        </InstgAgt>
        <InstdAgt>
          <FinInstnId>
            <BICFI>BANKUS33XXX</BICFI>
          </FinInstnId>
        </InstdAgt>
      </TxInfAndSts>
    </FIToFIPmtStsRpt>
  </Document>
</Envelope>`
  },
  'pacs.002-mt299': {
    name: 'ISO 20022 pacs.002 → MT299',
    description: 'Financial Institution Payment Status Report',
    targetFormat: 'SWIFT MT299',
    sample: `<?xml version="1.0" encoding="UTF-8"?>
<Envelope xmlns="urn:swift:xsd:$ahV10">
  <AppHdr>
    <Fr>
      <FIId>
        <FinInstnId>
          <BICFI>CHASUS33XXX</BICFI>
        </FinInstnId>
      </FIId>
    </Fr>
    <To>
      <FIId>
        <FinInstnId>
          <BICFI>DEUTDEFFXXX</BICFI>
        </FinInstnId>
      </FIId>
    </To>
    <BizMsgIdr>20250630-REJT-202</BizMsgIdr>
    <MsgDefIdr>pacs.002.001.10</MsgDefIdr>
    <CreDt>2025-06-30T15:45:00Z</CreDt>
    <BizSvc>CBPR</BizSvc>
  </AppHdr>
  <Document xmlns="urn:iso:std:iso:20022:tech:xsd:pacs.002.001.10">
    <FIToFIPmtStsRpt>
      <GrpHdr>
        <MsgId>REJT20250630202</MsgId>
        <CreDtTm>2025-06-30T15:45:00Z</CreDtTm>
        <InstgAgt>
          <FinInstnId>
            <BICFI>CHASUS33XXX</BICFI>
          </FinInstnId>
        </InstgAgt>
        <InstdAgt>
          <FinInstnId>
            <BICFI>DEUTDEFFXXX</BICFI>
          </FinInstnId>
        </InstdAgt>
        <SttlmInf>
          <SttlmMtd>INDA</SttlmMtd>
          <SttlmAcct>
            <Id>
              <IBAN>US12345678901234567890</IBAN>
            </Id>
          </SttlmAcct>
        </SttlmInf>
      </GrpHdr>
      <TxInfAndSts>
        <StsId>STATUS202</StsId>
        <OrgnlGrpInf>
          <OrgnlMsgId>20250630-MT202-001</OrgnlMsgId>
          <OrgnlMsgNmId>pacs.009.001.08</OrgnlMsgNmId>
          <OrgnlCreDtTm>2025-06-30T10:00:00Z</OrgnlCreDtTm>
        </OrgnlGrpInf>
        <OrgnlInstrId>202-FI-2025-001</OrgnlInstrId>
        <OrgnlEndToEndId>E2E-FI-2025-001</OrgnlEndToEndId>
        <OrgnlTxId>TXN-FI-2025-001</OrgnlTxId>
        <OrgnlUETR>87654321-4321-7654-2109-987654321098</OrgnlUETR>
        <TxSts>RJCT</TxSts>
        <StsRsnInf>
          <Rsn>
            <Cd>AM04</Cd>
          </Rsn>
          <AddtlInf>INSUFFICIENT FUNDS</AddtlInf>
          <AddtlInf>SETTLEMENT ACCOUNT BALANCE INSUFFICIENT</AddtlInf>
        </StsRsnInf>
        <OrgnlTxRef>
          <IntrBkSttlmAmt Ccy="EUR">2500000.00</IntrBkSttlmAmt>
          <IntrBkSttlmDt>2025-06-30</IntrBkSttlmDt>
          <InstgAgt>
            <FinInstnId>
              <BICFI>DEUTDEFFXXX</BICFI>
            </FinInstnId>
          </InstgAgt>
          <InstdAgt>
            <FinInstnId>
              <BICFI>CHASUS33XXX</BICFI>
            </FinInstnId>
          </InstdAgt>
          <Dbtr>
            <FinInstnId>
              <BICFI>DEUTDEFFXXX</BICFI>
              <Nm>DEUTSCHE BANK AG</Nm>
              <PstlAdr>
                <StrtNm>TAUNUSANLAGE 12</StrtNm>
                <TwnNm>FRANKFURT AM MAIN</TwnNm>
                <Ctry>DE</Ctry>
              </PstlAdr>
            </FinInstnId>
          </Dbtr>
          <DbtrAcct>
            <Id>
              <Othr>
                <Id>DE89370400440532013000</Id>
              </Othr>
            </Id>
          </DbtrAcct>
          <Cdtr>
            <FinInstnId>
              <BICFI>CHASUS33XXX</BICFI>
              <Nm>JPMORGAN CHASE BANK</Nm>
              <PstlAdr>
                <StrtNm>270 PARK AVENUE</StrtNm>
                <TwnNm>NEW YORK</TwnNm>
                <Ctry>US</Ctry>
              </PstlAdr>
            </FinInstnId>
          </Cdtr>
          <CdtrAcct>
            <Id>
              <Othr>
                <Id>US12345678901234567890</Id>
              </Othr>
            </Id>
          </CdtrAcct>
          <PmtTpInf>
            <SvcLvl>
              <Cd>NURG</Cd>
            </SvcLvl>
            <LclInstrm>
              <Cd>WIRE</Cd>
            </LclInstrm>
            <CtgyPurp>
              <Cd>INTC</Cd>
            </CtgyPurp>
          </PmtTpInf>
          <RmtInf>
            <Ustrd>/INS/DEUTDEFFXXX</Ustrd>
            <Ustrd>/INT/CHASUS33XXX</Ustrd>
            <Ustrd>/SVCLVL/NURG</Ustrd>
          </RmtInf>
        </OrgnlTxRef>
        <AccptncDtTm>2025-06-30T15:40:00Z</AccptncDtTm>
        <InstgAgt>
          <FinInstnId>
            <BICFI>CHASUS33XXX</BICFI>
          </FinInstnId>
        </InstgAgt>
        <InstdAgt>
          <FinInstnId>
            <BICFI>DEUTDEFFXXX</BICFI>
          </FinInstnId>
        </InstdAgt>
      </TxInfAndSts>
    </FIToFIPmtStsRpt>
  </Document>
</Envelope>`
  },
  'pacs.004': {
    name: 'ISO 20022 pacs.004 → MT103/MT202/MT205',
    description: 'Payment Return/Refund',
    targetFormat: 'SWIFT MT103/MT202/MT205',
    sample: `<?xml version="1.0" encoding="UTF-8"?>
<Envelope xmlns="urn:swift:xsd:$ahV10">
  <AppHdr>
    <Fr>
      <FIId>
        <FinInstnId>
          <BICFI>BANKDEFFXXX</BICFI>
        </FinInstnId>
      </FIId>
    </Fr>
    <To>
      <FIId>
        <FinInstnId>
          <BICFI>BANKUS33XXX</BICFI>
        </FinInstnId>
      </FIId>
    </To>
    <BizMsgIdr>RETURN20250627-001</BizMsgIdr>
    <MsgDefIdr>pacs.004.001.09</MsgDefIdr>
    <CreDt>2025-06-27T16:45:00Z</CreDt>
    <BizSvc>swift.cbprplus.02</BizSvc>
    <CpyDplct>CODU</CpyDplct>
    <PssblDplct>false</PssblDplct>
  </AppHdr>
  <Document xmlns="urn:iso:std:iso:20022:tech:xsd:pacs.004.001.09">
    <PmtRtr>
      <GrpHdr>
        <MsgId>RETURN20250627-001</MsgId>
        <CreDtTm>2025-06-27T16:45:00Z</CreDtTm>
        <NbOfTxs>1</NbOfTxs>
        <TtlRtrdIntrBkSttlmAmt Ccy="EUR">75000.00</TtlRtrdIntrBkSttlmAmt>
        <SttlmInf>
          <SttlmMtd>INDA</SttlmMtd>
          <SttlmAcct>
            <Id>
              <Othr>
                <Id>SETTLEMENT-RTN-123</Id>
              </Othr>
            </Id>
          </SttlmAcct>
        </SttlmInf>
        <InstgAgt>
          <FinInstnId>
            <BICFI>BANKDEFFXXX</BICFI>
            <Nm>Deutsche Bank AG</Nm>
            <PstlAdr>
              <StrtNm>Taunusanlage 12</StrtNm>
              <TwnNm>Frankfurt am Main</TwnNm>
              <Ctry>DE</Ctry>
              <PstCd>60325</PstCd>
            </PstlAdr>
          </FinInstnId>
        </InstgAgt>
        <InstdAgt>
          <FinInstnId>
            <BICFI>BANKUS33XXX</BICFI>
            <Nm>US Bank National Association</Nm>
            <PstlAdr>
              <StrtNm>425 Walnut Street</StrtNm>
              <TwnNm>Cincinnati</TwnNm>
              <Ctry>US</Ctry>
              <PstCd>45202</PstCd>
            </PstlAdr>
          </FinInstnId>
        </InstdAgt>
      </GrpHdr>
      <TxInf>
        <RtrId>RTN-TX-20250627-001</RtrId>
        <OrgnlGrpInf>
          <OrgnlMsgId>CREDIT-TX-20250626-789</OrgnlMsgId>
          <OrgnlMsgNmId>pacs.008.001.08</OrgnlMsgNmId>
          <OrgnlCreDtTm>2025-06-26T14:30:00Z</OrgnlCreDtTm>
          <OrgnlNbOfTxs>1</OrgnlNbOfTxs>
        </OrgnlGrpInf>
        <OrgnlInstrId>INSTR-ID-20250626-456</OrgnlInstrId>
        <OrgnlEndToEndId>E2E-PAYMENT-REF-20250626-123</OrgnlEndToEndId>
        <OrgnlTxId>TX-ID-20250626-789</OrgnlTxId>
        <OrgnlUETR>550e8400-e29b-41d4-a716-446655440001</OrgnlUETR>
        <OrgnlClrSysRef>RTGS-REF-DE-20250626-888</OrgnlClrSysRef>
        <OrgnlIntrBkSttlmAmt Ccy="EUR">75000.00</OrgnlIntrBkSttlmAmt>
        <OrgnlIntrBkSttlmDt>2025-06-26</OrgnlIntrBkSttlmDt>
        <RtrdIntrBkSttlmAmt Ccy="EUR">75000.00</RtrdIntrBkSttlmAmt>
        <IntrBkSttlmDt>2025-06-27</IntrBkSttlmDt>
        <SttlmPrty>NORM</SttlmPrty>
        <SttlmTmIndctn>
          <DbtDtTm>2025-06-27T09:00:00Z</DbtDtTm>
          <CdtDtTm>2025-06-27T17:00:00Z</CdtDtTm>
        </SttlmTmIndctn>
        <RtrdInstdAmt Ccy="USD">82500.00</RtrdInstdAmt>
        <XchgRate>1.1</XchgRate>
        <ChrgBr>SHAR</ChrgBr>
        <ChrgsInf>
          <Amt Ccy="EUR">50.00</Amt>
          <Agt>
            <FinInstnId>
              <BICFI>BANKDEFFXXX</BICFI>
            </FinInstnId>
          </Agt>
          <Tp>
            <Cd>DEBT</Cd>
          </Tp>
        </ChrgsInf>
        <ClrSysRef>RETURN-REF-DE-20250627-999</ClrSysRef>
        <InstgAgt>
          <FinInstnId>
            <BICFI>BANKDEFFXXX</BICFI>
            <ClrSysMmbId>
              <ClrSysId>
                <Cd>DEBLZ</Cd>
              </ClrSysId>
              <MmbId>50010517</MmbId>
            </ClrSysMmbId>
            <Nm>Deutsche Bank AG</Nm>
            <PstlAdr>
              <StrtNm>Taunusanlage 12</StrtNm>
              <TwnNm>Frankfurt am Main</TwnNm>
              <Ctry>DE</Ctry>
              <PstCd>60325</PstCd>
            </PstlAdr>
          </FinInstnId>
        </InstgAgt>
        <InstdAgt>
          <FinInstnId>
            <BICFI>BANKUS33XXX</BICFI>
            <ClrSysMmbId>
              <ClrSysId>
                <Cd>USABA</Cd>
              </ClrSysId>
              <MmbId>021000021</MmbId>
            </ClrSysMmbId>
            <Nm>US Bank National Association</Nm>
            <PstlAdr>
              <StrtNm>425 Walnut Street</StrtNm>
              <TwnNm>Cincinnati</TwnNm>
              <Ctry>US</Ctry>
              <PstCd>45202</PstCd>
            </PstlAdr>
          </FinInstnId>
        </InstdAgt>
        <RtrChain>
          <UltmtDbtr>
            <Nm>Ultimate Payer Corporation</Nm>
            <PstlAdr>
              <StrtNm>100 Corporate Plaza</StrtNm>
              <TwnNm>New York</TwnNm>
              <Ctry>US</Ctry>
              <PstCd>10001</PstCd>
            </PstlAdr>
            <Id>
              <OrgId>
                <LEI>549300ULTPAYER123456</LEI>
              </OrgId>
            </Id>
          </UltmtDbtr>
          <Dbtr>
            <Pty>
              <Nm>Corporate Services Inc</Nm>
              <PstlAdr>
                <StrtNm>456 Business Avenue</StrtNm>
                <TwnNm>New York</TwnNm>
                <Ctry>US</Ctry>
                <PstCd>10002</PstCd>
              </PstlAdr>
              <Id>
                <OrgId>
                  <Othr>
                    <Id>CORP-SERVICES-001</Id>
                    <SchmeNm>
                      <Cd>CUST</Cd>
                    </SchmeNm>
                  </Othr>
                </OrgId>
              </Id>
              <CtryOfRes>US</CtryOfRes>
            </Pty>
            <Agt>
              <FinInstnId>
                <BICFI>BANKUS33XXX</BICFI>
                <ClrSysMmbId>
                  <ClrSysId>
                    <Cd>USABA</Cd>
                  </ClrSysId>
                  <MmbId>021000021</MmbId>
                </ClrSysMmbId>
                <Nm>US Bank National Association</Nm>
                <PstlAdr>
                  <StrtNm>425 Walnut Street</StrtNm>
                  <TwnNm>Cincinnati</TwnNm>
                  <Ctry>US</Ctry>
                  <PstCd>45202</PstCd>
                </PstlAdr>
              </FinInstnId>
            </Agt>
          </Dbtr>
          <DbtrAgt>
            <FinInstnId>
              <BICFI>BANKUS33XXX</BICFI>
              <ClrSysMmbId>
                <ClrSysId>
                  <Cd>USABA</Cd>
                </ClrSysId>
                <MmbId>021000021</MmbId>
              </ClrSysMmbId>
              <Nm>US Bank National Association</Nm>
              <PstlAdr>
                <StrtNm>425 Walnut Street</StrtNm>
                <TwnNm>Cincinnati</TwnNm>
                <Ctry>US</Ctry>
                <PstCd>45202</PstCd>
              </PstlAdr>
            </FinInstnId>
          </DbtrAgt>
          <IntrmyAgt1>
            <FinInstnId>
              <BICFI>INTRMGB2LXXX</BICFI>
              <ClrSysMmbId>
                <ClrSysId>
                  <Cd>GBDSC</Cd>
                </ClrSysId>
                <MmbId>123456</MmbId>
              </ClrSysMmbId>
              <Nm>Intermediary Bank London</Nm>
              <PstlAdr>
                <StrtNm>789 Clearing Street</StrtNm>
                <TwnNm>London</TwnNm>
                <Ctry>GB</Ctry>
                <PstCd>EC1A 1BB</PstCd>
              </PstlAdr>
            </FinInstnId>
          </IntrmyAgt1>
          <CdtrAgt>
            <FinInstnId>
              <BICFI>BANKDEFFXXX</BICFI>
              <ClrSysMmbId>
                <ClrSysId>
                  <Cd>DEBLZ</Cd>
                </ClrSysId>
                <MmbId>50010517</MmbId>
              </ClrSysMmbId>
              <Nm>Deutsche Bank AG</Nm>
              <PstlAdr>
                <StrtNm>Taunusanlage 12</StrtNm>
                <TwnNm>Frankfurt am Main</TwnNm>
                <Ctry>DE</Ctry>
                <PstCd>60325</PstCd>
              </PstlAdr>
            </FinInstnId>
          </CdtrAgt>
          <Cdtr>
            <Pty>
              <Nm>European Technology GmbH</Nm>
              <PstlAdr>
                <StrtNm>321 Technology Park</StrtNm>
                <TwnNm>Frankfurt</TwnNm>
                <Ctry>DE</Ctry>
                <PstCd>60311</PstCd>
              </PstlAdr>
              <Id>
                <OrgId>
                  <LEI>549300EUROTECH654321</LEI>
                </OrgId>
              </Id>
              <CtryOfRes>DE</CtryOfRes>
            </Pty>
            <Agt>
              <FinInstnId>
                <BICFI>BANKDEFFXXX</BICFI>
                <ClrSysMmbId>
                  <ClrSysId>
                    <Cd>DEBLZ</Cd>
                  </ClrSysId>
                  <MmbId>50010517</MmbId>
                </ClrSysMmbId>
                <Nm>Deutsche Bank AG</Nm>
                <PstlAdr>
                  <StrtNm>Taunusanlage 12</StrtNm>
                  <TwnNm>Frankfurt am Main</TwnNm>
                  <Ctry>DE</Ctry>
                  <PstCd>60325</PstCd>
                </PstlAdr>
              </FinInstnId>
            </Agt>
          </Cdtr>
          <UltmtCdtr>
            <Nm>Ultimate Beneficiary Holdings AG</Nm>
            <PstlAdr>
              <StrtNm>654 Holdings Street</StrtNm>
              <TwnNm>Munich</TwnNm>
              <Ctry>DE</Ctry>
              <PstCd>80331</PstCd>
            </PstlAdr>
            <Id>
              <OrgId>
                <LEI>549300ULTBENEF987654</LEI>
              </OrgId>
            </Id>
          </UltmtCdtr>
        </RtrChain>
        <RtrRsnInf>
          <Orgtr>
            <Nm>Return Processing System</Nm>
            <Id>
              <OrgId>
                <Othr>
                  <Id>RTN-SYS-001</Id>
                  <SchmeNm>
                    <Cd>CUST</Cd>
                  </SchmeNm>
                </Othr>
              </OrgId>
            </Id>
          </Orgtr>
          <Rsn>
            <Cd>AM05</Cd>
          </Rsn>
          <AddtlInf>Duplication - payment already processed with same end-to-end reference</AddtlInf>
          <AddtlInf>Customer requested return due to duplicate processing</AddtlInf>
        </RtrRsnInf>
        <OrgnlTxRef>
          <IntrBkSttlmAmt Ccy="EUR">75000.00</IntrBkSttlmAmt>
          <IntrBkSttlmDt>2025-06-26</IntrBkSttlmDt>
          <InstdAmt Ccy="USD">82500.00</InstdAmt>
          <XchgRate>1.1</XchgRate>
          <ChrgBr>SHAR</ChrgBr>
          <PmtTpInf>
            <InstrPrty>NORM</InstrPrty>
            <ClrChanl>RTGS</ClrChanl>
            <SvcLvl>
              <Cd>G001</Cd>
            </SvcLvl>
            <LclInstrm>
              <Cd>WIRE</Cd>
            </LclInstrm>
            <CtgyPurp>
              <Cd>INTC</Cd>
            </CtgyPurp>
          </PmtTpInf>
          <PmtMtd>TRA</PmtMtd>
          <RmtInf>
            <Ustrd>Return of Payment for Invoice INV-2025-67890 - Duplicate processing detected</Ustrd>
          </RmtInf>
          <UltmtDbtr>
            <Nm>Ultimate Payer Corporation</Nm>
            <PstlAdr>
              <StrtNm>100 Corporate Plaza</StrtNm>
              <TwnNm>New York</TwnNm>
              <Ctry>US</Ctry>
              <PstCd>10001</PstCd>
            </PstlAdr>
            <Id>
              <OrgId>
                <LEI>549300ULTPAYER123456</LEI>
              </OrgId>
            </Id>
          </UltmtDbtr>
          <Dbtr>
            <Nm>Corporate Services Inc</Nm>
            <PstlAdr>
              <StrtNm>456 Business Avenue</StrtNm>
              <TwnNm>New York</TwnNm>
              <Ctry>US</Ctry>
              <PstCd>10002</PstCd>
            </PstlAdr>
            <Id>
              <OrgId>
                <Othr>
                  <Id>CORP-SERVICES-001</Id>
                  <SchmeNm>
                    <Cd>CUST</Cd>
                  </SchmeNm>
                </Othr>
              </OrgId>
            </Id>
            <CtryOfRes>US</CtryOfRes>
          </Dbtr>
          <DbtrAcct>
            <Id>
              <Othr>
                <Id>US-CORP-ACC-123456789</Id>
                <SchmeNm>
                  <Cd>BBAN</Cd>
                </SchmeNm>
              </Othr>
            </Id>
            <Tp>
              <Cd>CACC</Cd>
            </Tp>
            <Ccy>USD</Ccy>
            <Nm>Corporate Services Operating Account</Nm>
          </DbtrAcct>
          <DbtrAgt>
            <FinInstnId>
              <BICFI>BANKUS33XXX</BICFI>
            </FinInstnId>
          </DbtrAgt>
          <CdtrAgt>
            <FinInstnId>
              <BICFI>BANKDEFFXXX</BICFI>
            </FinInstnId>
          </CdtrAgt>
          <Cdtr>
            <Nm>European Technology GmbH</Nm>
            <PstlAdr>
              <StrtNm>321 Technology Park</StrtNm>
              <TwnNm>Frankfurt</TwnNm>
              <Ctry>DE</Ctry>
              <PstCd>60311</PstCd>
            </PstlAdr>
            <Id>
              <OrgId>
                <LEI>549300EUROTECH654321</LEI>
              </OrgId>
            </Id>
            <CtryOfRes>DE</CtryOfRes>
          </Cdtr>
          <CdtrAcct>
            <Id>
              <Othr>
                <Id>DE89370400440532013001</Id>
                <SchmeNm>
                  <Cd>IBAN</Cd>
                </SchmeNm>
              </Othr>
            </Id>
            <Tp>
              <Cd>CACC</Cd>
            </Tp>
            <Ccy>EUR</Ccy>
            <Nm>European Technology Operating Account</Nm>
          </CdtrAcct>
          <UltmtCdtr>
            <Nm>Ultimate Beneficiary Holdings AG</Nm>
            <PstlAdr>
              <StrtNm>654 Holdings Street</StrtNm>
              <TwnNm>Munich</TwnNm>
              <Ctry>DE</Ctry>
              <PstCd>80331</PstCd>
            </PstlAdr>
            <Id>
              <OrgId>
                <LEI>549300ULTBENEF987654</LEI>
              </OrgId>
            </Id>
          </UltmtCdtr>
          <Purp>
            <Cd>CBFF</Cd>
          </Purp>
        </OrgnlTxRef>
      </TxInf>
    </PmtRtr>
  </Document>
</Envelope>`
  },
  'pacs.008': {
    name: 'ISO 20022 pacs.008 → MT103',
    description: 'Customer Credit Transfer',
    targetFormat: 'SWIFT MT103',
    sample: `<?xml version="1.0" encoding="UTF-8"?>
<Envelope xmlns="urn:swift:xsd:$ahV10">
  <AppHdr>
    <Fr>
      <FIId>
        <FinInstnId>
          <BICFI>BANKUS33XXX</BICFI>
        </FinInstnId>
      </FIId>
    </Fr>
    <To>
      <FIId>
        <FinInstnId>
          <BICFI>BANKDEFFXXX</BICFI>
        </FinInstnId>
      </FIId>
    </To>
    <BizMsgIdr>20250627-12345</BizMsgIdr>
    <MsgDefIdr>pacs.008.001.08</MsgDefIdr>
    <CreDt>2025-06-27T12:00:00Z</CreDt>
    <BizSvc>CBPR</BizSvc>
  </AppHdr>
  <Document xmlns="urn:iso:std:iso:20022:tech:xsd:pacs.008.001.08">
    <FIToFICstmrCdtTrf>
      <GrpHdr>
        <MsgId>20250627-12345</MsgId>
        <CreDtTm>2025-06-27T12:00:00Z</CreDtTm>
        <NbOfTxs>1</NbOfTxs>
        <SttlmInf>
          <SttlmMtd>INDA</SttlmMtd>
          <SttlmAcct>
            <Id>
              <Othr>
                <Id>SETTLEMENT-ACCOUNT-123</Id>
              </Othr>
            </Id>
          </SttlmAcct>
          <InstgRmbrsmntAgt>
            <FinInstnId>
              <BICFI>INTRMDUS33X</BICFI>
              <ClrSysMmbId>
                <ClrSysId>
                  <Cd>USABA</Cd>
                </ClrSysId>
                <MmbId>123456789</MmbId>
              </ClrSysMmbId>
              <LEI>549300ABCDEF1234567890</LEI>
              <Nm>Instructing Reimbursement Agent</Nm>
              <PstlAdr>
                <StrtNm>123 Financial Street</StrtNm>
                <TwnNm>New York</TwnNm>
                <Ctry>US</Ctry>
                <AdrLine>Suite 100</AdrLine>
                <AdrLine>Manhattan District</AdrLine>
              </PstlAdr>
            </FinInstnId>
          </InstgRmbrsmntAgt>
          <InstgRmbrsmntAgtAcct>
            <Id>
              <Othr>
                <Id>REIMBURSEMENT-ACC-456</Id>
              </Othr>
            </Id>
          </InstgRmbrsmntAgtAcct>
          <InstdRmbrsmntAgt>
            <FinInstnId>
              <BICFI>INTRMDDEFXXX</BICFI>
              <ClrSysMmbId>
                <ClrSysId>
                  <Cd>DEBLZ</Cd>
                </ClrSysId>
                <MmbId>50010517</MmbId>
              </ClrSysMmbId>
              <LEI>549300FEDCBA0987654321</LEI>
              <Nm>Instructed Reimbursement Agent</Nm>
              <PstlAdr>
                <StrtNm>456 Banking Avenue</StrtNm>
                <TwnNm>Frankfurt</TwnNm>
                <Ctry>DE</Ctry>
                <AdrLine>Floor 10</AdrLine>
              </PstlAdr>
            </FinInstnId>
          </InstdRmbrsmntAgt>
          <InstdRmbrsmntAgtAcct>
            <Id>
              <Othr>
                <Id>REIMBURSEMENT-ACC-789</Id>
              </Othr>
            </Id>
          </InstdRmbrsmntAgtAcct>
        </SttlmInf>
      </GrpHdr>
      <CdtTrfTxInf>
        <PmtId>
          <InstrId>INSTR123456</InstrId>
          <EndToEndId>550e8400-e29b-41d4-a716-446655440000</EndToEndId>
          <TxId>TX123456</TxId>
          <UETR>550e8400-e29b-41d4-a716-446655440000</UETR>
          <ClrSysRef>CLEAR-REF-789</ClrSysRef>
        </PmtId>
        <PmtTpInf>
          <InstrPrty>NORM</InstrPrty>
          <ClrChanl>RTGS</ClrChanl>
          <SvcLvl>
            <Cd>SDVA</Cd>
          </SvcLvl>
          <LclInstrm>
            <Cd>LOCINS123</Cd>
          </LclInstrm>
          <CtgyPurp>
            <Cd>INTC</Cd>
          </CtgyPurp>
        </PmtTpInf>
        <IntrBkSttlmAmt Ccy="EUR">1000.00</IntrBkSttlmAmt>
        <IntrBkSttlmDt>2025-06-27</IntrBkSttlmDt>
        <SttlmPrty>NORM</SttlmPrty>
        <SttlmTmIndctn>
          <DbtDtTm>2025-06-27T09:00:00Z</DbtDtTm>
          <CdtDtTm>2025-06-27T15:00:00Z</CdtDtTm>
        </SttlmTmIndctn>
        <SttlmTmReq>
          <CLSTm>16:00:00</CLSTm>
          <TillTm>17:00:00</TillTm>
          <FrTm>08:00:00</FrTm>
          <RjctTm>18:00:00</RjctTm>
        </SttlmTmReq>
        <InstdAmt Ccy="USD">1100.00</InstdAmt>
        <XchgRate>1.1</XchgRate>
        <ChrgBr>SHAR</ChrgBr>
        <ChrgsInf>
          <Amt Ccy="EUR">5.00</Amt>
          <Agt>
            <FinInstnId>
              <BICFI>CHRGUS33XXX</BICFI>
            </FinInstnId>
          </Agt>
        </ChrgsInf>
        <ChrgsInf>
          <Amt Ccy="EUR">10.00</Amt>
          <Agt>
            <FinInstnId>
              <BICFI>RCVGDEFFXXX</BICFI>
            </FinInstnId>
          </Agt>
        </ChrgsInf>
        <PrvsInstgAgt1>
          <FinInstnId>
            <BICFI>PREVUS33XXX</BICFI>
          </FinInstnId>
        </PrvsInstgAgt1>
        <PrvsInstgAgt1Acct>
          <Id>
            <Othr>
              <Id>PREV-ACC-001</Id>
            </Othr>
          </Id>
        </PrvsInstgAgt1Acct>
        <InstgAgt>
          <FinInstnId>
            <BICFI>BANKUS33XXX</BICFI>
          </FinInstnId>
        </InstgAgt>
        <InstdAgt>
          <FinInstnId>
            <BICFI>BANKDEFFXXX</BICFI>
          </FinInstnId>
        </InstdAgt>
        <IntrmyAgt1>
          <FinInstnId>
            <BICFI>INTRMGB2LXXX</BICFI>
            <ClrSysMmbId>
              <ClrSysId>
                <Cd>GBDSC</Cd>
              </ClrSysId>
              <MmbId>123456</MmbId>
            </ClrSysMmbId>
            <LEI>549300INTERMEDIATE01</LEI>
            <Nm>First Intermediary Bank</Nm>
            <PstlAdr>
              <StrtNm>789 Clearing Street</StrtNm>
              <TwnNm>London</TwnNm>
              <Ctry>GB</Ctry>
              <AdrLine>Level 5</AdrLine>
            </PstlAdr>
          </FinInstnId>
        </IntrmyAgt1>
        <IntrmyAgt1Acct>
          <Id>
            <Othr>
              <Id>INTER-ACC-123</Id>
            </Othr>
          </Id>
        </IntrmyAgt1Acct>
        <IntrmyAgt2>
          <FinInstnId>
            <BICFI>INTRMCHZZXXX</BICFI>
            <ClrSysMmbId>
              <ClrSysId>
                <Cd>CHBCC</Cd>
              </ClrSysId>
              <MmbId>987654</MmbId>
            </ClrSysMmbId>
            <Nm>Second Intermediary Bank</Nm>
            <PstlAdr>
              <StrtNm>321 Banking Plaza</StrtNm>
              <TwnNm>Zurich</TwnNm>
              <Ctry>CH</Ctry>
            </PstlAdr>
          </FinInstnId>
        </IntrmyAgt2>
        <IntrmyAgt2Acct>
          <Id>
            <Othr>
              <Id>INTER-ACC-456</Id>
            </Othr>
          </Id>
        </IntrmyAgt2Acct>
        <UltmtDbtr>
          <Nm>Ultimate Debtor Corp</Nm>
          <PstlAdr>
            <StrtNm>100 Corporate Drive</StrtNm>
            <TwnNm>Chicago</TwnNm>
            <Ctry>US</Ctry>
          </PstlAdr>
          <Id>
            <OrgId>
              <LEI>549300ULTIMATE123456</LEI>
              <Othr>
                <Id>CORP-ID-789</Id>
                <SchmeNm>
                  <Cd>TXID</Cd>
                </SchmeNm>
              </Othr>
            </OrgId>
          </Id>
          <CtryOfRes>US</CtryOfRes>
        </UltmtDbtr>
        <InitgPty>
          <Nm>Initiating Party Ltd</Nm>
          <PstlAdr>
            <StrtNm>200 Initiator Street</StrtNm>
            <TwnNm>Boston</TwnNm>
            <Ctry>US</Ctry>
          </PstlAdr>
          <Id>
            <OrgId>
              <LEI>549300INITIATING001</LEI>
            </OrgId>
          </Id>
        </InitgPty>
        <Dbtr>
          <Nm>Jane Smith</Nm>
          <PstlAdr>
            <StrtNm>123 Main Street</StrtNm>
            <TwnNm>New York</TwnNm>
            <Ctry>US</Ctry>
            <AdrLine>Apartment 4B</AdrLine>
          </PstlAdr>
          <Id>
            <PrvtId>
              <DtAndPlcOfBirth>
                <BirthDt>1980-05-15</BirthDt>
                <CityOfBirth>New York</CityOfBirth>
                <CtryOfBirth>US</CtryOfBirth>
              </DtAndPlcOfBirth>
              <Othr>
                <Id>SSN123456789</Id>
                <SchmeNm>
                  <Cd>SOSE</Cd>
                </SchmeNm>
              </Othr>
            </PrvtId>
          </Id>
          <CtryOfRes>US</CtryOfRes>
        </Dbtr>
        <DbtrAcct>
          <Id>
            <Othr>
              <Id>ACC-US-123456789</Id>
              <SchmeNm>
                <Cd>BBAN</Cd>
              </SchmeNm>
            </Othr>
          </Id>
          <Tp>
            <Cd>SVGS</Cd>
          </Tp>
          <Ccy>USD</Ccy>
          <Nm>Jane Smith Savings Account</Nm>
        </DbtrAcct>
        <DbtrAgt>
          <FinInstnId>
            <BICFI>BANKUS33XXX</BICFI>
            <ClrSysMmbId>
              <ClrSysId>
                <Cd>USABA</Cd>
              </ClrSysId>
              <MmbId>021000021</MmbId>
            </ClrSysMmbId>
            <LEI>549300DEBTORBANK001</LEI>
            <Nm>US Debtor Bank</Nm>
            <PstlAdr>
              <StrtNm>456 Banking Street</StrtNm>
              <TwnNm>New York</TwnNm>
              <Ctry>US</Ctry>
            </PstlAdr>
          </FinInstnId>
        </DbtrAgt>
        <DbtrAgtAcct>
          <Id>
            <Othr>
              <Id>DBTR-AGENT-ACC-123</Id>
            </Othr>
          </Id>
        </DbtrAgtAcct>
        <CdtrAgt>
          <FinInstnId>
            <BICFI>BANKDEFFXXX</BICFI>
            <ClrSysMmbId>
              <ClrSysId>
                <Cd>DEBLZ</Cd>
              </ClrSysId>
              <MmbId>50010517</MmbId>
            </ClrSysMmbId>
            <LEI>549300CREDITORBANK1</LEI>
            <Nm>German Creditor Bank</Nm>
            <PstlAdr>
              <StrtNm>789 Deutsche Straße</StrtNm>
              <TwnNm>Frankfurt</TwnNm>
              <Ctry>DE</Ctry>
            </PstlAdr>
          </FinInstnId>
          <BrnchId>
            <Id>BRANCH-001</Id>
          </BrnchId>
        </CdtrAgt>
        <CdtrAgtAcct>
          <Id>
            <Othr>
              <Id>CDTR-AGENT-ACC-456</Id>
            </Othr>
          </Id>
        </CdtrAgtAcct>
        <Cdtr>
          <Nm>John Doe</Nm>
          <PstlAdr>
            <StrtNm>987 Recipient Avenue</StrtNm>
            <TwnNm>Frankfurt</TwnNm>
            <Ctry>DE</Ctry>
            <AdrLine>Building C</AdrLine>
          </PstlAdr>
          <Id>
            <PrvtId>
              <DtAndPlcOfBirth>
                <BirthDt>1975-12-10</BirthDt>
                <CityOfBirth>Berlin</CityOfBirth>
                <CtryOfBirth>DE</CtryOfBirth>
              </DtAndPlcOfBirth>
              <Othr>
                <Id>DE-ID-987654321</Id>
                <SchmeNm>
                  <Cd>NRIN</Cd>
                </SchmeNm>
              </Othr>
            </PrvtId>
          </Id>
          <CtryOfRes>DE</CtryOfRes>
        </Cdtr>
        <CdtrAcct>
          <Id>
            <Othr>
              <Id>DE89370400440532013000</Id>
              <SchmeNm>
                <Cd>IBAN</Cd>
              </SchmeNm>
            </Othr>
          </Id>
          <Tp>
            <Cd>CACC</Cd>
          </Tp>
          <Ccy>EUR</Ccy>
          <Nm>John Doe Current Account</Nm>
        </CdtrAcct>
        <UltmtCdtr>
          <Nm>Ultimate Creditor Company</Nm>
          <PstlAdr>
            <StrtNm>500 Final Street</StrtNm>
            <TwnNm>Munich</TwnNm>
            <Ctry>DE</Ctry>
          </PstlAdr>
          <Id>
            <OrgId>
              <LEI>549300ULTIMATECRED1</LEI>
            </OrgId>
          </Id>
        </UltmtCdtr>
        <InstrForCdtrAgt>
          <Cd>HOLD</Cd>
          <InstrInf>Hold payment for verification</InstrInf>
        </InstrForCdtrAgt>
        <InstrForCdtrAgt>
          <Cd>PHOB</Cd>
          <InstrInf>Contact by phone: +49-69-1234567</InstrInf>
        </InstrForCdtrAgt>
        <InstrForNxtAgt>
          <InstrInf>Please process with high priority
Additional instructions for next agent</InstrInf>
        </InstrForNxtAgt>
        <Purp>
          <Cd>CBFF</Cd>
        </Purp>
        <RgltryRptg>
          <DbtCdtRptgInd>BOTH</DbtCdtRptgInd>
          <Authrty>
            <Nm>Federal Financial Supervisory Authority</Nm>
            <Ctry>DE</Ctry>
          </Authrty>
          <Dtls>
            <Tp>BALANCE_OF_PAYMENTS</Tp>
            <Dt>2025-06-27</Dt>
            <Ctry>DE</Ctry>
            <Cd>BEXP</Cd>
            <Amt Ccy="EUR">1000.00</Amt>
            <Inf>Export payment for goods</Inf>
            <Inf>Additional regulatory information</Inf>
          </Dtls>
        </RgltryRptg>
        <RltdRmtInf>
          <RmtId>REMIT-ID-123456</RmtId>
          <RmtLctnDtls>
            <Mtd>EMAL</Mtd>
            <ElctrncAdr>remittance@email.com</ElctrncAdr>
          </RmtLctnDtls>
          <RmtLctnDtls>
            <Mtd>POST</Mtd>
            <PstlAdr>
              <Nm>Remittance Department</Nm>
              <Adr>
                <StrtNm>123 Remittance Street</StrtNm>
                <TwnNm>Processing City</TwnNm>
                <Ctry>DE</Ctry>
              </Adr>
            </PstlAdr>
          </RmtLctnDtls>
        </RltdRmtInf>
        <RmtInf>
          <Ustrd>Payment for invoice INV-2025-001234
Thank you for your business
Reference: CONTRACT-2025-789</Ustrd>
          <Strd>
            <RfrdDocInf>
              <Tp>
                <CdOrPrtry>
                  <Cd>CINV</Cd>
                </CdOrPrtry>
                <Issr>ACME Corp</Issr>
              </Tp>
              <Nb>INV-2025-001234</Nb>
              <RltdDt>2025-06-20</RltdDt>
              <LineDtls>
                <Id>
                  <Tp>
                    <CdOrPrtry>
                      <Cd>LINB</Cd>
                    </CdOrPrtry>
                  </Tp>
                  <Nb>001</Nb>
                  <RltdDt>2025-06-20</RltdDt>
                </Id>
                <Desc>Professional services</Desc>
                <Amt>
                  <DuePyblAmt Ccy="EUR">1000.00</DuePyblAmt>
                  <RmtdAmt Ccy="EUR">1000.00</RmtdAmt>
                </Amt>
              </LineDtls>
            </RfrdDocInf>
            <RfrdDocAmt>
              <DuePyblAmt Ccy="EUR">1000.00</DuePyblAmt>
              <RmtdAmt Ccy="EUR">1000.00</RmtdAmt>
            </RfrdDocAmt>
            <CdtrRefInf>
              <Tp>
                <CdOrPrtry>
                  <Cd>SCOR</Cd>
                </CdOrPrtry>
                <Issr>ACME Corp</Issr>
              </Tp>
              <Ref>REF-2025-789456</Ref>
            </CdtrRefInf>
            <Invcr>
              <Nm>ACME Corporation</Nm>
              <PstlAdr>
                <TwnNm>Business City</TwnNm>
                <Ctry>DE</Ctry>
              </PstlAdr>
              <Id>
                <OrgId>
                  <LEI>549300ACMECORP12345</LEI>
                </OrgId>
              </Id>
            </Invcr>
            <Invcee>
              <Nm>Customer Company Ltd</Nm>
              <PstlAdr>
                <TwnNm>Customer City</TwnNm>
                <Ctry>US</Ctry>
              </PstlAdr>
              <Id>
                <OrgId>
                  <LEI>549300CUSTOMER54321</LEI>
                </OrgId>
              </Id>
            </Invcee>
            <AddtlRmtInf>Additional remittance note</AddtlRmtInf>
            <AddtlRmtInf>Secondary reference information</AddtlRmtInf>
          </Strd>
        </RmtInf>
      </CdtTrfTxInf>
    </FIToFICstmrCdtTrf>
  </Document>
</Envelope>`
  },
  'pacs.009': {
    name: 'ISO 20022 pacs.009 → MT202/MT205',
    description: 'Financial Institution Credit Transfer',
    targetFormat: 'SWIFT MT202/MT205',
    sample: `<?xml version="1.0" encoding="UTF-8"?>
<AppHdr xmlns="urn:iso:std:iso:20022:tech:xsd:head.001.001.01">
    <Fr>
        <FIId>
            <FinInstnId>
                <BICFI>BANKBEBBXXX</BICFI>
            </FinInstnId>
        </FIId>
    </Fr>
    <To>
        <FIId>
            <FinInstnId>
                <BICFI>BOFAUS3NXXX</BICFI>
            </FinInstnId>
        </FIId>
    </To>
    <BizMsgIdr>CORE20240101001-pacs.009-rev</BizMsgIdr>
    <MsgDefIdr>pacs.009.001.08</MsgDefIdr>
    <BizSvc>swift.cbprplus.02</BizSvc>
    <CreDt>2024-01-01T10:30:00.000Z</CreDt>
</AppHdr>
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:pacs.009.001.08">
    <FIToFICdtTrf>
        <GrpHdr>
            <MsgId>PACS009CORE20240101001</MsgId>
            <CreDtTm>2024-01-01T10:30:00.000Z</CreDtTm>
            <NbOfTxs>1</NbOfTxs>
            <SttlmInf>
                <SttlmMtd>INGA</SttlmMtd>
                <InstrRmbrsmtAgt>
                    <FinInstnId>
                        <BICFI>DEUTDEFFXXX</BICFI>
                    </FinInstnId>
                </InstrRmbrsmtAgt>
                <InstdRmbrsmtAgt>
                    <FinInstnId>
                        <BICFI>BANKGB2LXXX</BICFI>
                    </FinInstnId>
                </InstdRmbrsmtAgt>
            </SttlmInf>
        </GrpHdr>
        <CdtTrfTxInf>
            <PmtId>
                <InstrId>CORE20240101001</InstrId>
                <EndToEndId>E2E20240101001</EndToEndId>
                <UETR>550e8400-e29b-41d4-a716-446655440000</UETR>
            </PmtId>
            <PmtTpInf>
                <SvcLvl>
                    <Cd>G004</Cd>
                </SvcLvl>
                <LclInstrm>
                    <Cd>SDVA</Cd>
                </LclInstrm>
                <CtgyPurp>
                    <Cd>INTC</Cd>
                </CtgyPurp>
            </PmtTpInf>
            <IntrBkSttlmAmt Ccy="USD">1250000.00</IntrBkSttlmAmt>
            <IntrBkSttlmDt>2024-01-01</IntrBkSttlmDt>
            <SttlmTmIndctn>
                <DbtDtTm>2024-01-01T14:30:00+00:00</DbtDtTm>
            </SttlmTmIndctn>
            <InstgAgt>
                <FinInstnId>
                    <BICFI>BANKBEBBXXX</BICFI>
                </FinInstnId>
            </InstgAgt>
            <InstdAgt>
                <FinInstnId>
                    <BICFI>BOFAUS3NXXX</BICFI>
                </FinInstnId>
            </InstdAgt>
            <IntrmyAgt1>
                <FinInstnId>
                    <BICFI>CHASUS33XXX</BICFI>
                </FinInstnId>
            </IntrmyAgt1>
            <Dbtr>
                <FinInstnId>
                    <BICFI>BANKBEBBXXX</BICFI>
                </FinInstnId>
            </Dbtr>
            <CdtrAgt>
                <FinInstnId>
                    <BICFI>BANKUS33XXX</BICFI>
                </FinInstnId>
            </CdtrAgt>
            <Cdtr>
                <FinInstnId>
                    <BICFI>BOFAUS3NXXX</BICFI>
                </FinInstnId>
            </Cdtr>
            <InstrForCdtrAgt>
                <InstrInf>/FIN53/DEUTDEFFXXX</InstrInf>
            </InstrForCdtrAgt>
            <InstrForNxtAgt>
                <InstrInf>/REC/URGENT PROCESSING REQUIRED</InstrInf>
            </InstrForNxtAgt>
            <Purp>
                <Cd>TRAD</Cd>
            </Purp>
            <RmtInf>
                <Ustrd>/BNF/TRADE SETTLEMENT PAYMENT</Ustrd>
            </RmtInf>
        </CdtTrfTxInf>
    </FIToFICdtTrf>
</Document>`
  },
  'camt.107': {
    name: 'ISO 20022 camt.107 → MT110',
    description: 'Cheque Presentment Notification',
    targetFormat: 'SWIFT MT110',
    sample: `<?xml version="1.0" encoding="UTF-8"?>
<Envelope xmlns="urn:swift:xsd:$ahV10">
  <AppHdr>
    <Fr>
      <FIId>
        <FinInstnId>
          <BICFI>BANKUS33XXX</BICFI>
        </FinInstnId>
      </FIId>
    </Fr>
    <To>
      <FIId>
        <FinInstnId>
          <BICFI>DEUTDEFFXXX</BICFI>
        </FinInstnId>
      </FIId>
    </To>
    <BizMsgIdr>20250630-CHQ-001</BizMsgIdr>
    <MsgDefIdr>camt.107.001.01</MsgDefIdr>
    <CreDt>2025-06-30T14:30:00Z</CreDt>
    <BizSvc>CBPR</BizSvc>
  </AppHdr>
  <Document xmlns="urn:iso:std:iso:20022:tech:xsd:camt.107.001.01">
    <ChqPresntmntNtfctn>
      <GrpHdr>
        <MsgId>CHQ20250630001</MsgId>
        <CreDtTm>2025-06-30T14:30:00Z</CreDtTm>
        <NbOfChqs>1</NbOfChqs>
      </GrpHdr>
      <Chq>
        <InstrId>INSTR20250630001</InstrId>
        <ChqNb>CHQ001234567</ChqNb>
        <IsseDt>2025-06-27</IsseDt>
        <Amt Ccy="USD">15000.00</Amt>
        <ValDt>
          <Dt>2025-06-30</Dt>
        </ValDt>
        <Pyer>
          <Nm>ACME CORPORATION</Nm>
          <PstlAdr>
            <AdrLine>123 MAIN STREET</AdrLine>
            <AdrLine>SUITE 400</AdrLine>
            <TwnNm>NEW YORK</TwnNm>
            <Ctry>US</Ctry>
          </PstlAdr>
          <Id>
            <OrgId>
              <Othr>
                <Id>TAX123456789</Id>
                <SchmeNm>
                  <Cd>TXID</Cd>
                </SchmeNm>
              </Othr>
            </OrgId>
          </Id>
        </Pyer>
        <PyerAcct>
          <Id>
            <Othr>
              <Id>1234567890</Id>
            </Othr>
          </Id>
        </PyerAcct>
        <DrwrAgt>
          <FinInstnId>
            <BICFI>BANKUS33XXX</BICFI>
            <Nm>BANK OF AMERICA</Nm>
            <PstlAdr>
              <StrtNm>100 FEDERAL STREET</StrtNm>
              <TwnNm>BOSTON</TwnNm>
              <Ctry>US</Ctry>
            </PstlAdr>
          </FinInstnId>
        </DrwrAgt>
        <DrwrAgtAcct>
          <Id>
            <Othr>
              <Id>US12345678901234567890</Id>
            </Othr>
          </Id>
        </DrwrAgtAcct>
        <Pyee>
          <Nm>MUELLER GMBH</Nm>
          <PstlAdr>
            <AdrLine>HAUPTSTRASSE 1</AdrLine>
            <AdrLine>BUILDING A</AdrLine>
            <TwnNm>BERLIN</TwnNm>
            <Ctry>DE</Ctry>
          </PstlAdr>
        </Pyee>
      </Chq>
    </ChqPresntmntNtfctn>
  </Document>
</Envelope>`
  },
  'camt.108': {
    name: 'ISO 20022 camt.108 → MT111',
    description: 'Cheque Cancellation or Stop Request',
    targetFormat: 'SWIFT MT111',
    sample: `<?xml version="1.0" encoding="UTF-8"?>
<Envelope xmlns="urn:swift:xsd:$ahV10">
  <AppHdr>
    <Fr>
      <FIId>
        <FinInstnId>
          <BICFI>BANKUS33XXX</BICFI>
        </FinInstnId>
      </FIId>
    </Fr>
    <To>
      <FIId>
        <FinInstnId>
          <BICFI>DEUTDEFFXXX</BICFI>
        </FinInstnId>
      </FIId>
    </To>
    <BizMsgIdr>20250630-CHQ-STOP-001</BizMsgIdr>
    <MsgDefIdr>camt.108.001.01</MsgDefIdr>
    <CreDt>2025-06-30T15:00:00Z</CreDt>
    <BizSvc>CBPR</BizSvc>
  </AppHdr>
  <Document xmlns="urn:iso:std:iso:20022:tech:xsd:camt.108.001.01">
    <ChqCxlOrStopReq>
      <GrpHdr>
        <MsgId>STOP20250630001</MsgId>
        <CreDtTm>2025-06-30T15:00:00Z</CreDtTm>
        <NbOfChqs>1</NbOfChqs>
      </GrpHdr>
      <Chq>
        <InstrId>STOPINSTR20250630001</InstrId>
        <OrgnlInstrId>INSTR20250630001</OrgnlInstrId>
        <ChqNb>CHQ001234567</ChqNb>
        <IsseDt>2025-06-27</IsseDt>
        <Amt Ccy="USD">15000.00</Amt>
        <EffctvDt>
          <Dt>2025-07-01</Dt>
        </EffctvDt>
        <DrwrAgt>
          <FinInstnId>
            <BICFI>BANKUS33XXX</BICFI>
            <Nm>BANK OF AMERICA</Nm>
            <PstlAdr>
              <StrtNm>100 FEDERAL STREET</StrtNm>
              <TwnNm>BOSTON</TwnNm>
              <Ctry>US</Ctry>
            </PstlAdr>
          </FinInstnId>
        </DrwrAgt>
        <DrwrAgtAcct>
          <Id>
            <Othr>
              <Id>US12345678901234567890</Id>
            </Othr>
          </Id>
        </DrwrAgtAcct>
        <Pyee>
          <Nm>MUELLER GMBH</Nm>
          <PstlAdr>
            <AdrLine>HAUPTSTRASSE 1</AdrLine>
            <AdrLine>BUILDING A</AdrLine>
            <TwnNm>BERLIN</TwnNm>
            <Ctry>DE</Ctry>
          </PstlAdr>
        </Pyee>
        <ChqCxlOrStopRsn>
          <Rsn>
            <Cd>LOST</Cd>
          </Rsn>
          <AddtlInf>Cheque reported lost by payee</AddtlInf>
        </ChqCxlOrStopRsn>
      </Chq>
    </ChqCxlOrStopReq>
  </Document>
</Envelope>`
  },
  'camt.109': {
    name: 'ISO 20022 camt.109 → MT112',
    description: 'Cheque Cancellation or Stop Report',
    targetFormat: 'SWIFT MT112',
    sample: `<?xml version="1.0" encoding="UTF-8"?>
<Envelope xmlns="urn:swift:xsd:$ahV10">
  <AppHdr>
    <Fr>
      <FIId>
        <FinInstnId>
          <BICFI>DEUTDEFFXXX</BICFI>
        </FinInstnId>
      </FIId>
    </Fr>
    <To>
      <FIId>
        <FinInstnId>
          <BICFI>BANKUS33XXX</BICFI>
        </FinInstnId>
      </FIId>
    </To>
    <BizMsgIdr>20250630-CHQ-STATUS-001</BizMsgIdr>
    <MsgDefIdr>camt.109.001.01</MsgDefIdr>
    <CreDt>2025-06-30T16:00:00Z</CreDt>
    <BizSvc>swift.cbprplus.03</BizSvc>
  </AppHdr>
  <Document xmlns="urn:iso:std:iso:20022:tech:xsd:camt.109.001.01">
    <ChqCxlOrStopRpt>
      <GrpHdr>
        <MsgId>RPT20250630001</MsgId>
        <CreDtTm>2025-06-30T16:00:00Z</CreDtTm>
        <NbOfChqs>1</NbOfChqs>
      </GrpHdr>
      <Chq>
        <InstrId>RPTINSTR20250630001</InstrId>
        <OrgnlInstrId>STOPINSTR20250630001</OrgnlInstrId>
        <ChqNb>CHQ001234567</ChqNb>
        <IsseDt>2025-06-27</IsseDt>
        <Amt Ccy="USD">15000.00</Amt>
        <EffctvDt>
          <Dt>2025-07-01</Dt>
        </EffctvDt>
        <DrwrAgt>
          <FinInstnId>
            <BICFI>BANKUS33XXX</BICFI>
            <Nm>BANK OF AMERICA</Nm>
            <PstlAdr>
              <StrtNm>100 FEDERAL STREET</StrtNm>
              <TwnNm>BOSTON</TwnNm>
              <Ctry>US</Ctry>
            </PstlAdr>
          </FinInstnId>
        </DrwrAgt>
        <DrwrAgtAcct>
          <Id>
            <Othr>
              <Id>US12345678901234567890</Id>
            </Othr>
          </Id>
        </DrwrAgtAcct>
        <Pyee>
          <Nm>MUELLER GMBH</Nm>
          <PstlAdr>
            <AdrLine>HAUPTSTRASSE 1</AdrLine>
            <AdrLine>BUILDING A</AdrLine>
            <TwnNm>BERLIN</TwnNm>
            <Ctry>DE</Ctry>
          </PstlAdr>
        </Pyee>
        <ChqCxlOrStopSts>
          <Sts>
            <Cd>ACCR</Cd>
          </Sts>
          <OrgnlReqId>STOPINSTR20250630001</OrgnlReqId>
          <AddtlInf>Stop payment request successfully processed and accepted</AddtlInf>
        </ChqCxlOrStopSts>
      </Chq>
    </ChqCxlOrStopRpt>
  </Document>
</Envelope>`
  },
  'camt.052': {
    name: 'ISO 20022 camt.052 → MT942',
    description: 'Bank to Customer Account Report (Interim Transaction Report)',
    targetFormat: 'SWIFT MT942',
    sample: `<?xml version="1.0" encoding="UTF-8"?>
<Envelope xmlns="urn:swift:xsd:$ahV10">
  <AppHdr>
    <Fr>
      <FIId>
        <FinInstnId>
          <BICFI>SVBKUS6SXXX</BICFI>
        </FinInstnId>
      </FIId>
    </Fr>
    <To>
      <FIId>
        <FinInstnId>
          <BICFI>TESTUS33XXX</BICFI>
        </FinInstnId>
      </FIId>
    </To>
    <BizMsgIdr>20231201-INTERIM-001</BizMsgIdr>
    <MsgDefIdr>camt.052.001.08</MsgDefIdr>
    <CreDt>2023-12-01T10:30:00Z</CreDt>
    <BizSvc>swift.cbprplus.03</BizSvc>
  </AppHdr>
  <Document xmlns="urn:iso:std:iso:20022:tech:xsd:camt.052.001.08">
    <BkToCstmrAcctRpt>
      <GrpHdr>
        <MsgId>INTERIM20231201001</MsgId>
        <CreDtTm>2023-12-01T10:30:15.000Z</CreDtTm>
        <MsgRcpt>
          <Nm>Test Bank Customer</Nm>
          <Id>
            <OrgId>
              <AnyBIC>TESTUS33XXX</AnyBIC>
            </OrgId>
          </Id>
        </MsgRcpt>
        <MsgPgntn>
          <PgNb>1</PgNb>
          <LastPgInd>true</LastPgInd>
        </MsgPgntn>
      </GrpHdr>
      <Rpt>
        <Id>STMT2023120100001</Id>
        <LglSeqNb>12345</LglSeqNb>
        <CreDtTm>2023-12-01T10:30:00.000Z</CreDtTm>
        <RptPgntn>
          <PgNb>1</PgNb>
          <LastPgInd>true</LastPgInd>
        </RptPgntn>
        <Acct>
          <Id>
            <IBAN>US64SVBKUS6S3300958879</IBAN>
          </Id>
          <Tp>
            <Cd>CACC</Cd>
          </Tp>
          <Ccy>USD</Ccy>
          <Ownr>
            <Nm>ACME Corporation</Nm>
            <Id>
              <OrgId>
                <AnyBIC>TESTUS33XXX</AnyBIC>
              </OrgId>
            </Id>
          </Ownr>
          <Svcr>
            <FinInstnId>
              <BICFI>SVBKUS6SXXX</BICFI>
              <Nm>Silicon Valley Bank</Nm>
            </FinInstnId>
          </Svcr>
        </Acct>
        <Bal>
          <Tp>
            <CdOrPrtry>
              <Cd>OPBD</Cd>
            </CdOrPrtry>
          </Tp>
          <Amt Ccy="USD">50000.00</Amt>
          <CdtDbtInd>CRDT</CdtDbtInd>
          <Dt>
            <Dt>2023-12-01</Dt>
          </Dt>
        </Bal>
        <TxsSummry>
          <TtlNtries>
            <NbOfNtries>3</NbOfNtries>
            <Sum>2500.75</Sum>
          </TtlNtries>
          <TtlCdtNtries>
            <NbOfNtries>2</NbOfNtries>
            <Sum>3000.00</Sum>
          </TtlCdtNtries>
          <TtlDbtNtries>
            <NbOfNtries>1</NbOfNtries>
            <Sum>499.25</Sum>
          </TtlDbtNtries>
        </TxsSummry>
        <Ntry>
          <Amt Ccy="USD">1500.00</Amt>
          <CdtDbtInd>CRDT</CdtDbtInd>
          <Sts>
            <Cd>BOOK</Cd>
          </Sts>
          <BookgDt>
            <Dt>2023-12-01</Dt>
          </BookgDt>
          <ValDt>
            <Dt>2023-12-01</Dt>
          </ValDt>
          <AcctSvcrRef>TRANS001-20231201</AcctSvcrRef>
          <NtryDtls>
            <TxDtls>
              <Refs>
                <EndToEndId>E2E-PAYMENT-001</EndToEndId>
                <TxId>TXN-ID-001-20231201</TxId>
              </Refs>
              <RltdPties>
                <Dbtr>
                  <Nm>John Smith</Nm>
                </Dbtr>
                <Cdtr>
                  <Nm>ACME Corporation</Nm>
                </Cdtr>
              </RltdPties>
              <RmtInf>
                <Ustrd>Payment for Invoice INV-2023-001</Ustrd>
              </RmtInf>
              <AddtlTxInf>Customer payment received</AddtlTxInf>
            </TxDtls>
          </NtryDtls>
          <AddtlNtryInf>Direct deposit from customer account</AddtlNtryInf>
        </Ntry>
        <AddtlRptInf>This is an interim transaction report showing all transactions processed during the business day.</AddtlRptInf>
      </Rpt>
    </BkToCstmrAcctRpt>
  </Document>
</Envelope>`
  },
  'camt.053': {
    name: 'ISO 20022 camt.053 → MT940',
    description: 'Bank to Customer Statement',
    targetFormat: 'SWIFT MT940',
    sample: `<?xml version="1.0" encoding="UTF-8"?>
<Envelope xmlns="urn:swift:xsd:$ahV10">
  <AppHdr>
    <Fr>
      <FIId>
        <FinInstnId>
          <BICFI>SVBKUS6SXXX</BICFI>
        </FinInstnId>
      </FIId>
    </Fr>
    <To>
      <FIId>
        <FinInstnId>
          <BICFI>TESTUS33XXX</BICFI>
        </FinInstnId>
      </FIId>
    </To>
    <BizMsgIdr>20231201-STMT-001</BizMsgIdr>
    <MsgDefIdr>camt.053.001.08</MsgDefIdr>
    <CreDt>2023-12-01T23:59:59Z</CreDt>
    <BizSvc>swift.cbprplus.03</BizSvc>
  </AppHdr>
  <Document xmlns="urn:iso:std:iso:20022:tech:xsd:camt.053.001.08">
    <BkToCstmrStmt>
      <GrpHdr>
        <MsgId>STMT20231201001</MsgId>
        <CreDtTm>2023-12-01T23:59:59.000Z</CreDtTm>
        <MsgRcpt>
          <Nm>Test Bank Customer</Nm>
          <Id>
            <OrgId>
              <AnyBIC>TESTUS33XXX</AnyBIC>
            </OrgId>
          </Id>
        </MsgRcpt>
        <MsgPgntn>
          <PgNb>1</PgNb>
          <LastPgInd>true</LastPgInd>
        </MsgPgntn>
      </GrpHdr>
      <Stmt>
        <Id>STMT2023120100001</Id>
        <LglSeqNb>12345</LglSeqNb>
        <CreDtTm>2023-12-01T23:59:59.000Z</CreDtTm>
        <FrToDt>
          <FrDtTm>2023-12-01T00:00:00.000Z</FrDtTm>
          <ToDtTm>2023-12-01T23:59:59.000Z</ToDtTm>
        </FrToDt>
        <StmtPgntn>
          <PgNb>1</PgNb>
          <LastPgInd>true</LastPgInd>
        </StmtPgntn>
        <Acct>
          <Id>
            <IBAN>US64SVBKUS6S3300958879</IBAN>
          </Id>
          <Tp>
            <Cd>CACC</Cd>
          </Tp>
          <Ccy>USD</Ccy>
          <Ownr>
            <Nm>ACME Corporation</Nm>
            <Id>
              <OrgId>
                <AnyBIC>TESTUS33XXX</AnyBIC>
              </OrgId>
            </Id>
          </Ownr>
          <Svcr>
            <FinInstnId>
              <BICFI>SVBKUS6SXXX</BICFI>
              <Nm>Silicon Valley Bank</Nm>
            </FinInstnId>
          </Svcr>
        </Acct>
        <Bal>
          <Tp>
            <CdOrPrtry>
              <Cd>OPBD</Cd>
            </CdOrPrtry>
          </Tp>
          <Amt Ccy="USD">47500.25</Amt>
          <CdtDbtInd>CRDT</CdtDbtInd>
          <Dt>
            <Dt>2023-11-30</Dt>
          </Dt>
        </Bal>
        <Bal>
          <Tp>
            <CdOrPrtry>
              <Cd>CLBD</Cd>
            </CdOrPrtry>
          </Tp>
          <Amt Ccy="USD">50000.75</Amt>
          <CdtDbtInd>CRDT</CdtDbtInd>
          <Dt>
            <Dt>2023-12-01</Dt>
          </Dt>
        </Bal>
        <Bal>
          <Tp>
            <CdOrPrtry>
              <Cd>CLAV</Cd>
            </CdOrPrtry>
          </Tp>
          <Amt Ccy="USD">48000.75</Amt>
          <CdtDbtInd>CRDT</CdtDbtInd>
          <Dt>
            <Dt>2023-12-01</Dt>
          </Dt>
        </Bal>
        <Ntry>
          <Amt Ccy="USD">3000.00</Amt>
          <CdtDbtInd>CRDT</CdtDbtInd>
          <Sts>
            <Cd>BOOK</Cd>
          </Sts>
          <BookgDt>
            <Dt>2023-12-01</Dt>
          </BookgDt>
          <ValDt>
            <Dt>2023-12-01</Dt>
          </ValDt>
          <AcctSvcrRef>TRANS001-20231201</AcctSvcrRef>
          <NtryDtls>
            <TxDtls>
              <Refs>
                <EndToEndId>E2E-PAYMENT-001</EndToEndId>
                <TxId>TXN-ID-001-20231201</TxId>
              </Refs>
              <RltdPties>
                <Dbtr>
                  <Nm>John Smith</Nm>
                </Dbtr>
                <Cdtr>
                  <Nm>ACME Corporation</Nm>
                </Cdtr>
              </RltdPties>
              <RmtInf>
                <Ustrd>Payment for Invoice INV-2023-001</Ustrd>
              </RmtInf>
              <AddtlTxInf>Customer payment received via ACH</AddtlTxInf>
            </TxDtls>
          </NtryDtls>
          <AddtlNtryInf>Direct deposit from customer account via ACH</AddtlNtryInf>
        </Ntry>
        <AddtlStmtInf>This is the daily customer statement for account US64SVBKUS6S3300958879 showing all transactions processed on 2023-12-01.</AddtlStmtInf>
      </Stmt>
    </BkToCstmrStmt>
  </Document>
</Envelope>`
  }
};

function App() {
  const [activeTab, setActiveTab] = useState('forward');
  const [selectedTransformation, setSelectedTransformation] = useState('MT103');
  const [inputMessage, setInputMessage] = useState('');
  const [outputMessage, setOutputMessage] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState([]);
  const [warnings, setWarnings] = useState([]);
  const [success, setSuccess] = useState(false);
  const [resultCount, setResultCount] = useState(0);

  // Fixed height for both input and output areas
  const FIXED_CONTENT_HEIGHT = '400px';

  // Detect if we're in development mode or GitHub Pages
  const isDevelopment = window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1';

  // Get current transformations based on active tab
  const getCurrentTransformations = () => {
    return activeTab === 'forward' ? FORWARD_TRANSFORMATIONS : REVERSE_TRANSFORMATIONS;
  };

  // Auto-load sample when transformation type changes or tab changes
  useEffect(() => {
    const transformations = getCurrentTransformations();
    const firstKey = Object.keys(transformations)[0];
    setSelectedTransformation(firstKey);
    setInputMessage(transformations[firstKey].sample);
    setError([]);
    setWarnings([]);
    setOutputMessage('');
    setSuccess(false);
  }, [activeTab]);

  useEffect(() => {
    const transformations = getCurrentTransformations();
    if (transformations[selectedTransformation]) {
      setInputMessage(transformations[selectedTransformation].sample);
      setError([]);
      setWarnings([]);
      setOutputMessage('');
      setSuccess(false);
    }
  }, [selectedTransformation, activeTab]);

  const formatXml = (xml) => {
    try {
      // Basic XML formatting - in production, use a proper XML formatter
      const formatted = xml
        .replace(/></g, '>\n<')
        .replace(/^\s*\n/gm, '')
        .split('\n')
        .map(line => {
          const depth = (line.match(/</g) || []).length - (line.match(/</g) || []).length;
          return '  '.repeat(Math.max(0, depth)) + line.trim();
        })
        .join('\n');
      return formatted;
    } catch (e) {
      return xml;
    }
  };

  const formatMT = (mt) => {
    // Basic MT formatting
    return mt.replace(/\n/g, '\n').trim();
  };

  // Handle multiple transformed messages array
  const processTransformedMessages = (transformedMessages) => {
    if (!transformedMessages || !Array.isArray(transformedMessages)) {
      // Handle single message or non-array response
      const singleMessage = Array.isArray(transformedMessages) ? transformedMessages[0] : transformedMessages;
      return typeof singleMessage === 'object' ? JSON.stringify(singleMessage, null, 2) : String(singleMessage || '');
    }

    // Handle array of messages
    const processedMessages = transformedMessages.map((message, index) => {
      let content = typeof message === 'object' ? JSON.stringify(message, null, 2) : String(message);
      
      // Format based on transformation direction
      if (activeTab === 'forward') {
        content = formatXml(content);
      } else {
        content = formatMT(content);
      }

      // Add comment header for multiple messages
      if (transformedMessages.length > 1) {
        const commentPrefix = activeTab === 'forward' ? '<!-- ' : '// ';
        const commentSuffix = activeTab === 'forward' ? ' -->' : '';
        return `${commentPrefix}Transformation Result ${index + 1} of ${transformedMessages.length}${commentSuffix}\n${content}`;
      }
      
      return content;
    });

    // Join multiple messages
    if (transformedMessages.length > 1) {
      const separator = activeTab === 'forward' 
        ? '\n\n<!-- ========================= -->\n\n'
        : '\n\n// =========================\n\n';
      return processedMessages.join(separator);
    }

    return processedMessages[0] || '';
  };

  const handleTransform = async () => {
    if (!inputMessage.trim()) {
      setError(['Please enter a message to transform']);
      return;
    }

    setLoading(true);
    setError([]);
    setWarnings([]);
    setSuccess(false);
    setOutputMessage('');
    setResultCount(0);

    try {
      const endpoint = API_ENDPOINTS[activeTab];
      const response = await fetch(endpoint, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          message: inputMessage,
          options: {
            validation: true,
            include_debug: false,
            format_output: true
          }
        }),
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const data = await response.json();
      
      // Handle the new response structure with array support
      if (data.success && data.transformed_message) {
        const processedOutput = processTransformedMessages(data.transformed_message);
        setOutputMessage(processedOutput);
        
        // Set result count for display
        const count = Array.isArray(data.transformed_message) ? data.transformed_message.length : 1;
        setResultCount(count);
        setSuccess(true);
        
        // Set warnings if any
        if (data.warnings && data.warnings.length > 0) {
          setWarnings(data.warnings);
        }
      } else {
        setError(data.errors || ['Transformation failed with no specific error']);
      }
    } catch (err) {
      console.error('Transformation error:', err);
      if (err.name === 'TypeError' && err.message.includes('fetch')) {
        setError([
          'Connection error - please ensure the Reframe server is running.',
          isDevelopment 
            ? 'Start the server with: cargo run' 
            : 'Please contact your system administrator.'
        ]);
      } else {
        setError([`Transformation failed: ${err.message}`]);
      }
    } finally {
      setLoading(false);
    }
  };

  const handleTransformationChange = (transformation) => {
    setSelectedTransformation(transformation);
  };

  const handleClear = () => {
    setInputMessage('');
    setOutputMessage('');
    setError([]);
    setWarnings([]);
    setSuccess(false);
    setResultCount(0);
  };

  const handleCopyToClipboard = async () => {
    try {
      await navigator.clipboard.writeText(outputMessage);
      // Could add a toast notification here
    } catch (err) {
      console.error('Failed to copy text: ', err);
    }
  };

  const getTransformationIcon = () => {
    return activeTab === 'forward' ? <IconArrowRight size={20} /> : <IconArrowLeft size={20} />;
  };

  const getCurrentSelectedTransformation = () => {
    const transformations = getCurrentTransformations();
    return transformations[selectedTransformation] || Object.values(transformations)[0];
  };

  return (
    <Box style={{ minHeight: '100vh', display: 'flex', flexDirection: 'column' }}>
      <Container style={{ flex: 1, display: 'flex', flexDirection: 'column', width: '100%', maxWidth: '100%', height: '100%' }}>
        <Stack gap="lg" style={{ flex: 1 }}>
          <Paper shadow="sm" p="md" radius="md">
            <Group justify="space-between" align="center">
              <div>
                <Title order={1} c="blue.6">
                  🔄 Reframe - SWIFT MT ↔ ISO 20022 MX Transformation
                </Title>
                <Text c="dimmed" size="sm">
                  Convert between SWIFT MT and ISO 20022 MX message formats
                </Text>
              </div>
              <Badge size="lg" color="green" variant="light">
                v2.0 - Bidirectional
              </Badge>
            </Group>
          </Paper>

          <Tabs value={activeTab} onChange={setActiveTab}>
            <Tabs.List>
              <Tabs.Tab value="forward" leftSection={<IconArrowRight size={16} />}>
                Forward (MT → MX)
              </Tabs.Tab>
              <Tabs.Tab value="reverse" leftSection={<IconArrowLeft size={16} />}>
                Reverse (MX → MT)
              </Tabs.Tab>
            </Tabs.List>

            <Tabs.Panel value="forward" pt="md">
              <Alert color="blue" icon={<IconArrowRight size={16} />} mb="md">
                <strong>Forward Transformation:</strong> Convert SWIFT MT messages to ISO 20022 MX format
              </Alert>
            </Tabs.Panel>

            <Tabs.Panel value="reverse" pt="md">
              <Alert color="orange" icon={<IconArrowLeft size={16} />} mb="md">
                <strong>Reverse Transformation:</strong> Convert ISO 20022 MX messages to SWIFT MT format
              </Alert>
            </Tabs.Panel>
          </Tabs>

          <Grid style={{ flex: 1, height: 'calc(100vh - 280px)' }}>
            <Grid.Col span={{ base: 12, md: 6 }} style={{ height: '100%' }}>
              <Card shadow="sm" padding="lg" radius="md" withBorder style={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
                <Stack gap="md" style={{ flex: 1 }}>
                  <Group justify="space-between" align="center">
                    <Title order={3} c="blue.7">
                      {activeTab === 'forward' ? 'Input MT Message' : 'Input MX Message'}
                    </Title>
                    <Select
                      placeholder="Select transformation"
                      value={selectedTransformation}
                      onChange={handleTransformationChange}
                      data={Object.keys(getCurrentTransformations()).map(key => ({
                        value: key,
                        label: getCurrentTransformations()[key].name
                      }))}
                      w={300}
                    />
                  </Group>
                  
                  <Box>
                    <Text size="sm" c="dimmed" mb="xs">
                      {getCurrentSelectedTransformation().description}
                    </Text>
                    <Badge variant="light" color="blue" size="sm">
                      Target: {getCurrentSelectedTransformation().targetFormat}
                    </Badge>
                  </Box>

                  <Textarea
                    value={inputMessage}
                    onChange={(e) => setInputMessage(e.target.value)}
                    placeholder={`Enter your ${activeTab === 'forward' ? 'SWIFT MT' : 'ISO 20022 MX'} message here...`}
                    style={{ flex: 1, minHeight: FIXED_CONTENT_HEIGHT }}
                    styles={{
                      input: {
                        fontFamily: 'Monaco, Consolas, "Courier New", monospace',
                        fontSize: '12px',
                        height: FIXED_CONTENT_HEIGHT,
                        resize: 'none',
                        overflow: 'auto'
                      },
                    }}
                  />

                  <Group justify="space-between">
                    <Group>
                      <Button
                        leftSection={getTransformationIcon()}
                        onClick={handleTransform}
                        loading={loading}
                        disabled={!inputMessage.trim()}
                        size="md"
                      >
                        {loading ? 'Transforming...' : 'Transform'}
                      </Button>
                      <Button
                        leftSection={<IconRefresh size={16} />}
                        variant="light"
                        onClick={handleClear}
                        disabled={loading}
                      >
                        Clear
                      </Button>
                    </Group>
                  </Group>
                </Stack>
              </Card>
            </Grid.Col>

            <Grid.Col span={{ base: 12, md: 6 }} style={{ height: '100%' }}>
              <Card shadow="sm" padding="lg" radius="md" withBorder style={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
                <Stack gap="md" style={{ flex: 1 }}>
                  <Group justify="space-between" align="center">
                    <Title order={3} c="green.7">
                      {activeTab === 'forward' ? 'Output MX Message' : 'Output MT Message'}
                    </Title>
                    
                    <Group>
                      {success && resultCount > 0 && (
                        <Badge color="green" variant="light">
                          <IconCheck size={12} style={{ marginRight: 4 }} />
                          {resultCount} result{resultCount > 1 ? 's' : ''}
                        </Badge>
                      )}
                      {outputMessage && (
                        <Button
                          size="xs"
                          variant="light"
                          leftSection={<IconCopy size={14} />}
                          onClick={handleCopyToClipboard}
                        >
                          Copy
                        </Button>
                      )}
                    </Group>
                  </Group>

                  {loading && (
                    <Box ta="center" py="xl">
                      <Loader size="lg" />
                      <Text size="sm" c="dimmed" mt="md">
                        Processing your {activeTab === 'forward' ? 'MT to MX' : 'MX to MT'} transformation...
                      </Text>
                      <Progress value={75} animated size="sm" mt="md" />
                    </Box>
                  )}

                  {error.length > 0 && (
                    <Alert color="red" icon={<IconAlertCircle size={16} />}>
                      <Stack gap="xs">
                        {error.map((err, index) => (
                          <Text key={index} size="sm">
                            {err}
                          </Text>
                        ))}
                      </Stack>
                    </Alert>
                  )}

                  {warnings.length > 0 && (
                    <Alert color="yellow" icon={<IconExclamationMark size={16} />}>
                      <Stack gap="xs">
                        <Text size="sm" fw={500}>Warnings:</Text>
                        {warnings.map((warning, index) => (
                          <Text key={index} size="sm">
                            {warning}
                          </Text>
                        ))}
                      </Stack>
                    </Alert>
                  )}

                  {outputMessage && (
                    <Box style={{ height: '100%', maxHeight: FIXED_CONTENT_HEIGHT, border: '1px solid var(--mantine-color-gray-4)', borderRadius: 'var(--mantine-radius-default)', overflow: 'auto' }}>
                      {activeTab === 'forward' ? (
                        <SyntaxHighlighter
                          language="xml"
                          style={vscDarkPlus}
                          customStyle={{
                            margin: 0,
                            padding: '12px',
                            borderRadius: 'var(--mantine-radius-default)',
                            fontSize: '12px',
                            fontFamily: 'Monaco, Consolas, "Courier New", monospace',
                            height: '100%',
                            overflow: 'auto',
                            backgroundColor: 'var(--mantine-color-body)',
                          }}
                          showLineNumbers={false}
                          wrapLines={false}
                          PreTag="div"
                        >
                          {outputMessage}
                        </SyntaxHighlighter>
                      ) : (
                        <pre
                          style={{
                            margin: 0,
                            padding: '12px',
                            fontSize: '12px',
                            fontFamily: 'Monaco, Consolas, "Courier New", monospace',
                            height: '100%',
                            overflow: 'auto',
                            backgroundColor: 'var(--mantine-color-body)',
                            whiteSpace: 'pre-wrap',
                            wordBreak: 'break-word',
                            border: 'none',
                            outline: 'none',
                          }}
                        >
                          {outputMessage}
                        </pre>
                      )}
                    </Box>
                  )}

                  {!loading && !outputMessage && error.length === 0 && (
                    <Box ta="center" py="xl" c="dimmed" style={{ height: FIXED_CONTENT_HEIGHT, display: 'flex', flexDirection: 'column', justifyContent: 'center', border: '1px solid var(--mantine-color-gray-4)', borderRadius: 'var(--mantine-radius-default)' }}>
                      <IconCode size={48} style={{ opacity: 0.3, margin: '0 auto' }} />
                      <Text size="sm" mt="md">
                        Transformed {activeTab === 'forward' ? 'XML' : 'MT'} output will appear here
                      </Text>
                    </Box>
                  )}
                </Stack>
              </Card>
            </Grid.Col>
          </Grid>

          <Paper shadow="xs" p="md" radius="md" bg="gray.0">
            <Text size="sm" c="dimmed" ta="center">
              Reframe v2.0 - Bidirectional SWIFT MT ↔ ISO 20022 MX Transformation Engine | 
              Powered by Rust + CBPR+ Translation Rules
            </Text>
          </Paper>
        </Stack>
      </Container>
    </Box>
  );
}

export default App; 