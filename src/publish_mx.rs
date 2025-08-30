use async_trait::async_trait;
use dataflow_rs::engine::error::DataflowError;
use dataflow_rs::engine::{
    AsyncFunctionHandler,
    error::Result,
    message::{Change, Message},
};
use mx_message::document::*;
use mx_message::header::*;
use quick_xml::se::to_string as xml_to_string;
use serde_json::Value;
use tracing::{debug, error, instrument, trace, warn};

pub struct PublishMX;

#[async_trait]
impl AsyncFunctionHandler for PublishMX {
    #[instrument(skip(self, message, input))]
    async fn execute(&self, message: &mut Message, input: &Value) -> Result<(usize, Vec<Change>)> {
        debug!("Starting MT to MX message publishing/conversion");

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

        let input_data = message.data.get(input_field_name).ok_or_else(|| {
            error!(
                input_field = input_field_name,
                available_fields = ?message.data.as_object().map(|obj| obj.keys().collect::<Vec<_>>()),
                "Input field not found in message data for MT to MX transformation"
            );
            DataflowError::Validation(format!(
                "Field {} not found in message data {}",
                input_field_name, message.data
            ))
        })?;

        let message_metadata = message.metadata.clone();
        let message_type = message_metadata
            .get("SwiftMT")
            .and_then(|v| v.get("message_type"))
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");
        let message_method = message_metadata
            .get("SwiftMT")
            .and_then(|v| v.get("method"))
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");
        debug!(
            message_type = %message_type,
            message_method = %message_method,
            "Processing MT message"
        );

        let result = if source_format.ends_with(".Header") {
            debug!(source_format = %source_format, "Processing MT to MX header");
            handle_mt_to_mx_header(
                input_data.clone(),
                message,
                output_field_name,
                message_type,
                message_method,
            )
        } else if source_format.ends_with(".Document") {
            debug!(source_format = %source_format, "Processing MT to MX document");
            handle_mt_to_mx_document(
                input_data.clone(),
                message,
                output_field_name,
                message_type,
                message_method,
            )
        } else {
            error!(source_format = %source_format, "Invalid source format for MT to MX - must end with .Header or .Document");
            Err(DataflowError::Validation(format!(
                "Invalid source format for MT to MX: {source_format}"
            )))
        };

        match &result {
            Ok((status, _)) => {
                debug!(
                    source_format = %source_format,
                    output_field = output_field_name,
                    status = status,
                    "MT to MX message publishing completed successfully"
                );
            }
            Err(e) => {
                error!(
                    source_format = %source_format,
                    error = %e,
                    "MT to MX message publishing failed"
                );
            }
        }

        result
    }
}

