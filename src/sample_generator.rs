use datafake_rs::DataGenerator;
use serde_json::Value;
use std::path::PathBuf;
use tracing::debug;

use crate::mt_generator;
use crate::mx_generator;
use crate::scenario_loader;
use crate::types::SampleGenerationOptions;

pub fn is_supported_message_type(message_type: &str) -> bool {
    is_supported_mt_type(message_type) || is_supported_mx_type(message_type)
}

pub fn is_supported_mt_type(message_type: &str) -> bool {
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
            | "camt.107"
            | "camt.108"
            | "camt.109"
            | "pain.001"
            | "pain.008"
            | "pain.002"
    )
}

fn generate_mt_sample_from_scenario(
    message_type: &str,
    config: &Value,
) -> Result<(String, Value), Box<dyn std::error::Error>> {
    let scenario_name = config.get("scenario").and_then(|s| s.as_str());

    debug!(
        "Generating {} sample with scenario: {:?}",
        message_type, scenario_name
    );

    let scenario_id = scenario_name.unwrap_or("standard");
    let scenario_file = match scenario_loader::get_scenario_file_path(message_type, scenario_id) {
        Ok(file) => file,
        Err(e) => {
            if scenario_id != "standard" {
                debug!("Scenario {} not found, trying standard: {}", scenario_id, e);
                scenario_loader::get_scenario_file_path(message_type, "standard")?
            } else {
                return Err(e);
            }
        }
    };

    let scenario_path = PathBuf::from("scenarios").join(&scenario_file);
    let scenario_content = std::fs::read_to_string(&scenario_path)?;
    
    // Pass the entire scenario JSON directly to DataGenerator (following basic.rs example)
    // DataGenerator will handle variables and schema internally
    let generator = DataGenerator::from_json(&scenario_content).map_err(|e| {
        format!("Failed to create datafake generator: {:?}", e)
    })?;

    let generated_data = generator.generate().map_err(|e| {
        format!("Datafake generation failed: {:?}", e)
    })?;

    // Clone the generated data for returning (before any modifications)
    let generated_json = generated_data.clone();
    
    // Debug: Log the generated data
    debug!("Generated data from datafake: {}", serde_json::to_string_pretty(&generated_data)?);
    
    // Parse the generated JSON into a proper SwiftMessage object and serialize to MT format
    match mt_generator::generate_mt_from_json(message_type, &generated_data) {
        Ok(swift_message) => {
            debug!("Successfully generated {} using scenario {}", message_type, scenario_id);
            Ok((swift_message, generated_json))
        }
        Err(e) => {
            // Return the error but include the generated JSON for debugging
            let error_msg = format!(
                "Failed to parse {} from generated JSON: {}. Generated JSON: {}",
                message_type,
                e,
                serde_json::to_string_pretty(&generated_json).unwrap_or_else(|_| "Invalid JSON".to_string())
            );
            Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, error_msg)))
        }
    }
}

pub async fn generate_mt_from_config(
    config: &Value,
    message_type: &str,
    options: &SampleGenerationOptions,
) -> Result<(String, Value), Box<dyn std::error::Error>> {
    debug!(
        "Generating {} with config: {:?} and options: {:?}",
        message_type,
        config,
        options
    );

    if options.validation {
        debug!("Validation enabled - message validated during parsing");
    }

    let (mt_message, generated_json) = generate_mt_sample_from_scenario(message_type, config)?;
    Ok((mt_generator::format_mt_message(&mt_message), generated_json))
}

fn generate_mx_sample_from_scenario(
    message_type: &str,
    config: &Value,
) -> Result<(String, Value), Box<dyn std::error::Error>> {
    let scenario_name = config.get("scenario").and_then(|s| s.as_str());

    debug!(
        "Generating {} sample with scenario: {:?}",
        message_type, scenario_name
    );

    let scenario_id = scenario_name.unwrap_or("standard");
    let scenario_file = match scenario_loader::get_scenario_file_path(message_type, scenario_id) {
        Ok(file) => file,
        Err(e) => {
            if scenario_id != "standard" {
                debug!("Scenario {} not found, trying standard: {}", scenario_id, e);
                scenario_loader::get_scenario_file_path(message_type, "standard")?
            } else {
                return Err(e);
            }
        }
    };

    let scenario_path = PathBuf::from("scenarios").join(&scenario_file);
    let scenario_content = std::fs::read_to_string(&scenario_path)?;
    
    // Pass the entire scenario JSON directly to DataGenerator (following basic.rs example)
    // DataGenerator will handle variables and schema internally
    let generator = DataGenerator::from_json(&scenario_content).map_err(|e| {
        format!("Failed to create datafake generator: {:?}", e)
    })?;

    let generated_data = generator.generate().map_err(|e| {
        format!("Datafake generation failed: {:?}", e)
    })?;

    // Clone the generated data for returning
    let generated_json = generated_data.clone();
    
    // Debug: Log the generated data
    debug!("Generated data from datafake: {}", serde_json::to_string_pretty(&generated_data)?);
    
    // Parse the generated JSON into proper MX XML format
    match mx_generator::generate_mx_from_json(message_type, &generated_data) {
        Ok(xml_message) => {
            debug!("Successfully generated {} XML using scenario {}", message_type, scenario_id);
            Ok((xml_message, generated_json))
        }
        Err(e) => {
            // Return the error but include the generated JSON for debugging
            let error_msg = format!(
                "Failed to generate {} XML from JSON: {}. Generated JSON: {}",
                message_type,
                e,
                serde_json::to_string_pretty(&generated_json).unwrap_or_else(|_| "Invalid JSON".to_string())
            );
            Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, error_msg)))
        }
    }
}

pub async fn generate_mx_from_config(
    config: &Value,
    message_type: &str,
    options: &SampleGenerationOptions,
) -> Result<(String, Value), Box<dyn std::error::Error>> {
    debug!(
        "Generating {} with config: {:?} and options: {:?}",
        message_type, config, options
    );

    if options.validation {
        debug!("Validation enabled - message will be validated during generation");
    }

    let (xml_message, generated_json) = generate_mx_sample_from_scenario(message_type, config)?;
    Ok((xml_message, generated_json))
}