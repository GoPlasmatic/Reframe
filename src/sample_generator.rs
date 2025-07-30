use serde_json::Value;
use swift_mt_message::{
    generate_sample,
    messages::{
        MT101, MT103, MT104, MT107, MT110, MT111, MT112, MT192, MT196, MT199, MT202, MT205, MT210,
        MT292, MT296, MT299, MT900, MT910, MT920, MT935, MT940, MT941, MT942, MT950,
    },
};
use tracing::{debug, error, info};

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

// Helper function to generate sample for a specific MT message type
fn generate_mt_sample_from_scenario<T>(
    message_type: &str,
    config: &Value,
) -> Result<String, Box<dyn std::error::Error>>
where
    T: swift_mt_message::traits::SwiftMessageBody + serde::de::DeserializeOwned,
{
    // Extract scenario name from config if provided
    let scenario_name = config.get("scenario").and_then(|s| s.as_str());

    // If config has custom fields, log warning as v3 uses scenario files
    if config.is_object() && !config.as_object().unwrap().is_empty() {
        if config.get("field_configs").is_some() || config.get("include_optional").is_some() {
            info!(
                "Note: swift-mt-message v3 uses scenario files. Custom field configs in request will be ignored. \n\
                 Using scenario: {:?} for {}",
                scenario_name.unwrap_or("standard"),
                message_type
            );
        }
    }

    debug!(
        "Generating {} sample with scenario: {:?}",
        message_type, scenario_name
    );

    // Generate sample using the v3 API
    match generate_sample::<T>(message_type, scenario_name) {
        Ok(swift_message) => Ok(swift_message.to_mt_message()),
        Err(e) => {
            error!(
                "Failed to generate {} sample with scenario {:?}: {}",
                message_type, scenario_name, e
            );
            Err(format!(
                "Sample generation failed for {} with scenario {:?}: {}",
                message_type, scenario_name, e
            )
            .into())
        }
    }
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
        "MT101" => generate_mt_sample_from_scenario::<MT101>(message_type, config)?,
        "MT103" => generate_mt_sample_from_scenario::<MT103>(message_type, config)?,
        "MT104" => generate_mt_sample_from_scenario::<MT104>(message_type, config)?,
        "MT107" => generate_mt_sample_from_scenario::<MT107>(message_type, config)?,
        "MT110" => generate_mt_sample_from_scenario::<MT110>(message_type, config)?,
        "MT111" => generate_mt_sample_from_scenario::<MT111>(message_type, config)?,
        "MT112" => generate_mt_sample_from_scenario::<MT112>(message_type, config)?,
        "MT192" => generate_mt_sample_from_scenario::<MT192>(message_type, config)?,
        "MT196" => generate_mt_sample_from_scenario::<MT196>(message_type, config)?,
        "MT199" => generate_mt_sample_from_scenario::<MT199>(message_type, config)?,
        "MT202" => generate_mt_sample_from_scenario::<MT202>(message_type, config)?,
        "MT205" => generate_mt_sample_from_scenario::<MT205>(message_type, config)?,
        "MT210" => generate_mt_sample_from_scenario::<MT210>(message_type, config)?,
        "MT292" => generate_mt_sample_from_scenario::<MT292>(message_type, config)?,
        "MT296" => generate_mt_sample_from_scenario::<MT296>(message_type, config)?,
        "MT299" => generate_mt_sample_from_scenario::<MT299>(message_type, config)?,
        "MT900" => generate_mt_sample_from_scenario::<MT900>(message_type, config)?,
        "MT910" => generate_mt_sample_from_scenario::<MT910>(message_type, config)?,
        "MT920" => generate_mt_sample_from_scenario::<MT920>(message_type, config)?,
        "MT935" => generate_mt_sample_from_scenario::<MT935>(message_type, config)?,
        "MT940" => generate_mt_sample_from_scenario::<MT940>(message_type, config)?,
        "MT941" => generate_mt_sample_from_scenario::<MT941>(message_type, config)?,
        "MT942" => generate_mt_sample_from_scenario::<MT942>(message_type, config)?,
        "MT950" => generate_mt_sample_from_scenario::<MT950>(message_type, config)?,
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