/// Helper function to route MT to MX header based on message type and method
fn route_header_by_type(data: Value, message_type: &str, message_method: &str) -> Result<String> {
    // Handle payment messages with method-based routing
    match (message_type, message_method) {
        // MT103 variants
        ("103", "reject") => {
            debug!("Serializing MT103 REJT to pacs.002 header");
            serialize_mt_to_mx_header::<bah_pacs_002_001_10::BusinessApplicationHeaderV02>(data)
        }
        ("103", "return") => {
            debug!("Serializing MT103 RETN to pacs.004 header");
            serialize_mt_to_mx_header::<bah_pacs_004_001_09::BusinessApplicationHeaderV02>(data)
        }
        ("103", "stp") => {
            debug!("Serializing MT103 STP to pacs.008 STP header");
            serialize_mt_to_mx_header::<bah_pacs_008_001_08_stp::BusinessApplicationHeaderV02>(data)
        }
        ("103", _) => {
            debug!("Serializing MT103 to pacs.008 header");
            serialize_mt_to_mx_header::<bah_pacs_008_001_08::BusinessApplicationHeaderV02>(data)
        }

        // MT202 variants
        ("202", "reject") => {
            debug!("Serializing MT202 REJT to pacs.002 header");
            serialize_mt_to_mx_header::<bah_pacs_002_001_10::BusinessApplicationHeaderV02>(data)
        }
        ("202", "return") => {
            debug!("Serializing MT202 RETN to pacs.004 header");
            serialize_mt_to_mx_header::<bah_pacs_004_001_09::BusinessApplicationHeaderV02>(data)
        }
        ("202", "cover") => {
            debug!("Serializing MT202 COV to pacs.009 COV header");
            serialize_mt_to_mx_header::<bah_pacs_009_001_08_cov::BusinessApplicationHeaderV02>(data)
        }
        ("202", _) => {
            debug!("Serializing MT202 to pacs.009 header");
            serialize_mt_to_mx_header::<bah_pacs_009_001_08::BusinessApplicationHeaderV02>(data)
        }

        // MT205 variants
        ("205", "reject") => {
            debug!("Serializing MT205 REJT to pacs.002 header");
            serialize_mt_to_mx_header::<bah_pacs_002_001_10::BusinessApplicationHeaderV02>(data)
        }
        ("205", "return") => {
            debug!("Serializing MT205 RETN to pacs.004 header");
            serialize_mt_to_mx_header::<bah_pacs_004_001_09::BusinessApplicationHeaderV02>(data)
        }
        ("205", "cover") => {
            debug!("Serializing MT205 COV to pacs.009 COV header");
            serialize_mt_to_mx_header::<bah_pacs_009_001_08_cov::BusinessApplicationHeaderV02>(data)
        }
        ("205", _) => {
            debug!("Serializing MT205 to pacs.009 header");
            serialize_mt_to_mx_header::<bah_pacs_009_001_08::BusinessApplicationHeaderV02>(data)
        }

        // Cash management messages
        ("900" | "910", _) => {
            debug!("Serializing MT900/910 to camt.054 header");
            serialize_mt_to_mx_header::<bah_camt_054_001::BusinessApplicationHeaderV02>(data)
        }
        ("192" | "292", _) => {
            debug!("Serializing MT192/292 to camt.056 header");
            serialize_mt_to_mx_header::<bah_camt_056_001_08::BusinessApplicationHeaderV02>(data)
        }
        ("196" | "296", _) => {
            debug!("Serializing MT196/296 to camt.029 header");
            serialize_mt_to_mx_header::<bah_camt_029_001::BusinessApplicationHeaderV02>(data)
        }

        // Other message types
        ("101", _) => {
            debug!("Serializing MT101 to pain.001 header");
            serialize_mt_to_mx_header::<bah_pain_001_001_09::BusinessApplicationHeaderV02>(data)
        }
        ("200", _) => {
            debug!("Serializing MT200 to pacs.009 header");
            serialize_mt_to_mx_header::<bah_pacs_009_001_08::BusinessApplicationHeaderV02>(data)
        }

        _ => {
            error!(message_type = %message_type, "Unsupported MT type for header");
            Err(DataflowError::Validation(format!(
                "Unsupported MT type: {}",
                message_type
            )))
        }
    }
}

/// Handles MT to MX header transformation based on message type and method
fn handle_mt_to_mx_header(
    data: Value,
    message: &mut Message,
    output_field_name: &str,
    message_type: &str,
    message_method: &str,
) -> Result<(usize, Vec<Change>)> {
    let xml_string = route_header_by_type(data, message_type, message_method)?;

    debug!("MT to MX header serialization completed");

    let result_value = Value::String(xml_string);
    message.data[output_field_name] = result_value.clone();

    Ok((
        200,
        vec![Change {
            path: format!("data.{}", output_field_name),
            old_value: Value::Null,
            new_value: result_value,
        }],
    ))
}

/// Helper function to extract required field from data with error handling
fn extract_field(data: &Value, field_name: &str, message_context: &str) -> Result<Value> {
    data.get(field_name)
        .ok_or_else(|| {
            DataflowError::Validation(format!(
                "Missing {} field for {}",
                field_name, message_context
            ))
        })
        .cloned()
}

