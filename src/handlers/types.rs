use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

use crate::types::ReframeError;

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

// Request/Response structures for transformation API
#[derive(Debug, Deserialize, ToSchema)]
pub struct TransformationRequest {
    /// Package ID to use for transformation (optional, uses default if not specified)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "swift-cbpr-mt-mx")]
    pub package: Option<String>,
    /// The SWIFT MT or ISO 20022 message to transform
    #[schema(example = "{1:F01BANKBEBBAXXX0237205215}{2:O103080908BANKBEBBAXXX...}")]
    pub message: String,
    /// Optional message type hint for routing (MT103, pacs.008, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_type_hint: Option<String>,
    /// Enable message validation
    #[schema(default = true)]
    #[serde(default = "default_true")]
    pub validation: bool,
    /// Include debug information in response
    #[schema(default = false)]
    #[serde(default)]
    pub debug: bool,
    /// Additional metadata for workflows
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

// Request/Response structures for sample generation API
#[derive(Debug, Deserialize, ToSchema)]
pub struct SampleGenerationRequest {
    /// Package ID to use for generation (optional, uses default if not specified)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "swift-cbpr-mt-mx")]
    pub package: Option<String>,
    /// Message type to generate (e.g., MT103, pacs.008)
    #[schema(example = "MT103")]
    pub message_type: String,
    /// Scenario selection
    #[schema(example = "standard")]
    pub scenario: String,
    /// Include debug information in response
    #[schema(default = false)]
    #[serde(default)]
    pub debug: bool,
    /// Additional metadata for workflows
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SampleGenerationResponse {
    pub success: bool,
    /// Package that handled the request
    pub package: String,
    pub message_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scenario: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<ReframeError>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
    /// Full Message object for debugging (when debug=true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug_info: Option<Value>,
    /// Request processing time in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processing_time_ms: Option<u64>,
}

pub fn default_true() -> bool {
    true
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TransformationResponse {
    pub success: bool,
    /// Package that handled the request
    pub package: String,
    /// Detected message type (not hint)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_type: Option<String>,
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<ReframeError>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
    /// Full Message object for debugging (when debug=true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug_info: Option<Value>,
    /// Request processing time in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processing_time_ms: Option<u64>,
}

// Health check response
#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: String,
    pub engines: EngineStatus,
    pub capabilities: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<ConfigInfo>,
}

#[derive(Serialize, ToSchema)]
pub struct EngineStatus {
    pub transform: String,
    pub generation: String,
    pub validation: String,
}

#[derive(Serialize, ToSchema)]
pub struct ConfigInfo {
    pub thread_count: usize,
}

#[derive(Serialize, ToSchema)]
pub struct ReloadResponse {
    pub success: bool,
    pub message: String,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// Package listing response
#[derive(Serialize, ToSchema)]
pub struct PackagesResponse {
    pub success: bool,
    pub packages: Vec<PackageDetails>,
}

#[derive(Serialize, ToSchema)]
pub struct PackageDetails {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: String,
    pub engine_version: String,
    pub status: String,
    pub workflows: WorkflowsInfo,
    pub loaded_at: String,
}

#[derive(Serialize, ToSchema)]
pub struct WorkflowsInfo {
    pub transform: WorkflowInfo,
    pub generate: WorkflowInfo,
    pub validate: WorkflowInfo,
}

#[derive(Serialize, ToSchema)]
pub struct WorkflowInfo {
    pub available: bool,
    pub description: String,
}

// Validation API types
#[derive(Debug, Deserialize, ToSchema)]
pub struct ValidationRequest {
    /// Package ID to use for validation (optional, uses default if not specified)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "swift-cbpr-mt-mx")]
    pub package: Option<String>,
    /// The SWIFT MT or ISO 20022 message to validate
    pub message: String,
    /// Include canonical JSON representation in response
    #[schema(default = false)]
    #[serde(default)]
    pub debug: bool,
    /// Additional metadata for workflows
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ValidationResponse {
    pub success: bool,
    /// Package that handled the request
    pub package: String,
    pub message_type: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<ReframeError>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
    /// Full Message object for debugging (when debug=true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug_info: Option<Value>,
    /// Request processing time in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processing_time_ms: Option<u64>,
}
