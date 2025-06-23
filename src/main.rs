use axum::{
    Router,
    extract::State,
    http::{StatusCode, header},
    response::Response,
    routing::{get, post},
};
use dataflow_rs::{Engine, Workflow};
use dataflow_rs::{RetryConfig, engine::message::Message};
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::services::ServeDir;
use tracing::{info, debug, warn, error, instrument};
use tracing_log::LogTracer;
use tracing_subscriber::{fmt, EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use tracing_appender::rolling;

mod parser;
use parser::ParserFunction;

mod publish;
use publish::PublishFunction;

// Application state
#[derive(Clone)]
struct AppState {
    engine: Arc<Mutex<Engine>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    let _guard = init_tracing();
    
    info!("🚀 Starting Reframe SWIFT message processing server");

    // Initialize the dataflow engine
    let mut engine = Engine::new().with_retry_config(RetryConfig {
        max_retries: 0,
        retry_delay_ms: 1,
        use_backoff: false,
    });

    // Register custom parse function
    engine.register_task_function("parse".to_string(), Box::new(ParserFunction));
    engine.register_task_function("publish".to_string(), Box::new(PublishFunction));
    info!("✅ Registered custom task functions: parse, publish");

    // Load workflows from directory
    setup_workflows(&mut engine).await?;

    // Create application state
    let state = AppState {
        engine: Arc::new(Mutex::new(engine)),
    };

    // Build the router with static file serving
    let app = Router::new()
        .route("/reframe", post(process_data))
        .route("/health", get(health_check))
        .nest_service("/", ServeDir::new("static"))
        .with_state(state);

    info!("🚀 Server starting on http://0.0.0.0:3000");
    info!("📱 Web UI available at: http://0.0.0.0:3000/");
    info!("🔄 API endpoint: http://0.0.0.0:3000/reframe");
    info!("💚 Health check: http://0.0.0.0:3000/health");

    // Start the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn init_tracing() -> impl Drop {
    // Set up file appender with daily rotation
    let file_appender = rolling::daily("logs", "reframe.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    // Configure filter from environment or default
    let filter = EnvFilter::try_from_env("RUST_LOG")
        .unwrap_or_else(|_| EnvFilter::new("info,reframe=debug,dataflow_rs=debug"));

    // Create subscriber with both console and file output
    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true)
        )
        .with(
            fmt::layer()
                .with_writer(non_blocking)
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true)
                .with_ansi(false) // No ANSI colors in log files
        )
        .with(filter)
        .init();

    // Initialize log tracer for compatibility with legacy log crates (after subscriber)
    let _ = LogTracer::init();

    guard
}

#[instrument(skip(engine))]
async fn setup_workflows(engine: &mut Engine) -> anyhow::Result<()> {
    let workflows_dir = Path::new("workflows");

    // Check if workflows directory exists
    if !workflows_dir.exists() {
        warn!("⚠️  Workflows directory not found at 'workflows/'. Creating directory...");
        fs::create_dir_all(workflows_dir)?;
        info!("📁 Workflows directory created. Please add workflow JSON files to this directory.");
        return Ok(());
    }

    // Check if index.json exists
    let index_path = workflows_dir.join("index.json");
    if !index_path.exists() {
        warn!("⚠️  index.json not found in workflows directory.");
        info!("💡 Please create an index.json file to define workflow loading order.");
        return Ok(());
    }

    // Load and parse index.json
    debug!(?index_path, "Loading workflow index");
    let index_content = fs::read_to_string(&index_path)
        .map_err(|e| anyhow::anyhow!("Failed to read index.json: {}", e))?;

    let index_data: Value = serde_json::from_str(&index_content)
        .map_err(|e| anyhow::anyhow!("Failed to parse index.json: {}", e))?;

    // Get workflows array from index
    let workflows_array = index_data
        .get("workflows")
        .and_then(|w| w.as_array())
        .ok_or_else(|| anyhow::anyhow!("index.json must contain a 'workflows' array"))?;

    let mut workflow_count = 0;
    debug!(workflow_count = workflows_array.len(), "Found workflows in index");

    // Load workflows in the order specified by index.json
    for workflow_entry in workflows_array {
        let workflow_path = workflow_entry
            .get("path")
            .and_then(|p| p.as_str())
            .ok_or_else(|| anyhow::anyhow!("Each workflow entry must have a 'path' field"))?;

        let full_path = workflows_dir.join(workflow_path);

        match load_workflow_from_file(&full_path) {
            Ok(workflow) => {
                engine.add_workflow(&workflow);
                workflow_count += 1;
                info!(
                    workflow_name = %workflow.name,
                    workflow_id = %workflow.id,
                    path = %workflow_path,
                    "✅ Loaded workflow"
                );
            }
            Err(e) => {
                error!(
                    path = %workflow_path,
                    error = %e,
                    "❌ Failed to load workflow"
                );
            }
        }
    }

    if workflow_count == 0 {
        warn!("⚠️  No workflows were successfully loaded.");
        info!("💡 Check that all workflow files referenced in index.json exist and are valid.");
    } else {
        info!(
            workflow_count = workflow_count,
            "🎯 Successfully loaded workflows from index.json"
        );
    }

    Ok(())
}

