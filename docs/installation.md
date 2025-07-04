# 🔧 Installation Guide

Complete setup instructions for Reframe across all environments.

## Table of Contents
- [Quick Start](#quick-start)
- [Docker Installation](#docker-installation)
- [From Source](#from-source)
- [Production Deployment](#production-deployment)
- [Configuration](#configuration)
- [Troubleshooting](#troubleshooting)

---

## Quick Start

### 🐳 Docker (Recommended)

The fastest way to get Reframe running:

```bash
# Pull and run the latest version
docker run -p 3000:3000 plasmatic/reframe:latest

# Access the application
open http://localhost:3000
```

### 🔍 Verify Installation

```bash
# Check health status
curl http://localhost:3000/health

# Test transformation
curl -X POST http://localhost:3000/transform/mt-to-mx \
  -H "Content-Type: text/plain" \
  -d "{1:F01BNPAFRPPXXX0000000000}{2:O1031234240101DEUTDEFFXXXX12345678952401011234N}{4::20:TEST123:32A:240101USD100,00:50K:SENDER:59:RECEIVER:71A:OUR-}"
```

---

## Docker Installation

### Prerequisites
- Docker 20.10+ or Docker Desktop
- 2GB available RAM
- 1GB available disk space

### Standard Installation

```bash
# Create a dedicated directory
mkdir ~/reframe && cd ~/reframe

# Download and run
docker run -d \
  --name reframe \
  -p 3000:3000 \
  --restart unless-stopped \
  plasmatic/reframe:latest
```

### Development Installation

```bash
# Run with custom workflows directory
docker run -d \
  --name reframe-dev \
  -p 3000:3000 \
  -v $(pwd)/workflows:/app/workflows \
  -v $(pwd)/logs:/app/logs \
  plasmatic/reframe:latest
```

### Docker Compose

Create `docker-compose.yml`:

```yaml
version: '3.8'

services:
  reframe:
    image: plasmatic/reframe:latest
    container_name: reframe
    ports:
      - "3000:3000"
    volumes:
      - ./workflows:/app/workflows
      - ./logs:/app/logs
    environment:
      - RUST_LOG=info
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
```

```bash
# Start services
docker-compose up -d

# View logs
docker-compose logs -f reframe

# Stop services
docker-compose down
```

---

## From Source

### Prerequisites

- **Rust**: 1.70 or later
- **Cargo**: Latest version
- **Node.js**: 18+ (for web UI development)
- **Git**: For cloning the repository

### Installation Steps

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

3. **Build Application**:
   ```bash
   # Debug build (faster compilation)
   cargo build

   # Release build (optimized performance)
   cargo build --release
   ```

4. **Run Application**:
   ```bash
   # Debug mode
   cargo run

   # Release mode
   ./target/release/reframe
   ```

### Web UI Development

For web UI development:

```bash
# Navigate to web UI directory
cd web-ui

# Install dependencies
npm install

# Start development server
npm run dev

# Build for production
npm run build
```

---

## Production Deployment

### System Requirements

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| **CPU** | 1 core | 2+ cores |
| **RAM** | 512MB | 2GB+ |
| **Storage** | 1GB | 10GB+ |
| **Network** | 100Mbps | 1Gbps+ |

### Environment Variables

```bash
# Logging level
export RUST_LOG=info

# Custom port (default: 3000)
export PORT=8080

# Custom workflows directory
export WORKFLOWS_DIR=/opt/reframe/workflows

# Log file location
export LOG_FILE=/var/log/reframe/application.log
```

## Configuration

### Workflow Configuration

Reframe uses JSON-based workflow definitions:

```bash
# Default workflow locations
workflows/
├── forward/           # MT → ISO 20022 transformations
│   ├── index.json    # Workflow loading order
│   ├── MT103/        # MT103 specific workflows
│   └── MT202/        # MT202 specific workflows
└── reverse/          # ISO 20022 → MT transformations
    ├── index.json    # Workflow loading order
    └── pacs008/      # pacs.008 specific workflows
```

### Logging Configuration

```bash
# Available log levels
export RUST_LOG=error    # Errors only
export RUST_LOG=warn     # Warnings and errors
export RUST_LOG=info     # General information
export RUST_LOG=debug    # Detailed debugging
export RUST_LOG=trace    # Extremely verbose

# Module-specific logging
export RUST_LOG=info,reframe::parse_mt=debug,dataflow_rs=trace
```
---

## Getting Help

### Documentation
- [Workflow Guide](workflow-guide.md)
- [Architecture Overview](architecture.md)
- [Message Formats](message-formats.md)

### Community Support
- [GitHub Issues](https://github.com/GoPlasmatic/Reframe/issues)
- [GitHub Discussions](https://github.com/GoPlasmatic/Reframe/discussions)

---

## Next Steps

After installation, explore:

1. **[Workflow Guide](workflow-guide.md)** - Learn how to create custom transformation workflows
2. **[Mapping Guide](mapping-guide.md)** - Configure field mappings and business rules
3. **[Architecture](architecture.md)** - Understand the technical architecture
4. **[Message Formats](message-formats.md)** - Complete list of supported message types

---

*Last updated: January 2024*