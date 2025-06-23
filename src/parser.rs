use async_trait::async_trait;
use dataflow_rs::engine::error::DataflowError;
use dataflow_rs::engine::{
    AsyncFunctionHandler,
    error::Result,
    message::{Change, Message},
};
use serde_json::{Value, json};
use swift_mt_message::SwiftParser;
use tracing::{info, debug, warn, error, instrument};

pub struct ParserFunction;

#[async_trait]
impl AsyncFunctionHandler for ParserFunction {
    #[instrument(skip(self, message, input))]
    async fn execute(&self, message: &mut Message, input: &Value) -> Result<(usize, Vec<Change>)> {
        debug!("Starting message parsing");

        let format = input
            .get("format")
            .and_then(Value::as_str)
            .ok_or_else(|| DataflowError::Validation("Missing format".to_string()))?;

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
            "Extracted payload for parsing"
        );

        if format == "SwiftMT" {
            debug!("Parsing SwiftMT message");
            
            let parsed_message = SwiftParser::parse_auto(&payload).map_err(|e| {
                error!(error = ?e, "SwiftMT parsing failed");
                DataflowError::Validation(format!("SwiftMT parser error: {:?}", e))
            })?;
            
            let message_type = parsed_message.message_type().to_string();
            info!(message_type = %message_type, "Successfully parsed SwiftMT message");
            
            let method: String;

            let parsed_data = if message_type == "103" {
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

                match serde_json::to_value(&mt103_message) {
                    Ok(json_value) => {
                        debug!("MT103 JSON conversion successful");
                        json_value
                    }
                    Err(e) => {
                        error!(error = ?e, "MT103 JSON conversion failed");
                        json!({
                            "conversion_error": format!("{:?}", e),
                            "message_type": message_type,
                            "raw_payload": payload
                        })
                    }
                }
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
                } else {
                    "normal".to_string()
                };

                debug!(method = %method, "Determined MT202 processing method");

                match serde_json::to_value(&mt202_message) {
                    Ok(json_value) => {
                        debug!("MT202 JSON conversion successful");
                        json_value
                    }
                    Err(e) => {
                        error!(error = ?e, "MT202 JSON conversion failed");
                        json!({
                            "conversion_error": format!("{:?}", e),
                            "message_type": message_type,
                            "raw_payload": payload
                        })
                    }
                }
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
                } else {
                    "normal".to_string()
                };

                debug!(method = %method, "Determined MT205 processing method");

