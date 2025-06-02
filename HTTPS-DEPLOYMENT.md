# HTTPS Deployment Guide

This guide explains how to deploy HTTPS endpoints for the Reframe API using Azure Application Gateway.

## Overview

The Reframe API now supports automatic HTTPS deployment through two methods:

1. **Automated CI/CD**: HTTPS is automatically deployed as part of the GitHub Actions workflow
2. **Manual Deployment**: Use the standalone script for immediate HTTPS setup

## Automated HTTPS Deployment (Recommended)

### How It Works

The GitHub Actions workflow (`.github/workflows/deploy-azure.yml`) now includes an HTTPS deployment job that:

1. **Deploys ACI**: First deploys the HTTP API to Azure Container Instances
2. **Gets ACI FQDN**: Captures the HTTP endpoint from ACI deployment
3. **Deploys HTTPS Infrastructure**: Creates Application Gateway with SSL termination
4. **Tests Endpoints**: Validates both HTTP and HTTPS connectivity
5. **Provides URLs**: Outputs both HTTP and HTTPS endpoints

### Triggering HTTPS Deployment

#### Automatic (Default)
```bash
# Push to main branch - HTTPS is deployed automatically
git push origin main
```

#### Manual Workflow Dispatch
```bash
# Use GitHub UI or CLI to trigger with options
gh workflow run deploy-azure.yml --field environment=production --field deploy_https=true
```

### HTTPS Infrastructure Components

The automated deployment creates:

- **Virtual Network**: Dedicated VNet with Application Gateway subnet
- **Network Security Group**: Security rules for HTTP/HTTPS traffic
- **Public IP**: Static IP with DNS label for HTTPS endpoint
- **Application Gateway**: 
  - SSL termination with self-signed certificate
  - HTTP to HTTPS redirect
  - Health probes for backend monitoring
  - Load balancing to ACI backend

### Endpoints After Deployment

- **HTTPS API**: `https://reframe-api-prod-https.eastus.cloudapp.azure.com/reframe`
- **HTTPS Health**: `https://reframe-api-prod-https.eastus.cloudapp.azure.com/health`
- **HTTP API** (fallback): `http://reframe-api-prod.eastus.azurecontainer.io:3000/reframe`

## Manual HTTPS Deployment

For immediate HTTPS setup without waiting for CI/CD:

### Prerequisites

1. Azure CLI installed and logged in (`az login`)
2. Existing ACI deployment
3. Appropriate Azure permissions

### Deploy HTTPS

```bash
# Make script executable
chmod +x scripts/deploy-https-manual.sh

# Run manual deployment
./scripts/deploy-https-manual.sh
```

### What the Script Does

1. **Checks Prerequisites**: Verifies Azure CLI login and ACI deployment
2. **Gets ACI Information**: Retrieves the current HTTP endpoint
3. **Deploys Infrastructure**: Creates Application Gateway and related resources
4. **Validates Deployment**: Provides testing instructions
5. **Outputs URLs**: Shows HTTPS endpoint and testing commands

### Example Output

```
🔒 Manual HTTPS Deployment for Reframe API
==============================================
✅ Found ACI at: reframe-api-prod.eastus.azurecontainer.io
🚀 Deploying HTTPS infrastructure...
✅ HTTPS infrastructure deployed successfully!

🎉 HTTPS Setup Complete!
==============================================
HTTPS Endpoint: https://reframe-api-prod-https.eastus.cloudapp.azure.com/reframe
Health Check: https://reframe-api-prod-https.eastus.cloudapp.azure.com/health
Public IP: 20.22.226.150
```

## Testing HTTPS Endpoints

### Health Check

```bash
# Test health endpoint (self-signed cert)
curl -k https://reframe-api-prod-https.eastus.cloudapp.azure.com/health
```

Expected response:
```json
{
  "status": "healthy",
  "service": "reframe-api",
  "version": "0.1.0"
}
```

### API Conversion Test

