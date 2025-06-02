#!/bin/bash

# Setup HTTPS for Reframe API using Azure Application Gateway
# This script deploys an Application Gateway in front of the existing ACI to enable HTTPS

set -e

# Configuration
RESOURCE_GROUP="rg-reframe-prod"
LOCATION="eastus"
NAME_PREFIX="reframe"
ENVIRONMENT="prod"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔒 Setting up HTTPS for Reframe API${NC}"
echo "=================================================="

# Check if user is logged in to Azure CLI
if ! az account show &> /dev/null; then
    echo -e "${RED}❌ Please log in to Azure CLI first: az login${NC}"
    exit 1
fi

# Get the current ACI FQDN
echo -e "${YELLOW}📋 Getting current ACI information...${NC}"
ACI_NAME="reframe-api-prod"
ACI_FQDN=$(az container show \
    --resource-group "$RESOURCE_GROUP" \
    --name "$ACI_NAME" \
    --query "ipAddress.fqdn" \
    --output tsv 2>/dev/null || echo "")

if [ -z "$ACI_FQDN" ]; then
    echo -e "${RED}❌ Could not find existing ACI deployment${NC}"
    echo "Please ensure your ACI is deployed first"
    exit 1
fi

echo -e "${GREEN}✅ Found ACI at: $ACI_FQDN${NC}"

# Check if Application Gateway already exists
APPGW_NAME="reframe-appgw-prod"
if az network application-gateway show --resource-group "$RESOURCE_GROUP" --name "$APPGW_NAME" &> /dev/null; then
    echo -e "${YELLOW}⚠️  Application Gateway already exists${NC}"
    read -p "Do you want to update it? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Skipping deployment"
        exit 0
    fi
fi

# Deploy the HTTPS infrastructure
echo -e "${YELLOW}🚀 Deploying HTTPS infrastructure...${NC}"
echo "This will create:"
echo "  • Virtual Network with Application Gateway subnet"
echo "  • Network Security Group with HTTPS rules"
echo "  • Public IP with DNS label"
echo "  • Application Gateway with SSL termination"
echo "  • HTTP to HTTPS redirect"

DEPLOYMENT_NAME="https-setup-$(date +%Y%m%d-%H%M%S)"

az deployment group create \
    --resource-group "$RESOURCE_GROUP" \
    --template-file "infrastructure/azure-https-setup.bicep" \
    --parameters \
        location="$LOCATION" \
        namePrefix="$NAME_PREFIX" \
        environment="$ENVIRONMENT" \
        aciFqdn="$ACI_FQDN" \
    --name "$DEPLOYMENT_NAME" \
    --verbose

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ HTTPS infrastructure deployed successfully!${NC}"
    
    # Get the new HTTPS endpoint
    HTTPS_ENDPOINT=$(az deployment group show \
        --resource-group "$RESOURCE_GROUP" \
        --name "$DEPLOYMENT_NAME" \
        --query "properties.outputs.httpsEndpoint.value" \
        --output tsv)
    
    PUBLIC_IP=$(az deployment group show \
        --resource-group "$RESOURCE_GROUP" \
        --name "$DEPLOYMENT_NAME" \
        --query "properties.outputs.publicIpAddress.value" \
        --output tsv)
    
    echo ""
    echo -e "${GREEN}🎉 HTTPS Setup Complete!${NC}"
    echo "=================================================="
    echo -e "${BLUE}HTTPS Endpoint:${NC} $HTTPS_ENDPOINT/reframe"
    echo -e "${BLUE}Public IP:${NC} $PUBLIC_IP"
    echo -e "${BLUE}Health Check:${NC} $HTTPS_ENDPOINT/health"
    echo ""
    echo -e "${YELLOW}📋 Next Steps:${NC}"
    echo "1. Test the HTTPS endpoint:"
    echo "   curl -k $HTTPS_ENDPOINT/health"
    echo ""
    echo "2. Update your web UI to use the HTTPS endpoint"
    echo ""
    echo "3. Wait 5-10 minutes for the Application Gateway to be fully ready"
    echo ""
    echo -e "${YELLOW}⚠️  Note:${NC} This setup uses a self-signed certificate for testing."
    echo "For production, replace with a valid SSL certificate."
    
else
    echo -e "${RED}❌ Deployment failed${NC}"
    exit 1
fi 