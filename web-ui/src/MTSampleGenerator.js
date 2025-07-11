import React, { useState } from 'react';
import {
  Card,
  Text,
  Title,
  Button,
  Stack,
  Group,
  Box,
  Select,
  Checkbox,
  NumberInput,
  Textarea,
  Grid,
} from '@mantine/core';
import {
  IconSettings,
  IconCode,
  IconRefresh,
  IconCopy,
  IconCheck,
  IconAlertCircle,
} from '@tabler/icons-react';

// MT Message types supported by the generator
const MT_MESSAGE_TYPES = [
  { value: 'MT101', label: 'MT101 - Request for Transfer' },
  { value: 'MT103', label: 'MT103 - Single Customer Credit Transfer' },
  { value: 'MT104', label: 'MT104 - Customer Direct Debit' },
  { value: 'MT107', label: 'MT107 - Request for Direct Debit Transfer' },
  { value: 'MT110', label: 'MT110 - Advice of Cheque(s)' },
  { value: 'MT111', label: 'MT111 - Request for Stop Payment of a Cheque' },
  { value: 'MT112', label: 'MT112 - Bank to Bank Transfer' },
  { value: 'MT192', label: 'MT192 - Request for Cancellation' },
  { value: 'MT196', label: 'MT196 - Client-to-Bank Information' },
  { value: 'MT199', label: 'MT199 - Free Format Message' },
  { value: 'MT202', label: 'MT202 - General Financial Institution Transfer' },
  { value: 'MT205', label: 'MT205 - Financial Institution Transfer for its Own Account' },
  { value: 'MT210', label: 'MT210 - Notice to Receive' },
  { value: 'MT292', label: 'MT292 - Request for Cancellation (COV)' },
  { value: 'MT296', label: 'MT296 - Client-to-Bank Information (COV)' },
  { value: 'MT299', label: 'MT299 - Free Format Message (COV)' },
  { value: 'MT900', label: 'MT900 - Confirmation of Debit' },
  { value: 'MT910', label: 'MT910 - Confirmation of Credit' },
  { value: 'MT920', label: 'MT920 - Request Message' },
  { value: 'MT935', label: 'MT935 - Rate Change Advice' },
  { value: 'MT940', label: 'MT940 - Customer Statement Message' },
  { value: 'MT941', label: 'MT941 - Balance Report Message' },
  { value: 'MT942', label: 'MT942 - Interim Transaction Report' },
  { value: 'MT950', label: 'MT950 - Statement Message' },
];

