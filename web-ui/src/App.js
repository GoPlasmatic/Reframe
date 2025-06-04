import React, { useState, useEffect } from 'react';
import {
  AppBar,
  Toolbar,
  Typography,
  Container,
  Grid,
  Paper,
  TextField,
  Button,
  Box,
  Alert,
  CircularProgress,
  Chip,
  Card,
  CardContent,
  Divider,
} from '@mui/material';
import {
  Transform as TransformIcon,
  Code as CodeIcon,
  CheckCircle as CheckIcon,
  Error as ErrorIcon,
} from '@mui/icons-material';
import { Light as SyntaxHighlighter } from 'react-syntax-highlighter';
import { docco } from 'react-syntax-highlighter/dist/esm/styles/hljs';
import xml from 'react-syntax-highlighter/dist/esm/languages/hljs/xml';

// Register XML language for syntax highlighting
SyntaxHighlighter.registerLanguage('xml', xml);

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

  // Detect if we're in development mode or GitHub Pages
  const isDevelopment = window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1';

  // Auto-load sample when transformation type changes or on initial load
  useEffect(() => {
    setInputMessage(TRANSFORMATIONS[selectedTransformation].sample);
    setError('');
    setOutputXml('');
    setSuccess(false);
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

    try {
      const response = await fetch(API_ENDPOINT, {
        method: 'POST',
        headers: {
          'Content-Type': 'text/plain',
        },
        body: inputMessage,
      });

      const responseText = await response.text();

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${responseText}`);
      }

      // Check if response is XML or JSON
      if (responseText.trim().startsWith('<')) {
        setOutputXml(formatXml(responseText));
      } else {
        try {
          const jsonResponse = JSON.parse(responseText);
          if (jsonResponse.result && jsonResponse.result.startsWith('<')) {
            setOutputXml(formatXml(jsonResponse.result));
          } else {
            setOutputXml(JSON.stringify(jsonResponse, null, 2));
          }
        } catch (jsonError) {
          setOutputXml(responseText);
        }
      }

      setSuccess(true);
      setLoading(false);

    } catch (err) {
      console.error('API Error:', err);
      setError(`Unable to connect to the API: ${err.message}`);
      setLoading(false);
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
  };

  return (
    <Box sx={{ flexGrow: 1, minHeight: '100vh', backgroundColor: '#f5f5f5' }}>
      {/* Header */}
      <AppBar position="static" elevation={2}>
        <Toolbar>
          <CodeIcon sx={{ mr: 2 }} />
          <Typography variant="h6" component="div" sx={{ flexGrow: 1 }}>
            Reframe - SWIFT Message to ISO 20022 Converter
          </Typography>
          <Chip 
            label={success ? 'Connected via HTTPS' : isDevelopment ? 'Ready (Development)' : 'Ready (GitHub Pages)'}
            color={success ? 'success' : 'default'}
            size="small"
            icon={<CheckIcon />}
          />
        </Toolbar>
      </AppBar>

      {/* Main Content */}
      <Container maxWidth="xl" sx={{ mt: 3, mb: 3 }}>
        {/* Description Card */}
        <Card sx={{ mb: 3 }}>
          <CardContent>
            <Typography variant="h5" gutterBottom>
              SWIFT Message to ISO 20022 Converter
            </Typography>
            <Typography variant="body1" color="text.secondary" gutterBottom>
              Convert SWIFT MT messages to ISO 20022 XML format
            </Typography>
            <Typography variant="body2" color="text.secondary">
              Paste your SWIFT message below or load a sample message to get started.
              The system automatically detects the message type and converts it to the appropriate ISO 20022 format.
              The converted XML will appear in the right panel with syntax highlighting.
            </Typography>
          </CardContent>
        </Card>

        {/* Sample Message Buttons */}
        <Card sx={{ mb: 3 }}>
          <CardContent>
            <Typography variant="h6" gutterBottom>
              Load Sample Messages
            </Typography>
            <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
              Click any button below to load a sample message for testing:
            </Typography>
            <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 1.5 }}>
              {Object.keys(TRANSFORMATIONS).map((transformation) => (
                <Button
                  key={transformation}
                  variant={selectedTransformation === transformation ? 'contained' : 'outlined'}
                  onClick={() => handleTransformationChange(transformation)}
                  disabled={loading}
                  sx={{
                    minWidth: '120px',
                    textTransform: 'none',
                    fontWeight: selectedTransformation === transformation ? 'bold' : 'normal',
                    boxShadow: selectedTransformation === transformation ? 3 : 1,
                  }}
                >
                  {transformation} Sample
                </Button>
              ))}
            </Box>
          </CardContent>
        </Card>

        {/* Action Buttons */}
        <Box sx={{ mb: 3, display: 'flex', gap: 2, flexWrap: 'wrap', alignItems: 'center' }}>
          <Button
            variant="contained"
            startIcon={<TransformIcon />}
            onClick={handleTransform}
            disabled={loading}
            size="large"
          >
            {loading ? <CircularProgress size={20} color="inherit" /> : 'Transform'}
          </Button>
          <Button
            variant="outlined"
            onClick={handleClear}
            disabled={loading}
          >
            Clear All
          </Button>
          
          {/* Inline Status Messages */}
          {error && (
            <Alert severity="error" sx={{ flexGrow: 1, maxWidth: '400px' }} icon={<ErrorIcon />}>
              {error}
            </Alert>
          )}
          {success && !error && (
            <Alert severity="success" sx={{ flexGrow: 1, maxWidth: '400px' }} icon={<CheckIcon />}>
              Message transformed successfully!
            </Alert>
          )}
        </Box>

        {/* Main Grid */}
        <Grid container spacing={3}>
          {/* Input Panel */}
          <Grid item xs={12} md={6}>
            <Paper elevation={3} sx={{ height: '600px', display: 'flex', flexDirection: 'column' }}>
              <Box sx={{ p: 2, backgroundColor: 'primary.main', color: 'white' }}>
                <Typography variant="h6">
                  SWIFT Message Input
                </Typography>
                <Typography variant="body2" sx={{ opacity: 0.8 }}>
                  Paste your SWIFT message or use a sample
                </Typography>
              </Box>
              <Divider />
              <Box sx={{ p: 2, flexGrow: 1, display: 'flex' }}>
                <TextField
                  multiline
                  fullWidth
                  value={inputMessage}
                  onChange={(e) => setInputMessage(e.target.value)}
                  placeholder="Paste your SWIFT message here..."
                  variant="outlined"
                  sx={{
                    '& .MuiOutlinedInput-root': {
                      height: '100%',
                      '& fieldset': {
                        border: 'none',
                      },
                    },
                    '& .MuiInputBase-input': {
                      fontFamily: 'monospace',
                      fontSize: '14px',
                      height: '100% !important',
                    },
                  }}
                />
              </Box>
            </Paper>
          </Grid>

          {/* Output Panel */}
          <Grid item xs={12} md={6}>
            <Paper elevation={3} sx={{ height: '600px', display: 'flex', flexDirection: 'column' }}>
              <Box sx={{ p: 2, backgroundColor: 'secondary.main', color: 'white' }}>
                <Typography variant="h6">
                  ISO 20022 XML Output
                </Typography>
                <Typography variant="body2" sx={{ opacity: 0.8 }}>
                  Converted XML output with syntax highlighting
                </Typography>
              </Box>
              <Divider />
              <Box sx={{ flexGrow: 1, overflow: 'auto', backgroundColor: '#f8f8f8' }}>
                {outputXml ? (
                  <SyntaxHighlighter
                    language="xml"
                    style={docco}
                    customStyle={{
                      margin: 0,
                      padding: '16px',
                      backgroundColor: 'transparent',
                      fontSize: '14px',
                      lineHeight: '1.4',
                    }}
                    showLineNumbers={true}
                  >
                    {outputXml}
                  </SyntaxHighlighter>
                ) : (
                  <Box sx={{ p: 3, color: 'text.secondary', fontStyle: 'italic' }}>
                    Converted XML will appear here after transformation...
                  </Box>
                )}
              </Box>
            </Paper>
          </Grid>
        </Grid>

        {/* Footer */}
        <Box sx={{ mt: 4, textAlign: 'center' }}>
          <Typography variant="body2" color="text.secondary">
            Powered by Reframe API • Built with React & Material-UI
          </Typography>
        </Box>
      </Container>
    </Box>
  );
}

export default App; 