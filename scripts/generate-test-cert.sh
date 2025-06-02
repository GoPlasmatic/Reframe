#!/bin/bash

# Script to generate a self-signed certificate for Application Gateway testing

set -e

# Configuration
CERT_NAME="reframe-test-cert"
DOMAIN="reframe-api-prod-https.eastus.cloudapp.azure.com"
PASSWORD="TestCert123!"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔒 Generating Self-Signed Certificate for Application Gateway${NC}"
echo "=============================================="

# Check if openssl is available
if ! command -v openssl &> /dev/null; then
    echo -e "${RED}❌ OpenSSL is not installed${NC}"
    exit 1
fi

# Generate private key
echo -e "${YELLOW}🔑 Generating private key...${NC}"
openssl genrsa -out ${CERT_NAME}.key 2048

# Generate certificate signing request
echo -e "${YELLOW}📝 Creating certificate signing request...${NC}"
openssl req -new -key ${CERT_NAME}.key -out ${CERT_NAME}.csr -subj "/CN=${DOMAIN}/O=Reframe Test/C=US"

# Generate self-signed certificate
echo -e "${YELLOW}📜 Generating self-signed certificate...${NC}"
openssl x509 -req -in ${CERT_NAME}.csr -signkey ${CERT_NAME}.key -out ${CERT_NAME}.crt -days 365

# Create PFX file with password
echo -e "${YELLOW}📦 Creating PFX file...${NC}"
openssl pkcs12 -export -out ${CERT_NAME}.pfx -inkey ${CERT_NAME}.key -in ${CERT_NAME}.crt -password pass:${PASSWORD}

# Convert PFX to base64 for Bicep template
echo -e "${YELLOW}🔄 Converting to base64...${NC}"
BASE64_CERT=$(base64 -i ${CERT_NAME}.pfx | tr -d '\n')

echo ""
echo -e "${GREEN}✅ Certificate generated successfully!${NC}"
echo "=============================================="
echo -e "${BLUE}Files created:${NC}"
echo "  • ${CERT_NAME}.key (private key)"
echo "  • ${CERT_NAME}.csr (certificate request)"
echo "  • ${CERT_NAME}.crt (certificate)"
echo "  • ${CERT_NAME}.pfx (PKCS#12 format)"
echo ""
echo -e "${BLUE}For Bicep template:${NC}"
echo "  Domain: ${DOMAIN}"
echo "  Password: ${PASSWORD}"
echo "  Base64 data (first 100 chars): ${BASE64_CERT:0:100}..."
echo ""
echo -e "${YELLOW}To update Bicep template:${NC}"
echo "1. Replace the 'data' field with: ${BASE64_CERT}"
echo "2. Replace the 'password' field with: ${PASSWORD}"

# Save to file for easy copying
cat > cert-data.txt << EOF
Certificate Data for Bicep Template:
===================================
Domain: ${DOMAIN}
Password: ${PASSWORD}
Base64 Data:
${BASE64_CERT}
EOF

echo ""
echo -e "${GREEN}📁 Certificate data saved to: cert-data.txt${NC}"

# Cleanup
rm -f ${CERT_NAME}.key ${CERT_NAME}.csr ${CERT_NAME}.crt ${CERT_NAME}.pfx

echo -e "${GREEN}🧹 Temporary files cleaned up${NC}" 