const MTSampleGenerator = () => {
  const [selectedMTType, setSelectedMTType] = useState('MT103');
  const [generatedMessage, setGeneratedMessage] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState([]);
  const [success, setSuccess] = useState(false);
  
  // Configuration options state
  const [config, setConfig] = useState({
    // Basic options
    validation: true,
    includeDebug: false,
    
    // Simplified configuration
    minAmount: 100,
    maxAmount: 10000,
    currency: 'USD',
    
    // Advanced options
    useRandomData: true,
    includeOptionalFields: false,
  });

  const FIXED_CONTENT_HEIGHT = '400px';

  const handleConfigChange = (field, value) => {
    setConfig(prev => ({
      ...prev,
      [field]: value
    }));
  };

  const buildRequestConfig = () => {
    // Build MessageConfig structure that the backend expects
    const messageConfig = {
      include_optional: config.includeOptionalFields || false,
      scenario: config.useRandomData ? "StpCompliant" : "Standard",
      field_configs: {}
    };
    
    // Configure amount fields based on message type and user input
    if (config.minAmount && config.maxAmount && config.currency) {
      // Field 32A - Value Date/Currency/Amount (mandatory in most MT types)
      messageConfig.field_configs["32A"] = {
        value_range: {
          Amount: {
            min: parseFloat(config.minAmount),
            max: parseFloat(config.maxAmount),
            currency: config.currency
          }
        }
      };
      
      // Field 33B - Currency/Instructed Amount (conditional in MT103, optional in others)
      messageConfig.field_configs["33B"] = {
        value_range: {
          Amount: {
            min: parseFloat(config.minAmount),
            max: parseFloat(config.maxAmount),
            currency: config.currency
          }
        }
      };
      
      // Configure charge fields with smaller amounts (typically fees)
      const chargeAmount = Math.min(parseFloat(config.maxAmount) * 0.1, 100); // 10% of max or 100, whichever is smaller
      
      // Field 71F - Sender's charges
      messageConfig.field_configs["71F"] = {
        value_range: {
          Amount: {
            min: 1.0,
            max: chargeAmount,
            currency: config.currency
          }
        }
      };
      
      // Field 71G - Receiver's charges  
      messageConfig.field_configs["71G"] = {
        value_range: {
          Amount: {
            min: 1.0,
            max: chargeAmount,
            currency: config.currency
          }
        }
      };
      
      // Field 32B - Alternative amount field format (used in some MT types)
      messageConfig.field_configs["32B"] = {
        value_range: {
          Amount: {
            min: parseFloat(config.minAmount),
            max: parseFloat(config.maxAmount),
            currency: config.currency
          }
        }
      };
    }

    return messageConfig;
  };

  const handleGenerate = async () => {
    if (!selectedMTType) {
      setError(['Please select an MT message type']);
      return;
    }

    setLoading(true);
    setError([]);
    setSuccess(false);
    setGeneratedMessage('');

    try {
      const requestConfig = buildRequestConfig();
      
      // Log the configuration being sent for debugging
      console.log('Sending MT sample request:', {
        message_type: selectedMTType,
        config: requestConfig,
        options: {
          validation: config.validation,
          include_debug: config.includeDebug
        }
      });
      
      // Validate that the config has the correct structure
      if (!requestConfig.hasOwnProperty('include_optional')) {
        console.error('❌ Configuration missing include_optional field!', requestConfig);
        setError(['Configuration error: missing include_optional field']);
        return;
      }
      
      const response = await fetch('/generate/mt-sample', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          message_type: selectedMTType,
          config: requestConfig,
          options: {
            validation: config.validation,
            include_debug: config.includeDebug
          }
        }),
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const data = await response.json();
      
      if (data.success && data.transformed_message) {
        const messageContent = typeof data.transformed_message === 'string' 
          ? data.transformed_message 
          : JSON.stringify(data.transformed_message, null, 2);
        
        setGeneratedMessage(messageContent);
        setSuccess(true);
      } else {
        setError(data.errors || ['Sample generation failed with no specific error']);
      }
    } catch (err) {
      console.error('Sample generation error:', err);
      if (err.name === 'TypeError' && err.message.includes('fetch')) {
        setError([
          'Connection error - please ensure the Reframe server is running.',
          'Start the server with: cargo run'
        ]);
      } else {
        setError([`Sample generation failed: ${err.message}`]);
      }
    } finally {
      setLoading(false);
    }
  };

  const handleClear = () => {
    setGeneratedMessage('');
    setError([]);
    setSuccess(false);
  };

  const handleCopyToClipboard = async () => {
    try {
      await navigator.clipboard.writeText(generatedMessage);
      // Could add a toast notification here
    } catch (err) {
      console.error('Failed to copy text: ', err);
    }
  };

  const resetConfig = () => {
    setConfig({
      validation: true,
      includeDebug: false,
      minAmount: 100,
      maxAmount: 10000,
      currency: 'USD',
      useRandomData: true,
      includeOptionalFields: false,
    });
  };

  return (
    <Grid style={{ flex: 1, height: 'calc(100vh - 280px)' }}>
      {/* Configuration Panel */}
      <Grid.Col span={{ base: 12, md: 6 }} style={{ height: '100%' }}>
        <Card className="plasmatic-card plasmatic-card-detailed" style={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
          <Stack gap="md" style={{ flex: 1 }}>
            <Group justify="space-between" align="center">
              <Title order={3} style={{ color: 'var(--plasmatic-blue-green)' }}>
                MT Sample Configuration
              </Title>
              <Button
                size="sm"
                leftSection={<IconRefresh size={14} />}
                onClick={resetConfig}
                className="plasmatic-btn plasmatic-btn-secondary plasmatic-btn-sm"
              >
                Reset
              </Button>
            </Group>

            <Stack gap="md" style={{ flex: 1 }}>
              {/* Message Type Selection */}
              <Select
                label="Message Type"
                placeholder="Select MT message type"
                value={selectedMTType}
                onChange={setSelectedMTType}
                data={MT_MESSAGE_TYPES}
                searchable
                className="plasmatic-input"
                styles={{
                  input: {
                    backgroundColor: 'var(--bg-card)',
                    border: '1px solid var(--border-primary)',
                    color: 'var(--text-primary)',
                    borderRadius: 'var(--border-radius)',
                  },
                  label: { color: 'var(--text-primary)' },
                  dropdown: {
                    backgroundColor: 'var(--plasmatic-midnight-green)',
                    border: '1px solid var(--border-primary)',
                    zIndex: 9999,
                  },
                  option: {
                    backgroundColor: 'var(--plasmatic-midnight-green)',
                    color: 'var(--text-primary)',
                    '&[data-selected]': {
                      backgroundColor: 'var(--plasmatic-emerald)',
                      color: 'var(--plasmatic-midnight-green)',
                    },
                    '&[data-hovered]': {
                      backgroundColor: 'var(--bg-card-hover)',
                      color: 'var(--text-primary)',
                    },
                  },
                }}
              />

              {/* Amount Range */}
              <Box>
                <Text size="sm" fw={500} mb="xs" style={{ color: 'var(--text-primary)' }}>
                  Amount Range
                </Text>
                <Text size="xs" mb="sm" style={{ color: 'var(--text-secondary)' }}>
                  Generates realistic amounts like 1,250.00, 850.50, etc. within your range
                </Text>
                <Group grow>
                  <NumberInput
                    label="Minimum Amount"
                    value={config.minAmount}
                    onChange={(value) => handleConfigChange('minAmount', value)}
                    min={1}
                    className="plasmatic-input"
                    styles={{
                      input: {
                        backgroundColor: 'var(--bg-card)',
                        border: '1px solid var(--border-primary)',
                        color: 'var(--text-primary)',
                      },
                      label: { color: 'var(--text-primary)' }
                    }}
                  />
                  <NumberInput
                    label="Maximum Amount"
                    value={config.maxAmount}
                    onChange={(value) => handleConfigChange('maxAmount', value)}
                    min={config.minAmount || 1}
                    className="plasmatic-input"
                    styles={{
                      input: {
                        backgroundColor: 'var(--bg-card)',
                        border: '1px solid var(--border-primary)',
                        color: 'var(--text-primary)',
                      },
                      label: { color: 'var(--text-primary)' }
                    }}
                  />
                </Group>
              </Box>

              {/* Currency */}
              <Select
                label="Currency"
                description="Uses realistic weighted distribution (USD 30%, EUR 15%, etc.)"
                value={config.currency}
                onChange={(value) => handleConfigChange('currency', value)}
                data={[
                  { value: 'USD', label: 'USD - US Dollar (Most Common)' },
                  { value: 'EUR', label: 'EUR - Euro' },
                  { value: 'GBP', label: 'GBP - British Pound' },
                  { value: 'JPY', label: 'JPY - Japanese Yen' },
                  { value: 'CHF', label: 'CHF - Swiss Franc' },
                  { value: 'CAD', label: 'CAD - Canadian Dollar' },
                  { value: 'AUD', label: 'AUD - Australian Dollar' },
                  { value: 'SGD', label: 'SGD - Singapore Dollar' },
                  { value: 'AED', label: 'AED - UAE Dirham' },
                  { value: 'CNY', label: 'CNY - Chinese Yuan' }
                ]}
                className="plasmatic-input"
                styles={{
                  input: {
                    backgroundColor: 'var(--bg-card)',
                    border: '1px solid var(--border-primary)',
                    color: 'var(--text-primary)',
                  },
                  label: { color: 'var(--text-primary)' },
                  dropdown: {
                    backgroundColor: 'var(--plasmatic-midnight-green)',
                    border: '1px solid var(--border-primary)',
                    zIndex: 9999,
                  },
                  option: {
                    backgroundColor: 'var(--plasmatic-midnight-green)',
                    color: 'var(--text-primary)',
                    '&[data-selected]': {
                      backgroundColor: 'var(--plasmatic-emerald)',
                      color: 'var(--plasmatic-midnight-green)',
                    },
                    '&[data-hovered]': {
                      backgroundColor: 'var(--bg-card-hover)',
                      color: 'var(--text-primary)',
                    },
                  },
                }}
              />

              {/* Basic Options */}
              <Box>
                <Text size="sm" fw={500} mb="xs" style={{ color: 'var(--text-primary)' }}>
                  Generation Options
                </Text>
                <Text size="xs" mb="sm" style={{ color: 'var(--text-secondary)' }}>
                  v2.3.4: Enhanced with realistic BIC codes, company names, and addresses
                </Text>
                <Stack gap="xs">
                  <Checkbox
                    label="Enable validation"
                    checked={config.validation}
                    onChange={(e) => handleConfigChange('validation', e.currentTarget.checked)}
                    styles={{
                      input: { backgroundColor: 'var(--bg-card)', borderColor: 'var(--border-primary)' },
                      label: { color: 'var(--text-primary)' }
                    }}
                  />
                  <Checkbox
                    label="Include debug information"
                    checked={config.includeDebug}
                    onChange={(e) => handleConfigChange('includeDebug', e.currentTarget.checked)}
                    styles={{
                      input: { backgroundColor: 'var(--bg-card)', borderColor: 'var(--border-primary)' },
                      label: { color: 'var(--text-primary)' }
                    }}
                  />
                  <Checkbox
                    label="Use realistic business data (Recommended)"
                    description="Real bank BIC codes, company names, and addresses from major financial centers"
                    checked={config.useRandomData}
                    onChange={(e) => handleConfigChange('useRandomData', e.currentTarget.checked)}
                    styles={{
                      input: { backgroundColor: 'var(--bg-card)', borderColor: 'var(--border-primary)' },
                      label: { color: 'var(--text-primary)' }
                    }}
                  />
                  <Checkbox
                    label="Include optional fields"
                    description="Adds extra fields for more comprehensive test samples"
                    checked={config.includeOptionalFields}
                    onChange={(e) => handleConfigChange('includeOptionalFields', e.currentTarget.checked)}
                    styles={{
                      input: { backgroundColor: 'var(--bg-card)', borderColor: 'var(--border-primary)' },
                      label: { color: 'var(--text-primary)' }
                    }}
                  />
                </Stack>
              </Box>
            </Stack>

            <Group justify="space-between">
              <Button
                leftSection={<IconSettings size={16} />}
                onClick={handleGenerate}
                loading={loading}
                disabled={!selectedMTType}
                size="md"
                className="plasmatic-btn plasmatic-btn-primary"
              >
                {loading ? 'Generating...' : 'Generate Sample'}
              </Button>
              <Button
                leftSection={<IconRefresh size={16} />}
                onClick={handleClear}
                disabled={loading}
                className="plasmatic-btn plasmatic-btn-secondary"
              >
                Clear Output
              </Button>
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
                Generated MT Sample
              </Title>
              
              <Group>
                {success && (
                  <div className="plasmatic-badge">
                    <IconCheck size={12} style={{ marginRight: 4 }} />
                    Generated
                  </div>
                )}
                {generatedMessage && (
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
                  Generating {selectedMTType} sample message...
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

            {generatedMessage && (
              <Textarea
                value={generatedMessage}
                readOnly
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
                  },
                }}
              />
            )}

            {!loading && !generatedMessage && error.length === 0 && (
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
                  Generated MT sample will appear here
                </Text>
              </Box>
            )}
          </Stack>
        </Card>
      </Grid.Col>
    </Grid>
  );
};

export default MTSampleGenerator;