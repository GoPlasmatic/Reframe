use crate::helper::Helper;
use async_trait::async_trait;
use dataflow_rs::engine::error::DataflowError;
use dataflow_rs::engine::{
    AsyncFunctionHandler,
    error::Result,
    message::{Change, Message},
};
use serde_json::Value;
use swift_mt_message::SwiftMessage;
use swift_mt_message::messages::{
    MT103, MT110, MT111, MT112, MT199, MT202, MT205, MT299, MT940, MT942,
};
use tracing::{debug, error, instrument};

pub struct PublishMT;

#[async_trait]
impl AsyncFunctionHandler for PublishMT {
    #[instrument(skip(self, message, input))]
    async fn execute(&self, message: &mut Message, input: &Value) -> Result<(usize, Vec<Change>)> {
        debug!("Starting MX to MT message publishing/conversion");

        let source_format = input
            .get("source_format")
            .and_then(Value::as_str)
            .ok_or_else(|| DataflowError::Validation("Missing source_format".to_string()))?;

        let input_field_name = input
            .get("input_field_name")
            .and_then(Value::as_str)
            .ok_or_else(|| DataflowError::Validation("Missing input_field_name".to_string()))?;

        let output_field_name = input
            .get("output_field_name")
            .and_then(Value::as_str)
            .ok_or_else(|| DataflowError::Validation("Missing output_field_name".to_string()))?;

        // Extract data first to avoid borrow checker issues
        let input_data = message.data.get(input_field_name).cloned().ok_or_else(|| {
            error!(
                input_field = %input_field_name,
                available_fields = ?message.data.as_object().map(|obj| obj.keys().collect::<Vec<_>>()),
                "Input field not found in message data for MX to MT transformation"
            );
            DataflowError::Validation(format!(
                "Field {} not found in message data {}",
                input_field_name, message.data
            ))
        })?;

        debug!(data_type = ?input_data, "Processing MX to MT conversion data");

        // Clean up null values from fields before serialization
        let mut cleaned_data = input_data.clone();
        if let Some(fields) = cleaned_data.get_mut("fields").and_then(|f| f.as_object_mut()) {
            fields.retain(|_key, value| {
                if let Some(obj) = value.as_object() {
                    // Check if any values are null and remove those fields
                    !obj.values().any(|v| v.is_null())
                } else {
                    true
                }
            });
        }
        
        let json_str = cleaned_data.to_string();
        let mt_message = if source_format == "pacs.008.001.08" {
            let data: SwiftMessage<MT103> = serde_json::from_str(&json_str).map_err(|e| {
                error!(error = ?e, "Failed to parse JSON string");
                DataflowError::Validation(format!("Failed to parse JSON string: {e}"))
            })?;
            data.to_mt_message()
        } else if source_format == "pacs.004.001.09" {
            let target_message_type = message
                .temp_data
                .get("target_message_type")
                .unwrap_or(&Value::Null)
                .to_string();
            let target_message_type = Helper::manual_unescape(&target_message_type);
            if target_message_type == "103" {
                let data: SwiftMessage<MT103> = serde_json::from_str(&json_str).map_err(|e| {
                    error!(error = ?e, "Failed to parse JSON string");
                    DataflowError::Validation(format!("Failed to parse JSON string: {e}"))
                })?;
                data.to_mt_message()
            } else if target_message_type == "202" {
                let data: SwiftMessage<MT202> = serde_json::from_str(&json_str).map_err(|e| {
                    error!(error = ?e, "Failed to parse JSON string");
                    DataflowError::Validation(format!("Failed to parse JSON string: {e}"))
                })?;
                data.to_mt_message()
            } else if target_message_type == "205" {
                let data: SwiftMessage<MT205> = serde_json::from_str(&json_str).map_err(|e| {
                    error!(error = ?e, "Failed to parse JSON string");
                    DataflowError::Validation(format!("Failed to parse JSON string: {e}"))
                })?;
                data.to_mt_message()
            } else {
                error!(target_message_type = %target_message_type, "Invalid target message type");
                return Err(DataflowError::Validation(format!(
                    "Invalid target message type: {target_message_type}"
                )));
            }
        } else if source_format == "pacs.009.001.08" {
            let target_message_type = message
                .temp_data
                .get("target_message_type")
                .unwrap_or(&Value::Null)
                .to_string();
            let target_message_type = Helper::manual_unescape(&target_message_type);
            if target_message_type == "202" {
                let data: SwiftMessage<MT202> = serde_json::from_str(&json_str).map_err(|e| {
                    error!(error = ?e, "Failed to parse JSON string");
                    DataflowError::Validation(format!("Failed to parse JSON string: {e}"))
                })?;
                data.to_mt_message()
            } else if target_message_type == "205" {
                let data: SwiftMessage<MT205> = serde_json::from_str(&json_str).map_err(|e| {
                    error!(error = ?e, "Failed to parse JSON string");
                    DataflowError::Validation(format!("Failed to parse JSON string: {e}"))
                })?;
                data.to_mt_message()
            } else {
                error!(target_message_type = %target_message_type, "Invalid target message type");
                return Err(DataflowError::Validation(format!(
                    "Invalid target message type: {target_message_type}"
                )));
            }
        } else if source_format == "pacs.002.001.10" {
            let target_message_type = message
                .temp_data
                .get("target_message_type")
                .unwrap_or(&Value::Null)
                .to_string();
            let target_message_type = Helper::manual_unescape(&target_message_type);

            debug!(target_message_type = %target_message_type, "Processing pacs.002 to MT199/MT299 transformation");

            if target_message_type == "199" {
                let data: SwiftMessage<MT199> = serde_json::from_str(&json_str).map_err(|e| {
                    error!(error = ?e, target_message_type = %target_message_type, "Failed to parse JSON string for pacs.002 MT199");
                    DataflowError::Validation(format!("Failed to parse JSON string for pacs.002 MT199: {e}"))
                })?;
                data.to_mt_message()
            } else if target_message_type == "299" {
                let data: SwiftMessage<MT299> = serde_json::from_str(&json_str).map_err(|e| {
                    error!(error = ?e, target_message_type = %target_message_type, "Failed to parse JSON string for pacs.002 MT299");
                    DataflowError::Validation(format!("Failed to parse JSON string for pacs.002 MT299: {e}"))
                })?;
                data.to_mt_message()
            } else {
                error!(target_message_type = %target_message_type, "Invalid target message type for pacs.002");
                return Err(DataflowError::Validation(format!(
                    "Invalid target message type for pacs.002: {target_message_type}"
                )));
            }
        } else if source_format == "camt.107.001.01" {
            debug!("Processing camt.107 to MT110 transformation");

            let data: SwiftMessage<MT110> = serde_json::from_str(&json_str).map_err(|e| {
                error!(error = ?e, "Failed to parse JSON string for camt.107");
                DataflowError::Validation(format!("Failed to parse JSON string for camt.107: {e}"))
            })?;
            data.to_mt_message()
        } else if source_format == "camt.108.001.01" {
            debug!("Processing camt.108 to MT111 transformation");

            let data: SwiftMessage<MT111> = serde_json::from_str(&json_str).map_err(|e| {
                error!(error = ?e, "Failed to parse JSON string for camt.108");
                DataflowError::Validation(format!("Failed to parse JSON string for camt.108: {e}"))
            })?;
            data.to_mt_message()
        } else if source_format == "camt.109.001.01" {
            debug!("Processing camt.109 to MT112 transformation");

            let data: SwiftMessage<MT112> = serde_json::from_str(&json_str).map_err(|e| {
                error!(error = ?e, "Failed to parse JSON string for camt.109");
                DataflowError::Validation(format!("Failed to parse JSON string for camt.109: {e}"))
            })?;
            data.to_mt_message()
        } else if source_format == "camt.052.001.08" {
            debug!("Processing camt.052 to MT942 transformation");

            let data: SwiftMessage<MT942> = serde_json::from_str(&json_str).map_err(|e| {
                error!(error = ?e, "Failed to parse JSON string for camt.052");
                DataflowError::Validation(format!("Failed to parse JSON string for camt.052: {e}"))
            })?;
            data.to_mt_message()
        } else if source_format == "camt.053.001.08" {
            debug!("Processing camt.053 to MT940 transformation");

            let data: SwiftMessage<MT940> = serde_json::from_str(&json_str).map_err(|e| {
                error!(error = ?e, "Failed to parse JSON string for camt.053");
                DataflowError::Validation(format!("Failed to parse JSON string for camt.053: {e}"))
            })?;
            data.to_mt_message()
        } else {
            error!(source_format = %source_format, "Invalid source format");
            return Err(DataflowError::Validation(format!(
                "Invalid source format: {source_format}"
            )));
        };

        debug!(mt_message = %mt_message, "MT message published successfully");

        let result_value = Value::String(mt_message.clone());
        message.data[output_field_name] = result_value.clone();

        Ok((
            200,
            vec![Change {
                path: format!("data.{output_field_name}"),
                old_value: Value::Null,
                new_value: Value::String(mt_message),
            }],
        ))
    }
}
