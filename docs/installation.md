# 🔧 Installation Guide

Get Reframe v3.1.1 up and running quickly with these comprehensive installation instructions.

## Table of Contents

- [Quick Start](#quick-start)
- [Docker Installation](#docker-installation)
- [From Source](#from-source)
- [Configuration](#configuration)
- [Production Deployment](#production-deployment)
- [Troubleshooting](#troubleshooting)

---

## Quick Start

Ready to dive in? Here's the quickest way to get Reframe working:

### 🐳 Docker (Recommended)

Docker provides the easiest and fastest setup:

```bash
# Pull and run the latest Reframe image
docker run -p 3000:3000 plasmatic/reframe:latest

# Open Reframe in your browser
open http://localhost:3000/swagger-ui
```

### 🔍 Verify Installation

To ensure Reframe is working correctly, run these commands:

```bash
# Check health status
curl http://localhost:3000/health

# Test unified transformation endpoint (auto-detects direction)
curl -X POST http://localhost:3000/api/transform \
  -H "Content-Type: application/json" \
  -d '{
    "message": "{1:F01BNPAFRPPXXX0000000000}{2:O1031234240101DEUTDEFFXXXX12345678952401011234N}{4::20:TEST123:32A:240101USD100,00:50K:SENDER:59:RECEIVER:71A:OUR-}",
    "validation": true
  }'
```

---

## Docker Installation

### Prerequisites

Before installing, ensure your system meets these requirements:

- Docker 20.10+ or Docker Desktop
- 2GB available RAM
- 1GB available disk space

### Standard Installation

For most users, this is the recommended installation method:

```bash
# Create a dedicated directory for Reframe
mkdir ~/reframe && cd ~/reframe

# Download and run the latest Reframe container
docker run -d \
  --name reframe \
  -p 3000:3000 \
  --restart unless-stopped \
  plasmatic/reframe:latest
```

### Development Installation

If you're developing custom workflows or using external packages, use this setup:

```bash
# Run Reframe with your local package and configuration
docker run -d \
  --name reframe-dev \
  -p 3000:3000 \
  -v $(pwd)/reframe-package-swift-cbpr:/packages/swift-cbpr:ro \
  -v $(pwd)/reframe.config.json:/app/reframe.config.json:ro \
  -v $(pwd)/logs:/var/log/reframe \
  -e REFRAME_PACKAGE_PATH=/packages/swift-cbpr \
  plasmatic/reframe:latest
```

### Docker Compose

For easier management of your Docker containers, use Docker Compose. First, create a `docker-compose.yml` file:

```yaml
version: '3.8'

services:
  reframe:
    image: plasmatic/reframe:latest
    container_name: reframe
    ports:
      - "3000:3000"
    volumes:
      # Mount workflow package
      - ./reframe-package-swift-cbpr:/packages/swift-cbpr:ro
      # Mount configuration file
      - ./reframe.config.json:/app/reframe.config.json:ro
      # Mount logs directory
      - ./logs:/var/log/reframe
    environment:
      - REFRAME_PACKAGE_PATH=/packages/swift-cbpr
      - RUST_LOG=info
      - TOKIO_WORKER_THREADS=auto
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
```

Then, use these commands to manage your Reframe services:

```bash
# Start all services defined in docker-compose.yml
docker-compose up -d

# View real-time logs for the Reframe service
docker-compose logs -f reframe

# Stop and remove all services
docker-compose down

# Reload workflows without restart
curl -X POST http://localhost:3000/admin/reload-workflows
```

---

## From Source

### Prerequisites

For those who want to build Reframe from source, ensure you have the following installed:

- **Rust**: 1.75 or later
- **Cargo**: Latest version
- **Git**: For cloning the repository
- **Workflow Package**: reframe-package-swift-cbpr (separate repository)

### Installation Steps

Follow these steps to install Reframe from source:

1. **Install Rust**:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   ```

2. **Clone Repository**:
   ```bash
   git clone https://github.com/GoPlasmatic/Reframe.git
   cd Reframe
   ```

3. **Clone Workflow Package**:
   ```bash
   # Clone the workflow package to parent directory
   cd ..
   git clone https://github.com/GoPlasmatic/reframe-package-swift-cbpr.git
   cd Reframe
   ```

4. **Build Application**:
   Choose between a debug or release build:
   ```bash
   # Debug build (faster compilation, for development)
   cargo build

   # Release build (optimized for performance)
   cargo build --release
   ```

5. **Run Application**:
   Start Reframe in your preferred mode:
   ```bash
   # Debug mode with info logging
   RUST_LOG=info cargo run

   # Release mode
   ./target/release/reframe

   # With custom configuration
   RUST_LOG=debug cargo run
   ```

### Directory Structure

After installation, your directory structure should look like:

```
parent-directory/
├── Reframe/                       # Main application
│   ├── src/                       # Source code
│   ├── Cargo.toml                 # Rust dependencies
│   ├── reframe.config.json        # Optional: Runtime configuration
│   └── docs/                      # Documentation
└── reframe-package-swift-cbpr/    # Workflow package (external)
    ├── reframe-package.json       # Package configuration
    ├── transform/                 # Transformation workflows
    ├── generate/                  # Generation workflows
    ├── validate/                  # Validation workflows
    └── scenarios/                 # Test scenarios
```

---

## Configuration

### Creating reframe.config.json

Create an optional configuration file in the Reframe root directory:

```bash
cd Reframe
cat > reframe.config.json << 'EOF'
{
  "packages": [
    {
      "path": "../reframe-package-swift-cbpr",
      "enabled": true
    }
  ],
  "server": {
    "host": "0.0.0.0",
    "port": 3000,
    "runtime": {
      "tokio_worker_threads": "auto",
      "thread_name_prefix": "reframe-worker"
    }
  },
  "logging": {
    "format": "compact",
    "level": "info",
    "show_target": false,
    "file_output": {
      "enabled": false
    }
  },
  "api_docs": {
    "enabled": true,
    "title": "Reframe API",
    "server_url": "http://localhost:3000"
  },
  "defaults": {
    "package_path": "../reframe-package-swift-cbpr"
  }
}
EOF
```

**Note**: If no `reframe.config.json` is present, Reframe uses built-in defaults and looks for the package at `../reframe-package-swift-cbpr`.

### Environment Variables

Configure these environment variables for your setup:

```bash
# Path to workflow package (overrides config file)
export REFRAME_PACKAGE_PATH=../reframe-package-swift-cbpr

# Logging level (error, warn, info, debug, trace)
export RUST_LOG=info

# Number of Tokio worker threads (auto = CPU count)
export TOKIO_WORKER_THREADS=auto

# OpenAPI server URL override
export API_SERVER_URL=http://localhost:3000
```

### Logging Configuration

Reframe offers flexible logging configurations:

```bash
# Available log levels
export RUST_LOG=error    # Errors only
export RUST_LOG=warn     # Warnings and errors
export RUST_LOG=info     # General information (recommended)
export RUST_LOG=debug    # Detailed debugging
export RUST_LOG=trace    # Extremely verbose

# Module-specific logging
export RUST_LOG=info,reframe::handlers=debug,dataflow_rs=trace
```

---

## Production Deployment

### System Requirements

For optimal performance in a production environment, ensure your system meets these requirements:

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| **CPU** | 2 cores | 4+ cores |
| **RAM** | 1GB | 4GB+ |
| **Storage** | 2GB | 10GB+ |
| **Network** | 100Mbps | 1Gbps+ |

### Production Configuration

Create a production-ready configuration:

**reframe.config.json**:
```json
{
  "packages": [
    {
      "path": "/opt/reframe/packages/swift-cbpr-v2.0",
      "enabled": true
    }
  ],
  "server": {
    "host": "0.0.0.0",
    "port": 3000,
    "runtime": {
      "tokio_worker_threads": "16"
    }
  },
  "logging": {
    "format": "json",
    "level": "info",
    "show_target": false,
    "file_output": {
      "enabled": true,
      "directory": "/var/log/reframe",
      "file_prefix": "reframe-prod",
      "rotation": "daily"
    }
  },
  "api_docs": {
    "enabled": true,
    "title": "Production Transformation API",
    "server_url": "https://api.production.com"
  }
}
```

### Systemd Service (Linux)

Create a systemd service for automatic startup:

```bash
sudo tee /etc/systemd/system/reframe.service << 'EOF'
[Unit]
Description=Reframe SWIFT MT ↔ ISO 20022 Transformation Service
After=network.target

[Service]
Type=simple
User=reframe
Group=reframe
WorkingDirectory=/opt/reframe
Environment="RUST_LOG=info"
Environment="REFRAME_PACKAGE_PATH=/opt/reframe/packages/swift-cbpr"
ExecStart=/opt/reframe/reframe
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

# Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable reframe
sudo systemctl start reframe

# Check status
sudo systemctl status reframe
```

### Kubernetes Deployment

Example Kubernetes deployment configuration:

**deployment.yaml**:
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: reframe
  namespace: financial-services
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
        image: plasmatic/reframe:3.1.1
        ports:
        - containerPort: 3000
          name: http
        env:
        - name: RUST_LOG
          value: "info"
        - name: TOKIO_WORKER_THREADS
          value: "8"
        - name: REFRAME_PACKAGE_PATH
          value: "/packages/swift-cbpr"
        volumeMounts:
        - name: package-volume
          mountPath: /packages
          readOnly: true
        - name: config-volume
          mountPath: /app/config
          readOnly: true
        resources:
          requests:
            memory: "1Gi"
            cpu: "500m"
          limits:
            memory: "4Gi"
            cpu: "2000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 10
          periodSeconds: 5
      volumes:
      - name: package-volume
        configMap:
          name: reframe-package
      - name: config-volume
        configMap:
          name: reframe-config
---
apiVersion: v1
kind: Service
metadata:
  name: reframe
  namespace: financial-services
spec:
  selector:
    app: reframe
  ports:
  - protocol: TCP
    port: 80
    targetPort: 3000
  type: LoadBalancer
```

### Performance Tuning

Optimize for your workload:

```bash
# High-throughput configuration
export TOKIO_WORKER_THREADS=16
export RUST_LOG=warn  # Reduce logging overhead
./target/release/reframe

# Low-latency configuration
export TOKIO_WORKER_THREADS=8
export RUST_LOG=info
./target/release/reframe

# Resource-constrained environment
export TOKIO_WORKER_THREADS=2
export RUST_LOG=error
./target/release/reframe
```

---

## Troubleshooting

### Common Issues

#### 1. Package Not Found

**Error**:
```
Error: Package configuration not found: ../reframe-package-swift-cbpr/reframe-package.json
```

**Solution**:
```bash
# Clone the workflow package
cd ..
git clone https://github.com/GoPlasmatic/reframe-package-swift-cbpr.git
cd Reframe

# Or set custom path
export REFRAME_PACKAGE_PATH=/path/to/package
cargo run
```

#### 2. Port Already in Use

**Error**:
```
Error: Address already in use (os error 48)
```

**Solution**:
```bash
# Check what's using port 3000
lsof -i :3000

# Kill the process or use a different port
export PORT=8080
cargo run
```

Or in `reframe.config.json`:
```json
{
  "server": {
    "port": 8080
  }
}
```

#### 3. Permission Denied (Linux)

**Error**:
```
Error: Permission denied (os error 13)
```

**Solution**:
```bash
# Option 1: Use port > 1024
export PORT=3000

# Option 2: Run with sudo (not recommended)
sudo -E cargo run

# Option 3: Grant capabilities (recommended)
sudo setcap 'cap_net_bind_service=+ep' ./target/release/reframe
```

#### 4. Workflow Reload Fails

**Error**:
```
Failed to reload workflows: Invalid workflow syntax
```

**Solution**:
```bash
# Check workflow JSON syntax
cd ../reframe-package-swift-cbpr
find . -name "*.json" -exec jsonlint {} \;

# View detailed error
RUST_LOG=debug cargo run
```

### Getting Help

#### Check Logs

```bash
# View all logs
RUST_LOG=debug cargo run 2>&1 | tee reframe.log

# View specific module
RUST_LOG=reframe::handlers=trace cargo run

# Production logs (if file output enabled)
tail -f /var/log/reframe/reframe-prod.log
```

#### Health Check

```bash
# Basic health check
curl http://localhost:3000/health | jq

# Expected response
{
  "status": "healthy",
  "timestamp": "2025-10-14T10:00:00Z",
  "engines": {
    "transform": "ready",
    "generation": "ready",
    "validation": "ready"
  },
  "capabilities": [
    "bidirectional_transformation",
    "sample_generation",
    "message_validation",
    "workflow_reload"
  ]
}
```

#### Test API Endpoints

```bash
# Test transformation
curl -X POST http://localhost:3000/api/transform \
  -H "Content-Type: application/json" \
  -d '{"message": "..."}' | jq

# Test generation
curl -X POST http://localhost:3000/api/generate \
  -H "Content-Type: application/json" \
  -d '{"message_type": "MT103", "scenario": "standard"}' | jq

# Test validation
curl -X POST http://localhost:3000/api/validate \
  -H "Content-Type: application/json" \
  -d '{"message": "..."}' | jq
```

### Documentation

- [Configuration Guide](configuration.md) - Detailed configuration options
- [Architecture Guide](architecture.md) - Technical architecture details
- [Message Formats](message-formats.md) - Supported message types

### Community Support

- [GitHub Issues](https://github.com/GoPlasmatic/Reframe/issues)
- [GitHub Discussions](https://github.com/GoPlasmatic/Reframe/discussions)

---

## Next Steps

After installation, explore these resources to get the most out of Reframe:

1. **[Configuration Guide](configuration.md)** - Learn about all configuration options
2. **[Architecture Guide](architecture.md)** - Understand the technical architecture
3. **[Message Formats](message-formats.md)** - Complete list of supported message types
4. **[Swagger UI](http://localhost:3000/swagger-ui)** - Interactive API documentation

---

## Quick Reference

### Common Commands

```bash
# Development
cargo run                           # Run in debug mode
RUST_LOG=debug cargo run           # Run with debug logging

# Production
cargo build --release              # Build optimized binary
./target/release/reframe           # Run release binary

# Docker
docker run -p 3000:3000 plasmatic/reframe:latest
docker-compose up -d

# Administration
curl -X POST http://localhost:3000/admin/reload-workflows
curl http://localhost:3000/health
curl http://localhost:3000/api/packages
```

### Environment Variables Quick Reference

| Variable | Purpose | Example |
|----------|---------|---------|
| `REFRAME_PACKAGE_PATH` | Workflow package path | `../reframe-package-swift-cbpr` |
| `RUST_LOG` | Logging level | `info` |
| `TOKIO_WORKER_THREADS` | Worker threads | `auto` or `8` |
| `API_SERVER_URL` | API server URL | `https://api.example.com` |

---
