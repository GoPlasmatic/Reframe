use async_trait::async_trait;
use dataflow_rs::engine::error::DataflowError;
use dataflow_rs::engine::{
    error::Result,
    message::{Change, Message},
    AsyncFunctionHandler,
};
use mx_message::{
    camt_029_001_09::ResolutionOfInvestigationV09,
    camt_056_001_08::FIToFIPaymentCancellationRequestV08,
    camt_057_001_06::NotificationToReceiveV06, document::Document,
    pacs_008_001_08::FIToFICustomerCreditTransferV08,
    pacs_009_001_08::FinancialInstitutionCreditTransferV08,
};
use quick_xml::se::to_string as xml_to_string;
use serde_json::Value;

pub struct PublishFunction;

#[async_trait]
impl AsyncFunctionHandler for PublishFunction {
    async fn execute(&self, message: &mut Message, input: &Value) -> Result<(usize, Vec<Change>)> {
        let output_format = input
            .get("output_format")
            .and_then(Value::as_str)
            .ok_or_else(|| DataflowError::Validation("Missing output_format".to_string()))?;

        let input_field_name = input
            .get("input_field_name")
            .and_then(Value::as_str)
            .ok_or_else(|| DataflowError::Validation("Missing input_field_name".to_string()))?;

        let data = message.data.get(input_field_name).ok_or_else(|| {
            DataflowError::Validation(format!(
                "Field {} not found in message data",
                input_field_name
            ))
        })?;

        match output_format {
            "pacs.008.001.08" => {
                // Handle pacs.008.001.08 for MT103, MT103+, MT102, MT202COV

                // Check if we have multiple FIToFI messages (for MT102)
                if let Some(fi_to_fi_array) = data.get("FIToFICstmrCdtTrf_Multiple") {
                    if let Some(messages) = fi_to_fi_array.as_array() {
                        let messages_clone = messages.clone();
                        return handle_multiple_pacs008(&messages_clone, message);
                    }
                }

                // Handle single FIToFI message (MT103, MT103+, MT202COV)
                if let Some(fi_to_fi) = data.get("FIToFICstmrCdtTrf") {
                    match serde_json::from_value::<FIToFICustomerCreditTransferV08>(
                        fi_to_fi.clone(),
                    ) {
                        Ok(pacs_data) => {
                            let document =
                                Document::FIToFICustomerCreditTransferV08(Box::new(pacs_data));
                            serialize_and_store_single(document, message)
                        }
                        Err(e) => {
                            println!(
                                "FIToFICustomerCreditTransferV08 deserialization failed: {}",
                                e
                            );
                            Err(DataflowError::Validation(format!(
                                "FIToFICustomerCreditTransferV08 deserialization failed: {}",
                                e
                            )))
                        }
                    }
                } else {
                    Err(DataflowError::Validation(
                        "FIToFICustomerCreditTransferV08 not found in data".to_string(),
                    ))
                }
            }

            "pacs.009.001.08" => {
                // Handle pacs.009.001.08 for MT202 and financial institution credit transfers
                if let Some(fi_credit) = data.get("FinInstnCdtTrf") {
                    match serde_json::from_value::<FinancialInstitutionCreditTransferV08>(
                        fi_credit.clone(),
                    ) {
                        Ok(pacs_data) => {
                            let document = Document::FinancialInstitutionCreditTransferV08(
                                Box::new(pacs_data),
                            );
                            serialize_and_store_single(document, message)
                        }
                        Err(e) => {
                            println!(
                                "FinancialInstitutionCreditTransferV08 deserialization failed: {}",
                                e
                            );
                            Err(DataflowError::Validation(format!(
                                "FinancialInstitutionCreditTransferV08 deserialization failed: {}",
                                e
                            )))
                        }
                    }
                } else {
                    Err(DataflowError::Validation(
                        "FinancialInstitutionCreditTransferV08 not found in data".to_string(),
                    ))
                }
            }

            "camt.057.001.06" => {
                // Handle camt.057.001.06 for MT210 (Notice to Receive)
                if let Some(notification) = data.get("NtfctnToRcv") {
                    match serde_json::from_value::<NotificationToReceiveV06>(notification.clone()) {
                        Ok(camt_data) => {
                            let document = Document::NotificationToReceiveV06(Box::new(camt_data));
                            serialize_and_store_single(document, message)
                        }
                        Err(e) => {
                            println!("NotificationToReceiveV06 deserialization failed: {}", e);
                            Err(DataflowError::Validation(format!(
                                "NotificationToReceiveV06 deserialization failed: {}",
                                e
                            )))
                        }
                    }
                } else {
                    Err(DataflowError::Validation(
                        "NotificationToReceiveV06 not found in data".to_string(),
                    ))
                }
            }

            "camt.056.001.08" => {
                // Handle camt.056.001.08 for MT192 (Payment Cancellation Request)
                if let Some(cancellation) = data.get("FIToFIPmtCxlReq") {
                    match serde_json::from_value::<FIToFIPaymentCancellationRequestV08>(
                        cancellation.clone(),
                    ) {
                        Ok(camt_data) => {
                            let document =
                                Document::FIToFIPaymentCancellationRequestV08(Box::new(camt_data));
                            serialize_and_store_single(document, message)
                        }
                        Err(e) => {
                            println!(
                                "FIToFIPaymentCancellationRequestV08 deserialization failed: {}",
                                e
                            );
                            Err(DataflowError::Validation(format!(
                                "FIToFIPaymentCancellationRequestV08 deserialization failed: {}",
                                e
                            )))
                        }
                    }
                } else {
                    Err(DataflowError::Validation(
                        "FIToFIPaymentCancellationRequestV08 not found in data".to_string(),
                    ))
                }
            }

            "camt.029.001.09" => {
                // Handle camt.029.001.09 for MT196 (Resolution of Investigation)
                if let Some(resolution) = data.get("RsltnOfInvstgtn") {
                    match serde_json::from_value::<ResolutionOfInvestigationV09>(resolution.clone())
                    {
                        Ok(camt_data) => {
                            let document =
                                Document::ResolutionOfInvestigationV09(Box::new(camt_data));
                            serialize_and_store_single(document, message)
                        }
                        Err(e) => {
                            println!("ResolutionOfInvestigationV09 deserialization failed: {}", e);
                            Err(DataflowError::Validation(format!(
                                "ResolutionOfInvestigationV09 deserialization failed: {}",
                                e
                            )))
                        }
                    }
                } else {
                    Err(DataflowError::Validation(
                        "ResolutionOfInvestigationV09 not found in data".to_string(),
                    ))
                }
            }

            _ => Err(DataflowError::Validation(format!(
                "Unsupported output format: {}",
                output_format
            ))),
        }
    }
}

