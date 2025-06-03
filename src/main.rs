use axum::{
    extract::State,
    http::{header, StatusCode},
    response::Response,
    routing::{get, post},
    Router,
};
use dataflow_rs::engine::message::Message;
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
    let mut engine = Engine::new();

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
            for entry in entries {
                let entry = entry?;
                let path = entry.path();

                // Only process .json files
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
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
            // Check if we have a result and if it looks like XML
            if let Some(result_value) = message.data.get("result") {
                let result_string = result_value.as_str().unwrap_or("");
                Response::builder()
                    .header(header::CONTENT_TYPE, "application/xml")
                    .body(result_string.to_string())
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
            } else {
                // Return the full message data as JSON if no specific result
                Response::builder()
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(serde_json::to_string(&message.data).unwrap_or_else(|_| "{}".to_string()))
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
        Err(e) => {
            let error_response =
                serde_json::json!({"error": format!("Error processing data: {:?}", e)});
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header(header::CONTENT_TYPE, "application/json")
                .body(error_response.to_string())
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
