# Reframe - SWIFT Message to ISO 20022 Converter

Reframe is a Rust-based REST API service that converts SWIFT MT messages to ISO 20022 XML format. Supporting multiple message types including MT103, MT192, MT196, MT202, and MT210, it provides a high-performance solution for financial message transformation with an integrated web interface. Built with Rust, Axum, and the dataflow-rs workflow engine.

## Features

- 🚀 **High Performance**: Built with Rust and Axum for maximum throughput
- 🔄 **Multiple Message Types**: Supports MT103, MT192, MT196, MT202, and MT210 transformations
- 🤖 **Auto-Detection**: Automatically detects SWIFT message type and applies appropriate transformation
- 📋 **SWIFT MT Parsing**: Built-in SWIFT MT message parsing using swift-mt-message library
- 🌐 **Integrated Web UI**: Modern Material Design web interface with automatic sample loading
- 🔧 **No CORS Issues**: Web UI and API served from the same origin
- ⚡ **Workflow Engine**: Powered by dataflow-rs for robust message processing
- 📊 **Error Handling**: Comprehensive error reporting for invalid messages
- 🔧 **Extensible**: Modular design allows for additional message formats
- 📁 **Configurable Workflows**: External JSON workflow definitions for easy customization
- 🚢 **Production Ready**: Complete CI/CD pipeline with Azure deployment
- ✅ **Schema Validated**: Full ISO 20022 schema compliance with real-time validation

## Supported Transformations

| SWIFT Message | ISO 20022 Format | Description | Status |
|---------------|------------------|-------------|--------|
| **MT103** | pacs.008.001.08 | Customer Credit Transfer | ✅ Complete |
| **MT192** | camt.056.001.08 | Request for Cancellation | ✅ Complete |
| **MT196** | camt.029.001.09 | Client Side Liquidity Management Answer | ✅ Complete |
| **MT202** | pacs.009.001.08 | General Financial Institution Transfer | ✅ Complete |
| **MT210** | camt.057.001.06 | Notice to Receive | ✅ Complete |

## 🗺️ Product Roadmap

**Current Status**: Core CBPR+ message types ✅ **COMPLETE**

Reframe has successfully implemented comprehensive CBPR+ (Cross-Border Payments and Reporting Plus) support for the most critical payment and cash management messages. See our detailed [**Product Roadmap**](roadmap.md) for:

- **Phase 1**: Core CBPR+ payment messages ✅ **COMPLETE** (MT103, MT202)
- **Phase 2**: Cash management & exceptions ✅ **COMPLETE** (MT210, MT192, MT196)  
- **Phase 3**: Enhanced CBPR+ features (LEI validation, sanctions screening) - **In Planning**
- **Phase 4**: Legacy message support (MT940, MT950, MT900, MT910) - **Backlog**

**🎯 Achievement**: 95%+ message coverage for core CBPR+ transformations

## Workflow Configuration

Reframe uses externalized JSON workflow definitions that can be modified without touching the source code. This allows for easy customization of message transformation logic.

### How It Works

- **Automatic Loading**: At startup, the application scans the `workflows/` directory for `.json` files
- **Hot Deployment**: Modify workflows and redeploy to update transformation logic
- **Auto-Detection**: Engine automatically detects message type and applies appropriate workflow
- **Extensible**: Add new workflows for different message types or transformation rules

### Current Workflows

The application includes comprehensive workflows for all supported message types:
- **MT103**: `workflows/01-mt-message-parser.json` + `workflows/02-mt103-pacs008-mapping.json`
- **MT192**: `workflows/03-mt192-camt056-mapping.json`
- **MT196**: `workflows/04-mt196-camt029-mapping.json`  
- **MT202**: `workflows/05-mt202-pacs009-mapping.json`
- **MT210**: `workflows/06-mt210-camt057-mapping.json`

### Customizing Workflows

1. **Modify Existing**: Edit workflow JSON files to change transformation logic
2. **Add New**: Create additional `.json` files in the `workflows/` directory
3. **Deploy**: Push changes and redeploy to activate new workflows

See the [Workflows README](workflows/README.md) for detailed configuration documentation.

## Quick Start

### Production Deployment
Access the live application at: **http://reframe-api-prod.eastus.azurecontainer.io:3000**

