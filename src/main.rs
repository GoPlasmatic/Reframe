use axum::{
    extract::State,
    http::{header, StatusCode},
    response::Response,
    routing::{get, post},
    Router,
};
use dataflow_rs::{engine::message::Message, RetryConfig};
use dataflow_rs::{Engine, Workflow};
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::services::ServeDir;

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
    // Initialize the dataflow engine
    let mut engine = Engine::new().with_retry_config(RetryConfig {
        max_retries: 0,
        retry_delay_ms: 1,
        use_backoff: false,
    });

    // Register custom parse function
    engine.register_task_function("parse".to_string(), Box::new(ParserFunction));
    engine.register_task_function("publish".to_string(), Box::new(PublishFunction));

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

    println!("🚀 Server starting on http://0.0.0.0:3000");
    println!("📱 Web UI available at: http://0.0.0.0:3000/");
    println!("🔄 API endpoint: http://0.0.0.0:3000/reframe");
    println!("💚 Health check: http://0.0.0.0:3000/health");

    // Start the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn setup_workflows(engine: &mut Engine) -> anyhow::Result<()> {
    let workflows_dir = Path::new("workflows");

    // Check if workflows directory exists
    if !workflows_dir.exists() {
        println!("⚠️  Workflows directory not found at 'workflows/'. Creating directory...");
        fs::create_dir_all(workflows_dir)?;
        println!(
            "📁 Workflows directory created. Please add workflow JSON files to this directory."
        );
        return Ok(());
    }

    // Read all JSON files from the workflows directory
    let mut workflow_count = 0;

    match fs::read_dir(workflows_dir) {
        Ok(entries) => {
            // Collect all JSON file paths and sort them by filename
            let mut json_files: Vec<_> = entries
                .filter_map(|entry| {
                    let entry = entry.ok()?;
                    let path = entry.path();

                    // Only include .json files
                    if path.extension().and_then(|s| s.to_str()) == Some("json") {
                        Some(path)
                    } else {
                        None
                    }
                })
                .collect();

            // Sort by filename
            json_files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

            // Process files in sorted order
            for path in json_files {
                match load_workflow_from_file(&path) {
                    Ok(workflow) => {
                        engine.add_workflow(&workflow);
                        workflow_count += 1;
                        println!(
                            "✅ Loaded workflow: {} from {}",
                            workflow.name,
                            path.display()
                        );
                    }
                    Err(e) => {
                        println!("❌ Failed to load workflow from {}: {}", path.display(), e);
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to read workflows directory: {}", e);
            return Err(anyhow::anyhow!("Could not read workflows directory: {}", e));
        }
    }

    if workflow_count == 0 {
        println!("⚠️  No workflow files found in 'workflows/' directory.");
        println!("💡 Add JSON workflow files to the 'workflows/' directory to enable message processing.");
    } else {
        println!("🎯 Successfully loaded {} workflow(s)", workflow_count);
    }

    Ok(())
}

fn load_workflow_from_file(path: &Path) -> anyhow::Result<Workflow> {
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

    Ok(workflow)
}

async fn process_data(
    State(state): State<AppState>,
    payload: String,
) -> Result<Response<String>, StatusCode> {
    let engine = state.engine.lock().await;

    // Create a message with the payload
    let mut message = Message::new(&Value::String(payload));

    // Process the message through workflows
    match engine.process_message(&mut message).await {
        Ok(_) => {
            // Check if we have multiple results (for 1-to-many transformations like MT102)
            if let Some(results_array) = message.data.get("result") {
                if let Some(results) = results_array.as_array() {
                    if !results.is_empty() {
                        let xml_results: Vec<String> = results
                            .iter()
                            .map(|result| result.as_str().unwrap_or("").to_string())
                            .collect();

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
        Err(_) => {
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
async fn health_check() -> Result<Response<String>, StatusCode> {
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .body(r#"{"status":"healthy","service":"reframe-api","version":"0.1.0"}"#.to_string())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(response)
}
