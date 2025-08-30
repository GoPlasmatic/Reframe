<div align="center">
  <img src="https://avatars.githubusercontent.com/u/207296579?s=200&v=4" alt="Plasmatic Logo" width="120" height="120">
  <h1>Reframe</h1>
  <p><strong>An Enterprise-Grade, Bidirectional SWIFT MT ↔ ISO 20022 Transformation Engine</strong></p>
  <p>Reframe is the world's first completely transparent, open-source platform for high-performance SWIFT message transformation.</p>
  <br>
  
  [![License: Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
  [![Rust](https://img.shields.io/badge/Rust-1.70+-orange)](https://rust-lang.org)
  [![Swift SR2025](https://img.shields.io/badge/Swift-SR2025%20Compliant-green.svg)](https://www.swift.com)
  
  [📚 **Read the Docs**](docs/) | [🔧 **Installation Guide**](docs/installation.md) | [💡 **Explore the Architecture**](docs/architecture.md)
</div>

---

## 🌟 Why Reframe?

In the evolving world of financial messaging, clarity and control are non-negotiable. **Reframe** was built to provide a powerful, open-source alternative to proprietary, black-box solutions for SWIFT MT and ISO 20022 transformation. It's designed for financial institutions that demand **auditable logic**, **elite performance**, and **complete control** over their messaging workflows.

-   **🔍 Full Transparency**: Say goodbye to black boxes. Every transformation rule is defined in human-readable JSON and is fully auditable.
-   **⚡ Blazing-Fast Performance**: Built in Rust, Reframe delivers sub-millisecond processing speeds, ensuring your operations are never bottlenecked.
-   **🔧 Total Configurability**: Define your own business logic and mappings with a simple yet powerful JSON-based configuration system.
-   **🔌 Extensible by Design**: A modular, pluggable architecture makes it easy to customize or extend Reframe to meet your specific needs.

---

## 🚀 Key Features

### 🔄 **Core Transformation Engine**

-   **True Bidirectional Processing**: Seamlessly transform messages both ways:
    -   **Forward**: SWIFT MT → ISO 20022 (MX)
    -   **Reverse**: ISO 20022 (MX) → SWIFT MT
-   **Broad Message Support**: Extensive coverage for over 30 message variants across payments, cash management, and status reporting families.
-   **Intelligent Routing**: Automatic message type detection and routing to the correct transformation workflow.

### 🏗️ **Built for the Enterprise**

-   **High Availability**: A stateless, container-native architecture allows for effortless horizontal scaling.
-   **Zero-Downtime Updates**: Hot-reload configurations and workflows on the fly via an API endpoint, no service restarts required.
-   **Production-Ready**: Comes with comprehensive error handling, monitoring, and observability baked in.
-   **Simple Deployment**: Ships as a single, lightweight Docker image for easy deployment in any environment.

### 📋 **Unmatched Transparency & Control**

-   **Externalized JSON Rules**: All transformation logic is stored in clear, auditable JSON files—no hidden or compiled logic.
-   **Powerful Workflow Engine**: Backed by a sophisticated dataflow engine ([dataflow-rs](https://github.com/GoPlasmatic/dataflow-rs)) to model complex processing pipelines.
-   **Declarative Logic**: Utilizes [datalogic-rs](https://github.com/GoPlasmatic/datalogic-rs) to enable powerful and clear declarative logic for complex field mappings.

---

## 🏆 How Reframe Compares

| Feature              | Reframe v3.0                                  | Traditional Solutions                 |
| -------------------- | --------------------------------------------- | ------------------------------------- |
| **Standards** | ✅ SR2025 compliant (November 2025 release)   | ❌ Often lagging on latest standards  |
| **Transparency** | ✅ Open source with 100% auditable JSON rules | ❌ Proprietary, black-box logic       |
| **Transformation** | ✅ Fully bidirectional (MT ↔ MX)              | ❌ Often one-way or limited reverse   |
| **Performance** | ✅ Rust-powered (sub-millisecond)             | ❌ Slower, often JVM-based            |
| **Configuration** | ✅ Hot-reloadable JSON, no downtime           | ❌ Requires vendor intervention       |
| **License** | ✅ Apache 2.0 (Free to use and modify)        | ❌ Expensive, restrictive licensing   |
| **Deployment** | ✅ Lightweight, single Docker container       | ❌ Heavy, complex infrastructure      |

---

## 🎮 Get Started in Minutes

### 🐳 **With Docker (Recommended)**

The fastest way to get Reframe running.

```bash
# Pull the latest image from Docker Hub
docker pull plasmatic/reframe:latest

# Run the container and expose the API on port 3000
docker run -p 3000:3000 plasmatic/reframe:latest

# The API is now live and waiting for requests at http://localhost:3000
```

### 🔧 **From Source**
For developers who want to build from the ground up.

```bash
# 1. Clone the repository
git clone https://github.com/GoPlasmatic/Reframe.git
cd Reframe

# 2. Build the project in release mode (optimized for performance)
cargo build --release

# 3. Run the application
./target/release/reframe
```

### 🌐 **Try It Now**
Use curl to send your first transformation request.

```bash
# Example: Transform an MT103 to a pacs.008
curl -X POST http://localhost:3000/transform/mt-to-mx \
  -H "Content-Type: application/json" \
  -d '{"message": "{1:F01BNPAFRPPXXX0000000000}{...}"}'

# Example: Transform a pacs.008 back to an MT103
curl -X POST http://localhost:3000/transform/mx-to-mt \
  -H "Content-Type: application/json" \
  -d '{"message": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><Document>...</Document>"}'

# Example: Hot-reload workflows after a configuration change
curl -X POST http://localhost:3000/admin/reload-workflows
```

---

## 🗺️ Roadmap

Our vision is to make Reframe the undisputed backbone for financial messaging.

### 🎯 **What's New in v3.0 (SR2025 Release)**

-   ✅ **Full Swift SR2025 Specification Compliance** - Complete implementation of the November 2025 SWIFT standards release.
-   ✅ **Enhanced CBPR+ Support** - Updated Business Application Headers and improved structured remittance information.
-   ✅ **SR2025 Validation Rules** - Comprehensive validation for new mandatory fields and enhanced data quality checks.
-   ✅ **Dynamic Configuration Management** - Hot-reload workflows without service interruption.

### 🌟 **On the Horizon**

-   🌐 A fully-managed, cloud-native SaaS offering.
-   🏢 Multi-tenant support for service providers.
-   📊 Enhanced monitoring and analytics dashboard.
-   🔌 Support for additional message formats (e.g., RTP, FedNow).
-   🔗 Integrations for blockchain-based settlement.

---

### 🗣️ **Get Involved**

This project thrives on community input. We welcome you to:

-   **💬 [Start a Discussion](https://github.com/GoPlasmatic/Reframe/discussions)**: Ask questions, share ideas, and connect with other users.
-   **🐛 [Report an Issue](https://github.com/GoPlasmatic/Reframe/issues)**: Found a bug or have a feature request? Let us know.
-   **📖 [Read the Docs](docs/)**: Dive deep into the architecture, guides, and API references.
-   **🔧 [Contribute](CONTRIBUTING.md)**: Help us improve Reframe by contributing your code and expertise.

---

## 📚 Documentation

| Guide | Description |
|-------|-------------|
| [🏗️ Architecture & Design](docs/architecture.md) | Complete technical architecture, design patterns, and implementation details |
| [🔧 Installation Guide](docs/installation.md) | Setup instructions for all environments |
| [📊 Workflow Guide](docs/workflow-guide.md) | Creating and configuring SR2025-compliant transformation workflows |
| [🗺️ Mapping Guide](docs/mapping-guide.md) | SR2025 field mappings and business rules |
| [📋 Message Formats](docs/message-formats.md) | SR2025 supported message types and variants |

---

## 🔒 Security & Compliance

We built Reframe with the stringent requirements of financial institutions in mind.

-   **🔐 Secure by Design**: Follows security best practices from the ground up.
-   **📋 Compliance-Ready**: Architected to support environments requiring PCI, SOX, and Basel III compliance.
-   **🔍 Full Audit Trail**: Provides complete logging and traceability for every transformation.
-   **🛡️ Proactive Security**: We actively monitor for vulnerabilities and provide regular security updates.

---

## 📄 License

Reframe is licensed under the Apache License, Version 2.0. You are free to use, modify, and distribute it. See [LICENSE](LICENSE) for more details.

---

<div align="center">
  <p><strong>Built with ❤️ by Plasmatic</strong></p>
  <p>Making financial messaging transparent, fast, and reliable.</p>
</div>