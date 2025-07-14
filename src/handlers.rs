use axum::{Json, extract::State, http::StatusCode};
use dataflow_rs::engine::message::Message;
use quick_xml::Reader;
use serde_json::Value;
use std::time::Instant;
use swift_mt_message::SwiftParser;
use tracing::{debug, error, info, instrument};

use crate::sample_generator::{generate_mt_from_config, is_supported_message_type};
use crate::engine::reload_engines;
use crate::types::{
    AppState, DebugInfo, EngineStatus, HealthResponse, ReloadResponse, SampleGenerationRequest,
    TransformationRequest, TransformationResponse,
};

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
fn extract_workflow_errors(message: &Message) -> Vec<String> {
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

// New endpoint handlers
#[instrument(skip(state, request), fields(message_length = request.message.len()))]
pub async fn transform_mt_to_mx(
    State(state): State<AppState>,
    Json(request): Json<TransformationRequest>,
) -> Result<Json<TransformationResponse>, StatusCode> {
    let start_time = Instant::now();

    info!("🔄 Processing MT to MX transformation request");
    debug!("Request options: {:?}", request.options);

    // Use forward engine for MT to MX transformation
    let engine = state.forward_engine.lock().await;

    let payload_value = Value::String(request.message.clone());
    let mut message = Message::new(&payload_value);
    message.metadata = serde_json::json!({
        "transformation_direction": "mt-to-mx"
    });

    let processing_time = start_time.elapsed().as_millis() as u64;

    match engine.process_message(&mut message).await {
        Ok(_) => {
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
                    transformed_message: None,
                    debug_info: if request.options.include_debug {
                        let message_json = serde_json::to_value(&message).unwrap();
                        Some(DebugInfo {
                            engine_state: "forward".to_string(),
                            workflow_execution: vec!["Failed - Validation errors".to_string()],
                            intermediate_data: message_json,
                        })
                    } else {
                        None
                    },
                    errors: workflow_errors,
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
                                    transformed_message: Some(result.clone()),
                                    debug_info: if request.options.include_debug {
                                        let message_json = serde_json::to_value(&message).unwrap();
                                        Some(DebugInfo {
                                            engine_state: "forward".to_string(),
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
                                        "❌ MT to MX transformation produced malformed XML: {}",
                                        xml_error
                                    );
                                    Ok(Json(TransformationResponse {
                                        success: false,
                                        transformed_message: None,
                                        debug_info: if request.options.include_debug {
                                            let message_json =
                                                serde_json::to_value(&message).unwrap();
                                            Some(DebugInfo {
                                                engine_state: "forward".to_string(),
                                                workflow_execution: vec![
                                                    "Failed - Malformed XML output".to_string(),
                                                ],
                                                intermediate_data: message_json,
                                            })
                                        } else {
                                            None
                                        },
                                        errors: vec![format!(
                                            "Transformation produced malformed XML: {}",
                                            xml_error
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
                                    transformed_message: Some(result.clone()),
                                    debug_info: if request.options.include_debug {
                                        let message_json = serde_json::to_value(&message).unwrap();
                                        Some(DebugInfo {
                                            engine_state: "forward".to_string(),
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
                                    "❌ MT to MX transformation produced malformed XML in multiple messages: {:?}",
                                    errors
                                );
                                Ok(Json(TransformationResponse {
                                    success: false,
                                    transformed_message: None,
                                    debug_info: if request.options.include_debug {
                                        let message_json = serde_json::to_value(&message).unwrap();
                                        Some(DebugInfo {
                                            engine_state: "forward".to_string(),
                                            workflow_execution: vec![format!(
                                                "Failed - XML validation errors in {} messages",
                                                errors.len()
                                            )],
                                            intermediate_data: message_json,
                                        })
                                    } else {
                                        None
                                    },
                                    errors,
                                    warnings: Vec::new(),
                                }))
                            }
                        }
                        _ => {
                            error!("❌ MT to MX transformation produced empty or invalid result");
                            Ok(Json(TransformationResponse {
                                success: false,
                                transformed_message: None,
                                debug_info: if request.options.include_debug {
                                    let message_json = serde_json::to_value(message).unwrap();
                                    Some(DebugInfo {
                                        engine_state: "forward".to_string(),
                                        workflow_execution: vec![
                                            "Failed - Empty or invalid result".to_string(),
                                        ],
                                        intermediate_data: message_json,
                                    })
                                } else {
                                    None
                                },
                                errors: vec![
                                    "Transformation completed but produced empty or invalid result"
                                        .to_string(),
                                ],
                                warnings: Vec::new(),
                            }))
                        }
                    }
                }
                _ => {
                    error!("❌ MT to MX transformation completed but no valid result found");
                    Ok(Json(TransformationResponse {
                        success: false,
                        transformed_message: None,
                        debug_info: if request.options.include_debug {
                            let message_json = serde_json::to_value(message).unwrap();
                            Some(DebugInfo {
                                engine_state: "forward".to_string(),
                                workflow_execution: vec!["Failed - No result produced".to_string()],
                                intermediate_data: message_json,
                            })
                        } else {
                            None
                        },
                        errors: vec![
                            "Transformation completed but no valid result was produced".to_string(),
                        ],
                        warnings: Vec::new(),
                    }))
                }
            }
        }
        Err(e) => {
            error!("❌ MT to MX transformation failed: {}", e);

            Ok(Json(TransformationResponse {
                success: false,
                transformed_message: None,
                debug_info: if request.options.include_debug {
                    let message_json = serde_json::to_value(message).unwrap();
                    Some(DebugInfo {
                        engine_state: "forward".to_string(),
                        workflow_execution: vec![format!("Failed - Engine error: {}", e)],
                        intermediate_data: message_json,
                    })
                } else {
                    None
                },
                errors: vec![e.to_string()],
                warnings: Vec::new(),
            }))
        }
    }
}

#[instrument(skip(state, request), fields(message_length = request.message.len()))]
pub async fn transform_mx_to_mt(
    State(state): State<AppState>,
    Json(request): Json<TransformationRequest>,
) -> Result<Json<TransformationResponse>, StatusCode> {
    let start_time = Instant::now();

    info!("🔄 Processing MX to MT transformation request");
    debug!("Request options: {:?}", request.options);

    // Use reverse engine for MX to MT transformation
    let engine = state.reverse_engine.lock().await;

    let payload_value = Value::String(request.message.clone());
    let mut message = Message::new(&payload_value);
    message.metadata = serde_json::json!({
        "transformation_direction": "mx-to-mt"
    });

    let processing_time = start_time.elapsed().as_millis() as u64;

    match engine.process_message(&mut message).await {
        Ok(_) => {
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
                    transformed_message: None,
                    debug_info: if request.options.include_debug {
                        let message_json = serde_json::to_value(&message).unwrap();
                        Some(DebugInfo {
                            engine_state: "reverse".to_string(),
                            workflow_execution: vec!["Failed - Validation errors".to_string()],
                            intermediate_data: message_json,
                        })
                    } else {
                        None
                    },
                    errors: workflow_errors,
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
                                    transformed_message: Some(Value::String(
                                        result_str.to_string(),
                                    )),
                                    debug_info: if request.options.include_debug {
                                        let message_json = serde_json::to_value(&message).unwrap();
                                        Some(DebugInfo {
                                            engine_state: "reverse".to_string(),
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
                                        transformed_message: None,
                                        debug_info: if request.options.include_debug {
                                            let message_json =
                                                serde_json::to_value(&message).unwrap();
                                            Some(DebugInfo {
                                                engine_state: "reverse".to_string(),
                                                workflow_execution: vec![
                                                    "Failed - Malformed SWIFT MT output"
                                                        .to_string(),
                                                ],
                                                intermediate_data: message_json,
                                            })
                                        } else {
                                            None
                                        },
                                        errors: vec![format!(
                                            "Transformation produced malformed SWIFT MT message: {}",
                                            mt_error
                                        )],
                                        warnings: Vec::new(),
                                    }))
                                }
                            }
                        }
                        _ => {
                            error!("❌ MX to MT transformation produced empty or invalid result");
                            Ok(Json(TransformationResponse {
                                success: false,
                                transformed_message: None,
                                debug_info: if request.options.include_debug {
                                    let message_json = serde_json::to_value(message).unwrap();
                                    Some(DebugInfo {
                                        engine_state: "reverse".to_string(),
                                        workflow_execution: vec![
                                            "Failed - Empty or invalid result".to_string(),
                                        ],
                                        intermediate_data: message_json,
                                    })
                                } else {
                                    None
                                },
                                errors: vec![
                                    "Transformation completed but produced empty or invalid result"
                                        .to_string(),
                                ],
                                warnings: Vec::new(),
                            }))
                        }
                    }
                }
                _ => {
                    error!("❌ MX to MT transformation completed but no valid result found");
                    Ok(Json(TransformationResponse {
                        success: false,
                        transformed_message: None,
                        debug_info: if request.options.include_debug {
                            let message_json = serde_json::to_value(message).unwrap();
                            Some(DebugInfo {
                                engine_state: "reverse".to_string(),
                                workflow_execution: vec!["Failed - No result produced".to_string()],
                                intermediate_data: message_json,
                            })
                        } else {
                            None
                        },
                        errors: vec![
                            "Transformation completed but no valid result was produced".to_string(),
                        ],
                        warnings: Vec::new(),
                    }))
                }
            }
        }
        Err(e) => {
            error!("❌ MX to MT transformation failed: {}", e);

            Ok(Json(TransformationResponse {
                success: false,
                transformed_message: None,
                debug_info: if request.options.include_debug {
                    let message_json = serde_json::to_value(message).unwrap();
                    Some(DebugInfo {
                        engine_state: "reverse".to_string(),
                        workflow_execution: vec![format!("Failed - Engine error: {}", e)],
                        intermediate_data: message_json,
                    })
                } else {
                    None
                },
                errors: vec![e.to_string()],
                warnings: Vec::new(),
            }))
        }
    }
}