The application provides:
- **Web Interface**: Integrated Material UI with split-panel layout
- **API Endpoint**: `/reframe` for programmatic access
- **Health Check**: `/health` for monitoring
- Sample MT103 message loader
- XML syntax highlighting
- Real-time error handling

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

### Simplified Architecture

- **Single Container**: Rust application serves both API and web UI
- **Azure Container Instances (ACI)**: Hosts the unified service
- **Azure Container Registry (ACR)**: Stores container images
- **GitHub Actions**: CI/CD automation with integrated web UI build
- **Static File Serving**: Web UI files served directly from Rust application
- **Multi-Message Support**: Automatic detection and transformation of 5 SWIFT message types

### Components

1. **API Layer**: Axum-based REST server with static file serving
2. **Web UI**: React Material-UI interface with automatic sample loading and message detection
3. **Workflow Engine**: dataflow-rs engine orchestrating multiple transformation pipelines
4. **Parser Module**: Custom SWIFT MT message parser supporting MT103/192/196/202/210
5. **Publisher Module**: XML serialization for multiple ISO 20022 formats (pacs, camt)
6. **Mapping Engine**: JSONLogic-based field mapping with schema validation

### Message Flow

1. User accesses web interface at `/` or makes API request to `/reframe`
2. **Parse Task**: Parses incoming SWIFT message and detects type (MT103/192/196/202/210)
3. **Auto-Detection**: Engine automatically selects appropriate workflow based on message type
4. **Mapping Tasks**: Message-specific transformation tasks:
   - **MT103**: Customer Credit Transfer → pacs.008.001.08
   - **MT192**: Request for Cancellation → camt.056.001.08  
   - **MT196**: Investigation Answer → camt.029.001.09
   - **MT202**: Financial Institution Transfer → pacs.009.001.08
   - **MT210**: Notice to Receive → camt.057.001.06
5. **Publish Task**: Serializes mapped data to appropriate ISO 20022 XML format
6. **Validation**: Real-time schema compliance checking
7. Returns validated XML response to client

## API Reference

### Web Interface
**GET** `/`

Serves the integrated React web interface with Material Design. Features include:
- **Auto-Detection**: Paste any supported SWIFT message and it's automatically detected
- **Sample Messages**: Load sample MT103, MT192, MT196, MT202, or MT210 messages
- **Real-time Transformation**: Convert messages with immediate feedback
- **Syntax Highlighting**: XML output with proper formatting

### Convert SWIFT Messages to ISO 20022
**POST** `/reframe`

Converts SWIFT messages to ISO 20022 XML format. The engine automatically detects the message type and applies the appropriate transformation workflow.

**Request:**
- **Content-Type**: `text/plain`
- **Body**: Raw SWIFT message (MT103, MT192, MT196, MT202, or MT210)

**Example 1: MT103 → pacs.008.001.08**
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

**Example 2: MT192 → camt.056.001.08**
```bash
curl -X POST http://reframe-api-prod.eastus.azurecontainer.io:3000/reframe \
  -H "Content-Type: text/plain" \
  -d "{1:F01BNPAFRPPXXX0000000000}{2:O1921234240101DEUTDEFFXXXX12345678952401011234N}{3:{108:MT192}}{4:
:20:REQ240101001
:21:FT21001234567890
:11S:103
:32A:240101USD1000,00
:52A:BNPAFRPPXXX
:57A:DEUTDEFFXXX
:72:/RETN/AC01/Invalid account number
/CASE/CASE240101001
-}"
```

**Example 3: MT210 → camt.057.001.06**
```bash
curl -X POST http://reframe-api-prod.eastus.azurecontainer.io:3000/reframe \
  -H "Content-Type: text/plain" \
  -d "{1:F01BNPAFRPPXXX0000000000}{2:O2101234240101DEUTDEFFXXXX12345678952401011234N}{3:{108:MT210}}{4:
:20:NTR240101001
:25:12345678/001
:32A:240101USD2500,00
:50A:ACME CORPORATION
:52A:BNPAFRPPXXX
:57A:DEUTDEFFXXX
:58A:CITIUS33XXX
:72:/REC/Expected incoming payment
/REF/Reference information
-}"
```

## Field Mapping

The application performs comprehensive mapping from SWIFT MT103 fields to ISO 20022 pacs.008.001.13 fields:

