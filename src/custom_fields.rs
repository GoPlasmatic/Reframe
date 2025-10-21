//! Custom Fields Module
//!
//! This module provides support for package-specific custom fields that are computed
//! using JSONLogic expressions and exposed through the GraphQL API.
//!
//! # Features
//! - JSONLogic-based field evaluation
//! - Three storage strategies: precompute, runtime, hybrid
//! - Error handling (returns null on failures)
//! - Support for multiple data types: string, number, boolean, json

use dataflow_rs::engine::message::Message;
use datalogic_rs::DataLogic;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tracing::{debug, warn};

use crate::graphql::types::TransformationMessage;

/// Storage strategy for custom fields
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CustomFieldStorage {
    /// Compute once at storage time, store in DB
    /// Best for: Fields used in filters/sorting, expensive computations, stable values
    Precompute,

    /// Never store, always compute at query time
    /// Best for: Time-dependent fields, volatile computations
    Runtime,

    /// Store for performance, but allow recomputation
    /// Best for: Fields where logic might change, need both speed and flexibility
    Hybrid,
}

/// Definition of a custom field from package configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CustomFieldDefinition {
    /// Unique field name (use snake_case)
    pub name: String,

    /// Human-readable description
    pub description: String,

    /// Field value type: "string" | "number" | "boolean" | "json"
    #[serde(rename = "type")]
    pub field_type: String,

    /// Storage strategy
    pub storage: CustomFieldStorage,

    /// JSONLogic expression to evaluate
    pub logic: Value,
}

/// API configuration structure loaded from api_config.json
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApiConfig {
    /// List of custom field definitions
    pub custom_fields: Vec<CustomFieldDefinition>,
}

/// Trait to extract context from message types for JSONLogic evaluation
trait ContextProvider {
    fn get_context(&self) -> &Value;
}

impl ContextProvider for Message {
    fn get_context(&self) -> &Value {
        &self.context
    }
}

impl ContextProvider for TransformationMessage {
    fn get_context(&self) -> &Value {
        &self.context
    }
}

/// Evaluate a JSONLogic expression against any message type that provides context
///
/// # Arguments
/// * `logic` - The JSONLogic expression to evaluate
/// * `message` - Any type that implements ContextProvider
///
/// # Returns
/// The evaluated value, or null if evaluation fails
fn evaluate_jsonlogic<T: ContextProvider>(logic: &Value, message: &T) -> Value {
    // Extract only the context from the message (consistent with workflow engine behavior)
    let context = message.get_context().clone();

    // Create DataLogic instance, compile and evaluate
    let datalogic = DataLogic::new();
    match datalogic.compile(logic) {
        Ok(compiled) => match datalogic.evaluate_owned(&compiled, context) {
            Ok(result) => result,
            Err(e) => {
                warn!(
                    error = %e,
                    logic = %serde_json::to_string(logic).unwrap_or_else(|_| "invalid".to_string()),
                    "JSONLogic evaluation failed, returning null"
                );
                Value::Null
            }
        },
        Err(e) => {
            warn!(
                error = %e,
                logic = %serde_json::to_string(logic).unwrap_or_else(|_| "invalid".to_string()),
                "JSONLogic compilation failed, returning null"
            );
            Value::Null
        }
    }
}

/// Compute and store custom fields at message storage time
///
/// This function evaluates all custom fields with storage strategy "precompute" or "hybrid"
/// and stores the results in the message context.
///
/// # Arguments
/// * `message` - The message to compute fields for (modified in place)
/// * `custom_field_defs` - Custom field definitions from the package
///
/// # Returns
/// Ok(()) on success, or an error if computation fails
pub fn compute_and_store_fields(
    message: &mut Message,
    custom_field_defs: &[CustomFieldDefinition],
) -> Result<(), String> {
    let mut stored_fields = serde_json::Map::new();

    for field_def in custom_field_defs {
        // Only compute storage-eligible fields
        match field_def.storage {
            CustomFieldStorage::Precompute | CustomFieldStorage::Hybrid => {
                debug!(
                    field_name = %field_def.name,
                    storage = ?field_def.storage,
                    "Computing custom field at storage time"
                );

                let value = evaluate_jsonlogic(&field_def.logic, message);
                stored_fields.insert(field_def.name.clone(), value);
            }
            CustomFieldStorage::Runtime => {
                // Skip - will be computed at query time
                debug!(
                    field_name = %field_def.name,
                    "Skipping runtime-only custom field at storage time"
                );
            }
        }
    }

    // Store in message context
    if let Some(context) = message.context.as_object_mut() {
        context.insert(
            "custom_fields".to_string(),
            Value::Object(stored_fields.clone()),
        );
        debug!(
            custom_fields = ?stored_fields,
            context_keys = ?context.keys().collect::<Vec<_>>(),
            "Stored custom fields in message context"
        );
    } else {
        return Err("Message context is not an object".to_string());
    }

    Ok(())
}

