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
    fn from_source_format(source_format: &str) -> Result<Self> {
        match source_format {
            "MT103.Header" | "MT103.Document" => Ok(MessageTypeConfig::MT103 {
                document_field: "FIToFICstmrCdtTrf",
            }),
            "MT103_REJT.Header" | "MT103_REJT.Document" => Ok(MessageTypeConfig::MT103Rejt {
                document_field: "FIToFIPmtStsRpt",
            }),
            "MT103_RETN.Header" | "MT103_RETN.Document" => Ok(MessageTypeConfig::MT103Retn {
                document_field: "PmtRtr",
            }),
            "MT103_STP.Header" | "MT103_STP.Document" => Ok(MessageTypeConfig::MT103Stp {
                document_field: "FIToFICstmrCdtTrf",
            }),
            "MT202_CORE.Header" | "MT202_CORE.Document" => Ok(MessageTypeConfig::MT202Core {
                document_field: "FIToFICdtTrf",
            }),
            "MT202_COV.Header" | "MT202_COV.Document" => Ok(MessageTypeConfig::MT202Cov {
                document_field: "FIToFICdtTrf",
            }),
            "MT202_REJT.Header" | "MT202_REJT.Document" => Ok(MessageTypeConfig::MT202Rejt {
                document_field: "FIToFIPmtStsRpt",
            }),
            "MT202_RETN.Header" | "MT202_RETN.Document" => Ok(MessageTypeConfig::MT202Retn {
                document_field: "PmtRtr",
            }),
            "MT205.Header" | "MT205.Document" => Ok(MessageTypeConfig::MT205 {
                document_field: "FIToFICdtTrf",
            }),
            "MT205_COV.Header" | "MT205_COV.Document" => Ok(MessageTypeConfig::MT205Cov {
                document_field: "FIToFICdtTrf",
            }),
            "MT205_REJT.Header" | "MT205_REJT.Document" => Ok(MessageTypeConfig::MT205Rejt {
                document_field: "FIToFIPmtStsRpt",
            }),
            "MT205_RETN.Header" | "MT205_RETN.Document" => Ok(MessageTypeConfig::MT205Retn {
                document_field: "PmtRtr",
            }),
            "MT900.Header" | "MT900.Document" => Ok(MessageTypeConfig::MT900 {
                document_field: "BkToCstmrDbtCdtNtfctn",
            }),
            "MT910.Header" | "MT910.Document" => Ok(MessageTypeConfig::MT910 {
                document_field: "BkToCstmrDbtCdtNtfctn",
            }),
            "MT192.Header" | "MT192.Document" => Ok(MessageTypeConfig::MT192 {
                document_field: "FIToFIPmtCxlReq",
            }),
            "MT292.Header" | "MT292.Document" => Ok(MessageTypeConfig::MT292 {
                document_field: "FIToFIPmtCxlReq",
            }),
            "MT196.Header" | "MT196.Document" => Ok(MessageTypeConfig::MT196 {
                document_field: "FIToFIPmtCxlReq",
            }),
            "MT296.Header" | "MT296.Document" => Ok(MessageTypeConfig::MT296 {
                document_field: "FIToFIPmtCxlReq",
            }),
            _ => Err(DataflowError::Validation(format!(
                "Unsupported source format: {}",
                source_format
            ))),
        }
    }
}

#[async_trait]
impl AsyncFunctionHandler for PublishFunction {
    async fn execute(&self, message: &mut Message, input: &Value) -> Result<(usize, Vec<Change>)> {
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
            DataflowError::Validation(format!(
                "Field {} not found in message data {}",
                input_field_name, message.data
            ))
        })?;

        let config = MessageTypeConfig::from_source_format(source_format)?;

        if source_format.ends_with(".Header") {
            handle_header(data.clone(), message, output_field_name, &config)
        } else if source_format.ends_with(".Document") {
            handle_document(data.clone(), message, output_field_name, &config)
        } else {
            Err(DataflowError::Validation(format!(
                "Invalid source format: {}",
                source_format
            )))
        }
    }
}

