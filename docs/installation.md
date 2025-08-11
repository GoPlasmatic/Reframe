# 🔧 Installation Guide

Get Reframe up and running quickly with these comprehensive installation instructions.

## Table of Contents

- [Quick Start](#quick-start)
- [Docker Installation](#docker-installation)
- [From Source](#from-source)
- [Production Deployment](#production-deployment)
- [Configuration](#configuration)
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
open http://localhost:3000
```

### 🔍 Verify Installation

To ensure Reframe is working correctly, run these commands:

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

If you're developing custom workflows, use this setup:

```bash
# Run Reframe with your local workflows and logs directories
docker run -d \
  --name reframe-dev \
  -p 3000:3000 \
  -v $(pwd)/workflows:/app/workflows \
  -v $(pwd)/logs:/app/logs \
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

Then, use these commands to manage your Reframe services:

```bash
# Start all services defined in docker-compose.yml
docker-compose up -d

# View real-time logs for the Reframe service
docker-compose logs -f reframe

# Stop and remove all services
docker-compose down
```

---

## From Source

### Prerequisites

For those who want to build Reframe from source, ensure you have the following installed:

- **Rust**: 1.70 or later
- **Cargo**: Latest version
- **Node.js**: 18+ (for web UI development)
- **Git**: For cloning the repository

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

3. **Build Application**:
   Choose between a debug or release build:
   ```bash
   # Debug build (faster compilation, for development)
   cargo build

   # Release build (optimized for performance)
   cargo build --release
   ```

4. **Run Application**:
   Start Reframe in your preferred mode:
   ```bash
   # Debug mode
   cargo run

   # Release mode
   ./target/release/reframe
   ```

### Web UI Development

To contribute to or develop the web UI, follow these steps:

```bash
# Navigate to the web UI directory
cd web-ui

# Install necessary Node.js dependencies
npm install

# Start the development server with hot reloading
npm run dev

# Build the web UI for production deployment
npm run build
```

---

## Production Deployment

### System Requirements

For optimal performance in a production environment, ensure your system meets these requirements:

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| **CPU** | 1 core | 2+ cores |
| **RAM** | 512MB | 2GB+ |
| **Storage** | 1GB | 10GB+ |
| **Network** | 100Mbps | 1Gbps+ |

### Environment Variables

Configure these environment variables for your production setup:

```bash
# Set the logging level (default: info)
export RUST_LOG=info

# Change the default port (default: 3000)
export PORT=8080

# Specify a custom directory for workflows
export WORKFLOWS_DIR=/opt/reframe/workflows

# Define the log file location
export LOG_FILE=/var/log/reframe/application.log
```

## Configuration

### Workflow Configuration

Reframe uses JSON-based workflow definitions. Here's the default directory structure:

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

Reframe offers flexible logging configurations:

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

After installation, explore these resources to get the most out of Reframe:

1. **[Workflow Guide](workflow-guide.md)** - Learn how to create custom transformation workflows
2. **[Mapping Guide](mapping-guide.md)** - Configure field mappings and business rules
3. **[Architecture](architecture.md)** - Understand the technical architecture
4. **[Message Formats](message-formats.md)** - Complete list of supported message types

---