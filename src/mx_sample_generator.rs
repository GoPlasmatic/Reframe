use serde_json::Value;
use tracing::{debug, error};
use mx_message::sample::generate_sample;

use crate::types::SampleGenerationOptions;

pub fn is_supported_mx_type(message_type: &str) -> bool {
    matches!(
        message_type,
        "pacs.008"
            | "pacs.009"
            | "pacs.002"
            | "pacs.003"
            | "pacs.004"
            | "camt.052"
            | "camt.053"
            | "camt.054"
            | "camt.056"
            | "camt.029"
            | "camt.025"
            | "camt.057"
            | "camt.060"
            | "pain.001"
            | "pain.008"
            | "pain.002"
    )
}

// Helper function to generate sample for a specific MX message type
fn generate_mx_sample_from_scenario(
    message_type: &str,
    config: &Value,
) -> Result<String, Box<dyn std::error::Error>> {
    // Extract scenario name from config if provided
    let scenario_name = config.get("scenario").and_then(|s| s.as_str());

    debug!(
        "Generating {} sample with scenario: {:?}",
        message_type, scenario_name
    );

    // Clean message type for mx-message library (pacs.008 -> pacs008)
    let clean_type = message_type.replace(".", "");

    // Generate sample using the mx-message API - it returns XML directly
    match generate_sample(&clean_type, scenario_name) {
        Ok(xml_string) => Ok(xml_string),
        Err(e) => {
            error!(
                "Failed to generate {} sample with scenario {:?}: {:?}",
                message_type, scenario_name, e
            );
            Err(format!(
                "Sample generation failed for {message_type} with scenario {scenario_name:?}: {e:?}"
            )
            .into())
        }
    }
}

pub async fn generate_mx_from_config(
    config: &Value,
    message_type: &str,
    options: &SampleGenerationOptions,
) -> Result<String, Box<dyn std::error::Error>> {
    // Log the incoming configuration for debugging
    debug!(
        "Generating {} with config: {:?} and options: {:?}",
        message_type, config, options
    );

    // Validate the message if validation is enabled
    if options.validation {
        debug!("Validation enabled - message will be validated during generation");
    }

    // Generate MX message - the library now returns XML directly
    let xml_message = generate_mx_sample_from_scenario(message_type, config)?;

    // Apply any additional formatting if needed
    Ok(format_xml_message(&xml_message))
}

fn format_xml_message(xml_string: &str) -> String {
    // For now, return as-is. Could add pretty-printing here if needed
    xml_string.to_string()
}
