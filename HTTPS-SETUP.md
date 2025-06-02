# HTTPS Setup for Reframe API

This guide explains how to enable HTTPS for your Reframe API to solve Mixed Content issues when accessing the API from GitHub Pages (HTTPS) sites.

## Problem

When your web UI is hosted on GitHub Pages (HTTPS), browsers block requests to HTTP APIs due to Mixed Content security policies. This requires the API to also be served over HTTPS.

## Solution Overview

We use **Azure Application Gateway** to provide SSL termination in front of your existing Azure Container Instance. This approach:

- ✅ Enables HTTPS without modifying your container
- ✅ Provides automatic HTTP to HTTPS redirect
- ✅ Includes health checks and load balancing
- ✅ Uses self-signed certificate for testing (easily replaceable)
- ✅ Maintains your existing ACI deployment

## Architecture

```
GitHub Pages (HTTPS) → Azure Application Gateway (HTTPS) → Azure Container Instance (HTTP)
```

## Quick Setup

### Prerequisites

- Azure CLI installed and logged in
- Existing Reframe API deployed to Azure Container Instance
- Resource group: `rg-reframe-prod`

### 1. Deploy HTTPS Infrastructure

Run the automated setup script:

```bash
./scripts/setup-https.sh
```

This script will:
1. ✅ Detect your existing ACI deployment
2. ✅ Create Virtual Network with Application Gateway subnet
3. ✅ Deploy Network Security Group with HTTPS rules
4. ✅ Create Public IP with DNS label
5. ✅ Deploy Application Gateway with SSL termination
6. ✅ Configure HTTP to HTTPS redirect
7. ✅ Set up health probes to your ACI

### 2. Test the HTTPS Endpoint

After deployment (wait 5-10 minutes for full readiness):

```bash
# Test health endpoint
curl -k https://reframe-api-prod-https.eastus.cloudapp.azure.com/health

# Test conversion endpoint
curl -k -X POST https://reframe-api-prod-https.eastus.cloudapp.azure.com/reframe \
  -H "Content-Type: text/plain" \
  -d "{1:F01BNPAFRPPXXX0000000000}{2:O1031234240101DEUTDEFFXXXX12345678952401011234N}{3:{103:EBA}}{4:
:20:FT21001234567890
:23B:CRED
:32A:240101USD1000,00
:50K:/1234567890
ACME CORPORATION
-}"
```

### 3. Update Web UI

The web UI has been updated to automatically try HTTPS first, then fall back to HTTP for development. No manual changes needed!

### 4. Deploy Updated Web UI

```bash
git add .
git commit -m "Add HTTPS support with Azure Application Gateway"
git push origin main
```

## Infrastructure Details

### Components Created

1. **Virtual Network**: `reframe-vnet-prod`
   - Address space: 10.0.0.0/16
   - Application Gateway subnet: 10.0.1.0/24

2. **Network Security Group**: `reframe-appgw-nsg-prod`
   - Allow HTTPS (443)
   - Allow HTTP (80) for redirect
   - Allow Azure Gateway Manager traffic

3. **Public IP**: `reframe-appgw-pip-prod`
   - Standard SKU for Application Gateway
   - DNS label: `reframe-api-prod-https`

4. **Application Gateway**: `reframe-appgw-prod`
   - Standard_v2 SKU
   - SSL termination with self-signed certificate
   - HTTP to HTTPS redirect
   - Health probes to ACI backend

### Endpoints

- **HTTPS API**: `https://reframe-api-prod-https.eastus.cloudapp.azure.com/reframe`
- **Health Check**: `https://reframe-api-prod-https.eastus.cloudapp.azure.com/health`
- **HTTP Redirect**: `http://reframe-api-prod-https.eastus.cloudapp.azure.com` → HTTPS

## Manual Setup (Alternative)

If you prefer manual setup or need customization:

### 1. Deploy Bicep Template

```bash
az deployment group create \
  --resource-group rg-reframe-prod \
  --template-file infrastructure/azure-https-setup.bicep \
  --parameters \
    aciFqdn="reframe-api-prod.eastus.azurecontainer.io" \
    location="eastus" \
    namePrefix="reframe" \
    environment="prod"
```

### 2. Get Outputs

```bash
az deployment group show \
  --resource-group rg-reframe-prod \
  --name <deployment-name> \
  --query "properties.outputs"
```

## Production SSL Certificate

For production use, replace the self-signed certificate:

### Option 1: Use Azure Key Vault

1. Upload your SSL certificate to Azure Key Vault
2. Update the Application Gateway to reference the Key Vault certificate:

```bash
az network application-gateway ssl-cert update \
  --resource-group rg-reframe-prod \
  --gateway-name reframe-appgw-prod \
  --name default-ssl-cert \
  --key-vault-secret-id "https://your-keyvault.vault.azure.net/secrets/ssl-cert"
```

### Option 2: Use Let's Encrypt

Consider using Azure's built-in integration with Let's Encrypt for automatic certificate management.

## Cost Estimation

Azure Application Gateway Standard_v2:
- **Base cost**: ~$22/month (730 hours)
- **Data processing**: ~$0.008/GB
- **Capacity units**: Auto-scaling based on load

For light usage (< 1GB/month): **~$22-25/month**

## Troubleshooting

### Common Issues

1. **502 Bad Gateway**: Wait 5-10 minutes after deployment for full readiness
2. **Certificate warnings**: Expected with self-signed cert, use `-k` flag with curl
3. **Health probe failures**: Check ACI is running and accessible on port 3000

### Verification Commands

```bash
# Check Application Gateway status
az network application-gateway show \
  --resource-group rg-reframe-prod \
  --name reframe-appgw-prod \
  --query "operationalState"

# Check backend health
az network application-gateway show-backend-health \
  --resource-group rg-reframe-prod \
  --name reframe-appgw-prod

# Check ACI status
az container show \
  --resource-group rg-reframe-prod \
  --name reframe-api-prod \
  --query "instanceView.state"
```

### Logs

```bash
# Application Gateway logs
az monitor activity-log list \
  --resource-group rg-reframe-prod \
  --max-events 50

# ACI logs
az container logs \
  --resource-group rg-reframe-prod \
  --name reframe-api-prod
```

## Cleanup

To remove HTTPS infrastructure:

```bash
az network application-gateway delete \
  --resource-group rg-reframe-prod \
  --name reframe-appgw-prod

az network vnet delete \
  --resource-group rg-reframe-prod \
  --name reframe-vnet-prod

az network public-ip delete \
  --resource-group rg-reframe-prod \
  --name reframe-appgw-pip-prod
```

## Next Steps

1. 🔒 **Setup complete**: Your API now supports HTTPS
2. 🌐 **Test web UI**: Visit `https://goplasmatic.github.io/Reframe`
3. 📝 **Update documentation**: Update any API documentation with HTTPS endpoints
4. 🔑 **Production certificate**: Replace self-signed cert for production use
5. 📊 **Monitor**: Set up monitoring and alerting for the new infrastructure 