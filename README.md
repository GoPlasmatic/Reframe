# Reframe - SWIFT Message to ISO 20022 Converter

Reframe is a Rust-based REST API service that converts SWIFT MT messages to ISO 20022 XML format. Currently specialized in comprehensive MT103, MT202, and MT205 transformations supporting normal processing, cover payments, rejection, and return scenarios. Built with Rust, Axum, and the dataflow-rs workflow engine for enterprise-grade CBPR+ compliance.

## Features

- 🚀 **High Performance**: Built with Rust and Axum for maximum throughput
- 🔄 **Comprehensive MT103, MT202 & MT205 Support**: Complete implementation of all MT103, MT202, and MT205 variants including cover payments, rejection and return processing
- 🤖 **Auto-Detection**: Automatically detects SWIFT message type and processing method
- 📋 **Advanced SWIFT MT Parsing**: Built-in SWIFT MT message parsing with method detection using swift-mt-message library
- 🌐 **Integrated Web UI**: Modern Material Design web interface with automatic sample loading and 16 sample messages
- 🔧 **No CORS Issues**: Web UI and API served from the same origin
- ⚡ **Advanced Workflow Engine**: Powered by dataflow-rs with 37+ specialized workflow stages
- 📊 **CBPR+ Compliance**: Full Cross-Border Payments and Reporting Plus compliance
- 🔧 **Extensible**: Modular design allows for additional message formats
- 📁 **Complex Workflow Management**: External JSON workflow definitions with conditional processing
- 🚢 **Production Ready**: Complete CI/CD pipeline with Azure deployment
- ✅ **Schema Validated**: Full ISO 20022 schema compliance with real-time validation
- 🏗️ **Consistent JSON Structure**: Recently updated with unified field references across all workflows
- 🎯 **Enhanced Sample Coverage**: Comprehensive test data samples matching backend validation

## Recent Major Updates (v1.5)

### 🔧 **Workflow System Overhaul**
- **Consistent JSON Structure**: All workflows updated to use unified field reference patterns
- **Enhanced Field Mapping**: BIC fields now use `.raw` suffix, field 20/21 use `.value`, field 72 uses `.lines`
- **Improved TR001 Logic**: Updated to use `basic_header.sender_bic.raw` for better accuracy
- **Dynamic Priority Logic**: Added intelligent priority determination based on field 23B values

### 📋 **MT202 Compliance Enhancements**
- **ISO 20022 Compliance**: Added missing Group Header fields for MT202-CORE specification
- **Settlement Amount Currency**: Added `TtlIntrBkSttlmAmt.@Ccy` and `TtlIntrBkSttlmAmt.$value`
- **Settlement Date**: Added `IntrBkSttlmDt` (Interbank Settlement Date) mapping
- **Cover vs Serial Routing**: Fixed publish conditions to properly route COVER and SERIAL payments

### 🌐 **Enhanced Web Interface**
- **16 Sample Messages**: Complete coverage of all test data scenarios
- **Exact Test Data Match**: Web UI samples now match backend test files precisely
- **Comprehensive Variants**: MT103, MT202, MT205 normal, cover, rejection, return, and detailed samples
- **Real-time Testing**: All workflow variants can be tested with authentic sample data

## Supported Transformations

### MT103 Message Types (Fully Implemented)

| SWIFT Message | ISO 20022 Format | Processing Method | Description | Status |
|---------------|------------------|------------------|-------------|--------|
| **MT103** | pacs.008.001.08 | Normal | Customer Credit Transfer | ✅ Complete |
| **MT103 STP** | pacs.008.001.08 | Straight Through Processing | Enhanced Customer Credit Transfer with STP compliance | ✅ Complete |
| **MT103 REJT** | pacs.002.001.10 | Rejection Processing | Payment Status Report for rejected transactions | ✅ Complete |
| **MT103 RETN** | pacs.004.001.09 | Return Processing | Payment Return for returned transactions | ✅ Complete |

### MT202 Message Types (Fully Implemented)

| SWIFT Message | ISO 20022 Format | Processing Method | Description | Status |
|---------------|------------------|------------------|-------------|--------|
| **MT202** | pacs.009.001.08 | Normal | Financial Institution Transfer | ✅ Complete |
| **MT202 COV** | pacs.009.001.08 COVE | Cover Payment | Cover payment using correspondent banks | ✅ Complete |
| **MT202 REJT** | pacs.002.001.10 | Rejection Processing | Payment Status Report for rejected interbank transfers | ✅ Complete |
| **MT202 RETN** | pacs.004.001.09 | Return Processing | Payment Return for returned interbank transfers | ✅ Complete |

