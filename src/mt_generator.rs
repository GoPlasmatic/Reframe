use serde_json::Value;
use serde_path_to_error::deserialize;
use swift_mt_message::SwiftMessage;
use swift_mt_message::messages::*;
use tracing::{debug, error};

/// Macro to parse MT message with detailed error path
macro_rules! parse_mt_message {
    ($message_type:ty, $type_name:expr, $json_data:expr) => {{
        // Convert Value to string first, then create deserializer
        let json_str = serde_json::to_string($json_data)
            .map_err(|e| format!("Failed to serialize JSON data: {}", e))?;
        let jd = &mut serde_json::Deserializer::from_str(&json_str);
        let msg: SwiftMessage<$message_type> = deserialize(jd).map_err(|e| {
            let path = e.path().to_string();
            let inner_err = e.into_inner().to_string();

            // Provide more context for common error patterns
            let error_context = if inner_err.contains("invalid characters") {
                format!(" (check for non-ASCII or control characters in this field)")
            } else if inner_err.contains("invalid type") {
                format!(" (type mismatch - check if field should be string, number, or object)")
            } else if inner_err.contains("missing field") {
                format!(" (required field is missing)")
            } else {
                String::new()
            };

            format!(
                "Failed to parse {} - Error at field path '{}': {}{}",
                $type_name, path, inner_err, error_context
            )
        })?;
        msg.to_mt_message()
    }};
}

/// Generate MT message from JSON data
pub fn generate_mt_from_json(
    message_type: &str,
    json_data: &Value,
) -> Result<String, Box<dyn std::error::Error>> {
    debug!("Generating {} from JSON data", message_type);

    // Parse JSON into appropriate SwiftMessage type and serialize to MT format
    let mt_string = match message_type {
        "MT101" => parse_mt_message!(MT101, "MT101", json_data),
        "MT103" => parse_mt_message!(MT103, "MT103", json_data),
        "MT104" => parse_mt_message!(MT104, "MT104", json_data),
        "MT107" => parse_mt_message!(MT107, "MT107", json_data),
        "MT110" => parse_mt_message!(MT110, "MT110", json_data),
        "MT111" => parse_mt_message!(MT111, "MT111", json_data),
        "MT112" => parse_mt_message!(MT112, "MT112", json_data),
        "MT192" => parse_mt_message!(MT192, "MT192", json_data),
        "MT196" => parse_mt_message!(MT196, "MT196", json_data),
        "MT199" => parse_mt_message!(MT199, "MT199", json_data),
        "MT200" => parse_mt_message!(MT200, "MT200", json_data),
        "MT202" => parse_mt_message!(MT202, "MT202", json_data),
        "MT205" => parse_mt_message!(MT205, "MT205", json_data),
        "MT210" => parse_mt_message!(MT210, "MT210", json_data),
        "MT292" => parse_mt_message!(MT292, "MT292", json_data),
        "MT296" => parse_mt_message!(MT296, "MT296", json_data),
        "MT299" => parse_mt_message!(MT299, "MT299", json_data),
        "MT900" => parse_mt_message!(MT900, "MT900", json_data),
        "MT910" => parse_mt_message!(MT910, "MT910", json_data),
        "MT920" => parse_mt_message!(MT920, "MT920", json_data),
        "MT935" => parse_mt_message!(MT935, "MT935", json_data),
        "MT940" => parse_mt_message!(MT940, "MT940", json_data),
        "MT941" => parse_mt_message!(MT941, "MT941", json_data),
        "MT942" => parse_mt_message!(MT942, "MT942", json_data),
        "MT950" => parse_mt_message!(MT950, "MT950", json_data),
        _ => {
            error!("Unsupported MT message type: {}", message_type);
            return Err(format!("Unsupported MT message type: {}", message_type).into());
        }
    };

    debug!("Successfully generated {} message", message_type);
    Ok(mt_string)
}

/// Format MT message string with proper line endings
pub fn format_mt_message(mt_string: &str) -> String {
    mt_string
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<&str>>()
        .join("\n")
}
