# Reframe v3.0 Deployment Guide - SR2025 Compliant

This guide explains how to deploy Reframe v3.0, the SR2025-compliant enterprise-grade SWIFT MT ↔ ISO 20022 transformation service.

## Overview

Reframe v3.0 is the SR2025-compliant release (SWIFT Standards Release November 2025) distributed as a Docker container for maximum portability and ease of deployment. The service provides full bidirectional transformation capabilities between SWIFT MT and ISO 20022 formats with complete transparency and auditability.

## Architecture

- **Container-based**: Single Docker image with all dependencies including SR2025 workflows
- **Stateless**: No persistent storage required, enabling horizontal scaling
- **REST API**: HTTP-based interface on port 3000 with OpenAPI documentation
- **Hot-reload**: SR2025-compliant workflow configurations can be updated at runtime
- **Performance**: Sub-millisecond transformation with Rust 1.75+ optimizations
- **Compliance**: Full SR2025 validation and Business Application Header v3 support

## Quick Start

### 1. Pull the Docker Image

```bash
# Pull the latest version
docker pull ghcr.io/goplasmatic/reframe:latest

# Or pull a specific version
docker pull ghcr.io/goplasmatic/reframe:v3.0.0
```

### 2. Run the Container

```bash
# Basic run
docker run -p 3000:3000 ghcr.io/goplasmatic/reframe:latest

# Run with custom workflows directory
docker run -p 3000:3000 \
  -v $(pwd)/workflows:/app/workflows \
  ghcr.io/goplasmatic/reframe:latest

# Run with environment variables
docker run -p 3000:3000 \
  -e RUST_LOG=debug \
  -e REFRAME_PORT=8080 \
  ghcr.io/goplasmatic/reframe:latest
```

### 3. Verify Deployment

```bash
# Check health endpoint
curl http://localhost:3000/health

# Expected response:
{
  "status": "healthy",
  "version": "3.0.0",
  "sr2025": "compliant",
  "engines": {
    "forward": {
      "status": "ready",
      "workflows_loaded": 45
    },
    "reverse": {
      "status": "ready",
      "workflows_loaded": 52
    }
  }
}
```

## Building from Source

### Prerequisites

- Docker 20.10 or later
- Git

### Build Steps

```bash
# Clone the repository
git clone https://github.com/GoPlasmatic/Reframe.git
cd Reframe

# Build the Docker image
docker build -t reframe:local .

# Run the locally built image
docker run -p 3000:3000 reframe:local
```

## Configuration Options

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RUST_LOG` | Logging level (debug, info, warn, error) | `info` |
| `REFRAME_PORT` | Port to listen on | `3000` |
| `REFRAME_SR2025` | Enable SR2025 compliance checks | `true` |
| `REFRAME_BAH_VERSION` | Business Application Header version | `v3` |

### Volume Mounts

| Path | Description |
|------|-------------|
| `/app/workflows` | SR2025-compliant workflow configuration files |
| `/app/scenarios` | SR2025 test message generation scenarios |
| `/app/logs` | Application logs (optional) |

## Deployment Scenarios

### Local Development

```bash
# Run with live workflow editing
docker run -p 3000:3000 \
  -v $(pwd)/workflows:/app/workflows \
  -v $(pwd)/scenarios:/app/scenarios \
  -e RUST_LOG=debug \
  reframe:local
```

### Docker Compose

Create a `docker-compose.yml`:

```yaml
version: '3.8'

services:
  reframe:
    image: ghcr.io/goplasmatic/reframe:latest
    ports:
      - "3000:3000"
    environment:
      - RUST_LOG=info
    volumes:
      - ./workflows:/app/workflows
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 3s
      retries: 3
```

Run with:
```bash
docker-compose up -d
```

### Kubernetes

Create a deployment manifest:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: reframe
spec:
  replicas: 3
  selector:
    matchLabels:
      app: reframe
  template:
    metadata:
      labels:
        app: reframe
    spec:
      containers:
      - name: reframe
        image: ghcr.io/goplasmatic/reframe:latest
        ports:
        - containerPort: 3000
        env:
        - name: RUST_LOG
          value: "info"
        livenessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 5
          periodSeconds: 30
        readinessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 5
          periodSeconds: 10
---
apiVersion: v1
kind: Service
metadata:
  name: reframe
spec:
  selector:
    app: reframe
  ports:
    - protocol: TCP
      port: 80
      targetPort: 3000
  type: LoadBalancer
```

Deploy with:
```bash
kubectl apply -f reframe-deployment.yaml
```

## Production Considerations