// Generic header handler
fn handle_header(
    data: Value,
    message: &mut Message,
    output_field_name: &str,
    config: &MessageTypeConfig,
) -> Result<(usize, Vec<Change>)> {
    let xml_string = match config {
        MessageTypeConfig::MT103 { .. } => {
            serialize_header::<bah_pacs_008_001_08::BusinessApplicationHeaderV02>(data)?
        }
        MessageTypeConfig::MT103Rejt { .. } => {
            serialize_header::<bah_pacs_002_001_10::BusinessApplicationHeaderV02>(data)?
        }
        MessageTypeConfig::MT103Retn { .. } => {
            serialize_header::<bah_pacs_004_001_09::BusinessApplicationHeaderV02>(data)?
        }
        MessageTypeConfig::MT103Stp { .. } => {
            serialize_header::<bah_pacs_008_001_08_stp::BusinessApplicationHeaderV02>(data)?
        }
        MessageTypeConfig::MT202Core { .. } | MessageTypeConfig::MT202Cov { .. } => {
            serialize_header::<bah_pacs_009_001_08::BusinessApplicationHeaderV02>(data)?
        }
        MessageTypeConfig::MT202Rejt { .. } => {
            serialize_header::<bah_pacs_002_001_10::BusinessApplicationHeaderV02>(data)?
        }
        MessageTypeConfig::MT202Retn { .. } => {
            serialize_header::<bah_pacs_004_001_09::BusinessApplicationHeaderV02>(data)?
        }
        MessageTypeConfig::MT205 { .. } | MessageTypeConfig::MT205Cov { .. } => {
            serialize_header::<bah_pacs_009_001_08::BusinessApplicationHeaderV02>(data)?
        }
        MessageTypeConfig::MT205Rejt { .. } => {
            serialize_header::<bah_pacs_002_001_10::BusinessApplicationHeaderV02>(data)?
        }
        MessageTypeConfig::MT205Retn { .. } => {
            serialize_header::<bah_pacs_004_001_09::BusinessApplicationHeaderV02>(data)?
        }
        MessageTypeConfig::MT900 { .. } => {
            serialize_header::<bah_camt_054_001::BusinessApplicationHeaderV02>(data)?
        }
        MessageTypeConfig::MT910 { .. } => {
            serialize_header::<bah_camt_054_001::BusinessApplicationHeaderV02>(data)?
        }
        MessageTypeConfig::MT192 { .. }
        | MessageTypeConfig::MT292 { .. }
        | MessageTypeConfig::MT196 { .. }
        | MessageTypeConfig::MT296 { .. } => {
            serialize_header::<bah_camt_056_001_08::BusinessApplicationHeaderV02>(data)?
        }
    };

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

// Generic document handler
fn handle_document(
    data: Value,
    message: &mut Message,
    output_field_name: &str,
    config: &MessageTypeConfig,
) -> Result<(usize, Vec<Change>)> {
    let xml_string = match config {
        MessageTypeConfig::MT103 { document_field } => serialize_document(
            data,
            document_field,
            |pacs_data: pacs_008_001_08::FIToFICustomerCreditTransferV08| {
                mx_message::app_document::Document::FIToFICustomerCreditTransferV08(Box::new(
                    pacs_data,
                ))
            },
        )?,
        MessageTypeConfig::MT103Rejt { document_field } => serialize_document(
            data,
            document_field,
            |pacs_data: pacs_002_001_10::FIToFIPaymentStatusReportV10| {
                mx_message::app_document::Document::FIToFIPaymentStatusReportV10(Box::new(
                    pacs_data,
                ))
            },
        )?,
        MessageTypeConfig::MT103Retn { document_field } => serialize_document(
            data,
            document_field,
            |pacs_data: pacs_004_001_09::PaymentReturnV09| {
                mx_message::app_document::Document::PaymentReturnV09(Box::new(pacs_data))
            },
        )?,
        MessageTypeConfig::MT103Stp { document_field } => serialize_document(
            data,
            document_field,
            |pacs_data: pacs_008_001_08_stp::FIToFICustomerCreditTransferV08| {
                mx_message::app_document::Document::FIToFICustomerCreditTransferV08STP(Box::new(
                    pacs_data,
                ))
            },
        )?,
        MessageTypeConfig::MT202Core { document_field }
        | MessageTypeConfig::MT202Cov { document_field }
        | MessageTypeConfig::MT205 { document_field }
        | MessageTypeConfig::MT205Cov { document_field } => serialize_document(
            data,
            document_field,
            |pacs_data: pacs_009_001_08::FinancialInstitutionCreditTransferV08| {
                mx_message::app_document::Document::FinancialInstitutionCreditTransferV08(Box::new(
                    pacs_data,
                ))
            },
        )?,
        MessageTypeConfig::MT202Rejt { document_field }
        | MessageTypeConfig::MT205Rejt { document_field } => serialize_document(
            data,
            document_field,
            |pacs_data: pacs_002_001_10::FIToFIPaymentStatusReportV10| {
                mx_message::app_document::Document::FIToFIPaymentStatusReportV10(Box::new(
                    pacs_data,
                ))
            },
        )?,
        MessageTypeConfig::MT202Retn { document_field }
        | MessageTypeConfig::MT205Retn { document_field } => serialize_document(
            data,
            document_field,
            |pacs_data: pacs_004_001_09::PaymentReturnV09| {
                mx_message::app_document::Document::PaymentReturnV09(Box::new(pacs_data))
            },
        )?,
        MessageTypeConfig::MT900 { document_field }
        | MessageTypeConfig::MT910 { document_field } => serialize_document(
            data,
            document_field,
            |camt_data: camt_054_001_08::BankToCustomerDebitCreditNotificationV08| {
                mx_message::app_document::Document::BankToCustomerDebitCreditNotificationV08(
                    Box::new(camt_data),
                )
            },
        )?,
        MessageTypeConfig::MT192 { document_field }
        | MessageTypeConfig::MT292 { document_field }
        | MessageTypeConfig::MT196 { document_field }
        | MessageTypeConfig::MT296 { document_field } => serialize_document(
            data,
            document_field,
            |camt_data: camt_056_001_08::FIToFIPaymentCancellationRequestV08| {
                mx_message::app_document::Document::FIToFIPaymentCancellationRequestV08(Box::new(
                    camt_data,
                ))
            },
        )?,
    };

    // Store as array with single document
    let result_array = vec![Value::String(xml_string)];
    let result_value = Value::Array(result_array);
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

// Generic header serialization helper
fn serialize_header<T>(data: Value) -> Result<String>
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    // Use serde_path_to_error for detailed path information
    let json_str = data.to_string();
    let mut deserializer = serde_json::Deserializer::from_str(&json_str);
    match serde_path_to_error::deserialize::<_, T>(&mut deserializer) {
        Ok(header_data) => xml_to_string(&header_data).map_err(|e| {
            let error_msg = format!(
                "Header XML serialization failed at path: {}, error: {}",
                get_xml_error_path(&e),
                e
            );
            println!("{}", error_msg);
            DataflowError::Validation(error_msg)
        }),
        Err(e) => {
            let error_msg = format!(
                "AppHdr JSON deserialization failed at path '{}': {}",
                e.path(),
                e.inner()
            );
            println!("{}", error_msg);
            Err(DataflowError::Validation(error_msg))
        }
    }
}

// Generic document serialization helper
fn serialize_document<T, F>(data: Value, field_name: &str, doc_wrapper: F) -> Result<String>
where
    T: serde::de::DeserializeOwned,
    F: FnOnce(T) -> mx_message::app_document::Document,
    mx_message::app_document::Document: serde::Serialize,
{
    let field_data = data.get(field_name).ok_or_else(|| {
        DataflowError::Validation(format!(
            "{} not found in document at path: data.{}",
            field_name, field_name
        ))
    })?;

    // Use serde_path_to_error for detailed path information
    let json_str = field_data.to_string();
    let mut deserializer = serde_json::Deserializer::from_str(&json_str);
    match serde_path_to_error::deserialize::<_, T>(&mut deserializer) {
        Ok(parsed_data) => {
            let document = doc_wrapper(parsed_data);
            xml_to_string(&document).map_err(|e| {
                let error_msg = format!(
                    "Document XML serialization failed at path: {}.{}, error: {}",
                    field_name,
                    get_xml_error_path(&e),
                    e
                );
                println!("{}", error_msg);
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
            println!("{}", error_msg);
            Err(DataflowError::Validation(error_msg))
        }
    }
}

// Helper function to extract path information from XML serialization errors
fn get_xml_error_path(error: &quick_xml::DeError) -> String {
    // Extract meaningful path information from the error message
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