/// Helper function to route MT to MX document based on message type and method
fn route_document_by_type(data: Value, message_type: &str, message_method: &str) -> Result<String> {
    match (message_type, message_method) {
        // MT103 variants
        ("103", "reject") => {
            let field_data = extract_field(&data, "FIToFIPmtStsRpt", "MT103 reject")?;
            serialize_mt_to_mx_document::<pacs_002_001_10::FIToFIPaymentStatusReportV10>(field_data)
        }
        ("103", "return") => {
            let field_data = extract_field(&data, "PmtRtr", "MT103 return")?;
            serialize_mt_to_mx_document::<pacs_004_001_09::PaymentReturnV09>(field_data)
        }
        ("103", "stp") => {
            let field_data = extract_field(&data, "FIToFICstmrCdtTrf", "MT103 STP")?;
            serialize_mt_to_mx_document::<pacs_008_001_08_stp::FIToFICustomerCreditTransferV08>(
                field_data,
            )
        }
        ("103", _) => {
            let field_data = extract_field(&data, "FIToFICstmrCdtTrf", "MT103")?;
            serialize_mt_to_mx_document::<pacs_008_001_08::FIToFICustomerCreditTransferV08>(
                field_data,
            )
        }

        // MT202 variants
        ("202", "reject") => {
            let field_data = extract_field(&data, "FIToFIPmtStsRpt", "MT202 reject")?;
            serialize_mt_to_mx_document::<pacs_002_001_10::FIToFIPaymentStatusReportV10>(field_data)
        }
        ("202", "return") => {
            let field_data = extract_field(&data, "PmtRtr", "MT202 return")?;
            serialize_mt_to_mx_document::<pacs_004_001_09::PaymentReturnV09>(field_data)
        }
        ("202", "cover") => {
            let field_data = extract_field(&data, "FIToFICdtTrf", "MT202 cover")?;
            serialize_mt_to_mx_document::<pacs_009_001_08_cov::FinancialInstitutionCreditTransferV08>(
                field_data,
            )
        }
        ("202", _) => {
            let field_data = extract_field(&data, "FIToFICdtTrf", "MT202")?;
            serialize_mt_to_mx_document::<pacs_009_001_08::FinancialInstitutionCreditTransferV08>(
                field_data,
            )
        }

        // MT205 variants
        ("205", "reject") => {
            let field_data = extract_field(&data, "FIToFIPmtStsRpt", "MT205 reject")?;
            serialize_mt_to_mx_document::<pacs_002_001_10::FIToFIPaymentStatusReportV10>(field_data)
        }
        ("205", "return") => {
            let field_data = extract_field(&data, "PmtRtr", "MT205 return")?;
            serialize_mt_to_mx_document::<pacs_004_001_09::PaymentReturnV09>(field_data)
        }
        ("205", "cover") => {
            let field_data = extract_field(&data, "FIToFICdtTrf", "MT205 cover")?;
            serialize_mt_to_mx_document::<pacs_009_001_08_cov::FinancialInstitutionCreditTransferV08>(
                field_data,
            )
        }
        ("205", _) => {
            let field_data = extract_field(&data, "FIToFICdtTrf", "MT205")?;
            serialize_mt_to_mx_document::<pacs_009_001_08::FinancialInstitutionCreditTransferV08>(
                field_data,
            )
        }

        // Cash management messages
        ("900" | "910", _) => {
            let field_data = extract_field(&data, "BkToCstmrDbtCdtNtfctn", "MT900/910")?;
            serialize_mt_to_mx_document::<camt_054_001_08::BankToCustomerDebitCreditNotificationV08>(
                field_data,
            )
        }
        ("192" | "292", _) => {
            let field_data = extract_field(&data, "FIToFIPmtCxlReq", "MT192/292")?;
            serialize_mt_to_mx_document::<camt_056_001_08::FIToFIPaymentCancellationRequestV08>(
                field_data,
            )
        }
        ("196" | "296", _) => {
            let field_data = extract_field(&data, "RsltnOfInvstgtn", "MT196/296")?;
            serialize_mt_to_mx_document::<camt_029_001_09::ResolutionOfInvestigationV09>(field_data)
        }

        // Other message types
        ("101", _) => {
            debug!("Serializing MT101 to pain.001 document");
            // Special handling for MT101 - check both direct field and nested structure
            let field_data = data
                .get("CstmrCdtTrfInitn")
                .or_else(|| data.as_object()?.get("CstmrCdtTrfInitn"))
                .ok_or_else(|| {
                    DataflowError::Validation(
                        "Missing CstmrCdtTrfInitn field for MT101".to_string(),
                    )
                })?
                .clone();
            serialize_mt_to_mx_document::<pain_001_001_09::CustomerCreditTransferInitiationV09>(
                field_data,
            )
        }
        ("200", _) => {
            debug!("Processing MT200 to pacs.009 document");
            let field_data = extract_field(&data, "FIToFICdtTrf", "MT200")?;
            serialize_mt_to_mx_document::<pacs_009_001_08::FinancialInstitutionCreditTransferV08>(
                field_data,
            )
        }

        _ => {
            error!(message_type = %message_type, "Unsupported MT type for document");
            Err(DataflowError::Validation(format!(
                "Unsupported MT type: {}",
                message_type
            )))
        }
    }
}