### MT205 Message Types (Fully Implemented)

| SWIFT Message | ISO 20022 Format | Processing Method | Description | Status |
|---------------|------------------|------------------|-------------|--------|
| **MT205** | pacs.009.001.08 | Normal | Corporate Financial Institution Transfer | ✅ Complete |
| **MT205 COV** | pacs.009.001.08 COVE | Cover Payment | Corporate cover payment using correspondent banks | ✅ Complete |
| **MT205 REJT** | pacs.002.001.10 | Rejection Processing | Payment Status Report for rejected corporate transfers | ✅ Complete |
| **MT205 RETN** | pacs.004.001.09 | Return Processing | Payment Return for returned corporate transfers | ✅ Complete |

### Other Message Types (Not Implemented)

| SWIFT Message | ISO 20022 Format | Description | Status |
|---------------|------------------|-------------|--------|
| **MT192** | camt.056.001.08 | Request for Cancellation | ❌ Not Implemented |
| **MT196** | camt.029.001.09 | Client Side Liquidity Management Answer | ❌ Not Implemented |
| **MT210** | camt.057.001.06 | Notice to Receive | ❌ Not Implemented |

## 🗺️ Current Implementation Status

**Focus**: Complete MT103, MT202, and MT205 ecosystems with CBPR+ compliance ✅ **COMPLETE**

The system has achieved comprehensive transformation capabilities covering all business scenarios for three major message types with recent major enhancements:

- **Phase 1**: Core MT103 payment processing ✅ **COMPLETE**
- **Phase 2**: MT103 exception handling (rejection/return) ✅ **COMPLETE**
- **Phase 3**: Core MT202 interbank processing ✅ **COMPLETE**
- **Phase 4**: MT202 exception handling (rejection/return) ✅ **COMPLETE**
- **Phase 5**: MT202 Cover payment processing ✅ **COMPLETE**
- **Phase 6**: Core MT205 corporate payment processing ✅ **COMPLETE**
- **Phase 7**: MT205 Cover payment processing ✅ **COMPLETE**
- **Phase 8**: MT205 exception handling (rejection/return) ✅ **COMPLETE**
- **Phase 9**: CBPR+ Business Application Header mapping ✅ **COMPLETE**
- **Phase 10**: Workflow system consistency and compliance updates ✅ **COMPLETE**
- **Future**: Expansion to other MT message types - **Planned**

**🎯 Achievement**: 100% MT103, MT202, and MT205 coverage including cover payments with enhanced compliance - **Production Ready**

## Advanced Workflow System

Reframe uses a sophisticated multi-stage workflow system with 37+ specialized workflow files for comprehensive MT103, MT202, and MT205 processing, recently enhanced with consistent JSON structure:

### Workflow Architecture

- **Conditional Processing**: Each workflow has complex condition logic for different message types and scenarios
- **Sequential Execution**: Workflows execute in dependency order based on previous task completion
- **Error Handling**: Comprehensive validation and precondition checks at each stage
- **CBPR+ Compliance**: Full implementation of Cross-Border Payments and Reporting Plus standards
- **Consistent JSON Structure**: Unified field reference patterns across all workflows for improved maintainability
- **Enhanced Field Mapping**: Standardized BIC field handling, transaction reference mapping, and sender-to-receiver information processing

### Recent Workflow Enhancements

#### Field Reference Standardization
- **BIC Fields**: Updated to use `.raw` suffix (e.g., `52A.bic.raw`, `58A.bic.raw`)
- **Transaction References**: Field 20/21 now use `.value` instead of separate reference fields
- **Information Fields**: Field 72 updated to use `.lines` for consistent array handling
- **Header Logic**: TR001 logic uses `basic_header.sender_bic.raw` for improved accuracy

#### Dynamic Priority Logic
- **Intelligent Priority**: Field 23B = "URGP" → Priority = "URGT", otherwise "NORM"
- **Consistent Implementation**: Applied across all MT103, MT202, and MT205 workflows
- **CBPR+ Compliance**: Enhanced priority handling for Cross-Border Payments compliance

