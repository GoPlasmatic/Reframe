use async_trait::async_trait;
use dataflow_rs::engine::error::DataflowError;
use dataflow_rs::engine::{
    AsyncFunctionHandler,
    error::Result,
    message::{Change, Message},
};
use serde_json::{Value, json};
use swift_mt_message::SwiftParser;
use swift_mt_message::messages::mt103::MT103;

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

        let mut message_type = "unknown".to_string();

        if format == "SwiftMT" {
            let parsed_data = match SwiftParser::parse::<MT103>(&payload) {
                Ok(mt103_message) => {
                    message_type = mt103_message.message_type.clone();

                    // Convert to JSON using serde_json
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
                }
                Err(e) => {
                    println!("Library parser failed: {:?}", e);

                    json!({
                        "library_error": format!("{:?}", e),
                        "raw_payload": payload,
                        "format": format
                    })
                }
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
                        "message_type": message_type
                    }),
                );
            } else {
                message.metadata = json!({
                    format.to_string(): {
                        "message_type": message_type
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