#[instrument]
fn load_workflow_from_file(path: &Path) -> anyhow::Result<Workflow> {
    debug!(?path, "Loading workflow file");
    
    // Read the file content
    let content = fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("Failed to read file {}: {}", path.display(), e))?;

    // Parse the JSON
    let workflow = Workflow::from_json(&content).map_err(|e| {
        anyhow::anyhow!(
            "Failed to parse workflow JSON from {}: {}",
            path.display(),
            e
        )
    })?;

    debug!(workflow_id = %workflow.id, workflow_name = %workflow.name, "Workflow loaded successfully");
    Ok(workflow)
}

#[instrument(skip(state, payload), fields(payload_length = payload.len()))]
async fn process_data(
    State(state): State<AppState>,
    payload: String,
) -> Result<Response<String>, StatusCode> {
    debug!("Processing SWIFT message data");
    
    let engine = state.engine.lock().await;

    // Create a message with the payload
    let mut message = Message::new(&Value::String(payload));

    // Process the message through workflows
    match engine.process_message(&mut message).await {
        Ok(_) => {
            debug!("Message processed successfully");
            
            // Check if we have multiple results (for 1-to-many transformations like MT102)
            if let Some(results_array) = message.data.get("result") {
                if let Some(results) = results_array.as_array() {
                    if !results.is_empty() {
                        let xml_results: Vec<String> = results
                            .iter()
                            .map(|result| result.as_str().unwrap_or("").to_string())
                            .collect();

                        info!(
                            result_count = xml_results.len(),
                            "Successfully processed message with multiple results"
                        );

                        let response_json = serde_json::json!({
                            "status": "success",
                            "results": xml_results,
                            "debug_message": message
                        });

                        return Response::builder()
                            .header(header::CONTENT_TYPE, "application/json")
                            .body(
                                serde_json::to_string(&response_json)
                                    .unwrap_or_else(|_| "{}".to_string()),
                            )
                            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
                    }
                }
            }

            warn!("Message processing completed but no results found");
            let response_json = serde_json::json!({
                "status": "error",
                "results": [],
                "errors": message.errors,
                "debug_message": message
            });

            Response::builder()
                .header(header::CONTENT_TYPE, "application/json")
                .body(serde_json::to_string(&response_json).unwrap_or_else(|_| "{}".to_string()))
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(e) => {
            error!(error = %e, "Message processing failed");
            
            let response_json = serde_json::json!({
                "status": "error",
                "results": [],
                "errors": message.errors,
                "debug_message": message
            });

            Response::builder()
                .header(header::CONTENT_TYPE, "application/json")
                .body(serde_json::to_string(&response_json).unwrap_or_else(|_| "{}".to_string()))
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Health check endpoint
#[instrument]
async fn health_check() -> Result<Response<String>, StatusCode> {
    debug!("Health check requested");
    
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .body(r#"{"status":"healthy","service":"reframe-api","version":"1.5.5"}"#.to_string())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(response)
}