#### MT202 Compliance Improvements
- **Group Header Fields**: Added missing `TtlIntrBkSttlmAmt.@Ccy`, `TtlIntrBkSttlmAmt.$value`, `IntrBkSttlmDt`
- **Settlement Logic**: Enhanced settlement method determination for COVER vs SERIAL payments
- **Publish Routing**: Fixed conditions to use `temp_data.MTType` for proper document format routing

### MT103 Processing Workflows

#### Core Processing Pipeline (5 workflows)
1. **01-parse.json** - Initial SWIFT MT message parsing with method detection
2. **02-mt103-bah-mapping.json** - Business Application Header mapping for normal/STP with enhanced field references
3. **03-mt103-precondition.json** - Validation and precondition checks with updated field 72 handling
4. **04-mt103-document-mapping.json** - Document structure mapping (58KB, 1099 lines) with consistent JSON structure
5. **05-mt103-combine-cbpr.json** - XML combination and final output

#### Rejection Processing Pipeline (4 workflows)
6. **06-mt103-rejt-bah-mapping.json** - BAH mapping for rejection messages with standardized BIC handling
7. **07-mt103-rejt-precondition.json** - Rejection-specific validation with enhanced field references
8. **08-mt103-rejt-document-mapping.json** - pacs.002 document mapping with updated JSON structure
9. **09-mt103-rejt-combine-cbpr.json** - Rejection XML combination

#### Return Processing Pipeline (4 workflows)
10. **10-mt103-retn-bah-mapping.json** - BAH mapping for return messages with consistent field references
11. **11-mt103-retn-precondition.json** - Return-specific validation (includes UETR checks) with updated structure
12. **12-mt103-retn-document-mapping.json** - pacs.004 document mapping (27KB, 576 lines) with enhanced handling
13. **13-mt103-retn-combine-cbpr.json** - Return XML combination with charge validation

### MT202 Processing Workflows

#### Core Processing Pipeline (5 workflows)
1. **01-parse.json** - Shared message parsing with method detection
2. **02-mt202-bah-mapping.json** - Business Application Header mapping with enhanced TR001 logic
3. **03-mt202-precondition.json** - Validation with updated field 72 references (.lines)
4. **04-mt202-document-mapping.json** - Document structure mapping with ISO 20022 compliance fixes
5. **05-mt202-combine-cbpr.json** - XML combination and final output

#### Cover Payment Processing Pipeline (4 workflows)
6. **06-mt202cov-bah-mapping.json** - BAH mapping for cover payment messages with consistent BIC handling
7. **07-mt202cov-precondition.json** - Cover-specific validation and message type setting
8. **08-mt202cov-document-mapping.json** - pacs.009.001.08 COVE document mapping with proper routing
9. **09-mt202cov-combine-cbpr.json** - Cover payment XML combination

#### Rejection Processing Pipeline (4 workflows)
10. **10-mt202-rejt-bah-mapping.json** - BAH mapping with standardized field references
11. **11-mt202-rejt-precondition.json** - Rejection-specific validation with field 72 .lines handling
12. **12-mt202-rejt-document-mapping.json** - pacs.002 document mapping with enhanced structure
13. **13-mt202-rejt-combine-cbpr.json** - Rejection XML combination

#### Return Processing Pipeline (4 workflows)
14. **14-mt202-retn-bah-mapping.json** - BAH mapping with consistent JSON structure
15. **15-mt202-retn-precondition.json** - Return-specific validation with UETR requirements
16. **16-mt202-retn-document-mapping.json** - pacs.004 document mapping with updated field handling
17. **17-mt202-retn-combine-cbpr.json** - Return XML combination

### MT205 Processing Workflows

#### Core Processing Pipeline (5 workflows)
1. **01-parse.json** - Shared message parsing with method detection
2. **02-mt205-bah-mapping.json** - Business Application Header mapping with enhanced field references
3. **03-mt205-precondition.json** - Validation with updated field 72 handling (.lines)
4. **04-mt205-document-mapping.json** - Document structure mapping with consistent JSON structure
5. **05-mt205-combine-cbpr.json** - XML combination and final output

