use dataflow_rs::Engine;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;

// Request/Response structures for transformation API
#[derive(Debug, Deserialize)]
pub struct TransformationRequest {
    pub message: String,
    #[serde(default)]
    pub options: TransformationOptions,
}

// Request/Response structures for sample generation API
#[derive(Debug, Deserialize)]
pub struct SampleGenerationRequest {
    pub message_type: String,
    pub config: Value,
    #[serde(default)]
    pub options: SampleGenerationOptions,
}

#[derive(Debug, Deserialize, Default)]
pub struct SampleGenerationOptions {
    #[serde(default = "default_true")]
    pub validation: bool,
    #[serde(default)]
    pub include_debug: bool,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct TransformationOptions {
    #[serde(default = "default_true")]
    pub validation: bool,
    #[serde(default)]
    pub include_debug: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Serialize)]
pub struct TransformationResponse {
    pub success: bool,
    pub transformed_message: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug_info: Option<DebugInfo>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct DebugInfo {
    pub engine_state: String,
    pub workflow_execution: Vec<String>,
    pub intermediate_data: Value,
}

// Application State with dual engines
#[derive(Clone)]
pub struct AppState {
    pub forward_engine: Arc<Mutex<Engine>>,
    pub reverse_engine: Arc<Mutex<Engine>>,
}

// Health check response
#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: String,
    pub engines: EngineStatus,
    pub capabilities: Vec<String>,
}

#[derive(Serialize)]
pub struct EngineStatus {
    pub forward: String,
    pub reverse: String,
}

#[derive(Serialize)]
pub struct ReloadResponse {
    pub success: bool,
    pub message: String,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
