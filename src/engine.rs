use dataflow_rs::{Engine, Workflow};
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn};

use crate::parse_mt::ParseMT;
use crate::parse_mx::ParseMX;
use crate::publish_mt::PublishMT;
use crate::publish_mx::PublishMX;
use crate::types::AppState;
use dataflow_rs::engine::functions::validation::ValidationFunction;

pub async fn initialize_engines() -> AppState {
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
    engine.register_task_function("validate".to_string(), Box::new(ValidationFunction::new()));

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
    engine.register_task_function("validate".to_string(), Box::new(ValidationFunction::new()));

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

pub async fn reload_engines(app_state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    info!("🔄 Reloading both Forward and Reverse Engines");

    let new_forward_engine = initialize_forward_engine().await?;
    let new_reverse_engine = initialize_reverse_engine().await?;

    {
        let mut forward_guard = app_state.forward_engine.lock().await;
        *forward_guard = new_forward_engine;
    }

    {
        let mut reverse_guard = app_state.reverse_engine.lock().await;
        *reverse_guard = new_reverse_engine;
    }

    info!("✅ Both engines reloaded successfully");
    Ok(())
}

