use dataflow_rs::engine::{AsyncFunctionHandler, error::Result, message::{Change, Message}};
use dataflow_rs::engine::error::DataflowError;
use async_trait::async_trait;
use serde_json::{Value, json};
use swift_mt_message::parse_message;

pub struct ParserFunction;

#[async_trait]
impl AsyncFunctionHandler for ParserFunction {
    async fn execute(&self, message: &mut Message, input: &Value) -> Result<(usize, Vec<Change>)> {
        let format = input.get("format").and_then(Value::as_str)
            .ok_or_else(|| DataflowError::Validation("Missing format".to_string()))?;

        let input_field_name = input.get("input_field_name").and_then(Value::as_str)
            .ok_or_else(|| DataflowError::Validation("Missing input_field_name".to_string()))?;

        let output_field_name = input.get("output_field_name").and_then(Value::as_str)
            .ok_or_else(|| DataflowError::Validation("Missing output_field_name".to_string()))?;

        let payload = if input_field_name == "payload" {
            message.payload.to_string().replace("\\n", "\n")
        } else {
            message.data.get(input_field_name).and_then(Value::as_str).unwrap_or("")
                .to_string()
        };

        if format == "SwiftMT" {
            let parsed_data = match parse_message(payload.as_str()) {
                Ok(mt_message) => {
                    // Convert MT message to key-value dictionary
                    let mut fields_dict = std::collections::HashMap::new();
                    
                    if let Ok(mt_value) = serde_json::to_value(&mt_message) {
                        // Navigate to the fields array and convert to dictionary
                        if let Some(mt_obj) = mt_value.as_object() {
                            for (_, message_data) in mt_obj {
                                if let Some(message_obj) = message_data.as_object() {
                                    if let Some(fields_array) = message_obj.get("fields").and_then(|f| f.as_array()) {
                                        for field in fields_array {
                                            if let Some(field_obj) = field.as_object() {
                                                if let (Some(tag), Some(value)) = (
                                                    field_obj.get("tag").and_then(|t| t.as_str()),
                                                    field_obj.get("value").and_then(|v| v.as_str())
                                                ) {
                                                    fields_dict.insert(tag.to_string(), value.to_string());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    fields_dict.insert("message_type".to_string(), mt_message.message_type().to_string());
                    json!(fields_dict)
                },
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

            Ok((200, vec![Change {
                path: format!("data.{}", output_field_name).to_string(),
                old_value: Value::Null,
                new_value: parsed_data,
            }]))
        } else {
            Err(DataflowError::Validation(format!("Unsupported format: {}", format)))
        }
    }
}