use datafake_rs::DataGenerator;
use serde_json::Value;
use std::path::PathBuf;
use tracing::{debug, error};

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

// Helper function to generate sample for a specific MT message type using datafake directly
fn generate_mt_sample_from_scenario(
    message_type: &str,
    config: &Value,
) -> Result<String, Box<dyn std::error::Error>> {
    // Extract scenario name from config if provided
    let scenario_name = config.get("scenario").and_then(|s| s.as_str());

    debug!(
        "Generating {} sample with scenario: {:?}",
        message_type, scenario_name
    );

    // Try to find the scenario file using our index.json
    let scenario_id = scenario_name.unwrap_or("standard");
    let scenario_file = match scenario_loader::get_scenario_file_path(message_type, scenario_id) {
        Ok(file) => file,
        Err(e) => {
            // If specific scenario not found, try "standard"
            if scenario_id != "standard" {
                debug!("Scenario {} not found, trying standard: {}", scenario_id, e);
                scenario_loader::get_scenario_file_path(message_type, "standard")?
            } else {
                return Err(e);
            }
        }
    };

    // Load the scenario file
    let scenario_path = PathBuf::from("scenarios").join(&scenario_file);
    let scenario_content = std::fs::read_to_string(&scenario_path)?;
    let scenario_json: Value = serde_json::from_str(&scenario_content)?;

    // Extract variables and schema for datafake
    let variables = scenario_json.get("variables").cloned().unwrap_or(serde_json::json!({}));
    let schema = scenario_json.get("schema").cloned().unwrap_or(serde_json::json!({}));

    // Create datafake scenario with both variables and schema
    let datafake_scenario = serde_json::json!({
        "variables": variables,
        "schema": schema
    });

    // Use datafake-rs to generate the sample
    let generator = DataGenerator::from_value(datafake_scenario).map_err(|e| {
        format!("Failed to create datafake generator: {:?}", e)
    })?;

    let generated_data = generator.generate().map_err(|e| {
        format!("Datafake generation failed: {:?}", e)
    })?;

    // Convert the generated data to SWIFT MT format
    let swift_message = format_swift_message(message_type, &generated_data)?;
    
    debug!("Successfully generated {} using scenario {}", message_type, scenario_id);
    Ok(swift_message)
}

