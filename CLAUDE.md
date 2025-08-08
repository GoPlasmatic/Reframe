# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Reframe** is an enterprise-grade, open-source bidirectional SWIFT MT ↔ ISO 20022 transformation service built in Rust. It provides REST API endpoints for converting between legacy SWIFT MT messages and modern ISO 20022 XML format in both directions.

## Development Commands

### Rust Backend Commands

```bash
# Build the project
cargo build

# Build for production
cargo build --release

# Run the application
cargo run

# Run with debug logging
RUST_LOG=debug cargo run

# Run with production logging
RUST_LOG=info cargo run

# Run tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run tests with debug logging
RUST_LOG=debug cargo test -- --nocapture

# Format code
cargo fmt

# Lint code
cargo clippy

# Check for linting errors
cargo clippy -- -D warnings
```

### Docker Commands

```bash
# Build container
docker build -t reframe .

# Run container
docker run -p 3000:3000 reframe

# Run container with volume (for development)
docker run -p 3000:3000 -v $(pwd)/workflows:/app/workflows reframe
```

## Architecture Overview

### Core Application Structure

- **Main Server** (`src/main.rs`): Axum-based HTTP server with dual-engine architecture
- **Bidirectional Processing**: Separate engines for forward (MT→MX) and reverse (MX→MT) transformations
- **Parser Modules**: 
  - `src/parse_mt.rs`: SWIFT MT message parsing and validation
  - `src/parse_mx.rs`: ISO 20022 MX message parsing and validation
- **Publisher Modules**: 
  - `src/publish_mx.rs`: ISO 20022 XML generation and serialization
  - `src/publish_mt.rs`: SWIFT MT generation and serialization
- **Helper Module** (`src/helper.rs`): Utility functions and shared logic

### Workflow Engine Architecture

The application uses `dataflow-rs` as the workflow engine with JSON-based configuration:

- **Forward Workflows** (`workflows/forward/`): MT → ISO 20022 transformations
- **Reverse Workflows** (`workflows/reverse/`): ISO 20022 → MT transformations
- **Index Files**: `index.json` files define workflow loading order
- **Message Type Specific**: Each message type (MT103, MT202, etc.) has dedicated workflow files

### Supported Message Types

**Forward Transformations (MT → ISO 20022):**
- MT103 (Customer Credit Transfer) → pacs.008, pacs.002, pacs.004
- MT202/MT205 (Financial Institution Transfer) → pacs.009, pacs.002, pacs.004
- MT900/MT910 (Confirmation messages) → camt.054
- MT192/MT292/MT196/MT296 (Cancellation/Investigation) → camt.056

**Reverse Transformations (ISO 20022 → MT):**
- pacs.008 → MT103
- pacs.009 → MT202/MT205
- pacs.004 → MT103RETN/MT202RETN/MT205RETN

### API Endpoints

- `GET /health` - Health check with engine status
- `POST /transform/mt-to-mx` - Forward transformation (MT to ISO 20022)
- `POST /transform/mx-to-mt` - Reverse transformation (ISO 20022 to MT)
- `POST /generate/mt-sample` - Generate sample MT messages from JSON configuration
- `POST /admin/reload-workflows` - Hot reload workflow configurations without restart

## Configuration and Workflow Management

### Hot Reload Workflows

The application supports hot reloading of workflow configurations without restarting:

```bash
# Reload all workflows from disk
curl -X POST http://localhost:3000/admin/reload-workflows

# The API returns timing and status information
{
  "success": true,
  "message": "Workflows reloaded successfully in 44ms",
  "timestamp": "2025-07-14T08:59:36.916060+00:00"
}
```

This feature enables:
- **Development productivity**: Test workflow changes immediately
- **Production updates**: Update transformation rules without downtime
- **A/B testing**: Quickly switch between different workflow configurations

### Adding New Message Types

1. Create workflow directories in `workflows/forward/[MESSAGE_TYPE]/` or `workflows/reverse/[MESSAGE_TYPE]/`
2. Add workflow files: `bah-mapping.json`, `document-mapping.json`, `precondition.json`
3. Update `workflows/forward/index.json` or `workflows/reverse/index.json` to include new workflows
4. Use the reload API to apply changes: `POST /admin/reload-workflows`
5. Test with sample messages in `test/data/` directory

### Sample Generation (Unified MT and MX Support)

The sample generation API now supports both SWIFT MT and ISO 20022 MX message generation using scenario-based templates. Both libraries provide pre-defined test scenarios for realistic message generation:

```bash
# Generate MT103 with default scenario
curl -X POST http://localhost:3000/generate/sample \
  -H "Content-Type: application/json" \
  -d '{
    "message_type": "MT103",
    "config": {
      "scenario": "standard"
    }
  }'

# Generate pacs.008 (ISO 20022) with specific scenario
curl -X POST http://localhost:3000/generate/sample \
  -H "Content-Type: application/json" \
  -d '{
    "message_type": "pacs.008",
    "config": {
      "scenario": "high_value"
    }
  }'
```

**Supported Message Types**:
- **MT Messages**: MT101, MT103, MT104, MT107, MT110, MT111, MT112, MT192, MT196, MT199, MT202, MT205, MT210, MT292, MT296, MT299, MT900, MT910, MT920, MT935, MT940, MT941, MT942, MT950
- **MX Messages**: pacs.002, pacs.003, pacs.004, pacs.008, pacs.009, camt.025, camt.029, camt.052, camt.053, camt.054, camt.056, camt.057, camt.060, pain.001, pain.002, pain.008

**Important Notes**: 
- The unified endpoint `/generate/sample` automatically detects MT vs MX message types
- MT messages are returned as SWIFT MT format strings
- MX messages are returned as JSON (XML serialization coming soon)
- Both libraries use JSON scenario files with datafake-rs for dynamic data generation

**Available Scenarios**: 
- MT scenarios located in `scenarios/SwiftMTMessage/[message_type]/`
- MX scenarios located in `scenarios/MXMessage/[message_type]/`
- Common scenarios include: `standard`, `high_value`, `cbpr_*` variants, `regulatory_compliant`
- See the full list in each message type's scenario directory

### Environment Variables

- `RUST_LOG`: Controls logging level (debug, info, warn, error)
- Standard Rust environment variables for compilation and runtime

### Testing

- **Unit Tests**: Run with `cargo test`
- **Sample Data**: Located in `test/data/` directory with various message types
- **Integration Testing**: API endpoints can be tested with curl

## Deployment

### Local Development

1. Run the Rust application: `cargo run`
2. Access the API endpoints at `http://localhost:3000`

### Production Docker Deployment

1. Build: `docker build -t reframe .`
2. Run: `docker run -p 3000:3000 reframe`
3. The container runs the API service

### Azure Deployment

- Automated CI/CD pipeline via GitHub Actions (`.github/workflows/deploy-azure.yml`)
- Deploys to Azure Container Instances
- Includes staging and production environments
- See `DEPLOYMENT.md` for detailed deployment instructions

## Important Development Notes

- The application uses dual engines - always ensure both forward and reverse engines are properly initialized
- **Hot Reload**: Workflow files can be reloaded at runtime using `POST /admin/reload-workflows` - no restart required
- **Legacy Note**: Previously required restart after modifying workflow configurations, now supports hot reload
- Message type detection is automatic based on content analysis
- All transformations are logged with structured tracing for debugging

## File Structure Key Points

- `src/`: Core Rust application code
- `workflows/`: JSON-based transformation workflows (forward and reverse)
- `test/data/`: Sample SWIFT MT and ISO 20022 messages for testing
- `specification/`: Message format specifications and mapping tables