use axum::{extract::Request, middleware::Next, response::Response};
use dataflow_rs::engine::message::Message;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tracing::{Span, debug, error, info, warn};
use uuid::Uuid;

use crate::custom_fields::compute_and_store_fields;
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

/// Compute and store custom fields for a message based on its package
///
/// This function:
/// 1. Extracts the package_id from message metadata
/// 2. Retrieves custom field definitions from the package
/// 3. Computes and stores custom fields in the message context
///
/// # Arguments
/// * `state` - Application state containing package manager
/// * `message` - Message to compute fields for (modified in place)
///
/// # Returns
/// Ok(()) on success, or an error message on failure
pub fn compute_and_store_custom_fields(
    state: &AppState,
    message: &mut Message,
) -> Result<(), String> {
    // Get package ID from context root (preferred) or metadata (fallback)
    // We store at context root to prevent workflow overwrites
    let package_id = message
        .context
        .get("package_id")
        .and_then(|v| v.as_str())
        .or_else(|| {
            message
                .metadata()
                .get("package_id")
                .and_then(|v| v.as_str())
        })
        .map(String::from)
        .ok_or_else(|| "No package_id in message context or metadata".to_string())?;

    // Get package from package manager
    let pm = state.package_manager.read().unwrap();
    let package = pm
        .get_package(&package_id)
        .ok_or_else(|| format!("Package '{}' not found", package_id))?;

    // Clone custom fields to avoid borrow issues
    let custom_fields = package.custom_fields.clone();

    // Drop the lock early
    drop(pm);

    // Skip if package has no custom fields
    if custom_fields.is_empty() {
        debug!("Package has no custom fields, skipping computation");
        return Ok(());
    }

    // Compute and store custom fields with package-specific accessor
    compute_and_store_fields(message, &custom_fields, &package_id)?;

    debug!(
        package_id = %package_id,
        field_count = custom_fields.len(),
        "Computed and stored custom fields with package-specific accessor"
    );

    Ok(())
}

/// Publish message to database with custom fields computation
///
/// This is a unified helper for all handlers to publish messages to the database.
/// It handles:
/// 1. Computing and storing custom fields before publishing
/// 2. Publishing the message if database client is available
/// 3. Error handling for custom field computation failures
///
/// # Arguments
/// * `state` - Application state
/// * `message` - Message to publish (modified in place to add custom fields)
/// * `persist_enabled` - Whether persistence is enabled for this operation type
pub fn publish_to_database_with_custom_fields(
    state: &AppState,
    message: &mut Message,
    persist_enabled: bool,
) {
    if !persist_enabled || state.database_client.is_none() {
        return;
    }

    if let Some(ref client) = state.database_client {
        // Extract values before mutable borrow to avoid lifetime issues
        let msg_id = message.id.clone();
        let direction = message
            .metadata()
            .get("transformation_direction")
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or_else(|| "unknown".to_string());
        let pkg_id_before = message
            .context
            .get("package_id")
            .or_else(|| message.metadata().get("package_id"))
            .and_then(|v| v.as_str())
            .map(String::from);

        debug!(
            message_id = %msg_id,
            direction = %direction,
            package_id_before = ?pkg_id_before,
            "About to compute custom fields"
        );

        // Compute and store custom fields before publishing
        let computation_result = compute_and_store_custom_fields(state, message);

        if let Err(e) = computation_result {
            error!(
                error = %e,
                message_id = %msg_id,
                direction = %direction,
                "Failed to compute custom fields, publishing without them"
            );
        } else {
            // Log after successful computation
            let has_custom_fields = message.context.get("custom_fields").is_some();
            debug!(
                message_id = %msg_id,
                direction = %direction,
                has_custom_fields = has_custom_fields,
                "Custom fields computed successfully"
            );
        }

        client.clone().publish_message(message);
    }
}
