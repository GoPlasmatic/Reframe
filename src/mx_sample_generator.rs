use mx_message::{
    document::{
        camt_025_001_08::ReceiptV08,
        camt_029_001_09::ResolutionOfInvestigationV09,
        camt_052_001_08::BankToCustomerAccountReportV08,
        camt_053_001_08::BankToCustomerStatementV08,
        camt_054_001_08::BankToCustomerDebitCreditNotificationV08,
        camt_056_001_08::FIToFIPaymentCancellationRequestV08,
        camt_057_001_06::NotificationToReceiveV06,
        camt_060_001_05::AccountReportingRequestV05,
        pacs_002_001_10::FIToFIPaymentStatusReportV10,
        pacs_003_001_08::FIToFICustomerDirectDebitV08,
        pacs_004_001_09::PaymentReturnV09,
        pacs_008_001_08::FIToFICustomerCreditTransferV08,
        pacs_009_001_08::FinancialInstitutionCreditTransferV08,
        pain_001_001_09::CustomerCreditTransferInitiationV09,
        pain_002_001_10::CustomerPaymentStatusReportV10,
        pain_008_001_08::CustomerDirectDebitInitiationV08,
    },
};
use serde::Serialize;
use serde_json::Value;
use tracing::{debug, error};

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

// Helper function to serialize MX message to XML
fn serialize_to_xml<T: Serialize>(msg: T) -> Result<String, Box<dyn std::error::Error>> {
    // For now, return JSON representation as we need proper XML serialization setup
    // TODO: Implement proper ISO 20022 XML serialization with namespaces
    let json_value = serde_json::to_value(&msg)?;
    let json_string = serde_json::to_string_pretty(&json_value)?;
    
    Ok(json_string)
}

// Helper function to generate sample for a specific MX message type
fn generate_mx_sample_from_scenario<T>(
    message_type: &str,
    config: &Value,
) -> Result<String, Box<dyn std::error::Error>>
where
    T: serde::de::DeserializeOwned + Serialize,
{
    // Extract scenario name from config if provided
    let scenario_name = config.get("scenario").and_then(|s| s.as_str());
    
    debug!(
        "Generating {} sample with scenario: {:?}",
        message_type, scenario_name
    );
    
    // Clean message type for mx-message library (pacs.008 -> pacs008)
    let clean_type = message_type.replace(".", "");
    
    // Generate sample using the mx-message API
    match mx_message::sample::generate_sample::<T>(&clean_type, scenario_name) {
        Ok(mx_message) => serialize_to_xml(mx_message),
        Err(e) => {
            error!(
                "Failed to generate {} sample with scenario {:?}: {:?}",
                message_type, scenario_name, e
            );
            Err(format!(
                "Sample generation failed for {} with scenario {:?}: {:?}",
                message_type, scenario_name, e
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
    
    // Generate MX message based on message type
    let xml_message = match message_type {
        "pacs.008" => generate_mx_sample_from_scenario::<FIToFICustomerCreditTransferV08>(message_type, config)?,
        "pacs.009" => generate_mx_sample_from_scenario::<FinancialInstitutionCreditTransferV08>(message_type, config)?,
        "pacs.002" => generate_mx_sample_from_scenario::<FIToFIPaymentStatusReportV10>(message_type, config)?,
        "pacs.003" => generate_mx_sample_from_scenario::<FIToFICustomerDirectDebitV08>(message_type, config)?,
        "pacs.004" => generate_mx_sample_from_scenario::<PaymentReturnV09>(message_type, config)?,
        "camt.025" => generate_mx_sample_from_scenario::<ReceiptV08>(message_type, config)?,
        "camt.029" => generate_mx_sample_from_scenario::<ResolutionOfInvestigationV09>(message_type, config)?,
        "camt.052" => generate_mx_sample_from_scenario::<BankToCustomerAccountReportV08>(message_type, config)?,
        "camt.053" => generate_mx_sample_from_scenario::<BankToCustomerStatementV08>(message_type, config)?,
        "camt.054" => generate_mx_sample_from_scenario::<BankToCustomerDebitCreditNotificationV08>(message_type, config)?,
        "camt.056" => generate_mx_sample_from_scenario::<FIToFIPaymentCancellationRequestV08>(message_type, config)?,
        "camt.057" => generate_mx_sample_from_scenario::<NotificationToReceiveV06>(message_type, config)?,
        "camt.060" => generate_mx_sample_from_scenario::<AccountReportingRequestV05>(message_type, config)?,
        "pain.001" => generate_mx_sample_from_scenario::<CustomerCreditTransferInitiationV09>(message_type, config)?,
        "pain.002" => generate_mx_sample_from_scenario::<CustomerPaymentStatusReportV10>(message_type, config)?,
        "pain.008" => generate_mx_sample_from_scenario::<CustomerDirectDebitInitiationV08>(message_type, config)?,
        _ => {
            return Err(format!("Message type {} not yet implemented", message_type).into());
        }
    };
    
    // Apply any additional formatting if needed
    Ok(format_xml_message(&xml_message))
}

fn format_xml_message(xml_string: &str) -> String {
    // For now, return as-is. Could add pretty-printing here if needed
    xml_string.to_string()
}