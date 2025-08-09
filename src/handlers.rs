use axum::{Json, extract::State, http::StatusCode};
use dataflow_rs::engine::message::Message;
use quick_xml::Reader;
use serde_json::Value;
use std::time::Instant;
use swift_mt_message::SwiftParser;
use tracing::{debug, error, info, instrument};

use crate::engine::reload_engines;
use crate::mx_sample_generator::{generate_mx_from_config, is_supported_mx_type};
use crate::sample_generator::{generate_mt_from_config, is_supported_message_type};
use crate::parse_mx::ParseMX;
use crate::types::{
    AppState, DebugInfo, EngineStatus, HealthResponse, MessageCategory, ReloadResponse,
    SampleGenerationRequest, TransformationRequest, TransformationResponse, ValidationError,
    ValidationRequest, ValidationResponse,
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

// Helper function to detect message category
fn detect_message_category(message_type: &str) -> MessageCategory {
    if message_type.starts_with("MT") {
        MessageCategory::MT
    } else if message_type.starts_with("pacs")
        || message_type.starts_with("camt")
        || message_type.starts_with("pain")
    {
        MessageCategory::MX
    } else {
        MessageCategory::Unknown
    }
}

// Unified sample generation endpoint
#[instrument(skip(request), fields(message_type = request.message_type.as_str()))]
pub async fn generate_sample(
    Json(request): Json<SampleGenerationRequest>,
) -> Result<Json<TransformationResponse>, StatusCode> {
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
                error!("❌ Unsupported MT message type: {}", request.message_type);
                return Ok(Json(TransformationResponse {
                    success: false,
                    transformed_message: None,
                    debug_info: None,
                    errors: vec![format!(
                        "Unsupported MT message type: {}",
                        request.message_type
                    )],
                    warnings: Vec::new(),
                }));
            }
            generate_mt_from_config(&request.config, &request.message_type, &request.options).await
        }
        MessageCategory::MX => {
            if !is_supported_mx_type(&request.message_type) {
                error!("❌ Unsupported MX message type: {}", request.message_type);
                return Ok(Json(TransformationResponse {
                    success: false,
                    transformed_message: None,
                    debug_info: None,
                    errors: vec![format!(
                        "Unsupported MX message type: {}",
                        request.message_type
                    )],
                    warnings: Vec::new(),
                }));
            }
            generate_mx_from_config(&request.config, &request.message_type, &request.options).await
        }
        MessageCategory::Unknown => {
            error!("❌ Unknown message type format: {}", request.message_type);
            return Ok(Json(TransformationResponse {
                success: false,
                transformed_message: None,
                debug_info: None,
                errors: vec![format!(
                    "Unknown message type format: {}. Expected MT* or pacs*/camt*/pain*",
                    request.message_type
                )],
                warnings: Vec::new(),
            }));
        }
    };

    match result {
        Ok(message) => {
            let processing_time = start_time.elapsed().as_millis() as u64;
            info!("✅ Sample generation completed in {}ms", processing_time);

            Ok(Json(TransformationResponse {
                success: true,
                transformed_message: Some(Value::String(message)),
                debug_info: if request.options.include_debug {
                    Some(DebugInfo {
                        engine_state: "sample_generation".to_string(),
                        workflow_execution: vec!["Sample generated from scenario".to_string()],
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
            error!("❌ Sample generation failed: {}", e);
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

// MT validation endpoint
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

    // Parse the MT message
    match SwiftParser::parse_auto(&normalized_content) {
        Ok(parsed_message) => {
            let message_type = parsed_message.message_type().to_string();
            info!(
                "✅ MT validation completed in {}ms - Message type: {}",
                start_time.elapsed().as_millis(),
                message_type
            );

            let parse_errors = Vec::new();
            let mut business_errors = Vec::new();
            let mut warnings = Vec::new();

            // Convert to canonical JSON if requested
            let canonical_json = if request.options.include_canonical_json {
                match &message_type[..] {
                    "103" => parsed_message
                        .clone()
                        .into_mt103()
                        .and_then(|msg| serde_json::to_value(msg).ok()),
                    "202" => parsed_message
                        .clone()
                        .into_mt202()
                        .and_then(|msg| serde_json::to_value(msg).ok()),
                    "205" => parsed_message
                        .clone()
                        .into_mt205()
                        .and_then(|msg| serde_json::to_value(msg).ok()),
                    "900" => parsed_message
                        .clone()
                        .into_mt900()
                        .and_then(|msg| serde_json::to_value(msg).ok()),
                    "910" => parsed_message
                        .clone()
                        .into_mt910()
                        .and_then(|msg| serde_json::to_value(msg).ok()),
                    "192" => parsed_message
                        .clone()
                        .into_mt192()
                        .and_then(|msg| serde_json::to_value(msg).ok()),
                    "292" => parsed_message
                        .clone()
                        .into_mt292()
                        .and_then(|msg| serde_json::to_value(msg).ok()),
                    "196" => parsed_message
                        .clone()
                        .into_mt196()
                        .and_then(|msg| serde_json::to_value(msg).ok()),
                    "296" => parsed_message
                        .clone()
                        .into_mt296()
                        .and_then(|msg| serde_json::to_value(msg).ok()),
                    _ => None,
                }
            } else {
                None
            };

            // Perform business validation if requested
            if request.options.include_business_validation {
                // Use the built-in validate() method for business validation
                let validation_result = parsed_message.validate();

                // Check if there are any validation errors
                if !validation_result.is_valid {
                    // Extract errors from the validation result
                    for error in validation_result.errors {
                        let (code, field, message) = match error {
                            swift_mt_message::ValidationError::FormatValidation {
                                field_tag,
                                message,
                            } => (format!("MT_{field_tag}_FORMAT"), Some(field_tag), message),
                            swift_mt_message::ValidationError::LengthValidation {
                                field_tag,
                                expected,
                                actual,
                            } => (
                                format!("MT_{field_tag}_LENGTH"),
                                Some(field_tag),
                                format!("Expected length {expected}, got {actual}"),
                            ),
                            swift_mt_message::ValidationError::PatternValidation {
                                field_tag,
                                message,
                            } => (format!("MT_{field_tag}_PATTERN"), Some(field_tag), message),
                            swift_mt_message::ValidationError::ValueValidation {
                                field_tag,
                                message,
                            } => (format!("MT_{field_tag}_VALUE"), Some(field_tag), message),
                            swift_mt_message::ValidationError::BusinessRuleValidation {
                                rule_name,
                                message,
                            } => (rule_name, None, message),
                        };

                        business_errors.push(ValidationError {
                            code,
                            message,
                            field,
                            location: None,
                        });
                    }
                }

                // Add any warnings from validation
                for warning in validation_result.warnings {
                    warnings.push(ValidationError {
                        code: "MT_VALIDATION_WARNING".to_string(),
                        message: warning,
                        field: None,
                        location: None,
                    });
                }

                // Extract additional validation information based on message type
                match &message_type[..] {
                    "103" => {
                        if let Some(mt103) = parsed_message.into_mt103() {
                            // Check for reject/return codes
                            if mt103.has_reject_codes() {
                                warnings.push(ValidationError {
                                    code: "MT103_HAS_REJECT".to_string(),
                                    message: "Message contains reject codes".to_string(),
                                    field: None,
                                    location: None,
                                });
                            }
                            if mt103.has_return_codes() {
                                warnings.push(ValidationError {
                                    code: "MT103_HAS_RETURN".to_string(),
                                    message: "Message contains return codes".to_string(),
                                    field: None,
                                    location: None,
                                });
                            }

                            // Validate basic header fields
                            if mt103.basic_header.service_id.is_empty() {
                                business_errors.push(ValidationError {
                                    code: "MT103_MISSING_SERVICE_ID".to_string(),
                                    message: "Service ID is missing in basic header".to_string(),
                                    field: Some("basic_header.service_id".to_string()),
                                    location: None,
                                });
                            }

                            // Validate sender BIC in basic header
                            if !is_valid_bic(&mt103.basic_header.sender_bic) {
                                business_errors.push(ValidationError {
                                    code: "MT103_INVALID_SENDER_BIC".to_string(),
                                    message: format!(
                                        "Invalid sender BIC in header: {}",
                                        mt103.basic_header.sender_bic
                                    ),
                                    field: Some("basic_header.sender_bic".to_string()),
                                    location: None,
                                });
                            }

                            // Check if this is an STP message
                            if mt103.is_stp_message() {
                                warnings.push(ValidationError {
                                    code: "MT103_IS_STP".to_string(),
                                    message:
                                        "Message is marked as STP (Straight Through Processing)"
                                            .to_string(),
                                    field: None,
                                    location: None,
                                });
                            }

                            // Additional business validations can be added here based on the
                            // specific fields exposed by the swift-mt-message library
                        }
                    }
                    "202" => {
                        if let Some(mt202) = parsed_message.into_mt202() {
                            // Similar business validations for MT202
                            if mt202.has_reject_codes() {
                                warnings.push(ValidationError {
                                    code: "MT202_HAS_REJECT".to_string(),
                                    message: "Message contains reject codes".to_string(),
                                    field: None,
                                    location: None,
                                });
                            }
                            if mt202.has_return_codes() {
                                warnings.push(ValidationError {
                                    code: "MT202_HAS_RETURN".to_string(),
                                    message: "Message contains return codes".to_string(),
                                    field: None,
                                    location: None,
                                });
                            }
                        }
                    }
                    _ => {
                        // Generic validations for other message types
                    }
                }
            }

            Ok(Json(ValidationResponse {
                valid: parse_errors.is_empty() && business_errors.is_empty(),
                message_type: Some(message_type),
                canonical_json,
                parse_errors,
                business_errors,
                warnings,
            }))
        }
        Err(parse_error) => {
            error!("❌ MT validation failed: {:?}", parse_error);

            // Extract detailed parse errors
            let mut parse_errors = vec![ValidationError {
                code: "MT_PARSE_ERROR".to_string(),
                message: format!("{parse_error:?}"),
                field: None,
                location: None,
            }];

            // Try to extract more specific error information
            let error_str = format!("{parse_error:?}");
            if error_str.contains("unexpected character") {
                parse_errors.push(ValidationError {
                    code: "MT_INVALID_CHARACTER".to_string(),
                    message: "Message contains invalid characters".to_string(),
                    field: None,
                    location: None,
                });
            } else if error_str.contains("missing field") {
                parse_errors.push(ValidationError {
                    code: "MT_MISSING_FIELD".to_string(),
                    message: "Required field is missing".to_string(),
                    field: None,
                    location: None,
                });
            }

            Ok(Json(ValidationResponse {
                valid: false,
                message_type: None,
                canonical_json: None,
                parse_errors,
                business_errors: Vec::new(),
                warnings: Vec::new(),
            }))
        }
    }
}

// MX validation endpoint
#[instrument(skip(request), fields(message_length = request.message.len()))]
pub async fn validate_mx(
    Json(request): Json<ValidationRequest>,
) -> Result<Json<ValidationResponse>, StatusCode> {
    let start_time = Instant::now();

    info!("🔍 Processing MX validation request");
    debug!("Request options: {:?}", request.options);

    // First validate XML well-formedness
    if let Err(xml_error) = validate_xml_well_formed(&request.message) {
        error!("❌ MX validation failed - XML malformed: {}", xml_error);
        return Ok(Json(ValidationResponse {
            valid: false,
            message_type: None,
            canonical_json: None,
            parse_errors: vec![ValidationError {
                code: "MX_XML_MALFORMED".to_string(),
                message: xml_error,
                field: None,
                location: None,
            }],
            business_errors: Vec::new(),
            warnings: Vec::new(),
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

            let mut parse_errors = Vec::new();
            let mut business_errors = Vec::new();
            let mut warnings = Vec::new();

            // Extract canonical JSON if requested
            let canonical_json = if request.options.include_canonical_json {
                // Try to parse the header and document into JSON
                if let (Some(app_hdr), Some(doc_content)) = (app_hdr_content, document_content) {
                    match (ParseMX::parse_header(&message_type, &app_hdr), ParseMX::parse_document(&message_type, &doc_content)) {
                        (Ok(header), Ok(document)) => {
                            Some(serde_json::json!({
                                "header": header,
                                "document": document,
                                "message_type": message_type
                            }))
                        },
                        _ => None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            // Perform business validation if requested
            if request.options.include_business_validation {

                // Basic business validations based on parsed JSON
                if let Some(ref json) = canonical_json {
                    // Check for required fields based on message type
                    match &message_type[..] {
                        "pacs.008.001.08" => {
                            // Check for UETR in pacs.008
                            if let Some(doc) = json.get("document") {
                                if let Some(cdt_trf) = doc.get("FIToFICstmrCdtTrf").or(doc.get("FIToFICustomerCreditTransferV08")) {
                                    if let Some(cdt_trf_tx_inf) = cdt_trf.get("CdtTrfTxInf") {
                                        if let Some(tx_array) = cdt_trf_tx_inf.as_array() {
                                            for tx in tx_array {
                                                if let Some(pmt_id) = tx.get("PmtId") {
                                                    if pmt_id.get("UETR").is_none() {
                                                        warnings.push(ValidationError {
                                                            code: "PACS008_MISSING_UETR".to_string(),
                                                            message: "UETR is recommended for payment tracking".to_string(),
                                                            field: Some("CdtTrfTxInf.PmtId.UETR".to_string()),
                                                            location: None,
                                                        });
                                                    }
                                                }
                                                
                                                // Check BIC codes
                                                if let Some(dbtr_agt) = tx.get("DbtrAgt") {
                                                    if let Some(fin_instn_id) = dbtr_agt.get("FinInstnId") {
                                                        if let Some(bicfi) = fin_instn_id.get("BICFI") {
                                                            if let Some(bic_str) = bicfi.as_str() {
                                                                if !is_valid_bic(bic_str) {
                                                                    business_errors.push(ValidationError {
                                                                        code: "PACS008_INVALID_DBTR_AGT_BIC".to_string(),
                                                                        message: format!("Invalid debtor agent BIC: {}", bic_str),
                                                                        field: Some("CdtTrfTxInf.DbtrAgt.FinInstnId.BICFI".to_string()),
                                                                        location: None,
                                                                    });
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                                
                                                if let Some(cdtr_agt) = tx.get("CdtrAgt") {
                                                    if let Some(fin_instn_id) = cdtr_agt.get("FinInstnId") {
                                                        if let Some(bicfi) = fin_instn_id.get("BICFI") {
                                                            if let Some(bic_str) = bicfi.as_str() {
                                                                if !is_valid_bic(bic_str) {
                                                                    business_errors.push(ValidationError {
                                                                        code: "PACS008_INVALID_CDTR_AGT_BIC".to_string(),
                                                                        message: format!("Invalid creditor agent BIC: {}", bic_str),
                                                                        field: Some("CdtTrfTxInf.CdtrAgt.FinInstnId.BICFI".to_string()),
                                                                        location: None,
                                                                    });
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        _ => {
                            // Other message types - basic validation only
                            info!("Business validation for {} uses basic checks only", message_type);
                        }
                    }
                } else {
                    warnings.push(ValidationError {
                        code: "MX_VALIDATION_LIMITED".to_string(),
                        message: "Business validation requires canonical JSON extraction".to_string(),
                        field: None,
                        location: None,
                    });
                }
            }

            Ok(Json(ValidationResponse {
                valid: parse_errors.is_empty() && business_errors.is_empty(),
                message_type: Some(message_type),
                canonical_json,
                parse_errors,
                business_errors,
                warnings,
            }))
        }
        Err(parse_error) => {
            error!("❌ MX validation failed: {:?}", parse_error);

            // Extract detailed parse errors
            let error_message = format!("{:?}", parse_error);
            let mut parse_errors = vec![ValidationError {
                code: "MX_PARSE_ERROR".to_string(),
                message: error_message.clone(),
                field: None,
                location: None,
            }];

            // Try to extract more specific error information
            if error_message.contains("Unknown message type") {
                parse_errors.push(ValidationError {
                    code: "MX_UNKNOWN_MESSAGE_TYPE".to_string(),
                    message: "Message type is not recognized or supported".to_string(),
                    field: Some("MsgDefIdr".to_string()),
                    location: None,
                });
            } else if error_message.contains("missing field") || error_message.contains("Missing") {
                parse_errors.push(ValidationError {
                    code: "MX_MISSING_REQUIRED_FIELD".to_string(),
                    message: "Required field is missing from the message".to_string(),
                    field: None,
                    location: None,
                });
            } else if error_message.contains("Invalid") || error_message.contains("invalid") {
                parse_errors.push(ValidationError {
                    code: "MX_INVALID_FORMAT".to_string(),
                    message: "Message format is invalid".to_string(),
                    field: None,
                    location: None,
                });
            }

            Ok(Json(ValidationResponse {
                valid: false,
                message_type: None,
                canonical_json: None,
                parse_errors,
                business_errors: Vec::new(),
                warnings: Vec::new(),
            }))
        }
    }
}

// Helper function to validate BIC codes
fn is_valid_bic(bic: &str) -> bool {
    // Basic BIC validation: 8 or 11 characters, alphanumeric
    let len = bic.len();
    (len == 8 || len == 11) && bic.chars().all(|c| c.is_alphanumeric())
}
