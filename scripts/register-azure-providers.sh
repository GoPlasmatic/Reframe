#!/bin/bash

# Script to register required Azure resource providers for Reframe deployment

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔧 Azure Resource Provider Registration for Reframe${NC}"
echo "=============================================="

# Check if user is logged in to Azure CLI
if ! az account show &> /dev/null; then
    echo -e "${RED}❌ Please log in to Azure CLI first: az login${NC}"
    exit 1
fi

# List of required resource providers
PROVIDERS=(
    "Microsoft.ContainerInstance"
    "Microsoft.ContainerRegistry"
    "Microsoft.Network"
    "Microsoft.Resources"
)

echo -e "${YELLOW}📋 Checking and registering required resource providers...${NC}"

for PROVIDER in "${PROVIDERS[@]}"; do
    echo ""
    echo -e "${BLUE}Checking: $PROVIDER${NC}"
    
    PROVIDER_STATE=$(az provider show --namespace "$PROVIDER" --query "registrationState" --output tsv 2>/dev/null || echo "NotRegistered")
    
    if [ "$PROVIDER_STATE" != "Registered" ]; then
        echo -e "${YELLOW}🔄 Registering $PROVIDER...${NC}"
        az provider register --namespace "$PROVIDER"
        
        # Wait for registration to complete
        echo -e "${YELLOW}⏳ Waiting for $PROVIDER registration...${NC}"
        while [ "$(az provider show --namespace "$PROVIDER" --query 'registrationState' --output tsv 2>/dev/null || echo 'NotRegistered')" != "Registered" ]; do
            echo "  Still registering... waiting 15 seconds"
            sleep 15
        done
        echo -e "${GREEN}✅ $PROVIDER registered successfully${NC}"
    else
        echo -e "${GREEN}✅ $PROVIDER already registered${NC}"
    fi
done

echo ""
echo -e "${GREEN}🎉 All required resource providers are registered!${NC}"
echo "=============================================="
echo -e "${BLUE}Registered providers:${NC}"

for PROVIDER in "${PROVIDERS[@]}"; do
    STATE=$(az provider show --namespace "$PROVIDER" --query "registrationState" --output tsv)
    echo "  • $PROVIDER: $STATE"
done

echo ""
echo -e "${YELLOW}📋 Next steps:${NC}"
echo "1. Run your deployment script"
echo "2. Or push to main branch to trigger CI/CD"
echo "" 