// Handle multiple pacs.008 messages for MT102
fn handle_multiple_pacs008(
    messages: &[Value],
    message: &mut Message,
) -> Result<(usize, Vec<Change>)> {
    let mut xml_results = Vec::new();
    let mut errors = Vec::new();

    for (index, fi_to_fi) in messages.iter().enumerate() {
        match serde_json::from_value::<FIToFICustomerCreditTransferV08>(fi_to_fi.clone()) {
            Ok(pacs_data) => {
                let document = Document::FIToFICustomerCreditTransferV08(Box::new(pacs_data));
                match xml_to_string(&document) {
                    Ok(xml_string) => {
                        xml_results.push(Value::String(xml_string));
                    }
                    Err(e) => {
                        let error_msg =
                            format!("XML serialization failed for message {}: {}", index, e);
                        errors.push(error_msg);
                    }
                }
            }
            Err(e) => {
                let error_msg = format!(
                    "FIToFICustomerCreditTransferV08 deserialization failed for message {}: {}",
                    index, e
                );
                errors.push(error_msg);
            }
        }
    }

    if !errors.is_empty() {
        return Err(DataflowError::Validation(format!(
            "Multiple errors occurred: {}",
            errors.join("; ")
        )));
    }

    if xml_results.is_empty() {
        return Err(DataflowError::Validation(
            "No valid XML results generated".to_string(),
        ));
    }

    // Store multiple results
    message.data["results"] = Value::Array(xml_results.clone());

    Ok((
        200,
        vec![Change {
            path: "data.results".to_string(),
            old_value: Value::Null,
            new_value: Value::Array(xml_results),
        }],
    ))
}

// Helper function to serialize and store a single document (existing functionality)
fn serialize_and_store_single(
    document: Document,
    message: &mut Message,
) -> Result<(usize, Vec<Change>)> {
    match xml_to_string(&document) {
        Ok(xml_string) => {
            message.data["result"] = Value::String(xml_string);

            Ok((
                200,
                vec![Change {
                    path: "data.result".to_string(),
                    old_value: Value::Null,
                    new_value: message.data["result"].clone(),
                }],
            ))
        }
        Err(e) => {
            println!("XML serialization failed: {}", e);
            Err(DataflowError::Validation(format!(
                "XML serialization failed: {}",
                e
            )))
        }
    }
}
