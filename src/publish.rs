use async_trait::async_trait;
use dataflow_rs::engine::error::DataflowError;
use dataflow_rs::engine::{
    AsyncFunctionHandler,
    error::Result,
    message::{Change, Message},
};
use quick_xml::se::to_string as xml_to_string;
use serde_json::Value;

pub struct PublishFunction;

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

        match source_format {
            "MT103.Header" => {
                // Handle MT103.Header
                handle_mt103_header(data.clone(), message, output_field_name)
            }
            "MT103.Document" => {
                // Handle MT103.Document
                handle_mt103_document(data.clone(), message, output_field_name)
            }
            "MT103_REJT.Header" => {
                // Handle MT103_REJT.Header
                handle_mt103_rejt_header(data.clone(), message, output_field_name)
            }
            "MT103_REJT.Document" => {
                // Handle MT103_REJT.Document
                handle_mt103_rejt_document(data.clone(), message, output_field_name)
            }
            "MT103_RETN.Header" => {
                // Handle MT103_RETN.Header
                handle_mt103_retn_header(data.clone(), message, output_field_name)
            }
            "MT103_RETN.Document" => {
                // Handle MT103_RETN.Document
                handle_mt103_retn_document(data.clone(), message, output_field_name)
            }
            _ => Err(DataflowError::Validation(format!(
                "Unsupported output format: {}",
                source_format
            ))),
        }
    }
}

