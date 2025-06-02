@description('Location for all resources')
param location string = resourceGroup().location

@description('Name prefix for all resources')
param namePrefix string = 'reframe'

@description('Environment (prod, staging, dev)')
param environment string = 'prod'

@description('Existing ACI FQDN (without protocol)')
param aciFqdn string

@description('Custom domain name (optional)')
param customDomain string = ''

var appGatewayName = '${namePrefix}-appgw-${environment}'
var publicIpName = '${namePrefix}-appgw-pip-${environment}'
var vnetName = '${namePrefix}-vnet-${environment}'
var subnetName = 'appgw-subnet'
var nsgName = '${namePrefix}-appgw-nsg-${environment}'

// Network Security Group for Application Gateway
resource networkSecurityGroup 'Microsoft.Network/networkSecurityGroups@2023-04-01' = {
  name: nsgName
  location: location
  properties: {
    securityRules: [
      {
        name: 'AllowHTTPS'
        properties: {
          priority: 1000
          protocol: 'Tcp'
          access: 'Allow'
          direction: 'Inbound'
          sourceAddressPrefix: '*'
          sourcePortRange: '*'
          destinationAddressPrefix: '*'
          destinationPortRange: '443'
        }
      }
      {
        name: 'AllowHTTP'
        properties: {
          priority: 1010
          protocol: 'Tcp'
          access: 'Allow'
          direction: 'Inbound'
          sourceAddressPrefix: '*'
          sourcePortRange: '*'
          destinationAddressPrefix: '*'
          destinationPortRange: '80'
        }
      }
      {
        name: 'AllowAppGatewayV2Inbound'
        properties: {
          priority: 1020
          protocol: 'Tcp'
          access: 'Allow'
          direction: 'Inbound'
          sourceAddressPrefix: 'GatewayManager'
          sourcePortRange: '*'
          destinationAddressPrefix: '*'
          destinationPortRange: '65200-65535'
        }
      }
    ]
  }
}

// Virtual Network for Application Gateway
resource virtualNetwork 'Microsoft.Network/virtualNetworks@2023-04-01' = {
  name: vnetName
  location: location
  properties: {
    addressSpace: {
      addressPrefixes: [
        '10.0.0.0/16'
      ]
    }
    subnets: [
      {
        name: subnetName
        properties: {
          addressPrefix: '10.0.1.0/24'
          networkSecurityGroup: {
            id: networkSecurityGroup.id
          }
        }
      }
    ]
  }
}

// Public IP for Application Gateway
resource publicIP 'Microsoft.Network/publicIPAddresses@2023-04-01' = {
  name: publicIpName
  location: location
  sku: {
    name: 'Standard'
  }
  properties: {
    publicIPAllocationMethod: 'Static'
    dnsSettings: {
      domainNameLabel: '${namePrefix}-api-${environment}-https'
    }
  }
}

// Self-signed certificate for testing (replace with real certificate in production)
resource certificate 'Microsoft.Network/applicationGateways/sslCertificates@2023-04-01' = {
  name: 'default-ssl-cert'
  parent: applicationGateway
  properties: {
    data: 'LS0tLS1CRUdJTiBDRVJUSUZJQ0FURS0tLS0tCk1JSUN1RENDQWFDZ0F3SUJBZ0lKQU9QRzhrQVVQOHJvTUEwR0NTcUdTSWIzRFFFQkN3VUFNQlF4RWpBUUJnTlYKQkFNTUNXeHZZMkZzYUc5emREQWVGdzB5TXpBeE1qZ3hOekE1TURsYUZ3MHpNekF4TWpVeE56QTVNRGxhTUJReApFakFRQmdOVkJBTU1DV3h2WTJGc2FHOXPMREJ0Q0FRRUFvSUJBUUMvRTRKZHc4NDBGWTIxODNQcGtxTm8rLwoKY3l2L2ZBMElrNi9iOUZwQzJJNDdIekVDbUpEa1VhV09FeUV4aVBiQktVV01UdWJOSU1rUnNzYnNxRXZKVQpGUndKU1YrM3NFbG9MYUNvNUUzeWN1dVhVV2FYbUJySlJYWmtRcGVnKzN0MkIvQWtyOEJSZDlrR3U2VHhSCktqU2ZUWnNESmkxNi9LUTZlWGNzNGQrTEtyS3ZXUlVTbkp3N3dUSE5LQzdtK1c0TUhhNUxuUFZrdHAyVzoKZmpKSFNRK2NzQzFJY1J5dWkvcUlWa05kZjJ5WlByNjFEV1B0S2t5QlFXTThkaGZnNDJCQ1VqQWdNQkFBRwojQ1FBd0VBWUhLb1pJemowRUF3SUZBREErTUJ3R0ExVWREZ1FWQkJSTXFMUzBwcUQ3TGNVTStWRm1MemtBClEzZEZaMEF0Q2VCZ056ZytzUjdUVnFrV1ZyRGYKLS0tLS1FTkQgQ0VSVElGSUNBVEUtLS0tLQ=='
    password: 'test123'
  }
}

