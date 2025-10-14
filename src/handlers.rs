use axum::{
    Json,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use dataflow_rs::engine::message::Message;
use serde_json::Value;
use std::sync::Arc;
use std::time::Instant;
use tracing::{Span, debug, error, info, instrument, warn};
use uuid::Uuid;

use crate::engine::reload_engines;
use crate::types::{
    AppState, DebugInfo, EngineStatus, HealthResponse, PackageDetails, PackagesResponse,
    ReframeError, ReloadResponse, SampleDebugInfo, SampleGenerationRequest,
    SampleGenerationResponse, TransformationRequest, TransformationResponse, ValidationRequest,
    ValidationResponse, WorkflowInfo, WorkflowsInfo,
};

// Note: SCENARIOS_INDEX removed - now using package manager for scenario loading

/// Extracts workflow errors from the message object
pub fn extract_workflow_errors(message: &Message) -> Vec<String> {
    let mut errors = Vec::new();

    // Check for errors in the message's error array
    if !message.errors.is_empty() {
        for error in &message.errors {
            let error_msg = &error.message;
            // Clean up the error message to make it more user-friendly
            let clean_error = if error_msg.starts_with("Validation error: ") {
                error_msg
                    .strip_prefix("Validation error: ")
                    .unwrap_or(error_msg)
            } else {
                error_msg
            };

            // Further clean up SwiftMT parser errors
            let clean_error = if clean_error.starts_with("SwiftMT parser error: ") {
                clean_error
                    .strip_prefix("SwiftMT parser error: ")
                    .unwrap_or(clean_error)
            } else {
                clean_error
            };

            errors.push(clean_error.to_string());
        }
    }

    errors
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

// Unified sample generation endpoint using workflow engine
#[utoipa::path(
    post,
    path = "/api/generate",
    tag = "generation",
    request_body = SampleGenerationRequest,
    responses(
        (status = 200, description = "Sample generated successfully", body = SampleGenerationResponse),
        (status = 400, description = "Invalid request", body = SampleGenerationResponse),
        (status = 500, description = "Internal server error", body = SampleGenerationResponse)
    )
)]
#[instrument(skip(state, request), fields(message_type = request.message_type.as_str()))]
pub async fn generate_sample(
    State(state): State<AppState>,
    Json(request): Json<SampleGenerationRequest>,
) -> Result<Json<SampleGenerationResponse>, StatusCode> {
    let start_time = Instant::now();

    // Get package to use (default if not specified)
    let package_id = if let Some(ref pkg) = request.package {
        pkg.clone()
    } else {
        let pm = state.package_manager.read().unwrap();
        pm.get_default_package()
            .map(|p| p.id.clone())
            .unwrap_or_else(|| "unknown".to_string())
    };

    info!(
        "🔄 Processing sample generation request for {} with scenario '{}' (package: {})",
        request.message_type, request.scenario, package_id
    );
    debug!("Debug: {}", request.debug);

    // Load scenario from package manager
    let scenario_name = &request.scenario;
    let schema_data = {
        // Get the package
        let pm = state.package_manager.read().unwrap();
        let package = if let Some(ref pkg_id) = request.package {
            pm.get_package(pkg_id)
        } else {
            pm.get_default_package()
        };

        let package = match package {
            Some(p) => p,
            None => {
                error!("Package '{}' not found", package_id);
                return Ok(Json(SampleGenerationResponse {
                    success: false,
                    package: package_id.clone(),
                    message_type: request.message_type.clone(),
                    result: None,
                    scenario: Some(scenario_name.to_string()),
                    errors: vec![ReframeError::generation_error(
                        "PACKAGE_NOT_FOUND",
                        format!("Package '{}' not found", package_id),
                    )],
                    debug_info: None,
                    processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
                }));
            }
        };

        // Determine which section to look in based on message type
        let section = if request.message_type.starts_with("MT") {
            "outgoing" // MT→MX (outgoing)
        } else {
            "incoming" // MX→MT (incoming)
        };

        // Load scenarios index from package
        let package_path = std::path::PathBuf::from(&package.package_path);
        let scenarios_index_path = package_path.join("scenarios").join("index.json");
        let index: Value = match std::fs::read_to_string(&scenarios_index_path) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(v) => v,
                Err(e) => {
                    error!("Failed to parse scenarios index: {}", e);
                    return Ok(Json(SampleGenerationResponse {
                        success: false,
                        package: package_id.clone(),
                        message_type: request.message_type.clone(),
                        result: None,
                        scenario: Some(scenario_name.to_string()),
                        errors: vec![ReframeError::generation_error(
                            "INDEX_PARSE_ERROR",
                            format!("Failed to parse scenarios index: {}", e),
                        )],
                        debug_info: None,
                        processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
                    }));
                }
            },
            Err(e) => {
                error!("Failed to read scenarios index: {}", e);
                return Ok(Json(SampleGenerationResponse {
                    success: false,
                    package: package_id.clone(),
                    message_type: request.message_type.clone(),
                    result: None,
                    scenario: Some(scenario_name.to_string()),
                    errors: vec![ReframeError::generation_error(
                        "INDEX_NOT_FOUND",
                        format!("Scenarios index not found in package '{}'", package_id),
                    )],
                    debug_info: None,
                    processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
                }));
            }
        };

        // Find the matching scenario in the index
        let scenario_file = if let Some(scenarios) = index.get(section).and_then(|s| s.as_array()) {
            let mut found_file = None;

            for scenario in scenarios {
                let source_field = scenario
                    .get("source")
                    .and_then(|s| s.as_str())
                    .unwrap_or("");
                let id_field = scenario.get("id").and_then(|s| s.as_str()).unwrap_or("");

                // Check if this scenario matches
                let matches = if request.message_type.starts_with("MT") {
                    source_field == request.message_type && id_field == scenario_name
                } else {
                    // For MX messages, normalize the comparison
                    let mx_type = request.message_type.replace(".", "").to_lowercase();
                    let source_type = source_field.replace(".", "").to_lowercase();
                    source_type == mx_type && id_field == scenario_name
                };

                if matches && let Some(file) = scenario.get("file").and_then(|f| f.as_str()) {
                    found_file = Some(package_path.join("scenarios").join(file));
                    break;
                }
            }

            match found_file {
                Some(f) => f,
                None => {
                    error!(
                        "Scenario '{}' not found for message type '{}' in package '{}'",
                        scenario_name, request.message_type, package_id
                    );
                    return Ok(Json(SampleGenerationResponse {
                        success: false,
                        package: package_id.clone(),
                        message_type: request.message_type.clone(),
                        result: None,
                        scenario: Some(scenario_name.to_string()),
                        errors: vec![ReframeError::generation_error(
                            "SCENARIO_NOT_FOUND",
                            format!(
                                "Scenario '{}' not found for message type '{}' in package '{}'",
                                scenario_name, request.message_type, package_id
                            ),
                        )],
                        debug_info: None,
                        processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
                    }));
                }
            }
        } else {
            error!(
                "Invalid scenarios index structure in package '{}'",
                package_id
            );
            return Ok(Json(SampleGenerationResponse {
                success: false,
                package: package_id.clone(),
                message_type: request.message_type.clone(),
                result: None,
                scenario: Some(scenario_name.to_string()),
                errors: vec![ReframeError::generation_error(
                    "INDEX_STRUCTURE_ERROR",
                    format!(
                        "Invalid scenarios index structure in package '{}'",
                        package_id
                    ),
                )],
                debug_info: None,
                processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
            }));
        };

        debug!("Loading scenario from: {}", scenario_file.display());

        // Load the scenario file
        match std::fs::read_to_string(&scenario_file) {
            Ok(content) => {
                match serde_json::from_str::<serde_json::Value>(&content) {
                    Ok(scenario_data) => {
                        // Use the entire scenario (contains both 'variables' and 'schema' for datafake)
                        scenario_data
                    }
                    Err(e) => {
                        error!("Failed to parse scenario file: {}", e);
                        return Ok(Json(SampleGenerationResponse {
                            success: false,
                            package: package_id.clone(),
                            message_type: request.message_type.clone(),
                            result: None,
                            scenario: Some(scenario_name.to_string()),
                            errors: vec![ReframeError::generation_error(
                                "SCENARIO_PARSE_ERROR",
                                format!("Failed to parse scenario file: {}", e),
                            )],
                            debug_info: None,
                            processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
                        }));
                    }
                }
            }
            Err(_e) => {
                error!("Scenario file not found: {}", scenario_file.display());
                return Ok(Json(SampleGenerationResponse {
                    success: false,
                    package: package_id.clone(),
                    message_type: request.message_type.clone(),
                    result: None,
                    scenario: Some(scenario_name.to_string()),
                    errors: vec![ReframeError::generation_error(
                        "SCENARIO_FILE_NOT_FOUND",
                        format!(
                            "Scenario file '{}' not found in package '{}'",
                            scenario_file.display(),
                            package_id
                        ),
                    )],
                    debug_info: None,
                    processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
                }));
            }
        }
    };

    // Prepare message for generation workflow
    // The new plugin approach expects the datafake scenario in the payload itself
    let mut message = Message::new(Arc::new(schema_data));

    // Set metadata for workflow detection
    if let Some(metadata_obj) = message.metadata_mut().as_object_mut() {
        // Set the actual message type (e.g., "MT103" or "pacs.008")
        metadata_obj.insert(
            "message_type".to_string(),
            request.message_type.clone().into(),
        );

        // Merge user-provided metadata if present
        if let Some(user_metadata) = &request.metadata
            && let Some(user_obj) = user_metadata.as_object()
        {
            for (key, value) in user_obj {
                metadata_obj.insert(key.clone(), value.clone());
            }
        }
    }

    // Debug: Log message structure BEFORE workflow execution
    debug!("Message data BEFORE workflow: {:?}", message.data());
    debug!("Message metadata BEFORE workflow: {:?}", message.metadata());

    // Process message using the generation engine
    match state
        .generation_engine
        .load()
        .process_message(&mut message)
        .await
    {
        Ok(()) => {
            let processing_time = start_time.elapsed().as_millis() as u64;

            // Debug: Print the entire message structure after workflow completion
            debug!("Message data after workflow: {:?}", message.data());
            debug!("Message metadata after workflow: {:?}", message.metadata());

            // Check if generation produced a result
            if let Some(result) = message.data().get("result") {
                if let Some(result_str) = result.as_str() {
                    info!("✅ Sample generation completed in {}ms", processing_time);

                    Ok(Json(SampleGenerationResponse {
                        success: true,
                        package: package_id.clone(),
                        message_type: request.message_type.clone(),
                        result: Some(result_str.to_string()),
                        scenario: Some(request.scenario.clone()),
                        errors: Vec::new(),
                        debug_info: if request.debug {
                            Some(SampleDebugInfo {
                                scenario_config: serde_json::json!({
                                    "scenario": request.scenario.clone()
                                }),
                                generation_time_ms: processing_time,
                                warnings: Vec::new(),
                                generated_json: message.data().get("generated_message").cloned(),
                            })
                        } else {
                            None
                        },
                        processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
                    }))
                } else {
                    error!("Generation workflow did not produce a string result");
                    Ok(Json(SampleGenerationResponse {
                        success: false,
                        package: package_id.clone(),
                        message_type: request.message_type.clone(),
                        result: None,
                        scenario: None,
                        errors: vec![ReframeError::generation_error(
                            "NO_RESULT",
                            "Generation workflow did not produce a valid result".to_string(),
                        )],
                        debug_info: None,
                        processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
                    }))
                }
            } else {
                // Check for errors in the message
                let workflow_errors = extract_workflow_errors(&message);
                if !workflow_errors.is_empty() {
                    error!("Generation failed with errors: {:?}", workflow_errors);
                    Ok(Json(SampleGenerationResponse {
                        success: false,
                        package: package_id.clone(),
                        message_type: request.message_type.clone(),
                        result: None,
                        scenario: None,
                        errors: workflow_errors
                            .into_iter()
                            .map(|e| ReframeError::generation_error("WORKFLOW_ERROR", e))
                            .collect(),
                        debug_info: None,
                        processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
                    }))
                } else {
                    error!("Generation workflow did not produce a result");
                    Ok(Json(SampleGenerationResponse {
                        success: false,
                        package: package_id.clone(),
                        message_type: request.message_type.clone(),
                        result: None,
                        scenario: None,
                        errors: vec![ReframeError::generation_error(
                            "NO_RESULT",
                            "Generation workflow did not produce a result".to_string(),
                        )],
                        debug_info: None,
                        processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
                    }))
                }
            }
        }
        Err(e) => {
            error!(
                error = %e,
                "Sample generation failed"
            );

            Ok(Json(SampleGenerationResponse {
                success: false,
                package: package_id.clone(),
                message_type: request.message_type.clone(),
                result: None,
                scenario: None,
                errors: vec![ReframeError::generation_error(
                    "GENERATION_FAILED",
                    e.to_string(),
                )],
                debug_info: if request.debug {
                    Some(SampleDebugInfo {
                        scenario_config: serde_json::json!({
                            "scenario": request.scenario.clone()
                        }),
                        generation_time_ms: start_time.elapsed().as_millis() as u64,
                        warnings: Vec::new(),
                        generated_json: None,
                    })
                } else {
                    None
                },
                processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
            }))
        }
    }
}