#### Cover Payment Processing Pipeline (4 workflows)
6. **06-mt205cov-bah-mapping.json** - BAH mapping with standardized BIC handling
7. **07-mt205cov-precondition.json** - Corporate cover-specific validation and message type setting
8. **08-mt205cov-document-mapping.json** - pacs.009.001.08 COVE document mapping for corporate cover
9. **09-mt205cov-combine-cbpr.json** - Corporate cover payment XML combination

#### Rejection Processing Pipeline (4 workflows)
10. **10-mt205-rejt-bah-mapping.json** - BAH mapping with consistent field references
11. **11-mt205-rejt-precondition.json** - Corporate rejection-specific validation with field 72 .lines
12. **12-mt205-rejt-document-mapping.json** - pacs.002 document mapping with enhanced structure
13. **13-mt205-rejt-combine-cbpr.json** - Corporate rejection XML combination

#### Return Processing Pipeline (4 workflows)
14. **14-mt205-retn-bah-mapping.json** - BAH mapping with standardized field references
15. **15-mt205-retn-precondition.json** - Corporate return-specific validation with UETR requirements
16. **16-mt205-retn-document-mapping.json** - pacs.004 document mapping with array handling fix
17. **17-mt205-retn-combine-cbpr.json** - Corporate return XML combination

### Workflow Features

- **Auto-Detection**: Workflows automatically determine processing path based on message content
- **Method Classification**: Automatic detection of normal, cover, rejection, or return processing
- **Field Validation**: Comprehensive field validation and mandatory field checks with consistent structure
- **Complex Mapping**: Advanced JSONLogic-based field transformation with standardized references
- **Settlement Logic**: Sophisticated settlement method determination with enhanced COVER vs SERIAL routing
- **Charge Processing**: Advanced charge information handling with proper array support

## Quick Start

### Production Deployment
Access the live application at: **http://reframe-api-prod.eastus.azurecontainer.io:3000**

The application provides:
- **Web Interface**: Integrated Material UI with split-panel layout and 16 sample messages
- **API Endpoint**: `/reframe` for programmatic access
- **Health Check**: `/health` for monitoring
- **Comprehensive Samples**: All MT103, MT202, and MT205 variants including detailed, minimal, and serial samples
- **XML Syntax Highlighting**: Real-time formatted output
- **Method Detection**: Automatic processing method identification
- **Test Data Consistency**: Web UI samples match exact backend test data

### Local Development

1. Clone the repository:
```bash
git clone <repository-url>
cd Reframe
```

2. Build the web UI:
```bash
cd web-ui
npm install
npm run build
cd ..
cp -r web-ui/build/* static/
```

3. Build and run the application:
```bash
cargo run
```

4. Open your browser to `http://localhost:3000`

## Deployment

### Automated Deployment

The project includes a complete CI/CD pipeline that automatically:

1. **Tests** the Rust code (format, clippy, unit tests)
2. **Builds** the React web UI and creates static files
3. **Builds** and pushes Docker images to Azure Container Registry
4. **Deploys** to staging environment for testing
5. **Deploys** to production environment
6. **Tests** both web UI and API endpoints
7. **Cleans up** staging resources

#### Triggering Deployment

- **Automatic**: Push to `main` branch
- **Manual**: Use GitHub Actions workflow dispatch

### Manual Deployment

To deploy manually:

```bash
# Build the web UI
cd web-ui
npm run build
cd ..
cp -r web-ui/build/* static/

# Build and run locally
cargo run

# Or build Docker image
docker build -t reframe .
docker run -p 3000:3000 reframe
```

## Architecture

### Enhanced Architecture

- **Single Container**: Rust application serves both API and web UI
- **Azure Container Instances (ACI)**: Hosts the unified service
- **Azure Container Registry (ACR)**: Stores container images
- **GitHub Actions**: CI/CD automation with integrated web UI build
- **Static File Serving**: Web UI files served directly from Rust application
- **Advanced MT103, MT202 & MT205 Processing**: Comprehensive support for all three message type business scenarios

### Components

1. **API Layer**: Axum-based REST server with static file serving
2. **Web UI**: React Material-UI interface with automatic sample loading and method detection
3. **Advanced Workflow Engine**: dataflow-rs engine orchestrating 37+ specialized transformation pipelines
4. **Parser Module**: Enhanced SWIFT MT message parser with method detection (normal/cover/rejection/return)
5. **Publisher Module**: XML serialization for multiple ISO 20022 formats (pacs.008, pacs.009, pacs.002, pacs.004)
6. **Mapping Engine**: Complex JSONLogic-based field mapping with CBPR+ compliance

