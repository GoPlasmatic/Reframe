# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Reframe** is an enterprise-grade, open-source bidirectional SWIFT MT ↔ ISO 20022 transformation service built in Rust. It provides REST API endpoints for converting between legacy SWIFT MT messages and modern ISO 20022 XML format in both directions.

## Development Commands

### Building and Running

```bash
# Build in debug mode (faster compilation, with debug symbols)
cargo build

# Build in release mode (optimized for performance)
cargo build --release

# Run the application
cargo run

# Run with debug logging to see detailed transformation steps
RUST_LOG=debug cargo run

# Run with info logging (recommended for development)
RUST_LOG=info cargo run

# Kill existing process on port 3000 and restart
lsof -i :3000 | grep LISTEN | awk '{print $2}' | xargs kill -9 2>/dev/null; RUST_LOG=info cargo run

# Run with custom Tokio thread count
TOKIO_WORKER_THREADS=8 cargo run --release

# Run benchmark to find optimal configuration
python3 test/simple_benchmark.py
```

### Performance Configuration

Reframe uses an async Engine with Tokio runtime for high-performance message processing. Performance settings can be configured via `reframe.config.json` or environment variables.

#### Runtime Architecture

- **Async I/O**: Non-blocking operations for network and file handling
- **CPU optimization**: Tokio worker threads configurable (defaults to all CPU cores)
- **Efficient concurrency**: Handles thousands of concurrent requests efficiently
- **Memory efficiency**: Async operations reduce memory overhead

#### Configuration Methods

Performance can be configured in three ways (priority order):

1. **Environment Variables** (highest priority)
   ```bash
   TOKIO_WORKER_THREADS=8 cargo run
   ```

2. **Configuration File** (`reframe.config.json`)
   ```json
   {
     "server": {
       "runtime": {
         "tokio_worker_threads": "8"
       }
     }
   }
   ```

3. **Auto-detection** (fallback)
   - Set `tokio_worker_threads: "auto"` in config
   - Automatically uses all available CPU cores

#### Performance Examples

```bash
# Use configuration from reframe.config.json
cargo run --release

# Override with environment variable
TOKIO_WORKER_THREADS=4 cargo run --release

# Conservative (low resources)
TOKIO_WORKER_THREADS=1 cargo run

# High performance (specific core count)
TOKIO_WORKER_THREADS=16 cargo run --release
```

#### Runtime Characteristics

- **Automatic scaling**: Configurable to match available resources
- **Efficient concurrency**: Handles multiple concurrent transformations
- **Resource awareness**: Can be tuned for resource-constrained environments
- **Hot-reload**: Workflow changes reload without restart

### Testing

```bash
# Run all tests
cargo test

# Run tests with output visible
cargo test -- --nocapture

# Run tests with debug logging
RUST_LOG=debug cargo test -- --nocapture

# Run a specific test
cargo test test_name -- --nocapture

# Test scenario generation for specific message types
python3 test/test_scenarios.py -m MT103 -d
python3 test/test_scenarios.py -m pacs.008 -d
python3 test/test_scenarios.py -m camt.052 -d

# Test all scenarios with verbose output
python3 test/test_scenarios.py --all -v

# Generate sample messages
python3 test/generate_sample.py MT103 -s standard
python3 test/generate_sample.py pacs.008 -s cbpr_standard -p

# Validate sample messages
python3 test/validate_sample.py MT103 -s standard -d
```

### Code Quality

```bash
# Format code
cargo fmt

# Check formatting without changes
cargo fmt -- --check

# Run clippy linter
cargo clippy

# Run clippy with all warnings as errors
cargo clippy -- -D warnings
```

### Docker Operations

```bash
# Build container (downloads package automatically)
docker build -t reframe .

# Build with specific package URL
docker build --build-arg PACKAGE_URL=https://github.com/GoPlasmatic/reframe-package-swift-cbpr/releases/download/v2.1.1/reframe-swift-cbpr-v2.1.1.zip -t reframe .

# Run container (package is baked into image)
docker run -p 3000:3000 reframe

# Build and run with docker-compose (recommended)
docker-compose up -d
```

