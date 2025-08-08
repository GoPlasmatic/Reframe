use std::fs;
use std::path::PathBuf;
use serde_json::Value;
use tracing::{debug, error};

pub struct ScenarioManager {
    base_path: PathBuf,
}

impl ScenarioManager {
    pub fn new() -> Self {
        Self {
            base_path: PathBuf::from("scenarios"),
        }
    }
    
    pub fn load_scenario(
        &self,
        message_type: &str,
        scenario_name: Option<&str>,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let (library, clean_type) = self.detect_library(message_type);
        let scenario = scenario_name.unwrap_or("standard");
        
        let scenario_path = self.base_path
            .join(library)
            .join(clean_type)
            .join(format!("{}.json", scenario));
        
        debug!(
            "Loading scenario from: {:?} for message type: {}",
            scenario_path, message_type
        );
        
        // Load and parse scenario JSON
        match fs::read_to_string(&scenario_path) {
            Ok(content) => {
                let json: Value = serde_json::from_str(&content)?;
                Ok(json)
            }
            Err(e) => {
                error!(
                    "Failed to load scenario from {:?}: {}",
                    scenario_path, e
                );
                Err(format!(
                    "Could not load scenario '{}' for message type '{}'",
                    scenario, message_type
                ).into())
            }
        }
    }
    
    pub fn list_available_scenarios(
        &self,
        message_type: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let (library, clean_type) = self.detect_library(message_type);
        let scenario_dir = self.base_path.join(library).join(clean_type);
        
        // Read index.json if it exists
        let index_path = scenario_dir.join("index.json");
        if index_path.exists() {
            let content = fs::read_to_string(index_path)?;
            let index: Value = serde_json::from_str(&content)?;
            
            if let Some(scenarios) = index.get("scenarios").and_then(|s| s.as_array()) {
                let scenario_names: Vec<String> = scenarios
                    .iter()
                    .filter_map(|s| s.as_str().map(|s| s.to_string()))
                    .collect();
                return Ok(scenario_names);
            }
        }
        
        // Fallback: list JSON files in directory
        let mut scenarios = Vec::new();
        if scenario_dir.exists() {
            for entry in fs::read_dir(scenario_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        if stem != "index" {
                            scenarios.push(stem.to_string());
                        }
                    }
                }
            }
        }
        
        Ok(scenarios)
    }
    
    fn detect_library(&self, message_type: &str) -> (&str, String) {
        if message_type.starts_with("MT") {
            // MT103 -> mt103
            ("SwiftMTMessage", message_type.to_lowercase())
        } else {
            // pacs.008 -> pacs008
            ("MXMessage", message_type.replace(".", ""))
        }
    }
}

impl Default for ScenarioManager {
    fn default() -> Self {
        Self::new()
    }
}