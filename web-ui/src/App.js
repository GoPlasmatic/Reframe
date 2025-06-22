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
} from '@mantine/core';
import {
  IconTransform,
  IconCode,
  IconCheck,
  IconAlertCircle,
  IconPlayerPlay,
  IconRefresh,
} from '@tabler/icons-react';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { vscDarkPlus } from 'react-syntax-highlighter/dist/esm/styles/prism';

// API Configuration - using relative URL since we're serving from the same origin
const API_ENDPOINT = '/reframe';

// Transformation configurations
const TRANSFORMATIONS = {
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
  'MT103 STP': {
    name: 'MT103 STP → ISO 20022 pacs.008.001.08',
    description: 'Straight Through Processing Customer Transfer',
    targetFormat: 'ISO 20022 pacs.008.001.08 XML (STP)',
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
  'MT103 REJT': {
    name: 'MT103 REJT → ISO 20022 pacs.002.001.10',
    description: 'Customer Transfer Rejection',
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
  'MT103 RETN': {
    name: 'MT103 RETN → ISO 20022 pacs.004.001.09',
    description: 'Customer Transfer Return',
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
  'MT202 COV': {
    name: 'MT202 COV → ISO 20022 pacs.009.001.08 COVE',
    description: 'Cover Payment using Correspondent Banks',
    targetFormat: 'ISO 20022 pacs.009.001.08 COVE XML',
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
  'MT202 REJT': {
    name: 'MT202 REJT → ISO 20022 pacs.002.001.10',
    description: 'Interbank Transfer Rejection',
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
  'MT202 RETN': {
    name: 'MT202 RETN → ISO 20022 pacs.004.001.09',
    description: 'Interbank Transfer Return',
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
    name: 'MT205 → ISO 20022 pacs.009.001.08',
    description: 'Corporate Financial Institution Transfer',
    targetFormat: 'ISO 20022 pacs.009.001.08 XML',
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
  'MT205 COV': {
    name: 'MT205 COV → ISO 20022 pacs.009.001.08 COVE',
    description: 'Corporate Cover Payment using Correspondent Banks',
    targetFormat: 'ISO 20022 pacs.009.001.08 COVE XML',
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
  'MT205 REJT': {
    name: 'MT205 REJT → ISO 20022 pacs.002.001.10',
    description: 'Corporate Transfer Rejection',
    targetFormat: 'ISO 20022 pacs.002.001.10 XML',
    sample: `{1:F01CORPDEFAXXX0000000000}
{2:I205CORPBEBBXXXXN}
{3:{108:MT205REJT}{121:987fcdeb-51a2-34b5-6789-426614174abd}}
{4:
:20:REJ2024123789457
:21:CP2024987654322
:32A:241218GBP850000,00
:52A:CORPDEFAXXX
:53D:CORPORATE CORRESPONDENT BANK
CORPORATE BANKING DIVISION
FRANKFURT AM MAIN GERMANY
:56D:CORPORATE INTERMEDIARY BANK LTD
567 CORPORATE BANKING STREET
LONDON UNITED KINGDOM
:57D:CORPORATE ACCOUNT INSTITUTION
890 CORPORATE FINANCE AVENUE
BRUSSELS BELGIUM
:58A:CORPBEBBXXX
:72:/REJT/
/MREF/CP2024987654322
/RREF/REJ2024123789457
/AC01/CORPORATE ACCOUNT IDENTIFIER
/TEXT/INVALID CORPORATE ACCOUNT
INCORRECT ACCOUNT INFORMATION
-}`
  },
  'MT205 RETN': {
    name: 'MT205 RETN → ISO 20022 pacs.004.001.09',
    description: 'Corporate Transfer Return',
    targetFormat: 'ISO 20022 pacs.004.001.09 XML',
    sample: `{1:F01CORPBEBBAXXX0000000000}
{2:I205CORPUS33XXXXN}
{3:{108:MT205RETN}{121:456a789b-12c3-45d6-e789-012345678de0}}
{4:
:20:RET2024567891235
:21:CP2024123456790
:32A:241219USD1150000,00
:52A:CORPBEBBXXX
:53B:/CH/1234567891
CORPORATE CORRESPONDENT LOCATION
:56A:BNPAFRPPXXX
:57B:/IBAN/GB29BARC20001234567891
CORPORATE BARCLAYS LOCATION
:58A:CORPUS33
:72:/RETN/
/RTRN/INSUFFICIENT CORPORATE FUNDS
/MREF/CP2024123456790
/RREF/RET2024567891235
/RC/AG01/CORPORATE TRANSACTION
/TEXT/CORPORATE RETURN INSUFFICIENT
-}`
  },
  'MT103 Detailed': {
    name: 'MT103 → ISO 20022 pacs.008.001.08',
    description: 'Customer Credit Transfer (Detailed Sample)',
    targetFormat: 'ISO 20022 pacs.008.001.08 XML',
    sample: `{1:F01BANKBEBBAXXX0000000000}
{2:I103BANKDEFFXXXXN}
{3:{121:180f1e65-90e0-44d5-a49a-92b55eb3025f}}
{4:
:20:REF123456789
:23B:CRED
:32A:250615EUR123456,78
:50K:/1234567890
John Doe
123 Street
City, Country
:52D:NOTPROVIDED
Ordering Institution Name
Some Address Line 1
Some Address Line 2
:53D:NOTPROVIDED
Sender's Correspondent Name
Correspondent Address 1
Correspondent Address 2
:56D:NOTPROVIDED
Intermediary Institution Name
Intermediary Address 1
Intermediary Address 2
:57D:NOTPROVIDED
Account With Institution Name
Account Institution Address 1
Account Institution Address 2
:59:Jane Smith
456 Avenue
Another City, Country
:70:INVOICE 45678
Multiple lines of remittance
information that exceeds
the normal limits for STP
:71A:OUR
-}`
  },
  'MT202 Minimal': {
    name: 'MT202 → ISO 20022 pacs.009.001.08',
    description: 'Financial Institution Transfer (Minimal)',
    targetFormat: 'ISO 20022 pacs.009.001.08 XML',
    sample: `{1:F01BANKBEBBAXXX0000000000}
{2:I202BANKDEFFXXXXN}
{3:{108:MT202}{121:550e8400-e29b-41d4-a716-446655440000}}
{4:
:20:FI2024123456789
:21:REL2024987654321
:32A:241215USD2500000,00
:58A:CHASUS33XXX
-}`
  },
  'MT202 Serial': {
    name: 'MT202 → ISO 20022 pacs.009.001.08',
    description: 'Financial Institution Transfer (Serial)',
    targetFormat: 'ISO 20022 pacs.009.001.08 XML',
    sample: `{1:F01BANKBEBBAXXX0000000000}
{2:I202DEUTDEFFXXXXN}
{3:{108:MT202}{121:111a222b-33c4-44d5-e666-777888999aaa}}
{4:
:20:SER2024567123890
:21:SERIAL2024890123
:13C:/SNDTIME/0900+0100
/RNCTIME/1500+0100
:32A:241223EUR3250000,00
:52A:BANKBEBBXXX
:53A:BANKBEBBXXX
:54A:DEUTDEFFXXX
:56D:INTERMEDIARY BANK GMBH
FINANCIAL SERVICES DIVISION
MUNICH BAVARIA GERMANY
:57A:DEUTDEFFXXX
:58A:DEUTDEFFXXX
:72:/ACC/SERIAL PAYMENT PROCESSING
/INS/BANKBEBBXXX
INTRABANK SETTLEMENT
STANDARD PROCESSING TIME
NO COVER PAYMENT REQUIRED
-}`
  },
  'MT205 Serial': {
    name: 'MT205 → ISO 20022 pacs.009.001.08',
    description: 'Corporate Financial Institution Transfer (Serial)',
    targetFormat: 'ISO 20022 pacs.009.001.08 XML',
    sample: `{1:F01CORPBEBBAXXX0000000000}
{2:I205CORPDEFAXXXXN}
{3:{108:MT205}{121:111a222b-33c4-44d5-e666-777888999ab1}}
{4:
:20:SER2024567123891
:21:SERIAL2024890124
:32A:241223EUR4250000,00
:52A:CORPBEBBXXX
:53A:CORPBEBBXXX
:54A:CORPDEFAXXX
:56D:CORPORATE INTERMEDIARY BANK GMBH
CORPORATE FINANCIAL SERVICES DIV
MUNICH BAVARIA GERMANY
:57A:CORPDEFAXXX
:58A:CORPDEFAXXX
:72:/ACC/CORPORATE SERIAL PAYMENT PROCESSING
/INS/CORPBEBBXXX
CORPORATE INTRABANK SETTLEMENT
STANDARD CORPORATE PROCESSING TIME
NO CORPORATE COVER PAYMENT REQUIRED
-}`
  },
};

function App() {
  const [selectedTransformation, setSelectedTransformation] = useState('MT103');
  const [inputMessage, setInputMessage] = useState('');
  const [outputXml, setOutputXml] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState([]);
  const [success, setSuccess] = useState(false);
  const [processingInfo, setProcessingInfo] = useState(null);
  const [resultCount, setResultCount] = useState(0);
  const [messageType, setMessageType] = useState('single');

  // Detect if we're in development mode or GitHub Pages
  const isDevelopment = window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1';

  // Auto-load sample when transformation type changes or on initial load
  useEffect(() => {
    setInputMessage(TRANSFORMATIONS[selectedTransformation].sample);
    setError('');
    setOutputXml('');
    setSuccess(false);
    setProcessingInfo(null);
    setResultCount(0);
    setMessageType('single');
  }, [selectedTransformation]);

  const formatXml = (xml) => {
    try {
      const PADDING = ' '.repeat(2);
      const reg = /(>)(<)(\/*)/g;
      let formatted = xml.replace(reg, function(match, p1, p2, p3) {
        return p1 + '\r\n' + p2 + p3;
      });
      let pad = 0;
      
      return formatted.split('\r\n').map((node) => {
        let indent = 0;
        if (node.match(/.+<\/\w[^>]*>$/)) {
          indent = 0;
        } else if (node.match(/^<\/\w/) && pad > 0) {
          pad -= 1;
        } else if (node.match(/^<\w[^>]*[^/]>.*$/)) {
          indent = 1;
        } else {
          indent = 0;
        }
        
        const padding = PADDING.repeat(pad);
        pad += indent;
        
        return padding + node;
      }).join('\r\n');
    } catch (e) {
      return xml;
    }
  };

  // Helper function to safely escape special characters in XML content
  const escapeXmlForTemplateString = (xml) => {
    if (typeof xml !== 'string') return xml;
    // Escape dollar signs and backslashes that could be interpreted as template literal placeholders
    return xml.replace(/\$/g, '\\$').replace(/`/g, '\\`');
  };

  const handleTransform = async () => {
    if (!inputMessage.trim()) {
      setError(['Please enter a SWIFT message']);
      return;
    }

    setLoading(true);
    setError([]);
    setSuccess(false);
    setOutputXml('');
    setProcessingInfo(null);
    setResultCount(0);
    setMessageType('single');

    try {
      const response = await fetch(API_ENDPOINT, {
        method: 'POST',
        headers: {
          'Content-Type': 'text/plain',
        },
        body: inputMessage,
      });

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: Server error`);
      }

      const responseText = await response.text();
      let jsonResponse;

      try {
        jsonResponse = JSON.parse(responseText);
      } catch (jsonError) {
        throw new Error('Invalid response format from server');
      }

      // Handle the new consistent JSON response format
      if (jsonResponse.status === 'success') {
        // Success case
        setSuccess(true);
        setProcessingInfo(jsonResponse.processing_info);
        setResultCount(jsonResponse.count);
        setMessageType(jsonResponse.message_type);

        if (jsonResponse.results && jsonResponse.results.length > 0) {
          if (jsonResponse.message_type === 'multiple') {
            // Multiple results - show as numbered XML outputs
            const formattedResults = jsonResponse.results.map((xml, index) => {
              const safeXml = escapeXmlForTemplateString(formatXml(xml));
              return `<!-- Result ${index + 1}/${jsonResponse.count} -->\n${safeXml}`;
            }).join('\n\n<!-- ========================== -->\n\n');
            setOutputXml(formattedResults);
          } else {
            // Single result
            setOutputXml(formatXml(jsonResponse.results[0]));
          }
        } else {
          setOutputXml('No XML output generated');
        }
      } else {
        // Error case
        setSuccess(false);
        if (jsonResponse.errors && jsonResponse.errors.length > 0) {
          // Extract all error messages from the errors array
          const errorMessages = jsonResponse.errors.map(errorObj => {
            return errorObj.error_message || 'Unknown error';
          });
          setError(errorMessages);
        } else {
          setError(['Unknown error occurred during processing']);
        }
        setProcessingInfo(jsonResponse.processing_info);
      }

      setLoading(false);

    } catch (err) {
      console.error('API Error:', err);
      setError([`Unable to connect to the API: ${err.message}`]);
      setLoading(false);
      setProcessingInfo(null);
    }
  };

  const handleTransformationChange = (transformation) => {
    setSelectedTransformation(transformation);
    // Sample will be auto-loaded by useEffect
  };

  const handleClear = () => {
    setInputMessage('');
    setOutputXml('');
    setError([]);
    setSuccess(false);
    setProcessingInfo(null);
    setResultCount(0);
    setMessageType('single');
  };

  return (
    <Box 
      style={{ 
        minHeight: '100vh', 
        background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
        position: 'relative'
      }}
    >
      {/* Header with Title */}
      <Paper 
        p="lg"
        style={{ 
          background: 'rgba(255, 255, 255, 0.1)',
          backdropFilter: 'blur(20px)',
          borderBottom: '1px solid rgba(255, 255, 255, 0.2)',
          borderRadius: 0
        }}
      >
        <Group justify="space-between" align="center">
          <Box style={{ width: '200px' }}>
            {/* Left spacer for centering */}
          </Box>
          
          <Group 
            gap="lg"
            style={{
              background: 'rgba(255, 255, 255, 0.1)',
              padding: '12px 24px',
              borderRadius: '12px',
              textAlign: 'center'
            }}
          >
            <Box style={{ textAlign: 'center' }}>
              <Title 
                order={1} 
                style={{ 
                  fontWeight: 900,
                  color: 'white',
                  letterSpacing: '-1px',
                  marginBottom: '4px',
                  fontSize: '2.5rem',
                  textShadow: '0 2px 8px rgba(0,0,0,0.3)'
                }}
              >
                Reframe
              </Title>
              <Text size="md" style={{ 
                color: 'rgba(255, 255, 255, 0.9)',
                fontWeight: 500,
                letterSpacing: '0.5px'
              }}>
                SWIFT to ISO 20022 Transformer
              </Text>
              <Text size="sm" style={{ 
                color: 'rgba(255, 255, 255, 0.7)',
                marginTop: '2px'
              }}>
                Transform MT messages to XML format with intelligent auto-detection
              </Text>
            </Box>
          </Group>
          
          <Box style={{ width: '200px', display: 'flex', justifyContent: 'flex-end' }}>
            <Badge 
              variant="outline"
              color="white"
              style={{ 
                color: 'white',
                borderColor: 'rgba(255, 255, 255, 0.3)',
                backgroundColor: 'rgba(255, 255, 255, 0.1)',
              }}
              leftSection={<IconCheck size={16} />}
            >
              {success ? 'Transform Complete' : isDevelopment ? 'Development Mode' : 'Production Ready'}
            </Badge>
          </Box>
        </Group>
      </Paper>

      {/* Main Panels */}
      <Container size="xl" py="md">
        <Grid gutter="md">
          {/* Input Panel */}
          <Grid.Col span={{ base: 12, lg: 6 }}>
            <Card
              style={{ 
                height: '500px',
                display: 'flex', 
                flexDirection: 'column',
                background: 'rgba(255, 255, 255, 0.95)',
                backdropFilter: 'blur(20px)',
                border: '1px solid rgba(255, 255, 255, 0.2)',
                boxShadow: '0 8px 32px rgba(0, 0, 0, 0.1)',
                overflow: 'hidden'
              }}
              radius="lg"
              p={0}
            >
              <Box 
                style={{ 
                  padding: '1rem', 
                  background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)', 
                  color: 'white', 
                  flexShrink: 0,
                  height: '70px'
                }}
              >
                <Group gap="md" align="center">
                  <IconCode size={24} />
                  <Box>
                    <Title order={5} style={{ fontWeight: 700, marginBottom: '2px' }}>
                      SWIFT Message Input
                    </Title>
                    <Text size="xs" style={{ opacity: 0.9 }}>
                      Paste your SWIFT message or use a sample
                    </Text>
                  </Box>
                </Group>
              </Box>
              <Box style={{ padding: '1rem', flex: 1, display: 'flex' }}>
                <Textarea
                  value={inputMessage}
                  onChange={(e) => setInputMessage(e.target.value)}
                  placeholder="Paste your SWIFT message here..."
                  style={{
                    flex: 1,
                    fontFamily: 'SF Mono, Monaco, Consolas, "Courier New", monospace',
                    fontSize: '13px',
                    lineHeight: '1.4',
                  }}
                  styles={{
                    input: {
                      height: '100%',
                      backgroundColor: '#fafafa',
                      border: '2px solid #f0f0f0',
                      borderRadius: '8px',
                      width: '100%',
                      resize: 'none',
                      '&:hover': {
                        border: '2px solid #667eea',
                      },
                      '&:focus': {
                        border: '2px solid #667eea',
                        boxShadow: '0 0 0 4px rgba(102, 126, 234, 0.1)'
                      },
                    }
                  }}
                  autosize
                  minRows={18}
                  maxRows={18}
                />
              </Box>
            </Card>
          </Grid.Col>

          {/* Output Panel */}
          <Grid.Col span={{ base: 12, lg: 6 }}>
            <Card
              style={{ 
                height: '500px',
                display: 'flex', 
                flexDirection: 'column',
                background: 'rgba(255, 255, 255, 0.95)',
                backdropFilter: 'blur(20px)',
                border: '1px solid rgba(255, 255, 255, 0.2)',
                boxShadow: '0 8px 32px rgba(0, 0, 0, 0.1)',
                overflow: 'hidden'
              }}
              radius="lg"
              p={0}
            >
              <Box 
                style={{ 
                  padding: '1rem', 
                  background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)', 
                  color: 'white', 
                  flexShrink: 0,
                  height: '70px'
                }}
              >
                <Group gap="md" align="center" justify="space-between">
                  <Group gap="md" align="center">
                    <IconTransform size={24} />
                    <Box>
                      <Title order={5} style={{ fontWeight: 700, marginBottom: '2px' }}>
                        ISO 20022 XML Output
                        {resultCount > 0 && (
                          <Badge 
                            size="xs" 
                            variant="light" 
                            color="white"
                            style={{ 
                              marginLeft: '8px',
                              color: 'white',
                              backgroundColor: 'rgba(255, 255, 255, 0.2)'
                            }}
                          >
                            {resultCount} {messageType === 'multiple' ? 'Messages' : 'Message'}
                          </Badge>
                        )}
                      </Title>
                      <Text size="xs" style={{ opacity: 0.9 }}>
                        {processingInfo ? 
                          `${processingInfo.detected_format} → Converted XML (${processingInfo.workflows_executed} workflows)` :
                          'Converted XML with syntax highlighting'
                        }
                      </Text>
                    </Box>
                  </Group>
                  
                  {processingInfo && (
                    <Group gap="xs">
                      <Badge 
                        size="xs" 
                        variant="outline"
                        color="white"
                        style={{ 
                          color: 'white',
                          borderColor: 'rgba(255, 255, 255, 0.5)'
                        }}
                      >
                        {processingInfo.input_size} chars
                      </Badge>
                      {messageType === 'multiple' && (
                        <Badge 
                          size="xs" 
                          variant="filled"
                          color="yellow"
                          style={{ 
                            color: '#333'
                          }}
                        >
                          1-to-Many
                        </Badge>
                      )}
                    </Group>
                  )}
                </Group>
              </Box>
              <Box style={{ 
                backgroundColor: '#fafafa',
                flex: 1,
                overflow: 'hidden',
                position: 'relative'
              }}>
                {outputXml ? (
                  <Box style={{ 
                    height: '100%',
                    overflow: 'auto',
                    position: 'relative'
                  }}>
                    <Box style={{ height: '100%', display: 'flex' }}>
                      {/* Copy Button */}
                      <Button
                        variant="subtle"
                        size="xs"
                        onClick={async () => {
                          try {
                            await navigator.clipboard.writeText(outputXml);
                            // You could add a notification here if needed
                          } catch (err) {
                            console.error('Failed to copy:', err);
                          }
                        }}
                        style={{
                          position: 'absolute',
                          top: '8px',
                          right: '8px',
                          zIndex: 10,
                          backgroundColor: 'rgba(255, 255, 255, 0.9)',
                          color: '#667eea',
                          border: '1px solid rgba(102, 126, 234, 0.3)',
                          fontSize: '11px',
                          padding: '4px 8px',
                          height: 'auto'
                        }}
                      >
                        Copy XML
                      </Button>
                      
                      {/* Line Numbers - Not selectable */}
                      <Box
                        style={{
                          width: '3.5em',
                          backgroundColor: '#f5f5f5',
                          borderRight: '1px solid #e0e0e0',
                          fontSize: '12px',
                          lineHeight: '1.4',
                          fontFamily: 'SF Mono, Monaco, Consolas, "Courier New", monospace',
                          color: '#999',
                          textAlign: 'right',
                          padding: '16px 8px 16px 4px',
                          userSelect: 'none',
                          WebkitUserSelect: 'none',
                          MozUserSelect: 'none',
                          overflow: 'hidden',
                          whiteSpace: 'pre',
                          flexShrink: 0
                        }}
                      >
                        {outputXml.split('\n').map((_, index) => (
                          <div key={index} style={{ height: '16.8px' }}>
                            {index + 1}
                          </div>
                        ))}
                      </Box>
                      
                      {/* XML Content - Selectable with syntax highlighting */}
                      <Box style={{ flex: 1, overflow: 'auto' }}>
                        <SyntaxHighlighter
                          language="xml"
                          style={vscDarkPlus}
                          customStyle={{
                            margin: 0,
                            padding: '16px',
                            backgroundColor: '#fafafa',
                            fontSize: '12px',
                            lineHeight: '1.4',
                            height: '100%',
                            minHeight: '100%',
                            fontFamily: 'SF Mono, Monaco, Consolas, "Courier New", monospace',
                            whiteSpace: 'pre-wrap',
                            wordBreak: 'break-word',
                            color: '#333',
                            border: 'none'
                          }}
                          showLineNumbers={false}
                          wrapLines={true}
                          wrapLongLines={true}
                          CodeTag={({ children, ...props }) => (
                            <code {...props} style={{ userSelect: 'text', WebkitUserSelect: 'text', MozUserSelect: 'text' }}>
                              {children}
                            </code>
                          )}
                          PreTag={({ children, ...props }) => (
                            <pre {...props} style={{ userSelect: 'text', WebkitUserSelect: 'text', MozUserSelect: 'text' }}>
                              {children}
                            </pre>
                          )}
                        >
                          {outputXml}
                        </SyntaxHighlighter>
                      </Box>
                    </Box>
                  </Box>
                ) : (
                  <Box style={{ 
                    height: '100%',
                    display: 'flex', 
                    alignItems: 'center', 
                    justifyContent: 'center',
                    color: '#888',
                    textAlign: 'center',
                    padding: '2rem',
                    background: '#fafafa'
                  }}>
                    <Stack align="center" gap="md">
                      <IconTransform size={40} style={{ opacity: 0.3 }} />
                      <Title order={5} style={{ fontWeight: 500, color: '#ccc' }}>
                        XML Output Preview
                      </Title>
                      <Text size="sm" style={{ color: '#888', maxWidth: 280 }}>
                        Your converted ISO 20022 XML will appear here with syntax highlighting
                      </Text>
                    </Stack>
                  </Box>
                )}
              </Box>
            </Card>
          </Grid.Col>
        </Grid>
      </Container>

      {/* Action Buttons and Sample Messages */}
      <Container size="xl" px="md" pb="md">
        <Card
          style={{
            background: 'rgba(255, 255, 255, 0.95)',
            backdropFilter: 'blur(20px)',
            border: '1px solid rgba(255, 255, 255, 0.2)',
            boxShadow: '0 4px 20px rgba(0, 0, 0, 0.1)',
          }}
          radius="md"
          p="lg"
        >
          <Stack gap="xl">
            {/* Action Buttons Section */}
            <Stack gap="md">
              <Group gap="md" align="center" style={{ flexWrap: 'wrap' }}>
                <Button
                  variant="filled"
                  leftSection={loading ? <Loader size={18} color="white" /> : <IconPlayerPlay size={18} />}
                  onClick={handleTransform}
                  disabled={loading}
                  size="lg"
                  radius="md"
                  style={{
                    background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
                    boxShadow: '0 4px 20px rgba(102, 126, 234, 0.4)',
                    minWidth: 'fit-content',
                    flexShrink: 0,
                  }}
                >
                  {loading ? 'Processing...' : 'Transform Message'}
                </Button>
                <Button
                  variant="outline"
                  leftSection={<IconRefresh size={18} />}
                  onClick={handleClear}
                  disabled={loading}
                  size="lg"
                  radius="md"
                  style={{
                    border: '2px solid #667eea',
                    color: '#667eea',
                    minWidth: 'fit-content',
                    flexShrink: 0,
                  }}
                >
                  Clear All
                </Button>
              </Group>
              
              {/* Status Messages */}
              {error && error.length > 0 && (
                <Transition mounted transition="fade">
                  {(styles) => (
                    <Alert 
                      style={styles}
                      variant="light"
                      color="red"
                      title={`Processing Error${error.length > 1 ? 's' : ''}`}
                      icon={<IconAlertCircle size={18} />}
                      radius="md"
                    >
                      <Stack gap="xs">
                        {error.map((errorMessage, index) => (
                          <Text key={index} size="sm" style={{ 
                            padding: error.length > 1 ? '4px 8px' : '0',
                            backgroundColor: error.length > 1 ? 'rgba(255, 0, 0, 0.05)' : 'transparent',
                            borderRadius: error.length > 1 ? '4px' : '0',
                            borderLeft: error.length > 1 ? '3px solid rgba(255, 0, 0, 0.3)' : 'none'
                          }}>
                            {error.length > 1 && <strong>Error {index + 1}:</strong>} {errorMessage}
                          </Text>
                        ))}
                        {processingInfo && (
                          <Text size="xs" c="dimmed">
                            Detected: {processingInfo.detected_format} • 
                            Input size: {processingInfo.input_size} chars • 
                            Workflows: {processingInfo.workflows_executed}
                          </Text>
                        )}
                      </Stack>
                    </Alert>
                  )}
                </Transition>
              )}
              {success && !error && (
                <Transition mounted transition="fade">
                  {(styles) => (
                    <Alert 
                      style={styles}
                      variant="light"
                      color="green"
                      title="Transformation Complete"
                      icon={<IconCheck size={18} />}
                      radius="md"
                    >
                      <Stack gap="xs">
                        <Text size="sm">
                          {messageType === 'multiple' 
                            ? `Generated ${resultCount} XML messages successfully!`
                            : 'Message transformed successfully!'
                          }
                        </Text>
                        {processingInfo && (
                          <Text size="xs" c="dimmed">
                            {processingInfo.detected_format} → ISO 20022 • 
                            {processingInfo.workflows_executed} workflow{processingInfo.workflows_executed !== 1 ? 's' : ''} executed • 
                            Output: {resultCount} message{resultCount !== 1 ? 's' : ''}
                          </Text>
                        )}
                      </Stack>
                    </Alert>
                  )}
                </Transition>
              )}
              {loading && (
                <Progress 
                  value={100}
                  animated
                  style={{ 
                    height: 6,
                    backgroundColor: 'rgba(102, 126, 234, 0.2)',
                    width: '100%',
                    maxWidth: '300px'
                  }}
                  radius="md"
                />
              )}
            </Stack>

            {/* Sample Message Buttons Section */}
            <Stack gap="sm">
              <Title order={5} mb="xs">Sample Messages</Title>
              <Group gap="sm" style={{ flexWrap: 'wrap' }}>
                {Object.keys(TRANSFORMATIONS).map((transformation) => (
                  <Button
                    key={transformation}
                    variant={selectedTransformation === transformation ? 'filled' : 'outline'}
                    onClick={() => handleTransformationChange(transformation)}
                    disabled={loading}
                    style={{
                      background: selectedTransformation === transformation 
                        ? 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)'
                        : 'transparent',
                      border: selectedTransformation === transformation 
                        ? 'none'
                        : '2px solid #e0e0e0',
                      minWidth: 'fit-content',
                      flexShrink: 0,
                    }}
                    radius="md"
                    size="sm"
                  >
                    {transformation}
                  </Button>
                ))}
              </Group>
            </Stack>
          </Stack>
        </Card>
      </Container>

      {/* Footer */}
      <Box style={{ textAlign: 'center', padding: '1rem' }}>
        <Text size="sm" style={{ 
          color: 'rgba(255, 255, 255, 0.8)',
          fontWeight: 500 
        }}>
          Powered by Reframe API • Built with React & Mantine
        </Text>
      </Box>
    </Box>
  );
}

export default App; 