# Reframe - SWIFT MT103 to ISO 20022 pacs.008.001.13 Converter

Reframe is a Rust-based REST API service that converts SWIFT MT103 messages to ISO 20022 pacs.008.001.13 (FIToFICustomerCreditTransferV13) XML format. Built with Rust, Axum, and the dataflow-rs workflow engine, it provides a high-performance solution for financial message transformation.

## Features

- 🚀 **High Performance**: Built with Rust and Axum for maximum throughput
- 🔄 **Message Transformation**: Converts SWIFT MT103 messages to ISO 20022 pacs.008.001.13 format
- 📋 **SWIFT MT Parsing**: Built-in SWIFT MT message parsing using swift-mt-message library
- 🌐 **REST API**: Simple HTTP endpoint for message conversion
- 🔒 **HTTPS Support**: Automated SSL termination with Azure Application Gateway
- ⚡ **Workflow Engine**: Powered by dataflow-rs for robust message processing
- 📊 **Error Handling**: Comprehensive error reporting for invalid messages
- 🔧 **Extensible**: Modular design allows for additional message formats
- 🚢 **Production Ready**: Complete CI/CD pipeline with Azure deployment

## Quick Start

### Web Interface
Access the live web interface at: **https://GoPlasmatic.github.io/Reframe**

The web interface provides:
- Material UI design with split-panel layout
- Sample MT103 message loader
- XML syntax highlighting
- Real-time error handling
- Secure HTTPS endpoint connection

### API Endpoints

#### Production (HTTPS)
- **HTTPS API**: `https://reframe-api-prod-https.eastus.cloudapp.azure.com/reframe`
- **Health Check**: `https://reframe-api-prod-https.eastus.cloudapp.azure.com/health`

### Local Development

1. Clone the repository:
```bash
git clone <repository-url>
cd Reframe
```

2. Build and run the application:
```bash
cargo run
```

3. The server will start on `http://0.0.0.0:3000`

## Deployment

### Automated Deployment

The project includes a complete CI/CD pipeline that automatically:

1. **Tests** the Rust code (format, clippy, unit tests)
2. **Builds** and pushes Docker images to Azure Container Registry
3. **Deploys** to staging environment for testing
4. **Deploys** to production environment
5. **Sets up HTTPS** infrastructure with Application Gateway
6. **Cleans up** staging resources

#### Triggering Deployment

- **Automatic**: Push to `main` branch
- **Manual**: Use GitHub Actions workflow dispatch

#### HTTPS Setup

HTTPS is automatically configured as part of the deployment pipeline using Azure Application Gateway with:
- SSL termination
- HTTP to HTTPS redirect
- Health probe monitoring
- Self-signed certificate (for testing)

#### Manual HTTPS Setup

If you need to setup HTTPS independently:

```bash
# Run the manual HTTPS deployment script
chmod +x scripts/deploy-https-manual.sh
./scripts/deploy-https-manual.sh
```

## Architecture

### Cloud Infrastructure

- **Azure Container Instances (ACI)**: Hosts the Rust API service
- **Azure Container Registry (ACR)**: Stores container images
- **Azure Application Gateway**: Provides HTTPS termination and load balancing
- **GitHub Actions**: CI/CD automation
- **GitHub Pages**: Hosts the web UI

### Components

1. **API Layer**: Axum-based REST server with CORS support
2. **Workflow Engine**: dataflow-rs engine orchestrating the conversion pipeline
3. **Parser Module**: Custom SWIFT MT message parser using swift-mt-message
4. **Publisher Module**: XML serialization using quick-xml and mx-message
5. **Mapping Engine**: JSONLogic-based field mapping from MT103 to pacs.008.001.13

### Message Flow

