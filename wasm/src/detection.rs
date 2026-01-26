//! Generic detection function for payload analysis
//!
//! Provides a minimal, workflow-controlled detection function that:
//! - Runs configurable regex patterns against the payload
//! - Extracts basic payload characteristics
//! - Allows workflows to implement their own detection logic

use async_trait::async_trait;
use dataflow_rs::engine::error::DataflowError;
use dataflow_rs::engine::{
    error::Result,
    message::{Change, Message},
    AsyncFunctionHandler, FunctionConfig,
};
use datalogic_rs::DataLogic;
use regex::Regex;
use serde_json::{json, Value};
use std::sync::Arc;

pub struct Detect;

impl Detect {
    /// Sets a value at a nested path in the message context using dot notation
    /// Example: "metadata.detection" will create/update metadata.detection field
    fn set_value_at_path(&self, message: &mut Message, path: &str, value: Value) -> Result<()> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = &mut message.context;

        // Navigate through the path, creating intermediate objects as needed
        for (i, &part) in parts.iter().enumerate() {
            // Ensure current level is an object
            if !current.is_object() {
                *current = json!({});
            }

            if i == parts.len() - 1 {
                // Last part - set the value
                current
                    .as_object_mut()
                    .ok_or_else(|| {
                        DataflowError::Validation(format!("Path '{}' is not an object", path))
                    })?
                    .insert(part.to_string(), value);
                return Ok(());
            } else {
                // Navigate to next level, creating if needed
                current = current
                    .as_object_mut()
                    .ok_or_else(|| {
                        DataflowError::Validation(format!("Path '{}' is not an object", path))
                    })?
                    .entry(part.to_string())
                    .or_insert_with(|| json!({}));
            }
        }

        Err(DataflowError::Validation(format!(
            "Failed to set value at path '{}'",
            path
        )))
    }
}

#[async_trait]
impl AsyncFunctionHandler for Detect {
    async fn execute(
        &self,
        message: &mut Message,
        config: &FunctionConfig,
        _datalogic: Arc<DataLogic>,
    ) -> Result<(usize, Vec<Change>)> {
        // Extract custom configuration
        let input = match config {
            FunctionConfig::Custom { input, .. } => input,
            _ => {
                return Err(DataflowError::Validation(
                    "Invalid configuration type".to_string(),
                ));
            }
        };

        // Get source field (default to "payload")
        let source = input
            .get("source")
            .and_then(Value::as_str)
            .unwrap_or("payload");

        // Get target field (default to "metadata.detection")
        let target = input
            .get("target")
            .and_then(Value::as_str)
            .unwrap_or("metadata.detection");

        // Extract payload content
        let content = if source == "payload" {
            // Get the actual string value from the payload, not the JSON representation
            message
                .payload
                .as_str()
                .ok_or_else(|| DataflowError::Validation("Payload is not a string".to_string()))?
                .to_string()
        } else {
            // Try to get from data
            message
                .data()
                .get(source)
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    DataflowError::Validation(format!("Source field '{}' not found", source))
                })?
                .to_string()
        };

        // Run regex patterns if provided
        let mut matches = serde_json::Map::new();
        if let Some(patterns) = input.get("patterns").and_then(Value::as_array) {
            for pattern in patterns {
                if let (Some(name), Some(regex_str)) = (
                    pattern.get("name").and_then(Value::as_str),
                    pattern.get("regex").and_then(Value::as_str),
                ) {
                    match Regex::new(regex_str) {
                        Ok(regex) => {
                            if let Some(captures) = regex.captures(&content) {
                                // If there are capture groups, store them as array
                                if captures.len() > 1 {
                                    let captured: Vec<String> = captures
                                        .iter()
                                        .skip(1) // Skip the full match
                                        .filter_map(|c| c.map(|m| m.as_str().to_string()))
                                        .collect();
                                    matches.insert(name.to_string(), json!(captured));
                                } else {
                                    // Just a boolean match
                                    matches.insert(name.to_string(), json!(true));
                                }
                            } else {
                                matches.insert(name.to_string(), json!(false));
                            }
                        }
                        Err(_e) => {
                            matches.insert(name.to_string(), json!(false));
                        }
                    }
                }
            }
        }

        // Extract basic info - always provide all standard fields
        let mut info = serde_json::Map::new();

        // First line
        let first_line = content.lines().next().unwrap_or("").to_string();
        info.insert("first_line".to_string(), json!(first_line));

        // Byte size
        info.insert("byte_size".to_string(), json!(content.len()));

        // Character count
        info.insert("char_count".to_string(), json!(content.chars().count()));

        // Line count
        info.insert("line_count".to_string(), json!(content.lines().count()));

        // Simple encoding detection (UTF-8 vs binary)
        let encoding = if content.is_ascii() {
            "ascii"
        } else if content
            .chars()
            .all(|c| !c.is_control() || c.is_whitespace())
        {
            "utf-8"
        } else {
            "binary"
        };
        info.insert("encoding".to_string(), json!(encoding));

        // Build result
        let result = json!({
            "matches": matches,
            "info": info
        });

        // Actually persist the data to message.context
        self.set_value_at_path(message, target, result.clone())?;

        // Write to target path for audit trail
        let changes = vec![Change {
            path: Arc::from(target),
            old_value: Arc::new(Value::Null),
            new_value: Arc::new(result),
        }];

        // Invalidate cache after modifications
        message.invalidate_context_cache();

        Ok((200, changes))
    }
}
