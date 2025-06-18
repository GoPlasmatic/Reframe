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
            "MT103_STP.Header" => {
                // Handle MT103_STP.Header
                handle_mt103_stp_header(data.clone(), message, output_field_name)
            }
            "MT103_STP.Document" => {
                // Handle MT103_STP.Document
                handle_mt103_stp_document(data.clone(), message, output_field_name)
            }
            "MT202_CORE.Header" => {
                // Handle MT202_CORE.Header
                handle_mt202_core_header(data.clone(), message, output_field_name)
            }
            "MT202_CORE.Document" => {
                // Handle MT202_CORE.Document
                handle_mt202_core_document(data.clone(), message, output_field_name)
            }
            "MT202_COV.Header" => {
                // Handle MT202_COV.Header
                handle_mt202_cov_header(data.clone(), message, output_field_name)
            }
            "MT202_COV.Document" => {
                // Handle MT202_COV.Document
                handle_mt202_cov_document(data.clone(), message, output_field_name)
            }
            "MT202_REJT.Header" => {
                // Handle MT202_REJT.Header
                handle_mt202_rejt_header(data.clone(), message, output_field_name)
            }
            "MT202_REJT.Document" => {
                // Handle MT202_REJT.Document
                handle_mt202_rejt_document(data.clone(), message, output_field_name)
            }
            "MT202_RETN.Header" => {
                // Handle MT202_RETN.Header
                handle_mt202_retn_header(data.clone(), message, output_field_name)
            }
            "MT202_RETN.Document" => {
                // Handle MT202_RETN.Document
                handle_mt202_retn_document(data.clone(), message, output_field_name)
            }
            "MT205.Header" => {
                // Handle MT205.Header
                handle_mt205_header(data.clone(), message, output_field_name)
            }
            "MT205.Document" => {
                // Handle MT205.Document
                handle_mt205_document(data.clone(), message, output_field_name)
            }
            "MT205_COV.Header" => {
                // Handle MT205_COV.Header
                handle_mt205_cov_header(data.clone(), message, output_field_name)
            }
            "MT205_COV.Document" => {
                // Handle MT205_COV.Document
                handle_mt205_cov_document(data.clone(), message, output_field_name)
            }
            "MT205_REJT.Header" => {
                // Handle MT205_REJT.Header
                handle_mt205_rejt_header(data.clone(), message, output_field_name)
            }
            "MT205_REJT.Document" => {
                // Handle MT205_REJT.Document
                handle_mt205_rejt_document(data.clone(), message, output_field_name)
            }
            "MT205_RETN.Header" => {
                // Handle MT205_RETN.Header
                handle_mt205_retn_header(data.clone(), message, output_field_name)
            }
            "MT205_RETN.Document" => {
                // Handle MT205_RETN.Document
                handle_mt205_retn_document(data.clone(), message, output_field_name)
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

fn handle_mt103_stp_header(
    data: Value,
    message: &mut Message,
    output_field_name: &str,
) -> Result<(usize, Vec<Change>)> {
    use mx_message::header::bah_pacs_008_001_08_stp::BusinessApplicationHeaderV02;

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
                    println!("STP Header XML serialization failed: {}", e);
                    Err(DataflowError::Validation(format!(
                        "STP Header XML serialization failed: {}",
                        e
                    )))
                }
            }
        }
        Err(e) => {
            println!("STP AppHdr deserialization failed: {}", e);
            Err(DataflowError::Validation(format!(
                "STP AppHdr deserialization failed: {}",
                e
            )))
        }
    }
}

