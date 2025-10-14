use axum::{
    Json,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use dataflow_rs::engine::message::Message;
use once_cell::sync::Lazy;
use serde_json::Value;
use std::sync::Arc;
use std::time::Instant;
use tracing::{Span, debug, error, info, instrument, warn};
use uuid::Uuid;

use crate::engine::reload_engines;
use crate::types::{
    AppState, DebugInfo, EngineStatus, HealthResponse, ReframeError, ReloadResponse,
    SampleDebugInfo, SampleGenerationRequest, SampleGenerationResponse, TransformationRequest,
    TransformationResponse, ValidationRequest, ValidationResponse,
};

// Preload scenarios index for better performance
static SCENARIOS_INDEX: Lazy<Value> =
    Lazy::new(|| match std::fs::read_to_string("scenarios/index.json") {
        Ok(content) => match serde_json::from_str(&content) {
            Ok(v) => {
                info!("Scenarios index loaded successfully");
                v
            }
            Err(e) => {
                error!("Failed to parse scenarios index: {}", e);
                Value::Object(serde_json::Map::new())
            }
        },
        Err(e) => {
            error!("Failed to read scenarios index: {}", e);
            Value::Object(serde_json::Map::new())
        }
    });

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

// New endpoint handlers
#[utoipa::path(
    post,
    path = "/transform/mt-to-mx",
    tag = "transformation",
    request_body = TransformationRequest,
    responses(
        (status = 200, description = "Transformation successful", body = TransformationResponse),
        (status = 400, description = "Invalid MT message", body = TransformationResponse),
        (status = 500, description = "Internal server error", body = TransformationResponse)
    )
)]
#[instrument(skip(state, request), fields(message_length = request.message.len()))]
pub async fn transform_mt_to_mx(
    State(state): State<AppState>,
    Json(request): Json<TransformationRequest>,
) -> Result<Json<TransformationResponse>, StatusCode> {
    let start_time = Instant::now();

    info!("🔄 Processing MT to MX transformation request");
    debug!("Request options: {:?}", request.options);

    // Prepare message for transformation
    let payload_value = Value::String(request.message.clone());
    let mut message = Message::new(Arc::new(payload_value));

    if let Some(metadata_obj) = message.metadata_mut().as_object_mut() {
        metadata_obj.insert("transformation_direction".to_string(), "mt-to-mx".into());
    }

    // Process message using the async forward engine
    match state
        .forward_engine
        .load()
        .process_message(&mut message)
        .await
    {
        Ok(()) => {
            let processing_time = start_time.elapsed().as_millis() as u64;
            info!(
                "✅ MT to MX transformation completed in {}ms",
                processing_time
            );

            // Check for validation/processing errors even if engine returned Ok
            let workflow_errors = extract_workflow_errors(&message);
            if !workflow_errors.is_empty() {
                error!(
                    "❌ MT to MX transformation failed with validation errors: {:?}",
                    workflow_errors
                );
                return Ok(Json(TransformationResponse {
                    success: false,
                    result: None,
                    debug_info: if request.options.debug {
                        let message_json = serde_json::to_value(&message).unwrap();
                        Some(DebugInfo {
                            engine_state: "forward (threaded)".to_string(),
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
                }));
            }

            // Validate that the transformation actually produced a result
            match message.data().get("result") {
                Some(result) if !result.is_null() => {
                    // Handle both string and array results (MT to MX can produce multiple messages)
                    match result {
                        Value::String(s) if !s.trim().is_empty() => {
                            // Single string result
                            Ok(Json(TransformationResponse {
                                success: true,
                                result: Some(result.clone()),
                                debug_info: if request.options.debug {
                                    let message_json = serde_json::to_value(&message).unwrap();
                                    Some(DebugInfo {
                                        engine_state: "forward (threaded)".to_string(),
                                        workflow_execution: vec!["Completed".to_string()],
                                        intermediate_data: message_json,
                                    })
                                } else {
                                    None
                                },
                                errors: Vec::new(),
                                warnings: Vec::new(),
                            }))
                        }
                        Value::Array(arr) if !arr.is_empty() => {
                            // Multiple results
                            Ok(Json(TransformationResponse {
                                success: true,
                                result: Some(result.clone()),
                                debug_info: if request.options.debug {
                                    let message_json = serde_json::to_value(&message).unwrap();
                                    Some(DebugInfo {
                                        engine_state: "forward (threaded)".to_string(),
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
                            }))
                        }
                        _ => {
                            error!(
                                message_type = "MT",
                                error_type = "EMPTY_RESULT",
                                "Transformation produced empty or invalid result"
                            );
                            Ok(Json(TransformationResponse {
                                success: false,
                                result: None,
                                debug_info: if request.options.debug {
                                    let message_json = serde_json::to_value(message).unwrap();
                                    Some(DebugInfo {
                                        engine_state: "forward (threaded)".to_string(),
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
                            }))
                        }
                    }
                }
                _ => {
                    error!(
                        message_type = "MT",
                        error_type = "NO_RESULT",
                        "Transformation completed but no valid result found"
                    );
                    Ok(Json(TransformationResponse {
                        success: false,
                        result: None,
                        debug_info: if request.options.debug {
                            let message_json = serde_json::to_value(message).unwrap();
                            Some(DebugInfo {
                                engine_state: "forward (threaded)".to_string(),
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
                    }))
                }
            }
        }
        Err(e) => {
            error!(
                message_type = "MT",
                error_type = "TRANSFORMATION_FAILED",
                error = %e,
                "Transformation failed"
            );

            Ok(Json(TransformationResponse {
                success: false,
                result: None,
                debug_info: if request.options.debug {
                    let message_json = serde_json::to_value(&message).unwrap();
                    Some(DebugInfo {
                        engine_state: "forward".to_string(),
                        workflow_execution: vec![format!("Failed - Engine error: {}", e)],
                        intermediate_data: message_json,
                    })
                } else {
                    None
                },
                errors: vec![ReframeError::internal_error(e.to_string())],
                warnings: Vec::new(),
            }))
        }
    }
}

#[utoipa::path(
    post,
    path = "/transform/mx-to-mt",
    tag = "transformation",
    request_body = TransformationRequest,
    responses(
        (status = 200, description = "Transformation successful", body = TransformationResponse),
        (status = 400, description = "Invalid MX message", body = TransformationResponse),
        (status = 500, description = "Internal server error", body = TransformationResponse)
    )
)]
#[instrument(skip(state, request), fields(message_length = request.message.len()))]
pub async fn transform_mx_to_mt(
    State(state): State<AppState>,
    Json(request): Json<TransformationRequest>,
) -> Result<Json<TransformationResponse>, StatusCode> {
    let start_time = Instant::now();

    info!("🔄 Processing MX to MT transformation request");
    debug!("Request options: {:?}", request.options);

    // Prepare message for transformation
    let payload_value = Value::String(request.message.clone());
    let mut message = Message::new(Arc::new(payload_value));
    if let Some(metadata_obj) = message.metadata_mut().as_object_mut() {
        metadata_obj.insert("transformation_direction".to_string(), "mx-to-mt".into());
    }

    // Process message using the async reverse engine
    match state
        .reverse_engine
        .load()
        .process_message(&mut message)
        .await
    {
        Ok(()) => {
            let processing_time = start_time.elapsed().as_millis() as u64;
            info!(
                "✅ MX to MT transformation completed in {}ms",
                processing_time
            );

            // Check for validation/processing errors even if engine returned Ok
            let workflow_errors = extract_workflow_errors(&message);
            if !workflow_errors.is_empty() {
                error!(
                    "❌ MX to MT transformation failed with validation errors: {:?}",
                    workflow_errors
                );
                return Ok(Json(TransformationResponse {
                    success: false,
                    result: None,
                    debug_info: if request.options.debug {
                        let message_json = serde_json::to_value(&message).unwrap();
                        Some(DebugInfo {
                            engine_state: "reverse (threaded)".to_string(),
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
                }));
            }

            // Validate that the transformation actually produced a result
            match message.data().get("result") {
                Some(result) if !result.is_null() => {
                    // Handle both string and array results (workflow can return either)
                    let result_str = match result {
                        Value::String(s) => Some(s.as_str()),
                        Value::Array(arr) if !arr.is_empty() => arr[0].as_str(),
                        _ => None,
                    };

                    // Check if result is a valid non-empty string
                    match result_str {
                        Some(result_str) if !result_str.trim().is_empty() => {
                            // Return the result
                            Ok(Json(TransformationResponse {
                                success: true,
                                result: Some(Value::String(result_str.to_string())),
                                debug_info: if request.options.debug {
                                    let message_json = serde_json::to_value(&message).unwrap();
                                    Some(DebugInfo {
                                        engine_state: "reverse (threaded)".to_string(),
                                        workflow_execution: vec!["Completed".to_string()],
                                        intermediate_data: message_json,
                                    })
                                } else {
                                    None
                                },
                                errors: Vec::new(),
                                warnings: Vec::new(),
                            }))
                        }
                        _ => {
                            error!(
                                message_type = "MX",
                                error_type = "EMPTY_RESULT",
                                "Transformation produced empty or invalid result"
                            );
                            Ok(Json(TransformationResponse {
                                success: false,
                                result: None,
                                debug_info: if request.options.debug {
                                    let message_json = serde_json::to_value(message).unwrap();
                                    Some(DebugInfo {
                                        engine_state: "reverse (threaded)".to_string(),
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
                            }))
                        }
                    }
                }
                _ => {
                    error!(
                        message_type = "MX",
                        error_type = "NO_RESULT",
                        "Transformation completed but no valid result found"
                    );
                    Ok(Json(TransformationResponse {
                        success: false,
                        result: None,
                        debug_info: if request.options.debug {
                            let message_json = serde_json::to_value(message).unwrap();
                            Some(DebugInfo {
                                engine_state: "reverse (threaded)".to_string(),
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
                    }))
                }
            }
        }
        Err(e) => {
            error!(
                message_type = "MX",
                error_type = "TRANSFORMATION_FAILED",
                error = %e,
                "Transformation failed"
            );

            Ok(Json(TransformationResponse {
                success: false,
                result: None,
                debug_info: if request.options.debug {
                    let message_json = serde_json::to_value(&message).unwrap();
                    Some(DebugInfo {
                        engine_state: "reverse".to_string(),
                        workflow_execution: vec![format!("Failed - Engine error: {}", e)],
                        intermediate_data: message_json,
                    })
                } else {
                    None
                },
                errors: vec![ReframeError::internal_error(e.to_string())],
                warnings: Vec::new(),
            }))
        }
    }
}

// Unified sample generation endpoint using workflow engine
#[utoipa::path(
    post,
    path = "/generate/sample",
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

    info!(
        "🔄 Processing sample generation request for {} with scenario '{}'",
        request.message_type, request.scenario
    );
    debug!("Request options: {:?}", request.options);

    // Load scenario from preloaded index
    let scenario_name = &request.scenario;
    let schema_data = {
        // Use the preloaded scenarios index
        let index = &*SCENARIOS_INDEX;

        // Determine which section to look in based on message type
        let (section, is_mt) = if request.message_type.starts_with("MT") {
            ("forward", true)
        } else {
            ("reverse", false)
        };

        // Find the matching scenario in the index
        let scenario_file = if let Some(scenarios) = index.get(section).and_then(|s| s.as_array()) {
            let mut found_file = None;

            for scenario in scenarios {
                // For MT, check source field; for MX, check source field
                let source_field = scenario
                    .get("source")
                    .and_then(|s| s.as_str())
                    .unwrap_or("");
                let id_field = scenario.get("id").and_then(|s| s.as_str()).unwrap_or("");

                // Check if this scenario matches
                let matches = if is_mt {
                    source_field == request.message_type && id_field == scenario_name
                } else {
                    // For MX messages, the source is the MX type (e.g., "pacs.008")
                    let mx_type = request.message_type.replace(".", "").to_lowercase();
                    let source_type = source_field.replace(".", "").to_lowercase();
                    source_type == mx_type && id_field == scenario_name
                };

                if matches && let Some(file) = scenario.get("file").and_then(|f| f.as_str()) {
                    found_file = Some(format!("scenarios/{}", file));
                    break;
                }
            }

            match found_file {
                Some(f) => f,
                None => {
                    error!(
                        "Scenario '{}' not found for message type '{}'",
                        scenario_name, request.message_type
                    );
                    return Ok(Json(SampleGenerationResponse {
                        success: false,
                        message_type: request.message_type.clone(),
                        result: None,
                        scenario: Some(scenario_name.to_string()),
                        errors: vec![ReframeError::generation_error(
                            "SCENARIO_NOT_FOUND",
                            format!(
                                "Scenario '{}' not found for message type '{}'",
                                scenario_name, request.message_type
                            ),
                        )],
                        debug_info: None,
                    }));
                }
            }
        } else {
            error!("Invalid scenarios index structure");
            return Ok(Json(SampleGenerationResponse {
                success: false,
                message_type: request.message_type.clone(),
                result: None,
                scenario: Some(scenario_name.to_string()),
                errors: vec![ReframeError::generation_error(
                    "INDEX_STRUCTURE_ERROR",
                    "Invalid scenarios index structure".to_string(),
                )],
                debug_info: None,
            }));
        };

        debug!("Loading scenario from: {}", scenario_file);

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
                            message_type: request.message_type.clone(),
                            result: None,
                            scenario: Some(scenario_name.to_string()),
                            errors: vec![ReframeError::generation_error(
                                "SCENARIO_PARSE_ERROR",
                                format!("Failed to parse scenario file: {}", e),
                            )],
                            debug_info: None,
                        }));
                    }
                }
            }
            Err(_e) => {
                error!("Scenario file not found: {}", scenario_file);
                return Ok(Json(SampleGenerationResponse {
                    success: false,
                    message_type: request.message_type.clone(),
                    result: None,
                    scenario: Some(scenario_name.to_string()),
                    errors: vec![ReframeError::generation_error(
                        "SCENARIO_FILE_NOT_FOUND",
                        format!("Scenario file '{}' not found", scenario_file),
                    )],
                    debug_info: None,
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
                        message_type: request.message_type.clone(),
                        result: Some(result_str.to_string()),
                        scenario: Some(request.scenario.clone()),
                        errors: Vec::new(),
                        debug_info: if request.options.debug {
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
                    }))
                } else {
                    error!("Generation workflow did not produce a string result");
                    Ok(Json(SampleGenerationResponse {
                        success: false,
                        message_type: request.message_type.clone(),
                        result: None,
                        scenario: None,
                        errors: vec![ReframeError::generation_error(
                            "NO_RESULT",
                            "Generation workflow did not produce a valid result".to_string(),
                        )],
                        debug_info: None,
                    }))
                }
            } else {
                // Check for errors in the message
                let workflow_errors = extract_workflow_errors(&message);
                if !workflow_errors.is_empty() {
                    error!("Generation failed with errors: {:?}", workflow_errors);
                    Ok(Json(SampleGenerationResponse {
                        success: false,
                        message_type: request.message_type.clone(),
                        result: None,
                        scenario: None,
                        errors: workflow_errors
                            .into_iter()
                            .map(|e| ReframeError::generation_error("WORKFLOW_ERROR", e))
                            .collect(),
                        debug_info: None,
                    }))
                } else {
                    error!("Generation workflow did not produce a result");
                    Ok(Json(SampleGenerationResponse {
                        success: false,
                        message_type: request.message_type.clone(),
                        result: None,
                        scenario: None,
                        errors: vec![ReframeError::generation_error(
                            "NO_RESULT",
                            "Generation workflow did not produce a result".to_string(),
                        )],
                        debug_info: None,
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
                message_type: request.message_type.clone(),
                result: None,
                scenario: None,
                errors: vec![ReframeError::generation_error(
                    "GENERATION_FAILED",
                    e.to_string(),
                )],
                debug_info: if request.options.debug {
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

    let forward_status = "healthy (Async Engine, Tokio runtime)".to_string();
    let reverse_status = "healthy (Async Engine, Tokio runtime)".to_string();

    Json(HealthResponse {
        status: "running".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        engines: EngineStatus {
            forward: forward_status.clone(),
            reverse: reverse_status.clone(),
        },
        config: Some(crate::types::ConfigInfo {
            thread_count: cpu_count,
        }),
        capabilities: vec![
            "MT-to-MX transformation (Async Engine, Tokio runtime)".to_string(),
            "MX-to-MT transformation (Async Engine, Tokio runtime)".to_string(),
            "High-performance async multi-threaded processing".to_string(),
            "MT sample generation (26 message types)".to_string(),
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

// MT validation endpoint using workflow engine
#[utoipa::path(
    post,
    path = "/validate/mt",
    tag = "validation",
    request_body = ValidationRequest,
    responses(
        (status = 200, description = "Validation completed", body = ValidationResponse),
        (status = 400, description = "Invalid request", body = ValidationResponse)
    )
)]
#[instrument(skip(state, request), fields(message_length = request.message.len()))]
pub async fn validate_mt(
    State(state): State<AppState>,
    Json(request): Json<ValidationRequest>,
) -> Result<Json<ValidationResponse>, StatusCode> {
    let start_time = Instant::now();

    info!("🔍 Processing MT validation request");
    debug!("Request options: {:?}", request.options);

    // Prepare message for validation workflow
    let payload_value = Value::String(request.message.clone());
    let mut message = Message::new(Arc::new(payload_value));

    // Set metadata for workflow detection (generic "MT" for routing)
    if let Some(metadata_obj) = message.metadata_mut().as_object_mut() {
        metadata_obj.insert("message_type".to_string(), "MT".into());
    }

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
                error!("MT validation failed with errors: {:?}", workflow_errors);
                return Ok(Json(ValidationResponse {
                    success: false,
                    message_type: None,
                    canonical_json: None,
                    errors: workflow_errors
                        .into_iter()
                        .map(|e| ReframeError::transformation_error("VALIDATION_ERROR", e))
                        .collect(),
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
                    "✅ MT validation completed in {}ms - Valid: {}, Type: {:?}",
                    processing_time, valid, message_type
                );

                Ok(Json(ValidationResponse {
                    success: valid,
                    message_type,
                    canonical_json: None,
                    errors,
                }))
            } else {
                error!("Validation workflow did not produce output");
                Ok(Json(ValidationResponse {
                    success: false,
                    message_type: None,
                    canonical_json: None,
                    errors: vec![ReframeError::internal_error(
                        "Validation workflow did not produce output".to_string(),
                    )],
                }))
            }
        }
        Err(e) => {
            error!(
                error = %e,
                "MT validation failed"
            );

            Ok(Json(ValidationResponse {
                success: false,
                message_type: None,
                canonical_json: None,
                errors: vec![ReframeError::internal_error(e.to_string())],
            }))
        }
    }
}

// MX validation endpoint
#[utoipa::path(
    post,
    path = "/validate/mx",
    tag = "validation",
    request_body = ValidationRequest,
    responses(
        (status = 200, description = "Validation completed", body = ValidationResponse),
        (status = 400, description = "Invalid request", body = ValidationResponse)
    )
)]
#[instrument(skip(state, request), fields(message_length = request.message.len()))]
pub async fn validate_mx(
    State(state): State<AppState>,
    Json(request): Json<ValidationRequest>,
) -> Result<Json<ValidationResponse>, StatusCode> {
    let start_time = Instant::now();

    info!("🔍 Processing MX validation request");
    debug!("Request options: {:?}", request.options);

    // Prepare message for validation workflow
    let payload_value = Value::String(request.message.clone());
    let mut message = Message::new(Arc::new(payload_value));

    // Set metadata for workflow detection (generic "MX" for routing)
    if let Some(metadata_obj) = message.metadata_mut().as_object_mut() {
        metadata_obj.insert("message_type".to_string(), "MX".into());
    }

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
                error!("MX validation failed with errors: {:?}", workflow_errors);
                return Ok(Json(ValidationResponse {
                    success: false,
                    message_type: None,
                    canonical_json: None,
                    errors: workflow_errors
                        .into_iter()
                        .map(|e| ReframeError::transformation_error("VALIDATION_ERROR", e))
                        .collect(),
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
                    "✅ MX validation completed in {}ms - Valid: {}, Type: {:?}",
                    processing_time, valid, message_type
                );

                Ok(Json(ValidationResponse {
                    success: valid,
                    message_type,
                    canonical_json: None,
                    errors,
                }))
            } else {
                error!("Validation workflow did not produce output");
                Ok(Json(ValidationResponse {
                    success: false,
                    message_type: None,
                    canonical_json: None,
                    errors: vec![ReframeError::internal_error(
                        "Validation workflow did not produce output".to_string(),
                    )],
                }))
            }
        }
        Err(e) => {
            error!(
                error = %e,
                "MX validation failed"
            );

            Ok(Json(ValidationResponse {
                success: false,
                message_type: None,
                canonical_json: None,
                errors: vec![ReframeError::internal_error(e.to_string())],
            }))
        }
    }
}
