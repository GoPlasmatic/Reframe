// Forward transformation configurations (MT to MX)
export const FORWARD_TRANSFORMATIONS = {
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
:11T:ETOEID20240101001
:13C:/RNCTIME/1308+0000
:32A:240101GBP750000,00
:52A:BANKGB2LXXX
:57A:BANKUS33XXX
:59:/9876543210
BENEFICIARY COMPANY INC
456 BROADWAY
NEW YORK NY 10013 US
:70:INVOICE PAYMENT REF 2024-INV-001
TRADE FINANCE SETTLEMENT
:71A:OUR
:72:/REJT/
/MREF/TR20240101095
/TREF/E2E-REF-2024-001
/ReasonCode/AC01
/TEXT/ACCOUNT IDENTIFIER INCORRECT
-}{5:{MAC:12345678}{CHK:123456789ABC}}`
  },
  'MT202': {
    name: 'MT202 → ISO 20022 pacs.009.001.08',
    description: 'General Financial Institution Transfer',
    targetFormat: 'ISO 20022 pacs.009.001.08 XML',
    sample: `{1:F01BANKBEBBAXXX0000000000}
{2:I202BANKDEFFXXXXN}
{3:{113:SEPA}{121:180f1e65-90e0-44d5-a49a-92b55eb3025f}}
{4:
:20:FI202123456789
:21:RELATED987654321
:32A:250615EUR500000,00
:52A:BANKBEBBXXX
:53A:DEUTDEFFXXX
:57A:BANKDEFFXXX
:58A:CHASUS33XXX
:72:/ACC/URGENT PAYMENT
/INS/SAME DAY VALUE
INTERBANK SETTLEMENT
-}`
  },
  'MT205': {
    name: 'MT205 → ISO 20022 pacs.009.001.08',
    description: 'Financial Institution Transfer COV',
    targetFormat: 'ISO 20022 pacs.009.001.08 XML',
    sample: `{1:F01BANKBEBBAXXX0000000000}
{2:I205BANKDEFFXXXXN}
{3:{113:SEPA}{121:180f1e65-90e0-44d5-a49a-92b55eb3025f}}
{4:
:20:COV205123456789
:21:RELATED987654321
:32A:250615EUR750000,00
:52A:BANKBEBBXXX
:53A:DEUTDEFFXXX
:57A:BANKDEFFXXX
:58A:CHASUS33XXX
:72:/ACC/COVER PAYMENT
/INS/UNDERLYING CUSTOMER TRANSFER
SWIFT COV PROCESSING
-}`
  },
  'MT292': {
    name: 'MT292 → ISO 20022 camt.056.001.08',
    description: 'Request for Cancellation (Institution)',
    targetFormat: 'ISO 20022 camt.056.001.08 XML',
    sample: `{1:F01BANKBEBBAXXX0000000000}
{2:I292BANKDEFFXXXXN}
{3:{113:CBPR}{121:123e4567-e89b-12d3-a456-426614174000}}
{4:
:20:CANC292123456789
:21:ORIG987654321
:11R:pacs.009.001.08,20220625140530
:11S:10324062112341234
:11T:ETOEID20220625001
:13C:/RNCTIME/1405+0200
:32A:220625USD25000,00
:52A:BANKUS33XXX
:57A:BANKGB2LXXX
:79:Institution requested cancellation due to 
duplicate payment instruction. Please 
process immediate cancellation.
-}`
  },
  'MT296': {
    name: 'MT296 → ISO 20022 pacs.002.001.10',
    description: 'Institution Side Rejection',
    targetFormat: 'ISO 20022 pacs.002.001.10 XML',
    sample: `{1:F01BANKGB2LXXX0000000000}{2:I296BANKUS33XXXXN}{3:{103:EBA}{108:FIREJT296}{111:001}{119:STP}{121:87654321-5678-9012-3456-789012345678}}{4:
:20:FI20240101296
:21:FI20240101095
:11R:240101BANKGB2L130800/pacs.009.001.08
:11S:20324062112341234
:11T:ETOEID20240101001
:13C:/RNCTIME/1308+0000
:32A:240101GBP750000,00
:52A:BANKGB2LXXX
:57A:BANKUS33XXX
:72:/REJT/
/MREF/FI20240101095
/TREF/E2E-REF-2024-001
/ReasonCode/AC01
/TEXT/INSUFFICIENT FUNDS
-}{5:{MAC:12345678}{CHK:123456789ABC}}`
  },
  'MT900': {
    name: 'MT900 → ISO 20022 camt.054.001.08',
    description: 'Confirmation of Debit',
    targetFormat: 'ISO 20022 camt.054.001.08 XML',
    sample: `{1:F01BANKBEBBAXXX0000000000}
{2:I900BANKDEFFXXXXN}
{3:{113:SEPA}{121:180f1e65-90e0-44d5-a49a-92b55eb3025f}}
{4:
:20:DBT900123456789
:21:RELATED987654321
:25:BANKBEBBXXX/ACC123456789
:32A:250615EUR123456,78
:52A:BANKBEBBXXX
:57A:BANKDEFFXXX
:59:/9876543210
BENEFICIARY COMPANY INC
456 BROADWAY
NEW YORK NY 10013 US
:70:DEBIT CONFIRMATION
PAYMENT PROCESSED
-}`
  },
  'MT910': {
    name: 'MT910 → ISO 20022 camt.054.001.08',
    description: 'Confirmation of Credit',
    targetFormat: 'ISO 20022 camt.054.001.08 XML',
    sample: `{1:F01BANKBEBBAXXX0000000000}
{2:I910BANKDEFFXXXXN}
{3:{113:SEPA}{121:180f1e65-90e0-44d5-a49a-92b55eb3025f}}
{4:
:20:CRD910123456789
:21:RELATED987654321
:25:BANKBEBBXXX/ACC123456789
:32A:250615EUR123456,78
:52A:BANKBEBBXXX
:57A:BANKDEFFXXX
:50K:/1234567890
CUSTOMER COMPANY INC
123 MAIN STREET
NEW YORK NY 10001 US
:70:CREDIT CONFIRMATION
PAYMENT RECEIVED
-}`
  }
};

// Reverse transformation configurations (MX to MT)
export const REVERSE_TRANSFORMATIONS = {
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
          <OrgnlMsgId>20250627-12345</OrgnlMsgId>
          <OrgnlMsgNmId>pacs.009.001.08</OrgnlMsgNmId>
          <OrgnlCreDtTm>2025-06-27T12:00:00Z</OrgnlCreDtTm>
        </OrgnlGrpInf>
        <OrgnlInstrId>FI21001234567890</OrgnlInstrId>
        <OrgnlEndToEndId>E2E-REF-2025-202</OrgnlEndToEndId>
        <OrgnlTxId>TXN-2025-202234</OrgnlTxId>
        <OrgnlUETR>12345678-1234-4567-8901-123456789012</OrgnlUETR>
        <TxSts>RJCT</TxSts>
        <StsRsnInf>
          <Rsn>
            <Cd>AC01</Cd>
          </Rsn>
          <AddtlInf>ACCOUNT IDENTIFIER INCORRECT</AddtlInf>
          <AddtlInf>FINANCIAL INSTITUTION ACCOUNT INVALID</AddtlInf>
        </StsRsnInf>
        <OrgnlTxRef>
          <IntrBkSttlmAmt Ccy="USD">2000000.00</IntrBkSttlmAmt>
          <IntrBkSttlmDt>2025-06-27</IntrBkSttlmDt>
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
    name: 'ISO 20022 pacs.004 → MT103RETN/MT202RETN/MT205RETN',
    description: 'Payment Return',
    targetFormat: 'SWIFT MT103RETN/MT202RETN/MT205RETN',
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
          <BICFI>BANKBEBBXXX</BICFI>
        </FinInstnId>
      </FIId>
    </To>
    <BizMsgIdr>20250630-RETN-001</BizMsgIdr>
    <MsgDefIdr>pacs.004.001.09</MsgDefIdr>
    <CreDt>2025-06-30T14:30:00Z</CreDt>
    <BizSvc>CBPR</BizSvc>
  </AppHdr>
  <Document xmlns="urn:iso:std:iso:20022:tech:xsd:pacs.004.001.09">
    <PmtRtr>
      <GrpHdr>
        <MsgId>RETN20250630001</MsgId>
        <CreDtTm>2025-06-30T14:30:00Z</CreDtTm>
        <InstgAgt>
          <FinInstnId>
            <BICFI>BANKDEFFXXX</BICFI>
          </FinInstnId>
        </InstgAgt>
        <InstdAgt>
          <FinInstnId>
            <BICFI>BANKBEBBXXX</BICFI>
          </FinInstnId>
        </InstdAgt>
        <SttlmInf>
          <SttlmMtd>INDA</SttlmMtd>
          <SttlmAcct>
            <Id>
              <IBAN>DE89370400440532013000</IBAN>
            </Id>
          </SttlmAcct>
        </SttlmInf>
      </GrpHdr>
      <TxInf>
        <RtrId>RTR001</RtrId>
        <OrgnlGrpInf>
          <OrgnlMsgId>20250627-12345</OrgnlMsgId>
          <OrgnlMsgNmId>pacs.008.001.08</OrgnlMsgNmId>
          <OrgnlCreDtTm>2025-06-27T12:00:00Z</OrgnlCreDtTm>
        </OrgnlGrpInf>
        <OrgnlInstrId>FT21001234567890</OrgnlInstrId>
        <OrgnlEndToEndId>E2E-REF-2025-001</OrgnlEndToEndId>
        <OrgnlTxId>TXN-2025-001234</OrgnlTxId>
        <OrgnlUETR>12345678-1234-4567-8901-123456789012</OrgnlUETR>
        <RtrdIntrBkSttlmAmt Ccy="EUR">123456.78</RtrdIntrBkSttlmAmt>
        <IntrBkSttlmDt>2025-06-30</IntrBkSttlmDt>
        <RtrRsnInf>
          <Rsn>
            <Cd>AC01</Cd>
          </Rsn>
          <AddtlInf>ACCOUNT IDENTIFIER INCORRECT</AddtlInf>
          <AddtlInf>BENEFICIARY ACCOUNT NUMBER INVALID</AddtlInf>
        </RtrRsnInf>
        <OrgnlTxRef>
          <IntrBkSttlmAmt Ccy="EUR">123456.78</IntrBkSttlmAmt>
          <IntrBkSttlmDt>2025-06-27</IntrBkSttlmDt>
          <InstgAgt>
            <FinInstnId>
              <BICFI>BANKBEBBXXX</BICFI>
            </FinInstnId>
          </InstgAgt>
          <InstdAgt>
            <FinInstnId>
              <BICFI>BANKDEFFXXX</BICFI>
            </FinInstnId>
          </InstdAgt>
          <DbtrAgt>
            <FinInstnId>
              <BICFI>BANKBEBBXXX</BICFI>
            </FinInstnId>
          </DbtrAgt>
          <CdtrAgt>
            <FinInstnId>
              <BICFI>BANKDEFFXXX</BICFI>
            </FinInstnId>
          </CdtrAgt>
          <Dbtr>
            <Nm>John Doe</Nm>
            <PstlAdr>
              <StrtNm>123 Street</StrtNm>
              <TwnNm>City</TwnNm>
              <Ctry>BE</Ctry>
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
            <Nm>Jane Smith</Nm>
            <PstlAdr>
              <StrtNm>456 Avenue</StrtNm>
              <TwnNm>Another City</TwnNm>
              <Ctry>DE</Ctry>
            </PstlAdr>
          </Cdtr>
          <CdtrAcct>
            <Id>
              <Othr>
                <Id>9876543210</Id>
              </Othr>
            </Id>
          </CdtrAcct>
          <RmtInf>
            <Ustrd>RETURN OF FUNDS</Ustrd>
          </RmtInf>
        </OrgnlTxRef>
        <RtrdInstdAmt Ccy="EUR">123456.78</RtrdInstdAmt>
        <InstgAgt>
          <FinInstnId>
            <BICFI>BANKDEFFXXX</BICFI>
          </FinInstnId>
        </InstgAgt>
        <InstdAgt>
          <FinInstnId>
            <BICFI>BANKBEBBXXX</BICFI>
          </FinInstnId>
        </InstdAgt>
      </TxInf>
    </PmtRtr>
  </Document>
</Envelope>`
  },
  'pacs.008': {
    name: 'ISO 20022 pacs.008 → MT103',
    description: 'Customer Credit Transfer Initiation',
    targetFormat: 'SWIFT MT103',
    sample: `<?xml version="1.0" encoding="UTF-8"?>
<Envelope xmlns="urn:swift:xsd:$ahV10">
  <AppHdr>
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
          <BICFI>BANKDEFFXXX</BICFI>
        </FinInstnId>
      </FIId>
    </To>
    <BizMsgIdr>20250630-103-001</BizMsgIdr>
    <MsgDefIdr>pacs.008.001.08</MsgDefIdr>
    <CreDt>2025-06-30T14:30:00Z</CreDt>
    <BizSvc>CBPR</BizSvc>
  </AppHdr>
  <Document xmlns="urn:iso:std:iso:20022:tech:xsd:pacs.008.001.08">
    <FIToFICstmrCdtTrf>
      <GrpHdr>
        <MsgId>103-20250630001</MsgId>
        <CreDtTm>2025-06-30T14:30:00Z</CreDtTm>
        <NbOfTxs>1</NbOfTxs>
        <TtlIntrBkSttlmAmt Ccy="EUR">123456.78</TtlIntrBkSttlmAmt>
        <IntrBkSttlmDt>2025-06-15</IntrBkSttlmDt>
        <SttlmInf>
          <SttlmMtd>INDA</SttlmMtd>
          <SttlmAcct>
            <Id>
              <IBAN>DE89370400440532013000</IBAN>
            </Id>
          </SttlmAcct>
        </SttlmInf>
        <InstgAgt>
          <FinInstnId>
            <BICFI>BANKBEBBXXX</BICFI>
          </FinInstnId>
        </InstgAgt>
        <InstdAgt>
          <FinInstnId>
            <BICFI>BANKDEFFXXX</BICFI>
          </FinInstnId>
        </InstdAgt>
      </GrpHdr>
      <CdtTrfTxInf>
        <PmtId>
          <InstrId>REF123456789</InstrId>
          <EndToEndId>E2E-REF-2025-001</EndToEndId>
          <TxId>TXN-2025-001234</TxId>
          <UETR>180f1e65-90e0-44d5-a49a-92b55eb3025f</UETR>
        </PmtId>
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
        <IntrBkSttlmAmt Ccy="EUR">123456.78</IntrBkSttlmAmt>
        <IntrBkSttlmDt>2025-06-15</IntrBkSttlmDt>
        <ChrgBr>OUR</ChrgBr>
        <ChrgsInf>
          <Amt Ccy="EUR">25.00</Amt>
          <Agt>
            <FinInstnId>
              <BICFI>BANKBEBBXXX</BICFI>
            </FinInstnId>
          </Agt>
          <Tp>
            <Cd>CRED</Cd>
          </Tp>
        </ChrgsInf>
        <InstgAgt>
          <FinInstnId>
            <BICFI>BANKBEBBXXX</BICFI>
          </FinInstnId>
        </InstgAgt>
        <InstdAgt>
          <FinInstnId>
            <BICFI>BANKDEFFXXX</BICFI>
          </FinInstnId>
        </InstdAgt>
        <IntrmyAgt1>
          <FinInstnId>
            <BICFI>BNPAFRPPXXX</BICFI>
          </FinInstnId>
        </IntrmyAgt1>
        <Dbtr>
          <Nm>John Doe</Nm>
          <PstlAdr>
            <StrtNm>123 Street</StrtNm>
            <TwnNm>City</TwnNm>
            <Ctry>BE</Ctry>
          </PstlAdr>
        </Dbtr>
        <DbtrAcct>
          <Id>
            <Othr>
              <Id>1234567890</Id>
            </Othr>
          </Id>
        </DbtrAcct>
        <DbtrAgt>
          <FinInstnId>
            <BICFI>BANKBEBBXXX</BICFI>
            <Nm>ORDERING BANK NAME</Nm>
            <PstlAdr>
              <StrtNm>123 BANK STREET</StrtNm>
              <TwnNm>BRUSSELS</TwnNm>
              <Ctry>BE</Ctry>
            </PstlAdr>
          </FinInstnId>
        </DbtrAgt>
        <CdtrAgt>
          <FinInstnId>
            <BICFI>BANKDEFFXXX</BICFI>
          </FinInstnId>
        </CdtrAgt>
        <Cdtr>
          <Nm>Jane Smith</Nm>
          <PstlAdr>
            <StrtNm>456 Avenue</StrtNm>
            <TwnNm>Another City</TwnNm>
            <Ctry>DE</Ctry>
          </PstlAdr>
        </Cdtr>
        <CdtrAcct>
          <Id>
            <Othr>
              <Id>9876543210</Id>
            </Othr>
          </Id>
        </CdtrAcct>
        <RmtInf>
          <Ustrd>INVOICE 45678</Ustrd>
          <Ustrd>PAYMENT FOR SERVICES</Ustrd>
          <Ustrd>RENDERED IN DECEMBER</Ustrd>
          <Ustrd>WITH ADDITIONAL NOTES</Ustrd>
        </RmtInf>
        <InstrForDbtrAgt>
          <InstrInf>/ACC/MANUAL PROCESSING REQUIRED</InstrInf>
          <InstrInf>/INS/SPECIAL HANDLING NEEDED</InstrInf>
          <InstrInf>REQUIRES COMPLIANCE REVIEW</InstrInf>
        </InstrForDbtrAgt>
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
<Envelope xmlns="urn:swift:xsd:$ahV10">
  <AppHdr>
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
          <BICFI>BANKDEFFXXX</BICFI>
        </FinInstnId>
      </FIId>
    </To>
    <BizMsgIdr>20250630-202-001</BizMsgIdr>
    <MsgDefIdr>pacs.009.001.08</MsgDefIdr>
    <CreDt>2025-06-30T14:30:00Z</CreDt>
    <BizSvc>CBPR</BizSvc>
  </AppHdr>
  <Document xmlns="urn:iso:std:iso:20022:tech:xsd:pacs.009.001.08">
    <FIToFICstmrCdtTrf>
      <GrpHdr>
        <MsgId>202-20250630001</MsgId>
        <CreDtTm>2025-06-30T14:30:00Z</CreDtTm>
        <NbOfTxs>1</NbOfTxs>
        <TtlIntrBkSttlmAmt Ccy="EUR">500000.00</TtlIntrBkSttlmAmt>
        <IntrBkSttlmDt>2025-06-15</IntrBkSttlmDt>
        <SttlmInf>
          <SttlmMtd>INDA</SttlmMtd>
          <SttlmAcct>
            <Id>
              <IBAN>DE89370400440532013000</IBAN>
            </Id>
          </SttlmAcct>
        </SttlmInf>
        <InstgAgt>
          <FinInstnId>
            <BICFI>BANKBEBBXXX</BICFI>
          </FinInstnId>
        </InstgAgt>
        <InstdAgt>
          <FinInstnId>
            <BICFI>BANKDEFFXXX</BICFI>
          </FinInstnId>
        </InstdAgt>
      </GrpHdr>
      <CdtTrfTxInf>
        <PmtId>
          <InstrId>FI202123456789</InstrId>
          <EndToEndId>E2E-REF-2025-202</EndToEndId>
          <TxId>TXN-2025-202234</TxId>
          <UETR>180f1e65-90e0-44d5-a49a-92b55eb3025f</UETR>
        </PmtId>
        <PmtTpInf>
          <SvcLvl>
            <Cd>NURG</Cd>
          </SvcLvl>
          <LclInstrm>
            <Cd>SWIFT</Cd>
          </LclInstrm>
        </PmtTpInf>
        <IntrBkSttlmAmt Ccy="EUR">500000.00</IntrBkSttlmAmt>
        <IntrBkSttlmDt>2025-06-15</IntrBkSttlmDt>
        <InstgAgt>
          <FinInstnId>
            <BICFI>BANKBEBBXXX</BICFI>
          </FinInstnId>
        </InstgAgt>
        <InstdAgt>
          <FinInstnId>
            <BICFI>BANKDEFFXXX</BICFI>
          </FinInstnId>
        </InstdAgt>
        <IntrmyAgt1>
          <FinInstnId>
            <BICFI>DEUTDEFFXXX</BICFI>
          </FinInstnId>
        </IntrmyAgt1>
        <DbtrAgt>
          <FinInstnId>
            <BICFI>BANKBEBBXXX</BICFI>
          </FinInstnId>
        </DbtrAgt>
        <CdtrAgt>
          <FinInstnId>
            <BICFI>BANKDEFFXXX</BICFI>
          </FinInstnId>
        </CdtrAgt>
        <CdtrAcct>
          <Id>
            <IBAN>DE89370400440532013000</IBAN>
          </Id>
        </CdtrAcct>
        <InstrForDbtrAgt>
          <InstrInf>/ACC/URGENT PAYMENT</InstrInf>
          <InstrInf>/INS/SAME DAY VALUE</InstrInf>
          <InstrInf>INTERBANK SETTLEMENT</InstrInf>
        </InstrForDbtrAgt>
      </CdtTrfTxInf>
    </FIToFICstmrCdtTrf>
  </Document>
</Envelope>`
  },
  'camt.054': {
    name: 'ISO 20022 camt.054 → MT900/MT910',
    description: 'Bank-to-Customer Debit/Credit Notification',
    targetFormat: 'SWIFT MT900/MT910',
    sample: `<?xml version="1.0" encoding="UTF-8"?>
<Envelope xmlns="urn:swift:xsd:$ahV10">
  <AppHdr>
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
          <BICFI>BANKDEFFXXX</BICFI>
        </FinInstnId>
      </FIId>
    </To>
    <BizMsgIdr>20250630-054-001</BizMsgIdr>
    <MsgDefIdr>camt.054.001.08</MsgDefIdr>
    <CreDt>2025-06-30T14:30:00Z</CreDt>
    <BizSvc>CBPR</BizSvc>
  </AppHdr>
  <Document xmlns="urn:iso:std:iso:20022:tech:xsd:camt.054.001.08">
    <BkToCstmrDbtCdtNtfctn>
      <GrpHdr>
        <MsgId>054-20250630001</MsgId>
        <CreDtTm>2025-06-30T14:30:00Z</CreDtTm>
        <MsgRcpt>
          <Nm>CUSTOMER COMPANY INC</Nm>
          <Id>
            <OrgId>
              <Othr>
                <Id>CUST001</Id>
              </Othr>
            </OrgId>
          </Id>
        </MsgRcpt>
      </GrpHdr>
      <Ntfctn>
        <Id>NTF001</Id>
        <CreDtTm>2025-06-30T14:30:00Z</CreDtTm>
        <Acct>
          <Id>
            <IBAN>BE89370400440532013000</IBAN>
          </Id>
          <Ccy>EUR</Ccy>
          <Ownr>
            <Nm>CUSTOMER COMPANY INC</Nm>
          </Ownr>
          <Svcr>
            <FinInstnId>
              <BICFI>BANKBEBBXXX</BICFI>
            </FinInstnId>
          </Svcr>
        </Acct>
        <Ntry>
          <Amt Ccy="EUR">123456.78</Amt>
          <CdtDbtInd>CRDT</CdtDbtInd>
          <Sts>BOOK</Sts>
          <BookgDt>
            <Dt>2025-06-15</Dt>
          </BookgDt>
          <ValDt>
            <Dt>2025-06-15</Dt>
          </ValDt>
          <BkTxCd>
            <Domn>
              <Cd>PMNT</Cd>
              <Fmly>
                <Cd>RCDT</Cd>
                <SubFmlyCd>ESCT</SubFmlyCd>
              </Fmly>
            </Domn>
          </BkTxCd>
          <NtryDtls>
            <TxDtls>
              <Refs>
                <InstrId>CRD910123456789</InstrId>
                <EndToEndId>E2E-REF-2025-910</EndToEndId>
                <TxId>TXN-2025-910234</TxId>
                <UETR>180f1e65-90e0-44d5-a49a-92b55eb3025f</UETR>
              </Refs>
              <Amt Ccy="EUR">123456.78</Amt>
              <CdtDbtInd>CRDT</CdtDbtInd>
              <BkTxCd>
                <Domn>
                  <Cd>PMNT</Cd>
                  <Fmly>
                    <Cd>RCDT</Cd>
                    <SubFmlyCd>ESCT</SubFmlyCd>
                  </Fmly>
                </Domn>
              </BkTxCd>
              <RltdPties>
                <Dbtr>
                  <Nm>CUSTOMER COMPANY INC</Nm>
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
                <DbtrAgt>
                  <FinInstnId>
                    <BICFI>BANKBEBBXXX</BICFI>
                  </FinInstnId>
                </DbtrAgt>
                <CdtrAgt>
                  <FinInstnId>
                    <BICFI>BANKDEFFXXX</BICFI>
                  </FinInstnId>
                </CdtrAgt>
              </RltdPties>
              <RmtInf>
                <Ustrd>CREDIT CONFIRMATION</Ustrd>
                <Ustrd>PAYMENT RECEIVED</Ustrd>
              </RmtInf>
            </TxDtls>
          </NtryDtls>
        </Ntry>
      </Ntfctn>
    </BkToCstmrDbtCdtNtfctn>
  </Document>
</Envelope>`
  },
  'camt.056': {
    name: 'ISO 20022 camt.056 → MT192/MT292',
    description: 'Cancellation Request',
    targetFormat: 'SWIFT MT192/MT292',
    sample: `<?xml version="1.0" encoding="UTF-8"?>
<Envelope xmlns="urn:swift:xsd:$ahV10">
  <AppHdr>
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
          <BICFI>BANKDEFFXXX</BICFI>
        </FinInstnId>
      </FIId>
    </To>
    <BizMsgIdr>20250630-056-001</BizMsgIdr>
    <MsgDefIdr>camt.056.001.08</MsgDefIdr>
    <CreDt>2025-06-30T14:30:00Z</CreDt>
    <BizSvc>CBPR</BizSvc>
  </AppHdr>
  <Document xmlns="urn:iso:std:iso:20022:tech:xsd:camt.056.001.08">
    <FIToFIPmtCxlReq>
      <GrpHdr>
        <MsgId>056-20250630001</MsgId>
        <CreDtTm>2025-06-30T14:30:00Z</CreDtTm>
        <InstgAgt>
          <FinInstnId>
            <BICFI>BANKBEBBXXX</BICFI>
          </FinInstnId>
        </InstgAgt>
        <InstdAgt>
          <FinInstnId>
            <BICFI>BANKDEFFXXX</BICFI>
          </FinInstnId>
        </InstdAgt>
      </GrpHdr>
      <TxInf>
        <CxlId>CANC123456789</CxlId>
        <OrgnlGrpInf>
          <OrgnlMsgId>pacs.008.001.08</OrgnlMsgId>
          <OrgnlMsgNmId>20220625140530</OrgnlMsgNmId>
          <OrgnlCreDtTm>2022-06-25T14:05:30Z</OrgnlCreDtTm>
        </OrgnlGrpInf>
        <OrgnlInstrId>10324062112341234</OrgnlInstrId>
        <OrgnlEndToEndId>ETOEID20220625001</OrgnlEndToEndId>
        <OrgnlTxId>TXN-2022-001234</OrgnlTxId>
        <OrgnlUETR>123e4567-e89b-12d3-a456-426614174000</OrgnlUETR>
        <OrgnlIntrBkSttlmAmt Ccy="USD">25000.00</OrgnlIntrBkSttlmAmt>
        <OrgnlIntrBkSttlmDt>2022-06-25</OrgnlIntrBkSttlmDt>
        <CxlRsnInf>
          <Rsn>
            <Cd>DUPL</Cd>
          </Rsn>
          <AddtlInf>Customer requested cancellation due to</AddtlInf>
          <AddtlInf>duplicate payment instruction. Please</AddtlInf>
          <AddtlInf>process immediate cancellation.</AddtlInf>
        </CxlRsnInf>
        <OrgnlTxRef>
          <IntrBkSttlmAmt Ccy="USD">25000.00</IntrBkSttlmAmt>
          <IntrBkSttlmDt>2022-06-25</IntrBkSttlmDt>
          <InstgAgt>
            <FinInstnId>
              <BICFI>BANKUS33XXX</BICFI>
            </FinInstnId>
          </InstgAgt>
          <InstdAgt>
            <FinInstnId>
              <BICFI>BANKGB2LXXX</BICFI>
            </FinInstnId>
          </InstdAgt>
        </OrgnlTxRef>
        <InstgAgt>
          <FinInstnId>
            <BICFI>BANKBEBBXXX</BICFI>
          </FinInstnId>
        </InstgAgt>
        <InstdAgt>
          <FinInstnId>
            <BICFI>BANKDEFFXXX</BICFI>
          </FinInstnId>
        </InstdAgt>
      </TxInf>
    </FIToFIPmtCxlReq>
  </Document>
</Envelope>`
  }
};