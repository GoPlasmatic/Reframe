use axum::{Json, extract::State, http::StatusCode};
use dataflow_rs::engine::message::Message;
use serde_json::Value;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, error, info, instrument};

use super::helpers::{create_debug_info, extract_workflow_errors, resolve_package_id};
use super::types::{TransformationRequest, TransformationResponse};
use crate::types::{AppState, ReframeError};

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
    let package_id = resolve_package_id(&state, request.package.as_ref());

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

    // Set metadata for workflow detection
    // Note: Package workflows are responsible for detecting message format
    // and setting transformation_direction in metadata
    if let Some(metadata_obj) = message.metadata_mut().as_object_mut() {
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

    // Process message using the unified transform engine
    // Note: Package workflows will detect format and route appropriately
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
                    debug_info: create_debug_info(request.debug, &message),
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
                                debug_info: create_debug_info(request.debug, &message),
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
                                debug_info: create_debug_info(request.debug, &message),
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
                                debug_info: create_debug_info(request.debug, &message),
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
                        debug_info: create_debug_info(request.debug, &message),
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
                debug_info: create_debug_info(request.debug, &message),
                errors: vec![ReframeError::internal_error(e.to_string())],
                warnings: Vec::new(),
                processing_time_ms: Some(processing_time),
            }))
        }
    }
}
