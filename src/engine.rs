use dataflow_rs::{Engine, Workflow};
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tracing::{info, warn, debug};

use crate::parse_mt::ParseMT;
use crate::parse_mx::ParseMX;
use crate::publish_mt::PublishMT;
use crate::publish_mx::PublishMX;
use crate::types::AppState;
use dataflow_rs::engine::functions::validation::ValidationFunction;

pub async fn initialize_engines() -> AppState {
    info!("Initializing forward and reverse transformation engines");

    // Get concurrency from environment or use default
    let concurrency = std::env::var("ENGINE_CONCURRENCY")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(16); // Default to 16 concurrent operations

    info!("Initializing engines with concurrency level: {}", concurrency);

    let forward_engine = initialize_forward_engine(concurrency)
        .await
        .expect("Failed to initialize forward engine");

    let reverse_engine = initialize_reverse_engine(concurrency)
        .await
        .expect("Failed to initialize reverse engine");

    info!("Both engines initialized successfully");

    AppState {
        forward_engine,
        reverse_engine,
    }
}

async fn initialize_forward_engine(concurrency: usize) -> Result<Arc<Engine>, Box<dyn std::error::Error>> {
    debug!("Setting up forward engine (MT to ISO 20022) with concurrency: {}", concurrency);

    let mut engine = Engine::with_concurrency(concurrency);

    // Register MT-specific functions for forward transformation
    engine.register_task_function("ParseMT".to_string(), Box::new(ParseMT));
    engine.register_task_function("PublishMX".to_string(), Box::new(PublishMX));
    engine.register_task_function("validate".to_string(), Box::new(ValidationFunction::new()));

    // Load forward workflows
    load_workflows_for_engine(&mut engine, "workflows/forward").await?;

    debug!("Forward engine ready");
    Ok(Arc::new(engine))
}

async fn initialize_reverse_engine(concurrency: usize) -> Result<Arc<Engine>, Box<dyn std::error::Error>> {
    debug!("Setting up reverse engine (ISO 20022 to MT) with concurrency: {}", concurrency);

    let mut engine = Engine::with_concurrency(concurrency);

    // Register MX-specific functions for reverse transformation
    engine.register_task_function("ParseMX".to_string(), Box::new(ParseMX));
    engine.register_task_function("PublishMT".to_string(), Box::new(PublishMT));
    engine.register_task_function("validate".to_string(), Box::new(ValidationFunction::new()));

    // Load reverse workflows
    load_workflows_for_engine(&mut engine, "workflows/reverse").await?;

    debug!("Reverse engine ready");
    Ok(Arc::new(engine))
}

async fn load_workflows_for_engine(
    engine: &mut Engine,
    workflow_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let workflows = load_workflows(workflow_dir).await?;
    
    for workflow in workflows {
        engine.add_workflow(&workflow);
    }
    
    Ok(())
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

    // Load forward workflows
    let forward_workflows = load_workflows("workflows/forward").await?;
    app_state.forward_engine.reload_workflows(forward_workflows)?;
    
    // Load reverse workflows
    let reverse_workflows = load_workflows("workflows/reverse").await?;
    app_state.reverse_engine.reload_workflows(reverse_workflows)?;

    info!("Engines reloaded successfully");
    Ok(())
}
