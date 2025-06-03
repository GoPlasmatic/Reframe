# Scripts Directory

This directory contains setup scripts for the Reframe automated deployment.

## Available Scripts

### `setup-github-secrets.sh`

**Purpose**: Creates Azure service principal and configures GitHub secrets for automated deployment.

**What it does**:
- Creates an Azure service principal with Contributor role at subscription level
- Generates the required credentials for GitHub Actions
- Optionally sets GitHub secrets automatically (if GitHub CLI is available)
- Provides manual instructions if GitHub CLI is not installed

**Prerequisites**:
- Azure CLI installed and logged in (`az login`)
- GitHub CLI installed (optional, for automatic secret setup)
- Azure subscription with appropriate permissions

**Usage**:
```bash
chmod +x scripts/setup-github-secrets.sh
./scripts/setup-github-secrets.sh
```

**Output**:
- Creates service principal `sp-reframe-cicd`
- Sets GitHub secret `AZURE_CREDENTIALS` (or provides instructions)
- Displays next steps for deployment

## Automated Deployment

After running the setup script, the GitHub Actions workflow will automatically handle:

1. **Azure Resource Provider Registration**
2. **Resource Group Creation** (`rg-reframe-prod`)
3. **Azure Container Registry Setup**
4. **Container Image Building and Deployment**
5. **Staging and Production Deployments**

## No Manual Infrastructure Setup Required

All Azure infrastructure provisioning is now automated in the GitHub workflow. The scripts directory only contains the minimal setup needed to configure GitHub Actions authentication.

## Migration from Previous Setup

If you previously used the old setup scripts, you can safely delete any existing Azure resources and let the new automated workflow recreate them, or continue using existing resources - the workflow will detect and use them automatically. 