```bash
curl -k -X POST "https://reframe-api-prod-https.eastus.cloudapp.azure.com/reframe" \
  -H "Content-Type: text/plain" \
  -d "{1:F01BNPAFRPPXXX0000000000}{2:O1031234240101DEUTDEFFXXXX12345678952401011234N}{3:{103:EBA}}{4:
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
-}"
```

## Web UI Integration

The web UI at `https://GoPlasmatic.github.io/Reframe` automatically:

1. **Tries HTTPS First**: Always attempts HTTPS endpoint before HTTP fallback
2. **Shows Connection Status**: Displays whether connected via HTTPS or HTTP
3. **Handles Fallback**: Gracefully falls back to HTTP if HTTPS isn't available
4. **Updates UI**: Shows appropriate status indicators and messages

## SSL Certificate Management

### Current Setup (Self-Signed)

The deployment uses a self-signed certificate for testing and immediate deployment. This:

- ✅ **Enables HTTPS**: Provides encrypted connections
- ✅ **Works Immediately**: No external certificate required
- ⚠️ **Browser Warnings**: Shows "Not Secure" in browsers
- ⚠️ **Certificate Errors**: Requires `-k` flag in curl

### Production Certificate Setup

For production use, replace the self-signed certificate:

1. **Get SSL Certificate**: Obtain from Let's Encrypt, DigiCert, etc.
2. **Convert to Base64**: Encode certificate and private key
3. **Update Bicep Template**: Replace certificate data in `infrastructure/azure-https-setup.bicep`
4. **Redeploy**: Run the deployment again

Example for updating certificate:
```bicep
sslCertificates: [
  {
    name: 'production-ssl-cert'
    properties: {
      data: 'BASE64_ENCODED_CERTIFICATE_DATA'
      password: 'CERTIFICATE_PASSWORD'
    }
  }
]
```

## Monitoring and Troubleshooting

### View Application Gateway Status

```bash
# Check Application Gateway health
az network application-gateway show \
  --resource-group rg-reframe-prod \
  --name reframe-appgw-prod

# Check backend health
az network application-gateway show-backend-health \
  --resource-group rg-reframe-prod \
  --name reframe-appgw-prod
```

### Common Issues

1. **502 Bad Gateway**: Backend ACI may be down or unhealthy
2. **Timeout Errors**: Application Gateway may still be starting (wait 5-10 minutes)
3. **DNS Issues**: Public IP DNS may need propagation time
4. **Certificate Warnings**: Expected with self-signed certificates

### Logs and Debugging

```bash
# View ACI logs
az container logs \
  --resource-group rg-reframe-prod \
  --name reframe-api-prod

# Check recent deployments
az deployment group list \
  --resource-group rg-reframe-prod \
  --query "[?contains(name, 'https')].{Name:name,Status:properties.provisioningState,Timestamp:properties.timestamp}"
```

## Cost Considerations

### Additional Costs for HTTPS

- **Application Gateway**: ~$18-25/month for Standard_v2 SKU
- **Public IP**: ~$3-4/month for static IP
- **Virtual Network**: No additional cost
- **NSG**: No additional cost

Total additional cost: **~$21-29/month** for HTTPS infrastructure.

### Cost Optimization

- Application Gateway scales automatically based on traffic
- Consider using Azure Front Door for global distribution
- Monitor usage and scale down if needed

## Security Considerations

1. **SSL Termination**: Traffic encrypted from client to Application Gateway
2. **Backend Traffic**: HTTP between Application Gateway and ACI (within Azure)
3. **Network Security**: NSG rules restrict access to required ports only
4. **Certificate Management**: Self-signed certificates for testing only

## Next Steps

1. **Deploy HTTPS**: Use automated or manual deployment
2. **Test Endpoints**: Verify both HTTP and HTTPS work
3. **Update DNS**: Point your domain to the Application Gateway IP (optional)
4. **Replace Certificate**: Use production SSL certificate for public use
5. **Monitor**: Set up Azure Monitor alerts for Application Gateway health 