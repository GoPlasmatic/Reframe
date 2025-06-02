#!/bin/bash

# Manual HTTPS Deployment Script for Reframe API
# This script deploys HTTPS infrastructure independently of the CI/CD pipeline

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

echo -e "${BLUE}🔒 Manual HTTPS Deployment for Reframe API${NC}"
echo "=============================================="

# Check if user is logged in to Azure CLI
if ! az account show &> /dev/null; then
    echo -e "${RED}❌ Please log in to Azure CLI first: az login${NC}"
    exit 1
fi

# Check and register Microsoft.Network resource provider if needed
echo -e "${YELLOW}📋 Checking Microsoft.Network resource provider registration...${NC}"
NETWORK_PROVIDER_STATE=$(az provider show --namespace Microsoft.Network --query "registrationState" --output tsv 2>/dev/null || echo "NotRegistered")

if [ "$NETWORK_PROVIDER_STATE" != "Registered" ]; then
    echo -e "${YELLOW}🔄 Registering Microsoft.Network resource provider...${NC}"
    az provider register --namespace Microsoft.Network
    
    # Wait for registration to complete
    echo -e "${YELLOW}⏳ Waiting for Microsoft.Network provider registration...${NC}"
    while [ "$(az provider show --namespace Microsoft.Network --query 'registrationState' --output tsv 2>/dev/null || echo 'NotRegistered')" != "Registered" ]; do
        echo "Still registering... waiting 30 seconds"
        sleep 30
    done
    echo -e "${GREEN}✅ Microsoft.Network provider registered successfully${NC}"
else
    echo -e "${GREEN}✅ Microsoft.Network provider already registered${NC}"
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
    echo -e "${RED}❌ Could not find existing ACI deployment: $ACI_NAME${NC}"
    echo "Available container instances:"
    az container list --resource-group "$RESOURCE_GROUP" --query "[].{Name:name,FQDN:ipAddress.fqdn,Status:provisioningState}" --output table
    exit 1
fi

echo -e "${GREEN}✅ Found ACI at: $ACI_FQDN${NC}"

# Check if Application Gateway already exists
APPGW_NAME="reframe-appgw-prod"
if az network application-gateway show --resource-group "$RESOURCE_GROUP" --name "$APPGW_NAME" &> /dev/null; then
    echo -e "${YELLOW}⚠️  Application Gateway already exists${NC}"
    
    HTTPS_ENDPOINT=$(az network public-ip show \
        --resource-group "$RESOURCE_GROUP" \
        --name "reframe-appgw-pip-prod" \
        --query "dnsSettings.fqdn" \
        --output tsv 2>/dev/null || echo "")
    
    if [ -n "$HTTPS_ENDPOINT" ]; then
        echo -e "${GREEN}✅ HTTPS endpoint already available: https://$HTTPS_ENDPOINT${NC}"
        exit 0
    fi
    
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
echo ""

DEPLOYMENT_NAME="https-manual-$(date +%Y%m%d-%H%M%S)"

echo -e "${BLUE}Deployment parameters:${NC}"
echo "  Resource Group: $RESOURCE_GROUP"
echo "  ACI FQDN: $ACI_FQDN"
echo "  Location: $LOCATION"
echo "  Environment: $ENVIRONMENT"
echo ""

read -p "Proceed with deployment? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Deployment cancelled"
    exit 0
fi

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
    echo "=============================================="
    echo -e "${BLUE}HTTPS Endpoint:${NC} $HTTPS_ENDPOINT/reframe"
    echo -e "${BLUE}Health Check:${NC} $HTTPS_ENDPOINT/health"
    echo -e "${BLUE}Public IP:${NC} $PUBLIC_IP"
    echo ""
    echo -e "${YELLOW}📋 Next Steps:${NC}"
    echo "1. Wait 5-10 minutes for the Application Gateway to be fully ready"
    echo ""
    echo "2. Test the HTTPS endpoint:"
    echo "   curl -k $HTTPS_ENDPOINT/health"
    echo ""
    echo "3. Test with sample data:"
    echo '   curl -k -X POST "'$HTTPS_ENDPOINT'/reframe" \'
    echo '     -H "Content-Type: text/plain" \'
    echo '     -d "{1:F01BNPAFRPPXXX0000000000}{2:O1031234240101DEUTDEFFXXXX12345678952401011234N}{3:{103:EBA}}{4:'
    echo '   :20:FT21001234567890'
    echo '   :23B:CRED'
    echo '   :32A:240101USD1000,00'
    echo '   :50K:/1234567890'
    echo '   ACME CORPORATION'
    echo '   :52A:BNPAFRPPXXX'
    echo '   :57A:DEUTDEFFXXX'
    echo '   :59:/DE89370400440532013000'
    echo '   MUELLER GMBH'
    echo '   :70:PAYMENT FOR INVOICE 12345'
    echo '   :71A:OUR'
    echo '   -}"'
    echo ""
    echo "4. Update your web UI configuration with the new HTTPS endpoint"
    echo ""
    echo -e "${YELLOW}⚠️  Note:${NC} This setup uses a self-signed certificate for testing."
    echo "For production, replace with a valid SSL certificate in the Bicep template."
    echo ""
    echo -e "${BLUE}Monitoring:${NC}"
    echo "- View Application Gateway: https://portal.azure.com/"
    echo "- Check logs: az network application-gateway show-backend-health --resource-group $RESOURCE_GROUP --name $APPGW_NAME"
    
else
    echo -e "${RED}❌ Deployment failed${NC}"
    echo "Check the deployment logs in Azure Portal or run:"
    echo "az deployment group show --resource-group $RESOURCE_GROUP --name $DEPLOYMENT_NAME"
    exit 1
fi 