fn handle_mt103_stp_document(
    data: Value,
    message: &mut Message,
    output_field_name: &str,
) -> Result<(usize, Vec<Change>)> {
    use mx_message::{
        app_document::Document, document::pacs_008_001_08_stp::FIToFICustomerCreditTransferV08,
    };

    // Extract FIToFICstmrCdtTrf from the data (STP uses same structure as regular MT103)
    let fi_to_fi = data.get("FIToFICstmrCdtTrf").ok_or_else(|| {
        DataflowError::Validation("FIToFICstmrCdtTrf not found in STP document".to_string())
    })?;

    // Serialize using mx-message structures
    match serde_json::from_value::<FIToFICustomerCreditTransferV08>(fi_to_fi.clone()) {
        Ok(pacs_data) => {
            let document = Document::FIToFICustomerCreditTransferV08STP(Box::new(pacs_data));
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
                    println!("STP Document XML serialization failed: {}", e);
                    Err(DataflowError::Validation(format!(
                        "STP Document XML serialization failed: {}",
                        e
                    )))
                }
            }
        }
        Err(e) => Err(DataflowError::Validation(format!(
            "STP FIToFICustomerCreditTransferV08 deserialization failed: {}",
            e
        ))),
    }
}

// Handle MT202 Core Header - generates AppHdr XML for MT202
fn handle_mt202_core_header(
    data: Value,
    message: &mut Message,
    output_field_name: &str,
) -> Result<(usize, Vec<Change>)> {
    use mx_message::header::bah_pacs_009_001_08::BusinessApplicationHeaderV02;

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
                    println!("MT202 Core Header XML serialization failed: {}", e);
                    Err(DataflowError::Validation(format!(
                        "MT202 Core Header XML serialization failed: {}",
                        e
                    )))
                }
            }
        }
        Err(e) => {
            println!("MT202 Core AppHdr deserialization failed: {}", e);
            Err(DataflowError::Validation(format!(
                "MT202 Core AppHdr deserialization failed: {}",
                e
            )))
        }
    }
}

// Handle MT202 Core Document - generates Document XML for MT202 Core
fn handle_mt202_core_document(
    data: Value,
    message: &mut Message,
    output_field_name: &str,
) -> Result<(usize, Vec<Change>)> {
    use mx_message::{
        app_document::Document, document::pacs_009_001_08::FinancialInstitutionCreditTransferV08,
    };

    // Extract FIToFICdtTrf from the data (MT202 uses pacs.009 instead of pacs.008)
    let fi_to_fi = data.get("FIToFICdtTrf").ok_or_else(|| {
        DataflowError::Validation("FIToFICdtTrf not found in MT202 document".to_string())
    })?;

    // Serialize using mx-message structures
    match serde_json::from_value::<FinancialInstitutionCreditTransferV08>(fi_to_fi.clone()) {
        Ok(pacs_data) => {
            let document = Document::FinancialInstitutionCreditTransferV08(Box::new(pacs_data));
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
                    println!("MT202 Core Document XML serialization failed: {}", e);
                    Err(DataflowError::Validation(format!(
                        "MT202 Core Document XML serialization failed: {}",
                        e
                    )))
                }
            }
        }
        Err(e) => Err(DataflowError::Validation(format!(
            "MT202 Core FinancialInstitutionCreditTransferV08 deserialization failed: {}",
            e
        ))),
    }
}

// Handle MT202 COV Header - generates AppHdr XML for MT202 Cover
fn handle_mt202_cov_header(
    data: Value,
    message: &mut Message,
    output_field_name: &str,
) -> Result<(usize, Vec<Change>)> {
    use mx_message::header::bah_pacs_009_001_08::BusinessApplicationHeaderV02;

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
                    println!("MT202 COV Header XML serialization failed: {}", e);
                    Err(DataflowError::Validation(format!(
                        "MT202 COV Header XML serialization failed: {}",
                        e
                    )))
                }
            }
        }
        Err(e) => {
            println!("MT202 COV AppHdr deserialization failed: {}", e);
            Err(DataflowError::Validation(format!(
                "MT202 COV AppHdr deserialization failed: {}",
                e
            )))
        }
    }
}