### Message Flow

1. User accesses web interface at `/` or makes API request to `/reframe`
2. **Parse Task**: Parses incoming SWIFT message and detects type and processing method
3. **Method Detection**: Engine determines processing path:
   - **MT103 Normal**: Standard customer transfer → pacs.008.001.08
   - **MT103 STP**: STP-compliant customer transfer → pacs.008.001.08 (STP variant)
   - **MT103 Rejection**: Customer transfer rejection → pacs.002.001.10
   - **MT103 Return**: Customer transfer return → pacs.004.001.09
   - **MT202 Normal**: Standard interbank transfer → pacs.009.001.08
   - **MT202 Cover**: Cover payment using correspondent banks → pacs.009.001.08 COVE
   - **MT202 Rejection**: Interbank transfer rejection → pacs.002.001.10
   - **MT202 Return**: Interbank transfer return → pacs.004.001.09
   - **MT205 Normal**: Standard corporate transfer → pacs.009.001.08
   - **MT205 Cover**: Corporate cover payment using correspondent banks → pacs.009.001.08 COVE
   - **MT205 Rejection**: Corporate transfer rejection → pacs.002.001.10
   - **MT205 Return**: Corporate transfer return → pacs.004.001.09
4. **Workflow Execution**: Appropriate workflow pipeline executes based on detected method
5. **Business Application Header**: CBPR+ compliant header generation
6. **Document Mapping**: Comprehensive field mapping with settlement logic
7. **XML Generation**: Schema-validated ISO 20022 XML output
8. **Validation**: Real-time schema compliance checking
9. Returns complete XML response with proper headers

## API Reference

### Web Interface
**GET** `/`

Serves the integrated React web interface with Material Design. Features include:
- **Method Auto-Detection**: Paste any MT103, MT202, or MT205 variant and processing method is automatically detected
- **Sample Messages**: Load sample MT103, MT202, and MT205 normal, cover, rejection, or return messages
- **Real-time Transformation**: Convert messages with immediate feedback and method identification
- **Syntax Highlighting**: XML output with proper formatting
- **Processing Method Display**: Clear indication of detected processing method

### Convert SWIFT Messages to ISO 20022
**POST** `/reframe`

Converts SWIFT MT103, MT202, and MT205 messages to appropriate ISO 20022 XML format. The engine automatically detects the message type and processing method, applying the appropriate transformation workflow.

**Request:**
- **Content-Type**: `text/plain`
- **Body**: Raw SWIFT MT103, MT202, or MT205 message (any variant)

**Example 1: MT103 Normal → pacs.008.001.08**
```bash
curl -X POST http://reframe-api-prod.eastus.azurecontainer.io:3000/reframe \
  -H "Content-Type: text/plain" \
  -d "{1:F01BNPAFRPPXXX0000000000}{2:O1031234240101DEUTDEFFXXXX12345678952401011234N}{3:{103:EBA}}{4:
:20:FT21001234567890
:23B:CRED
:32A:240101USD1000,00
:50K:/1234567890
ACME CORPORATION
123 MAIN STREET
NEW YORK NY 10001
:52A:BNPAFRPPXXX
:57A:DEUTDEFFXXX
:59:/DE89370400440532013000
MUELLER GMBH
HAUPTSTRASSE 1
10115 BERLIN
:70:PAYMENT FOR INVOICE 12345
:71A:OUR
-}"
```

**Example 2: MT205 Normal → pacs.009.001.08**
```bash
curl -X POST http://reframe-api-prod.eastus.azurecontainer.io:3000/reframe \
  -H "Content-Type: text/plain" \
  -d "{1:F01CORPBEBBAXXX0000000000}{2:I205CORPDEFAXXXXN}{3:{108:MT205}{121:660e8400-e29b-41d4-a716-446655440001}}{4:
:20:CP2024123456789
:21:REL2024987654321
:32A:241215USD1750000,00
:52A:CORPBEBBXXX
:58A:CORPDEFAXXX
:72:/INS/DIRECT CORPORATE PAYMENT
/PUR/CORPORATE SERVICES PAYMENT
STANDARD CORPORATE SETTLEMENT
-}"
```

