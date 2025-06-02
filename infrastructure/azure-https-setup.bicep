@description('Location for all resources')
param location string = resourceGroup().location

@description('Name prefix for all resources')
param namePrefix string = 'reframe'

@description('Environment (prod, staging, dev)')
param environment string = 'prod'

@description('Existing ACI FQDN (without protocol)')
param aciFqdn string

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
          data: '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'
          password: 'TestCert123!'
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