/// Compute custom fields at query time for GraphQL resolvers
///
/// This function evaluates custom fields based on their storage strategy:
/// - Runtime: Always compute fresh
/// - Hybrid: Use stored value by default, recompute if requested
/// - Precompute: Use stored value by default, recompute if requested
///
/// # Arguments
/// * `message` - The transformation message from the database
/// * `custom_field_defs` - Custom field definitions from the package
/// * `recompute_all` - If true, recompute all fields regardless of storage strategy
///
/// # Returns
/// A map of field names to computed values
pub fn compute_runtime_fields(
    message: &TransformationMessage,
    custom_field_defs: &[CustomFieldDefinition],
    recompute_all: bool,
) -> HashMap<String, Value> {
    let mut fields = HashMap::new();

    for field_def in custom_field_defs {
        let value = match field_def.storage {
            CustomFieldStorage::Runtime => {
                // Always compute fresh for runtime fields
                debug!(
                    field_name = %field_def.name,
                    "Computing runtime custom field"
                );
                evaluate_jsonlogic(&field_def.logic, message)
            }
            CustomFieldStorage::Hybrid if recompute_all => {
                // Recompute hybrid fields on demand
                debug!(
                    field_name = %field_def.name,
                    "Recomputing hybrid custom field"
                );
                evaluate_jsonlogic(&field_def.logic, message)
            }
            CustomFieldStorage::Precompute | CustomFieldStorage::Hybrid => {
                // Use stored value from context.custom_fields
                message
                    .context
                    .get("custom_fields")
                    .and_then(|cf| cf.get(&field_def.name))
                    .cloned()
                    .unwrap_or_else(|| {
                        debug!(
                            field_name = %field_def.name,
                            "Custom field not found in stored data, returning null"
                        );
                        Value::Null
                    })
            }
        };

        fields.insert(field_def.name.clone(), value);
    }

    debug!(
        field_count = fields.len(),
        recompute = recompute_all,
        "Computed runtime custom fields"
    );

    fields
}

/// Validate custom field definitions
///
/// Ensures that field definitions are valid and consistent.
///
/// # Arguments
/// * `definitions` - The custom field definitions to validate
///
/// # Returns
/// Ok(()) if valid, or an error describing the validation failure
pub fn validate_custom_field_definitions(
    definitions: &[CustomFieldDefinition],
) -> Result<(), String> {
    let mut seen_names = std::collections::HashSet::new();

    for def in definitions {
        // Check for duplicate names
        if !seen_names.insert(&def.name) {
            return Err(format!("Duplicate custom field name: {}", def.name));
        }

        // Validate field type
        if !matches!(
            def.field_type.as_str(),
            "string" | "number" | "int" | "boolean" | "datetime" | "json"
        ) {
            return Err(format!(
                "Invalid field type '{}' for field '{}'. Must be: string, number, int, boolean, datetime, or json",
                def.field_type, def.name
            ));
        }

        // Validate field name (snake_case convention)
        if !def
            .name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c == '_' || c.is_ascii_digit())
        {
            warn!(
                field_name = %def.name,
                "Custom field name should use snake_case convention"
            );
        }

        // Validate JSONLogic is valid JSON
        if !def.logic.is_object()
            && !def.logic.is_array()
            && !def.logic.is_string()
            && !def.logic.is_number()
            && !def.logic.is_boolean()
        {
            return Err(format!(
                "Invalid JSONLogic for field '{}': logic must be a valid JSON value",
                def.name
            ));
        }
    }

    debug!(
        field_count = definitions.len(),
        "Custom field definitions validated successfully"
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_field_storage_serialization() {
        let precompute = CustomFieldStorage::Precompute;
        let json = serde_json::to_string(&precompute).unwrap();
        assert_eq!(json, "\"precompute\"");

        let runtime = CustomFieldStorage::Runtime;
        let json = serde_json::to_string(&runtime).unwrap();
        assert_eq!(json, "\"runtime\"");

        let hybrid = CustomFieldStorage::Hybrid;
        let json = serde_json::to_string(&hybrid).unwrap();
        assert_eq!(json, "\"hybrid\"");
    }

    #[test]
    fn test_custom_field_definition_deserialization() {
        let json = r#"{
            "name": "test_field",
            "description": "Test field",
            "type": "string",
            "storage": "precompute",
            "logic": {"var": "context.data.field"}
        }"#;

        let def: CustomFieldDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(def.name, "test_field");
        assert_eq!(def.field_type, "string");
        assert_eq!(def.storage, CustomFieldStorage::Precompute);
    }

    #[test]
    fn test_validate_custom_field_definitions() {
        let valid_defs = vec![
            CustomFieldDefinition {
                name: "field1".to_string(),
                description: "Test".to_string(),
                field_type: "string".to_string(),
                storage: CustomFieldStorage::Precompute,
                logic: serde_json::json!({"var": "test"}),
            },
            CustomFieldDefinition {
                name: "field2".to_string(),
                description: "Test".to_string(),
                field_type: "number".to_string(),
                storage: CustomFieldStorage::Runtime,
                logic: serde_json::json!({"var": "test"}),
            },
        ];

        assert!(validate_custom_field_definitions(&valid_defs).is_ok());
    }

    #[test]
    fn test_validate_duplicate_field_names() {
        let invalid_defs = vec![
            CustomFieldDefinition {
                name: "field1".to_string(),
                description: "Test".to_string(),
                field_type: "string".to_string(),
                storage: CustomFieldStorage::Precompute,
                logic: serde_json::json!({"var": "test"}),
            },
            CustomFieldDefinition {
                name: "field1".to_string(), // Duplicate
                description: "Test".to_string(),
                field_type: "number".to_string(),
                storage: CustomFieldStorage::Runtime,
                logic: serde_json::json!({"var": "test"}),
            },
        ];

        assert!(validate_custom_field_definitions(&invalid_defs).is_err());
    }

    #[test]
    fn test_validate_invalid_field_type() {
        let invalid_defs = vec![CustomFieldDefinition {
            name: "field1".to_string(),
            description: "Test".to_string(),
            field_type: "invalid_type".to_string(),
            storage: CustomFieldStorage::Precompute,
            logic: serde_json::json!({"var": "test"}),
        }];

        assert!(validate_custom_field_definitions(&invalid_defs).is_err());
    }
}
