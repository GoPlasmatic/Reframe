use async_trait::async_trait;
use dataflow_rs::engine::error::DataflowError;
use dataflow_rs::engine::{
    AsyncFunctionHandler,
    error::Result,
    message::{Change, Message},
};
use serde_json::{Value, json};
use swift_mt_message::SwiftParser;

pub struct ParserFunction;

#[async_trait]
impl AsyncFunctionHandler for ParserFunction {
    async fn execute(&self, message: &mut Message, input: &Value) -> Result<(usize, Vec<Change>)> {
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

        if format == "SwiftMT" {
            let parsed_message = SwiftParser::parse_auto(&payload)
                .map_err(|e| DataflowError::Validation(format!("SwiftMT parser error: {:?}", e)))?;
            let message_type = parsed_message.message_type().to_string();
            let method: String;

            let parsed_data = if message_type == "103" {
                let Some(mt103_message) = parsed_message.into_mt103() else {
                    return Err(DataflowError::Validation(
                        "MT103 message not found in SwiftMT message".to_string(),
                    ));
                };

                method = if mt103_message.fields.has_reject_codes() {
                    "reject".to_string()
                } else if mt103_message.fields.has_return_codes() {
                    "return".to_string()
                } else if mt103_message.fields.is_stp_compliant() {
                    "stp".to_string()
                } else {
                    "normal".to_string()
                };

                match serde_json::to_value(&mt103_message) {
                    Ok(json_value) => json_value,
                    Err(e) => {
                        println!("JSON conversion failed: {:?}", e);
                        json!({
                            "conversion_error": format!("{:?}", e),
                            "message_type": message_type,
                            "raw_payload": payload
                        })
                    }
                }
            } else if message_type == "202" {
                let Some(mt202_message) = parsed_message.into_mt202() else {
                    return Err(DataflowError::Validation(
                        "MT202 message not found in SwiftMT message".to_string(),
                    ));
                };
                method = "normal".to_string();
                match serde_json::to_value(&mt202_message) {
                    Ok(json_value) => json_value,
                    Err(e) => {
                        println!("JSON conversion failed: {:?}", e);
                        json!({
                            "conversion_error": format!("{:?}", e),
                            "message_type": message_type,
                            "raw_payload": payload
                        })
                    }
                }
            } else {
                method = "normal".to_string();
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

            Ok((
                200,
                vec![Change {
                    path: format!("data.{}", output_field_name).to_string(),
                    old_value: Value::Null,
                    new_value: parsed_data,
                }],
            ))
        } else {
            Err(DataflowError::Validation(format!(
                "Unsupported format: {}",
                format
            )))
        }
    }
}