**Example 3: MT205 Cover → pacs.009.001.08 COVE**
```bash
curl -X POST http://reframe-api-prod.eastus.azurecontainer.io:3000/reframe \
  -H "Content-Type: text/plain" \
  -d "{1:F01CORPBEBBAXXX0000000000}{2:I205CORPUS33XXXXN}{3:{108:MT205COV}{121:abcd1234-5678-90ef-ghij-klmnopqrstuv}}{4:
:20:CP2024567890123
:21:REL2024345678901
:32A:241220EUR4250000,00
:52A:/DE89370400440532013001
CORPBEBBXXX
:53A:CORPUS33XXX
:56A:CITIUS33XXX
:57A:HSBCSGSGXXX
:58A:/SG56HSBC000012345679
HSBCSGSG
:72:/INS/CORPORATE COVER PAYMENT
/PUR/QUARTERLY DIVIDEND DISTRIBUTION
CORPORATE CORRESPONDENT ROUTING
-}"
```

**Example 4: MT205 Rejection → pacs.002.001.10**
```bash
curl -X POST http://reframe-api-prod.eastus.azurecontainer.io:3000/reframe \
  -H "Content-Type: text/plain" \
  -d "{1:F01CORPDEFAXXX0000000000}{2:I205CORPBEBBXXXXN}{3:{108:MT205REJT}{121:987fcdeb-51a2-34b5-6789-426614174abd}}{4:
:20:REJ2024123789457
:21:CP2024987654322
:32A:241218GBP850000,00
:52A:CORPDEFAXXX
:53D:CORPORATE CORRESPONDENT BANK
CORPORATE BANKING DIVISION
FRANKFURT AM MAIN GERMANY
:56D:CORPORATE INTERMEDIARY BANK LTD
567 CORPORATE BANKING STREET
LONDON UNITED KINGDOM
:57D:CORPORATE ACCOUNT INSTITUTION
890 CORPORATE FINANCE AVENUE
BRUSSELS BELGIUM
:58A:CORPBEBBXXX
:72:/REJT/
/MREF/CP2024987654322
/RREF/REJ2024123789457
/AC01/CORPORATE ACCOUNT IDENTIFIER
/TEXT/INVALID CORPORATE ACCOUNT
INCORRECT ACCOUNT INFORMATION
-}"
```

**Example 5: MT205 Return → pacs.004.001.09**
```bash
curl -X POST http://reframe-api-prod.eastus.azurecontainer.io:3000/reframe \
  -H "Content-Type: text/plain" \
  -d "{1:F01CORPBEBBAXXX0000000000}{2:I205CORPUS33XXXXN}{3:{108:MT205RETN}{121:456a789b-12c3-45d6-e789-012345678de0}}{4:
:20:RET2024567891235
:21:CP2024123456790
:32A:241219USD1150000,00
:52A:CORPBEBBXXX
:53B:/CH/1234567891
CORPORATE CORRESPONDENT LOCATION
:56A:BNPAFRPPXXX
:57B:/IBAN/GB29BARC20001234567891
CORPORATE BARCLAYS LOCATION
:58A:CORPUS33
:72:/RETN/
/RTRN/INSUFFICIENT CORPORATE FUNDS
/MREF/CP2024123456790
/RREF/RET2024567891235
/RC/AG01/CORPORATE TRANSACTION
/TEXT/CORPORATE RETURN INSUFFICIENT
-}"
```

## Advanced Field Mapping

The application performs comprehensive mapping with different target schemas based on message type and processing method:

### MT103 Normal/STP Processing (→ pacs.008.001.08)
- **Group Header (GrpHdr)**: Message identification, creation date/time, settlement information
- **Credit Transfer Transaction Info**: Payment identification, settlement amounts, charge bearer
- **Agents and Parties**: Debtor/creditor agents, intermediary agents, settlement agents
- **Settlement Logic**: Advanced 4-table decision logic for settlement method determination
- **Charge Processing**: Comprehensive charge information mapping

### MT202 Normal Processing (→ pacs.009.001.08)
- **Group Header (GrpHdr)**: Message identification, creation date/time, settlement information
- **Financial Institution Transfer**: Interbank payment identification, settlement amounts
- **Agents and Parties**: Instructing/instructed agents, intermediary agents
- **Settlement Logic**: Interbank settlement method determination (INDA/INGA)
- **Time Indication**: Settlement time requirements and processing

