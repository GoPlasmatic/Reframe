# Reframe: Open-Source SWIFT MT to ISO 20022 Transformation

**Reframe is an enterprise-grade, high-performance REST API that seamlessly converts legacy SWIFT MT messages into the modern ISO 20022 XML format. Built on a foundation of transparency and open-source principles, Reframe empowers financial institutions to accelerate their transition to CBPR+ with confidence.**

[![CI/CD Pipeline](https://github.com/GoPlasmatic/Reframe/actions/workflows/deploy-azure.yml/badge.svg?branch=main)](https://github.com/GoPlasmatic/Reframe/actions/workflows/deploy-azure.yml)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

**Live Demo**: [http://reframe-api-prod.eastus.azurecontainer.io:3000](http://reframe-api-prod.eastus.azurecontainer.io:3000)

---

## Why Reframe? The Value for Your Business

In an era of evolving payment standards, Reframe offers a strategic advantage by simplifying the complexities of ISO 20022 migration.

*   ✅ **Accelerate CBPR+ Compliance**: Effortlessly transform SWIFT MT messages to the ISO 20022 standard, ensuring you meet regulatory deadlines and stay ahead in the market.
*   🤝 **Full Transparency, Zero Black Boxes**: As an open-source solution, Reframe provides complete visibility into its conversion logic. The transformation rules are defined in simple JSON, allowing for easy auditing, customization, and trust.
*   ⚙️ **Reduce Operational Risk**: Our robust, schema-validated engine minimizes the risk of manual errors and ensures the integrity of your payment messages.
*   🚀 **Boost Efficiency**: Built in Rust, Reframe is designed for high-throughput, low-latency processing, handling your message volumes with ease.
*   🌐 **Comprehensive Message Coverage**: Full support for the entire lifecycle of **MT103, MT202, and MT205** messages, including normal payments, cover payments, rejections, returns, and **complete cancellation & investigation workflows**.

---

## A Modern, Transparent Technology Stack

Reframe combines cutting-edge technology with a commitment to openness, delivering a powerful and maintainable solution.

*   **Core Engine**: A high-performance Rust application using the Axum framework provides a robust and scalable API.
*   **Transparent Workflow Engine**: Powered by `dataflow-rs`, Reframe's logic is not hidden in compiled code. It's defined in external JSON files, making the transformation process transparent and easily adaptable.
*   **Integrated Web UI**: A modern React-based interface for easy testing, demonstration, and manual conversions.
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

Reframe offers complete, production-ready support for the following message types:

| SWIFT Message Family | ISO 20022 Format | Scenarios Supported |
|----------------------|------------------|---------------------|
| **MT103** | `pacs.008`, `pacs.002`, `pacs.004` | Normal, STP, Rejection, Return |
| **MT202** | `pacs.009`, `pacs.002`, `pacs.004` | Normal, Cover, Rejection, Return |
| **MT205** | `pacs.009`, `pacs.002`, `pacs.004` | Normal, Cover, Rejection, Return |
| **MT192** | `camt.056` | Request for Cancellation (Customer Credit Transfer) |
| **MT292** | `camt.056` | Request for Cancellation (Financial Institution Transfer) |
| **MT196** | `camt.056` | Answer to Request for Cancellation (Customer Transfer) |
| **MT296** | `camt.056` | Answer to Request for Cancellation (Financial Institution Transfer) |

### **Complete Payment & Cancellation Ecosystem**
- **16 Message Scenarios**: Comprehensive coverage across payment processing and cancellation workflows
- **5 ISO 20022 Schemas**: `pacs.008`, `pacs.009`, `pacs.002`, `pacs.004`, `camt.056`
- **End-to-End Lifecycle**: From payment initiation through cancellation and investigation
- **UETR Support**: Full Unique End-to-End Transaction Reference integration for cancellation workflows

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

Convert any supported message with a simple POST request. Reframe automatically detects the message type and applies the correct transformation.

**POST** `/reframe`

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

- **SWIFT Message Parsing**: Supports MT103, MT202, MT205, MT900, MT910, MT192, MT292, MT196, MT296 message types
- **Multiple Processing Methods**: Handles normal, reject, return, STP, and cover payment flows
- **XML Conversion**: Converts parsed SWIFT messages to industry-standard XML formats
- **Workflow Engine**: Configurable message processing pipelines via JSON workflows
- **Web Interface**: User-friendly web UI for message processing
- **REST API**: HTTP endpoints for programmatic integration
- **Structured Logging**: Comprehensive tracing and observability features

## Architecture

### Core Components

- **Parser Module** (`src/parser.rs`): SWIFT message parsing and validation
- **Publisher Module** (`src/publish.rs`): XML conversion and serialization  
- **Workflow Engine**: Dataflow-based message processing pipelines
- **Web Server**: Axum-based HTTP server with static file serving

### Message Flow

1. **Input**: Raw SWIFT message received via API
2. **Parse**: Message type detection and structured parsing
3. **Process**: Workflow-based transformation and validation
4. **Publish**: XML serialization and output generation
5. **Response**: Structured JSON response with results

## Logging and Observability

Reframe includes comprehensive structured logging and tracing capabilities:

### Tracing Configuration

- **File Logging**: Daily rotating log files in `logs/reframe.log`
- **Console Output**: Structured logs to stdout/stderr
- **Environment Control**: Configure via `RUST_LOG` environment variable
- **Span Tracking**: Request correlation and timing information

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

- **Request Tracing**: Every API request gets a unique span
- **Message Type Tracking**: SWIFT message type and processing method logged
- **Error Context**: Detailed error paths for debugging
- **Performance Metrics**: Processing times and payload sizes
- **Workflow Monitoring**: Step-by-step workflow execution tracking

### Log Structure

Logs include contextual information:
- **Timestamp**: ISO 8601 format
- **Level**: DEBUG, INFO, WARN, ERROR
- **Target**: Module/component name
- **Span**: Request correlation ID
- **Fields**: Structured key-value data

Example log entry:
```bash
2024-01-15T10:30:45.123Z INFO reframe::parser{message_type="103" method="normal"}: Message parsing completed successfully
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

1. **Workflow Setup**: Place workflow JSON files in `workflows/` directory
2. **Index Configuration**: Create `workflows/index.json` to define loading order
3. **Environment Variables**: Set `RUST_LOG` for logging control

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

**Process SWIFT Message**:
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

**Web Interface**:
Navigate to `http://localhost:3000/` for the web UI.

## Supported Message Types

| Message Type | Description | Processing Methods |
|--------------|-------------|-------------------|
| MT103 | Single Customer Credit Transfer | Normal, STP, Reject, Return |
| MT202 | General Financial Institution Transfer | Normal, Cover, Reject, Return |
| MT205 | Corporate Trade | Normal, Cover, Reject, Return |
| MT900 | Confirmation of Debit | Normal |
| MT910 | Confirmation of Credit | Normal |
| MT192 | Request for Cancellation | Normal |
| MT292 | Request for Cancellation | Normal |
| MT196 | Client Notification | Normal |
| MT296 | Client Notification | Normal |

## Workflow Configuration

### Example Workflow Structure

```json
{
  "workflows": [
    {
      "path": "MT103/workflow.json"
    },
    {
      "path": "MT202/workflow.json"  
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

Reframe provides detailed error information:

- **Parse Errors**: Invalid SWIFT message format or unsupported fields
- **Workflow Errors**: Missing workflows or configuration issues  
- **Conversion Errors**: XML serialization failures with detailed paths
- **Validation Errors**: Business rule violations

All errors include:
- Error message and type
- Contextual information (message type, field paths)
- Debug data for troubleshooting
- Structured logging for monitoring

## Performance

- **Async Processing**: Tokio-based async runtime
- **Memory Efficiency**: Streaming JSON processing
- **Error Recovery**: Graceful handling of malformed messages
- **Monitoring**: Built-in metrics and tracing

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
├── parser.rs        # SWIFT message parsing logic
└── publish.rs       # XML conversion and serialization

workflows/           # Message processing workflows
├── index.json      # Workflow loading configuration
├── MT103/          # MT103 processing workflows
├── MT202/          # MT202 processing workflows
└── ...

static/             # Web UI assets
├── index.html      # Main web interface
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
  "version": "1.5.5"
}
```

### Performance Monitoring

Key metrics logged include:
- Request processing time
- Message payload size
- Conversion success/failure rates
- Memory usage patterns
- Workflow execution times

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add comprehensive tests
4. Ensure all tests pass
5. Add logging/tracing for new features
6. Submit a pull request

## License

[License information]