## Architecture and Code Structure

### Core Components

The application follows a unified package-based architecture with clear separation of concerns:

1. **Main Server** (`src/main.rs`):
   - Axum HTTP server setup
   - Route configuration with unified API endpoints
   - Engine initialization from external packages
   - Package-based workflow loading

2. **Transformation Engines** (`src/engine.rs`):
   - **Transform Engine**: Unified bidirectional MT ↔ ISO 20022 transformations
   - **Generation Engine**: Sample message generation for both MT and MX
   - **Validation Engine**: Message validation for both MT and MX
   - All engines use dataflow-rs for workflow orchestration
   - Engines are initialized once from package configuration and reused across requests
   - Package loading system supports `REFRAME_PACKAGE_PATH` environment variable

3. **Message Parsing**:
   - `src/parse_mt.rs`: SWIFT MT parsing with custom parser
   - `src/parse_mx.rs`: ISO 20022 XML parsing and validation
   - `src/validation_helpers.rs`: Common validation utilities

4. **Message Generation**:
   - `src/mx_generator.rs`: Converts JSON to ISO 20022 XML using mx-message library
   - `src/mt_generator.rs`: Generates SWIFT MT messages from structured data
   - `src/sample_generator.rs`: Creates sample messages using datafake-rs
   - `src/scenario_loader.rs`: Loads and manages test scenarios

5. **Message Publishing**:
   - `src/publish_mt.rs`: Formats and publishes MT messages
   - `src/publish_mx.rs`: Formats and publishes MX messages

6. **API Handlers**:
   - `src/handlers.rs`: Sample generation, health check, and workflow reload handlers
   - `src/handlers_unified.rs`: Unified transformation and validation handlers
   - Request validation and routing
   - Engine invocation with unified architecture
   - Response formatting and error handling

7. **OpenAPI Documentation** (`src/openapi.rs`):
   - Swagger/OpenAPI spec generation using utoipa
   - API documentation endpoints

8. **Type Definitions** (`src/types.rs`):
   - Common data structures and types
   - Request/response models with unified AppState (3 engines)

9. **Helper Functions** (`src/helper.rs`):
   - Utility functions used across the codebase

### Package-Based Workflow System

Workflows are now loaded from external packages (default: `../reframe-package-swift-cbpr/`):

```
reframe-package-swift-cbpr/
├── reframe-package.json    # Package configuration
├── transform/
│   ├── index.json          # Unified bidirectional workflow index
│   ├── outgoing/           # MT → MX transformations
│   │   ├── parse-mt.json   # Common MT parsing
│   │   ├── MT103/          # Message-specific workflows
│   │   │   ├── bah-mapping.json
│   │   │   ├── document-mapping.json
│   │   │   ├── precondition.json
│   │   │   └── postcondition.json
│   │   └── combine-xml.json
│   └── incoming/           # MX → MT transformations
│       ├── parse-mx.json
│       └── pacs008/
│           ├── 01-variant-detection.json
│           ├── 02-preconditions.json
│           └── ...
├── generate/
│   ├── index.json
│   ├── generate-mt.json
│   └── generate-mx.json
├── validate/
│   ├── index.json
│   ├── validate-mt.json
│   └── validate-mx.json
└── scenarios/
    ├── index.json           # Scenario registry
    ├── outgoing/            # MT → MX test scenarios
    │   └── mt103_to_pacs008_cbpr_standard.json
    └── incoming/            # MX → MT test scenarios
        ├── camt052_to_mt942_cbpr.json
        └── pacs008_to_mt103_cbpr_standard.json
```

#### Package Configuration (`reframe-package.json`)

