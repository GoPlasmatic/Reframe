#!/bin/bash

# Simple GitHub Secrets Setup for Reframe Automated Deployment
# This script creates the Azure service principal and optionally sets GitHub secrets

set -e

# Configuration
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

# Create service principal for CI/CD
create_service_principal() {
    print_status "Creating service principal: $SERVICE_PRINCIPAL_NAME"
    
    # Check if service principal already exists
    SP_APP_ID=$(az ad sp list --display-name "$SERVICE_PRINCIPAL_NAME" --query "[0].appId" --output tsv)
    
    if [ "$SP_APP_ID" != "" ] && [ "$SP_APP_ID" != "null" ]; then
        print_warning "Service principal already exists with App ID: $SP_APP_ID"
        print_status "Recreating service principal with updated permissions..."
        
        # Delete existing service principal
        az ad sp delete --id "$SP_APP_ID"
        print_status "Deleted existing service principal"
    fi
    
    # Create service principal with subscription-level Contributor role
    print_status "Creating new service principal with Contributor role at subscription level..."
    SP_CREDENTIALS=$(az ad sp create-for-rbac \
        --name "$SERVICE_PRINCIPAL_NAME" \
        --role Contributor \
        --scopes "/subscriptions/$SUBSCRIPTION_ID" \
        --sdk-auth)
    
    SP_APP_ID=$(echo "$SP_CREDENTIALS" | jq -r '.clientId')
    print_success "Service principal created with App ID: $SP_APP_ID"
    
    # Store credentials for GitHub secrets
    echo "$SP_CREDENTIALS" > azure-credentials.json
    print_status "Service principal credentials saved to azure-credentials.json"
}

# Set GitHub secrets
set_github_secrets() {
    if ! command -v gh &> /dev/null; then
        print_warning "GitHub CLI not available. Please set the following secret manually in your GitHub repository:"
        echo
        echo "Go to: Settings → Secrets and variables → Actions"
        echo "Add secret: AZURE_CREDENTIALS"
        echo "Value:"
        cat azure-credentials.json
        echo
        return
    fi
    
    print_status "Setting GitHub secrets..."
    
    # Check if we're in a git repository
    if ! git rev-parse --git-dir &> /dev/null; then
        print_error "Not in a git repository. Please run this script from your project root."
        return
    fi
    
    # Set secret
    gh secret set AZURE_CREDENTIALS --body "$(cat azure-credentials.json)"
    
    print_success "GitHub secret AZURE_CREDENTIALS set successfully"
}

# Cleanup temporary files
cleanup() {
    print_status "Cleaning up temporary files..."
    rm -f azure-credentials.json
    print_success "Cleanup completed"
}

# Display summary
display_summary() {
    echo
    echo "=========================================="
    echo "           SETUP SUMMARY"
    echo "=========================================="
    echo "Service Principal: $SERVICE_PRINCIPAL_NAME"
    echo "Subscription: $SUBSCRIPTION_ID"
    echo
    echo "✅ Azure service principal created with Contributor role"
    echo "✅ GitHub secret configured (or instructions provided)"
    echo
    echo "Next steps:"
    echo "1. Push your code to the main branch"
    echo "2. The workflow will automatically:"
    echo "   • Register Azure resource providers"
    echo "   • Create resource group and ACR"
    echo "   • Build and deploy your application"
    echo "3. Monitor progress in GitHub Actions tab"
    echo
    echo "🚀 Your deployment is now fully automated!"
    echo "=========================================="
}

# Main execution
main() {
    echo "=========================================="
    echo "    GitHub Secrets Setup for Reframe"
    echo "=========================================="
    echo
    
    check_prerequisites
    get_subscription_id
    create_service_principal
    set_github_secrets
    cleanup
    display_summary
}

# Run main function
main "$@" 