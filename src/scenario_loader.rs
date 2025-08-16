use serde_json::Value;
use std::fs;
use std::path::Path;
use tracing::debug;

#[derive(Debug)]
pub struct ScenarioIndex {
    forward: Vec<ScenarioEntry>,
    reverse: Vec<ScenarioEntry>,
}

#[derive(Debug, Clone)]
struct ScenarioEntry {
    id: String,
    file: String,
    source: String,
    #[allow(dead_code)]
    target: String,
    #[allow(dead_code)]
    description: String,
}

impl ScenarioIndex {
    /// Load the scenario index from scenarios/index.json
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let index_path = Path::new("scenarios/index.json");
        let content = fs::read_to_string(index_path)?;
        let data: Value = serde_json::from_str(&content)?;
        
        let forward = data["forward"]
            .as_array()
            .ok_or("Missing forward array")?
            .iter()
            .filter_map(|entry| {
                Some(ScenarioEntry {
                    id: entry["id"].as_str()?.to_string(),
                    file: entry["file"].as_str()?.to_string(),
                    source: entry["source"].as_str()?.to_string(),
                    target: entry["target"].as_str()?.to_string(),
                    description: entry["description"].as_str()?.to_string(),
                })
            })
            .collect();
            
        let reverse = data["reverse"]
            .as_array()
            .ok_or("Missing reverse array")?
            .iter()
            .filter_map(|entry| {
                Some(ScenarioEntry {
                    id: entry["id"].as_str()?.to_string(),
                    file: entry["file"].as_str()?.to_string(),
                    source: entry["source"].as_str()?.to_string(),
                    target: entry["target"].as_str()?.to_string(),
                    description: entry["description"].as_str()?.to_string(),
                })
            })
            .collect();
            
        Ok(ScenarioIndex { forward, reverse })
    }
    
    /// Find a scenario by message type and scenario ID
    pub fn find_scenario(&self, message_type: &str, scenario_id: &str) -> Option<String> {
        // Search in forward scenarios
        for entry in &self.forward {
            if entry.source == message_type && entry.id == scenario_id {
                return Some(entry.file.clone());
            }
        }
        
        // Search in reverse scenarios
        for entry in &self.reverse {
            if entry.source == message_type && entry.id == scenario_id {
                return Some(entry.file.clone());
            }
        }
        
        None
    }
    
    /// Get all scenario IDs for a message type
    pub fn get_scenario_ids(&self, message_type: &str) -> Vec<String> {
        let mut ids = Vec::new();
        
        // Search in forward scenarios
        for entry in &self.forward {
            if entry.source == message_type {
                ids.push(entry.id.clone());
            }
        }
        
        // Search in reverse scenarios
        for entry in &self.reverse {
            if entry.source == message_type {
                ids.push(entry.id.clone());
            }
        }
        
        ids
    }
}

#[allow(dead_code)]
/// Load a scenario file and return its schema and variables
pub fn load_scenario_file(file_path: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let full_path = Path::new("scenarios").join(file_path);
    let content = fs::read_to_string(&full_path)?;
    let data: Value = serde_json::from_str(&content)?;
    
    Ok(data)
}

/// Get scenario file path from index
pub fn get_scenario_file_path(
    message_type: &str,
    scenario_id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    debug!("Finding scenario file for {} with ID: {}", message_type, scenario_id);
    
    // Load the scenario index
    let index = ScenarioIndex::load()?;
    
    // Find the scenario file
    let scenario_file = index
        .find_scenario(message_type, scenario_id)
        .ok_or_else(|| {
            let available_ids = index.get_scenario_ids(message_type);
            format!(
                "Scenario '{}' not found for {}. Available scenarios: {:?}",
                scenario_id, message_type, available_ids
            )
        })?;
    
    debug!("Found scenario file: {}", scenario_file);
    Ok(scenario_file)
}

