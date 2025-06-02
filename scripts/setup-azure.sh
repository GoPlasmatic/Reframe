#!/bin/bash

# Azure Setup Script for Reframe API Deployment
# This script sets up Azure resources and configures GitHub secrets for CI/CD

set -e

# Configuration
RESOURCE_GROUP="rg-reframe-prod"
LOCATION="eastus"
SERVICE_PRINCIPAL_NAME="sp-reframe-cicd"
SUBSCRIPTION_ID=""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check prerequisites
check_prerequisites() {
    print_status "Checking prerequisites..."
    
    if ! command -v az &> /dev/null; then
        print_error "Azure CLI is not installed. Please install it first."
        exit 1
    fi
    
    if ! command -v gh &> /dev/null; then
        print_warning "GitHub CLI is not installed. You'll need to set secrets manually."
    fi
    
    # Check if logged in to Azure
    if ! az account show &> /dev/null; then
        print_error "Not logged in to Azure. Please run 'az login' first."
        exit 1
    fi
    
    print_success "Prerequisites check completed"
}

# Get subscription ID
get_subscription_id() {
    if [ -z "$SUBSCRIPTION_ID" ]; then
        SUBSCRIPTION_ID=$(az account show --query id --output tsv)
        print_status "Using subscription: $SUBSCRIPTION_ID"
    fi
}

# Create resource group
create_resource_group() {
    print_status "Creating resource group: $RESOURCE_GROUP"
    
    if az group show --name "$RESOURCE_GROUP" &> /dev/null; then
        print_warning "Resource group $RESOURCE_GROUP already exists"
    else
        az group create --name "$RESOURCE_GROUP" --location "$LOCATION"
        print_success "Resource group created successfully"
    fi
}

# Create service principal for CI/CD
create_service_principal() {
    print_status "Creating service principal: $SERVICE_PRINCIPAL_NAME"
    
    # Check if service principal already exists
    SP_APP_ID=$(az ad sp list --display-name "$SERVICE_PRINCIPAL_NAME" --query "[0].appId" --output tsv)
    
    if [ "$SP_APP_ID" != "" ]; then
        print_warning "Service principal already exists with App ID: $SP_APP_ID"
    else
        # Create service principal
        SP_CREDENTIALS=$(az ad sp create-for-rbac \
            --name "$SERVICE_PRINCIPAL_NAME" \
            --role Contributor \
            --scopes "/subscriptions/$SUBSCRIPTION_ID/resourceGroups/$RESOURCE_GROUP" \
            --sdk-auth)
        
        SP_APP_ID=$(echo "$SP_CREDENTIALS" | jq -r '.clientId')
        print_success "Service principal created with App ID: $SP_APP_ID"
        
        # Store credentials for later use
        echo "$SP_CREDENTIALS" > sp-credentials.json
        print_status "Service principal credentials saved to sp-credentials.json"
    fi
}

# Deploy initial infrastructure
deploy_infrastructure() {
    print_status "Deploying initial infrastructure (ACR only)..."
    
    az deployment group create \
        --resource-group "$RESOURCE_GROUP" \
        --template-file infrastructure/azure-setup.bicep \
        --parameters environment=prod deployContainer=false
    
    print_success "Initial infrastructure (ACR) deployed successfully"
}

# Get ACR credentials
get_acr_credentials() {
    print_status "Retrieving ACR credentials..."
    
    # Get registry name
    REGISTRY_NAME=$(az acr list --resource-group "$RESOURCE_GROUP" --query "[0].name" --output tsv)
    
    if [ -z "$REGISTRY_NAME" ]; then
        print_error "No ACR found in resource group"
        exit 1
    fi
    
    # Enable admin user
    az acr update --name "$REGISTRY_NAME" --admin-enabled true
    
    # Get credentials
    ACR_USERNAME=$(az acr credential show --name "$REGISTRY_NAME" --query username --output tsv)
    ACR_PASSWORD=$(az acr credential show --name "$REGISTRY_NAME" --query passwords[0].value --output tsv)
    
    print_success "ACR credentials retrieved for registry: $REGISTRY_NAME"
    
    # Save to file
    cat > acr-credentials.json << EOF
{
    "registry": "$REGISTRY_NAME.azurecr.io",
    "username": "$ACR_USERNAME",
    "password": "$ACR_PASSWORD"
}
EOF
    
    print_status "ACR credentials saved to acr-credentials.json"
}

# Set GitHub secrets
set_github_secrets() {
    if ! command -v gh &> /dev/null; then
        print_warning "GitHub CLI not available. Please set the following secrets manually in your GitHub repository:"
        echo
        echo "AZURE_CREDENTIALS:"
        cat sp-credentials.json
        echo
        echo "ACR_USERNAME: $ACR_USERNAME"
        echo "ACR_PASSWORD: $ACR_PASSWORD"
        return
    fi
    
    print_status "Setting GitHub secrets..."
    
    # Check if we're in a git repository
    if ! git rev-parse --git-dir &> /dev/null; then
        print_error "Not in a git repository. Please run this script from your project root."
        return
    fi
    
    # Set secrets
    gh secret set AZURE_CREDENTIALS --body "$(cat sp-credentials.json)"
    gh secret set ACR_USERNAME --body "$ACR_USERNAME"
    gh secret set ACR_PASSWORD --body "$ACR_PASSWORD"
    
    print_success "GitHub secrets set successfully"
}

# Cleanup temporary files
cleanup() {
    print_status "Cleaning up temporary files..."
    rm -f sp-credentials.json acr-credentials.json
    print_success "Cleanup completed"
}

# Display summary
display_summary() {
    echo
    echo "=========================================="
    echo "           SETUP SUMMARY"
    echo "=========================================="
    echo "Resource Group: $RESOURCE_GROUP"
    echo "Location: $LOCATION"
    echo "ACR Registry: $REGISTRY_NAME.azurecr.io"
    echo "Service Principal: $SERVICE_PRINCIPAL_NAME"
    echo
    echo "Next steps:"
    echo "1. Push your code to the main branch to trigger deployment"
    echo "2. Monitor the GitHub Actions workflow"
    echo "3. Access your API at the URL provided in the workflow output"
    echo
    echo "Manual deployment:"
    echo "az deployment group create \\"
    echo "  --resource-group $RESOURCE_GROUP \\"
    echo "  --template-file infrastructure/azure-setup.bicep \\"
    echo "  --parameters environment=prod"
    echo "=========================================="
}

# Main execution
main() {
    echo "=========================================="
    echo "    Azure Setup for Reframe API"
    echo "=========================================="
    echo
    
    check_prerequisites
    get_subscription_id
    create_resource_group
    create_service_principal
    deploy_infrastructure
    get_acr_credentials
    set_github_secrets
    cleanup
    display_summary
}

# Run main function
main "$@" 