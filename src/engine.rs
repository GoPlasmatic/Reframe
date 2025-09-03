use dataflow_rs::{Engine, Workflow};
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

use crate::parse_mt::ParseMT;
use crate::parse_mx::ParseMX;
use crate::publish_mt::PublishMT;
use crate::publish_mx::PublishMX;
use crate::types::{AppState, EngineWrapper};

pub async fn initialize_engines() -> AppState {
    info!("Initializing forward and reverse transformation engines");

    let forward_engine = initialize_forward_engine()
        .await
        .expect("Failed to initialize forward engine");

    let reverse_engine = initialize_reverse_engine()
        .await
        .expect("Failed to initialize reverse engine");

    info!("Both engines initialized successfully");

    AppState {
        forward_engine: Arc::new(Mutex::new(forward_engine)),
        reverse_engine: Arc::new(Mutex::new(reverse_engine)),
    }
}

async fn initialize_forward_engine() -> Result<Arc<EngineWrapper>, Box<dyn std::error::Error>> {
    debug!("Setting up forward engine (MT to ISO 20022)");

    // Load forward workflows
    let workflows = load_workflows("workflows/forward").await?;

    // Register MT-specific functions for forward transformation
    let mut custom_functions = std::collections::HashMap::new();
    custom_functions.insert(
        "ParseMT".to_string(),
        Box::new(ParseMT) as Box<dyn dataflow_rs::FunctionHandler + Send + Sync>,
    );
    custom_functions.insert(
        "PublishMX".to_string(),
        Box::new(PublishMX) as Box<dyn dataflow_rs::FunctionHandler + Send + Sync>,
    );

    // Create engine with workflows and custom functions
    // Disable compiled logic due to issue with context variables in filter/reduce operations
    let engine = Engine::new(workflows, Some(custom_functions), Some(false), None, None);

    debug!("Forward engine ready");
    Ok(Arc::new(EngineWrapper::new(engine)))
}

async fn initialize_reverse_engine() -> Result<Arc<EngineWrapper>, Box<dyn std::error::Error>> {
    debug!("Setting up reverse engine (ISO 20022 to MT)");

    // Load reverse workflows
    let workflows = load_workflows("workflows/reverse").await?;

    // Register MX-specific functions for reverse transformation
    let mut custom_functions = std::collections::HashMap::new();
    custom_functions.insert(
        "ParseMX".to_string(),
        Box::new(ParseMX) as Box<dyn dataflow_rs::FunctionHandler + Send + Sync>,
    );
    custom_functions.insert(
        "PublishMT".to_string(),
        Box::new(PublishMT) as Box<dyn dataflow_rs::FunctionHandler + Send + Sync>,
    );

    // Create engine with workflows and custom functions
    // Disable compiled logic due to issue with context variables in filter/reduce operations
    let engine = Engine::new(workflows, Some(custom_functions), Some(false), None, None);

    debug!("Reverse engine ready");
    Ok(Arc::new(EngineWrapper::new(engine)))
}

async fn load_workflows(workflow_dir: &str) -> Result<Vec<Workflow>, Box<dyn std::error::Error>> {
    debug!(directory = %workflow_dir, "Loading workflow configurations");

    let mut workflows = Vec::new();
    let index_path = format!("{workflow_dir}/index.json");

    if !Path::new(&index_path).exists() {
        warn!(
            "No index.json found in {}, skipping workflow loading",
            workflow_dir
        );
        return Ok(workflows);
    }

    let index_content = fs::read_to_string(&index_path)?;
    let index: Value = serde_json::from_str(&index_content)?;

    if let Some(workflow_entries) = index.get("workflows").and_then(|w| w.as_array()) {
        for workflow_entry in workflow_entries {
            if let Some(path) = workflow_entry.get("path").and_then(|p| p.as_str()) {
                let full_path = format!("{workflow_dir}/{path}");
                if Path::new(&full_path).exists() {
                    let workflow_content = fs::read_to_string(&full_path)?;
                    let workflow: Workflow = serde_json::from_str(&workflow_content)?;
                    workflows.push(workflow);
                    debug!(workflow = %path, "Loaded workflow file");
                } else {
                    warn!(file = %full_path, "Workflow file not found");
                }
            }
        }
    }

    Ok(workflows)
}

pub async fn reload_engines(app_state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    info!("Reloading workflow configurations for both engines");

    // Since the new engine doesn't support dynamic workflow reloading,
    // we need to create new engine instances

    // Create new forward engine
    let new_forward_engine = initialize_forward_engine().await?;
    *app_state.forward_engine.lock().await = new_forward_engine;

    // Create new reverse engine
    let new_reverse_engine = initialize_reverse_engine().await?;
    *app_state.reverse_engine.lock().await = new_reverse_engine;

    info!("Engines reloaded successfully");
    Ok(())
}
