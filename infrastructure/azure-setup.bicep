@description('Location for all resources')
param location string = resourceGroup().location

@description('Name prefix for all resources')
param namePrefix string = 'reframe'

@description('Environment (prod, staging, dev)')
param environment string = 'prod'

@description('Container image tag')
param imageTag string = 'latest'

@description('ACR admin username')
@secure()
param acrUsername string = ''

@description('ACR admin password')
@secure()
param acrPassword string = ''

@description('Deploy container instance (false for initial setup)')
param deployContainer bool = false

var registryName = '${namePrefix}acr${uniqueString(resourceGroup().id)}'
var containerGroupName = '${namePrefix}-api-${environment}'
var dnsLabel = environment == 'prod' ? '${namePrefix}-api-prod' : '${namePrefix}-api-staging-${substring(uniqueString(deployment().name), 0, 8)}'

// Azure Container Registry
resource containerRegistry 'Microsoft.ContainerRegistry/registries@2023-01-01-preview' = {
  name: registryName
  location: location
  sku: {
    name: 'Basic'
  }
  properties: {
    adminUserEnabled: true
    publicNetworkAccess: 'Enabled'
  }
}

// Container Group for ACI (conditional deployment)
resource containerGroup 'Microsoft.ContainerInstance/containerGroups@2023-05-01' = if (deployContainer && !empty(acrUsername)) {
  name: containerGroupName
  location: location
  properties: {
    containers: [
      {
        name: 'reframe-api'
        properties: {
          image: '${containerRegistry.properties.loginServer}/reframe:${imageTag}'
          ports: [
            {
              port: 3000
              protocol: 'TCP'
            }
          ]
          environmentVariables: [
            {
              name: 'RUST_LOG'
              value: 'info'
            }
            {
              name: 'PORT'
              value: '3000'
            }
          ]
          resources: {
            requests: {
              cpu: environment == 'prod' ? 1 : json('0.5')
              memoryInGB: environment == 'prod' ? 2 : 1
            }
          }
          livenessProbe: {
            httpGet: {
              path: '/health'
              port: 3000
              scheme: 'HTTP'
            }
            initialDelaySeconds: 30
            periodSeconds: 30
            timeoutSeconds: 5
            successThreshold: 1
            failureThreshold: 3
          }
          readinessProbe: {
            httpGet: {
              path: '/health'
              port: 3000
              scheme: 'HTTP'
            }
            initialDelaySeconds: 10
            periodSeconds: 10
            timeoutSeconds: 3
            successThreshold: 1
            failureThreshold: 3
          }
        }
      }
    ]
    imageRegistryCredentials: [
      {
        server: containerRegistry.properties.loginServer
        username: acrUsername
        password: acrPassword
      }
    ]
    restartPolicy: 'Always'
    ipAddress: {
      type: 'Public'
      ports: [
        {
          port: 3000
          protocol: 'TCP'
        }
      ]
      dnsNameLabel: dnsLabel
    }
    osType: 'Linux'
  }
  dependsOn: [
    containerRegistry
  ]
}

// Outputs
output registryName string = containerRegistry.name
output registryLoginServer string = containerRegistry.properties.loginServer
output containerGroupName string = deployContainer ? containerGroup.name : 'not-deployed'
output fqdn string = deployContainer && !empty(acrUsername) ? containerGroup.properties.ipAddress.fqdn : 'not-deployed'
output apiEndpoint string = deployContainer && !empty(acrUsername) ? 'http://${containerGroup.properties.ipAddress.fqdn}:3000' : 'not-deployed' 
