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

Reframe uses an async Engine with Tokio runtime for high-performance message processing:
- **Async I/O**: Non-blocking operations for network and file handling
- **CPU optimization**: Tokio worker threads automatically utilize all available CPU cores
- **Efficient concurrency**: Handles thousands of concurrent requests efficiently

#### Environment Variables

- **`TOKIO_WORKER_THREADS`**: Number of Tokio async runtime worker threads (default: CPU count)
  - Controls the Tokio runtime thread pool size for async I/O and request handling
  - Defaults to all available CPU cores for maximum performance
  - Recommended: Leave unset for automatic CPU detection, or set explicitly for resource-constrained environments

#### Configuration Examples

```bash
# Default (uses all CPU cores)
cargo run --release

# Conservative (low resources, single-core VPS)
TOKIO_WORKER_THREADS=1 cargo run

# Balanced (4-core server)
TOKIO_WORKER_THREADS=4 cargo run --release

# High performance (8-core server)
TOKIO_WORKER_THREADS=8 cargo run --release

# Maximum throughput (16+ core servers)
TOKIO_WORKER_THREADS=16 cargo run --release
```

#### Performance Characteristics

- **Async operations**: Non-blocking I/O for efficient resource utilization
- **Automatic scaling**: Tokio runtime automatically scales with available CPU cores
- **Efficient concurrency**: Handles multiple concurrent transformations efficiently
- **Memory efficiency**: Async operations reduce memory overhead compared to thread-per-request models

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
# Build container
docker build -t reframe .

# Run container
docker run -p 3000:3000 reframe

# Run with local workflow mount for development
docker run -p 3000:3000 -v $(pwd)/workflows:/app/workflows reframe
```

## Architecture and Code Structure

### Core Components

The application follows a dual-engine architecture with clear separation of concerns:

1. **Main Server** (`src/main.rs`): 
   - Axum HTTP server setup
   - Route configuration
   - Engine initialization
   - Sets environment variables for scenario paths

2. **Transformation Engines** (`src/engine.rs`):
   - **Forward Engine**: MT → ISO 20022 transformations
   - **Reverse Engine**: ISO 20022 → MT transformations
   - Both engines use dataflow-rs for workflow orchestration
   - Engines are initialized once and reused across requests

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

6. **API Handlers** (`src/handlers.rs`):
   - Request validation and routing
   - Engine invocation
   - Response formatting
   - Error handling and reporting

7. **OpenAPI Documentation** (`src/openapi.rs`):
   - Swagger/OpenAPI spec generation using utoipa
   - API documentation endpoints

8. **Type Definitions** (`src/types.rs`):
   - Common data structures and types
   - Request/response models

9. **Helper Functions** (`src/helper.rs`):
   - Utility functions used across the codebase

### Workflow System

Workflows are JSON-based transformation rules processed by dataflow-rs:

```
workflows/
├── forward/           # MT → MX transformations
│   ├── index.json    # Workflow loading order
│   ├── parse-mt.json # Common MT parsing
│   ├── MT103/        # Message-specific workflows
│   │   ├── bah-mapping.json      # Business Application Header
│   │   ├── document-mapping.json # Document body mapping
│   │   ├── precondition.json     # Validation rules
│   │   └── postcondition.json    # Post-processing rules
│   └── combine-xml.json          # Final XML assembly
└── reverse/           # MX → MT transformations
    ├── index.json
    ├── parse-mx.json
    └── pacs008/      # Message-specific workflows
        ├── 01-variant-detection.json
        ├── 02-preconditions.json
        └── ...
```

### Scenario System

Scenarios provide test data generation using datafake-rs:

```
scenarios/
├── index.json         # Scenario registry
├── forward/          # MT → MX test scenarios
│   └── mt103_to_pacs008_cbpr_standard.json
└── reverse/          # MX → MT test scenarios
    ├── camt052_to_mt942_cbpr.json
    └── pacs008_to_mt103_cbpr_standard.json
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

### Core Transformation APIs

- `POST /transform/mt-to-mx`: Convert SWIFT MT to ISO 20022
- `POST /transform/mx-to-mt`: Convert ISO 20022 to SWIFT MT

### Sample Generation

- `POST /generate/sample`: Generate sample messages for testing
  - Automatically detects MT vs MX message types
  - Uses scenario files for realistic data

### Validation

- `POST /validate/mt`: Validate SWIFT MT messages
- `POST /validate/mx`: Validate ISO 20022 messages

### Administration

- `POST /admin/reload-workflows`: Hot reload workflow configurations
- `GET /health`: Health check with engine status

## Common Development Tasks

### Adding Support for a New Message Type

1. **For MT → MX transformation**:
   - Create workflow directory: `workflows/forward/MT{XXX}/`
   - Add workflow files: `bah-mapping.json`, `document-mapping.json`, `precondition.json`, `postcondition.json`
   - Update `workflows/forward/index.json`
   - Add test scenario in `scenarios/forward/`

2. **For MX → MT transformation**:
   - Create workflow directory: `workflows/reverse/{message_type}/`
   - Add numbered workflow files following existing patterns (01-variant-detection.json, etc.)
   - Update `workflows/reverse/index.json`
   - Add test scenario in `scenarios/reverse/`

3. **Register scenarios**:
   - Update `scenarios/index.json` with new scenario mappings
   - Test with: `python3 test/test_scenarios.py -m {message_type} -d`

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
- The condition field in workflow gets the entire metadata field as the context, so no need to add `metadata.` prefix in variable access
- Date format in reverse mappings must be `yyyy-mm-dd` (parsed as NativeDate)
- Numeric path components need `#` prefix to avoid array notation interpretation
- `one_of` is not valid in dataflow-rs JSONLogic (use alternative logic)

### Engine Management
- The application maintains separate forward and reverse engines that persist across requests
- Workflow modifications can be hot-reloaded without restart via `/admin/reload-workflows`
- All transformations are logged with structured tracing

### Message Handling
- MT messages use custom parsing logic in `parse_mt.rs`
- MX messages are validated against ISO 20022 schemas
- Both directions support Business Application Header (BAH) generation
- Envelope structures are automatically handled in transformations

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
   curl -X POST http://localhost:3000/transform/mt-to-mx \
     -H "Content-Type: application/json" \
     -d @test/data/mt103.json
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
3. **Scenario loading**: Clones SwiftMTMessage and MXMessage repos for test scenarios

Container runs as non-root user (appuser) on port 3000.

## Workflow-Specific Notes

- Any path field within mapping function (ex: "path": "data.SwiftMT.fields.25") with a pure numeric component without variant shall have # in front to represent the number as string key instead of array index notation. Correct: "path": "data.SwiftMT.fields.#25". This hash is not needed for fields with variant "path": "data.SwiftMT.fields.32A"
- Need to call reload workflow API for the workflow changes to take effect