### Group Header (GrpHdr)
| MT103 Field | pacs.008 Field | Description |
|-------------|----------------|-------------|
| :20: | MsgId | Message ID/Transaction Reference |
| System timestamp | CreDtTm | Creation Date Time |
| Fixed: "1" | NbOfTxs | Number of Transactions |
| :32A: amount | CtrlSum | Control Sum |
| Fixed: "Reframe Processing System" | InitgPty.Nm | Initiating Party Name |
| Fixed: "CLRG" | SttlmInf.SttlmMtd | Settlement Method |

### Credit Transfer Transaction Information (CdtTrfTxInf)
| MT103 Field | pacs.008 Field | Description |
|-------------|----------------|-------------|
| :20: | PmtId.InstrId | Instruction ID |
| :20: | PmtId.EndToEndId | End to End ID |
| :20: | PmtId.UETR | Unique End-to-end Transaction Reference |
| :32A: currency | IntrBkSttlmAmt.@Ccy | Settlement Amount Currency |
| :32A: amount | IntrBkSttlmAmt.$value | Settlement Amount Value |
| :32A: date | IntrBkSttlmDt | Settlement Date |
| :71A: | ChrgBr | Charge Bearer (OUR→DEBT, BEN→CRED, default→SHAR) |

### Agents and Parties
| MT103 Field | pacs.008 Field | Description |
|-------------|----------------|-------------|
| :52A: | InstgAgt.FinInstnId.BICFI | Instructing Agent BIC |
| :57A: | InstdAgt.FinInstnId.BICFI | Instructed Agent BIC |
| :50K: | Dbtr.Nm | Debtor Name |
| :50A: account | DbtrAcct.Id.IBAN | Debtor Account |
| :52A: | DbtrAgt.FinInstnId.BICFI | Debtor Agent BIC |
| :59: | Cdtr.Nm | Creditor Name |
| :59: account | CdtrAcct.Id.IBAN | Creditor Account |
| :57A: | CdtrAgt.FinInstnId.BICFI | Creditor Agent BIC |
| :56A: | IntrmyAgt1.FinInstnId.BICFI | Intermediary Agent 1 BIC |
| :56C: | IntrmyAgt2.FinInstnId.BICFI | Intermediary Agent 2 BIC |

### Additional Information
| MT103 Field | pacs.008 Field | Description |
|-------------|----------------|-------------|
| :70: | RmtInf.Ustrd.0 | Remittance Information |
| :79: | RgltryRptg.0.Dtls.0.Cd | Regulatory Reporting Code |

## Error Handling

The application provides detailed error messages for various failure scenarios:

- **Parse Errors**: Invalid SWIFT MT103 format
- **Validation Errors**: Missing required fields
- **Serialization Errors**: XML generation failures
- **Mapping Errors**: Field transformation issues

Example error responses:
```json
{"error": "Error processing data: Validation(\"Missing required field: 20\")"}
{"error": "Error processing data: Validation(\"Invalid MT103 format\")"}
{"error": "Error processing data: Validation(\"XML serialization failed\")"}
```

## Development

### Project Structure
```
src/
├── main.rs          # Application entry point and workflow configuration
├── parser.rs        # SWIFT MT message parsing functionality
└── publish.rs       # XML serialization and publishing
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

To extend support for other SWIFT message types or ISO 20022 formats:

1. **Add new parser logic** in `src/parser.rs` for different SWIFT message types
2. **Create new mapping workflows** in `setup_workflows()` function
3. **Extend publisher** in `src/publish.rs` for different output formats
4. **Register new custom functions** if needed

Example workflow for MT202 support:
```rust
let mt202_workflow = r#"
{
    "id": "mt202_to_pacs009_mapper",
    "name": "MT202 to pacs.009.001.11 Mapper",
    "tasks": [
        {
            "id": "parse_mt202",
            "name": "Parse MT202 Message", 
            "function": {
                "name": "parse",
                "input": {
                    "format": "SwiftMT",
                    "input_field_name": "payload",
                    "output_field_name": "SwiftMT"
                }
            }
        },
        // ... mapping tasks specific to MT202 → pacs.009
    ]
}
"#;
```

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
- **dataflow-rs** (0.1.8): Workflow processing engine
- **swift-mt-message** (0.1.1): SWIFT MT message parsing
- **mx-message** (0.1.1): ISO 20022 message structures

## License

This project is licensed under the Apache License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request
