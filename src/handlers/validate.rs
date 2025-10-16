use axum::{Json, extract::State, http::StatusCode};
use dataflow_rs::engine::message::Message;
use serde_json::Value;
use std::sync::Arc;
use std::time::Instant;
use tracing::{error, info, instrument};

use super::helpers::{create_debug_info, extract_workflow_errors, resolve_package_id};
use super::types::{ValidationRequest, ValidationResponse};
use crate::types::{AppState, ReframeError};

/// Publish message to database for audit logging (if enabled and persist_validate is true)
fn publish_to_database(state: &AppState, message: &Message) {
    if state.database_config.persist_validate
        && state.database_client.is_some()
        && let Some(ref client) = state.database_client
    {
        client.clone().publish_message(message);
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
    let package_id = resolve_package_id(&state, request.package.as_ref());

    info!(
        "🔍 Processing unified validation request (package: {})",
        package_id
    );

    // Prepare message for validation workflow
    let payload_value = Value::String(request.message.clone());
    let mut message = Message::new(Arc::new(payload_value));

    // Set metadata for workflow detection
    // Note: Package workflows are responsible for detecting message format
    // and setting message_type in metadata
    if let Some(metadata_obj) = message.metadata_mut().as_object_mut() {
        // Add Reframe version
        metadata_obj.insert(
            "reframe_version".to_string(),
            env!("CARGO_PKG_VERSION").to_string().into(),
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

    // Process message using the validation engine
    // Note: Package workflows will detect format and perform validation
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

                // Publish validation to database
                publish_to_database(&state, &message);

                return Ok(Json(ValidationResponse {
                    success: false,
                    package: package_id.clone(),
                    message_type: None,
                    errors: workflow_errors
                        .into_iter()
                        .map(|e| ReframeError::transformation_error("VALIDATION_ERROR", e))
                        .collect(),
                    warnings: Vec::new(),
                    debug_info: create_debug_info(request.debug, &message),
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

                // Publish validation to database
                publish_to_database(&state, &message);

                Ok(Json(ValidationResponse {
                    success: valid,
                    package: package_id.clone(),
                    message_type,
                    errors,
                    warnings: Vec::new(),
                    debug_info: create_debug_info(request.debug, &message),
                    processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
                }))
            } else {
                error!("Validation workflow did not produce output");

                // Publish failed validation to database
                publish_to_database(&state, &message);

                Ok(Json(ValidationResponse {
                    success: false,
                    package: package_id.clone(),
                    message_type: None,
                    errors: vec![ReframeError::internal_error(
                        "Validation workflow did not produce output".to_string(),
                    )],
                    warnings: Vec::new(),
                    debug_info: create_debug_info(request.debug, &message),
                    processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
                }))
            }
        }
        Err(e) => {
            error!(
                error = %e,
                "Validation failed"
            );

            // Publish failed validation to database
            publish_to_database(&state, &message);

            Ok(Json(ValidationResponse {
                success: false,
                package: package_id.clone(),
                message_type: None,
                errors: vec![ReframeError::internal_error(e.to_string())],
                warnings: Vec::new(),
                debug_info: create_debug_info(request.debug, &message),
                processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
            }))
        }
    }
}
