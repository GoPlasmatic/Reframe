use axum::{
    Json,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use dataflow_rs::engine::message::Message;
use quick_xml::Reader;
use serde_json::Value;
use std::time::Instant;
use swift_mt_message::SwiftParser;
use tracing::{Span, debug, error, info, instrument, warn};
use uuid::Uuid;

use crate::engine::reload_engines;
use crate::parse_mx::ParseMX;
use crate::sample_generator::{
    generate_mt_from_config, generate_mx_from_config, is_supported_message_type,
    is_supported_mx_type,
};
use crate::types::{
    AppState, DebugInfo, EngineStatus, ErrorType, HealthResponse, MessageCategory, ReframeError,
    ReloadResponse, SampleDebugInfo, SampleGenerationRequest, SampleGenerationResponse,
    TransformationRequest, TransformationResponse, ValidationRequest, ValidationResponse,
};
use mx_message::parse_result::ParserConfig as MxParserConfig;

/// Validates that the given string is well-formed XML
fn validate_xml_well_formed(xml_content: &str) -> Result<(), String> {
    if xml_content.trim().is_empty() {
        return Err("XML content is empty".to_string());
    }

    let mut reader = Reader::from_str(xml_content);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(quick_xml::events::Event::Eof) => break,
            Ok(_) => {}
            Err(e) => {
                return Err(format!("XML parsing error: {e}"));
            }
        }
        buf.clear();
    }

    Ok(())
}

/// Validates that the given string is a well-formed SWIFT MT message
fn validate_mt_well_formed(mt_content: &str) -> Result<(), String> {
    if mt_content.trim().is_empty() {
        return Err("MT content is empty".to_string());
    }

    // Normalize line endings - convert \n to \r\n if needed for SWIFT format
    let normalized_content = if mt_content.contains("\r\n") {
        mt_content.to_string()
    } else {
        mt_content.replace('\n', "\r\n")
    };

    match SwiftParser::parse_auto(&normalized_content) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("SWIFT MT parsing error: {e:?}")),
    }
}

