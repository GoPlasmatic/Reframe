use async_trait::async_trait;
use dataflow_rs::engine::error::DataflowError;
use dataflow_rs::engine::{
    AsyncFunctionHandler,
    error::Result,
    message::{Change, Message},
};
use serde_json::{Value, json};
use swift_mt_message::SwiftParser;
use tracing::{debug, error, info, instrument};

pub struct ParseMT;

#[async_trait]
impl AsyncFunctionHandler for ParseMT {
    #[instrument(skip(self, message, input))]
    async fn execute(&self, message: &mut Message, input: &Value) -> Result<(usize, Vec<Change>)> {
        debug!("Starting MT message parsing for forward transformation");

        let input_field_name = input
            .get("input_field_name")
            .and_then(Value::as_str)
            .ok_or_else(|| DataflowError::Validation("Missing input_field_name".to_string()))?;

        let output_field_name = input
            .get("output_field_name")
            .and_then(Value::as_str)
            .ok_or_else(|| DataflowError::Validation("Missing output_field_name".to_string()))?;

        let payload = if input_field_name == "payload" {
            message.payload.to_string().replace("\\n", "\n")
        } else {
            message
                .data
                .get(input_field_name)
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string()
        };

        debug!(
            payload_length = payload.len(),
            "Extracted MT payload for parsing"
        );

        self.parse_swift_mt(message, &payload, output_field_name)
            .await
    }
}

