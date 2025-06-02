#!/bin/bash

# Script to fix service principal permissions for resource provider registration

set -e

# Configuration
SUBSCRIPTION_ID=""
SERVICE_PRINCIPAL_NAME="sp-reframe-cicd"
RESOURCE_GROUP="rg-reframe-prod"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔧 Fixing Service Principal Permissions for Reframe${NC}"
echo "=============================================="

# Check if user is logged in to Azure CLI
if ! az account show &> /dev/null; then
    echo -e "${RED}❌ Please log in to Azure CLI first: az login${NC}"
    exit 1
fi

# Get subscription ID if not set
if [ -z "$SUBSCRIPTION_ID" ]; then
    SUBSCRIPTION_ID=$(az account show --query id --output tsv)
    echo -e "${BLUE}Using subscription: $SUBSCRIPTION_ID${NC}"
fi

# Find the service principal
echo -e "${YELLOW}📋 Finding service principal: $SERVICE_PRINCIPAL_NAME${NC}"
SP_APP_ID=$(az ad sp list --display-name "$SERVICE_PRINCIPAL_NAME" --query "[0].appId" --output tsv)

if [ -z "$SP_APP_ID" ] || [ "$SP_APP_ID" == "null" ]; then
    echo -e "${RED}❌ Service principal not found: $SERVICE_PRINCIPAL_NAME${NC}"
    echo "Available service principals:"
    az ad sp list --query "[?contains(displayName, 'reframe')].{Name:displayName,AppId:appId}" --output table
    exit 1
fi

echo -e "${GREEN}✅ Found service principal: $SP_APP_ID${NC}"

# Check current role assignments
echo -e "${YELLOW}📋 Current role assignments:${NC}"
az role assignment list --assignee "$SP_APP_ID" --query "[].{Role:roleDefinitionName,Scope:scope}" --output table

# Add Resource Provider Contributor role at subscription level
echo -e "${YELLOW}🔄 Adding Resource Provider Contributor role...${NC}"
az role assignment create \
    --assignee "$SP_APP_ID" \
    --role "Resource Provider Contributor" \
    --scope "/subscriptions/$SUBSCRIPTION_ID" \
    2>/dev/null || echo -e "${YELLOW}⚠️  Role may already exist${NC}"

# Keep the existing Contributor role at resource group level
echo -e "${YELLOW}🔄 Ensuring Contributor role at resource group level...${NC}"
az role assignment create \
    --assignee "$SP_APP_ID" \
    --role "Contributor" \
    --scope "/subscriptions/$SUBSCRIPTION_ID/resourceGroups/$RESOURCE_GROUP" \
    2>/dev/null || echo -e "${YELLOW}⚠️  Role may already exist${NC}"

echo ""
echo -e "${GREEN}✅ Service principal permissions updated!${NC}"
echo "=============================================="
echo -e "${BLUE}Updated role assignments:${NC}"
az role assignment list --assignee "$SP_APP_ID" --query "[].{Role:roleDefinitionName,Scope:scope}" --output table

echo ""
echo -e "${YELLOW}📋 Next steps:${NC}"
echo "1. The service principal can now register resource providers"
echo "2. Re-run your GitHub Actions workflow or manual deployment"
echo "3. Resource provider registration is a one-time operation"
echo ""
echo -e "${BLUE}GitHub Actions will now work with:${NC}"
echo "• Resource provider registration"
echo "• HTTPS infrastructure deployment"
echo "• Complete CI/CD pipeline" 