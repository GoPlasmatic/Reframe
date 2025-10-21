use axum::{Json, extract::State, http::StatusCode};
use dataflow_rs::engine::message::Message;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, error, info, instrument};

use super::helpers::{
    create_debug_info, extract_workflow_errors, load_scenario_from_package,
    publish_to_database_with_custom_fields, resolve_package_id,
};
use super::types::{SampleGenerationRequest, SampleGenerationResponse};
use crate::types::{AppState, ReframeError};

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
    let package_id = resolve_package_id(&state, request.package.as_ref());

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
                    warnings: Vec::new(),
                    debug_info: None,
                    processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
                }));
            }
        };

        // Load scenario using helper function
        match load_scenario_from_package(package, &request.message_type, scenario_name) {
            Ok(data) => data,
            Err(err) => {
                error!("{}", err.message);
                return Ok(Json(SampleGenerationResponse {
                    success: false,
                    package: package_id.clone(),
                    message_type: request.message_type.clone(),
                    result: None,
                    scenario: Some(scenario_name.to_string()),
                    errors: vec![err],
                    warnings: Vec::new(),
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
        // Add Reframe version
        metadata_obj.insert(
            "reframe_version".to_string(),
            env!("CARGO_PKG_VERSION").to_string().into(),
        );

        // Add package ID for custom fields
        metadata_obj.insert("package_id".to_string(), package_id.clone().into());

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
                    // Clone result string before publishing
                    let result_string = result_str.to_string();

                    info!("✅ Sample generation completed in {}ms", processing_time);

                    // Publish successful generation to database
                    publish_to_database_with_custom_fields(
                        &state,
                        &mut message,
                        state.database_config.persist_generate,
                    );

                    Ok(Json(SampleGenerationResponse {
                        success: true,
                        package: package_id.clone(),
                        message_type: request.message_type.clone(),
                        result: Some(result_string),
                        scenario: Some(request.scenario.clone()),
                        errors: Vec::new(),
                        warnings: Vec::new(),
                        debug_info: create_debug_info(request.debug, &message),
                        processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
                    }))
                } else {
                    error!("Generation workflow did not produce a string result");

                    // Publish failed generation to database
                    publish_to_database_with_custom_fields(
                        &state,
                        &mut message,
                        state.database_config.persist_generate,
                    );

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
                        warnings: Vec::new(),
                        debug_info: None,
                        processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
                    }))
                }
            } else {
                // Check for errors in the message
                let workflow_errors = extract_workflow_errors(&message);
                if !workflow_errors.is_empty() {
                    error!("Generation failed with errors: {:?}", workflow_errors);

                    // Publish failed generation to database
                    publish_to_database_with_custom_fields(
                        &state,
                        &mut message,
                        state.database_config.persist_generate,
                    );

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
                        warnings: Vec::new(),
                        debug_info: None,
                        processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
                    }))
                } else {
                    error!("Generation workflow did not produce a result");

                    // Publish failed generation to database
                    publish_to_database_with_custom_fields(
                        &state,
                        &mut message,
                        state.database_config.persist_generate,
                    );

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
                        warnings: Vec::new(),
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

            // Publish failed generation to database
            publish_to_database_with_custom_fields(
                &state,
                &mut message,
                state.database_config.persist_generate,
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
                warnings: Vec::new(),
                debug_info: create_debug_info(request.debug, &message),
                processing_time_ms: Some(start_time.elapsed().as_millis() as u64),
            }))
        }
    }
}