                match serde_json::to_value(&mt205_message) {
                    Ok(json_value) => {
                        debug!("MT205 JSON conversion successful");
                        json_value
                    }
                    Err(e) => {
                        error!(error = ?e, "MT205 JSON conversion failed");
                        json!({
                            "conversion_error": format!("{:?}", e),
                            "message_type": message_type,
                            "raw_payload": payload
                        })
                    }
                }
            } else if message_type == "900" {
                let Some(mt900_message) = parsed_message.into_mt900() else {
                    error!("Failed to convert SwiftMessage to MT900");
                    return Err(DataflowError::Validation(
                        "MT900 message not found in SwiftMT message".to_string(),
                    ));
                };

                method = "normal".to_string();
                debug!("Processing MT900 with normal method");

                match serde_json::to_value(&mt900_message) {
                    Ok(json_value) => {
                        debug!("MT900 JSON conversion successful");
                        json_value
                    }
                    Err(e) => {
                        error!(error = ?e, "MT900 JSON conversion failed");
                        json!({
                            "conversion_error": format!("{:?}", e),
                            "message_type": message_type,
                            "raw_payload": payload
                        })
                    }
                }
            } else if message_type == "910" {
                let Some(mt910_message) = parsed_message.into_mt910() else {
                    error!("Failed to convert SwiftMessage to MT910");
                    return Err(DataflowError::Validation(
                        "MT910 message not found in SwiftMT message".to_string(),
                    ));
                };

                method = "normal".to_string();
                debug!("Processing MT910 with normal method");

                match serde_json::to_value(&mt910_message) {
                    Ok(json_value) => {
                        debug!("MT910 JSON conversion successful");
                        json_value
                    }
                    Err(e) => {
                        error!(error = ?e, "MT910 JSON conversion failed");
                        json!({
                            "conversion_error": format!("{:?}", e),
                            "message_type": message_type,
                            "raw_payload": payload
                        })
                    }
                }
            } else if message_type == "192" {
                let Some(mt192_message) = parsed_message.into_mt192() else {
                    error!("Failed to convert SwiftMessage to MT192");
                    return Err(DataflowError::Validation(
                        "MT192 message not found in SwiftMT message".to_string(),
                    ));
                };

                method = "normal".to_string();
                debug!("Processing MT192 with normal method");

                match serde_json::to_value(&mt192_message) {
                    Ok(json_value) => {
                        debug!("MT192 JSON conversion successful");
                        json_value
                    }
                    Err(e) => {
                        error!(error = ?e, "MT192 JSON conversion failed");
                        json!({
                            "conversion_error": format!("{:?}", e),
                            "message_type": message_type,
                            "raw_payload": payload
                        })
                    }
                }
            } else if message_type == "292" {
                let Some(mt292_message) = parsed_message.into_mt292() else {
                    error!("Failed to convert SwiftMessage to MT292");
                    return Err(DataflowError::Validation(
                        "MT292 message not found in SwiftMT message".to_string(),
                    ));
                };

                method = "normal".to_string();
                debug!("Processing MT292 with normal method");

                match serde_json::to_value(&mt292_message) {
                    Ok(json_value) => {
                        debug!("MT292 JSON conversion successful");
                        json_value
                    }
                    Err(e) => {
                        error!(error = ?e, "MT292 JSON conversion failed");
                        json!({
                            "conversion_error": format!("{:?}", e),
                            "message_type": message_type,
                            "raw_payload": payload
                        })
                    }
                }
            } else if message_type == "196" {
                let Some(mt196_message) = parsed_message.into_mt196() else {
                    error!("Failed to convert SwiftMessage to MT196");
                    return Err(DataflowError::Validation(
                        "MT196 message not found in SwiftMT message".to_string(),
                    ));
                };

                method = "normal".to_string();
                debug!("Processing MT196 with normal method");

                match serde_json::to_value(&mt196_message) {
                    Ok(json_value) => {
                        debug!("MT196 JSON conversion successful");
                        json_value
                    }
                    Err(e) => {
                        error!(error = ?e, "MT196 JSON conversion failed");
                        json!({
                            "conversion_error": format!("{:?}", e),
                            "message_type": message_type,
                            "raw_payload": payload
                        })
                    }
                }
            } else if message_type == "296" {
                let Some(mt296_message) = parsed_message.into_mt296() else {
                    error!("Failed to convert SwiftMessage to MT296");
                    return Err(DataflowError::Validation(
                        "MT296 message not found in SwiftMT message".to_string(),
                    ));
                };

                method = "normal".to_string();
                debug!("Processing MT296 with normal method");

                match serde_json::to_value(&mt296_message) {
                    Ok(json_value) => {
                        debug!("MT296 JSON conversion successful");
                        json_value
                    }
                    Err(e) => {
                        error!(error = ?e, "MT296 JSON conversion failed");
                        json!({
                            "conversion_error": format!("{:?}", e),
                            "message_type": message_type,
                            "raw_payload": payload
                        })
                    }
                }
            } else {
                method = "normal".to_string();
                warn!(message_type = %message_type, "Unsupported message type encountered");
                json!({
                    "conversion_error": "Unsupported message type",
                    "message_type": message_type,
                    "raw_payload": payload
                })
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
                    format.to_string(),
                    json!({
                        "message_type": message_type,
                        "method": method
                    }),
                );
            } else {
                message.metadata = json!({
                    format.to_string(): {
                        "message_type": message_type,
                        "method": method
                    }
                });
            }

            info!(
                message_type = %message_type,
                method = %method,
                output_field = output_field_name,
                "Message parsing completed successfully"
            );

            Ok((
                200,
                vec![Change {
                    path: format!("data.{}", output_field_name).to_string(),
                    old_value: Value::Null,
                    new_value: parsed_data,
                }],
            ))
        } else {
            error!(format = format, "Unsupported message format");
            Err(DataflowError::Validation(format!(
                "Unsupported format: {}",
                format
            )))
        }
    }
}
