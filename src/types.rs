use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Clone, ToSchema)]
pub struct ResponseMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processing_time_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug_info: Option<Value>,
}

// Message category enum for routing
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub enum MessageCategory {
    MT,
    MX,
    Unknown,
}

// Request/Response structures for transformation API
#[derive(Debug, Deserialize, ToSchema)]
pub struct TransformationRequest {
    /// The SWIFT MT or ISO 20022 message to transform
    #[schema(example = "{1:F01BANKBEBBAXXX0237205215}{2:O103080908BANKBEBBAXXX...}")]
    pub message: String,
    #[serde(default)]
    pub options: TransformationOptions,
}

// Request/Response structures for sample generation API
#[derive(Debug, Deserialize, ToSchema)]
pub struct SampleGenerationRequest {
    /// Message type to generate (e.g., MT103, pacs.008)
    #[schema(example = "MT103")]
    pub message_type: String,
    /// Configuration for sample generation including scenario selection
    pub config: Value,
    #[serde(default)]
    pub options: SampleGenerationOptions,
}

#[derive(Debug, Deserialize, Default, ToSchema)]
pub struct SampleGenerationOptions {
    /// Enable validation of generated message
    #[schema(default = true)]
    #[serde(default = "default_true")]
    pub validation: bool,
    /// Include debug information in response
    #[schema(default = false)]
    #[serde(default)]
    pub debug: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SampleGenerationResponse {
    pub success: bool,
    pub message_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scenario: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<ReframeError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug_info: Option<SampleDebugInfo>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SampleDebugInfo {
    pub scenario_config: Value,
    pub generation_time_ms: u64,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generated_json: Option<Value>,
}

#[derive(Debug, Deserialize, Serialize, Default, ToSchema)]
pub struct TransformationOptions {
    /// Enable message validation
    #[schema(default = true)]
    #[serde(default = "default_true")]
    pub validation: bool,
    /// Include debug information in response
    #[schema(default = false)]
    #[serde(default)]
    pub debug: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TransformationResponse {
    pub success: bool,
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug_info: Option<DebugInfo>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<ReframeError>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DebugInfo {
    pub engine_state: String,
    pub workflow_execution: Vec<String>,
    pub intermediate_data: Value,
}


// Application State with threaded engines for vertical scaling
#[derive(Clone)]
pub struct AppState {
    pub forward_engine: Arc<dataflow_rs::ThreadedEngine>,
    pub reverse_engine: Arc<dataflow_rs::ThreadedEngine>,
    pub max_concurrent_tasks: Arc<tokio::sync::Semaphore>,
}

// Health check response
#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: String,
    pub engines: EngineStatus,
    pub capabilities: Vec<String>,
}

#[derive(Serialize, ToSchema)]
pub struct EngineStatus {
    pub forward: String,
    pub reverse: String,
}

#[derive(Serialize, ToSchema)]
pub struct ReloadResponse {
    pub success: bool,
    pub message: String,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// Validation API types
#[derive(Debug, Deserialize, ToSchema)]
pub struct ValidationRequest {
    /// The SWIFT MT or ISO 20022 message to validate
    pub message: String,
    #[serde(default)]
    pub options: ValidationOptions,
}

#[derive(Debug, Deserialize, Default, ToSchema)]
pub struct ValidationOptions {
    /// Include canonical JSON representation in response
    #[schema(default = false)]
    #[serde(default)]
    pub canonical: bool,
    /// Perform business rule validation
    #[schema(default = false)]
    #[serde(default)]
    pub business_validation: bool,
    /// Stop validation on first error
    #[schema(default = false)]
    #[serde(default)]
    pub fail_fast: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ValidationResponse {
    pub success: bool,
    pub message_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub canonical_json: Option<Value>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<ReframeError>,
}

#[derive(Debug, Serialize, Clone, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ErrorType {
    // Informational
    Info,

    // Validation errors
    ParserError,
    BusinessValidationError,

    // Operation errors
    TransformationError,
    GenerationError,

    // System errors
    InternalError,
}

#[derive(Debug, Serialize, Clone, ToSchema)]
pub struct ReframeError {
    /// Type of error
    pub error_type: ErrorType,
    /// Error code for programmatic handling
    #[schema(example = "INVALID_FORMAT")]
    pub code: String,
    /// Human-readable error message
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

// Helper methods for creating common error types
impl ReframeError {
    pub fn transformation_error(code: &str, message: String) -> Self {
        Self {
            error_type: ErrorType::TransformationError,
            code: code.to_string(),
            message,
            field: None,
            location: None,
            details: None,
        }
    }

    pub fn generation_error(code: &str, message: String) -> Self {
        Self {
            error_type: ErrorType::GenerationError,
            code: code.to_string(),
            message,
            field: None,
            location: None,
            details: None,
        }
    }

    pub fn internal_error(message: String) -> Self {
        Self {
            error_type: ErrorType::InternalError,
            code: "INTERNAL_ERROR".to_string(),
            message,
            field: None,
            location: None,
            details: None,
        }
    }
}