### MT205 Normal Processing (→ pacs.009.001.08)
- **Group Header (GrpHdr)**: Message identification, creation date/time, settlement information
- **Financial Institution Transfer**: Corporate interbank payment identification, settlement amounts
- **Agents and Parties**: Instructing/instructed agents, intermediary agents for corporate transfers
- **Settlement Logic**: Corporate settlement method determination (INDA/INGA)
- **Corporate Context**: Enhanced field mapping for corporate payment scenarios

### MT202/MT205 Cover Processing (→ pacs.009.001.08 COVE)
- **Cover Detection**: Automatic detection based on fields 53A/54A vs sender/receiver BIC comparison
- **Group Header (GrpHdr)**: Message identification with INDA settlement method for cover payments
- **Credit Transfer Transaction**: Cover payment structure with correspondent bank routing
- **Agents and Parties**: Instructing/instructed agents, debtor/creditor, correspondent banks
- **Underlying Customer Transfer**: Customer credit transfer information within cover structure
- **Settlement Logic**: INDA (Instructed Agent) settlement method for cover payments

### Rejection Processing (→ pacs.002.001.10)
- **Payment Status Report**: Transaction status, original transaction references
- **Status Reason Information**: Rejection codes, additional information
- **Original Group Information**: References to original MT103/MT202/MT205 message
- **Transaction Information**: Instructing/instructed agent mapping

### Return Processing (→ pacs.004.001.09)
- **Payment Return**: Return identification, original UETR, return amounts
- **Return Chain**: Original debtor/creditor information, return path
- **Return Reason**: Return codes, additional explanatory information
- **Charges Information**: Return-related charges and fees
- **Settlement Time Indication**: Debit/credit timing for returns

### CBPR+ Business Application Header (All Methods)
- **From/To Financial Institution**: BIC code mapping with fallback logic
- **Business Message Identifier**: Unique message identification
- **Message Definition Identifier**: Appropriate schema version (pacs.008/009/002/004)
- **Business Service**: CBPR+ service identification
- **Priority**: Message priority mapping (URGT/NORM)

## Error Handling

The application provides detailed error messages for various failure scenarios:

- **Parse Errors**: Invalid SWIFT MT103/MT202/MT205 format or unsupported message types
- **Method Detection Errors**: Unable to determine processing method
- **Validation Errors**: Missing required fields, invalid field values
- **Workflow Errors**: Precondition failures, mapping errors
- **Serialization Errors**: XML generation failures

Example error responses:
```json
{"error": "Error processing data: Validation(\"Missing required field: 20\")"}
{"error": "Error processing data: Validation(\"Invalid MT103 format\")"}
{"error": "Error processing data: Validation(\"UETR is mandatory for MT103 RETN messages\")"}
{"error": "Error processing data: Validation(\"UETR is mandatory for MT202 RETN messages\")"}
{"error": "Error processing data: Validation(\"UETR is mandatory for MT205 RETN messages\")"}
```

## Development