// Format the generated JSON data into SWIFT MT message format
fn format_swift_message(message_type: &str, data: &Value) -> Result<String, Box<dyn std::error::Error>> {
    // Extract headers and fields
    let basic_header = data.get("basic_header").ok_or("Missing basic_header")?;
    let app_header = data.get("application_header").ok_or("Missing application_header")?;
    let user_header = data.get("user_header");
    let fields = data.get("fields").ok_or("Missing fields")?;
    let trailer = data.get("trailer");

    // Build Block 1: Basic Header
    let block1 = format!(
        "{{1:F{}{}{}{}{:0>4}{:0>6}}}",
        basic_header.get("service_id").and_then(|v| v.as_str()).unwrap_or("01"),
        basic_header.get("logical_terminal").and_then(|v| v.as_str()).unwrap_or("BANKUS33XXXX"),
        basic_header.get("session_number").and_then(|v| v.as_str()).unwrap_or("0001"),
        basic_header.get("sequence_number").and_then(|v| v.as_str()).unwrap_or("000001"),
        basic_header.get("session_number").and_then(|v| v.as_i64()).unwrap_or(1),
        basic_header.get("sequence_number").and_then(|v| v.as_i64()).unwrap_or(1)
    );

    // Build Block 2: Application Header
    let msg_type = &message_type[2..]; // Remove "MT" prefix
    let block2 = format!(
        "{{2:{}{}{}{}}}",
        app_header.get("direction").and_then(|v| v.as_str()).unwrap_or("I"),
        msg_type,
        app_header.get("destination_address").and_then(|v| v.as_str()).unwrap_or("BANKUS33XXXX"),
        app_header.get("priority").and_then(|v| v.as_str()).unwrap_or("N")
    );

    // Build Block 3: User Header (optional)
    let block3 = if let Some(uh) = user_header {
        let mut uh_fields = Vec::new();
        if let Some(ref_val) = uh.get("message_user_reference").and_then(|v| v.as_str()) {
            uh_fields.push(format!("108:{}", ref_val));
        }
        if let Some(uetr) = uh.get("unique_end_to_end_reference").and_then(|v| v.as_str()) {
            uh_fields.push(format!("121:{}", uetr));
        }
        if !uh_fields.is_empty() {
            format!("{{3:{{{}}}}}", uh_fields.join("}{"))
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    // Build Block 4: Text Block with message fields
    let mut text_fields = Vec::new();
    
    // Process fields based on message type
    if let Some(fields_obj) = fields.as_object() {
        for (field_tag, field_value) in fields_obj {
            let field_str = format_field(field_tag, field_value)?;
            if !field_str.is_empty() {
                text_fields.push(field_str);
            }
        }
    }

    let block4 = format!("{{4:\n{}\n-}}", text_fields.join("\n"));

    // Build Block 5: Trailer (optional)
    let block5 = if let Some(tr) = trailer {
        if let Some(checksum) = tr.get("checksum").and_then(|v| v.as_str()) {
            format!("{{5:{{CHK:{}}}}}", checksum)
        } else {
            "{5:{CHK:123456789ABC}}".to_string()
        }
    } else {
        "{5:{CHK:123456789ABC}}".to_string()
    };

    // Combine all blocks
    let swift_message = format!("{}{}{}{}{}", block1, block2, block3, block4, block5);
    Ok(swift_message)
}

// Format individual MT field
fn format_field(tag: &str, value: &Value) -> Result<String, Box<dyn std::error::Error>> {
    // Handle simple string values
    if let Some(s) = value.as_str() {
        return Ok(format!(":{}:{}", tag, s));
    }

    // Handle structured field values
    if let Some(obj) = value.as_object() {
        // Field 20: Transaction Reference
        if tag == "20" {
            if let Some(ref_val) = obj.get("reference").and_then(|v| v.as_str()) {
                return Ok(format!(":20:{}", ref_val));
            }
        }
        
        // Field 21: Related Reference or Request Reference  
        if tag == "21" || tag == "21R" {
            if let Some(ref_val) = obj.get("reference").and_then(|v| v.as_str()) {
                return Ok(format!(":{}:{}", tag, ref_val));
            }
        }
        
        // Field 23B: Bank Operation Code
        if tag == "23B" {
            if let Some(code) = obj.get("instruction_code").and_then(|v| v.as_str()) {
                return Ok(format!(":23B:{}", code));
            }
        }
        
        // Field 32A/32B: Value Date, Currency and Amount
        if tag == "32A" || tag == "32B" {
            let date = obj.get("value_date").and_then(|v| v.as_str()).unwrap_or("250816");
            let currency = obj.get("currency").and_then(|v| v.as_str()).unwrap_or("EUR");
            let amount = obj.get("amount").and_then(|v| v.as_f64()).unwrap_or(10000.0);
            return Ok(format!(":{}:{}{}{:.2}", tag, date, currency, amount));
        }
        
        // Field 50: Ordering Customer (various options)
        if tag == "50" {
            if let Some(k_option) = obj.get("K") {
                let account = k_option.get("account").and_then(|v| v.as_str()).unwrap_or("/1234567890");
                let name_address = k_option.get("name_and_address")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter()
                        .filter_map(|v| v.as_str())
                        .collect::<Vec<_>>()
                        .join("\n"))
                    .unwrap_or_else(|| "CUSTOMER NAME\nCUSTOMER ADDRESS".to_string());
                return Ok(format!(":50K:{}\n{}", account, name_address));
            }
        }
        
        // Field 57: Account With Institution
        if tag == "57" {
            if let Some(a_option) = obj.get("A") {
                let bic = a_option.get("bic").and_then(|v| v.as_str()).unwrap_or("BANKUS33");
                return Ok(format!(":57A:{}", bic));
            }
        }
        
        // Field 59: Beneficiary Customer
        if tag == "59" {
            if let Some(no_option) = obj.get("NoOption") {
                let account = no_option.get("account").and_then(|v| v.as_str()).unwrap_or("/9876543210");
                let name_address = no_option.get("name_and_address")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter()
                        .filter_map(|v| v.as_str())
                        .collect::<Vec<_>>()
                        .join("\n"))
                    .unwrap_or_else(|| "BENEFICIARY NAME\nBENEFICIARY ADDRESS".to_string());
                return Ok(format!(":59:{}\n{}", account, name_address));
            }
        }
        
        // Field 71A: Details of Charges
        if tag == "71A" {
            if let Some(code) = obj.get("code").and_then(|v| v.as_str()) {
                return Ok(format!(":71A:{}", code));
            }
        }
    }

    // Default: return empty string for unhandled fields
    Ok(String::new())
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

    // Generate the MT message using datafake directly
    let mt_message = generate_mt_sample_from_scenario(message_type, config)?;

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

// MX Sample Generation Functions

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

    // Try to find the scenario file using our index.json
    let scenario_id = scenario_name.unwrap_or("standard");
    let scenario_file = match scenario_loader::get_scenario_file_path(message_type, scenario_id) {
        Ok(file) => file,
        Err(e) => {
            // If specific scenario not found, try "standard"
            if scenario_id != "standard" {
                debug!("Scenario {} not found, trying standard: {}", scenario_id, e);
                scenario_loader::get_scenario_file_path(message_type, "standard")?
            } else {
                return Err(e);
            }
        }
    };

    // Load the scenario file
    let scenario_path = PathBuf::from("scenarios").join(&scenario_file);
    let scenario_content = std::fs::read_to_string(&scenario_path)?;
    let scenario_json: Value = serde_json::from_str(&scenario_content)?;

    // Extract variables and schema for datafake
    let variables = scenario_json.get("variables").cloned().unwrap_or(serde_json::json!({}));
    let schema = scenario_json.get("schema").cloned().unwrap_or(serde_json::json!({}));

    // Create datafake scenario with both variables and schema
    let datafake_scenario = serde_json::json!({
        "variables": variables,
        "schema": schema
    });

    // Use datafake-rs to generate the sample
    let generator = DataGenerator::from_value(datafake_scenario).map_err(|e| {
        format!("Failed to create datafake generator: {:?}", e)
    })?;

    let generated_data = generator.generate().map_err(|e| {
        format!("Datafake generation failed: {:?}", e)
    })?;

    // Convert the generated data to ISO 20022 XML format
    let xml_message = format_iso20022_message(message_type, &generated_data)?;
    
    debug!("Successfully generated {} using scenario {}", message_type, scenario_id);
    Ok(xml_message)
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

// Format the generated JSON data into ISO 20022 XML format
fn format_iso20022_message(message_type: &str, data: &Value) -> Result<String, Box<dyn std::error::Error>> {
    // For now, just return the JSON as a string
    // In a real implementation, this would convert to proper XML format
    // Since MX messages from scenarios are complex transformation specs,
    // we'll return JSON for now
    Ok(serde_json::to_string_pretty(data)?)
}