// Health check endpoint
#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse),
    )
)]
pub async fn health_check(State(_state): State<AppState>) -> Json<HealthResponse> {
    // Async Engine is always healthy if it exists
    let cpu_count = num_cpus::get();

    let transform_status =
        "healthy (Unified bidirectional, Async Engine, Tokio runtime)".to_string();
    let generation_status = "healthy (Async Engine, Tokio runtime)".to_string();
    let validation_status = "healthy (Async Engine, Tokio runtime)".to_string();

    Json(HealthResponse {
        status: "running".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        engines: EngineStatus {
            transform: transform_status,
            generation: generation_status,
            validation: validation_status,
        },
        config: Some(crate::types::ConfigInfo {
            thread_count: cpu_count,
        }),
        capabilities: vec![
            "Unified bidirectional transformation MT ↔ MX (Async Engine, Tokio runtime)"
                .to_string(),
            "Package-based workflow architecture".to_string(),
            "High-performance async multi-threaded processing".to_string(),
            "Sample generation for MT and MX messages".to_string(),
            "Unified validation for MT and MX messages".to_string(),
        ],
    })
}

#[utoipa::path(
    post,
    path = "/admin/reload-workflows",
    tag = "admin",
    responses(
        (status = 200, description = "Workflows reloaded successfully", body = ReloadResponse),
        (status = 500, description = "Failed to reload workflows", body = ReloadResponse)
    )
)]
#[instrument(skip(state))]
pub async fn reload_workflows(
    State(state): State<AppState>,
) -> Result<Json<ReloadResponse>, StatusCode> {
    let start_time = Instant::now();

    info!("🔄 Processing workflow reload request");

    match reload_engines(&state).await {
        Ok(()) => {
            let reload_time = start_time.elapsed().as_millis() as u64;
            info!(
                "✅ Workflow reload completed successfully in {}ms",
                reload_time
            );

            Ok(Json(ReloadResponse {
                success: true,
                message: format!("Workflows reloaded successfully in {reload_time}ms"),
                timestamp: chrono::Utc::now().to_rfc3339(),
                error: None,
            }))
        }
        Err(e) => {
            let reload_time = start_time.elapsed().as_millis() as u64;
            error!(
                reload_time_ms = reload_time,
                error = %e,
                "Workflow reload failed"
            );

            Ok(Json(ReloadResponse {
                success: false,
                message: format!("Workflow reload failed after {reload_time}ms"),
                timestamp: chrono::Utc::now().to_rfc3339(),
                error: Some(e.to_string()),
            }))
        }
    }
}

