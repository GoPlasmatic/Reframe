use serde_json::Value;
use swift_mt_message::{
    SwiftMessage,
    messages::{
        MT101, MT103, MT104, MT107, MT110, MT111, MT112, MT192, MT196, MT199, MT202, MT205, MT210,
        MT292, MT296, MT299, MT900, MT910, MT920, MT935, MT940, MT941, MT942, MT950,
    },
    sample::MessageConfig,
};

use crate::types::SampleGenerationOptions;

pub fn is_supported_message_type(message_type: &str) -> bool {
    matches!(
        message_type,
        "MT101"
            | "MT103"
            | "MT104"
            | "MT107"
            | "MT110"
            | "MT111"
            | "MT112"
            | "MT192"
            | "MT196"
            | "MT199"
            | "MT202"
            | "MT205"
            | "MT210"
            | "MT292"
            | "MT296"
            | "MT299"
            | "MT900"
            | "MT910"
            | "MT920"
            | "MT935"
            | "MT940"
            | "MT941"
            | "MT942"
            | "MT950"
    )
}

// Macro to generate sample for a specific MT message type
macro_rules! generate_mt_sample {
    ($mt_type:ty, $type_name:expr, $config:expr) => {
        if $config.is_object() && !$config.as_object().unwrap().is_empty() {
            // Try to parse config as MessageConfig and generate sample
            match serde_json::from_value::<MessageConfig>($config.clone()) {
                Ok(message_config) => {
                    tracing::info!(
                        "Successfully parsed MessageConfig for {}: include_optional={}, scenario={:?}, field_configs count={}",
                        $type_name,
                        message_config.include_optional,
                        message_config.scenario,
                        message_config.field_configs.len()
                    );

                    // Log amount field configurations specifically
                    for (field_name, field_config) in &message_config.field_configs {
                        if field_name.starts_with("32") || field_name.starts_with("33") || field_name.starts_with("71") {
                            tracing::info!(
                                "Amount field {} config: {:?}",
                                field_name, field_config
                            );
                        }
                    }

                    // Generate message with proper config support
                    let swift_message: SwiftMessage<$mt_type> =
                        SwiftMessage::sample_with_config(&message_config);
                    swift_message.to_mt_message()
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to parse MessageConfig for {}: {}. Config: {:?}",
                        $type_name, e, $config
                    );
                    // Return error instead of falling back to default
                    return Err(format!("Configuration parsing failed for {}: {}", $type_name, e).into());
                }
            }
        } else {
            // Generate default sample when no config provided
            tracing::debug!("No configuration provided for {}, using default sample", $type_name);
            let swift_message: SwiftMessage<$mt_type> = SwiftMessage::sample();
            swift_message.to_mt_message()
        }
    };
}

pub async fn generate_mt_from_config(
    config: &Value,
    message_type: &str,
    options: &SampleGenerationOptions,
) -> Result<String, Box<dyn std::error::Error>> {
    // Log the incoming configuration for debugging
    tracing::debug!(
        "Generating {} with config: {:?} and options: {:?}",
        message_type,
        config,
        options
    );

    // Validate the message if validation is enabled
    if options.validation {
        tracing::debug!("Validation enabled - message validated during parsing");
    }

    // Generate complete SwiftMessage with headers based on message type
    let mt_message = match message_type {
        "MT101" => generate_mt_sample!(MT101, "MT101", config),
        "MT103" => generate_mt_sample!(MT103, "MT103", config),
        "MT104" => generate_mt_sample!(MT104, "MT104", config),
        "MT107" => generate_mt_sample!(MT107, "MT107", config),
        "MT110" => generate_mt_sample!(MT110, "MT110", config),
        "MT111" => generate_mt_sample!(MT111, "MT111", config),
        "MT112" => generate_mt_sample!(MT112, "MT112", config),
        "MT192" => generate_mt_sample!(MT192, "MT192", config),
        "MT196" => generate_mt_sample!(MT196, "MT196", config),
        "MT199" => generate_mt_sample!(MT199, "MT199", config),
        "MT202" => generate_mt_sample!(MT202, "MT202", config),
        "MT205" => generate_mt_sample!(MT205, "MT205", config),
        "MT210" => generate_mt_sample!(MT210, "MT210", config),
        "MT292" => generate_mt_sample!(MT292, "MT292", config),
        "MT296" => generate_mt_sample!(MT296, "MT296", config),
        "MT299" => generate_mt_sample!(MT299, "MT299", config),
        "MT900" => generate_mt_sample!(MT900, "MT900", config),
        "MT910" => generate_mt_sample!(MT910, "MT910", config),
        "MT920" => generate_mt_sample!(MT920, "MT920", config),
        "MT935" => generate_mt_sample!(MT935, "MT935", config),
        "MT940" => generate_mt_sample!(MT940, "MT940", config),
        "MT941" => generate_mt_sample!(MT941, "MT941", config),
        "MT942" => generate_mt_sample!(MT942, "MT942", config),
        "MT950" => generate_mt_sample!(MT950, "MT950", config),
        _ => {
            return Err(format!("Message type {message_type} not yet implemented").into());
        }
    };

    // Apply any additional formatting if needed
    Ok(format_mt_message(&mt_message))
}

fn format_mt_message(mt_string: &str) -> String {
    // Apply consistent formatting to MT message
    mt_string
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<&str>>()
        .join("\n")
}