// Handle MT103 Header - generates AppHdr XML
fn handle_mt103_header(
    data: Value,
    message: &mut Message,
    output_field_name: &str,
) -> Result<(usize, Vec<Change>)> {
    use mx_message::header::bah_pacs_008_001_08::BusinessApplicationHeaderV02;

    // Try to use the AppHdr from mx-message if the data structure is compatible
    match serde_json::from_value::<BusinessApplicationHeaderV02>(data.clone()) {
        Ok(header_data) => {
            // Use mx-message serialization
            match xml_to_string(&header_data) {
                Ok(xml_string) => {
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
                Err(e) => {
                    println!("Header XML serialization failed: {}", e);
                    Err(DataflowError::Validation(format!(
                        "Header XML serialization failed: {}",
                        e
                    )))
                }
            }
        }
        Err(e) => {
            println!("AppHdr deserialization failed: {}", e);
            Err(DataflowError::Validation(format!(
                "AppHdr deserialization failed: {}",
                e
            )))
        }
    }
}

// Handle MT103 Document - generates Document XML in array format
fn handle_mt103_document(
    data: Value,
    message: &mut Message,
    output_field_name: &str,
) -> Result<(usize, Vec<Change>)> {
    use mx_message::{
        app_document::Document, document::pacs_008_001_08::FIToFICustomerCreditTransferV08,
    };

    // Extract FIToFICstmrCdtTrf from the data
    let fi_to_fi = data.get("FIToFICstmrCdtTrf").ok_or_else(|| {
        DataflowError::Validation("FIToFICstmrCdtTrf not found in document".to_string())
    })?;

    // Serialize using mx-message structures
    match serde_json::from_value::<FIToFICustomerCreditTransferV08>(fi_to_fi.clone()) {
        Ok(pacs_data) => {
            let document = Document::FIToFICustomerCreditTransferV08(Box::new(pacs_data));
            match xml_to_string(&document) {
                Ok(xml_string) => {
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
                Err(e) => {
                    println!("Document XML serialization failed: {}", e);
                    Err(DataflowError::Validation(format!(
                        "Document XML serialization failed: {}",
                        e
                    )))
                }
            }
        }
        Err(e) => Err(DataflowError::Validation(format!(
            "FIToFICustomerCreditTransferV08 deserialization failed: {}",
            e
        ))),
    }
}

fn handle_mt103_rejt_header(
    data: Value,
    message: &mut Message,
    output_field_name: &str,
) -> Result<(usize, Vec<Change>)> {
    use mx_message::header::bah_pacs_002_001_10::BusinessApplicationHeaderV02;

    // Try to use the AppHdr from mx-message if the data structure is compatible
    match serde_json::from_value::<BusinessApplicationHeaderV02>(data.clone()) {
        Ok(header_data) => {
            // Use mx-message serialization
            match xml_to_string(&header_data) {
                Ok(xml_string) => {
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
                Err(e) => {
                    println!("Header XML serialization failed: {}", e);
                    Err(DataflowError::Validation(format!(
                        "Header XML serialization failed: {}",
                        e
                    )))
                }
            }
        }
        Err(e) => Err(DataflowError::Validation(format!(
            "AppHdr deserialization failed: {}",
            e
        ))),
    }
}

fn handle_mt103_rejt_document(
    data: Value,
    message: &mut Message,
    output_field_name: &str,
) -> Result<(usize, Vec<Change>)> {
    use mx_message::{
        app_document::Document, document::pacs_002_001_10::FIToFIPaymentStatusReportV10,
    };

    // Extract FIToFIPaymentStatusReport from the data
    let fi_to_fi = data.get("FIToFIPmtStsRpt").ok_or_else(|| {
        DataflowError::Validation("FIToFIPmtStsRpt not found in document".to_string())
    })?;

    // Serialize using mx-message structures
    match serde_json::from_value::<FIToFIPaymentStatusReportV10>(fi_to_fi.clone()) {
        Ok(pacs_data) => {
            let document = Document::FIToFIPaymentStatusReportV10(Box::new(pacs_data));
            match xml_to_string(&document) {
                Ok(xml_string) => {
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
                Err(e) => {
                    println!("Document XML serialization failed: {}", e);
                    Err(DataflowError::Validation(format!(
                        "Document XML serialization failed: {}",
                        e
                    )))
                }
            }
        }
        Err(e) => Err(DataflowError::Validation(format!(
            "FIToFIPaymentStatusReportV10 deserialization failed: {}",
            e
        ))),
    }
}

fn handle_mt103_retn_header(
    data: Value,
    message: &mut Message,
    output_field_name: &str,
) -> Result<(usize, Vec<Change>)> {
    use mx_message::header::bah_pacs_004_001_09::BusinessApplicationHeaderV02;

    // Try to use the AppHdr from mx-message if the data structure is compatible
    match serde_json::from_value::<BusinessApplicationHeaderV02>(data.clone()) {
        Ok(header_data) => {
            // Use mx-message serialization
            match xml_to_string(&header_data) {
                Ok(xml_string) => {
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
                Err(e) => {
                    println!("Header XML serialization failed: {}", e);
                    Err(DataflowError::Validation(format!(
                        "Header XML serialization failed: {}",
                        e
                    )))
                }
            }
        }
        Err(e) => {
            println!("AppHdr deserialization failed: {}", e);
            Err(DataflowError::Validation(format!(
                "AppHdr deserialization failed: {}",
                e
            )))
        }
    }
}

fn handle_mt103_retn_document(
    data: Value,
    message: &mut Message,
    output_field_name: &str,
) -> Result<(usize, Vec<Change>)> {
    use mx_message::{app_document::Document, document::pacs_004_001_09::PaymentReturnV09};

    // Extract PaymentReturn from the data
    let pmt_rtr = data
        .get("PmtRtr")
        .ok_or_else(|| DataflowError::Validation("PmtRtr not found in document".to_string()))?;

    // Serialize using mx-message structures
    match serde_json::from_value::<PaymentReturnV09>(pmt_rtr.clone()) {
        Ok(pacs_data) => {
            let document = Document::PaymentReturnV09(Box::new(pacs_data));
            match xml_to_string(&document) {
                Ok(xml_string) => {
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
                Err(e) => {
                    println!("Document XML serialization failed: {}", e);
                    Err(DataflowError::Validation(format!(
                        "Document XML serialization failed: {}",
                        e
                    )))
                }
            }
        }
        Err(e) => Err(DataflowError::Validation(format!(
            "PaymentReturnV09 deserialization failed: {}",
            e
        ))),
    }
}