/// Unified transformation endpoint that handles both MT↔MX transformations
#[utoipa::path(
    post,
    path = "/api/transform",
    tag = "transformation",
    request_body = TransformationRequest,
    responses(
        (status = 200, description = "Transformation successful", body = TransformationResponse),
        (status = 400, description = "Invalid message", body = TransformationResponse),
        (status = 500, description = "Internal server error", body = TransformationResponse)
    )
)]
#[instrument(skip(state, request), fields(message_length = request.message.len()))]
pub async fn transform(
    State(state): State<AppState>,
    Json(request): Json<TransformationRequest>,
) -> Result<Json<TransformationResponse>, StatusCode> {
    let start_time = Instant::now();

    // Get package to use (default if not specified)
    let package_id = if let Some(ref pkg) = request.package {
        pkg.clone()
    } else {
        let pm = state.package_manager.read().unwrap();
        pm.get_default_package()
            .map(|p| p.id.clone())
            .unwrap_or_else(|| "unknown".to_string())
    };

    info!(
        "🔄 Processing unified transformation request (package: {})",
        package_id
    );
    debug!(
        "Validation: {}, Debug: {}",
        request.validation, request.debug
    );
    if let Some(ref hint) = request.message_type_hint {
        debug!("Message type hint: {}", hint);
    }

    // Prepare message for transformation
    let payload_value = Value::String(request.message.clone());
    let mut message = Message::new(Arc::new(payload_value));

    // Detect transformation direction (MT→MX or MX→MT)
    let transformation_direction = if request.message.trim_start().starts_with('<')
        || request.message.trim_start().starts_with("<?xml")
    {
        "mx-to-mt" // MX message, convert to MT
    } else {
        "mt-to-mx" // MT message, convert to MX
    };

    // Set metadata for workflow detection
    if let Some(metadata_obj) = message.metadata_mut().as_object_mut() {
        metadata_obj.insert(
            "transformation_direction".to_string(),
            transformation_direction.into(),
        );
        metadata_obj.insert("validation".to_string(), request.validation.into());

        // Add message type hint if provided
        if let Some(ref hint) = request.message_type_hint {
            metadata_obj.insert("message_type_hint".to_string(), hint.clone().into());
        }

        // Merge user-provided metadata if present
        if let Some(user_metadata) = &request.metadata
            && let Some(user_obj) = user_metadata.as_object()
        {
            for (key, value) in user_obj {
                metadata_obj.insert(key.clone(), value.clone());
            }
        }
    }

    debug!(
        "Detected transformation direction: {}",
        transformation_direction
    );

    // Process message using the unified transform engine
    match state
        .transform_engine
        .load()
        .process_message(&mut message)
        .await
    {
        Ok(()) => {
            let processing_time = start_time.elapsed().as_millis() as u64;
            info!("✅ Transformation completed in {}ms", processing_time);

            // Check for validation/processing errors even if engine returned Ok
            let workflow_errors = extract_workflow_errors(&message);
            if !workflow_errors.is_empty() {
                error!(
                    "❌ Transformation failed with validation errors: {:?}",
                    workflow_errors
                );
                return Ok(Json(TransformationResponse {
                    success: false,
                    package: package_id.clone(),
                    message_type: None,
                    result: None,
                    debug_info: if request.debug {
                        let message_json = serde_json::to_value(&message).unwrap();
                        Some(DebugInfo {
                            engine_state: "transform (unified)".to_string(),
                            workflow_execution: vec!["Failed - Validation errors".to_string()],
                            intermediate_data: message_json,
                        })
                    } else {
                        None
                    },
                    errors: workflow_errors
                        .into_iter()
                        .map(|e| ReframeError::transformation_error("WORKFLOW_ERROR", e))
                        .collect(),
                    warnings: Vec::new(),
                    processing_time_ms: Some(processing_time),
                }));
            }

            // Validate that the transformation actually produced a result
            match message.data().get("result") {
                Some(result) if !result.is_null() => {
                    // Handle both string and array results
                    match result {
                        Value::String(s) if !s.trim().is_empty() => {
                            // Single string result
                            Ok(Json(TransformationResponse {
                                success: true,
                                package: package_id.clone(),
                                message_type: None,
                                result: Some(result.clone()),
                                debug_info: if request.debug {
                                    let message_json = serde_json::to_value(&message).unwrap();
                                    Some(DebugInfo {
                                        engine_state: "transform (unified)".to_string(),
                                        workflow_execution: vec!["Completed".to_string()],
                                        intermediate_data: message_json,
                                    })
                                } else {
                                    None
                                },
                                errors: Vec::new(),
                                warnings: Vec::new(),
                                processing_time_ms: Some(processing_time),
                            }))
                        }
                        Value::Array(arr) if !arr.is_empty() => {
                            // Multiple results
                            Ok(Json(TransformationResponse {
                                success: true,
                                package: package_id.clone(),
                                message_type: None,
                                result: Some(result.clone()),
                                debug_info: if request.debug {
                                    let message_json = serde_json::to_value(&message).unwrap();
                                    Some(DebugInfo {
                                        engine_state: "transform (unified)".to_string(),
                                        workflow_execution: vec![format!(
                                            "Completed - {} messages generated",
                                            arr.len()
                                        )],
                                        intermediate_data: message_json,
                                    })
                                } else {
                                    None
                                },
                                errors: Vec::new(),
                                warnings: Vec::new(),
                                processing_time_ms: Some(processing_time),
                            }))
                        }
                        _ => {
                            error!(
                                error_type = "EMPTY_RESULT",
                                "Transformation produced empty or invalid result"
                            );
                            Ok(Json(TransformationResponse {
                                success: false,
                                package: package_id.clone(),
                                message_type: None,
                                result: None,
                                debug_info: if request.debug {
                                    let message_json = serde_json::to_value(message).unwrap();
                                    Some(DebugInfo {
                                        engine_state: "transform (unified)".to_string(),
                                        workflow_execution: vec![
                                            "Failed - Empty or invalid result".to_string(),
                                        ],
                                        intermediate_data: message_json,
                                    })
                                } else {
                                    None
                                },
                                errors: vec![ReframeError::transformation_error(
                                    "EMPTY_RESULT",
                                    "Transformation completed but produced empty or invalid result"
                                        .to_string(),
                                )],
                                warnings: Vec::new(),
                                processing_time_ms: Some(processing_time),
                            }))
                        }
                    }
                }
                _ => {
                    error!(
                        error_type = "NO_RESULT",
                        "Transformation completed but no valid result found"
                    );
                    Ok(Json(TransformationResponse {
                        success: false,
                        package: package_id.clone(),
                        message_type: None,
                        result: None,
                        debug_info: if request.debug {
                            let message_json = serde_json::to_value(message).unwrap();
                            Some(DebugInfo {
                                engine_state: "transform (unified)".to_string(),
                                workflow_execution: vec!["Failed - No result produced".to_string()],
                                intermediate_data: message_json,
                            })
                        } else {
                            None
                        },
                        errors: vec![ReframeError::transformation_error(
                            "NO_RESULT",
                            "Transformation completed but no valid result was produced".to_string(),
                        )],
                        warnings: Vec::new(),
                        processing_time_ms: Some(processing_time),
                    }))
                }
            }
        }
        Err(e) => {
            let processing_time = start_time.elapsed().as_millis() as u64;
            error!(
                error_type = "TRANSFORMATION_FAILED",
                error = %e,
                "Transformation failed"
            );

            Ok(Json(TransformationResponse {
                success: false,
                package: package_id.clone(),
                message_type: None,
                result: None,
                debug_info: if request.debug {
                    let message_json = serde_json::to_value(&message).unwrap();
                    Some(DebugInfo {
                        engine_state: "transform (unified)".to_string(),
                        workflow_execution: vec![format!("Failed - Engine error: {}", e)],
                        intermediate_data: message_json,
                    })
                } else {
                    None
                },
                errors: vec![ReframeError::internal_error(e.to_string())],
                warnings: Vec::new(),
                processing_time_ms: Some(processing_time),
            }))
        }
    }
}

