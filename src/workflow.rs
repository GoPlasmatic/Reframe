use dataflow_rs::{Engine, Workflow};
use std::fs;
use std::path::Path;
use tracing::{debug, info};

pub fn load_workflows(engine: &mut Engine, workflow_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let workflow_path = Path::new(workflow_dir);
    
    // Check if index.json exists
    let index_path = workflow_path.join("index.json");
    if !index_path.exists() {
        return Err(format!("Index file not found: {:?}", index_path).into());
    }
    
    // Read index file
    let index_content = fs::read_to_string(&index_path)?;
    let index: serde_json::Value = serde_json::from_str(&index_content)?;
    
    // Load workflows from index
    if let Some(workflows) = index.get("workflows").and_then(|w| w.as_array()) {
        for workflow_entry in workflows {
            if let Some(workflow_name) = workflow_entry.as_str() {
                let workflow_subdir = workflow_path.join(workflow_name);
                
                // Load BAH mapping
                let bah_path = workflow_subdir.join("bah-mapping.json");
                if bah_path.exists() {
                    let bah_content = fs::read_to_string(&bah_path)?;
                    let workflow: Workflow = serde_json::from_str(&bah_content)?;
                    engine.add_workflow(&workflow);
                    debug!("Loaded BAH workflow for {}", workflow_name);
                }
                
                // Load document mapping
                let doc_path = workflow_subdir.join("document-mapping.json");
                if doc_path.exists() {
                    let doc_content = fs::read_to_string(&doc_path)?;
                    let workflow: Workflow = serde_json::from_str(&doc_content)?;
                    engine.add_workflow(&workflow);
                    debug!("Loaded document workflow for {}", workflow_name);
                }
                
                // Load preconditions
                let precond_path = workflow_subdir.join("precondition.json");
                if precond_path.exists() {
                    let precond_content = fs::read_to_string(&precond_path)?;
                    let workflow: Workflow = serde_json::from_str(&precond_content)?;
                    engine.add_workflow(&workflow);
                    debug!("Loaded precondition workflow for {}", workflow_name);
                }
                
                info!("✅ Loaded workflows for {}", workflow_name);
            }
        }
    }
    
    Ok(())
}