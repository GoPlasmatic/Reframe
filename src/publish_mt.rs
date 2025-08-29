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
    MT101, MT103, MT104, MT110, MT111, MT112, MT190, MT191, MT192, MT196, MT199, MT202, MT204, MT205, MT210, 
    MT290, MT291, MT292, MT296, MT299, MT900, MT910, MT940, MT942,
};
use tracing::{debug, error, instrument};

/// Macro for parsing JSON to SwiftMessage with detailed error reporting
macro_rules! parse_swift_message {
    ($json_str:expr, $mt_type:ty) => {{
        let mut deserializer = serde_json::Deserializer::from_str($json_str);
        let result: SwiftMessage<$mt_type> = serde_path_to_error::deserialize(&mut deserializer)
            .map_err(|e| {
                let error_msg = format!(
                    "Failed to parse JSON at path '{}': {}",
                    e.path(),
                    e.inner()
                );
                error!(
                    error = %error_msg,
                    json_preview = %&$json_str[..$json_str.len().min(500)],
                    "Failed to parse JSON string"
                );
                DataflowError::Validation(error_msg)
            })?;
        result
    }};
}

/// Helper function to clean null values from fields before serialization
fn clean_null_fields(data: &Value) -> Value {
    let mut cleaned = data.clone();
    if let Some(fields) = cleaned.get_mut("fields").and_then(|f| f.as_object_mut()) {
        fields.retain(|_key, value| {
            if let Some(obj) = value.as_object() {
                // Remove fields where any nested values are null
                !obj.values().any(|v| v.is_null())
            } else {
                true // Keep non-object values
            }
        });
    }
    cleaned
}

/// Helper function to parse MT message based on message type number
fn parse_mt_by_type(json_str: &str, mt_type: &str) -> Result<String> {
    macro_rules! mt_match {
        ($($num:literal => $type:ty),+) => {
            match mt_type {
                $(
                    $num => {
                        let data = parse_swift_message!(json_str, $type);
                        Ok(data.to_mt_message())
                    }
                )+
                _ => {
                    error!("Unsupported MT type: {}", mt_type);
                    Err(DataflowError::Validation(format!("Unsupported MT type: {}", mt_type)))
                }
            }
        };
    }

    mt_match!(
        "101" => MT101,
        "103" => MT103,
        "104" => MT104,
        "110" => MT110,
        "111" => MT111,
        "112" => MT112,
        "190" => MT190,
        "191" => MT191,
        "192" => MT192,
        "196" => MT196,
        "199" => MT199,
        "202" => MT202,
        "204" => MT204,
        "205" => MT205,
        "210" => MT210,
        "290" => MT290,
        "291" => MT291,
        "292" => MT292,
        "296" => MT296,
        "299" => MT299,
        "900" => MT900,
        "910" => MT910,
        "940" => MT940,
        "942" => MT942
    )
}

/// Generic routing for messages that require message_type field
fn route_by_message_type(
    json_str: &str,
    input_data: &Value,
    source_format: &str,
    valid_types: &[&str],
) -> Result<String> {
    // Extract message_type from input data
    let message_type = match input_data.get("message_type") {
        Some(Value::String(s)) => s.clone(),
        Some(v) if !v.is_null() => {
            error!("message_type field is not a string: {:?}", v);
            return Err(DataflowError::Validation(format!(
                "message_type field must be a string for {}",
                source_format
            )));
        }
        _ => {
            error!("Missing message_type field for {}", source_format);
            return Err(DataflowError::Validation(format!(
                "message_type field is required for {} transformation",
                source_format
            )));
        }
    };

    debug!(message_type = %message_type, "Processing {} to MT{} transformation", source_format, message_type);

    // Validate and route to appropriate MT type
    if valid_types.contains(&message_type.as_str()) {
        parse_mt_by_type(json_str, &message_type)
    } else {
        error!(message_type = %message_type, "Invalid message type for {}", source_format);
        Err(DataflowError::Validation(format!(
            "Invalid message type '{}' for {}. Expected one of: {}",
            message_type,
            source_format,
            valid_types.join(", ")
        )))
    }
}

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

        // Clean null values from fields before serialization (required for swift-mt-message library)
        let cleaned_data = clean_null_fields(&input_data);
        let json_str = cleaned_data.to_string();

        // Determine the target MT type(s) for this source format
        let mt_types = match source_format {
            "pain.001.001.09" => vec!["101"],
            "pacs.008.001.08" => vec!["103"],
            "pacs.003.001.08" => vec!["104"],
            "pacs.004.001.09" => vec!["103", "202", "205"],
            "pacs.009.001.08" => vec!["202", "205"],
            "pacs.010.001.03" => vec!["204"],
            "pacs.002.001.10" => vec!["199", "299"],
            "camt.107.001.01" => vec!["110"],
            "camt.108.001.01" => vec!["111"],
            "camt.109.001.01" => vec!["112"],
            "camt.029.001.09" => vec!["196", "296"],
            "camt.052.001.08" => vec!["942"],
            "camt.053.001.08" => vec!["940"],
            "camt.054.001.08" => vec!["103", "202", "900", "910"],
            "camt.056.001.08" => vec!["192", "292"],
            "camt.057.001.06" => vec!["210"],
            "camt.058.001.08" => vec!["292"],
            "camt.105.001.02" => vec!["190", "290"],
            "camt.106.001.02" => vec!["191", "291"],
            "admi.024.001.01" => vec!["199"],
            _ => {
                error!(source_format = %source_format, "Invalid source format");
                return Err(DataflowError::Validation(format!(
                    "Invalid source format: {source_format}"
                )));
            }
        };

        // Check if routing is needed (more than one possible MT type)
        let needs_routing = mt_types.len() > 1;

        // Log the transformation being performed
        if needs_routing {
            debug!(
                source = %source_format,
                possible_targets = ?mt_types,
                "Processing {} transformation with message_type routing",
                source_format
            );
        } else {
            debug!(
                source = %source_format,
                target = %mt_types[0],
                "Processing {} to MT{} transformation",
                source_format,
                mt_types[0]
            );
        }

        // Perform the transformation
        let mt_message = if needs_routing {
            route_by_message_type(&json_str, &input_data, source_format, &mt_types)?
        } else {
            parse_mt_by_type(&json_str, mt_types[0])?
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
