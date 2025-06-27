# Reframe: Open-Source Bidirectional SWIFT MT ↔ ISO 20022 Transformation

**Reframe is an enterprise-grade, high-performance REST API that seamlessly converts between legacy SWIFT MT messages and modern ISO 20022 XML format in both directions. Built on a foundation of transparency and open-source principles, Reframe empowers financial institutions to accelerate their transition to CBPR+ with confidence and maintain backward compatibility.**

[![CI/CD Pipeline](https://github.com/GoPlasmatic/Reframe/actions/workflows/deploy-azure.yml/badge.svg?branch=main)](https://github.com/GoPlasmatic/Reframe/actions/workflows/deploy-azure.yml)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

**Live Demo**: [http://reframe-api-prod.eastus.azurecontainer.io:3000](http://reframe-api-prod.eastus.azurecontainer.io:3000)

---

## Why Reframe? The Value for Your Business

In an era of evolving payment standards, Reframe offers a strategic advantage by simplifying the complexities of ISO 20022 migration and providing full bidirectional transformation capabilities.

*   ✅ **Accelerate CBPR+ Compliance**: Effortlessly transform SWIFT MT messages to the ISO 20022 standard, ensuring you meet regulatory deadlines and stay ahead in the market.
*   🔄 **Full Bidirectional Support**: Convert both ways - MT to ISO 20022 AND ISO 20022 back to MT - providing complete flexibility for legacy system integration.
*   🤝 **Full Transparency, Zero Black Boxes**: As an open-source solution, Reframe provides complete visibility into its conversion logic. The transformation rules are defined in simple JSON, allowing for easy auditing, customization, and trust.
*   ⚙️ **Reduce Operational Risk**: Our robust, schema-validated engine minimizes the risk of manual errors and ensures the integrity of your payment messages in both directions.
*   🚀 **Boost Efficiency**: Built in Rust, Reframe is designed for high-throughput, low-latency processing, handling your message volumes with ease.
*   🌐 **Comprehensive Message Coverage**: Full support for the entire lifecycle of **MT103, MT202, and MT205** messages, including normal payments, cover payments, rejections, returns, and **complete cancellation & investigation workflows**, plus **cash management messages MT900/MT910**.

---

## A Modern, Transparent Technology Stack

Reframe combines cutting-edge technology with a commitment to openness, delivering a powerful and maintainable solution.

*   **Core Engine**: A high-performance Rust application using the Axum framework provides a robust and scalable API.
*   **Transparent Workflow Engine**: Powered by `dataflow-rs`, Reframe's logic is not hidden in compiled code. It's defined in external JSON files, making the transformation process transparent and easily adaptable.
*   **Bidirectional Processing**: Separate workflow engines for forward (MT→ISO 20022) and reverse (ISO 20022→MT) transformations with intelligent message type detection.
*   **Integrated Web UI**: A modern React-based interface for easy testing, demonstration, and manual conversions in both directions.
*   **Containerized & Cloud-Ready**: Shipped as a single Docker container, ready for deployment on-premises or in the cloud.

---

## Streamlined Maintenance and Operations

We designed Reframe to be as simple to operate as it is powerful.

*   **Simple Deployment**: The entire application—API and web UI—is packaged into a single container. Run it with a single `docker run` command.
*   **Maintain with Ease**: Need to tweak the mapping for a specific field? Simply update a JSON file. No need to recompile or redeploy the entire application.
*   **Automated CI/CD**: A production-ready GitHub Actions pipeline is included for automated testing, building, and deployment to Azure.
*   **Built-in Monitoring**: A `/health` endpoint provides simple, effective monitoring for integration with your existing infrastructure.

---

## Supported Transformations

Reframe offers complete, production-ready bidirectional support for the following message types:

### Forward Transformations (SWIFT MT → ISO 20022)

| SWIFT Message Family | ISO 20022 Format | Scenarios Supported |
|----------------------|------------------|---------------------|
| **MT103** | `pacs.008`, `pacs.002`, `pacs.004` | Normal, STP, Rejection, Return |
| **MT202** | `pacs.009`, `pacs.002`, `pacs.004` | Normal, Cover, Rejection, Return |
| **MT205** | `pacs.009`, `pacs.002`, `pacs.004` | Normal, Cover, Rejection, Return |
| **MT192** | `camt.056` | Request for Cancellation (Customer Credit Transfer) |
| **MT292** | `camt.056` | Request for Cancellation (Financial Institution Transfer) |
| **MT196** | `camt.056` | Answer to Request for Cancellation (Customer Transfer) |
| **MT296** | `camt.056` | Answer to Request for Cancellation (Financial Institution Transfer) |
| **MT900** | `camt.054` | Confirmation of Debit |
| **MT910** | `camt.054` | Confirmation of Credit |

### Reverse Transformations (ISO 20022 → SWIFT MT) ✨ **NEW**

| ISO 20022 Format | SWIFT Message | Scenarios Supported |
|------------------|---------------|---------------------|
| **pacs.008** | `MT103` | Customer Credit Transfer |
| *(More reverse transformations coming soon)* |

### **Complete Payment & Cancellation Ecosystem**
- **18+ Message Scenarios**: Comprehensive coverage across payment processing, cancellation workflows, and cash management
- **6 ISO 20022 Schemas**: `pacs.008`, `pacs.009`, `pacs.002`, `pacs.004`, `camt.056`, `camt.054`
- **End-to-End Lifecycle**: From payment initiation through cancellation, investigation, and cash reporting
- **UETR Support**: Full Unique End-to-End Transaction Reference integration for cancellation workflows
- **Bidirectional Processing**: Forward and reverse transformation capabilities for maximum flexibility

---

## Getting Started

### Quick Start

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/Plasmatic/Reframe.git
    cd Reframe
    ```
2.  **Build and run with Docker:**
    ```bash
    docker build -t reframe .
    docker run -p 3000:3000 reframe
    ```
3.  **Access the application** at `http://localhost:3000`.

### API Usage

Convert any supported message with a simple POST request. Reframe automatically detects the message type and transformation direction, applying the correct transformation.

**POST** `/reframe`

#### Forward Transformation (MT → ISO 20022)
```bash
curl -X POST http://localhost:3000/reframe \
  -H "Content-Type: text/plain" \
  --data-binary @path/to/your/mt_message.txt
```

**Example: MT103 to pacs.008**
```bash
curl -X POST http://localhost:3000/reframe \
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

#### Reverse Transformation (ISO 20022 → MT) ✨ **NEW**
```bash
curl -X POST http://localhost:3000/reframe \
  -H "Content-Type: application/xml" \
  --data-binary @path/to/your/pacs008_message.xml
```

**Example: pacs.008 to MT103**
```bash
curl -X POST http://localhost:3000/reframe \
  -H "Content-Type: application/xml" \
  -d '<?xml version="1.0" encoding="UTF-8"?>
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:pacs.008.001.08">
  <FIToFICstmrCdtTrf>
    <GrpHdr>
      <MsgId>MSG123456789</MsgId>
    </GrpHdr>
  </FIToFICstmrCdtTrf>
</Document>'
```

**Example: MT192 Cancellation Request to camt.056**
```bash
curl -X POST http://localhost:3000/reframe \
  -H "Content-Type: text/plain" \
  -d "{1:F01BANKGB2LXXX0000000000}{2:O1921234240101BANKUS33XXXX12345678952401011234N}{3:{121:12345678-1234-4567-8901-123456789012}}{4:
:20:CANC123456789
:21:FT21001234567890
:11S:1922024101
:79:DUPLICATE PAYMENT DETECTED
CUSTOMER REQUEST FOR CANCELLATION
PLEASE PROCESS IMMEDIATELY
-}"
```

---

## Open Source and Contributing

Reframe is an open-source project licensed under the Apache 2.0 License. We believe in the power of community and welcome contributions. Please feel free to open issues or submit pull requests.

1.  Fork the Project
2.  Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3.  Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4.  Push to the Branch (`git push origin feature/AmazingFeature`)
5.  Open a Pull Request

## Features

- **Bidirectional SWIFT Message Processing**: Supports both MT→ISO 20022 and ISO 20022→MT transformations
- **Comprehensive Message Support**: MT103, MT202, MT205, MT900, MT910, MT192, MT292, MT196, MT296 message types
- **Multiple Processing Methods**: Handles normal, reject, return, STP, and cover payment flows
- **XML/MT Conversion**: Converts between SWIFT MT and industry-standard XML formats in both directions
- **Intelligent Workflow Engine**: Configurable message processing pipelines via JSON workflows with automatic direction detection
- **Web Interface**: User-friendly web UI for bidirectional message processing
- **REST API**: HTTP endpoints for programmatic integration with automatic format detection
- **Structured Logging**: Comprehensive tracing and observability features

## Architecture

### Core Components

- **Parser Modules**: 
  - `src/parse_mt.rs`: SWIFT MT message parsing and validation
  - `src/parse_mx.rs`: ISO 20022 MX message parsing and validation
- **Publisher Modules**: 
  - `src/publish_mt.rs`: SWIFT MT generation and serialization
  - `src/publish_mx.rs`: ISO 20022 XML conversion and serialization  
- **Bidirectional Workflow Engine**: Separate forward and reverse transformation pipelines
- **Web Server**: Axum-based HTTP server with static file serving and intelligent content-type detection

### Message Flow

#### Forward Transformation (MT → ISO 20022)
1. **Input**: Raw SWIFT MT message received via API
2. **Parse**: MT message type detection and structured parsing
3. **Process**: Forward workflow-based transformation and validation
4. **Publish**: ISO 20022 XML serialization and output generation
5. **Response**: Structured JSON response with XML results

#### Reverse Transformation (ISO 20022 → MT) ✨ **NEW**
1. **Input**: ISO 20022 XML message received via API
2. **Parse**: MX message type detection and structured parsing
3. **Process**: Reverse workflow-based transformation and validation
4. **Publish**: SWIFT MT serialization and output generation
5. **Response**: Structured JSON response with MT results

## Logging and Observability

Reframe includes comprehensive structured logging and tracing capabilities for both transformation directions:

### Tracing Configuration

- **File Logging**: Daily rotating log files in `logs/reframe.log`
- **Console Output**: Structured logs to stdout/stderr
- **Environment Control**: Configure via `RUST_LOG` environment variable
- **Span Tracking**: Request correlation and timing information
- **Bidirectional Tracking**: Separate tracking for forward and reverse transformations

### Log Levels

```bash
# Debug level for all components
export RUST_LOG=debug

# Info level with debug for specific modules
export RUST_LOG=info,reframe=debug,dataflow_rs=debug

# Production settings
export RUST_LOG=info
```

### Observability Features

- **Request Tracing**: Every API request gets a unique span with transformation direction
- **Message Type Tracking**: Both SWIFT MT and ISO 20022 MX message types logged
- **Error Context**: Detailed error paths for debugging both directions
- **Performance Metrics**: Processing times and payload sizes for all transformations
- **Workflow Monitoring**: Step-by-step workflow execution tracking for forward and reverse flows

### Log Structure

Logs include contextual information:
- **Timestamp**: ISO 8601 format
- **Level**: DEBUG, INFO, WARN, ERROR
- **Target**: Module/component name
- **Span**: Request correlation ID
- **Direction**: Forward (MT→MX) or Reverse (MX→MT)
- **Fields**: Structured key-value data

Example log entry:
```bash
2024-01-15T10:30:45.123Z INFO reframe::parse_mt{message_type="103" method="normal" direction="forward"}: Message parsing completed successfully
```

## Quick Start

### Prerequisites

- Rust 1.70+ 
- Cargo

### Installation

```bash
git clone <repository-url>
cd Reframe
cargo build --release
```

### Configuration

1. **Forward Workflow Setup**: Place forward workflow JSON files in `workflows/forward/` directory
2. **Reverse Workflow Setup**: Place reverse workflow JSON files in `workflows/reverse/` directory
3. **Index Configuration**: Create `workflows/forward/index.json` and `workflows/reverse/index.json` to define loading order
4. **Environment Variables**: Set `RUST_LOG` for logging control

### Running

```bash
# Development with debug logging
RUST_LOG=debug cargo run

# Production
RUST_LOG=info ./target/release/Reframe
```

### API Usage

**Health Check**:
```bash
curl http://localhost:3000/health
```

**Process SWIFT MT Message (Forward)**:
```bash
curl -X POST http://localhost:3000/reframe \
  -H "Content-Type: text/plain" \
  -d "{1:F01BANKUS33AXXX0000000000}{2:O1031234567890123456789012345678901234567890}{4:
:20:REFERENCE123
:23B:CRED
:32A:240115USD1000,00
:50K:/1234567890
SENDER NAME
SENDER ADDRESS
:59:/0987654321
BENEFICIARY NAME
BENEFICIARY ADDRESS
:71A:OUR
-}"
```

**Process ISO 20022 Message (Reverse)**:
```bash
curl -X POST http://localhost:3000/reframe \
  -H "Content-Type: application/xml" \
  -d '<?xml version="1.0" encoding="UTF-8"?>
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:pacs.008.001.08">
  <FIToFICstmrCdtTrf>
    <GrpHdr>
      <MsgId>MSG123456789</MsgId>
    </GrpHdr>
  </FIToFICstmrCdtTrf>
</Document>'
```

**Web Interface**:
Navigate to `http://localhost:3000/` for the web UI with bidirectional transformation support.

## Supported Message Types

### Forward Transformations (MT → ISO 20022)

| Message Type | Description | Processing Methods | Target Schema |
|--------------|-------------|-------------------|---------------|
| MT103 | Single Customer Credit Transfer | Normal, STP, Reject, Return | pacs.008, pacs.002, pacs.004 |
| MT202 | General Financial Institution Transfer | Normal, Cover, Reject, Return | pacs.009, pacs.002, pacs.004 |
| MT205 | Corporate Trade | Normal, Cover, Reject, Return | pacs.009, pacs.002, pacs.004 |
| MT900 | Confirmation of Debit | Normal | camt.054 |
| MT910 | Confirmation of Credit | Normal | camt.054 |
| MT192 | Request for Cancellation (Customer) | Normal | camt.056 |
| MT292 | Request for Cancellation (FI) | Normal | camt.056 |
| MT196 | Client Notification (Customer) | Normal | camt.056 |
| MT296 | Client Notification (FI) | Normal | camt.056 |

### Reverse Transformations (ISO 20022 → MT) ✨ **NEW**

| ISO 20022 Schema | Description | Target MT | Processing Methods |
|------------------|-------------|-----------|-------------------|
| pacs.008 | Customer Credit Transfer | MT103 | Normal |
| *(Additional reverse transformations in development)* |

## Workflow Configuration

### Forward Workflow Structure

```json
{
  "workflows": [
    {
      "path": "parse-mt.json"
    },
    {
      "path": "MT103/bah-mapping.json"  
    },
    {
      "path": "MT103/document-mapping.json"  
    }
  ]
}
```

### Reverse Workflow Structure ✨ **NEW**

```json
{
  "workflows": [
    {
      "path": "parse-mx.json"
    },
    {
      "path": "pacs008/field-mapping.json"  
    }
  ]
}
```

### Individual Workflow

```json
{
  "id": "mt103-processor",
  "name": "MT103 Processing Workflow",
  "tasks": [
    {
      "id": "parse",
      "type": "parse",
      "config": {
        "format": "SwiftMT",
        "input_field_name": "payload",
        "output_field_name": "parsed_data"
      }
    },
    {
      "id": "publish",
      "type": "publish", 
      "config": {
        "source_format": "MT103.Document",
        "input_field_name": "parsed_data",
        "output_field_name": "result"
      }
    }
  ]
}
```

## Error Handling

Reframe provides detailed error information for both transformation directions:

- **Parse Errors**: Invalid SWIFT MT or ISO 20022 message format or unsupported fields
- **Workflow Errors**: Missing workflows or configuration issues for forward/reverse processing
- **Conversion Errors**: XML/MT serialization failures with detailed paths
- **Validation Errors**: Business rule violations for both message types

All errors include:
- Error message and type
- Transformation direction (forward/reverse)
- Contextual information (message type, field paths)
- Debug data for troubleshooting
- Structured logging for monitoring

## Performance

- **Async Processing**: Tokio-based async runtime for both directions
- **Memory Efficiency**: Streaming JSON and XML processing
- **Error Recovery**: Graceful handling of malformed messages in both formats
- **Monitoring**: Built-in metrics and tracing for bidirectional transformations

## Development

### Running Tests

```bash
# Unit tests
cargo test

# Integration tests with logging
RUST_LOG=debug cargo test -- --nocapture
```

### Building for Production

```bash
cargo build --release
```

### Code Structure

```
src/
├── main.rs          # Server initialization and routing
├── parse_mt.rs      # SWIFT MT message parsing logic
├── parse_mx.rs      # ISO 20022 MX message parsing logic
├── publish_mt.rs    # SWIFT MT generation and serialization
└── publish_mx.rs    # ISO 20022 XML conversion and serialization

workflows/           # Message processing workflows
├── forward/         # MT → ISO 20022 transformation workflows
│   ├── index.json   # Forward workflow loading configuration
│   ├── MT103/       # MT103 processing workflows
│   ├── MT202/       # MT202 processing workflows
│   └── ...
└── reverse/         # ISO 20022 → MT transformation workflows ✨ NEW
    ├── index.json   # Reverse workflow loading configuration
    ├── parse-mx.json # ISO 20022 message parser
    └── pacs008/     # pacs.008 to MT103 mapping

static/             # Web UI assets
├── index.html      # Main web interface with bidirectional support
├── assets/         # CSS, JS, and other static files
└── ...

logs/               # Application log files (auto-created)
└── reframe.log.*   # Daily rotating log files
```

## Monitoring and Operations

### Log Files

- **Location**: `logs/reframe.log.YYYY-MM-DD`
- **Rotation**: Daily automatic rotation
- **Format**: Structured JSON-like format with timestamps
- **Retention**: Manual cleanup required

### Health Monitoring

The `/health` endpoint provides service status:

```json
{
  "status": "healthy",
  "service": "reframe-api", 
  "version": "1.6.0",
  "features": ["bidirectional_transformation", "forward_mt_to_mx", "reverse_mx_to_mt"]
}
```

### Performance Monitoring

Key metrics logged include:
- Request processing time (forward and reverse)
- Message payload size for both MT and MX formats
- Conversion success/failure rates by direction
- Memory usage patterns
- Workflow execution times for both transformation directions

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add comprehensive tests for both transformation directions
4. Ensure all tests pass
5. Add logging/tracing for new features
6. Update documentation for bidirectional capabilities
7. Submit a pull request

## License

[License information]
