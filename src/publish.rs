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
use tracing::{info, debug, warn, error, instrument};

pub struct PublishFunction;

// Configuration enum for different message types
#[derive(Debug, Clone)]
enum MessageTypeConfig {
    MT103 { document_field: &'static str },
    MT103Rejt { document_field: &'static str },
    MT103Retn { document_field: &'static str },
    MT103Stp { document_field: &'static str },
    MT202Core { document_field: &'static str },
    MT202Cov { document_field: &'static str },
    MT202Rejt { document_field: &'static str },
    MT202Retn { document_field: &'static str },
    MT205 { document_field: &'static str },
    MT205Cov { document_field: &'static str },
    MT205Rejt { document_field: &'static str },
    MT205Retn { document_field: &'static str },
    MT900 { document_field: &'static str },
    MT910 { document_field: &'static str },
    MT192 { document_field: &'static str },
    MT292 { document_field: &'static str },
    MT196 { document_field: &'static str },
    MT296 { document_field: &'static str },
}

impl MessageTypeConfig {
    #[instrument(level = "debug")]
    fn from_source_format(source_format: &str) -> Result<Self> {
        debug!(source_format = %source_format, "Determining message type configuration");
        
        let config = match source_format {
            "MT103.Header" | "MT103.Document" => MessageTypeConfig::MT103 {
                document_field: "FIToFICstmrCdtTrf",
            },
            "MT103_REJT.Header" | "MT103_REJT.Document" => MessageTypeConfig::MT103Rejt {
                document_field: "FIToFIPmtStsRpt",
            },
            "MT103_RETN.Header" | "MT103_RETN.Document" => MessageTypeConfig::MT103Retn {
                document_field: "PmtRtr",
            },
            "MT103_STP.Header" | "MT103_STP.Document" => MessageTypeConfig::MT103Stp {
                document_field: "FIToFICstmrCdtTrf",
            },
            "MT202_CORE.Header" | "MT202_CORE.Document" => MessageTypeConfig::MT202Core {
                document_field: "FIToFICdtTrf",
            },
            "MT202_COV.Header" | "MT202_COV.Document" => MessageTypeConfig::MT202Cov {
                document_field: "FIToFICdtTrf",
            },
            "MT202_REJT.Header" | "MT202_REJT.Document" => MessageTypeConfig::MT202Rejt {
                document_field: "FIToFIPmtStsRpt",
            },
            "MT202_RETN.Header" | "MT202_RETN.Document" => MessageTypeConfig::MT202Retn {
                document_field: "PmtRtr",
            },
            "MT205.Header" | "MT205.Document" => MessageTypeConfig::MT205 {
                document_field: "FIToFICdtTrf",
            },
            "MT205_COV.Header" | "MT205_COV.Document" => MessageTypeConfig::MT205Cov {
                document_field: "FIToFICdtTrf",
            },
            "MT205_REJT.Header" | "MT205_REJT.Document" => MessageTypeConfig::MT205Rejt {
                document_field: "FIToFIPmtStsRpt",
            },
            "MT205_RETN.Header" | "MT205_RETN.Document" => MessageTypeConfig::MT205Retn {
                document_field: "PmtRtr",
            },
            "MT900.Header" | "MT900.Document" => MessageTypeConfig::MT900 {
                document_field: "BkToCstmrDbtCdtNtfctn",
            },
            "MT910.Header" | "MT910.Document" => MessageTypeConfig::MT910 {
                document_field: "BkToCstmrDbtCdtNtfctn",
            },
            "MT192.Header" | "MT192.Document" => MessageTypeConfig::MT192 {
                document_field: "FIToFIPmtCxlReq",
            },
            "MT292.Header" | "MT292.Document" => MessageTypeConfig::MT292 {
                document_field: "FIToFIPmtCxlReq",
            },
            "MT196.Header" | "MT196.Document" => MessageTypeConfig::MT196 {
                document_field: "FIToFIPmtCxlReq",
            },
            "MT296.Header" | "MT296.Document" => MessageTypeConfig::MT296 {
                document_field: "FIToFIPmtCxlReq",
            },
            _ => {
                error!(source_format = %source_format, "Unsupported source format");
                return Err(DataflowError::Validation(format!(
                    "Unsupported source format: {}",
                    source_format
                )));
            }
        };
        
        debug!(config = ?config, "Message type configuration determined");
        Ok(config)
    }
}

#[async_trait]
impl AsyncFunctionHandler for PublishFunction {
    #[instrument(skip(self, message, input))]
    async fn execute(&self, message: &mut Message, input: &Value) -> Result<(usize, Vec<Change>)> {
        debug!("Starting message publishing/conversion");

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

        let data = message.data.get(input_field_name).ok_or_else(|| {
            error!(
                input_field = input_field_name,
                available_fields = ?message.data.as_object().map(|obj| obj.keys().collect::<Vec<_>>()),
                "Input field not found in message data"
            );
            DataflowError::Validation(format!(
                "Field {} not found in message data {}",
                input_field_name, message.data
            ))
        })?;

        let config = MessageTypeConfig::from_source_format(source_format)?;

        let result = if source_format.ends_with(".Header") {
            info!(source_format = %source_format, "Processing header");
            handle_header(data.clone(), message, output_field_name, &config)
        } else if source_format.ends_with(".Document") {
            info!(source_format = %source_format, "Processing document");
            handle_document(data.clone(), message, output_field_name, &config)
        } else {
            error!(source_format = %source_format, "Invalid source format - must end with .Header or .Document");
            Err(DataflowError::Validation(format!(
                "Invalid source format: {}",
                source_format
            )))
        };

        match &result {
            Ok((status, _)) => {
                info!(
                    source_format = %source_format,
                    output_field = output_field_name,
                    status = status,
                    "Message publishing completed successfully"
                );
            }
            Err(e) => {
                error!(
                    source_format = %source_format,
                    error = %e,
                    "Message publishing failed"
                );
            }
        }

        result
    }
}

// Generic header handler
#[instrument(skip(data, message), fields(output_field = output_field_name))]
fn handle_header(
    data: Value,
    message: &mut Message,
    output_field_name: &str,
    config: &MessageTypeConfig,
) -> Result<(usize, Vec<Change>)> {
    debug!(config = ?config, "Processing header with configuration");

    let xml_string = match config {
        MessageTypeConfig::MT103 { .. } => {
            debug!("Serializing MT103 header");
            serialize_header::<bah_pacs_008_001_08::BusinessApplicationHeaderV02>(data)?
        }
        MessageTypeConfig::MT103Rejt { .. } => {
            debug!("Serializing MT103 REJT header");
            serialize_header::<bah_pacs_002_001_10::BusinessApplicationHeaderV02>(data)?
        }
        MessageTypeConfig::MT103Retn { .. } => {
            debug!("Serializing MT103 RETN header");
            serialize_header::<bah_pacs_004_001_09::BusinessApplicationHeaderV02>(data)?
        }
        MessageTypeConfig::MT103Stp { .. } => {
            debug!("Serializing MT103 STP header");
            serialize_header::<bah_pacs_008_001_08_stp::BusinessApplicationHeaderV02>(data)?
        }
        MessageTypeConfig::MT202Core { .. } | MessageTypeConfig::MT202Cov { .. } => {
            debug!("Serializing MT202 header");
            serialize_header::<bah_pacs_009_001_08::BusinessApplicationHeaderV02>(data)?
        }
        MessageTypeConfig::MT202Rejt { .. } => {
            debug!("Serializing MT202 REJT header");
            serialize_header::<bah_pacs_002_001_10::BusinessApplicationHeaderV02>(data)?
        }
        MessageTypeConfig::MT202Retn { .. } => {
            debug!("Serializing MT202 RETN header");
            serialize_header::<bah_pacs_004_001_09::BusinessApplicationHeaderV02>(data)?
        }
        MessageTypeConfig::MT205 { .. } | MessageTypeConfig::MT205Cov { .. } => {
            debug!("Serializing MT205 header");
            serialize_header::<bah_pacs_009_001_08::BusinessApplicationHeaderV02>(data)?
        }
        MessageTypeConfig::MT205Rejt { .. } => {
            debug!("Serializing MT205 REJT header");
            serialize_header::<bah_pacs_002_001_10::BusinessApplicationHeaderV02>(data)?
        }
        MessageTypeConfig::MT205Retn { .. } => {
            debug!("Serializing MT205 RETN header");
            serialize_header::<bah_pacs_004_001_09::BusinessApplicationHeaderV02>(data)?
        }
        MessageTypeConfig::MT900 { .. } => {
            debug!("Serializing MT900 header");
            serialize_header::<bah_camt_054_001::BusinessApplicationHeaderV02>(data)?
        }
        MessageTypeConfig::MT910 { .. } => {
            debug!("Serializing MT910 header");
            serialize_header::<bah_camt_054_001::BusinessApplicationHeaderV02>(data)?
        }
        MessageTypeConfig::MT192 { .. }
        | MessageTypeConfig::MT292 { .. }
        | MessageTypeConfig::MT196 { .. }
        | MessageTypeConfig::MT296 { .. } => {
            debug!("Serializing MT192/MT292/MT196/MT296 header");
            serialize_header::<bah_camt_056_001_08::BusinessApplicationHeaderV02>(data)?
        }
    };

    let result_value = Value::String(xml_string);
    message.data[output_field_name] = result_value.clone();

    debug!(
        output_field = output_field_name,
        xml_length = result_value.as_str().map(|s| s.len()).unwrap_or(0),
        "Header serialization completed"
    );

    Ok((
        200,
        vec![Change {
            path: format!("data.{}", output_field_name),
            old_value: Value::Null,
            new_value: result_value,
        }],
    ))
}

// Generic document handler
#[instrument(skip(data, message), fields(output_field = output_field_name))]
fn handle_document(
    data: Value,
    message: &mut Message,
    output_field_name: &str,
    config: &MessageTypeConfig,
) -> Result<(usize, Vec<Change>)> {
    debug!(config = ?config, "Processing document with configuration");

    let xml_string = match config {
        MessageTypeConfig::MT103 { document_field } => {
            debug!(document_field = document_field, "Serializing MT103 document");
            serialize_document(
                data,
                document_field,
                |pacs_data: pacs_008_001_08::FIToFICustomerCreditTransferV08| {
                    mx_message::app_document::Document::FIToFICustomerCreditTransferV08(Box::new(
                        pacs_data,
                    ))
                },
            )?
        }
        MessageTypeConfig::MT103Rejt { document_field } => {
            debug!(document_field = document_field, "Serializing MT103 REJT document");
            serialize_document(
                data,
                document_field,
                |pacs_data: pacs_002_001_10::FIToFIPaymentStatusReportV10| {
                    mx_message::app_document::Document::FIToFIPaymentStatusReportV10(Box::new(
                        pacs_data,
                    ))
                },
            )?
        }
        MessageTypeConfig::MT103Retn { document_field } => {
            debug!(document_field = document_field, "Serializing MT103 RETN document");
            serialize_document(
                data,
                document_field,
                |pacs_data: pacs_004_001_09::PaymentReturnV09| {
                    mx_message::app_document::Document::PaymentReturnV09(Box::new(pacs_data))
                },
            )?
        }
        MessageTypeConfig::MT103Stp { document_field } => {
            debug!(document_field = document_field, "Serializing MT103 STP document");
            serialize_document(
                data,
                document_field,
                |pacs_data: pacs_008_001_08_stp::FIToFICustomerCreditTransferV08| {
                    mx_message::app_document::Document::FIToFICustomerCreditTransferV08STP(Box::new(
                        pacs_data,
                    ))
                },
            )?
        }
        MessageTypeConfig::MT202Core { document_field }
        | MessageTypeConfig::MT202Cov { document_field }
        | MessageTypeConfig::MT205 { document_field }
        | MessageTypeConfig::MT205Cov { document_field } => {
            debug!(document_field = document_field, "Serializing MT202/MT205 document");
            serialize_document(
                data,
                document_field,
                |pacs_data: pacs_009_001_08::FinancialInstitutionCreditTransferV08| {
                    mx_message::app_document::Document::FinancialInstitutionCreditTransferV08(Box::new(
                        pacs_data,
                    ))
                },
            )?
        }
        MessageTypeConfig::MT202Rejt { document_field }
        | MessageTypeConfig::MT205Rejt { document_field } => {
            debug!(document_field = document_field, "Serializing MT202/MT205 REJT document");
            serialize_document(
                data,
                document_field,
                |pacs_data: pacs_002_001_10::FIToFIPaymentStatusReportV10| {
                    mx_message::app_document::Document::FIToFIPaymentStatusReportV10(Box::new(
                        pacs_data,
                    ))
                },
            )?
        }
        MessageTypeConfig::MT202Retn { document_field }
        | MessageTypeConfig::MT205Retn { document_field } => {
            debug!(document_field = document_field, "Serializing MT202/MT205 RETN document");
            serialize_document(
                data,
                document_field,
                |pacs_data: pacs_004_001_09::PaymentReturnV09| {
                    mx_message::app_document::Document::PaymentReturnV09(Box::new(pacs_data))
                },
            )?
        }
        MessageTypeConfig::MT900 { document_field }
        | MessageTypeConfig::MT910 { document_field } => {
            debug!(document_field = document_field, "Serializing MT900/MT910 document");
            serialize_document(
                data,
                document_field,
                |camt_data: camt_054_001_08::BankToCustomerDebitCreditNotificationV08| {
                    mx_message::app_document::Document::BankToCustomerDebitCreditNotificationV08(
                        Box::new(camt_data),
                    )
                },
            )?
        }
        MessageTypeConfig::MT192 { document_field }
        | MessageTypeConfig::MT292 { document_field }
        | MessageTypeConfig::MT196 { document_field }
        | MessageTypeConfig::MT296 { document_field } => {
            debug!(document_field = document_field, "Serializing MT192/MT292/MT196/MT296 document");
            serialize_document(
                data,
                document_field,
                |camt_data: camt_056_001_08::FIToFIPaymentCancellationRequestV08| {
                    mx_message::app_document::Document::FIToFIPaymentCancellationRequestV08(Box::new(
                        camt_data,
                    ))
                },
            )?
        }
    };

    // Store as array with single document
    let result_array = vec![Value::String(xml_string)];
    let result_value = Value::Array(result_array);
    message.data[output_field_name] = result_value.clone();

    debug!(
        output_field = output_field_name,
        document_count = 1,
        "Document serialization completed"
    );

    Ok((
        200,
        vec![Change {
            path: format!("data.{}", output_field_name),
            old_value: Value::Null,
            new_value: result_value,
        }],
    ))
}

// Generic header serialization helper
#[instrument(skip(data))]
fn serialize_header<T>(data: Value) -> Result<String>
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    debug!("Deserializing header data");
    
    // Use serde_path_to_error for detailed path information
    let json_str = data.to_string();
    let mut deserializer = serde_json::Deserializer::from_str(&json_str);
    match serde_path_to_error::deserialize::<_, T>(&mut deserializer) {
        Ok(header_data) => {
            debug!("Header deserialization successful, converting to XML");
            xml_to_string(&header_data).map_err(|e| {
                let error_msg = format!(
                    "Header XML serialization failed at path: {}, error: {}",
                    get_xml_error_path(&e),
                    e
                );
                error!(error = %error_msg, "Header XML serialization failed");
                DataflowError::Validation(error_msg)
            })
        }
        Err(e) => {
            let error_msg = format!(
                "AppHdr JSON deserialization failed at path '{}': {}",
                e.path(),
                e.inner()
            );
            error!(error = %error_msg, "Header JSON deserialization failed");
            Err(DataflowError::Validation(error_msg))
        }
    }
}

// Generic document serialization helper
#[instrument(skip(data, doc_wrapper), fields(field_name = field_name))]
fn serialize_document<T, F>(data: Value, field_name: &str, doc_wrapper: F) -> Result<String>
where
    T: serde::de::DeserializeOwned,
    F: FnOnce(T) -> mx_message::app_document::Document,
    mx_message::app_document::Document: serde::Serialize,
{
    debug!(field_name = field_name, "Extracting document field data");
    
    let field_data = data.get(field_name).ok_or_else(|| {
        error!(field_name = field_name, "Document field not found in data");
        DataflowError::Validation(format!(
            "{} not found in document at path: data.{}",
            field_name, field_name
        ))
    })?;

    debug!("Deserializing document field data");
    
    // Use serde_path_to_error for detailed path information
    let json_str = field_data.to_string();
    let mut deserializer = serde_json::Deserializer::from_str(&json_str);
    match serde_path_to_error::deserialize::<_, T>(&mut deserializer) {
        Ok(parsed_data) => {
            debug!("Document deserialization successful, wrapping and converting to XML");
            let document = doc_wrapper(parsed_data);
            xml_to_string(&document).map_err(|e| {
                let error_msg = format!(
                    "Document XML serialization failed at path: {}.{}, error: {}",
                    field_name,
                    get_xml_error_path(&e),
                    e
                );
                error!(error = %error_msg, "Document XML serialization failed");
                DataflowError::Validation(error_msg)
            })
        }
        Err(e) => {
            let error_msg = format!(
                "{} JSON deserialization failed at path: {}.{}: {}",
                std::any::type_name::<T>(),
                field_name,
                e.path(),
                e.inner()
            );
            error!(error = %error_msg, "Document JSON deserialization failed");
            Err(DataflowError::Validation(error_msg))
        }
    }
}

// Helper function to extract path information from XML serialization errors
#[instrument(skip(error))]
fn get_xml_error_path(error: &quick_xml::DeError) -> String {
    let error_msg = error.to_string();
    
    // Try to extract field name from error message patterns
    if let Some(start) = error_msg.find("field `") {
        if let Some(end) = error_msg[start + 7..].find('`') {
            return error_msg[start + 7..start + 7 + end].to_string();
        }
    }

    // Try to extract struct name for context
    if let Some(start) = error_msg.find("struct `") {
        if let Some(end) = error_msg[start + 8..].find('`') {
            return format!("in_struct_{}", &error_msg[start + 8..start + 8 + end]);
        }
    }

    // Try to extract any identifiers in backticks
    if let Some(start) = error_msg.find('`') {
        if let Some(end) = error_msg[start + 1..].find('`') {
            return error_msg[start + 1..start + 1 + end].to_string();
        }
    }

    // Fallback to generic path info
    format!(
        "unknown_path ({})",
        error_msg.chars().take(50).collect::<String>()
    )
}
