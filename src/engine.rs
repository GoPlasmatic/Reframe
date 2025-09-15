use dataflow_rs::{AsyncFunctionHandler, Engine, Workflow};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use arc_swap::ArcSwap;
use tracing::{debug, info, warn};

use crate::parse_mt::ParseMT;
use crate::parse_mx::ParseMX;
use crate::publish_mt::PublishMT;
use crate::publish_mx::PublishMX;
use crate::types::AppState;

pub async fn initialize_forward_engine() -> Result<Arc<Engine>, Box<dyn std::error::Error>> {
    debug!("Setting up forward engine (MT to ISO 20022) with async Engine");

    // Load forward workflows
    let workflows = load_workflows("workflows/forward").await?;

    // Register MT-specific functions for forward transformation
    let mut custom_functions: HashMap<String, Box<dyn AsyncFunctionHandler + Send + Sync>> =
        HashMap::new();
    custom_functions.insert(
        "ParseMT".to_string(),
        Box::new(ParseMT) as Box<dyn AsyncFunctionHandler + Send + Sync>,
    );
    custom_functions.insert(
        "PublishMX".to_string(),
        Box::new(PublishMX) as Box<dyn AsyncFunctionHandler + Send + Sync>,
    );

    // Create async engine with workflows and custom functions
    let engine = Engine::new(workflows, Some(custom_functions));

    debug!("Forward engine ready with async Engine");
    Ok(Arc::new(engine))
}

pub async fn initialize_reverse_engine() -> Result<Arc<Engine>, Box<dyn std::error::Error>> {
    debug!("Setting up reverse engine (ISO 20022 to MT) with async Engine");

    // Load reverse workflows
    let workflows = load_workflows("workflows/reverse").await?;

    // Register MX-specific functions for reverse transformation
    let mut custom_functions: HashMap<String, Box<dyn AsyncFunctionHandler + Send + Sync>> =
        HashMap::new();
    custom_functions.insert(
        "ParseMX".to_string(),
        Box::new(ParseMX) as Box<dyn AsyncFunctionHandler + Send + Sync>,
    );
    custom_functions.insert(
        "PublishMT".to_string(),
        Box::new(PublishMT) as Box<dyn AsyncFunctionHandler + Send + Sync>,
    );

    // Create async engine with workflows and custom functions
    let engine = Engine::new(workflows, Some(custom_functions));

    debug!("Reverse engine ready with async Engine");
    Ok(Arc::new(engine))
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

/// Initialize async engines for high-performance multi-threaded processing
pub async fn initialize_engines() -> AppState {
    info!("Initializing async Engine for high-performance multi-threaded processing");

    info!("Configuration:");
    info!("  • CPU cores available: {}", num_cpus::get());
    info!("  • Engine type: Async Engine (Tokio runtime)");
    info!("  • Runtime: Tokio multi-threaded");

    // Create forward engine with async support
    let forward_engine = initialize_forward_engine()
        .await
        .expect("Failed to initialize forward engine");

    // Create reverse engine with async support
    let reverse_engine = initialize_reverse_engine()
        .await
        .expect("Failed to initialize reverse engine");

    info!("Async Engine instances initialized successfully");

    AppState {
        forward_engine: Arc::new(ArcSwap::from(forward_engine)),
        reverse_engine: Arc::new(ArcSwap::from(reverse_engine)),
    }
}

pub async fn reload_engines(app_state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    info!("Reloading workflow configurations for async Engine");

    // Create new forward engine with updated workflows
    let new_forward_engine = initialize_forward_engine().await?;

    // Create new reverse engine with updated workflows
    let new_reverse_engine = initialize_reverse_engine().await?;

    // Atomically swap the engines
    app_state.forward_engine.store(new_forward_engine);
    app_state.reverse_engine.store(new_reverse_engine);

    info!("✅ Workflow reload completed successfully");
    info!("All new requests will use the updated workflows");

    Ok(())
}
