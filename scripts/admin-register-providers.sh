#!/bin/bash

# Admin script to register Azure resource providers for Reframe
# Run this as an Azure administrator when service principals lack provider registration permissions

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔧 Admin: Azure Resource Provider Registration for Reframe${NC}"
echo "=============================================="
echo -e "${YELLOW}⚠️  This script should be run by an Azure subscription administrator${NC}"
echo ""

# Check if user is logged in to Azure CLI
if ! az account show &> /dev/null; then
    echo -e "${RED}❌ Please log in to Azure CLI first: az login${NC}"
    exit 1
fi

# Show current account info
ACCOUNT_INFO=$(az account show --query "{Name:name,SubscriptionId:id,User:user.name}" --output table)
echo -e "${BLUE}Current Azure account:${NC}"
echo "$ACCOUNT_INFO"
echo ""

# Confirm admin wants to proceed
read -p "Do you want to register resource providers for Reframe deployment? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Registration cancelled"
    exit 0
fi

# List of required resource providers
PROVIDERS=(
    "Microsoft.ContainerInstance"
    "Microsoft.ContainerRegistry"
    "Microsoft.Network"
    "Microsoft.Resources"
)

echo -e "${YELLOW}📋 Registering required resource providers...${NC}"

for PROVIDER in "${PROVIDERS[@]}"; do
    echo ""
    echo -e "${BLUE}Processing: $PROVIDER${NC}"
    
    PROVIDER_STATE=$(az provider show --namespace "$PROVIDER" --query "registrationState" --output tsv 2>/dev/null || echo "NotRegistered")
    
    if [ "$PROVIDER_STATE" != "Registered" ]; then
        echo -e "${YELLOW}🔄 Registering $PROVIDER...${NC}"
        az provider register --namespace "$PROVIDER"
        echo -e "${GREEN}✅ $PROVIDER registration initiated${NC}"
    else
        echo -e "${GREEN}✅ $PROVIDER already registered${NC}"
    fi
done

echo ""
echo -e "${YELLOW}⏳ Waiting for all registrations to complete...${NC}"
echo "This can take 5-15 minutes for the first registration."

# Wait for all providers to be registered
for PROVIDER in "${PROVIDERS[@]}"; do
    echo -n "Waiting for $PROVIDER... "
    while [ "$(az provider show --namespace "$PROVIDER" --query 'registrationState' --output tsv 2>/dev/null || echo 'NotRegistered')" != "Registered" ]; do
        echo -n "."
        sleep 10
    done
    echo " ✅ Complete"
done

echo ""
echo -e "${GREEN}🎉 All required resource providers are now registered!${NC}"
echo "=============================================="
echo -e "${BLUE}Final status:${NC}"

for PROVIDER in "${PROVIDERS[@]}"; do
    STATE=$(az provider show --namespace "$PROVIDER" --query "registrationState" --output tsv)
    echo "  • $PROVIDER: $STATE"
done

echo ""
echo -e "${YELLOW}📋 Next steps:${NC}"
echo "1. ✅ Resource providers are now registered"
echo "2. 🚀 GitHub Actions CI/CD will now work without permission issues"
echo "3. 🔒 HTTPS deployment will proceed automatically"
echo "4. 📊 Monitor deployment in GitHub Actions"
echo ""
echo -e "${BLUE}The development team can now:${NC}"
echo "• Push code changes to trigger automatic deployment"
echo "• Use manual deployment scripts without provider registration errors"
echo "• Deploy HTTPS infrastructure successfully" 