use arc_swap::ArcSwap;
use serde::Serialize;
use serde_json::Value;
use std::sync::Arc;
use utoipa::ToSchema;

// Application State with async engines for high-performance multi-threaded processing
#[derive(Clone)]
pub struct AppState {
    pub transform_engine: Arc<ArcSwap<dataflow_rs::Engine>>,
    pub generation_engine: Arc<ArcSwap<dataflow_rs::Engine>>,
    pub validation_engine: Arc<ArcSwap<dataflow_rs::Engine>>,
    pub package_manager: Arc<std::sync::RwLock<crate::package_manager::PackageManager>>,
}

#[derive(Debug, Serialize, Clone, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ErrorType {
    // Operation errors
    Transformation,
    Generation,

    // System errors
    Internal,
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
    /// Box the details field to keep ReframeError size small
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Box<Value>>,
}

// Helper methods for creating common error types
impl ReframeError {
    pub fn transformation_error(code: &str, message: String) -> Self {
        Self {
            error_type: ErrorType::Transformation,
            code: code.to_string(),
            message,
            field: None,
            location: None,
            details: None,
        }
    }

    pub fn generation_error(code: &str, message: String) -> Self {
        Self {
            error_type: ErrorType::Generation,
            code: code.to_string(),
            message,
            field: None,
            location: None,
            details: None,
        }
    }

    pub fn internal_error(message: String) -> Self {
        Self {
            error_type: ErrorType::Internal,
            code: "INTERNAL_ERROR".to_string(),
            message,
            field: None,
            location: None,
            details: None,
        }
    }
}