```json
{
  "id": "swift-cbpr-mt-mx",
  "name": "SWIFT CBPR+ MT <-> MX Transformations - SR2025",
  "version": "2.0.0",
  "workflows": {
    "transform": {
      "path": "transform",
      "description": "Unified bidirectional transformation workflows",
      "entry_point": "index.json"
    },
    "generate": {
      "path": "generate",
      "description": "Sample message generation",
      "entry_point": "index.json"
    },
    "validate": {
      "path": "validate",
      "description": "Message validation workflows",
      "entry_point": "index.json"
    }
  },
  "scenarios": {
    "path": "scenarios",
    "description": "Test scenarios",
    "entry_point": "index.json"
  }
}
```

Each scenario file contains:
- `variables`: Reusable values (BICs, amounts, etc.)
- `schema`: Message structure with datafake generators

### Key Libraries and Dependencies

- **mx-message** (3.0): ISO 20022 message structures and serialization
- **swift-mt-message** (3.0): SWIFT MT message handling  
- **dataflow-rs** (1.0): Workflow engine for transformation pipelines
- **datafake-rs** (0.1): Test data generation from JSON schemas
- **axum** (0.8): Async web framework
- **tokio** (1.47): Async runtime
- **quick-xml** (0.38): XML serialization
- **utoipa** (5.4): OpenAPI documentation generation

## API Endpoints

### Core Transformation API (Unified)

- `POST /api/transform`: Unified bidirectional transformation endpoint
  - Automatically detects message format (MT or MX) from content
  - Supports optional `message_type_hint` field for routing optimization
  - Handles both MT → MX and MX → MT transformations
  - Request body:
    ```json
    {
      "message": "<MT or MX message content>",
      "message_type_hint": "MT103",  // Optional
      "options": {
        "validation": true,
        "debug": false
      }
    }
    ```

### Sample Generation

- `POST /api/generate`: Generate sample messages for testing
  - Automatically detects MT vs MX message types
  - Uses scenario files from package for realistic data
  - Request body:
    ```json
    {
      "message_type": "MT103",
      "scenario": "standard",
      "options": { "debug": false }
    }
    ```

### Validation (Unified)

- `POST /api/validate`: Unified validation endpoint
  - Automatically detects and validates MT or MX messages
  - Returns validation status and errors
  - Request body:
    ```json
    {
      "message": "<MT or MX message content>",
      "options": {
        "canonical": false,
        "business_validation": false
      }
    }
    ```

### Administration

- `POST /admin/reload-workflows`: Hot reload workflow configurations from package
- `GET /health`: Health check with engine status (shows 3 engines: transform, generate, validate)
- `GET /swagger-ui`: Interactive API documentation
- `GET /api-docs/openapi.json`: OpenAPI specification

## Common Development Tasks

### Adding Support for a New Message Type

All workflows are now in the external package (`../reframe-package-swift-cbpr/`):

1. **For MT → MX transformation (outgoing)**:
   - Navigate to package: `cd ../reframe-package-swift-cbpr`
   - Create workflow directory: `transform/outgoing/MT{XXX}/`
   - Add workflow files: `bah-mapping.json`, `document-mapping.json`, `precondition.json`, `postcondition.json`
   - Update `transform/index.json` to include new workflows (with `outgoing/MT{XXX}/` prefix)
   - Add test scenario in `scenarios/outgoing/`

2. **For MX → MT transformation (incoming)**:
   - Navigate to package: `cd ../reframe-package-swift-cbpr`
   - Create workflow directory: `transform/incoming/{message_type}/`
   - Add numbered workflow files following existing patterns (01-variant-detection.json, etc.)
   - Update `transform/index.json` to include new workflows (with `incoming/{message_type}/` prefix)
   - Add test scenario in `scenarios/incoming/`

3. **Register scenarios**:
   - Update `scenarios/index.json` with new scenario mappings (use "outgoing"/"incoming" keys)
   - Reload workflows in Reframe: `POST /admin/reload-workflows`
   - Test with: `python3 test/test_scenarios.py -m {message_type} -d`

