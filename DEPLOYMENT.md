# Automated Azure Deployment Guide

This guide explains how to deploy the Reframe SWIFT MT to ISO 20022 converter to Azure using the fully automated GitHub Actions CI/CD pipeline.

## Architecture Overview

The deployment uses **Azure Container Instances (ACI)** as the most cost-effective solution for publicly accessible container services:

- **Azure Container Registry (ACR)**: Stores container images
- **Azure Container Instances (ACI)**: Runs the containerized API
- **GitHub Actions**: Fully automated CI/CD pipeline with infrastructure provisioning
- **Cost**: ~$15-30/month for light usage

## Prerequisites

Before deploying, ensure you have:

1. **Azure subscription** with appropriate permissions
2. **GitHub repository** with the source code
3. **Azure service principal** configured for GitHub Actions

## Quick Setup

### 1. Create Azure Service Principal

**Option A: Using the Setup Script (Recommended)**

```bash
# Make script executable and run
chmod +x scripts/setup-github-secrets.sh
./scripts/setup-github-secrets.sh
```

This script will:
- Create the Azure service principal with proper permissions
- Automatically set GitHub secrets (if GitHub CLI is available)
- Provide manual instructions if GitHub CLI is not installed

**Option B: Manual Creation**

```bash
# Login to Azure
az login

# Create service principal with Contributor role at subscription level
az ad sp create-for-rbac \
  --name "sp-reframe-cicd" \
  --role "Contributor" \
  --scopes "/subscriptions/{your-subscription-id}" \
  --sdk-auth
```

Copy the entire JSON output - you'll need it for GitHub secrets.

### 2. Configure GitHub Secrets (if not done by script)

In your GitHub repository, go to **Settings → Secrets and variables → Actions** and add:

- `AZURE_CREDENTIALS`: Paste the complete JSON output from the service principal creation

That's it! The workflow will automatically handle:
- Azure resource provider registration
- Resource group creation
- Azure Container Registry setup
- Infrastructure deployment
- Container image building and deployment

### 3. Deploy

Push to main branch to trigger the automated deployment:

```bash
git add .
git commit -m "Initial deployment setup"
git push origin main
```

## Automated Workflow Overview

The GitHub Actions workflow (`.github/workflows/deploy-azure.yml`) provides complete automation:

### 1. Test Stage
- Rust format checking (`cargo fmt`)
- Linting with Clippy (`cargo clippy`)
- Unit tests (`cargo test`)

### 2. Web UI Build
- Node.js setup and dependency installation
- React application build
- Static file preparation

### 3. Azure Infrastructure Setup
- **Automatic detection**: Checks if infrastructure already exists
- **Resource provider registration**: Registers required Azure providers
- **Resource group creation**: Creates `rg-reframe-prod` in East US
- **ACR deployment**: Sets up Azure Container Registry
- **Credential management**: Automatically configures registry access

### 4. Build & Push Stage
- Multi-architecture Docker build
- Automatic ACR credential retrieval
- Push to Azure Container Registry
- Image tagging with Git SHA

### 5. Staging Deployment
- Deploy to staging ACI instance
- Automated API testing
- Health check validation

### 6. Production Deployment
- Deploy to production ACI instance
- Manual approval required (GitHub environment protection)
- Comprehensive testing
- Cleanup staging resources

## Environment Configuration

### Staging Environment
- **CPU**: 0.5 cores
- **Memory**: 1 GB
- **URL**: `reframe-api-staging-{sha}.eastus.azurecontainer.io:3000`
- **Auto-cleanup**: After successful production deployment

### Production Environment
- **CPU**: 1 core
- **Memory**: 2 GB
- **URL**: `reframe-api-prod.eastus.azurecontainer.io:3000`
- **High Availability**: Restart policy enabled

## API Endpoints

Once deployed, your API will be available at:

### Health Check
```bash
GET http://{your-domain}:3000/health
```

Response:
```json
{
  "status": "healthy",
  "service": "reframe-api",
  "version": "0.1.0"
}
```

### Web UI
```bash
GET http://{your-domain}:3000/
```