// Handle MT202 COV Document - generates Document XML for MT202 Cover
fn handle_mt202_cov_document(
    data: Value,
    message: &mut Message,
    output_field_name: &str,
) -> Result<(usize, Vec<Change>)> {
    use mx_message::{
        app_document::Document, document::pacs_009_001_08::FinancialInstitutionCreditTransferV08,
    };

    // Extract FIToFICdtTrf from the data (MT202 COV uses pacs.009 COVE)
    let fi_to_fi = data.get("FIToFICdtTrf").ok_or_else(|| {
        DataflowError::Validation("FIToFICdtTrf not found in MT202 COV document".to_string())
    })?;

    // Serialize using mx-message structures
    match serde_json::from_value::<FinancialInstitutionCreditTransferV08>(fi_to_fi.clone()) {
        Ok(pacs_data) => {
            let document = Document::FinancialInstitutionCreditTransferV08(Box::new(pacs_data));
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
                    println!("MT202 COV Document XML serialization failed: {}", e);
                    Err(DataflowError::Validation(format!(
                        "MT202 COV Document XML serialization failed: {}",
                        e
                    )))
                }
            }
        }
        Err(e) => Err(DataflowError::Validation(format!(
            "MT202 COV FinancialInstitutionCreditTransferV08 deserialization failed: {}",
            e
        ))),
    }
}

fn handle_mt202_rejt_header(
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

fn handle_mt202_rejt_document(
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

fn handle_mt202_retn_header(
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

fn handle_mt202_retn_document(
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

// Handle MT205 Header - generates AppHdr XML for MT205 Corporate payments
fn handle_mt205_header(
    data: Value,
    message: &mut Message,
    output_field_name: &str,
) -> Result<(usize, Vec<Change>)> {
    use mx_message::header::bah_pacs_009_001_08::BusinessApplicationHeaderV02;

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
                    println!("MT205 Header XML serialization failed: {}", e);
                    Err(DataflowError::Validation(format!(
                        "MT205 Header XML serialization failed: {}",
                        e
                    )))
                }
            }
        }
        Err(e) => {
            println!("MT205 AppHdr deserialization failed: {}", e);
            Err(DataflowError::Validation(format!(
                "MT205 AppHdr deserialization failed: {}",
                e
            )))
        }
    }
}

// Handle MT205 Document - generates Document XML for MT205 Corporate payments
fn handle_mt205_document(
    data: Value,
    message: &mut Message,
    output_field_name: &str,
) -> Result<(usize, Vec<Change>)> {
    use mx_message::{
        app_document::Document, document::pacs_009_001_08::FinancialInstitutionCreditTransferV08,
    };

    // Extract FIToFICdtTrf from the data (MT205 uses pacs.009 like MT202)
    let fi_to_fi = data.get("FIToFICdtTrf").ok_or_else(|| {
        DataflowError::Validation("FIToFICdtTrf not found in MT205 document".to_string())
    })?;

    // Serialize using mx-message structures
    match serde_json::from_value::<FinancialInstitutionCreditTransferV08>(fi_to_fi.clone()) {
        Ok(pacs_data) => {
            let document = Document::FinancialInstitutionCreditTransferV08(Box::new(pacs_data));
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
                    println!("MT205 Document XML serialization failed: {}", e);
                    Err(DataflowError::Validation(format!(
                        "MT205 Document XML serialization failed: {}",
                        e
                    )))
                }
            }
        }
        Err(e) => Err(DataflowError::Validation(format!(
            "MT205 FinancialInstitutionCreditTransferV08 deserialization failed: {}",
            e
        ))),
    }
}

// Handle MT205 COV Header - generates AppHdr XML for MT205 Cover payments
fn handle_mt205_cov_header(
    data: Value,
    message: &mut Message,
    output_field_name: &str,
) -> Result<(usize, Vec<Change>)> {
    use mx_message::header::bah_pacs_009_001_08::BusinessApplicationHeaderV02;

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
                    println!("MT205 COV Header XML serialization failed: {}", e);
                    Err(DataflowError::Validation(format!(
                        "MT205 COV Header XML serialization failed: {}",
                        e
                    )))
                }
            }
        }
        Err(e) => {
            println!("MT205 COV AppHdr deserialization failed: {}", e);
            Err(DataflowError::Validation(format!(
                "MT205 COV AppHdr deserialization failed: {}",
                e
            )))
        }
    }
}