// Sample generation endpoint
#[instrument(skip(request), fields(message_type = request.message_type.as_str()))]
pub async fn generate_mt_sample(
    Json(request): Json<SampleGenerationRequest>,
) -> Result<Json<TransformationResponse>, StatusCode> {
    let start_time = Instant::now();

    info!(
        "🔄 Processing MT sample generation request for {}",
        request.message_type
    );
    debug!("Request options: {:?}", request.options);

    // Validate message type
    if !is_supported_message_type(&request.message_type) {
        error!("❌ Unsupported message type: {}", request.message_type);
        return Ok(Json(TransformationResponse {
            success: false,
            transformed_message: None,
            debug_info: None,
            errors: vec![format!(
                "Unsupported message type: {}",
                request.message_type
            )],
            warnings: Vec::new(),
        }));
    }

    // Generate MT message from JSON config
    match generate_mt_from_config(&request.config, &request.message_type, &request.options).await {
        Ok(mt_message) => {
            let processing_time = start_time.elapsed().as_millis() as u64;
            info!("✅ MT sample generation completed in {}ms", processing_time);

            Ok(Json(TransformationResponse {
                success: true,
                transformed_message: Some(Value::String(mt_message.clone())),
                debug_info: if request.options.include_debug {
                    Some(DebugInfo {
                        engine_state: "sample_generation".to_string(),
                        workflow_execution: vec!["Sample generated from JSON config".to_string()],
                        intermediate_data: request.config,
                    })
                } else {
                    None
                },
                errors: Vec::new(),
                warnings: Vec::new(),
            }))
        }
        Err(e) => {
            error!("❌ MT sample generation failed: {}", e);
            Ok(Json(TransformationResponse {
                success: false,
                transformed_message: None,
                debug_info: if request.options.include_debug {
                    Some(DebugInfo {
                        engine_state: "sample_generation".to_string(),
                        workflow_execution: vec![format!("Failed - Generation error: {}", e)],
                        intermediate_data: request.config,
                    })
                } else {
                    None
                },
                errors: vec![e.to_string()],
                warnings: Vec::new(),
            }))
        }
    }
}

