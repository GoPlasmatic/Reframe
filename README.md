<div align="center">
  <img src="static/plasmatic_logo.png" alt="Plasmatic Logo" width="200" height="200">
  <h1>Reframe</h1>
  <p><strong>Enterprise-Grade Bidirectional SWIFT MT ↔ ISO 20022 Transformation Engine</strong></p>
  <p>The world's first completely transparent, open-source, bidirectional SWIFT message transformation platform</p>
  <br>
  
  [![CI/CD Pipeline](https://github.com/GoPlasmatic/Reframe/actions/workflows/deploy-azure.yml/badge.svg?branch=main)](https://github.com/GoPlasmatic/Reframe/actions/workflows/deploy-azure.yml)
  [![License: Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
  [![Docker](https://img.shields.io/badge/Docker-Available-blue)](https://hub.docker.com/)
  [![Rust](https://img.shields.io/badge/Rust-1.70+-orange)](https://rust-lang.org)
  
  [🚀 Live Demo](http://reframe-api-prod.eastus.azurecontainer.io:3000) | [📚 Documentation](docs/) | [🔧 Installation Guide](docs/installation.md) | [💡 Architecture](docs/architecture.md)
</div>

---

## 🌟 Why Reframe?

In the rapidly evolving landscape of financial messaging, **Reframe** stands as the definitive solution for SWIFT MT ↔ ISO 20022 transformation. Unlike traditional black-box solutions, Reframe delivers **complete transparency**, **enterprise-grade performance**, and **bidirectional capabilities** that set the standard for financial message processing.

### 🎯 Built for Financial Institutions Who Value:

- **🔍 Complete Transparency**: Zero black-box processing - every transformation rule is auditable
- **⚡ High Performance**: Rust-powered engine delivering sub-millisecond processing
- **🔧 Configurable Logic**: JSON-based workflow definitions for complete control
- **🔌 Pluggable Architecture**: Modular design enabling easy customization and extension
- **📊 Workflow-Driven**: Sophisticated dataflow engine with visual workflow capabilities

---

## 🚀 Key Features

### 🔄 **Bidirectional Transformation Engine**
- **Forward**: SWIFT MT → ISO 20022 (9 message families, 30+ variants)
- **Reverse**: ISO 20022 → SWIFT MT (9 message families, 30+ variants)
- **Automatic Detection**: Automatic message type recognition and routing
- **Real-time Processing**: Immediate transformation with comprehensive validation

### 🏗️ **Enterprise Architecture**
- **High Availability**: Stateless architecture enabling horizontal scaling
- **Production-Ready**: Comprehensive error handling, monitoring, and observability
- **Container-Native**: Single Docker image for seamless deployment

### 🌐 **Comprehensive Message Coverage**
- **Payment Messages**: MT103, MT202, MT205 with all variants (normal, cover, rejection, return)
- **Cancellation Workflows**: MT192, MT292, MT196, MT296 with UETR support
- **Cash Management**: MT900, MT910, MT940, MT942 account reporting
- **Cheque Processing**: Complete MT110, MT111, MT112 lifecycle support
- **Status Reporting**: MT199, MT299 rejection and status notifications

### 📋 **Transparent Workflow System**
- **JSON-Based Rules**: All transformation logic externalized in auditable JSON files
- **No Hidden Logic**: Every mapping rule visible and modifiable
- **Workflow Engine**: Powered by [dataflow-rs](https://github.com/GoPlasmatic/dataflow-rs) for sophisticated processing pipelines
- **Logic Engine**: [datalogic-rs](https://github.com/GoPlasmatic/datalogic-rs) enables declarative JSON Logic transformations
- **Visual Configuration**: Clear, hierarchical workflow organization

---

## 🏆 What Makes Reframe Different

| Feature | Reframe | Traditional Solutions |
|---------|---------|----------------------|
| **Transparency** | ✅ Complete source code + JSON workflows | ❌ Proprietary black boxes |
| **Bidirectional** | ✅ Full MT ↔ ISO 20022 support | ❌ Usually one-way only |
| **Performance** | ✅ Rust-powered, sub-millisecond | ❌ Often JVM-based, slower |
| **Configurability** | ✅ JSON-based, runtime configurable | ❌ Requires recompilation |
| **Open Source** | ✅ Apache 2.0 licensed | ❌ Proprietary licensing |
| **Message Coverage** | ✅ 30+ message variants | ❌ Limited message support |

---

## 🎮 Quick Start

### 🐳 **Docker (Recommended)**
```bash
# Pull and run the latest version
docker run -p 3000:3000 plasmatic/reframe:latest

# Access the web interface
open http://localhost:3000
```

### 🔧 **From Source**
```bash
# Clone and build
git clone https://github.com/GoPlasmatic/Reframe.git
cd Reframe
cargo build --release

# Run the application
./target/release/reframe
```

### 🌐 **Try It Now**
Transform your first message instantly:

```bash
# Forward transformation (MT103 → pacs.008)
curl -X POST http://localhost:3000/transform/mt-to-mx \
  -H "Content-Type: text/plain" \
  -d "{1:F01BNPAFRPPXXX0000000000}...{-}"

# Reverse transformation (pacs.008 → MT103)  
curl -X POST http://localhost:3000/transform/mx-to-mt \
  -H "Content-Type: application/xml" \
  -d "<?xml version=\"1.0\"?>..."
```

---

## 🗺️ Roadmap

### 🎯 **Currently Available (v2.2)**
- ✅ Complete bidirectional MT ↔ ISO 20022 transformation
- ✅ 30+ message variants with full lifecycle support
- ✅ Production-ready container deployment
- ✅ Comprehensive Web UI with testing capabilities
- ✅ AI-powered message mapping generation

### 🌟 **Future Vision**
- 🌐 Cloud-native SaaS offering
- 🚧 Advanced workflow visual editor
- 🚧 Multi-tenant architecture support
- 🚧 Enhanced monitoring and analytics dashboard
- 🚧 Additional message format support (RTP, FedNow)
- 🔗 Blockchain settlement integration

---

### 🗣️ **Get Involved**
- **💬 [Discussions](https://github.com/GoPlasmatic/Reframe/discussions)** - Ask questions and share ideas
- **🐛 [Issues](https://github.com/GoPlasmatic/Reframe/issues)** - Report bugs and request features
- **📖 [Documentation](docs/)** - Comprehensive guides and references
- **🔧 [Contributing](CONTRIBUTING.md)** - Help improve Reframe
---

## 📚 Documentation

| Guide | Description |
|-------|-------------|
| [🔧 Installation Guide](docs/installation.md) | Complete setup instructions for all environments |
| [📊 Workflow Guide](docs/workflow-guide.md) | How to create and configure transformation workflows |
| [🗺️ Mapping Guide](docs/mapping-guide.md) | Field mapping and business rule configuration |
| [🏗️ Architecture](docs/architecture.md) | Technical architecture and design decisions |
| [📋 Message Formats](docs/message-formats.md) | Complete list of supported message types |

---

## 🔒 Security & Compliance

- **🔐 Security-First**: Built with security best practices from the ground up
- **📋 Compliance Ready**: Designed for regulatory environments (PCI, SOX, Basel III)
- **🔍 Audit Trail**: Complete transformation logging and traceability
- **🛡️ Vulnerability Management**: Regular security updates and monitoring

---

## 📄 License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

---

<div align="center">
  <p><strong>Built with ❤️ by Plasmatic</strong></p>
  <p>Making financial messaging transformation transparent, fast, and reliable</p>
</div>