/// Extracts workflow errors from the message object
pub fn extract_workflow_errors(message: &Message) -> Vec<String> {
    let mut errors = Vec::new();

    // Check for errors in the message's error array
    if !message.errors.is_empty() {
        for error in &message.errors {
            let error_msg = &error.error_message;
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
    let mut message = Message::new(&payload_value);
    message.data = serde_json::json!({}); // Initialize data field as empty object
    message.metadata = serde_json::json!({
        "transformation_direction": "mt-to-mx"
    });

    // Process message using the threaded forward engine
    match state.forward_engine.process_message(message.clone()).await {
        Ok(result_message) => {
            let processing_time = start_time.elapsed().as_millis() as u64;
            info!(
                "✅ MT to MX transformation completed in {}ms",
                processing_time
            );
            // Update message with result
            message = result_message;

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
            match message.data.get("result") {
                Some(result) if !result.is_null() => {
                    // Handle both string and array results (MT to MX can produce multiple messages)
                    match result {
                        Value::String(s) if !s.trim().is_empty() => {
                            // Single string result - validate XML
                            match validate_xml_well_formed(s) {
                                Ok(()) => Ok(Json(TransformationResponse {
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
                                })),
                                Err(xml_error) => {
                                    error!(
                                        message_type = "MT",
                                        error_type = "MALFORMED_XML",
                                        "Transformation produced malformed XML: {}",
                                        xml_error
                                    );
                                    Ok(Json(TransformationResponse {
                                        success: false,
                                        result: None,
                                        debug_info: if request.options.debug {
                                            let message_json =
                                                serde_json::to_value(&message).unwrap();
                                            Some(DebugInfo {
                                                engine_state: "forward (threaded)".to_string(),
                                                workflow_execution: vec![
                                                    "Failed - Malformed XML output".to_string(),
                                                ],
                                                intermediate_data: message_json,
                                            })
                                        } else {
                                            None
                                        },
                                        errors: vec![ReframeError::transformation_error(
                                            "MALFORMED_XML",
                                            format!(
                                                "Transformation produced malformed XML: {xml_error}"
                                            ),
                                        )],
                                        warnings: Vec::new(),
                                    }))
                                }
                            }
                        }
                        Value::Array(arr) if !arr.is_empty() => {
                            // Multiple results - validate each XML message
                            let mut errors = Vec::new();
                            for (i, item) in arr.iter().enumerate() {
                                if let Some(xml_str) = item.as_str() {
                                    if let Err(xml_error) = validate_xml_well_formed(xml_str) {
                                        errors.push(format!("Message {}: {}", i + 1, xml_error));
                                    }
                                } else {
                                    errors.push(format!("Message {}: Not a valid string", i + 1));
                                }
                            }

                            if errors.is_empty() {
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
                            } else {
                                error!(
                                    message_type = "MT",
                                    error_type = "XML_VALIDATION_ERROR",
                                    error_count = errors.len(),
                                    "Transformation produced malformed XML in multiple messages: {:?}",
                                    errors
                                );
                                Ok(Json(TransformationResponse {
                                    success: false,
                                    result: None,
                                    debug_info: if request.options.debug {
                                        let message_json = serde_json::to_value(&message).unwrap();
                                        Some(DebugInfo {
                                            engine_state: "forward (threaded)".to_string(),
                                            workflow_execution: vec![format!(
                                                "Failed - XML validation errors in {} messages",
                                                errors.len()
                                            )],
                                            intermediate_data: message_json,
                                        })
                                    } else {
                                        None
                                    },
                                    errors: errors
                                        .into_iter()
                                        .map(|e| {
                                            ReframeError::transformation_error(
                                                "XML_VALIDATION_ERROR",
                                                e,
                                            )
                                        })
                                        .collect(),
                                    warnings: Vec::new(),
                                }))
                            }
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
                    let message_json = serde_json::to_value(message).unwrap();
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
    let mut message = Message::new(&payload_value);
    message.data = serde_json::json!({}); // Initialize data field as empty object
    message.metadata = serde_json::json!({
        "transformation_direction": "mx-to-mt"
    });

    // Process message using the threaded reverse engine
    match state.reverse_engine.process_message(message.clone()).await {
        Ok(result_message) => {
            let processing_time = start_time.elapsed().as_millis() as u64;
            info!(
                "✅ MX to MT transformation completed in {}ms",
                processing_time
            );
            // Update message with result
            message = result_message;

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
            match message.data.get("result") {
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
                            // Validate that the result is well-formed SWIFT MT message
                            match validate_mt_well_formed(result_str) {
                                Ok(()) => Ok(Json(TransformationResponse {
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
                                })),
                                Err(mt_error) => {
                                    error!(
                                        "❌ MX to MT transformation produced malformed SWIFT MT message: {}",
                                        mt_error
                                    );
                                    Ok(Json(TransformationResponse {
                                        success: false,
                                        result: None,
                                        debug_info: if request.options.debug {
                                            let message_json =
                                                serde_json::to_value(&message).unwrap();
                                            Some(DebugInfo {
                                                engine_state: "reverse (threaded)".to_string(),
                                                workflow_execution: vec![
                                                    "Failed - Malformed SWIFT MT output"
                                                        .to_string(),
                                                ],
                                                intermediate_data: message_json,
                                            })
                                        } else {
                                            None
                                        },
                                        errors: vec![ReframeError::transformation_error(
                                            "MALFORMED_MT",
                                            format!(
                                                "Transformation produced malformed SWIFT MT message: {mt_error}"
                                            ),
                                        )],
                                        warnings: Vec::new(),
                                    }))
                                }
                            }
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
                    let message_json = serde_json::to_value(message).unwrap();
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

// Helper function to detect message category
fn detect_message_category(message_type: &str) -> MessageCategory {
    if message_type.starts_with("MT") {
        MessageCategory::MT
    } else if message_type.starts_with("pacs")
        || message_type.starts_with("camt")
        || message_type.starts_with("pain")
        || message_type.starts_with("admi")
    {
        MessageCategory::MX
    } else {
        MessageCategory::Unknown
    }
}

// Unified sample generation endpoint
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
#[instrument(skip(request), fields(message_type = request.message_type.as_str()))]
pub async fn generate_sample(
    Json(request): Json<SampleGenerationRequest>,
) -> Result<Json<SampleGenerationResponse>, StatusCode> {
    let start_time = Instant::now();

    info!(
        "🔄 Processing sample generation request for {}",
        request.message_type
    );
    debug!("Request options: {:?}", request.options);

    // Detect message category and route to appropriate generator
    let result = match detect_message_category(&request.message_type) {
        MessageCategory::MT => {
            if !is_supported_message_type(&request.message_type) {
                error!(
                    message_type = %request.message_type,
                    "Unsupported MT message type"
                );
                return Ok(Json(SampleGenerationResponse {
                    success: false,
                    message_type: request.message_type.clone(),
                    result: None,
                    scenario: None,
                    errors: vec![ReframeError::generation_error(
                        "UNSUPPORTED_TYPE",
                        format!("Unsupported MT message type: {}", request.message_type),
                    )],
                    debug_info: None,
                }));
            }
            generate_mt_from_config(&request.config, &request.message_type, &request.options).await
        }
        MessageCategory::MX => {
            if !is_supported_mx_type(&request.message_type) {
                error!(
                    message_type = %request.message_type,
                    "Unsupported MX message type"
                );
                return Ok(Json(SampleGenerationResponse {
                    success: false,
                    message_type: request.message_type.clone(),
                    result: None,
                    scenario: None,
                    errors: vec![ReframeError::generation_error(
                        "UNSUPPORTED_TYPE",
                        format!("Unsupported MX message type: {}", request.message_type),
                    )],
                    debug_info: None,
                }));
            }
            generate_mx_from_config(&request.config, &request.message_type, &request.options).await
        }
        MessageCategory::Unknown => {
            error!(
                message_type = %request.message_type,
                "Unknown message type format"
            );
            return Ok(Json(SampleGenerationResponse {
                success: false,
                message_type: request.message_type.clone(),
                result: None,
                scenario: None,
                errors: vec![ReframeError::generation_error(
                    "UNSUPPORTED_TYPE",
                    format!(
                        "Unknown message type format: {}. Expected MT* or pacs*/camt*/pain*",
                        request.message_type
                    ),
                )],
                debug_info: None,
            }));
        }
    };

    match result {
        Ok((message, generated_json)) => {
            let processing_time = start_time.elapsed().as_millis() as u64;
            info!("✅ Sample generation completed in {}ms", processing_time);

            // TODO: If validation is enabled, we should validate the generated message
            // and collect validation errors. For now, we'll return an empty list.
            let errors = Vec::new();

            Ok(Json(SampleGenerationResponse {
                success: true,
                message_type: request.message_type.clone(),
                result: Some(message),
                scenario: request
                    .config
                    .get("scenario")
                    .and_then(|s| s.as_str())
                    .map(|s| s.to_string()),
                errors,
                debug_info: if request.options.debug {
                    Some(SampleDebugInfo {
                        scenario_config: request.config,
                        generation_time_ms: processing_time,
                        warnings: Vec::new(),
                        generated_json: Some(generated_json),
                    })
                } else {
                    None
                },
            }))
        }
        Err(e) => {
            error!(
                error = %e,
                "Sample generation failed"
            );

            // Try to extract any JSON from the error message for debugging
            let error_str = e.to_string();
            let generated_json = if error_str.contains("Generated JSON:") {
                // Try to extract the JSON from the error message
                if let Some(json_start) = error_str.find("Generated JSON:") {
                    let json_str = &error_str[json_start + 15..];
                    serde_json::from_str(json_str).ok()
                } else {
                    None
                }
            } else {
                None
            };

            // Clean up the error message to remove the large JSON dump
            let clean_error = if let Some(json_idx) = error_str.find("Generated JSON:") {
                error_str[..json_idx].trim().to_string()
            } else {
                error_str
            };

            Ok(Json(SampleGenerationResponse {
                success: false,
                message_type: request.message_type.clone(),
                result: None,
                scenario: None,
                errors: vec![ReframeError::generation_error(
                    "GENERATION_FAILED",
                    clean_error,
                )],
                debug_info: if request.options.debug {
                    Some(SampleDebugInfo {
                        scenario_config: request.config.clone(),
                        generation_time_ms: start_time.elapsed().as_millis() as u64,
                        warnings: Vec::new(),
                        generated_json,
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
pub async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    // RayonEngine is always healthy if it exists
    // Get actual thread count from the engines
    let forward_thread_count = state.forward_engine.thread_count();
    let reverse_thread_count = state.reverse_engine.thread_count();

    let thread_count = std::env::var("REFRAME_THREAD_COUNT")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or_else(num_cpus::get);

    let forward_status = format!("healthy (RayonEngine, {} workers)", forward_thread_count);
    let reverse_status = format!("healthy (RayonEngine, {} workers)", reverse_thread_count);

    Json(HealthResponse {
        status: "running".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        engines: EngineStatus {
            forward: forward_status.clone(),
            reverse: reverse_status.clone(),
        },
        config: Some(crate::types::ConfigInfo { thread_count }),
        capabilities: vec![
            format!(
                "MT-to-MX transformation (RayonEngine, {} workers)",
                forward_thread_count
            ),
            format!(
                "MX-to-MT transformation (RayonEngine, {} workers)",
                reverse_thread_count
            ),
            "High-performance CPU-optimized processing with work-stealing".to_string(),
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
                message: format!(
                    "Workflows reloaded successfully in {reload_time}ms (threaded mode)"
                ),
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

// MT validation endpoint
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
#[instrument(skip(request), fields(message_length = request.message.len()))]
pub async fn validate_mt(
    Json(request): Json<ValidationRequest>,
) -> Result<Json<ValidationResponse>, StatusCode> {
    let start_time = Instant::now();

    info!("🔍 Processing MT validation request");
    debug!("Request options: {:?}", request.options);

    // Normalize line endings - convert \n to \r\n if needed for SWIFT format
    let normalized_content = if request.message.contains("\r\n") {
        request.message.clone()
    } else {
        request.message.replace('\n', "\r\n")
    };

    // Parse the MT message with config based on fail_fast option
    let parse_result = if request.options.fail_fast {
        let config = swift_mt_message::errors::ParserConfig {
            fail_fast: false,
            validate_optional_fields: true,
            collect_all_errors: true,
        };
        SwiftParser::with_config(config).parse_message_auto(&normalized_content)
    } else {
        SwiftParser::parse_auto(&normalized_content)
    };

    match parse_result {
        Ok(parsed_message) => {
            let message_type = parsed_message.message_type().to_string();
            info!(
                "✅ MT validation completed in {}ms - Message type: {}",
                start_time.elapsed().as_millis(),
                message_type
            );

            let mut errors = Vec::new();

            // Convert to canonical JSON if requested - now supports all MT types
            let canonical_json = if request.options.canonical {
                // Convert the parsed message to JSON
                match serde_json::to_value(&parsed_message) {
                    Ok(json) => Some(json),
                    Err(e) => {
                        debug!("Failed to convert MT message to JSON: {}", e);
                        None
                    }
                }
            } else {
                None
            };

            // Perform business validation if requested
            if request.options.business_validation {
                crate::validation_helpers::perform_mt_business_validation(
                    &parsed_message,
                    &mut errors,
                );
            }

            // Determine success based on presence of parser or business validation errors
            let has_critical_errors = errors.iter().any(|e| {
                matches!(
                    e.error_type,
                    ErrorType::ParserError | ErrorType::BusinessValidationError
                )
            });

            Ok(Json(ValidationResponse {
                success: !has_critical_errors,
                message_type: Some(message_type),
                canonical_json,
                errors,
            }))
        }
        Err(parse_error) => {
            error!(
                error = ?parse_error,
                "MT validation failed"
            );

            // Extract detailed parse errors
            let errors = extract_mt_parse_errors(parse_error);

            Ok(Json(ValidationResponse {
                success: false,
                message_type: None,
                canonical_json: None,
                errors,
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
#[instrument(skip(request), fields(message_length = request.message.len()))]
pub async fn validate_mx(
    Json(request): Json<ValidationRequest>,
) -> Result<Json<ValidationResponse>, StatusCode> {
    let start_time = Instant::now();

    info!("🔍 Processing MX validation request");
    debug!("Request options: {:?}", request.options);

    // First validate XML well-formedness
    if let Err(xml_error) = validate_xml_well_formed(&request.message) {
        error!(
            error = %xml_error,
            "MX validation failed - XML malformed"
        );
        return Ok(Json(ValidationResponse {
            success: false,
            message_type: None,
            canonical_json: None,
            errors: vec![ReframeError {
                error_type: ErrorType::ParserError,
                code: "MX_XML_MALFORMED".to_string(),
                message: xml_error,
                field: None,
                location: None,
                details: None,
            }],
        }));
    }

    // Try to parse the MX message and extract message type
    let document_xmlns = ParseMX::extract_document_xmlns(&request.message);
    let app_hdr_content = ParseMX::extract_app_hdr_content(&request.message);
    let document_content = ParseMX::extract_document_content(&request.message);

    match ParseMX::extract_message_type(document_xmlns.clone(), app_hdr_content.clone()) {
        Ok(message_type) => {
            info!(
                "✅ MX validation completed in {}ms - Message type: {}",
                start_time.elapsed().as_millis(),
                message_type
            );

            let mut errors = Vec::new();

            // Extract canonical JSON if requested
            let canonical_json = if request.options.canonical {
                if let (Some(app_hdr), Some(doc_content)) =
                    (app_hdr_content.clone(), document_content.clone())
                {
                    match (
                        ParseMX::parse_header(&message_type, &app_hdr),
                        ParseMX::parse_document(&message_type, &doc_content),
                    ) {
                        (Ok(header), Ok(document)) => Some(serde_json::json!({
                            "header": header,
                            "document": document,
                            "message_type": message_type
                        })),
                        _ => None,
                    }
                } else {
                    None
                }
            } else {
                None
            };

            // Perform full MX validation
            if let Some(doc_content) = document_content {
                debug!(
                    "MX Validation: Document content available, length: {} bytes",
                    doc_content.len()
                );

                // Create parser config based on request options
                let mx_config = MxParserConfig {
                    fail_fast: false,
                    validate_optional_fields: true,
                    collect_all_errors: true,
                };

                // Validate based on message type using the refactored approach
                validate_mx_by_type(
                    &message_type,
                    &app_hdr_content,
                    &doc_content,
                    &mx_config,
                    &mut errors,
                    request.options.business_validation,
                );
            } else {
                info!("MX Validation: No document content extracted from MX message!");
                errors.push(ReframeError {
                    error_type: ErrorType::ParserError,
                    code: "MX_NO_DOCUMENT_CONTENT".to_string(),
                    message: "Failed to extract document content from MX message".to_string(),
                    field: None,
                    location: None,
                    details: None,
                });
            }

            // Determine success based on presence of parser or business validation errors
            let has_critical_errors = errors.iter().any(|e| {
                matches!(
                    e.error_type,
                    ErrorType::ParserError | ErrorType::BusinessValidationError
                )
            });

            Ok(Json(ValidationResponse {
                success: !has_critical_errors,
                message_type: Some(message_type),
                canonical_json,
                errors,
            }))
        }
        Err(parse_error) => {
            error!(
                error = ?parse_error,
                "MX validation failed"
            );

            let error_message = format!("{parse_error:?}");
            let mut errors = vec![ReframeError {
                error_type: ErrorType::ParserError,
                code: "MX_PARSE_ERROR".to_string(),
                message: error_message.clone(),
                field: None,
                location: None,
                details: None,
            }];

            // Try to extract more specific error information
            if error_message.contains("Unknown message type") {
                errors.push(ReframeError {
                    error_type: ErrorType::ParserError,
                    code: "MX_UNKNOWN_MESSAGE_TYPE".to_string(),
                    message: "Message type is not recognized or supported".to_string(),
                    field: Some("MsgDefIdr".to_string()),
                    location: None,
                    details: None,
                });
            }

            Ok(Json(ValidationResponse {
                success: false,
                message_type: None,
                canonical_json: None,
                errors,
            }))
        }
    }
}

// Centralized MX validation by message type
fn validate_mx_by_type(
    message_type: &str,
    app_hdr_content: &Option<String>,
    doc_content: &str,
    mx_config: &MxParserConfig,
    errors: &mut Vec<ReframeError>,
    business_validation: bool,
) {
    use crate::validate_mx_message;
    use mx_message::document::*;
    use mx_message::header::*;

    match message_type {
        // PACS messages
        "pacs.008.001.08" => {
            validate_mx_message!(
                message_type,
                app_hdr_content,
                doc_content,
                mx_config,
                errors,
                {
                    header: bah_pacs_008_001_08::BusinessApplicationHeaderV02,
                    document: pacs_008_001_08::FIToFICustomerCreditTransferV08
                }
            );
        }
        "pacs.009.001.08" => {
            validate_mx_message!(
                message_type,
                app_hdr_content,
                doc_content,
                mx_config,
                errors,
                {
                    header: bah_pacs_009_001_08::BusinessApplicationHeaderV02,
                    document: pacs_009_001_08::FinancialInstitutionCreditTransferV08
                }
            );
        }
        "pacs.004.001.09" => {
            validate_mx_message!(
                message_type,
                app_hdr_content,
                doc_content,
                mx_config,
                errors,
                {
                    header: bah_pacs_004_001_09::BusinessApplicationHeaderV02,
                    document: pacs_004_001_09::PaymentReturnV09
                }
            );
        }
        "pacs.002.001.10" => {
            validate_mx_message!(
                message_type,
                app_hdr_content,
                doc_content,
                mx_config,
                errors,
                {
                    header: bah_pacs_002_001_10::BusinessApplicationHeaderV02,
                    document: pacs_002_001_10::FIToFIPaymentStatusReportV10
                }
            );
        }
        "pacs.003.001.08" => {
            validate_mx_message!(
                message_type,
                app_hdr_content,
                doc_content,
                mx_config,
                errors,
                {
                    header: bah_pacs_003_001_08::BusinessApplicationHeaderV02,
                    document: pacs_003_001_08::FIToFICustomerDirectDebitV08
                }
            );
        }

        // CAMT messages
        "camt.054.001.08" => {
            validate_mx_message!(
                message_type,
                app_hdr_content,
                doc_content,
                mx_config,
                errors,
                {
                    header: bah_camt_054_001::BusinessApplicationHeaderV02,
                    document: camt_054_001_08::BankToCustomerDebitCreditNotificationV08
                }
            );
        }
        "camt.052.001.08" => {
            validate_mx_message!(
                message_type,
                app_hdr_content,
                doc_content,
                mx_config,
                errors,
                {
                    header: bah_camt_052_001_08::BusinessApplicationHeaderV02,
                    document: camt_052_001_08::BankToCustomerAccountReportV08
                }
            );
        }
        "camt.053.001.08" => {
            validate_mx_message!(
                message_type,
                app_hdr_content,
                doc_content,
                mx_config,
                errors,
                {
                    header: bah_camt_053_001_08::BusinessApplicationHeaderV02,
                    document: camt_053_001_08::BankToCustomerStatementV08
                }
            );
        }
        "camt.056.001.08" => {
            validate_mx_message!(
                message_type,
                app_hdr_content,
                doc_content,
                mx_config,
                errors,
                {
                    header: bah_camt_056_001_08::BusinessApplicationHeaderV02,
                    document: camt_056_001_08::FIToFIPaymentCancellationRequestV08
                }
            );
        }
        "camt.057.001.06" => {
            validate_mx_message!(
                message_type,
                app_hdr_content,
                doc_content,
                mx_config,
                errors,
                {
                    header: bah_camt_057_001_06::BusinessApplicationHeaderV02,
                    document: camt_057_001_06::NotificationToReceiveV06
                }
            );
        }
        "camt.025.001.08" => {
            validate_mx_message!(
                message_type,
                app_hdr_content,
                doc_content,
                mx_config,
                errors,
                {
                    header: bah_camt_025_001_08::BusinessApplicationHeaderV02,
                    document: camt_025_001_08::ReceiptV08
                }
            );
        }
        "camt.029.001.09" => {
            validate_mx_message!(
                message_type,
                app_hdr_content,
                doc_content,
                mx_config,
                errors,
                {
                    header: bah_camt_029_001::BusinessApplicationHeaderV02,
                    document: camt_029_001_09::ResolutionOfInvestigationV09
                }
            );
        }
        "camt.060.001.05" => {
            validate_mx_message!(
                message_type,
                app_hdr_content,
                doc_content,
                mx_config,
                errors,
                {
                    header: bah_camt_060_001_05::BusinessApplicationHeaderV02,
                    document: camt_060_001_05::AccountReportingRequestV05
                }
            );
        }

        // PAIN messages
        "pain.001.001.09" => {
            validate_mx_message!(
                message_type,
                app_hdr_content,
                doc_content,
                mx_config,
                errors,
                {
                    header: bah_pain_001_001_09::BusinessApplicationHeaderV02,
                    document: pain_001_001_09::CustomerCreditTransferInitiationV09
                }
            );
        }
        "pain.002.001.10" => {
            validate_mx_message!(
                message_type,
                app_hdr_content,
                doc_content,
                mx_config,
                errors,
                {
                    header: bah_pain_002_001_10::BusinessApplicationHeaderV02,
                    document: pain_002_001_10::CustomerPaymentStatusReportV10
                }
            );
        }
        "pain.008.001.08" => {
            validate_mx_message!(
                message_type,
                app_hdr_content,
                doc_content,
                mx_config,
                errors,
                {
                    header: bah_pain_008_001_08::BusinessApplicationHeaderV02,
                    document: pain_008_001_08::CustomerDirectDebitInitiationV08
                }
            );
        }

        _ => {
            // For unsupported message types
            if business_validation {
                errors.push(ReframeError {
                    error_type: ErrorType::Info,
                    code: "MX_VALIDATION_LIMITED".to_string(),
                    message: format!(
                        "Full validation not yet implemented for message type: {message_type}"
                    ),
                    field: None,
                    location: None,
                    details: None,
                });
            }
        }
    }
}

// Helper function to extract MT parse errors
fn extract_mt_parse_errors(parse_error: swift_mt_message::errors::ParseError) -> Vec<ReframeError> {
    let mut errors = Vec::new();

    // Check if it's a MultipleErrors variant containing individual errors
    match parse_error {
        swift_mt_message::errors::ParseError::MultipleErrors(individual_errors) => {
            // Extract each individual error into separate array entries
            for err in individual_errors {
                errors.push(convert_mt_parse_error(&err));
            }
        }
        _ => {
            // Handle single error
            errors.push(convert_mt_parse_error(&parse_error));
        }
    }

    errors
}

// Helper to convert a single MT parse error to ReframeError
fn convert_mt_parse_error(err: &swift_mt_message::errors::ParseError) -> ReframeError {
    let (code, message, field, location) = match err {
        swift_mt_message::errors::ParseError::FieldParsingFailed {
            field_tag,
            field_type,
            position,
            original_error,
        } => (
            format!("MT_{field_tag}_PARSE_ERROR"),
            format!("Failed to parse field {field_tag} ({field_type}): {original_error}"),
            Some(field_tag.clone()),
            Some(format!("Line {position}")),
        ),
        swift_mt_message::errors::ParseError::MissingRequiredField {
            field_tag,
            field_name,
            message_type,
            ..
        } => (
            format!("MT_{field_tag}_MISSING"),
            format!("Missing required field {field_tag} ({field_name}) in {message_type}"),
            Some(field_tag.clone()),
            None,
        ),
        swift_mt_message::errors::ParseError::InvalidFieldFormat(err_box) => (
            format!("MT_{}_FORMAT_ERROR", err_box.field_tag),
            format!(
                "Invalid field format - Field: {}, Component: {}, Value: '{}', Expected: {}",
                err_box.field_tag, err_box.component_name, err_box.value, err_box.format_spec
            ),
            Some(err_box.field_tag.clone()),
            err_box.position.map(|p| format!("Position {p}")),
        ),
        swift_mt_message::errors::ParseError::ComponentParseError {
            field_tag,
            component_name,
            component_index,
            expected_format,
            actual_value,
        } => (
            format!("MT_{field_tag}_COMPONENT_ERROR"),
            format!(
                "Component parse error in field {field_tag}: {component_name} (index {component_index}), expected: {expected_format}, got: '{actual_value}'"
            ),
            Some(field_tag.clone()),
            None,
        ),
        swift_mt_message::errors::ParseError::InvalidBlockStructure { block, message } => (
            format!("MT_BLOCK_{block}_ERROR"),
            format!("Invalid block {block} structure: {message}"),
            None,
            None,
        ),
        _ => ("MT_PARSE_ERROR".to_string(), format!("{err}"), None, None),
    };

    ReframeError {
        error_type: ErrorType::ParserError,
        code,
        message,
        field,
        location,
        details: None,
    }
}