// Health check endpoint
pub async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    let forward_status = if state.forward_engine.try_lock().is_ok() {
        "healthy"
    } else {
        "busy"
    };

    let reverse_status = if state.reverse_engine.try_lock().is_ok() {
        "healthy"
    } else {
        "busy"
    };

    Json(HealthResponse {
        status: "running".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        engines: EngineStatus {
            forward: forward_status.to_string(),
            reverse: reverse_status.to_string(),
        },
        capabilities: vec![
            "MT-to-MX transformation".to_string(),
            "MX-to-MT transformation".to_string(),
            "MT sample generation (24 message types)".to_string(),
            "Supported MT types: MT101, MT103, MT104, MT107, MT110, MT111, MT112, MT192, MT196, MT199, MT202, MT205, MT210, MT292, MT296, MT299, MT900, MT910, MT920, MT935, MT940, MT941, MT942, MT950".to_string(),
        ],
    })
}

#[instrument(skip(state))]
pub async fn reload_workflows(State(state): State<AppState>) -> Result<Json<ReloadResponse>, StatusCode> {
    let start_time = Instant::now();
    
    info!("🔄 Processing workflow reload request");

    match reload_engines(&state).await {
        Ok(()) => {
            let reload_time = start_time.elapsed().as_millis() as u64;
            info!("✅ Workflow reload completed successfully in {}ms", reload_time);

            Ok(Json(ReloadResponse {
                success: true,
                message: format!("Workflows reloaded successfully in {reload_time}ms"),
                timestamp: chrono::Utc::now().to_rfc3339(),
                error: None,
            }))
        }
        Err(e) => {
            let reload_time = start_time.elapsed().as_millis() as u64;
            error!("❌ Workflow reload failed after {}ms: {}", reload_time, e);

            Ok(Json(ReloadResponse {
                success: false,
                message: format!("Workflow reload failed after {reload_time}ms"),
                timestamp: chrono::Utc::now().to_rfc3339(),
                error: Some(e.to_string()),
            }))
        }
    }
}
