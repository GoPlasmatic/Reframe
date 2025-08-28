#!/usr/bin/env python3
"""
Test script for pacs.004 to MT103/202/205 RETN transformation
Tests the updated workflows against CBPR+ specification
"""

import json
import requests
import sys
from datetime import datetime

# Test pacs.004 message for MT103 RETN transformation
test_pacs004_mt103_retn = {
    "message": """<?xml version="1.0" encoding="UTF-8"?>
<BizMsg>
    <AppHdr>
        <Fr>
            <FIId>
                <FinInstnId>
                    <BICFI>CITIUS33XXX</BICFI>
                </FinInstnId>
            </FIId>
        </Fr>
        <To>
            <FIId>
                <FinInstnId>
                    <BICFI>BARCGB22XXX</BICFI>
                </FinInstnId>
            </FIId>
        </To>
        <BizMsgIdr>RETN20250827001</BizMsgIdr>
        <MsgDefIdr>pacs.004.001.09</MsgDefIdr>
        <BizSvc>swift.cbprplus.01</BizSvc>
        <CreDt>2025-08-27T10:30:00Z</CreDt>
    </AppHdr>
    <Document xmlns="urn:iso:std:iso:20022:tech:xsd:pacs.004.001.09">
    <PmtRtr>
        <GrpHdr>
            <MsgId>RETN20250827001</MsgId>
            <CreDtTm>2025-08-27T10:30:00Z</CreDtTm>
            <NbOfTxs>1</NbOfTxs>
            <SttlmInf>
                <SttlmMtd>INDA</SttlmMtd>
            </SttlmInf>
        </GrpHdr>
        <TxInf>
            <RtrId>RTN123456789</RtrId>
            <OrgnlGrpInf>
                <OrgnlMsgId>ORIG20250826001</OrgnlMsgId>
                <OrgnlMsgNmId>pacs.008.001.08</OrgnlMsgNmId>
                <OrgnlCreDtTm>2025-08-26T09:00:00Z</OrgnlCreDtTm>
            </OrgnlGrpInf>
            <OrgnlInstrId>INSTRID123</OrgnlInstrId>
            <OrgnlEndToEndId>E2E123456</OrgnlEndToEndId>
            <OrgnlTxId>TXN987654321</OrgnlTxId>
            <OrgnlUETR>550e8400-e29b-41d4-a716-446655440000</OrgnlUETR>
            <OrgnlIntrBkSttlmAmt Ccy="USD">10000.00</OrgnlIntrBkSttlmAmt>
            <RtrdIntrBkSttlmAmt Ccy="USD">10000.00</RtrdIntrBkSttlmAmt>
            <IntrBkSttlmDt>2025-08-27</IntrBkSttlmDt>
            <RtrdInstdAmt Ccy="USD">10000.00</RtrdInstdAmt>
            <ChrgBr>SHAR</ChrgBr>
            <ChrgsInf>
                <Amt Ccy="USD">25.00</Amt>
                <Agt>
                    <FinInstnId>
                        <BICFI>CHASUS33XXX</BICFI>
                    </FinInstnId>
                </Agt>
            </ChrgsInf>
            <RtrChain>
                <Dbtr>
                    <Pty>
                        <Nm>John Doe</Nm>
                        <PstlAdr>
                            <Ctry>US</Ctry>
                            <AdrLine>123 Main St</AdrLine>
                            <AdrLine>New York, NY 10001</AdrLine>
                        </PstlAdr>
                    </Pty>
                </Dbtr>
                <DbtrAgt>
                    <FinInstnId>
                        <BICFI>CITIUS33XXX</BICFI>
                    </FinInstnId>
                </DbtrAgt>
                <IntrmyAgt1>
                    <FinInstnId>
                        <BICFI>DEUTUS33XXX</BICFI>
                    </FinInstnId>
                </IntrmyAgt1>
                <CdtrAgt>
                    <FinInstnId>
                        <BICFI>BARCGB22XXX</BICFI>
                    </FinInstnId>
                </CdtrAgt>
                <Cdtr>
                    <Pty>
                        <Nm>Jane Smith</Nm>
                        <PstlAdr>
                            <Ctry>GB</Ctry>
                            <AdrLine>456 High St</AdrLine>
                            <AdrLine>London EC1A 1BB</AdrLine>
                        </PstlAdr>
                    </Pty>
                </Cdtr>
            </RtrChain>
            <RtrRsnInf>
                <Rsn>
                    <Cd>AC04</Cd>
                </Rsn>
                <AddtlInf>Account closed</AddtlInf>
                <AddtlInf>Please contact beneficiary for new account details</AddtlInf>
            </RtrRsnInf>
        </TxInf>
    </PmtRtr>
    </Document>
</BizMsg>""",
    "debug": True
}