## Configuration

Reframe uses a centralized `reframe.config.json` configuration file with sensible defaults. All configuration can be overridden via environment variables.

### Configuration File (`reframe.config.json`)

The configuration file supports the following sections:

```json
{
  "version": "1.0",
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
      "tokio_worker_threads": "auto",
      "thread_name_prefix": "reframe-worker"
    }
  },
  "logging": {
    "format": "compact",
    "level": "info",
    "show_target": false,
    "show_thread": false,
    "show_file_location": false,
    "show_span_events": false,
    "file_output": {
      "enabled": false,
      "directory": "./logs",
      "file_prefix": "reframe",
      "rotation": "daily"
    }
  },
  "api_docs": {
    "enabled": true,
    "title": "Reframe API",
    "version": "3.1.1",
    "description": "Enterprise-grade bidirectional SWIFT MT ↔ ISO 20022 transformation service",
    "contact": {
      "name": "Plasmatic Team",
      "email": "enquires@goplasmatic.io"
    },
    "license": {
      "name": "Apache 2.0",
      "identifier": "Apache-2.0",
      "url": "https://opensource.org/license/apache-2-0"
    },
    "external_docs_url": "https://sandbox.goplasmatic.io",
    "server_url": "http://localhost:3000"
  },
  "defaults": {
    "package_id": null,
    "package_path": "../reframe-package-swift-cbpr"
  }
}
```

### Configuration Priority

Configuration values are resolved in this order (highest priority first):
1. **Environment Variables** - Override everything
2. **Configuration File** - `reframe.config.json`
3. **Built-in Defaults** - Hardcoded fallbacks

### Environment Variables

All configuration values can be overridden via environment variables:

#### Server Configuration
- **`REFRAME_HOST`**: Server bind address (default: from config or `0.0.0.0`)
- **`REFRAME_PORT`**: Server port (default: from config or `3000`)
- **`TOKIO_WORKER_THREADS`**: Number of Tokio async runtime worker threads (default: from config or CPU count)
- **`TOKIO_THREAD_NAME`**: Thread name prefix (default: from config or `reframe-worker`)

#### Package Configuration
- **`REFRAME_PACKAGE_PATH`**: Path to workflow package (default: from config or `../reframe-package-swift-cbpr`)
- **`REFRAME_DEFAULT_PACKAGE`**: Default package ID to use (optional)

#### Logging Configuration
- **`RUST_LOG`**: Log level (default: from config or `info`)
- **`LOG_FORMAT`**: Log format - `json`, `compact`, or `pretty` (default: from config or `compact`)

#### API Documentation
- **`API_SERVER_URL`**: Server URL for OpenAPI docs (default: from config or `http://localhost:3000`)

### Configuration Examples

#### Development Configuration
```json
{
  "server": {
    "host": "127.0.0.1",
    "port": 3000,
    "runtime": {
      "tokio_worker_threads": "auto"
    }
  },
  "logging": {
    "format": "pretty",
    "level": "debug",
    "show_file_location": true
  }
}
```

#### Production Configuration
```json
{
  "server": {
    "host": "0.0.0.0",
    "port": 8080,
    "runtime": {
      "tokio_worker_threads": "16"
    }
  },
  "logging": {
    "format": "json",
    "level": "info",
    "file_output": {
      "enabled": true,
      "directory": "/var/log/reframe",
      "rotation": "daily"
    }
  },
  "api_docs": {
    "server_url": "https://api.example.com"
  }
}
```

#### Testing Configuration
```json
{
  "server": {
    "port": 3001
  },
  "logging": {
    "format": "compact",
    "level": "debug"
  }
}
```

### Debugging Transformation Issues

1. Run with debug logging: `RUST_LOG=debug cargo run`
2. Check workflow execution in logs
3. Use debug option in API requests for detailed output
4. Test individual workflows with the test script
5. Use dataflow tracing: `RUST_LOG=debug,dataflow_rs=trace cargo run`