1. Client sends raw SWIFT MT103 message to `/reframe` endpoint
2. **Parse Task**: Parses SWIFT MT103 into structured data using swift-mt-message
3. **Mapping Tasks**: Series of mapping tasks that transform MT103 fields to pacs.008.001.13 structure:
   - Group Header mapping
   - Credit Transfer Transaction Information
   - Charge Bearer mapping
   - Instructing/Instructed Agent mapping
   - Debtor/Creditor information mapping
   - Remittance information mapping
   - Intermediary agent mapping
   - Regulatory reporting mapping
4. **Publish Task**: Serializes the mapped data to pacs.008.001.13 XML format
5. Returns XML response to client

## Quick Start

### Prerequisites

- Rust 1.70+ installed
- Cargo package manager

### Installation & Running

1. Clone the repository:
```bash
git clone <repository-url>
cd Reframe
```

2. Build and run the application:
```bash
cargo run
```

3. The server will start on `http://0.0.0.0:3000`

## API Reference

### Convert SWIFT MT103 to pacs.008.001.13
**POST** `/reframe`

Converts a SWIFT MT103 message to ISO 20022 pacs.008.001.13 XML format.

**Request:**
- **Content-Type**: `text/plain`
- **Body**: Raw SWIFT MT103 message

**Example Request:**
```bash
curl -X POST https://reframe-api-prod-https.eastus.cloudapp.azure.com/reframe \
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

**Response:**
- **Content-Type**: `application/xml`
- **Body**: ISO 20022 pacs.008.001.13 XML document

**Example Response:**
```xml
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:pacs.008.001.13">
  <FIToFICstmrCdtTrf>
    <GrpHdr>
      <MsgId>FT21001234567890</MsgId>
      <CreDtTm>2024-01-01T12:34:00Z</CreDtTm>
      <NbOfTxs>1</NbOfTxs>
      <CtrlSum>1000.00</CtrlSum>
      <InitgPty>
        <Nm>Reframe Processing System</Nm>
      </InitgPty>
      <SttlmInf>
        <SttlmMtd>CLRG</SttlmMtd>
      </SttlmInf>
    </GrpHdr>
    <CdtTrfTxInf>
      <PmtId>
        <InstrId>FT21001234567890</InstrId>
        <EndToEndId>FT21001234567890</EndToEndId>
        <UETR>FT21001234567890</UETR>
      </PmtId>
      <IntrBkSttlmAmt Ccy="USD">1000.00</IntrBkSttlmAmt>
      <IntrBkSttlmDt>2024-01-01</IntrBkSttlmDt>
      <ChrgBr>DEBT</ChrgBr>
      <InstgAgt>
        <FinInstnId>
          <BICFI>BNPAFRPPXXX</BICFI>
        </FinInstnId>
      </InstgAgt>
      <InstdAgt>
        <FinInstnId>
          <BICFI>DEUTDEFFXXX</BICFI>
        </FinInstnId>
      </InstdAgt>
      <Dbtr>
        <Nm>ACME CORPORATION</Nm>
      </Dbtr>
      <DbtrAcct>
        <Id>
          <IBAN>1234567890</IBAN>
        </Id>
      </DbtrAcct>
      <DbtrAgt>
        <FinInstnId>
          <BICFI>BNPAFRPPXXX</BICFI>
        </FinInstnId>
      </DbtrAgt>
      <Cdtr>
        <Nm>MUELLER GMBH</Nm>
      </Cdtr>
      <CdtrAcct>
        <Id>
          <IBAN>DE89370400440532013000</IBAN>
        </Id>
      </CdtrAcct>
      <CdtrAgt>
        <FinInstnId>
          <BICFI>DEUTDEFFXXX</BICFI>
        </FinInstnId>
      </CdtrAgt>
      <RmtInf>
        <Ustrd>PAYMENT FOR INVOICE 12345</Ustrd>
      </RmtInf>
    </CdtTrfTxInf>
  </FIToFICstmrCdtTrf>
</Document>
```

**Error Response:**
```json
{
  "error": "Error processing data: Validation(\"Missing required field: 20\")"
}
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