### Project Structure
```
src/
├── main.rs          # Application entry point and workflow management
├── parser.rs        # SWIFT MT message parsing with method detection
└── publish.rs       # XML serialization for multiple ISO 20022 formats

workflows/
├── 01-parse.json                      # Shared parsing and method detection
│
├── MT103/                             # MT103 Customer Credit Transfer workflows
│   ├── bah-mapping.json              # BAH mapping (normal/STP)
│   ├── precondition.json             # Validation checks
│   ├── document-mapping.json         # Document mapping (58KB, 1099 lines)
│   └── combine-cbpr.json             # XML combination
│
├── MT103REJT/                         # MT103 Rejection workflows
│   ├── bah-mapping.json              # BAH mapping (rejection)
│   ├── precondition.json             # Rejection validation
│   ├── document-mapping.json         # Rejection document mapping
│   └── combine-cbpr.json             # Rejection XML combination
│
├── MT103RETN/                         # MT103 Return workflows
│   ├── bah-mapping.json              # BAH mapping (return)
│   ├── precondition.json             # Return validation
│   ├── document-mapping.json         # Return document mapping (576 lines)
│   └── combine-cbpr.json             # Return XML combination
│
├── MT202/                             # MT202 Interbank Transfer workflows
│   ├── bah-mapping.json              # BAH mapping (normal)
│   ├── precondition.json             # Validation checks
│   ├── document-mapping.json         # Document mapping (547 lines)
│   └── combine-cbpr.json             # XML combination
│
├── MT202COVER/                        # MT202 Cover workflows
│   ├── bah-mapping.json              # BAH mapping (cover)
│   ├── precondition.json             # Cover validation and message type setting
│   ├── document-mapping.json         # Cover document mapping (257 lines)
│   └── combine-cbpr.json             # Cover XML combination
│
├── MT202REJT/                         # MT202 Rejection workflows
│   ├── bah-mapping.json              # BAH mapping (rejection)
│   ├── precondition.json             # Rejection validation
│   ├── document-mapping.json         # Rejection document mapping (255 lines)
│   └── combine-cbpr.json             # Rejection XML combination
│
├── MT202RETN/                         # MT202 Return workflows
│   ├── bah-mapping.json               # BAH mapping (return)
│   ├── precondition.json              # Return validation
│   ├── document-mapping.json          # Return document mapping (433 lines)
│   └── combine-cbpr.json              # Return XML combination
│
├── MT205/                             # MT205 Corporate Transfer workflows
│   ├── bah-mapping.json              # BAH mapping (normal)
│   ├── precondition.json             # Validation checks
│   ├── document-mapping.json         # Document mapping
│   └── combine-cbpr.json             # XML combination
│
├── MT205COVER/                        # MT205 Cover workflows
│   ├── bah-mapping.json              # BAH mapping (cover)
│   ├── precondition.json             # Cover validation and message type setting
│   ├── document-mapping.json         # Cover document mapping
│   └── combine-cbpr.json             # Cover XML combination
│
├── MT205REJT/                         # MT205 Rejection workflows
│   ├── bah-mapping.json              # BAH mapping (rejection)
│   ├── precondition.json             # Rejection validation
│   ├── document-mapping.json         # Rejection document mapping
│   └── combine-cbpr.json             # Rejection XML combination
│
└── MT205RETN/                         # MT205 Return workflows
    ├── bah-mapping.json               # BAH mapping (return)
    ├── precondition.json              # Return validation
    ├── document-mapping.json          # Return document mapping
    └── combine-cbpr.json              # Return XML combination
```

### Building
```bash
cargo build
```

### Running Tests
```bash
cargo test
```

### Development Mode
```bash
cargo run
```

### Adding Support for Other Message Types

To extend support for other SWIFT message types:

1. **Add parser logic** in `src/parser.rs` for new message types
2. **Create workflow pipelines** following the MT103/MT202/MT205 pattern
3. **Extend publisher** in `src/publish.rs` for new output formats
4. **Add method detection** for different processing scenarios

Example workflow structure for new message types:
- Parse workflow for message detection (shared)
- BAH mapping workflow for header generation
- Precondition workflow for validation
- Document mapping workflow for field transformation
- Combination workflow for final XML output

Each message type should have its own directory under `workflows/` with separate subdirectories for rejection and return processing variants.

## Dependencies

### Core Dependencies
- **axum** (0.7): Modern web framework for the REST API
- **tokio** (1.0): Async runtime
- **tower** (0.4): Service abstractions and middleware
- **tower-http** (0.5): HTTP-specific middleware (CORS)
- **anyhow** (1.0): Error handling
- **async-trait** (0.1): Async traits

### Serialization
- **serde** (1.0): Serialization framework
- **serde_json** (1.0): JSON serialization
- **quick-xml** (0.31): XML serialization

### Financial Message Processing
- **dataflow-rs** (0.1.8): Advanced workflow processing engine
- **swift-mt-message** (0.1.1): SWIFT MT message parsing with method detection
- **mx-message** (0.1.1): ISO 20022 message structures (pacs.008, pacs.002, pacs.004)

## License

This project is licensed under the Apache License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## 🔧 Settlement Method Logic

**Fixed Settlement Method Determination:**
- **INDA** (Instructed Agent): Default for most MT103 payments and serial payments with intermediaries
- **INGA** (Instructing Agent): When field 53B contains "/C" prefix (clearing instruction)  
- **COVE** (Cover Payment): Only when fields 53A AND 54A are present AND no serial routing fields (56A/57A)
- **CLRG** (Clearing): For domestic clearing system settlement

**Recent Fix:** Corrected logic that incorrectly set COVE for serial payments with field 54A present. Now properly distinguishes cover payments from serial payments with correspondent banks.