/// Handles MT to MX document transformation based on message type and method
fn handle_mt_to_mx_document(
    data: Value,
    message: &mut Message,
    output_field_name: &str,
    message_type: &str,
    message_method: &str,
) -> Result<(usize, Vec<Change>)> {
    let xml_string = route_document_by_type(data, message_type, message_method)?;

    debug!("MT to MX document serialization completed");

    let result_value = Value::String(xml_string);
    message.data[output_field_name] = result_value.clone();

    Ok((
        200,
        vec![Change {
            path: format!("data.{}", output_field_name),
            old_value: Value::Null,
            new_value: result_value,
        }],
    ))
}

// Generic MT to MX header serialization helper
#[instrument(skip(data))]
fn serialize_mt_to_mx_header<T>(data: Value) -> Result<String>
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    debug!("Deserializing MT to MX header data");

    // Use serde_path_to_error for detailed path information
    let json_str = data.to_string();
    let mut deserializer = serde_json::Deserializer::from_str(&json_str);
    match serde_path_to_error::deserialize::<_, T>(&mut deserializer) {
        Ok(header_data) => {
            debug!("MT to MX header deserialization successful, converting to XML");
            xml_to_string(&header_data).map_err(|e| {
                let error_msg = format!("MT to MX header XML serialization failed: {e}");
                error!(error = %error_msg, "MT to MX header XML serialization failed");
                DataflowError::Validation(error_msg)
            })
        }
        Err(e) => {
            let error_msg = format!(
                "MT to MX AppHdr JSON deserialization failed at path '{}': {}",
                e.path(),
                e.inner()
            );
            error!(error = %error_msg, "MT to MX header JSON deserialization failed");
            Err(DataflowError::Validation(error_msg))
        }
    }
}

// Generic MT to MX document serialization helper
#[instrument(skip(data))]
fn serialize_mt_to_mx_document<T>(data: Value) -> Result<String>
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    debug!("Deserializing MT to MX document data");

    // Use serde_path_to_error for detailed path information
    let json_str = data.to_string();
    trace!("MT101 document JSON: {}", json_str);
    let mut deserializer = serde_json::Deserializer::from_str(&json_str);
    match serde_path_to_error::deserialize::<_, T>(&mut deserializer) {
        Ok(document_data) => {
            debug!("MT to MX document deserialization successful, converting to XML");
            xml_to_string(&document_data).map_err(|e| {
                let error_msg = format!("MT to MX document XML serialization failed: {e}");
                error!(error = %error_msg, "MT to MX document XML serialization failed");
                DataflowError::Validation(error_msg)
            })
        }
        Err(e) => {
            let error_msg = format!(
                "MT to MX document JSON deserialization failed at path '{}': {}",
                e.path(),
                e.inner()
            );
            error!(error = %error_msg, "MT to MX document JSON deserialization failed");
            Err(DataflowError::Validation(error_msg))
        }
    }
}