/// Unified validation endpoint that handles both MT and MX validation
#[utoipa::path(
    post,
    path = "/api/validate",
    tag = "validation",
    request_body = ValidationRequest,
    responses(
        (status = 200, description = "Validation completed", body = ValidationResponse),
        (status = 400, description = "Invalid request", body = ValidationResponse)
    )
)]
#[instrument(skip(state, request), fields(message_length = request.message.len()))]
pub async fn validate(
    State(state): State<AppState>,
    Json(request): Json<ValidationRequest>,
) -> Result<Json<ValidationResponse>, StatusCode> {
    let start_time = Instant::now();

    // Get package to use (default if not specified)
    let package_id = if let Some(ref pkg) = request.package {
        pkg.clone()
    } else {
        let pm = state.package_manager.read().unwrap();
        pm.get_default_package()
            .map(|p| p.id.clone())
            .unwrap_or_else(|| "unknown".to_string())
    };

    info!(
        "🔍 Processing unified validation request (package: {})",
        package_id
    );
    debug!(
        "Canonical: {}, Business validation: {}",
        request.canonical, request.business_validation
    );

    // Prepare message for validation workflow
    let payload_value = Value::String(request.message.clone());
    let mut message = Message::new(Arc::new(payload_value));

    // Detect message type (MT or MX) and set in metadata for workflow selection
    let message_type = if request.message.trim_start().starts_with('<')
        || request.message.trim_start().starts_with("<?xml")
    {
        "MX"
    } else {
        "MT"
    };

    // Set metadata for workflow detection
    if let Some(metadata_obj) = message.metadata_mut().as_object_mut() {
        metadata_obj.insert("message_type".to_string(), message_type.into());
        metadata_obj.insert("canonical".to_string(), request.canonical.into());
        metadata_obj.insert(
            "business_validation".to_string(),
            request.business_validation.into(),
        );

        // Merge user-provided metadata if present
        if let Some(user_metadata) = &request.metadata
            && let Some(user_obj) = user_metadata.as_object()
        {
            for (key, value) in user_obj {
                metadata_obj.insert(key.clone(), value.clone());
            }
        }
    }

    debug!("Detected message type: {}", message_type);

    // Process message using the validation engine
    match state
        .validation_engine
        .load()
        .process_message(&mut message)
        .await
    {
        Ok(()) => {
            let processing_time = start_time.elapsed().as_millis() as u64;

            // Check for workflow errors
            let workflow_errors = extract_workflow_errors(&message);
            if !workflow_errors.is_empty() {
                error!("Validation failed with errors: {:?}", workflow_errors);
                return Ok(Json(ValidationResponse {
                    success: false,
                    package: package_id.clone(),
                    message_type: None,
                    canonical_json: None,
                    errors: workflow_errors
                        .into_iter()
                        .map(|e| ReframeError::transformation_error("VALIDATION_ERROR", e))
                        .collect(),
                    processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
                }));
            }

            // Extract validation result from workflow output
            if let Some(output) = message.data().get("output") {
                let valid = output
                    .get("valid")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let message_type = output
                    .get("message_type")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                // Extract errors array
                let errors: Vec<ReframeError> = output
                    .get("errors")
                    .and_then(|e| e.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|e| e.as_str())
                            .map(|msg| {
                                ReframeError::transformation_error(
                                    "VALIDATION_ERROR",
                                    msg.to_string(),
                                )
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                info!(
                    "✅ Validation completed in {}ms - Valid: {}, Type: {:?}",
                    processing_time, valid, message_type
                );

                Ok(Json(ValidationResponse {
                    success: valid,
                    package: package_id.clone(),
                    message_type,
                    canonical_json: None,
                    errors,
                    processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
                }))
            } else {
                error!("Validation workflow did not produce output");
                Ok(Json(ValidationResponse {
                    success: false,
                    package: package_id.clone(),
                    message_type: None,
                    canonical_json: None,
                    errors: vec![ReframeError::internal_error(
                        "Validation workflow did not produce output".to_string(),
                    )],
                    processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
                }))
            }
        }
        Err(e) => {
            error!(
                error = %e,
                "Validation failed"
            );

            Ok(Json(ValidationResponse {
                success: false,
                package: package_id.clone(),
                message_type: None,
                canonical_json: None,
                errors: vec![ReframeError::internal_error(e.to_string())],
                processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
            }))
        }
    }
}

/// List all loaded packages with their details
#[utoipa::path(
    get,
    path = "/api/packages",
    tag = "packages",
    responses(
        (status = 200, description = "Packages listed successfully", body = PackagesResponse),
    )
)]
pub async fn list_packages(State(state): State<AppState>) -> Json<PackagesResponse> {
    info!("📦 Processing package list request");

    let pm = state.package_manager.read().unwrap();
    let packages = pm.get_packages();

    let package_details: Vec<PackageDetails> = packages
        .values()
        .map(|pkg| {
            // Build workflow info
            let transform_info = pkg
                .workflows
                .get("transform")
                .map(|w| WorkflowInfo {
                    available: true,
                    description: w.description.clone(),
                })
                .unwrap_or_else(|| WorkflowInfo {
                    available: false,
                    description: "Not available".to_string(),
                });

            let generate_info = pkg
                .workflows
                .get("generate")
                .map(|w| WorkflowInfo {
                    available: true,
                    description: w.description.clone(),
                })
                .unwrap_or_else(|| WorkflowInfo {
                    available: false,
                    description: "Not available".to_string(),
                });

            let validate_info = pkg
                .workflows
                .get("validate")
                .map(|w| WorkflowInfo {
                    available: true,
                    description: w.description.clone(),
                })
                .unwrap_or_else(|| WorkflowInfo {
                    available: false,
                    description: "Not available".to_string(),
                });

            PackageDetails {
                id: pkg.id.clone(),
                name: pkg.name.clone(),
                version: pkg.version.clone(),
                description: pkg.description.clone(),
                author: pkg.author.clone(),
                license: pkg.license.clone(),
                engine_version: pkg.engine_version.clone(),
                status: if pkg.enabled { "loaded" } else { "disabled" }.to_string(),
                workflows: WorkflowsInfo {
                    transform: transform_info,
                    generate: generate_info,
                    validate: validate_info,
                },
                loaded_at: pkg.loaded_at.to_rfc3339(),
            }
        })
        .collect();

    info!("✅ Listed {} package(s)", package_details.len());

    Json(PackagesResponse {
        success: true,
        packages: package_details,
    })
}
