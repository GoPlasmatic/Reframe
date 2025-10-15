use axum::{extract::Request, middleware::Next, response::Response};
use dataflow_rs::engine::message::Message;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tracing::{Span, debug, error, info, warn};
use uuid::Uuid;

use crate::package_manager::PackageInfo;
use crate::types::{AppState, ReframeError};

/// Extracts workflow errors from the message object
pub fn extract_workflow_errors(message: &Message) -> Vec<String> {
    let mut errors = Vec::new();

    // Check for errors in the message's error array
    if !message.errors.is_empty() {
        for error in &message.errors {
            // Return error messages as-is from workflows
            // Packages are responsible for formatting their own error messages
            errors.push(error.message.clone());
        }
    }

    errors
}

/// Resolves the package ID to use for a request
/// Returns the requested package ID or the default package ID if none specified
pub fn resolve_package_id(state: &AppState, requested_package: Option<&String>) -> String {
    if let Some(pkg) = requested_package {
        pkg.clone()
    } else {
        let pm = state.package_manager.read().unwrap();
        pm.get_default_package()
            .map(|p| p.id.clone())
            .unwrap_or_else(|| "unknown".to_string())
    }
}

/// Creates debug info for a response if debug mode is enabled
pub fn create_debug_info(debug: bool, message: &Message) -> Option<Value> {
    if debug {
        serde_json::to_value(message).ok()
    } else {
        None
    }
}

/// Middleware to add correlation IDs to requests
pub async fn correlation_middleware(mut req: Request, next: Next) -> Response {
    let request_id = Uuid::new_v4().to_string();
    let method = req.method().to_string();
    let path = req.uri().path().to_string();

    // Create a span for this request
    let span = tracing::info_span!(
        "http_request",
        request_id = %request_id,
        method = %method,
        path = %path,
        latency_ms = tracing::field::Empty,
        status = tracing::field::Empty,
    );

    let _enter = span.enter();

    // Add request ID to extensions so handlers can access it
    req.extensions_mut().insert(request_id.clone());

    let start = Instant::now();
    debug!("Request started");

    let response = next.run(req).await;

    let latency = start.elapsed();
    let status = response.status();

    // Record metrics in the span
    Span::current().record("latency_ms", latency.as_millis() as u64);
    Span::current().record("status", status.as_u16());

    if status.is_success() {
        info!(
            status = status.as_u16(),
            latency_ms = latency.as_millis(),
            "Request completed successfully"
        );
    } else if status.is_client_error() {
        warn!(
            status = status.as_u16(),
            latency_ms = latency.as_millis(),
            "Request failed with client error"
        );
    } else {
        error!(
            status = status.as_u16(),
            latency_ms = latency.as_millis(),
            "Request failed with server error"
        );
    }

    response
}

/// Loads a scenario from a package
/// Returns the scenario JSON data or an error
pub fn load_scenario_from_package(
    package: &PackageInfo,
    message_type: &str,
    scenario_name: &str,
) -> Result<Value, ReframeError> {
    // Load scenarios index from package
    let package_path = PathBuf::from(&package.package_path);
    let scenarios_index_path = package_path.join("scenarios").join("index.json");

    // Read and parse the scenarios index
    let index_content = std::fs::read_to_string(&scenarios_index_path).map_err(|e| {
        ReframeError::generation_error(
            "INDEX_NOT_FOUND",
            format!("Scenarios index not found: {}", e),
        )
    })?;

    let index: Value = serde_json::from_str(&index_content).map_err(|e| {
        ReframeError::generation_error(
            "INDEX_PARSE_ERROR",
            format!("Failed to parse scenarios index: {}", e),
        )
    })?;

    // Find the matching scenario file
    let scenario_file =
        find_scenario_file(&index, package_path.as_path(), message_type, scenario_name)?;

    debug!("Loading scenario from: {}", scenario_file.display());

    // Load and parse the scenario file
    let scenario_content = std::fs::read_to_string(&scenario_file).map_err(|_| {
        ReframeError::generation_error(
            "SCENARIO_FILE_NOT_FOUND",
            format!("Scenario file '{}' not found", scenario_file.display()),
        )
    })?;

    serde_json::from_str(&scenario_content).map_err(|e| {
        ReframeError::generation_error(
            "SCENARIO_PARSE_ERROR",
            format!("Failed to parse scenario file: {}", e),
        )
    })
}

/// Finds the scenario file path in the package index
fn find_scenario_file(
    index: &Value,
    package_path: &Path,
    message_type: &str,
    scenario_name: &str,
) -> Result<PathBuf, ReframeError> {
    // Normalize message type for case-insensitive matching
    let message_type_normalized = message_type.replace(".", "").to_lowercase();

    // Search through all sections in the index
    if let Some(sections) = index.as_object() {
        for (_section_name, section_value) in sections {
            if let Some(scenarios) = section_value.as_array() {
                for scenario in scenarios {
                    let source_field = scenario
                        .get("source")
                        .and_then(|s| s.as_str())
                        .unwrap_or("");
                    let id_field = scenario.get("id").and_then(|s| s.as_str()).unwrap_or("");

                    // Normalize for comparison
                    let source_normalized = source_field.replace(".", "").to_lowercase();

                    // Check if this scenario matches
                    let matches = (source_field == message_type
                        || source_normalized == message_type_normalized)
                        && id_field == scenario_name;

                    if matches && let Some(file) = scenario.get("file").and_then(|f| f.as_str()) {
                        return Ok(package_path.join("scenarios").join(file));
                    }
                }
            }
        }
    }

    Err(ReframeError::generation_error(
        "SCENARIO_NOT_FOUND",
        format!(
            "Scenario '{}' not found for message type '{}'",
            scenario_name, message_type
        ),
    ))
}