// Handle MT205 COV Document - generates Document XML for MT205 Cover payments
fn handle_mt205_cov_document(
    data: Value,
    message: &mut Message,
    output_field_name: &str,
) -> Result<(usize, Vec<Change>)> {
    use mx_message::{
        app_document::Document, document::pacs_009_001_08::FinancialInstitutionCreditTransferV08,
    };

    // Extract FIToFICdtTrf from the data (MT205 COV uses pacs.009 COVE)
    let fi_to_fi = data.get("FIToFICdtTrf").ok_or_else(|| {
        DataflowError::Validation("FIToFICdtTrf not found in MT205 COV document".to_string())
    })?;

    // Serialize using mx-message structures
    match serde_json::from_value::<FinancialInstitutionCreditTransferV08>(fi_to_fi.clone()) {
        Ok(pacs_data) => {
            let document = Document::FinancialInstitutionCreditTransferV08(Box::new(pacs_data));
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
                    println!("MT205 COV Document XML serialization failed: {}", e);
                    Err(DataflowError::Validation(format!(
                        "MT205 COV Document XML serialization failed: {}",
                        e
                    )))
                }
            }
        }
        Err(e) => Err(DataflowError::Validation(format!(
            "MT205 COV FinancialInstitutionCreditTransferV08 deserialization failed: {}",
            e
        ))),
    }
}

fn handle_mt205_rejt_header(
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
                    println!("MT205 REJT Header XML serialization failed: {}", e);
                    Err(DataflowError::Validation(format!(
                        "MT205 REJT Header XML serialization failed: {}",
                        e
                    )))
                }
            }
        }
        Err(e) => Err(DataflowError::Validation(format!(
            "MT205 REJT AppHdr deserialization failed: {}",
            e
        ))),
    }
}

fn handle_mt205_rejt_document(
    data: Value,
    message: &mut Message,
    output_field_name: &str,
) -> Result<(usize, Vec<Change>)> {
    use mx_message::{
        app_document::Document, document::pacs_002_001_10::FIToFIPaymentStatusReportV10,
    };

    // Extract FIToFIPaymentStatusReport from the data
    let fi_to_fi = data.get("FIToFIPmtStsRpt").ok_or_else(|| {
        DataflowError::Validation("FIToFIPmtStsRpt not found in MT205 REJT document".to_string())
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
                    println!("MT205 REJT Document XML serialization failed: {}", e);
                    Err(DataflowError::Validation(format!(
                        "MT205 REJT Document XML serialization failed: {}",
                        e
                    )))
                }
            }
        }
        Err(e) => Err(DataflowError::Validation(format!(
            "MT205 REJT FIToFIPaymentStatusReportV10 deserialization failed: {}",
            e
        ))),
    }
}

fn handle_mt205_retn_header(
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
                    println!("MT205 RETN Header XML serialization failed: {}", e);
                    Err(DataflowError::Validation(format!(
                        "MT205 RETN Header XML serialization failed: {}",
                        e
                    )))
                }
            }
        }
        Err(e) => {
            println!("MT205 RETN AppHdr deserialization failed: {}", e);
            Err(DataflowError::Validation(format!(
                "MT205 RETN AppHdr deserialization failed: {}",
                e
            )))
        }
    }
}

fn handle_mt205_retn_document(
    data: Value,
    message: &mut Message,
    output_field_name: &str,
) -> Result<(usize, Vec<Change>)> {
    use mx_message::{app_document::Document, document::pacs_004_001_09::PaymentReturnV09};

    // Extract PaymentReturn from the data
    let pmt_rtr = data.get("PmtRtr").ok_or_else(|| {
        DataflowError::Validation("PmtRtr not found in MT205 RETN document".to_string())
    })?;

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
                    println!("MT205 RETN Document XML serialization failed: {}", e);
                    Err(DataflowError::Validation(format!(
                        "MT205 RETN Document XML serialization failed: {}",
                        e
                    )))
                }
            }
        }
        Err(e) => Err(DataflowError::Validation(format!(
            "MT205 RETN PaymentReturnV09 deserialization failed: {}",
            e
        ))),
    }
}