### SWIFT MT103 Conversion
```bash
POST http://{your-domain}:3000/reframe
Content-Type: text/plain

{1:F01BNPAFRPPXXX0000000000}{2:O1031234240101DEUTDEFFXXXX12345678952401011234N}{3:{103:EBA}}{4:
:20:FT21001234567890
:23B:CRED
:32A:240101USD1000,00
:50K:/1234567890
ACME CORPORATION
:52A:BNPAFRPPXXX
:57A:DEUTDEFFXXX
:59:/DE89370400440532013000
MUELLER GMBH
:70:PAYMENT FOR INVOICE 12345
:71A:OUR
-}
```

## Workflow Features

### Intelligent Infrastructure Management
- **Idempotent setup**: Only creates resources that don't exist
- **Automatic detection**: Skips setup if infrastructure is already deployed
- **Force setup option**: Manual trigger to recreate infrastructure if needed

### Security Best Practices
- **Dynamic credential retrieval**: No hardcoded secrets in workflow
- **Masked sensitive data**: Passwords are automatically masked in logs
- **Scoped permissions**: Service principal limited to necessary resources

### Cost Optimization
- **Pay-per-second billing**: Azure Container Instances
- **Automatic cleanup**: Staging environments removed after production deployment
- **Resource right-sizing**: Optimized CPU/memory allocation

## Manual Operations

### Force Infrastructure Rebuild
If you need to recreate the Azure infrastructure:

1. Go to **Actions** tab in your GitHub repository
2. Click **Automated Azure Deployment**
3. Click **Run workflow**
4. Check **Force Azure infrastructure setup**
5. Click **Run workflow**

### Deploy to Staging Only
1. Go to **Actions** tab in your GitHub repository
2. Click **Automated Azure Deployment**
3. Click **Run workflow**
4. Select **staging** environment
5. Click **Run workflow**

## Monitoring and Maintenance

### View Deployment Status
- Check the **Actions** tab in your GitHub repository
- Each deployment shows detailed logs for all stages
- Failed deployments include error details and troubleshooting information

### View Application Logs
```bash
# Production logs
az container logs --resource-group rg-reframe-prod --name reframe-api-prod

# Staging logs (if running)
az container logs --resource-group rg-reframe-prod --name reframe-api-staging
```

### Check Resource Status
```bash
# View all containers
az container list --resource-group rg-reframe-prod --output table

# Check specific container
az container show --resource-group rg-reframe-prod --name reframe-api-prod
```

## Cost Optimization

The deployment is optimized for cost:

1. **Azure Container Instances**: Pay-per-second billing
2. **Basic ACR**: Minimal registry costs
3. **Auto-cleanup**: Staging environments are automatically removed
4. **Resource limits**: Right-sized CPU/memory allocation

Estimated monthly cost: **$15-30** for light usage.

## Troubleshooting

### Common Issues

1. **Service Principal Permissions**: Ensure the service principal has Contributor role at subscription level
2. **Resource Provider Registration**: The workflow automatically handles this, but may take 5-15 minutes on first run
3. **Container Startup**: Check logs if containers fail to start
4. **Network Connectivity**: Verify container group public IP configuration

### Debug Commands

```bash
# Check deployment status
az deployment group list --resource-group rg-reframe-prod

# View container status
az container show --resource-group rg-reframe-prod --name reframe-api-prod

# Check service principal permissions
az role assignment list --assignee {service-principal-id}
```

### Workflow Debugging

1. **Check Actions logs**: Detailed logs available in GitHub Actions tab
2. **Review failed steps**: Each step shows specific error messages
3. **Verify secrets**: Ensure `AZURE_CREDENTIALS` is properly configured
4. **Check Azure permissions**: Service principal needs Contributor access

## Security Considerations

1. **Service Principal**: Scoped to subscription with Contributor role
2. **Registry Access**: Credentials dynamically retrieved during deployment
3. **Container Security**: Non-root user in container
4. **Network**: Public IP with port 3000 only
5. **Secrets Management**: No hardcoded credentials in workflow

## Scaling and Production Readiness

For production workloads, consider:

1. **Azure Container Apps**: Better autoscaling and HTTPS termination
2. **Application Gateway**: SSL termination and WAF
3. **Azure Monitor**: Comprehensive logging and alerting
4. **Key Vault**: Secure secret management
5. **Virtual Network**: Private networking

## Support

For deployment issues:

1. Check GitHub Actions logs for detailed error information
2. Review Azure resource status in the Azure portal
3. Verify service principal permissions
4. Consult Azure documentation for specific error codes 