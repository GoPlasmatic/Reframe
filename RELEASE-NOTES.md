# Reframe Release Notes

## Version 1.4 - Major Release: Complete MT205 Corporate Payment Ecosystem

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

#### MT205 Corporate Financial Institution Transfers ✨ NEW
- **MT205 Normal** → `pacs.009.001.08` (Corporate Financial Institution Transfer)
- **MT205 COV** → `pacs.009.001.08 COVE` (Corporate Cover Payment using Correspondent Banks)
- **MT205 REJT** → `pacs.002.001.10` (Payment Status Report - Corporate Rejection)
- **MT205 RETN** → `pacs.004.001.09` (Corporate Payment Return)

### ✨ Key Features

- **🤖 Automatic Detection**: Intelligent message type and processing method detection for MT103, MT202, and MT205
- **🔄 Complete Workflow Engine**: 37+ specialized transformation workflows across three message ecosystems
- **📋 CBPR+ Compliance**: Full Cross-Border Payments and Reporting Plus compliance
- **🌐 Enhanced Web Interface**: Modern Material Design UI with 12 sample messages covering all variants
- **⚡ High Performance**: Built with Rust for maximum throughput
- **✅ Schema Validation**: Real-time ISO 20022 compliance checking
- **🏢 Corporate Payment Support**: Complete MT205 ecosystem for corporate financial transfers

### 🚀 Advanced Capabilities

- **Cover Payment Detection**: Automatic identification of correspondent banking scenarios for MT202 and MT205
- **Settlement Method Logic**: Intelligent INDA/INGA/COVE settlement determination across all message types
- **Exception Handling**: Complete rejection and return processing workflows for all three message ecosystems
- **Multi-Stage Processing**: Business Application Header + Document mapping + XML generation
- **Corporate Context**: Enhanced field mapping and validation for corporate payment scenarios
- **Method Auto-Classification**: Automatic detection of normal, cover, rejection, or return processing

### 🎯 Production Ready

- **Enterprise Grade**: Complete transformation coverage for MT103, MT202, and MT205 ecosystems
- **API Endpoint**: RESTful API with comprehensive error handling for all message types
- **Sample Messages**: 12 pre-loaded sample messages covering all scenarios and variants
- **Real-time Processing**: Immediate transformation with detailed processing feedback
- **Comprehensive Documentation**: Updated README and roadmap reflecting complete implementation

### 🔧 Technical Implementation

#### Workflow Architecture
- **MT103 Workflows**: 13 specialized workflows (5 core + 4 rejection + 4 return)
- **MT202 Workflows**: 17 specialized workflows (5 core + 4 cover + 4 rejection + 4 return)
- **MT205 Workflows**: 17 specialized workflows (5 core + 4 cover + 4 rejection + 4 return) ✨ NEW
- **Shared Components**: Common parsing and validation workflows

#### Message Processing Capabilities
- **Normal Processing**: Standard payment transfers for all three message types
- **Cover Processing**: Correspondent banking scenarios for MT202 and MT205
- **Rejection Processing**: Comprehensive rejection handling with reason codes
- **Return Processing**: Complete return workflows with UETR validation

#### Enhanced Field Mapping
- **Corporate Context**: Specialized mapping for MT205 corporate scenarios
- **Settlement Logic**: Advanced 4-table decision logic for all message types
- **Charge Processing**: Comprehensive charge information handling
- **Validation Engine**: Enhanced precondition checks for corporate payments

### 📊 Implementation Statistics

- **Message Types Supported**: 3 complete ecosystems (MT103, MT202, MT205)
- **Total Variants**: 12 message processing scenarios
- **Workflow Files**: 37 specialized transformation workflows
- **ISO 20022 Schemas**: 4 target formats (pacs.008, pacs.009, pacs.002, pacs.004)
- **Coverage**: 100% for core payment ecosystems including corporate transfers

---

**Total Supported Formats**: 12 message variants → 4 ISO 20022 schemas  
**Workflow Coverage**: 100% for MT103, MT202, and MT205 business scenarios  
**Corporate Support**: Complete MT205 implementation with all variants  
**Deployment**: Production-ready with Azure Container deployment

## Previous Releases

### Version 1.3 - Production Release
- Complete MT103 and MT202 ecosystem implementation
- 27 specialized workflows
- 8 message variants supported
- Production deployment with Azure Container Instances

### Version 1.2 - Cover Payment Support
- MT202 COV implementation
- Correspondent banking detection
- Enhanced settlement logic

### Version 1.1 - Exception Handling
- MT103/MT202 rejection and return processing
- UETR validation
- Enhanced error handling

### Version 1.0 - Initial Release
- Core MT103 and MT202 normal processing
- Basic workflow engine
- Web UI foundation 