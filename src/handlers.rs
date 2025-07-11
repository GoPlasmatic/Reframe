use axum::{Json, extract::State, http::StatusCode};
use dataflow_rs::engine::message::Message;
use serde_json::Value;
use std::time::Instant;
use tracing::{debug, error, info, instrument};

use crate::sample_generator::{generate_mt_from_config, is_supported_message_type};
use crate::types::{
    AppState, DebugInfo, EngineStatus, HealthResponse, SampleGenerationRequest,
    TransformationRequest, TransformationResponse,
};

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

            Ok(Json(TransformationResponse {
                success: true,
                transformed_message: Some(
                    message.data.get("result").unwrap_or(&Value::Null).clone(),
                ),
                debug_info: if request.options.include_debug {
                    let message_json = serde_json::to_value(message).unwrap();
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
            }))
        }
        Err(e) => {
            error!("❌ MT to MX transformation failed: {}", e);

            Ok(Json(TransformationResponse {
                success: false,
                transformed_message: None,
                debug_info: None,
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

            Ok(Json(TransformationResponse {
                success: true,
                transformed_message: Some(
                    message.data.get("result").unwrap_or(&Value::Null).clone(),
                ),
                debug_info: if request.options.include_debug {
                    let message_json = serde_json::to_value(message).unwrap();
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
            }))
        }
        Err(e) => {
            error!("❌ MX to MT transformation failed: {}", e);

            Ok(Json(TransformationResponse {
                success: false,
                transformed_message: None,
                debug_info: None,
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
                debug_info: None,
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
