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
  Box,
  Paper,
  Tabs,
  Select,
} from '@mantine/core';
import {
  IconCode,
  IconCheck,
  IconAlertCircle,
  IconRefresh,
  IconArrowRight,
  IconArrowLeft,
  IconExclamationMark,
  IconCopy,
  IconBrandGithub,
  IconSettings,
} from '@tabler/icons-react';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { vscDarkPlus } from 'react-syntax-highlighter/dist/esm/styles/prism';
import { FORWARD_TRANSFORMATIONS, REVERSE_TRANSFORMATIONS } from './transformationSamples';
import MTSampleGenerator from './MTSampleGenerator';

// API Configuration - using relative URL since we're serving from the same origin
const API_ENDPOINTS = {
  forward: '/transform/mt-to-mx',
  reverse: '/transform/mx-to-mt',
  legacy: '/reframe'
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
    <Box style={{ 
      minHeight: '100vh', 
      display: 'flex', 
      flexDirection: 'column',
      background: 'linear-gradient(135deg, var(--plasmatic-midnight-green) 0%, var(--plasmatic-midnight-green-dark) 100%)'
    }}>
      <Container 
        className="plasmatic-container" 
        style={{ 
          flex: 1, 
          display: 'flex', 
          flexDirection: 'column', 
          width: '100%', 
          maxWidth: '100%', 
          height: '100%' 
        }}
      >
        <Stack gap="lg" style={{ flex: 1 }} className="plasmatic-fade-in">
          {/* Header */}
          <Paper className="plasmatic-card plasmatic-card-detailed" style={{ marginBottom: 0 }}>
            <Group justify="space-between" align="center">
              <div style={{ display: 'flex', alignItems: 'center', gap: '16px' }}>
                <img 
                  src="/plasmatic_logo.png" 
                  alt="Plasmatic Logo" 
                  style={{ 
                    height: '48px', 
                    width: 'auto',
                    filter: 'drop-shadow(0 0 10px rgba(0, 255, 135, 0.3))'
                  }} 
                />
                <div>
                  <Title order={1} style={{ color: 'var(--text-primary)', fontFamily: 'var(--font-heading)' }}>
                    Reframe
                  </Title>
                  <Text className="plasmatic-text-secondary" size="sm">
                    Convert between SWIFT MT and ISO 20022 MX message formats
                  </Text>
                </div>
              </div>
              <Group gap="md">
                <Button
                  component="a"
                  href="https://github.com/GoPlasmatic/Reframe"
                  target="_blank"
                  rel="noopener noreferrer"
                  leftSection={<IconBrandGithub size={16} />}
                  className="plasmatic-btn plasmatic-btn-secondary plasmatic-btn-sm"
                  style={{ textDecoration: 'none' }}
                >
                  GitHub
                </Button>
                <div className="plasmatic-badge">
                  <IconCheck size={16} style={{ marginRight: '4px' }} />
                  v2.2.0 - Bidirectional
                </div>
              </Group>
            </Group>
          </Paper>

          {/* Tabs */}
          <Tabs value={activeTab} onChange={setActiveTab} className="plasmatic-slide-in">
            <Tabs.List style={{ backgroundColor: 'var(--bg-card)', borderRadius: 'var(--border-radius)' }}>
              <Tabs.Tab 
                value="forward" 
                leftSection={<IconArrowRight size={16} />}
                style={{ 
                  color: activeTab === 'forward' ? 'var(--text-primary)' : 'var(--text-secondary)',
                  borderColor: activeTab === 'forward' ? 'var(--plasmatic-emerald)' : 'transparent'
                }}
              >
                Forward (MT → MX)
              </Tabs.Tab>
              <Tabs.Tab 
                value="reverse" 
                leftSection={<IconArrowLeft size={16} />}
                style={{ 
                  color: activeTab === 'reverse' ? 'var(--text-primary)' : 'var(--text-secondary)',
                  borderColor: activeTab === 'reverse' ? 'var(--plasmatic-emerald)' : 'transparent'
                }}
              >
                Reverse (MX → MT)
              </Tabs.Tab>
              <Tabs.Tab 
                value="generator" 
                leftSection={<IconSettings size={16} />}
                style={{ 
                  color: activeTab === 'generator' ? 'var(--text-primary)' : 'var(--text-secondary)',
                  borderColor: activeTab === 'generator' ? 'var(--plasmatic-emerald)' : 'transparent'
                }}
              >
                MT Sample Generator
              </Tabs.Tab>
            </Tabs.List>

            <Tabs.Panel value="forward" pt="md">
              <div className="plasmatic-alert success">
                <IconArrowRight size={16} style={{ color: 'var(--plasmatic-emerald)' }} />
                <div>
                  <strong>Forward Transformation:</strong> Convert SWIFT MT messages to ISO 20022 MX format
                </div>
              </div>
            </Tabs.Panel>

            <Tabs.Panel value="reverse" pt="md">
              <div className="plasmatic-alert warning">
                <IconArrowLeft size={16} style={{ color: 'var(--plasmatic-sun-glow)' }} />
                <div>
                  <strong>Reverse Transformation:</strong> Convert ISO 20022 MX messages to SWIFT MT format
                </div>
              </div>
            </Tabs.Panel>

            <Tabs.Panel value="generator" pt="md">
              <div className="plasmatic-alert info">
                <IconSettings size={16} style={{ color: 'var(--plasmatic-blue-green)' }} />
                <div>
                  <strong>MT Sample Generator v2.3.4:</strong> Generate realistic SWIFT MT messages with enhanced BIC codes, company names, and addresses from major financial centers
                </div>
              </div>
            </Tabs.Panel>
          </Tabs>

          {/* Main Content Grid */}
          {activeTab === 'generator' ? (
            <MTSampleGenerator />
          ) : (
            <Grid style={{ flex: 1, height: 'calc(100vh - 280px)' }}>
            {/* Input Panel */}
            <Grid.Col span={{ base: 12, md: 6 }} style={{ height: '100%' }}>
              <Card className="plasmatic-card plasmatic-card-detailed" style={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
                <Stack gap="md" style={{ flex: 1 }}>
                  <Group justify="space-between" align="center">
                    <Title order={3} style={{ color: 'var(--plasmatic-blue-green)' }}>
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
                      className="plasmatic-input"
                      styles={{
                        input: {
                          backgroundColor: 'var(--bg-card)',
                          border: '1px solid var(--border-primary)',
                          color: 'var(--text-primary)',
                          borderRadius: 'var(--border-radius)',
                        },
                        dropdown: {
                          backgroundColor: 'var(--bg-card)',
                          border: '1px solid var(--border-primary)',
                        },
                        option: {
                          backgroundColor: 'var(--bg-card)',
                          color: 'var(--text-primary)',
                        },
                      }}
                    />
                  </Group>
                  
                  <Box>
                    <Text size="sm" className="plasmatic-text-secondary" mb="xs">
                      {getCurrentSelectedTransformation().description}
                    </Text>
                    <div className="plasmatic-badge secondary">
                      Target: {getCurrentSelectedTransformation().targetFormat}
                    </div>
                  </Box>

                  <Textarea
                    value={inputMessage}
                    onChange={(e) => setInputMessage(e.target.value)}
                    placeholder={`Enter your ${activeTab === 'forward' ? 'SWIFT MT' : 'ISO 20022 MX'} message here...`}
                    style={{ flex: 1, minHeight: FIXED_CONTENT_HEIGHT }}
                    className="plasmatic-textarea"
                    styles={{
                      input: {
                        backgroundColor: 'var(--bg-card)',
                        border: '1px solid var(--border-primary)',
                        color: 'var(--text-primary)',
                        fontFamily: 'Monaco, Consolas, "Courier New", monospace',
                        fontSize: '12px',
                        height: FIXED_CONTENT_HEIGHT,
                        resize: 'none',
                        overflow: 'auto',
                        '&:focus': {
                          borderColor: 'var(--plasmatic-emerald)',
                          boxShadow: '0 0 0 3px rgba(0, 255, 135, 0.1)',
                        },
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
                        className="plasmatic-btn plasmatic-btn-primary"
                      >
                        {loading ? 'Transforming...' : 'Transform'}
                      </Button>
                      <Button
                        leftSection={<IconRefresh size={16} />}
                        onClick={handleClear}
                        disabled={loading}
                        className="plasmatic-btn plasmatic-btn-secondary"
                      >
                        Clear
                      </Button>
                    </Group>
                  </Group>
                </Stack>
              </Card>
            </Grid.Col>

            {/* Output Panel */}
            <Grid.Col span={{ base: 12, md: 6 }} style={{ height: '100%' }}>
              <Card className="plasmatic-card plasmatic-card-detailed" style={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
                <Stack gap="md" style={{ flex: 1 }}>
                  <Group justify="space-between" align="center">
                    <Title order={3} style={{ color: 'var(--plasmatic-emerald)' }}>
                      {activeTab === 'forward' ? 'Output MX Message' : 'Output MT Message'}
                    </Title>
                    
                    <Group>
                      {success && resultCount > 0 && (
                        <div className="plasmatic-badge">
                          <IconCheck size={12} style={{ marginRight: 4 }} />
                          {resultCount} result{resultCount > 1 ? 's' : ''}
                        </div>
                      )}
                      {outputMessage && (
                        <Button
                          size="sm"
                          leftSection={<IconCopy size={14} />}
                          onClick={handleCopyToClipboard}
                          className="plasmatic-btn plasmatic-btn-secondary plasmatic-btn-sm"
                        >
                          Copy
                        </Button>
                      )}
                    </Group>
                  </Group>

                  {loading && (
                    <Box ta="center" py="xl">
                      <div className="plasmatic-loader plasmatic-pulse" style={{ margin: '0 auto' }}></div>
                      <Text size="sm" className="plasmatic-text-secondary" mt="md">
                        Processing your {activeTab === 'forward' ? 'MT to MX' : 'MX to MT'} transformation...
                      </Text>
                      <div className="plasmatic-progress plasmatic-mt-2">
                        <div className="plasmatic-progress-bar" style={{ width: '75%' }}></div>
                      </div>
                    </Box>
                  )}

                  {error.length > 0 && (
                    <div className="plasmatic-alert error">
                      <IconAlertCircle size={16} style={{ color: 'var(--plasmatic-amaranth)' }} />
                      <Stack gap="xs">
                        {error.map((err, index) => (
                          <Text key={index} size="sm">
                            {err}
                          </Text>
                        ))}
                      </Stack>
                    </div>
                  )}

                  {warnings.length > 0 && (
                    <div className="plasmatic-alert warning">
                      <IconExclamationMark size={16} style={{ color: 'var(--plasmatic-sun-glow)' }} />
                      <Stack gap="xs">
                        <Text size="sm" fw={500}>Warnings:</Text>
                        {warnings.map((warning, index) => (
                          <Text key={index} size="sm">
                            {warning}
                          </Text>
                        ))}
                      </Stack>
                    </div>
                  )}

                  {outputMessage && (
                    <Box style={{ 
                      height: '100%', 
                      maxHeight: FIXED_CONTENT_HEIGHT, 
                      border: '1px solid var(--border-primary)', 
                      borderRadius: 'var(--border-radius)', 
                      overflow: 'auto',
                      backgroundColor: 'var(--bg-card)',
                    }}>
                      {activeTab === 'forward' ? (
                        <SyntaxHighlighter
                          language="xml"
                          style={vscDarkPlus}
                          customStyle={{
                            margin: 0,
                            padding: '12px',
                            borderRadius: 'var(--border-radius)',
                            fontSize: '12px',
                            fontFamily: 'Monaco, Consolas, "Courier New", monospace',
                            height: '100%',
                            overflow: 'auto',
                            backgroundColor: 'var(--bg-card)',
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
                            backgroundColor: 'var(--bg-card)',
                            color: 'var(--text-primary)',
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
                    <Box 
                      ta="center" 
                      py="xl" 
                      className="plasmatic-text-muted" 
                      style={{ 
                        height: FIXED_CONTENT_HEIGHT, 
                        display: 'flex', 
                        flexDirection: 'column', 
                        justifyContent: 'center', 
                        border: '1px solid var(--border-primary)', 
                        borderRadius: 'var(--border-radius)',
                        backgroundColor: 'var(--bg-card)'
                      }}
                    >
                      <IconCode size={48} style={{ opacity: 0.3, margin: '0 auto', color: 'var(--text-muted)' }} />
                      <Text size="sm" mt="md" className="plasmatic-text-muted">
                        Transformed {activeTab === 'forward' ? 'XML' : 'MT'} output will appear here
                      </Text>
                    </Box>
                  )}
                </Stack>
              </Card>
            </Grid.Col>
          </Grid>
          )}

          {/* Footer */}
          <Paper className="plasmatic-card" style={{ backgroundColor: 'var(--bg-secondary)', marginTop: 'auto' }}>
            <Text size="sm" className="plasmatic-text-center plasmatic-text-muted">
              Reframe v2.2.0 - Bidirectional SWIFT MT ↔ ISO 20022 MX Transformation Engine | 
              Powered by Rust + CBPR+ Translation Rules
            </Text>
          </Paper>
        </Stack>
      </Container>
    </Box>
  );
}

export default App;