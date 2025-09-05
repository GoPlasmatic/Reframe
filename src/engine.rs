use dataflow_rs::{ThreadedEngine, Workflow};
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, info, warn};

use crate::parse_mt::ParseMT;
use crate::parse_mx::ParseMX;
use crate::publish_mt::PublishMT;
use crate::publish_mx::PublishMX;
use crate::types::AppState;

pub async fn initialize_forward_engine(
    thread_count: usize,
) -> Result<Arc<ThreadedEngine>, Box<dyn std::error::Error>> {
    debug!(
        "Setting up forward engine (MT to ISO 20022) with {} threads",
        thread_count
    );

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

    // Create threaded engine with workflows, custom functions, and thread count
    let engine = ThreadedEngine::new(workflows, Some(custom_functions), None, thread_count);

    debug!("Forward engine ready with {} worker threads", thread_count);
    Ok(Arc::new(engine))
}

pub async fn initialize_reverse_engine(
    thread_count: usize,
) -> Result<Arc<ThreadedEngine>, Box<dyn std::error::Error>> {
    debug!(
        "Setting up reverse engine (ISO 20022 to MT) with {} threads",
        thread_count
    );

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

    // Create threaded engine with workflows, custom functions, and thread count
    let engine = ThreadedEngine::new(workflows, Some(custom_functions), None, thread_count);

    debug!("Reverse engine ready with {} worker threads", thread_count);
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

/// Initialize threaded engines for vertical scaling
pub async fn initialize_engines() -> AppState {
    info!("Initializing threaded engines for vertical scaling");

    // Get thread count from environment or use CPU count
    let thread_count = std::env::var("REFRAME_THREAD_COUNT")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or_else(num_cpus::get);

    // Get max concurrent tasks from environment or use default based on threads
    let max_concurrent_tasks = std::env::var("REFRAME_MAX_CONCURRENT_TASKS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(thread_count * 4); // Default: 4x thread count

    info!("Configuration:");
    info!("  • Worker threads per engine: {}", thread_count);
    info!("  • Max concurrent tasks: {}", max_concurrent_tasks);
    info!("  • CPU cores available: {}", num_cpus::get());

    // Create forward engine with thread pool
    let forward_engine = initialize_forward_engine(thread_count)
        .await
        .expect("Failed to initialize forward engine");

    // Create reverse engine with thread pool
    let reverse_engine = initialize_reverse_engine(thread_count)
        .await
        .expect("Failed to initialize reverse engine");

    // Create semaphore for concurrent task limiting
    let max_concurrent_tasks = Arc::new(tokio::sync::Semaphore::new(max_concurrent_tasks));

    info!("Threaded engines initialized successfully");

    AppState {
        forward_engine,
        reverse_engine,
        max_concurrent_tasks,
    }
}

pub async fn reload_engines(_app_state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    info!("Reloading workflow configurations for threaded engines");

    // Get thread count for new engines
    let _thread_count = std::env::var("REFRAME_THREAD_COUNT")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or_else(num_cpus::get);

    // Note: Since ThreadedEngine doesn't have a reload method, we need to create new engines
    // In a production system, you might want to implement a more graceful reload mechanism
    // For now, this is a placeholder that would need to be coordinated with the handlers

    info!("Engine reload would require recreating engines with updated workflows");
    info!("This operation is not currently supported without restarting the service");

    Err("Hot reload not supported with ThreadedEngine. Please restart the service.".into())
}
