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
    sample: `{1:F01BNPAFRPPXXX0000000000}{2:O1031234240101DEUTDEFFXXXX12345678952401011234N}{3:{103:EBA}}{4:
:20:FT21001234567890
:23B:CRED
:32A:240101USD1000,00
:50K:/1234567890
ACME CORPORATION
123 MAIN STREET
NEW YORK NY 10001
:52A:BNPAFRPPXXX
:57A:DEUTDEFFXXX
:59:/DE89370400440532013000
MUELLER GMBH
HAUPTSTRASSE 1
10115 BERLIN
:70:PAYMENT FOR INVOICE 12345
:71A:OUR
-}`
  },
  'MT102': {
    name: 'MT102 → ISO 20022 pacs.008.001.08 (Multiple)',
    description: 'Multiple Customer Credit Transfer (1-to-Many)',
    targetFormat: 'Multiple ISO 20022 pacs.008.001.08 XML',
    sample: `{1:F01BNPAFRPPXXX0000000000}{2:O1021234240101DEUTDEFFXXXX12345678952401011234N}{3:{103:EBA}}{4:
:20:MT102SAMPLE001
:23B:CRED
:32A:240101USD1500,00
:19:2
:32B:USD750,00
:50K:/1234567890
ORDERING CUSTOMER NAME
123 MAIN STREET
:52A:BNPAFRPPXXX
:58A:DEUTDEFFXXX
:59:987654321/BENEFICIARY NAME 1
BENEFICIARY ADDRESS 1
:70:PAYMENT DETAILS 1
:32B:USD750,00
:59:123456789/BENEFICIARY NAME 2
BENEFICIARY ADDRESS 2
:70:PAYMENT DETAILS 2
-}`
  },
  'MT103+': {
    name: 'MT103+ → ISO 20022 pacs.008.001.08 (Enhanced)',
    description: 'Enhanced Customer Credit Transfer with STP',
    targetFormat: 'ISO 20022 pacs.008.001.08 XML (Enhanced)',
    sample: `{1:F01BNPAFRPPXXX0000000000}{2:O1031234240101DEUTDEFFXXXX12345678952401011234N}{3:{103:EBA}}{4:
:20:FT21001234567890
:23B:CRED
:32A:240101USD1000,00
:50K:/1234567890
ACME CORPORATION
123 MAIN STREET
NEW YORK NY 10001
:52A:BNPAFRPPXXX
:57A:DEUTDEFFXXX
:59:/DE89370400440532013000
MUELLER GMBH
HAUPTSTRASSE 1
10115 BERLIN
:70:PAYMENT FOR INVOICE 12345
:71A:OUR
:121:3ec6a2b8-7b2f-4c5e-8f4d-9a1b2c3e4f5g
:77B:/ORDERRES/BE//MEILAAN 1, 1000 BRUSSELS
:77T:/BENEFRES/DE//HAUPTSTRASSE 1, 10115 BERLIN
-}`
  },
  'MT192': {
    name: 'MT192 → ISO 20022 camt.056.001.08',
    description: 'Request for Cancellation',
    targetFormat: 'ISO 20022 camt.056.001.08 XML',
    sample: `{1:F01BNPAFRPPXXX0000000000}{2:O1921234240101DEUTDEFFXXXX12345678952401011234N}{3:{108:MT192}}{4:
:20:REQ240101001
:21:FT21001234567890
:11S:103
:32A:240101USD1000,00
:52A:BNPAFRPPXXX
:57A:DEUTDEFFXXX
:72:/RETN/AC01/Invalid account number
/CASE/CASE240101001
-}`
  },
  'MT196': {
    name: 'MT196 → ISO 20022 camt.029.001.09',
    description: 'Client Side Liquidity Management Answer',
    targetFormat: 'ISO 20022 camt.029.001.09 XML',
    sample: `{1:F01BNPAFRPPXXX0000000000}{2:O1961234240101DEUTDEFFXXXX12345678952401011234N}{3:{108:MT196}}{4:
:20:ANSW240101001
:21:REQ240101001
:32A:240101USD1000,00
:52A:BNPAFRPPXXX
:57A:DEUTDEFFXXX
:72:/ACC/Account management request
/ANSW/ACCP
-}`
  },
  'MT202': {
    name: 'MT202 → ISO 20022 pacs.009.001.08',
    description: 'General Financial Institution Transfer',
    targetFormat: 'ISO 20022 pacs.009.001.08 XML',
    sample: `{1:F01BNPAFRPPXXX0000000000}{2:O2021234240101DEUTDEFFXXXX12345678952401011234N}{3:{103:EBA}}{4:
:20:FTI21001234567890
:21:ORI240101001
:32A:240101USD50000,00
:52A:BNPAFRPPXXX
:53A:CHASUS33XXX
:58A:DEUTDEFFXXX
:72:/BNF/Final beneficiary information
/INS/Payment instruction details
-}`
  },
  'MT202COV': {
    name: 'MT202 COV → ISO 20022 pacs.009.001.08 (Cover)',
    description: 'Cover Payment for Underlying Customer Credit Transfer',
    targetFormat: 'ISO 20022 pacs.009.001.08 XML (Cover)',
    sample: `{1:F01BNPAFRPPXXX0000000000}{2:O2021234240101DEUTDEFFXXXX12345678952401011234N}{3:{103:EBA}}{4:
:20:COV21001234567890
:21:FT21001234567890
:32A:240101USD50000,00
:50A:ORDERING CUSTOMER
:52A:BNPAFRPPXXX
:53A:CHASUS33XXX
:58A:DEUTDEFFXXX
:59:BENEFICIARY CUSTOMER
:70:/COVPAY/Cover payment details
/ORIG/Original payment reference
-}`
  },
  'MT210': {
    name: 'MT210 → ISO 20022 camt.057.001.06',
    description: 'Notice to Receive',
    targetFormat: 'ISO 20022 camt.057.001.06 XML',
    sample: `{1:F01BNPAFRPPXXX0000000000}{2:O2101234240101DEUTDEFFXXXX12345678952401011234N}{3:{108:MT210}}{4:
:20:NTR240101001
:25:12345678/001
:32A:240101USD2500,00
:50A:ACME CORPORATION
:52A:BNPAFRPPXXX
:57A:DEUTDEFFXXX
:58A:CITIUS33XXX
:72:/REC/Expected incoming payment
/REF/Reference information
-}`
  }
};