// Application Gateway
resource applicationGateway 'Microsoft.Network/applicationGateways@2023-04-01' = {
  name: appGatewayName
  location: location
  properties: {
    sku: {
      name: 'Standard_v2'
      tier: 'Standard_v2'
      capacity: 1
    }
    gatewayIPConfigurations: [
      {
        name: 'appGatewayIpConfig'
        properties: {
          subnet: {
            id: resourceId('Microsoft.Network/virtualNetworks/subnets', vnetName, subnetName)
          }
        }
      }
    ]
    frontendIPConfigurations: [
      {
        name: 'appGatewayFrontendIP'
        properties: {
          publicIPAddress: {
            id: publicIP.id
          }
        }
      }
    ]
    frontendPorts: [
      {
        name: 'appGatewayFrontendPort80'
        properties: {
          port: 80
        }
      }
      {
        name: 'appGatewayFrontendPort443'
        properties: {
          port: 443
        }
      }
    ]
    backendAddressPools: [
      {
        name: 'appGatewayBackendPool'
        properties: {
          backendAddresses: [
            {
              fqdn: aciFqdn
            }
          ]
        }
      }
    ]
    backendHttpSettingsCollection: [
      {
        name: 'appGatewayBackendHttpSettings'
        properties: {
          port: 3000
          protocol: 'Http'
          cookieBasedAffinity: 'Disabled'
          pickHostNameFromBackendAddress: false
          requestTimeout: 30
          probe: {
            id: resourceId('Microsoft.Network/applicationGateways/probes', appGatewayName, 'healthProbe')
          }
        }
      }
    ]
    httpListeners: [
      {
        name: 'appGatewayHttpListener'
        properties: {
          frontendIPConfiguration: {
            id: resourceId('Microsoft.Network/applicationGateways/frontendIPConfigurations', appGatewayName, 'appGatewayFrontendIP')
          }
          frontendPort: {
            id: resourceId('Microsoft.Network/applicationGateways/frontendPorts', appGatewayName, 'appGatewayFrontendPort80')
          }
          protocol: 'Http'
        }
      }
      {
        name: 'appGatewayHttpsListener'
        properties: {
          frontendIPConfiguration: {
            id: resourceId('Microsoft.Network/applicationGateways/frontendIPConfigurations', appGatewayName, 'appGatewayFrontendIP')
          }
          frontendPort: {
            id: resourceId('Microsoft.Network/applicationGateways/frontendPorts', appGatewayName, 'appGatewayFrontendPort443')
          }
          protocol: 'Https'
          sslCertificate: {
            id: resourceId('Microsoft.Network/applicationGateways/sslCertificates', appGatewayName, 'default-ssl-cert')
          }
        }
      }
    ]
    requestRoutingRules: [
      {
        name: 'httpToHttpsRedirect'
        properties: {
          ruleType: 'Basic'
          priority: 100
          httpListener: {
            id: resourceId('Microsoft.Network/applicationGateways/httpListeners', appGatewayName, 'appGatewayHttpListener')
          }
          redirectConfiguration: {
            id: resourceId('Microsoft.Network/applicationGateways/redirectConfigurations', appGatewayName, 'httpsRedirect')
          }
        }
      }
      {
        name: 'httpsRule'
        properties: {
          ruleType: 'Basic'
          priority: 200
          httpListener: {
            id: resourceId('Microsoft.Network/applicationGateways/httpListeners', appGatewayName, 'appGatewayHttpsListener')
          }
          backendAddressPool: {
            id: resourceId('Microsoft.Network/applicationGateways/backendAddressPools', appGatewayName, 'appGatewayBackendPool')
          }
          backendHttpSettings: {
            id: resourceId('Microsoft.Network/applicationGateways/backendHttpSettingsCollection', appGatewayName, 'appGatewayBackendHttpSettings')
          }
        }
      }
    ]
    redirectConfigurations: [
      {
        name: 'httpsRedirect'
        properties: {
          redirectType: 'Permanent'
          targetListener: {
            id: resourceId('Microsoft.Network/applicationGateways/httpListeners', appGatewayName, 'appGatewayHttpsListener')
          }
          includePath: true
          includeQueryString: true
        }
      }
    ]
    probes: [
      {
        name: 'healthProbe'
        properties: {
          protocol: 'Http'
          host: aciFqdn
          path: '/health'
          interval: 30
          timeout: 10
          unhealthyThreshold: 3
          pickHostNameFromBackendHttpSettings: false
          minServers: 0
          match: {
            statusCodes: [
              '200-399'
            ]
          }
        }
      }
    ]
    sslCertificates: [
      {
        name: 'default-ssl-cert'
        properties: {
          data: 'LS0tLS1CRUdJTiBDRVJUSUZJQ0FURS0tLS0tCk1JSUN1RENDQWFDZ0F3SUJBZ0lKQU9QRzhrQVVQOHJvTUEwR0NTcUdTSWIzRFFFQkN3VUFNQlF4RWpBUUJnTlYKQkFNTUNXeHZZMkZzYUc5emREQWVGdzB5TXpBeE1qZ3hOekE1TURsYUZ3MHpNekF4TWpVeE56QTVNRGxhTUJReApFakFRQmdOVkJBTU1DV3h2WTJGc2FHOXPMREJ0Q0FRRUFvSUJBUUMvRTRKZHc4NDBGWTIxODNQcGtxTm8rLwoKY3l2L2ZBMElrNi9iOUZwQzJJNDdIekVDbUpEa1VhV09FeUV4aVBiQktVV01UdWJOSU1rUnNzYnNxRXZKVQpGUndKU1YrM3NFbG9MYUNvNUUzeWN1dVhVV2FYbUJySlJYWmtRcGVnKzN0MkIvQWtyOEJSZDlrR3U2VHhSCktqU2ZUWnNESmkxNi9LUTZlWGNzNGQrTEtyS3ZXUlVTbkp3N3dUSE5LQzdtK1c0TUhhNUxuUFZrdHAyVzoKZmpKSFNRK2NzQzFJY1J5dWkvcUlWa05kZjJ5WlByNjFEV1B0S2t5QlFXTThkaGZnNDJCQ1VqQWdNQkFBRwojQ1FBd0VBWUhLb1pJemowRUF3SUZBREErTUJ3R0ExVWREZ1FWQkJSTXFMUzBwcUQ3TGNVTStWRm1MemtBClEzZEZaMEF0Q2VCZ056ZytzUjdUVnFrV1ZyRGYKLS0tLS1FTkQgQ0VSVElGSUNBVEUtLS0tLQ=='
          password: 'test123'
        }
      }
    ]
  }
  dependsOn: [
    virtualNetwork
  ]
}

// Outputs
output applicationGatewayName string = applicationGateway.name
output httpsEndpoint string = 'https://${publicIP.properties.dnsSettings.fqdn}'
output publicIpAddress string = publicIP.properties.ipAddress 
