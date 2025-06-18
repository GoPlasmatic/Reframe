# Reframe Release Notes

## Version 1.3 - Production Release

### 🎯 Supported Message Transformations

#### MT103 Customer Credit Transfers
- **MT103 Normal** → `pacs.008.001.08` (Customer Credit Transfer)
- **MT103 STP** → `pacs.008.001.08` (Straight Through Processing)
- **MT103 REJT** → `pacs.002.001.10` (Payment Status Report - Rejection)
- **MT103 RETN** → `pacs.004.001.09` (Payment Return)

#### MT202 Financial Institution Transfers
- **MT202 Normal** → `pacs.009.001.08` (Financial Institution Credit Transfer)
- **MT202 COV** → `pacs.009.001.08 COVE` (Cover Payment using Correspondent Banks)
- **MT202 REJT** → `pacs.002.001.10` (Payment Status Report - Rejection)
- **MT202 RETN** → `pacs.004.001.09` (Payment Return)

### ✨ Key Features

- **🤖 Automatic Detection**: Intelligent message type and processing method detection
- **🔄 Complete Workflow Engine**: 27+ specialized transformation workflows
- **📋 CBPR+ Compliance**: Full Cross-Border Payments and Reporting Plus compliance
- **🌐 Web Interface**: Modern Material Design UI with sample messages
- **⚡ High Performance**: Built with Rust for maximum throughput
- **✅ Schema Validation**: Real-time ISO 20022 compliance checking

### 🚀 Advanced Capabilities

- **Cover Payment Detection**: Automatic identification of correspondent banking scenarios
- **Settlement Method Logic**: Intelligent INDA/INGA/COVE settlement determination
- **Exception Handling**: Complete rejection and return processing workflows
- **Multi-Stage Processing**: Business Application Header + Document mapping + XML generation

### 🎯 Production Ready

- **Enterprise Grade**: Complete transformation coverage for MT103 and MT202 ecosystems
- **API Endpoint**: RESTful API with comprehensive error handling
- **Sample Messages**: 8 pre-loaded sample messages covering all scenarios
- **Real-time Processing**: Immediate transformation with detailed processing feedback

---

**Total Supported Formats**: 8 message types → 4 ISO 20022 schemas  
**Workflow Coverage**: 100% for MT103 and MT202 business scenarios  
**Deployment**: Production-ready with Azure Container deployment 