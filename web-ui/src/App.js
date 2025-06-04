import React, { useState, useEffect } from 'react';
import {
  AppBar,
  Toolbar,
  Typography,
  Container,
  Grid,
  TextField,
  Button,
  Box,
  Alert,
  CircularProgress,
  Chip,
  Card,
  CardContent,
  Stack,
  Fade,
  LinearProgress,
} from '@mui/material';
import {
  Transform as TransformIcon,
  Code as CodeIcon,
  CheckCircle as CheckIcon,
  Error as ErrorIcon,
  PlayArrow as PlayIcon,
  Refresh as RefreshIcon,
} from '@mui/icons-material';
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
    <Box sx={{ 
      flexGrow: 1, 
      minHeight: '100vh', 
      background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
      position: 'relative'
    }}>
      {/* Modern Header */}
      <AppBar 
        position="static" 
        elevation={0}
        sx={{ 
          background: 'rgba(255, 255, 255, 0.1)',
          backdropFilter: 'blur(20px)',
          borderBottom: '1px solid rgba(255, 255, 255, 0.2)'
        }}
      >
        <Toolbar sx={{ py: 1 }}>
          <Box sx={{ 
            display: 'flex', 
            alignItems: 'center', 
            gap: 2,
            background: 'rgba(255, 255, 255, 0.1)',
            px: 2,
            py: 1,
            borderRadius: 2
          }}>
            <CodeIcon sx={{ fontSize: 28, color: 'white' }} />
            <Typography variant="h5" component="div" sx={{ 
              fontWeight: 700,
              color: 'white',
              letterSpacing: '-0.5px'
            }}>
              Reframe
            </Typography>
          </Box>
          <Box sx={{ flexGrow: 1 }} />
          <Chip 
            label={success ? 'Transform Complete' : isDevelopment ? 'Development Mode' : 'Production Ready'}
            color={success ? 'success' : 'default'}
            variant="outlined"
            sx={{ 
              color: 'white',
              borderColor: 'rgba(255, 255, 255, 0.3)',
              backgroundColor: 'rgba(255, 255, 255, 0.1)',
              fontWeight: 500
            }}
            icon={<CheckIcon sx={{ color: 'white !important' }} />}
          />
        </Toolbar>
      </AppBar>

      {/* Content Sections - Contained */}
      <Box sx={{ py: 4 }}>
        {/* Hero Section */}
        <Container maxWidth="xl" sx={{ mb: 4, px: 2 }}>
          <Fade in timeout={800}>
            <Card sx={{ 
              background: 'rgba(255, 255, 255, 0.95)',
              backdropFilter: 'blur(20px)',
              borderRadius: 4,
              border: '1px solid rgba(255, 255, 255, 0.2)',
              boxShadow: '0 8px 32px rgba(0, 0, 0, 0.1)'
            }}>
              <CardContent sx={{ p: 4 }}>
                <Stack spacing={2} alignItems="center" textAlign="center">
                  <Typography variant="h3" sx={{ 
                    fontWeight: 800,
                    background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
                    backgroundClip: 'text',
                    WebkitBackgroundClip: 'text',
                    WebkitTextFillColor: 'transparent',
                    mb: 1
                  }}>
                    SWIFT to ISO 20022 Converter
                  </Typography>
                  <Typography variant="h6" color="text.secondary" sx={{ maxWidth: 800, fontWeight: 400 }}>
                    Transform SWIFT MT messages to ISO 20022 XML format with intelligent auto-detection
                  </Typography>
                  <Typography variant="body1" color="text.secondary" sx={{ maxWidth: 900 }}>
                    Paste your SWIFT message below or load a sample. Our engine automatically detects the message type 
                    and converts it to the appropriate ISO 20022 format with real-time validation.
                  </Typography>
                </Stack>
              </CardContent>
            </Card>
          </Fade>
        </Container>

        {/* Sample Message Buttons */}
        <Container maxWidth="xl" sx={{ mb: 3, px: 2 }}>
          <Card sx={{ 
            background: 'rgba(255, 255, 255, 0.95)',
            backdropFilter: 'blur(20px)',
            borderRadius: 3,
            border: '1px solid rgba(255, 255, 255, 0.2)',
            boxShadow: '0 4px 20px rgba(0, 0, 0, 0.1)'
          }}>
            <CardContent sx={{ p: 3 }}>
              <Typography variant="h6" sx={{ fontWeight: 600, mb: 1 }}>
                Sample Messages
              </Typography>
              <Typography variant="body2" color="text.secondary" sx={{ mb: 3 }}>
                Load pre-configured samples to test different message types
              </Typography>
              <Stack direction="row" spacing={2} flexWrap="wrap" useFlexGap>
                {Object.keys(TRANSFORMATIONS).map((transformation) => (
                  <Button
                    key={transformation}
                    variant={selectedTransformation === transformation ? 'contained' : 'outlined'}
                    onClick={() => handleTransformationChange(transformation)}
                    disabled={loading}
                    sx={{
                      minWidth: '140px',
                      py: 1.5,
                      px: 3,
                      borderRadius: 3,
                      textTransform: 'none',
                      fontWeight: 600,
                      fontSize: '0.95rem',
                      background: selectedTransformation === transformation 
                        ? 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)'
                        : 'transparent',
                      border: selectedTransformation === transformation 
                        ? 'none'
                        : '2px solid #e0e0e0',
                      '&:hover': {
                        background: selectedTransformation === transformation 
                          ? 'linear-gradient(135deg, #5a6fd8 0%, #6a4190 100%)'
                          : 'rgba(102, 126, 234, 0.1)',
                        border: '2px solid #667eea',
                        transform: 'translateY(-2px)',
                        boxShadow: '0 4px 12px rgba(102, 126, 234, 0.3)'
                      },
                      transition: 'all 0.3s ease'
                    }}
                  >
                    {transformation}
                  </Button>
                ))}
              </Stack>
            </CardContent>
          </Card>
        </Container>

        {/* Action Buttons */}
        <Container maxWidth="xl" sx={{ mb: 4, px: 2 }}>
          <Stack direction="row" spacing={2} alignItems="center" flexWrap="wrap" useFlexGap>
            <Button
              variant="contained"
              startIcon={loading ? <CircularProgress size={20} color="inherit" /> : <PlayIcon />}
              onClick={handleTransform}
              disabled={loading}
              size="large"
              sx={{
                py: 1.5,
                px: 4,
                borderRadius: 3,
                background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
                fontWeight: 600,
                fontSize: '1.1rem',
                textTransform: 'none',
                boxShadow: '0 4px 20px rgba(102, 126, 234, 0.4)',
                '&:hover': {
                  background: 'linear-gradient(135deg, #5a6fd8 0%, #6a4190 100%)',
                  transform: 'translateY(-2px)',
                  boxShadow: '0 6px 25px rgba(102, 126, 234, 0.5)'
                },
                '&:disabled': {
                  background: 'rgba(0, 0, 0, 0.12)'
                },
                transition: 'all 0.3s ease'
              }}
            >
              {loading ? 'Processing...' : 'Transform Message'}
            </Button>
            <Button
              variant="outlined"
              startIcon={<RefreshIcon />}
              onClick={handleClear}
              disabled={loading}
              sx={{
                py: 1.5,
                px: 3,
                borderRadius: 3,
                border: '2px solid rgba(255, 255, 255, 0.8)',
                color: 'white',
                fontWeight: 600,
                textTransform: 'none',
                '&:hover': {
                  border: '2px solid white',
                  backgroundColor: 'rgba(255, 255, 255, 0.1)',
                  transform: 'translateY(-1px)'
                },
                transition: 'all 0.3s ease'
              }}
            >
              Clear All
            </Button>
            
            {/* Status Messages */}
            {error && (
              <Fade in>
                <Alert 
                  severity="error" 
                  icon={<ErrorIcon />}
                  sx={{ 
                    borderRadius: 3,
                    background: 'rgba(211, 47, 47, 0.1)',
                    border: '1px solid rgba(211, 47, 47, 0.3)',
                    '& .MuiAlert-icon': { color: '#d32f2f' }
                  }}
                >
                  {error}
                </Alert>
              </Fade>
            )}
            {success && !error && (
              <Fade in>
                <Alert 
                  severity="success" 
                  icon={<CheckIcon />}
                  sx={{ 
                    borderRadius: 3,
                    background: 'rgba(46, 125, 50, 0.1)',
                    border: '1px solid rgba(46, 125, 50, 0.3)',
                    '& .MuiAlert-icon': { color: '#2e7d32' }
                  }}
                >
                  Message transformed successfully!
                </Alert>
              </Fade>
            )}
          </Stack>
          {loading && (
            <Box sx={{ mt: 2 }}>
              <LinearProgress 
                sx={{ 
                  borderRadius: 2,
                  height: 6,
                  backgroundColor: 'rgba(255, 255, 255, 0.3)',
                  '& .MuiLinearProgress-bar': {
                    background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)'
                  }
                }} 
              />
            </Box>
          )}
        </Container>
      </Box>

      {/* Main Panels - Full Width Container */}
      <Container maxWidth="xl" sx={{ px: 2, pb: 4 }}>
        <Grid container spacing={2} sx={{ 
          height: '600px',
          width: '100%'
        }}>
          {/* Input Panel */}
          <Grid item xs={12} lg={6} sx={{ 
            height: '600px',
            display: 'flex'
          }}>
            <Card sx={{ 
              width: '100%',
              height: '600px',
              display: 'flex', 
              flexDirection: 'column',
              background: 'rgba(255, 255, 255, 0.95)',
              backdropFilter: 'blur(20px)',
              borderRadius: 4,
              border: '1px solid rgba(255, 255, 255, 0.2)',
              boxShadow: '0 8px 32px rgba(0, 0, 0, 0.1)',
              overflow: 'hidden'
            }}>
              <Box sx={{ 
                p: 3, 
                background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)', 
                color: 'white', 
                flexShrink: 0,
                height: '100px'
              }}>
                <Stack direction="row" alignItems="center" spacing={2}>
                  <CodeIcon sx={{ fontSize: 28 }} />
                  <Box>
                    <Typography variant="h6" sx={{ fontWeight: 700, mb: 0.5 }}>
                      SWIFT Message Input
                    </Typography>
                    <Typography variant="body2" sx={{ opacity: 0.9 }}>
                      Paste your SWIFT message or use a sample
                    </Typography>
                  </Box>
                </Stack>
              </Box>
              <Box sx={{ 
                p: 3, 
                height: '500px',
                overflow: 'hidden',
                display: 'flex'
              }}>
                <TextField
                  multiline
                  fullWidth
                  value={inputMessage}
                  onChange={(e) => setInputMessage(e.target.value)}
                  placeholder="Paste your SWIFT message here..."
                  variant="outlined"
                  sx={{
                    height: '100%',
                    '& .MuiOutlinedInput-root': {
                      height: '100%',
                      borderRadius: 2,
                      backgroundColor: '#fafafa',
                      '& fieldset': {
                        border: '2px solid #f0f0f0',
                      },
                      '&:hover fieldset': {
                        border: '2px solid #667eea',
                      },
                      '&.Mui-focused fieldset': {
                        border: '2px solid #667eea',
                        boxShadow: '0 0 0 4px rgba(102, 126, 234, 0.1)'
                      },
                    },
                    '& .MuiInputBase-input': {
                      fontFamily: 'SF Mono, Monaco, Consolas, "Courier New", monospace',
                      fontSize: '14px',
                      lineHeight: '1.5',
                      height: '100% !important',
                      overflow: 'auto !important',
                      resize: 'none',
                      padding: '16px !important',
                      color: '#2d3748',
                      boxSizing: 'border-box'
                    },
                  }}
                />
              </Box>
            </Card>
          </Grid>

          {/* Output Panel */}
          <Grid item xs={12} lg={6} sx={{ 
            height: '600px',
            display: 'flex'
          }}>
            <Card sx={{ 
              width: '100%',
              height: '600px',
              display: 'flex', 
              flexDirection: 'column',
              background: 'rgba(255, 255, 255, 0.95)',
              backdropFilter: 'blur(20px)',
              borderRadius: 4,
              border: '1px solid rgba(255, 255, 255, 0.2)',
              boxShadow: '0 8px 32px rgba(0, 0, 0, 0.1)',
              overflow: 'hidden'
            }}>
              <Box sx={{ 
                p: 3, 
                background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)', 
                color: 'white', 
                flexShrink: 0,
                height: '100px'
              }}>
                <Stack direction="row" alignItems="center" spacing={2}>
                  <TransformIcon sx={{ fontSize: 28 }} />
                  <Box>
                    <Typography variant="h6" sx={{ fontWeight: 700, mb: 0.5 }}>
                      ISO 20022 XML Output
                    </Typography>
                    <Typography variant="body2" sx={{ opacity: 0.9 }}>
                      Converted XML with syntax highlighting
                    </Typography>
                  </Box>
                </Stack>
              </Box>
              <Box sx={{ 
                backgroundColor: '#1a1a1a',
                height: '500px',
                overflow: 'hidden',
                position: 'relative'
              }}>
                {outputXml ? (
                  <Box sx={{ 
                    height: '100%',
                    overflow: 'auto',
                    '&::-webkit-scrollbar': {
                      width: '8px',
                      height: '8px'
                    },
                    '&::-webkit-scrollbar-track': {
                      background: '#2d2d2d'
                    },
                    '&::-webkit-scrollbar-thumb': {
                      background: '#666',
                      borderRadius: '4px',
                      '&:hover': {
                        background: '#888'
                      }
                    }
                  }}>
                    <SyntaxHighlighter
                      language="xml"
                      style={vscDarkPlus}
                      customStyle={{
                        margin: 0,
                        padding: '20px',
                        backgroundColor: 'transparent',
                        fontSize: '14px',
                        lineHeight: '1.6',
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
                        minWidth: '3em',
                        paddingRight: '1em',
                        color: '#666',
                        textAlign: 'right'
                      }}
                    >
                      {outputXml}
                    </SyntaxHighlighter>
                  </Box>
                ) : (
                  <Box sx={{ 
                    height: '100%',
                    display: 'flex', 
                    alignItems: 'center', 
                    justifyContent: 'center',
                    color: '#888',
                    textAlign: 'center',
                    p: 4,
                    background: 'linear-gradient(45deg, #1a1a1a 0%, #2d2d2d 100%)'
                  }}>
                    <Stack alignItems="center" spacing={2}>
                      <TransformIcon sx={{ fontSize: 48, opacity: 0.3 }} />
                      <Typography variant="h6" sx={{ fontWeight: 500, color: '#ccc' }}>
                        XML Output Preview
                      </Typography>
                      <Typography variant="body2" sx={{ color: '#888', maxWidth: 300 }}>
                        Your converted ISO 20022 XML will appear here with beautiful syntax highlighting
                      </Typography>
                    </Stack>
                  </Box>
                )}
              </Box>
            </Card>
          </Grid>
        </Grid>
      </Container>

      {/* Footer */}
      <Box sx={{ textAlign: 'center', py: 3 }}>
        <Typography variant="body2" sx={{ 
          color: 'rgba(255, 255, 255, 0.8)',
          fontWeight: 500 
        }}>
          Powered by Reframe API • Built with React & Material-UI v7
        </Typography>
      </Box>
    </Box>
  );
}

export default App; 