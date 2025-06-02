# Deployment Guide

This guide explains how to deploy the Reframe SWIFT MT to ISO 20022 converter to Azure using the automated CI/CD pipeline.

## Architecture Overview

The deployment uses **Azure Container Instances (ACI)** as the most cost-effective solution for publicly accessible container services:

- **Azure Container Registry (ACR)**: Stores container images
- **Azure Container Instances (ACI)**: Runs the containerized API
- **GitHub Actions**: Automated CI/CD pipeline
- **Cost**: ~$15-30/month for light usage

## Prerequisites

Before deploying, ensure you have:

1. **Azure CLI** installed and configured
2. **GitHub CLI** (optional, for automatic secret setup)
3. **Azure subscription** with appropriate permissions
4. **GitHub repository** with the source code

## Quick Setup

### 1. Clone and Setup Repository

```bash
git clone <your-repo-url>
cd Reframe
```

### 2. Run Azure Setup Script

```bash
# Make script executable
chmod +x scripts/setup-azure.sh

# Run setup (will prompt for Azure login if needed)
./scripts/setup-azure.sh
```

This script will:
- Create Azure resource group
- Deploy Azure Container Registry
- Create service principal for CI/CD
- Set up GitHub secrets automatically

### 3. Deploy

Push to main branch to trigger deployment:

```bash
git add .
git commit -m "Initial deployment setup"
git push origin main
```

## Manual Setup (Alternative)

If you prefer to set up manually or the script fails:

### 1. Create Azure Resources

```bash
# Login to Azure
az login

# Create resource group
az group create --name rg-reframe-prod --location eastus

# Deploy infrastructure
az deployment group create \
  --resource-group rg-reframe-prod \
  --template-file infrastructure/azure-setup.bicep \
  --parameters environment=prod
```

### 2. Configure GitHub Secrets

Add these secrets to your GitHub repository (Settings → Secrets and variables → Actions):

```bash
# Get service principal credentials
az ad sp create-for-rbac \
  --name sp-reframe-cicd \
  --role Contributor \
  --scopes "/subscriptions/{subscription-id}/resourceGroups/rg-reframe-prod" \
  --sdk-auth
```

Set GitHub secrets:
- `AZURE_CREDENTIALS`: Output from the above command
- `ACR_USERNAME`: From ACR admin credentials
- `ACR_PASSWORD`: From ACR admin credentials

### 3. Get ACR Credentials

```bash
# Get registry name
REGISTRY_NAME=$(az acr list --resource-group rg-reframe-prod --query "[0].name" -o tsv)

# Enable admin user and get credentials
az acr update --name $REGISTRY_NAME --admin-enabled true
az acr credential show --name $REGISTRY_NAME
```

## CI/CD Pipeline Overview

The GitHub Actions workflow (`.github/workflows/deploy-azure.yml`) includes:

### 1. Test Stage
- Rust format checking (`cargo fmt`)
- Linting with Clippy (`cargo clippy`)
- Unit tests (`cargo test`)

### 2. Build & Push Stage
- Multi-architecture Docker build
- Push to Azure Container Registry
- Image tagging with Git SHA

### 3. Staging Deployment
- Deploy to staging ACI instance
- Automated API testing
- Health check validation

### 4. Production Deployment
- Deploy to production ACI instance
- Manual approval required (GitHub environment protection)
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

## Cost Optimization

The deployment is optimized for cost:

1. **Azure Container Instances**: Pay-per-second billing
2. **Basic ACR**: Minimal registry costs
3. **Auto-cleanup**: Staging environments are automatically removed
4. **Resource limits**: Right-sized CPU/memory allocation

Estimated monthly cost: **$15-30** for light usage.

## Monitoring and Maintenance

### View Logs
```bash
# Production logs
az container logs --resource-group rg-reframe-prod --name reframe-api-prod

# Staging logs (if running)
az container logs --resource-group rg-reframe-prod --name reframe-api-staging
```

### Scale Resources
Update `infrastructure/azure-setup.bicep` and redeploy:

```bash
az deployment group create \
  --resource-group rg-reframe-prod \
  --template-file infrastructure/azure-setup.bicep \
  --parameters environment=prod
```

### Manual Deployment
```bash
# Deploy specific image tag
az deployment group create \
  --resource-group rg-reframe-prod \
  --template-file infrastructure/azure-setup.bicep \
  --parameters environment=prod imageTag=sha-abc123
```

## Troubleshooting

### Common Issues

1. **Container startup failure**: Check logs and health endpoint
2. **Image pull errors**: Verify ACR credentials and permissions
3. **Resource limits**: Increase CPU/memory in Bicep template
4. **Network issues**: Check container group public IP configuration

### Debug Commands

```bash
# Check container status
az container show --resource-group rg-reframe-prod --name reframe-api-prod

# View recent deployments
az deployment group list --resource-group rg-reframe-prod

# Check service principal permissions
az role assignment list --assignee {service-principal-id}
```

## Security Considerations

1. **Service Principal**: Scoped to resource group only
2. **Registry Access**: Admin credentials stored as GitHub secrets
3. **Container Security**: Non-root user in container
4. **Network**: Public IP with port 3000 only

## Scaling and Production Readiness

For production workloads, consider:

1. **Azure Container Apps**: Better autoscaling and HTTPS termination
2. **Application Gateway**: SSL termination and WAF
3. **Azure Monitor**: Comprehensive logging and alerting
4. **Key Vault**: Secure secret management
5. **Virtual Network**: Private networking

## Support

For issues with the deployment:

1. Check GitHub Actions logs
2. Review Azure resource status
3. Consult Azure documentation
4. Contact your Azure administrator 