### Working with MX Message Library

When the mx-message library is updated:
1. Update version in Cargo.toml
2. Run `cargo update -p mx-message`
3. Fix any compilation errors (usually import path changes)
4. Test affected message types

## MX Message Scenario Generation Issues

Common issues when creating/fixing MX scenarios:

1. **Array vs String for Ustrd**: 
   ```json
   // Wrong
   "RmtInf": {"Ustrd": [{"fake": ["words", 3, 7]}]}
   // Correct
   "RmtInf": {"Ustrd": {"fake": ["words", 3, 7]}}
   ```

2. **Missing required fields in TxDtls**:
   ```json
   "TxDtls": {
       "Amt": {"@Ccy": {"var": "currency"}, "$value": {"fake": ["f64", 1000.0, 50000.0]}},
       "CdtDbtInd": "CRDT",  // Required!
       "AmtDtls": {...}
   }
   ```

3. **NtryDtls array structure** - ensure proper closing:
   ```json
   "NtryDtls": [{"TxDtls": {...}}]  // Note the closing ]
   ```

4. **String conversion for numbers** - use cat operator:
   ```json
   "NbOfNtries": {"cat": [{"var": "num_transactions"}]}
   ```

5. **Pagination fields**:
   - camt.052: Add `RptPgntn` to `Rpt`
   - camt.053: Add `StmtPgntn` to `Stmt`

## Important Implementation Notes

### Workflow Processing
- Date format in reverse mappings must be `yyyy-mm-dd` (parsed as NativeDate)
- Numeric path components need `#` prefix to avoid array notation interpretation
- `one_of` is not valid in dataflow-rs JSONLogic (use alternative logic)

### Engine Management
- The application maintains 3 unified engines (transform, generate, validate) that persist across requests
- Transform engine handles both outgoing (MT→MX) and incoming (MX→MT) transformations
- Workflow packages are loaded from external directory (configurable via `REFRAME_PACKAGE_PATH`)
- Workflow modifications can be hot-reloaded without restart via `/admin/reload-workflows`
- All transformations are logged with structured tracing

### Message Handling
- MT messages use custom parsing logic in `parse_mt.rs`
- MX messages are validated against ISO 20022 schemas
- Both directions support Business Application Header (BAH) generation
- Envelope structures are automatically handled in transformations

## Custom Fields

Reframe supports package-specific custom fields that are computed using JSONLogic expressions and exposed through the GraphQL API. This feature enables packages to add domain-specific business logic and derived fields without modifying the core Reframe codebase.

### Overview

- **Package-Specific**: Each package can define its own custom fields in `api_config.json`
- **JSONLogic-Based**: Fields are computed using JSONLogic expressions evaluated against message data
- **Flexible Storage**: Three storage strategies (precompute, runtime, hybrid)
- **GraphQL Integration**: Custom fields are queryable and filterable through GraphQL
- **Automatic Computation**: Fields are automatically computed and stored when messages are saved to the database

### Package Configuration

Create `api_config.json` in the package root directory:

```json
{
  "custom_fields": [
    {
      "name": "transaction_risk_score",
      "description": "Computed risk score based on amount",
      "type": "number",
      "storage": "precompute",
      "logic": {
        "if": [
          {">": [{"var": "context.data.amount"}, 100000]},
          100,
          {">": [{"var": "context.data.amount"}, 50000]},
          70,
          30
        ]
      }
    },
    {
      "name": "is_cross_border",
      "description": "Whether transaction crosses borders",
      "type": "boolean",
      "storage": "precompute",
      "logic": {
        "!=": [
          {"var": "context.data.debtor_country"},
          {"var": "context.data.creditor_country"}
        ]
      }
    }
  ]
}
```

### Storage Strategies

1. **precompute**: Computed at storage time, stored in DB (fast queries, best for filters/sorting)
2. **runtime**: Computed at query time only (not stored, best for time-dependent fields)
3. **hybrid**: Stored but can be recomputed with latest logic (flexible)

