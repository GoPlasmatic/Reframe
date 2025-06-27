use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use dataflow_rs::engine::message::Message;
use dataflow_rs::{Engine, Workflow};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use tower_http::services::ServeDir;
use tracing::{debug, error, info, instrument, warn};
use tracing_subscriber::EnvFilter;

// Separate parser modules for different directions
mod parse_mt;
use parse_mt::ParseMT;

mod parse_mx;
use parse_mx::ParseMX;

// Separate publish modules for different directions
mod publish_mx;
use publish_mx::PublishMX;

mod publish_mt;
use publish_mt::PublishMT;

// Request/Response structures for new API
#[derive(Debug, Deserialize)]
struct TransformationRequest {
    message: String,
    #[serde(default)]
    options: TransformationOptions,
}

#[derive(Debug, Deserialize, Serialize, Default)]
struct TransformationOptions {
    #[serde(default = "default_true")]
    validation: bool,
    #[serde(default)]
    include_debug: bool,
    #[serde(default)]
    format_output: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Serialize)]
struct TransformationResponse {
    success: bool,
    transformed_message: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    debug_info: Option<DebugInfo>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    errors: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
struct DebugInfo {
    engine_state: String,
    workflow_execution: Vec<String>,
    intermediate_data: Value,
}

// Application State with dual engines
#[derive(Clone)]
struct AppState {
    forward_engine: Arc<Mutex<Engine>>,
    reverse_engine: Arc<Mutex<Engine>>,
}

// Health check response
#[derive(Serialize)]
struct HealthResponse {
    status: String,
    timestamp: String,
    engines: EngineStatus,
}

#[derive(Serialize)]
struct EngineStatus {
    forward: String,
    reverse: String,
}

#[tokio::main]
async fn main() {
    // Initialize logging
    initialize_logging();

    info!("🚀 Starting Reframe Bidirectional Transformation Service");

    // Initialize dual engines
    let app_state = initialize_engines().await;

    // Build router with new endpoints
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/transform/mt-to-mx", post(transform_mt_to_mx))
        .route("/transform/mx-to-mt", post(transform_mx_to_mt))
        .nest_service("/", ServeDir::new("static"))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("🌐 Server running on http://0.0.0.0:3000");
    info!("📡 Forward endpoint: POST /transform/mt-to-mx");
    info!("📡 Reverse endpoint: POST /transform/mx-to-mt");
    info!("🏥 Health check: GET /health");

    axum::serve(listener, app).await.unwrap();
}

fn initialize_logging() {
    // Simple tracing initialization to avoid conflicts
    if std::env::var("RUST_LOG").is_err() {
        unsafe {
            std::env::set_var("RUST_LOG", "reframe=debug,info");
        }
    }

    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_env_filter(EnvFilter::from_default_env())
        .try_init()
        .ok(); // Ignore errors if already initialized
}

async fn initialize_engines() -> AppState {
    info!("🔧 Initializing Forward and Reverse Engines");

    let forward_engine = initialize_forward_engine()
        .await
        .expect("Failed to initialize forward engine");

    let reverse_engine = initialize_reverse_engine()
        .await
        .expect("Failed to initialize reverse engine");

    info!("✅ Both engines initialized successfully");

    AppState {
        forward_engine: Arc::new(Mutex::new(forward_engine)),
        reverse_engine: Arc::new(Mutex::new(reverse_engine)),
    }
}

async fn initialize_forward_engine() -> Result<Engine, Box<dyn std::error::Error>> {
    info!("🔄 Setting up Forward Engine (MT → MX)");

    let mut engine = Engine::new();

    // Register MT-specific functions for forward transformation
    engine.register_task_function("ParseMT".to_string(), Box::new(ParseMT));
    engine.register_task_function("PublishMX".to_string(), Box::new(PublishMX));

    // Load forward workflows
    load_workflows_for_engine(&mut engine, "workflows/forward").await?;

    info!("✅ Forward Engine (MT → MX) ready");
    Ok(engine)
}

async fn initialize_reverse_engine() -> Result<Engine, Box<dyn std::error::Error>> {
    info!("🔄 Setting up Reverse Engine (MX → MT)");

    let mut engine = Engine::new();

    // Register MX-specific functions for reverse transformation
    engine.register_task_function("ParseMX".to_string(), Box::new(ParseMX));
    engine.register_task_function("PublishMT".to_string(), Box::new(PublishMT));

    // Load reverse workflows
    load_workflows_for_engine(&mut engine, "workflows/reverse").await?;

    info!("✅ Reverse Engine (MX → MT) ready");
    Ok(engine)
}

async fn load_workflows_for_engine(
    engine: &mut Engine,
    workflow_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("📁 Loading workflows from {}", workflow_dir);

    let index_path = format!("{workflow_dir}/index.json");
    if !Path::new(&index_path).exists() {
        warn!(
            "No index.json found in {}, skipping workflow loading",
            workflow_dir
        );
        return Ok(());
    }

    let index_content = fs::read_to_string(&index_path)?;
    let index: Value = serde_json::from_str(&index_content)?;

    if let Some(workflows) = index.get("workflows").and_then(|w| w.as_array()) {
        for workflow_entry in workflows {
            if let Some(path) = workflow_entry.get("path").and_then(|p| p.as_str()) {
                let full_path = format!("{workflow_dir}/{path}");
                if Path::new(&full_path).exists() {
                    let workflow_content = fs::read_to_string(&full_path)?;
                    let workflow: Workflow = serde_json::from_str(&workflow_content)?;

                    engine.add_workflow(&workflow);

                    info!("📄 Loaded workflow: {}", path);
                } else {
                    warn!("Workflow file not found: {}", full_path);
                }
            }
        }
    }

    Ok(())
}

// New endpoint handlers
#[instrument(skip(state, request), fields(message_length = request.message.len()))]
async fn transform_mt_to_mx(
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
async fn transform_mx_to_mt(
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

// Health check endpoint
async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
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
    })
}