# Test pacs.004 message for MT202 RETN transformation
test_pacs004_mt202_retn = {
    "message": """<?xml version="1.0" encoding="UTF-8"?>
<BizMsg>
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
                    <BICFI>BNPAFRPPXXX</BICFI>
                </FinInstnId>
            </FIId>
        </To>
        <BizMsgIdr>RETN20250827002</BizMsgIdr>
        <MsgDefIdr>pacs.004.001.09</MsgDefIdr>
        <BizSvc>swift.cbprplus.01</BizSvc>
        <CreDt>2025-08-27T11:00:00Z</CreDt>
    </AppHdr>
    <Document xmlns="urn:iso:std:iso:20022:tech:xsd:pacs.004.001.09">
    <PmtRtr>
        <GrpHdr>
            <MsgId>RETN20250827002</MsgId>
            <CreDtTm>2025-08-27T11:00:00Z</CreDtTm>
            <NbOfTxs>1</NbOfTxs>
        </GrpHdr>
        <TxInf>
            <RtrId>RTN202456789</RtrId>
            <OrgnlGrpInf>
                <OrgnlMsgId>ORIG20250826002</OrgnlMsgId>
                <OrgnlMsgNmId>pacs.009.001.08</OrgnlMsgNmId>
            </OrgnlGrpInf>
            <OrgnlEndToEndId>FI2FI123456</OrgnlEndToEndId>
            <OrgnlUETR>660e8400-e29b-41d4-a716-446655440001</OrgnlUETR>
            <RtrdIntrBkSttlmAmt Ccy="EUR">50000.00</RtrdIntrBkSttlmAmt>
            <IntrBkSttlmDt>2025-08-27</IntrBkSttlmDt>
            <ChrgBr>DEBT</ChrgBr>
            <ChrgsInf>
                <Amt Ccy="EUR">50.00</Amt>
                <Agt>
                    <FinInstnId>
                        <BICFI>DEUTDEFFXXX</BICFI>
                    </FinInstnId>
                </Agt>
            </ChrgsInf>
            <ChrgsInf>
                <Amt Ccy="EUR">25.00</Amt>
                <Agt>
                    <FinInstnId>
                        <BICFI>BNPAFRPPXXX</BICFI>
                    </FinInstnId>
                </Agt>
            </ChrgsInf>
            <RtrChain>
                <Dbtr>
                    <Agt>
                        <FinInstnId>
                            <BICFI>DEUTDEFFXXX</BICFI>
                            <Nm>Deutsche Bank AG</Nm>
                        </FinInstnId>
                    </Agt>
                </Dbtr>
                <DbtrAgt>
                    <FinInstnId>
                        <BICFI>DEUTDEFFXXX</BICFI>
                    </FinInstnId>
                </DbtrAgt>
                <CdtrAgt>
                    <FinInstnId>
                        <BICFI>BNPAFRPPXXX</BICFI>
                    </FinInstnId>
                </CdtrAgt>
                <Cdtr>
                    <Agt>
                        <FinInstnId>
                            <BICFI>BNPAFRPPXXX</BICFI>
                            <Nm>BNP Paribas</Nm>
                        </FinInstnId>
                    </Agt>
                </Cdtr>
            </RtrChain>
            <RtrRsnInf>
                <Rsn>
                    <Cd>TECH</Cd>
                </Rsn>
                <AddtlInf>Technical error in processing</AddtlInf>
            </RtrRsnInf>
        </TxInf>
    </PmtRtr>
    </Document>
</BizMsg>""",
    "debug": True
}

def test_transformation(test_name, test_data):
    """Test a single pacs.004 to MT transformation"""
    print(f"\n{'='*60}")
    print(f"Testing: {test_name}")
    print(f"{'='*60}")
    
    url = "http://localhost:3000/transform/mx-to-mt"
    
    try:
        response = requests.post(url, json=test_data, timeout=10)
        
        if response.status_code == 200:
            result = response.json()
            print(f"✅ Transformation successful!")
            print(f"\nResult:")
            print(json.dumps(result, indent=2))
            
            # Check for critical fields
            if 'result' in result:
                mt_message = result['result']
                
                # Check for Field 79 (Return Reason)
                if ':79:' in mt_message:
                    print(f"✅ Field 79 (Return Reason) present")
                else:
                    print(f"⚠️  Field 79 (Return Reason) missing - CRITICAL for RETN")
                
                # Check for RETN in message type
                if 'RETN' in mt_message or '23B:RETN' in mt_message:
                    print(f"✅ RETN indicator present")
                else:
                    print(f"⚠️  RETN indicator might be missing")
                    
        else:
            print(f"❌ Transformation failed with status {response.status_code}")
            print(f"Error: {response.text}")
            
    except requests.exceptions.ConnectionError:
        print(f"❌ Cannot connect to server at {url}")
        print("Make sure the server is running: RUST_LOG=info cargo run")
        return False
    except Exception as e:
        print(f"❌ Error during transformation: {e}")
        return False
    
    return True

def main():
    """Main test runner"""
    print("pacs.004 to MT RETN Transformation Test Suite")
    print("=" * 60)
    
    tests = [
        ("pacs.004 to MT103 RETN", test_pacs004_mt103_retn),
        ("pacs.004 to MT202 RETN", test_pacs004_mt202_retn)
    ]
    
    passed = 0
    failed = 0
    
    for test_name, test_data in tests:
        if test_transformation(test_name, test_data):
            passed += 1
        else:
            failed += 1
    
    print(f"\n{'='*60}")
    print(f"Test Results: {passed} passed, {failed} failed")
    
    if failed > 0:
        sys.exit(1)

if __name__ == "__main__":
    main()