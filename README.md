# Reframe - Workflow Processing API

Reframe is a Rust-based application that exposes a REST API for processing data through configurable workflows using the [dataflow-rs](https://docs.rs/dataflow-rs) library. The application allows you to send payload data via HTTP requests, process it through pre-configured workflows, and receive the transformed results.

## Features

- 🚀 **Fast & Lightweight**: Built with Rust and Axum for high performance
- 🔄 **Workflow Processing**: Uses dataflow-rs for robust workflow orchestration
- ✅ **Data Validation**: Built-in validation tasks using JSONLogic expressions
- 🗺️ **Data Transformation**: Map and transform data between different structures
- 🌐 **REST API**: Clean HTTP endpoints for easy integration
- 🔧 **Configurable**: Define custom workflows with JSON configuration
- 📊 **Error Handling**: Comprehensive error reporting and recovery

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

3. The server will start on `http://localhost:3000`

## API Endpoints

### Health Check
**GET** `/` or `/health`

Returns the health status and available workflows.

**Response:**
```json
{
  "status": "healthy",
  "workflows": ["data_validator", "data_enricher"]
}
```

### List Workflows
**GET** `/workflows`

Returns detailed information about available workflows.

**Response:**
```json
{
  "workflows": [
    {
      "id": "data_validator",
      "name": "Data Validation and Transformation",
      "description": "Validates input data (email, name) and transforms it into a structured format"
    },
    {
      "id": "data_enricher",
      "name": "Data Enrichment Workflow", 
      "description": "Validates user ID and enriches data with additional metadata"
    }
  ]
}
```

### Process Data
**POST** `/process`

Processes payload data through configured workflows.

**Request Body:**
```json
{
  "workflow_id": "data_validator", // Optional: specific workflow ID
  "payload": {
    "email": "user@example.com",
    "name": "John Doe"
  }
}
```

**Response:**
```json
{
  "success": true,
  "result": {
    "data": {
      "email": "user@example.com",
      "name": "John Doe",
      "user_info": {
        "full_name": "John Doe",
        "email_address": "user@example.com",
        "processed_at": "2024-01-01T12:00:00Z"
      }
    },
    "metadata": {
      "timestamp": "2024-01-01T12:00:00Z"
    }
  },
  "errors": [],
  "message": "Processing completed successfully"
}
```

## Configured Workflows

### 1. Data Validator (`data_validator`)

**Purpose**: Validates input data and transforms it into a structured format.

**Tasks**:
- **validate_input**: Ensures `email` and `name` fields are present and not null
- **transform_data**: Maps input data to a structured `user_info` object

**Input Example**:
```json
{
  "email": "jane@example.com",
  "name": "Jane Smith"
}
```

**Output**: Structured user information with validation and transformation applied.

### 2. Data Enricher (`data_enricher`)

**Purpose**: Validates user ID and enriches data with metadata.

**Tasks**:
- **validate_user_id**: Ensures `user_id` field is present and not null
- **enrich_user_data**: Adds enrichment metadata and processing timestamp

**Input Example**:
```json
{
  "user_id": "12345"
}
```

**Output**: Enriched data with additional metadata and processing information.

## Usage Examples

### Example 1: User Data Validation and Transformation

```bash
curl -X POST http://localhost:3000/process \
  -H "Content-Type: application/json" \
  -d '{
    "payload": {
      "email": "alice@example.com",
      "name": "Alice Johnson"
    }
  }'
```

### Example 2: User Data Enrichment

```bash
curl -X POST http://localhost:3000/process \
  -H "Content-Type: application/json" \
  -d '{
    "payload": {
      "user_id": "user_123"
    }
  }'
```

### Example 3: Error Handling (Invalid Data)

```bash
curl -X POST http://localhost:3000/process \
  -H "Content-Type: application/json" \
  -d '{
    "payload": {
      "invalid_field": "some_value"
    }
  }'
```

This will return validation errors for missing required fields.

## Architecture

### Components

1. **API Layer**: Axum-based REST API server
2. **Workflow Engine**: dataflow-rs engine for processing workflows
3. **Message Processing**: Handles data transformation through workflow tasks
4. **Error Handling**: Comprehensive error collection and reporting

### Workflow Tasks

The application uses built-in dataflow-rs functions:

- **validate**: Validates data using JSONLogic expressions
- **map**: Transforms and maps data between different structures
- **http**: Can fetch data from external APIs (configurable)

### Data Flow

1. Client sends POST request to `/process` with payload
2. Application creates a `Message` with the payload data
3. Dataflow engine processes the message through configured workflows
4. Each workflow executes its tasks sequentially (validate → transform)
5. Results and any errors are collected and returned to the client

## Extending the Application

### Adding New Workflows

To add new workflows, modify the `setup_workflows` function in `src/main.rs`:

```rust
let new_workflow = r#"
{
    "id": "custom_workflow",
    "name": "Custom Processing Workflow",
    "tasks": [
        {
            "id": "custom_task",
            "name": "Custom Task",
            "function": {
                "name": "validate", // or "map", "http"
                "input": {
                    // Task configuration
                }
            }
        }
    ]
}
"#;

let workflow = Workflow::from_json(new_workflow)?;
engine.add_workflow(&workflow);
```

### Custom Function Handlers

You can extend the engine with custom function handlers:

```rust
use dataflow_rs::AsyncFunctionHandler;

struct CustomFunction;

#[async_trait]
impl AsyncFunctionHandler for CustomFunction {
    async fn execute(&self, message: &mut Message, input: &Value) -> Result<(usize, Vec<Change>)> {
        // Custom processing logic
        Ok((200, vec![]))
    }
}

// Register the custom function
engine.register_task_function("custom".to_string(), Box::new(CustomFunction));
```

## Development

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

## Dependencies

- **axum**: Modern, ergonomic web framework
- **tokio**: Async runtime
- **dataflow-rs**: Workflow processing engine
- **serde**: Serialization/deserialization
- **tower-http**: HTTP middleware (CORS support)

## License

This project is licensed under the MIT License.