### Resource Requirements

**Minimum:**
- CPU: 0.5 vCPU
- Memory: 512MB
- Storage: 100MB

**Recommended:**
- CPU: 2 vCPU
- Memory: 2GB
- Storage: 500MB

### Scaling

Reframe is stateless and can be horizontally scaled:

```bash
# Docker Swarm
docker service create --replicas 3 --name reframe -p 3000:3000 ghcr.io/goplasmatic/reframe:latest

# Kubernetes
kubectl scale deployment reframe --replicas=5
```

### Load Balancing

Place Reframe behind a load balancer for high availability:

- **nginx**: Use as reverse proxy with round-robin
- **HAProxy**: Advanced load balancing with health checks
- **Container orchestration platforms**: Platform-specific load balancers

### Monitoring

Monitor the following endpoints:

- `/health` - Service health and engine status
- Container metrics - CPU, memory, network I/O
- Application logs - Via `RUST_LOG` environment variable

### Security

1. **Network Security**
   - Run behind a firewall/security group
   - Use HTTPS termination at load balancer
   - Restrict access to trusted IPs

2. **Container Security**
   - Run as non-root user (default in image)
   - Use read-only root filesystem where possible
   - Scan images for vulnerabilities

3. **API Security**
   - Implement rate limiting at proxy level
   - Add authentication if needed (OAuth2, API keys)
   - Monitor for suspicious patterns

## Workflow Management

### Hot Reload Workflows

Update workflows without restarting:

```bash
# Modify workflow files
vi workflows/forward/MT103/document-mapping.json

# Reload workflows via API
curl -X POST http://localhost:3000/admin/reload-workflows

# Verify reload
{
  "success": true,
  "message": "Workflows reloaded successfully in 44ms"
}
```

### Custom Workflows

Mount your custom workflows:

```bash
docker run -p 3000:3000 \
  -v /path/to/custom/workflows:/app/workflows \
  ghcr.io/goplasmatic/reframe:latest
```

## Troubleshooting

### Check Container Logs

```bash
# Docker
docker logs [container-id]

# Docker Compose
docker-compose logs reframe

# Kubernetes
kubectl logs deployment/reframe
```

### Common Issues

1. **Port Already in Use**
   ```bash
   # Use a different port
   docker run -p 8080:3000 ghcr.io/goplasmatic/reframe:latest
   ```

2. **Workflow Loading Errors**
   ```bash
   # Check workflow syntax
   docker run --rm -v $(pwd)/workflows:/app/workflows \
     ghcr.io/goplasmatic/reframe:latest \
     ./reframe --validate-workflows
   ```

3. **Memory Issues**
   ```bash
   # Increase container memory
   docker run -p 3000:3000 -m 4g ghcr.io/goplasmatic/reframe:latest
   ```

### Debug Mode

Run with debug logging:

```bash
docker run -p 3000:3000 \
  -e RUST_LOG=debug \
  ghcr.io/goplasmatic/reframe:latest
```

## API Usage Examples

### Transform MT to ISO 20022

```bash
curl -X POST http://localhost:3000/transform/mt-to-mx \
  -H "Content-Type: application/json" \
  -d '{
    "message": "{1:F01BNPAFRPPXXX0000000000}..."
  }'
```

### Transform ISO 20022 to MT

```bash
curl -X POST http://localhost:3000/transform/mx-to-mt \
  -H "Content-Type: application/json" \
  -d '{
    "message": "<?xml version=\"1.0\"?>..."
  }'
```

### Generate Sample Messages

```bash
curl -X POST http://localhost:3000/generate/sample \
  -H "Content-Type: application/json" \
  -d '{
    "message_type": "MT103",
    "config": {
      "scenario": "standard"
    }
  }'
```

## SR2025 Compliance Notes

### Key SR2025 Features
- Business Application Header with enhanced party identification
- Mandatory UETR (Unique End-to-end Transaction Reference)
- LEI (Legal Entity Identifier) support
- Structured remittance information

### Validation
Reframe v3.0 automatically validates:
- SR2025 mandatory fields
- Service level codes (G001, G002, G003, G004)
- Cross-field business rules

## Support

For issues and questions:

1. Check the [GitHub Issues](https://github.com/GoPlasmatic/Reframe/issues)
2. Review application logs with `RUST_LOG=debug`
3. Verify SR2025 workflow configurations
4. Test with SR2025-compliant sample messages
5. Consult the [SR2025 Standards Guide](https://www.swift.com/standards/release-guide/sr2025)

## License

Reframe is distributed under the Apache 2.0 License. See LICENSE file for details.