### GraphQL Queries

Custom fields are accessible through the GraphQL API:

```graphql
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
      package_id
      custom_fields  # JSON object with all custom fields
    }
    total_count
  }
}
```

### Filter Operators

Supported operators for custom field filters:
- `eq`, `ne`: Equality/inequality
- `gt`, `gte`, `lt`, `lte`: Comparison
- `in`: Value in array
- `exists`: Field exists

### Database Structure

Custom fields are stored in `context.custom_fields`:

```json
{
  "id": "msg-123",
  "context": {
    "metadata": {
      "package_id": "swift-cbpr-mt-mx"
    },
    "custom_fields": {
      "transaction_risk_score": 70,
      "is_cross_border": true
    }
  }
}
```

### Implementation Details

- **Module**: `src/custom_fields.rs` - Core JSONLogic evaluation
- **Package Loading**: `src/package_manager.rs` - Loads `api_config.json` during package discovery
- **Handler Integration**: All handlers (transform, validate, generate) automatically compute custom fields before database persistence
- **GraphQL Types**: `src/graphql/types.rs` - Adds `package_id` and `custom_fields` to TransformationMessage
- **MongoDB Filtering**: `src/database/mongodb.rs` - Implements custom field query filters

### Documentation

For complete documentation, see:
- **Full Guide**: `docs/custom-fields.md`
- **Example Configuration**: `examples/api_config.example.json`

## Testing Strategy

1. **Unit tests**: Test individual components
   ```bash
   cargo test
   cargo test -- --nocapture  # See println! output
   ```

2. **Scenario tests**: Test end-to-end with realistic data
   ```bash
   python3 test/test_scenarios.py --all
   python3 test/test_scenarios.py -m MT103 -d
   ```

3. **Manual testing**: Use curl or Postman
   ```bash
   # Unified transformation endpoint (auto-detects direction)
   curl -X POST http://localhost:3000/api/transform \
     -H "Content-Type: application/json" \
     -d '{"message": "{1:F01BANKBEBBAXXX...}", "options": {"debug": true}}'

   # With message type hint for optimization
   curl -X POST http://localhost:3000/api/transform \
     -H "Content-Type: application/json" \
     -d '{"message": "...", "message_type_hint": "MT103", "options": {"debug": false}}'
   ```

4. **Validation testing**: Ensure generated messages pass validation
   ```bash
   python3 test/validate_sample.py MT103 -s standard
   ```

## Performance Considerations

- Use release builds for performance testing (`cargo build --release`)
- The application is stateless and can scale horizontally
- Workflow engines are initialized once and reused
- JSON parsing is a potential bottleneck for large messages
- Consider using `RUST_LOG=info` in production (debug logging impacts performance)

## Docker Deployment

The Dockerfile uses a multi-stage build:
1. **Builder stage**: Compiles the Rust application with dependencies cached
2. **Runtime stage**: Minimal Debian image with only runtime dependencies
3. **Package Download**: Downloads and extracts workflow package from configurable URL during build
4. **Configuration**: Bakes reframe.config.json into the image with Docker-appropriate paths

Container runs as non-root user (appuser) on port 3000.

**Key Features**:
- Workflow packages downloaded automatically from GitHub releases or custom URLs
- No need to clone packages locally - fully self-contained build
- Package URL configurable via `PACKAGE_URL` build argument or docker-compose .env file
- Packages stored at `/packages/swift-cbpr` inside container

## Workflow-Specific Notes

- Any path field within mapping function (ex: "path": "data.SwiftMT.fields.25") with a pure numeric component without variant shall have # in front to represent the number as string key instead of array index notation. Correct: "path": "data.SwiftMT.fields.#25". This hash is not needed for fields with variant "path": "data.SwiftMT.fields.32A"
- Need to call reload workflow API for the workflow changes to take effect