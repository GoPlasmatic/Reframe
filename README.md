<div align="center">
  <img src="https://avatars.githubusercontent.com/u/207296579?s=200&v=4" alt="Plasmatic Logo" width="120" height="120">
  <h1>Reframe</h1>
  <p><strong>A Universal Message Transformation Engine</strong></p>
  <p>High-performance, package-based transformation engine for financial messaging standards</p>
  <br>

  [![License: Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
  [![Rust](https://img.shields.io/badge/Rust-1.89+-orange)](https://rust-lang.org)
  [![Docker](https://img.shields.io/badge/Docker-Ready-blue)](https://hub.docker.com/r/plasmatic/reframe)

  [📚 **Documentation**](docs/) | [🔧 **Installation**](docs/installation.md) | [🏗️ **Architecture**](docs/architecture.md) | [🐳 **Docker Guide**](DOCKER.md)
</div>

---

## 🌟 What is Reframe?

**Reframe v3.1** is a **universal transformation engine** designed for financial messaging. Unlike traditional solutions, Reframe separates the core engine from transformation logic, allowing you to:

- 🔌 **Load any transformation package** - Not limited to any specific standard
- 🔄 **Process bidirectionally** - Transform messages in both directions seamlessly
- ⚡ **Scale effortlessly** - Stateless, async architecture built in Rust
- 🔍 **Maintain full transparency** - All transformation rules in auditable JSON

### Package-Based Architecture

Reframe is **not** specific to SWIFT or any particular standard. It's a general-purpose engine that loads transformation packages:

```
┌─────────────────────────────────┐
│   Reframe Engine (v3.1)         │
│   • Transformation workflows    │
│   • Sample generation           │
│   • Message validation          │
│   • Hot-reload capabilities     │
└─────────────────────────────────┘
              ↓ loads
┌─────────────────────────────────┐
│   Transformation Package        │
│   (e.g., SWIFT CBPR+ MT ↔ MX)   │
│   • Workflow definitions        │
│   • Mapping rules               │
│   • Validation logic            │
│   • Test scenarios              │
└─────────────────────────────────┘
```

**Example Package**: [SWIFT CBPR+ MT ↔ ISO 20022](https://github.com/GoPlasmatic/reframe-package-swift-cbpr) - SR2025 compliant bidirectional transformations

---

## 🚀 Key Features

### 🏗️ **Universal Transformation Engine**

- **Package-Based**: Load any transformation package - not limited to SWIFT
- **Bidirectional Processing**: Transform messages in both directions
- **Automatic Detection**: Intelligently routes messages to correct workflows
- **Hot-Reload**: Update workflows without restarting the service

### ⚡ **Enterprise Performance**

- **Rust-Powered**: Sub-millisecond processing speeds
- **Async/Await**: Non-blocking I/O with Tokio runtime
- **Horizontal Scaling**: Stateless architecture for easy scaling
- **Resource Efficient**: Configurable worker threads for optimal CPU utilization

### 🔧 **Developer-Friendly**

- **RESTful API**: Simple, unified endpoints for all operations
- **OpenAPI Documentation**: Interactive Swagger UI included
- **GraphQL API**: Query audit data with custom fields support
- **Docker-Ready**: Single container deployment with volume mounts
- **Configuration Management**: Three-tier config system (env vars, config file, defaults)

### 🔍 **Complete Transparency**

- **Open Source**: Apache 2.0 licensed
- **Auditable Rules**: All transformation logic in JSON
- **Workflow Engine**: Powered by [dataflow-rs](https://github.com/GoPlasmatic/dataflow-rs)
- **Declarative Logic**: Uses [datalogic-rs](https://github.com/GoPlasmatic/datalogic-rs)

### 📊 **Custom Fields & Analytics**

- **Package-Specific Fields**: Define computed fields using JSONLogic expressions
- **GraphQL Integration**: Query and filter by custom fields
- **Flexible Storage**: Choose between precompute, runtime, or hybrid strategies
- **Business Intelligence**: Add risk scoring, categorization, and derived insights

---

## 🎯 Quick Start

### Prerequisites

- **Docker** (for containerized deployment) - Recommended
- **Rust 1.89+** (for building from source)
- **Internet connection** (for Docker to download workflow packages)

**Note**: When using Docker, workflow packages are downloaded automatically - no manual setup required!

### Option 1: Docker (Recommended)

```bash
# 1. Clone Reframe
git clone https://github.com/GoPlasmatic/Reframe.git
cd Reframe

# 2. Run with docker-compose (downloads package automatically)
docker-compose up -d

# 3. Check health
curl http://localhost:3000/health

# 4. View API documentation
open http://localhost:3000/swagger-ui
```

**Note**: No need to clone workflow packages separately - they are downloaded automatically during the Docker build from GitHub releases.

### Option 2: From Source

```bash
# 1. Clone Reframe engine
git clone https://github.com/GoPlasmatic/Reframe.git
cd Reframe

# 2. Clone a transformation package
cd ..
git clone https://github.com/GoPlasmatic/reframe-package-swift-cbpr.git
cd Reframe

# 3. Build and run
cargo build --release
REFRAME_PACKAGE_PATH=../reframe-package-swift-cbpr cargo run --release
```

---

## 🌐 API Endpoints

### Unified Transformation
```bash
# Transform any message (auto-detects MT or MX)
curl -X POST http://localhost:3000/api/transform \
  -H "Content-Type: application/json" \
  -d '{
    "message": "{1:F01BANKBEBBAXXX0000000000}{2:O1030...}",
    "validation": true,
    "debug": false
  }'
```

### Sample Generation
```bash
# Generate sample messages
curl -X POST http://localhost:3000/api/generate \
  -H "Content-Type: application/json" \
  -d '{
    "message_type": "MT103",
    "scenario": "standard"
  }'
```

### Message Validation
```bash
# Validate any message
curl -X POST http://localhost:3000/api/validate \
  -H "Content-Type: application/json" \
  -d '{
    "message": "<your-message-here>",
    "business_validation": true
  }'
```

### Administration
```bash
# Hot-reload workflows
curl -X POST http://localhost:3000/admin/reload-workflows

# List loaded packages
curl http://localhost:3000/packages
```

### GraphQL API
```graphql
# Query messages with custom fields
query {
  messages(
    filter: {
      package_id: "swift-cbpr-mt-mx",
      custom_field_filters: {
        transaction_risk_score: { gte: 70 },
        is_cross_border: true
      }
    },
    limit: 10
  ) {
    messages {
      id
      custom_fields
    }
    total_count
  }
}
```

### Interactive Documentation
- **REST API**: Visit `http://localhost:3000/swagger-ui` for full API documentation
- **GraphQL API**: Visit `http://localhost:3000/graphql` for interactive GraphQL playground

---

## 📦 Transformation Packages

Reframe uses external packages to define transformation logic. This separation allows:

- **Flexibility**: Switch between different standard implementations
- **Version Control**: Manage transformation rules independently
- **Customization**: Create packages for proprietary formats
- **Collaboration**: Share and reuse transformation packages

### Available Packages

| Package | Description | Link |
|---------|-------------|------|
| **SWIFT CBPR+ MT ↔ ISO 20022** | SR2025 compliant bidirectional transformations | [GitHub](https://github.com/GoPlasmatic/reframe-package-swift-cbpr) |
| _Your custom package_ | Create your own transformation package | [Package Guide](docs/architecture.md#package-structure) |

### Package Structure

```
reframe-package-swift-cbpr/
├── reframe-package.json       # Package metadata and configuration
├── transform/                 # Transformation workflows
│   ├── index.json
│   └── [workflow files]
├── generate/                  # Sample generation workflows
│   ├── index.json
│   └── [generation workflows]
├── validate/                  # Validation workflows
│   ├── index.json
│   └── [validation workflows]
└── scenarios/                 # Test scenarios
    └── [scenario files]
```

---

## ⚙️ Configuration

Reframe uses a three-tier configuration system:

1. **Environment Variables** (highest priority)
2. **reframe.config.json** (optional configuration file)
3. **Built-in Defaults** (fallback values)

### Example Configuration

```json
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
      "tokio_worker_threads": "auto"
    }
  },
  "logging": {
    "format": "compact",
    "level": "info"
  },
  "api_docs": {
    "enabled": true,
    "title": "Reframe API"
  }
}
```

### Key Environment Variables

- `REFRAME_PACKAGE_PATH` - Path to transformation package
- `TOKIO_WORKER_THREADS` - Number of async runtime workers (default: CPU count)
- `RUST_LOG` - Logging level (debug, info, warn, error)
- `API_SERVER_URL` - Server URL for OpenAPI docs

See [Configuration Guide](docs/configuration.md) for full details.

---

## 🏆 Why Choose Reframe?

| Feature | Reframe v3.1 | Traditional Solutions |
|---------|--------------|----------------------|
| **Architecture** | ✅ Package-based, engine + packages | ❌ Monolithic, vendor-locked |
| **Standards** | ✅ Universal - any standard via packages | ❌ Limited to specific standards |
| **Transparency** | ✅ 100% auditable JSON workflows | ❌ Proprietary black boxes |
| **Performance** | ✅ Rust-powered, sub-ms latency | ❌ Slower, often JVM-based |
| **Updates** | ✅ Hot-reload, zero downtime | ❌ Requires restarts/deployments |
| **Analytics** | ✅ Custom fields with GraphQL API | ❌ Limited or no built-in analytics |
| **Extensibility** | ✅ Create custom packages easily | ❌ Vendor-dependent customization |
| **License** | ✅ Apache 2.0 (free & open) | ❌ Expensive proprietary licenses |
| **Deployment** | ✅ Single Docker container | ❌ Complex multi-component setups |

---

## 📚 Documentation

| Guide | Description |
|-------|-------------|
| [🏗️ Architecture](docs/architecture.md) | Technical architecture and design patterns |
| [🔧 Installation](docs/installation.md) | Setup instructions for all environments |
| [⚙️ Configuration](docs/configuration.md) | Complete configuration reference |
| [🐳 Docker Guide](DOCKER.md) | Docker setup and deployment |
| [📋 Message Formats](docs/message-formats.md) | Supported message types (package-specific) |
| [📊 Custom Fields](docs/custom-fields.md) | Package-specific computed fields and GraphQL API |

---

## 🗺️ What's New in v3.1

### Package-Based Architecture
- **Engine-Package Separation**: Core engine independent of transformation logic
- **Package Manager**: Dynamic package discovery and loading
- **Configuration System**: Flexible three-tier configuration
- **Hot-Reload**: Update packages without service restart

### Custom Fields & Analytics
- **Package-Specific Fields**: Define computed fields using JSONLogic expressions
- **GraphQL Integration**: Query transformation history with custom fields
- **Flexible Storage**: Choose precompute, runtime, or hybrid strategies
- **Business Logic**: Add risk scoring, categorization, and compliance checks

### Unified API
- **Single Transform Endpoint**: Auto-detection replaces separate MT/MX endpoints
- **Metadata Support**: Pass custom metadata through workflows
- **Improved Validation**: Unified validation for all message types
- **Enhanced Error Handling**: Structured error responses

### Performance Improvements
- **Async Engine**: Non-blocking I/O with Tokio runtime
- **Optimized Threading**: Configurable worker threads
- **Resource Efficiency**: Better CPU and memory utilization
- **Database Integration**: MongoDB persistence with audit trail

---

## 🔒 Security & Compliance

- **🔐 Secure by Design**: Security best practices throughout
- **📋 Compliance-Ready**: Supports PCI, SOX, Basel III requirements
- **🔍 Audit Trail**: Complete logging and traceability
- **🛡️ Regular Updates**: Active security monitoring and patches

---

## 🤝 Contributing

We welcome contributions! Whether you're:

- 🐛 Reporting bugs
- 💡 Suggesting features
- 📖 Improving documentation
- 🔧 Submitting code

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

---

## 📞 Community & Support

- **💬 [Discussions](https://github.com/GoPlasmatic/Reframe/discussions)** - Ask questions and share ideas
- **🐛 [Issues](https://github.com/GoPlasmatic/Reframe/issues)** - Report bugs or request features
- **📧 Email** - enquires@goplasmatic.io
- **🌐 Website** - [goplasmatic.io](https://goplasmatic.io)

---

## 📄 License

Reframe is licensed under the **Apache License 2.0**. You are free to use, modify, and distribute it. See [LICENSE](LICENSE) for details.

---

<div align="center">
  <p><strong>Built with ❤️ by Plasmatic</strong></p>
  <p>Making message transformation transparent, fast, and reliable.</p>
  <br>
  <p>
    <a href="https://github.com/GoPlasmatic/Reframe">⭐ Star us on GitHub</a> •
    <a href="https://github.com/GoPlasmatic/reframe-package-swift-cbpr">📦 SWIFT Package</a> •
    <a href="docs/">📚 Documentation</a>
  </p>
</div>