function App() {
  const [selectedTransformation, setSelectedTransformation] = useState('MT103');
  const [inputMessage, setInputMessage] = useState('');
  const [outputXml, setOutputXml] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
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
      let formatted = xml.replace(reg, '$1\r\n$2$3');
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

  const handleTransform = async () => {
    if (!inputMessage.trim()) {
      setError('Please enter a SWIFT message');
      return;
    }

    setLoading(true);
    setError('');
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
            const formattedResults = jsonResponse.results.map((xml, index) => 
              `<!-- Result ${index + 1}/${jsonResponse.count} -->\n${formatXml(xml)}`
            ).join('\n\n<!-- ========================== -->\n\n');
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
        if (jsonResponse.error) {
          setError(`${jsonResponse.error.message}`);
          if (jsonResponse.error.details) {
            console.log('Detailed error info:', jsonResponse.error.details);
          }
        } else {
          setError('Unknown error occurred during processing');
        }
        setProcessingInfo(jsonResponse.processing_info);
      }

      setLoading(false);

    } catch (err) {
      console.error('API Error:', err);
      setError(`Unable to connect to the API: ${err.message}`);
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
    setError('');
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
                backgroundColor: '#1a1a1a',
                flex: 1,
                overflow: 'hidden',
                position: 'relative'
              }}>
                {outputXml ? (
                  <Box style={{ 
                    height: '100%',
                    overflow: 'auto',
                  }}>
                    <SyntaxHighlighter
                      language="xml"
                      style={vscDarkPlus}
                      customStyle={{
                        margin: 0,
                        padding: '16px',
                        backgroundColor: 'transparent',
                        fontSize: '12px',
                        lineHeight: '1.4',
                        height: '100%',
                        minHeight: '100%',
                        fontFamily: 'SF Mono, Monaco, Consolas, "Courier New", monospace',
                        overflow: 'auto',
                        whiteSpace: 'pre-wrap',
                        wordBreak: 'break-word'
                      }}
                      showLineNumbers={true}
                      wrapLines={true}
                      wrapLongLines={true}
                      lineNumberStyle={{
                        minWidth: '2.5em',
                        paddingRight: '0.8em',
                        color: '#666',
                        textAlign: 'right'
                      }}
                    >
                      {outputXml}
                    </SyntaxHighlighter>
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
                    background: 'linear-gradient(45deg, #1a1a1a 0%, #2d2d2d 100%)'
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
              {error && (
                <Transition mounted transition="fade">
                  {(styles) => (
                    <Alert 
                      style={styles}
                      variant="light"
                      color="red"
                      title="Processing Error"
                      icon={<IconAlertCircle size={18} />}
                      radius="md"
                    >
                      <Stack gap="xs">
                        <Text size="sm">{error}</Text>
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