impl ParseMT {
    async fn parse_swift_mt(
        &self,
        message: &mut Message,
        payload: &str,
        output_field_name: &str,
    ) -> Result<(usize, Vec<Change>)> {
        debug!("Parsing SwiftMT message for forward transformation");

        let payload = ParseMT::manual_unescape(payload);
        debug!("Parsing MT message with payload length: {}", payload.len());
        let parsed_message = SwiftParser::parse_auto(&payload).map_err(|e| {
            error!(error = ?e, "SwiftMT parsing failed");
            DataflowError::Validation(format!("SwiftMT parser error: {e:?}"))
        })?;

        let message_type = parsed_message.message_type().to_string();
        info!(message_type = %message_type, "Successfully parsed SwiftMT message");

        let method: String;

        let parsed_data = if message_type == "101" {
            let Some(mt101_message) = parsed_message.into_mt101() else {
                error!("Failed to convert SwiftMessage to MT101");
                return Err(DataflowError::Validation(
                    "MT101 message not found in SwiftMT message".to_string(),
                ));
            };

            method = "normal".to_string();
            debug!(method = %method, "Determined MT101 processing method");
            
            // Debug: Check transaction count
            debug!(
                "MT101 has {} transactions", 
                mt101_message.fields.transactions.len()
            );
            
            // Debug: Print first transaction if exists
            if let Some(first_tx) = mt101_message.fields.transactions.first() {
                debug!("First transaction field_21: {:?}", first_tx.field_21);
                debug!("First transaction field_32b: {:?}", first_tx.field_32b);
            }

            let json_value = serde_json::to_value(&mt101_message).map_err(|e| {
                error!(error = ?e, "MT101 JSON conversion failed");
                DataflowError::Validation(format!("MT101 JSON conversion failed: {e}"))
            })?;
            
            // Debug: Check if "#" key exists in JSON
            if let Some(obj) = json_value.as_object() {
                debug!("MT101 JSON keys: {:?}", obj.keys().collect::<Vec<_>>());
                if let Some(hash_field) = obj.get("#") {
                    debug!("Found '#' field with type: {:?}", hash_field);
                    if let Some(arr) = hash_field.as_array() {
                        debug!("'#' field is an array with {} items", arr.len());
                    }
                }
            }
            
            json_value
        } else if message_type == "103" {
            let Some(mt103_message) = parsed_message.into_mt103() else {
                error!("Failed to convert SwiftMessage to MT103");
                return Err(DataflowError::Validation(
                    "MT103 message not found in SwiftMT message".to_string(),
                ));
            };

            method = if mt103_message.has_reject_codes() {
                "reject".to_string()
            } else if mt103_message.has_return_codes() {
                "return".to_string()
            } else if mt103_message.is_stp_message() {
                "stp".to_string()
            } else {
                "normal".to_string()
            };

            debug!(method = %method, "Determined MT103 processing method");

            serde_json::to_value(&mt103_message).map_err(|e| {
                error!(error = ?e, "MT103 JSON conversion failed");
                DataflowError::Validation(format!("MT103 JSON conversion failed: {e}"))
            })?
        } else if message_type == "200" {
            let Some(mt200_message) = parsed_message.into_mt200() else {
                error!("Failed to convert SwiftMessage to MT200");
                return Err(DataflowError::Validation(
                    "MT200 message not found in SwiftMT message".to_string(),
                ));
            };

            method = "normal".to_string();
            debug!("Processing MT200 with normal method");

            serde_json::to_value(&mt200_message).map_err(|e| {
                error!(error = ?e, "MT200 JSON conversion failed");
                DataflowError::Validation(format!("MT200 JSON conversion failed: {e}"))
            })?
        } else if message_type == "202" {
            let Some(mt202_message) = parsed_message.into_mt202() else {
                error!("Failed to convert SwiftMessage to MT202");
                return Err(DataflowError::Validation(
                    "MT202 message not found in SwiftMT message".to_string(),
                ));
            };

            method = if mt202_message.has_reject_codes() {
                "reject".to_string()
            } else if mt202_message.has_return_codes() {
                "return".to_string()
            } else if mt202_message.is_cover_message() {
                "cover".to_string()
            } else if mt202_message.user_header.as_ref()
                .and_then(|h| h.validation_flag.as_ref())
                .map(|flag| flag.as_str() == "COV")
                .unwrap_or(false) {
                "cover".to_string()
            } else if mt202_message.user_header.as_ref()
                .and_then(|h| h.validation_flag.as_ref())
                .map(|flag| flag.as_str() == "RETN")
                .unwrap_or(false) {
                "return".to_string()
            } else if mt202_message.user_header.as_ref()
                .and_then(|h| h.validation_flag.as_ref())
                .map(|flag| flag.as_str() == "REJT")
                .unwrap_or(false) {
                "reject".to_string()
            } else {
                "normal".to_string()
            };

            debug!(method = %method, "Determined MT202 processing method");

            serde_json::to_value(&mt202_message).map_err(|e| {
                error!(error = ?e, "MT202 JSON conversion failed");
                DataflowError::Validation(format!("MT202 JSON conversion failed: {e}"))
            })?
        } else if message_type == "205" {
            let Some(mt205_message) = parsed_message.into_mt205() else {
                error!("Failed to convert SwiftMessage to MT205");
                return Err(DataflowError::Validation(
                    "MT205 message not found in SwiftMT message".to_string(),
                ));
            };

            method = if mt205_message.has_reject_codes() {
                "reject".to_string()
            } else if mt205_message.has_return_codes() {
                "return".to_string()
            } else if mt205_message.is_cover_message() {
                "cover".to_string()
            } else if mt205_message.user_header.as_ref()
                .and_then(|h| h.validation_flag.as_ref())
                .map(|flag| flag.as_str() == "COV")
                .unwrap_or(false) {
                "cover".to_string()
            } else if mt205_message.user_header.as_ref()
                .and_then(|h| h.validation_flag.as_ref())
                .map(|flag| flag.as_str() == "RETN")
                .unwrap_or(false) {
                "return".to_string()
            } else if mt205_message.user_header.as_ref()
                .and_then(|h| h.validation_flag.as_ref())
                .map(|flag| flag.as_str() == "REJT")
                .unwrap_or(false) {
                "reject".to_string()
            } else {
                "normal".to_string()
            };

            debug!(method = %method, "Determined MT205 processing method");

            serde_json::to_value(&mt205_message).map_err(|e| {
                error!(error = ?e, "MT205 JSON conversion failed");
                DataflowError::Validation(format!("MT205 JSON conversion failed: {e}"))
            })?
        } else if message_type == "900" {
            let Some(mt900_message) = parsed_message.into_mt900() else {
                error!("Failed to convert SwiftMessage to MT900");
                return Err(DataflowError::Validation(
                    "MT900 message not found in SwiftMT message".to_string(),
                ));
            };

            method = "normal".to_string();
            debug!("Processing MT900 with normal method");

            serde_json::to_value(&mt900_message).map_err(|e| {
                error!(error = ?e, "MT900 JSON conversion failed");
                DataflowError::Validation(format!("MT900 JSON conversion failed: {e}"))
            })?
        } else if message_type == "910" {
            let Some(mt910_message) = parsed_message.into_mt910() else {
                error!("Failed to convert SwiftMessage to MT910");
                return Err(DataflowError::Validation(
                    "MT910 message not found in SwiftMT message".to_string(),
                ));
            };

            method = "normal".to_string();
            debug!("Processing MT910 with normal method");

            serde_json::to_value(&mt910_message).map_err(|e| {
                error!(error = ?e, "MT910 JSON conversion failed");
                DataflowError::Validation(format!("MT910 JSON conversion failed: {e}"))
            })?
        } else if message_type == "192" {
            let Some(mt192_message) = parsed_message.into_mt192() else {
                error!("Failed to convert SwiftMessage to MT192");
                return Err(DataflowError::Validation(
                    "MT192 message not found in SwiftMT message".to_string(),
                ));
            };

            method = "normal".to_string();
            debug!("Processing MT192 with normal method");

            serde_json::to_value(&mt192_message).map_err(|e| {
                error!(error = ?e, "MT192 JSON conversion failed");
                DataflowError::Validation(format!("MT192 JSON conversion failed: {e}"))
            })?
        } else if message_type == "292" {
            let Some(mt292_message) = parsed_message.into_mt292() else {
                error!("Failed to convert SwiftMessage to MT292");
                return Err(DataflowError::Validation(
                    "MT292 message not found in SwiftMT message".to_string(),
                ));
            };

            method = "normal".to_string();
            debug!("Processing MT292 with normal method");

            serde_json::to_value(&mt292_message).map_err(|e| {
                error!(error = ?e, "MT292 JSON conversion failed");
                DataflowError::Validation(format!("MT292 JSON conversion failed: {e}"))
            })?
        } else if message_type == "196" {
            let Some(mt196_message) = parsed_message.into_mt196() else {
                error!("Failed to convert SwiftMessage to MT196");
                return Err(DataflowError::Validation(
                    "MT196 message not found in SwiftMT message".to_string(),
                ));
            };

            method = "normal".to_string();
            debug!("Processing MT196 with normal method");

            serde_json::to_value(&mt196_message).map_err(|e| {
                error!(error = ?e, "MT196 JSON conversion failed");
                DataflowError::Validation(format!("MT196 JSON conversion failed: {e}"))
            })?
        } else if message_type == "296" {
            let Some(mt296_message) = parsed_message.into_mt296() else {
                error!("Failed to convert SwiftMessage to MT296");
                return Err(DataflowError::Validation(
                    "MT296 message not found in SwiftMT message".to_string(),
                ));
            };

            method = "normal".to_string();
            debug!("Processing MT296 with normal method");

            serde_json::to_value(&mt296_message).map_err(|e| {
                error!(error = ?e, "MT296 JSON conversion failed");
                DataflowError::Validation(format!("MT296 JSON conversion failed: {e}"))
            })?
        } else {
            error!(message_type = %message_type, "Unsupported message type encountered");
            return Err(DataflowError::Validation(format!(
                "Unsupported message type: {message_type}"
            )));
        };

        // Store the parsed result in message data
        if let Some(data_obj) = message.data.as_object_mut() {
            data_obj.insert(output_field_name.to_string(), parsed_data.clone());
        } else {
            message.data = json!({
                output_field_name: parsed_data
            });
        }

        if let Some(data_obj) = message.metadata.as_object_mut() {
            data_obj.insert(
                output_field_name.to_string(),
                json!({
                    "message_type": message_type,
                    "method": method,
                }),
            );
        } else {
            message.metadata = json!({
                output_field_name.to_string(): {
                    "message_type": message_type,
                    "method": method,
                }
            });
        }

        info!(
            message_type = %message_type,
            method = %method,
            output_field = output_field_name,
            "MT message parsing completed successfully for forward transformation"
        );

        Ok((
            200,
            vec![Change {
                path: format!("data.{output_field_name}").to_string(),
                old_value: Value::Null,
                new_value: parsed_data,
            }],
        ))
    }
}

impl ParseMT {
    /// Manual string unescaping for common escape sequences
    fn manual_unescape(input: &str) -> String {
        let mut result = input.trim();

        // Remove surrounding double quotes if present
        if result.starts_with('"') && result.ends_with('"') && result.len() > 1 {
            result = &result[1..result.len() - 1];
        }

        // Now unescape the inner content
        result
            .replace("\\r\\n", "\n")
            .replace("\\r", "\r")
            .replace("\\n", "\n")
            .replace("\\t", "\t")
            .replace("\\\"", "\"")
            .replace("\\'", "'")
            .replace("\\\\", "\\")
            .replace("\\u0020", " ")
            .replace("\\u0022", "\"")
            .replace("\\u003C", "<")
            .replace("\\u003E", ">")
            .replace("\\u003D", "=")
            .replace("\\u002F", "/")
    }
}
