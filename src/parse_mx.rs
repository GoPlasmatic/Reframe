use crate::helper::Helper;
use async_trait::async_trait;
use dataflow_rs::engine::error::DataflowError;
use dataflow_rs::engine::{
    AsyncFunctionHandler,
    error::Result,
    message::{Change, Message},
};
use mx_message::document::*;
use mx_message::header::*;
use quick_xml::de::from_str;
use serde_json::{Value, json};
use tracing::{debug, error, info, instrument};

pub struct ParseMX;

#[async_trait]
impl AsyncFunctionHandler for ParseMX {
    #[instrument(skip(self, message, input))]
    async fn execute(&self, message: &mut Message, input: &Value) -> Result<(usize, Vec<Change>)> {
        debug!("Starting MX message parsing for reverse transformation");

        let input_field_name = input
            .get("input_field_name")
            .and_then(Value::as_str)
            .ok_or_else(|| DataflowError::Validation("Missing input_field_name".to_string()))?;

        let output_field_name = input
            .get("output_field_name")
            .and_then(Value::as_str)
            .ok_or_else(|| DataflowError::Validation("Missing output_field_name".to_string()))?;

        let payload = if input_field_name == "payload" {
            let raw_payload = message.payload.to_string();
            Helper::manual_unescape(&raw_payload)
        } else {
            message
                .data
                .get(input_field_name)
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    DataflowError::Validation(format!(
                        "Field {input_field_name} not found or not a string in message data for MX parsing"
                    ))
                })?
                .to_string()
        };

        let payload = Helper::manual_unescape(&payload);

        let document_xmlns = Self::extract_document_xmlns(&payload);
        let app_hdr_content = Self::extract_app_hdr_content(&payload);
        let document_content = Self::extract_document_content(&payload);

        let message_type = Self::extract_message_type(document_xmlns, app_hdr_content.clone())?;
        info!("Message type: {:?}", message_type);

        let app_hdr_content = app_hdr_content.ok_or_else(|| {
            DataflowError::Validation(
                "Failed to extract application header content from MX message".to_string(),
            )
        })?;
        let document_content = document_content.ok_or_else(|| {
            DataflowError::Validation(
                "Failed to extract document content from MX message".to_string(),
            )
        })?;

        let header = Self::parse_header(&message_type, &app_hdr_content)?;
        let document = Self::parse_document(&message_type, &document_content)?;

        let parsed_result = json!({
            "header": header,
            "document": document,
            "message_type": message_type.clone(),
        });

        // Store the parsed result in message data
        if let Some(data_obj) = message.data.as_object_mut() {
            data_obj.insert(output_field_name.to_string(), parsed_result.clone());
        } else {
            message.data = json!({
                output_field_name: parsed_result.clone()
            });
        }

        if let Some(data_obj) = message.metadata.as_object_mut() {
            data_obj.insert(
                output_field_name.to_string(),
                json!({
                    "message_type": message_type,
                }),
            );
        } else {
            message.metadata = json!({
                output_field_name.to_string(): {
                    "message_type": message_type,
                }
            });
        }

        Ok((
            200,
            vec![Change {
                path: format!("data.{output_field_name}"),
                old_value: Value::Null,
                new_value: parsed_result,
            }],
        ))
    }
}

impl ParseMX {
    pub fn extract_message_type(
        document_xmlns: Option<String>,
        app_hdr_content: Option<String>,
    ) -> Result<String> {
        // First try to get the message type from the AppHdr MsgDefIdr (more specific)
        let message_type = if let Some(app_hdr_content) = app_hdr_content {
            match Self::parse_header("", &app_hdr_content) {
                Ok(header) => match header.get("MsgDefIdr") {
                    Some(value) => {
                        let msg_def_string = value.to_string();
                        let msg_def = value.as_str().unwrap_or(&msg_def_string);
                        // Remove quotes if present
                        let msg_def = msg_def.trim_matches('"');
                        Helper::manual_unescape(msg_def)
                    }
                    None => {
                        // Fall back to xmlns if MsgDefIdr not found
                        if let Some(xmlns) = document_xmlns {
                            match xmlns.split(":").last() {
                                Some(message_type) => message_type.to_string(),
                                None => {
                                    return Err(DataflowError::Validation(
                                        "Message type not found".to_string(),
                                    ));
                                }
                            }
                        } else {
                            return Err(DataflowError::Validation(
                                "MsgDefIdr not found in header and no xmlns available".to_string(),
                            ));
                        }
                    }
                },
                Err(_) => {
                    // If header parsing fails, try xmlns
                    if let Some(xmlns) = document_xmlns {
                        match xmlns.split(":").last() {
                            Some(message_type) => message_type.to_string(),
                            None => {
                                return Err(DataflowError::Validation(
                                    "Message type not found".to_string(),
                                ));
                            }
                        }
                    } else {
                        return Err(DataflowError::Validation(
                            "Failed to parse header and no xmlns available".to_string(),
                        ));
                    }
                }
            }
        } else if let Some(xmlns) = document_xmlns {
            match xmlns.split(":").last() {
                Some(message_type) => message_type.to_string(),
                None => {
                    return Err(DataflowError::Validation(
                        "Message type not found".to_string(),
                    ));
                }
            }
        } else {
            return Err(DataflowError::Validation(
                "Message type not found".to_string(),
            ));
        };
        Ok(message_type)
    }

    /// Extract xmlns attribute from Document element
    pub fn extract_document_xmlns(xml: &str) -> Option<String> {
        // Find the Document element start tag
        if let Some(start) = xml.find("<Document") {
            // Find the end of the opening tag
            if let Some(end) = xml[start..].find(">") {
                let doc_tag = &xml[start..start + end + 1];

                // Look for xmlns attribute
                if let Some(xmlns_start) = doc_tag.find("xmlns=\"")
                    && let Some(xmlns_end) = doc_tag[xmlns_start + 7..].find("\"")
                {
                    return Some(doc_tag[xmlns_start + 7..xmlns_start + 7 + xmlns_end].to_string());
                }
            }
        }
        None
    }

    /// Extract AppHdr content including the wrapper element
    pub fn extract_app_hdr_content(xml: &str) -> Option<String> {
        if let Some(start) = xml.find("<AppHdr")
            && let Some(end) = xml.find("</AppHdr>")
        {
            let end_pos = end + "</AppHdr>".len();
            return Some(xml[start..end_pos].to_string());
        }
        None
    }

    /// Extract Document content including the wrapper element
    pub fn extract_document_content(xml: &str) -> Option<String> {
        // Find Document opening tag
        if let Some(start) = xml.find("<Document")
            && let Some(end) = xml.find("</Document>")
        {
            let end_pos = end + "</Document>".len();
            return Some(xml[start..end_pos].to_string());
        }
        None
    }

    pub fn parse_header(message_type: &str, app_hdr_content: &str) -> Result<Value> {
        match message_type {
            "pacs.008.001.08" => {
                let header = match from_str::<bah_pacs_008_001_08::BusinessApplicationHeaderV02>(
                    app_hdr_content,
                ) {
                    Ok(header) => header,
                    Err(e) => {
                        error!("Failed to parse header: {:?}", e);
                        return Err(DataflowError::Validation(format!(
                            "Failed to parse header: {e}"
                        )));
                    }
                };
                match serde_json::to_value(header) {
                    Ok(value) => Ok(value),
                    Err(e) => {
                        error!("Failed to convert header: {:?}", e);
                        Err(DataflowError::Validation(format!(
                            "Failed to convert header to value: {e}"
                        )))
                    }
                }
            }
            "pacs.004.001.09" => {
                let header = match from_str::<bah_pacs_004_001_09::BusinessApplicationHeaderV02>(
                    app_hdr_content,
                ) {
                    Ok(header) => header,
                    Err(e) => {
                        error!("Failed to parse header: {:?}", e);
                        return Err(DataflowError::Validation(format!(
                            "Failed to parse header: {e}"
                        )));
                    }
                };
                match serde_json::to_value(header) {
                    Ok(value) => Ok(value),
                    Err(e) => {
                        error!("Failed to convert header: {:?}", e);
                        Err(DataflowError::Validation(format!(
                            "Failed to convert header to value: {e}"
                        )))
                    }
                }
            }
            "pacs.009.001.08" => {
                let header = match from_str::<bah_pacs_009_001_08::BusinessApplicationHeaderV02>(
                    app_hdr_content,
                ) {
                    Ok(header) => header,
                    Err(e) => {
                        error!("Failed to parse header: {:?}", e);
                        return Err(DataflowError::Validation(format!(
                            "Failed to parse header: {e}"
                        )));
                    }
                };
                match serde_json::to_value(header) {
                    Ok(value) => Ok(value),
                    Err(e) => {
                        error!("Failed to convert header: {:?}", e);
                        Err(DataflowError::Validation(format!(
                            "Failed to convert header to value: {e}"
                        )))
                    }
                }
            }
            "pacs.002.001.10" => {
                let header = match from_str::<bah_pacs_002_001_10::BusinessApplicationHeaderV02>(
                    app_hdr_content,
                ) {
                    Ok(header) => header,
                    Err(e) => {
                        error!("Failed to parse header: {:?}", e);
                        return Err(DataflowError::Validation(format!(
                            "Failed to parse pacs.002 header: {e}"
                        )));
                    }
                };
                match serde_json::to_value(header) {
                    Ok(value) => Ok(value),
                    Err(e) => {
                        error!("Failed to convert header: {:?}", e);
                        Err(DataflowError::Validation(format!(
                            "Failed to convert header to value: {e}"
                        )))
                    }
                }
            }
            "camt.107.001.01" => {
                let header = match from_str::<bah_camt_107_001_01::BusinessApplicationHeaderV02>(
                    app_hdr_content,
                ) {
                    Ok(header) => header,
                    Err(e) => {
                        error!("Failed to parse header: {:?}", e);
                        return Err(DataflowError::Validation(format!(
                            "Failed to parse camt.107 header: {e}"
                        )));
                    }
                };
                match serde_json::to_value(header) {
                    Ok(value) => Ok(value),
                    Err(e) => {
                        error!("Failed to convert header: {:?}", e);
                        Err(DataflowError::Validation(format!(
                            "Failed to convert header to value: {e}"
                        )))
                    }
                }
            }
            "camt.108.001.01" => {
                let header = match from_str::<bah_camt_108_001_01::BusinessApplicationHeaderV02>(
                    app_hdr_content,
                ) {
                    Ok(header) => header,
                    Err(e) => {
                        error!("Failed to parse header: {:?}", e);
                        return Err(DataflowError::Validation(format!(
                            "Failed to parse camt.108 header: {e}"
                        )));
                    }
                };
                match serde_json::to_value(header) {
                    Ok(value) => Ok(value),
                    Err(e) => {
                        error!("Failed to convert header: {:?}", e);
                        Err(DataflowError::Validation(format!(
                            "Failed to convert header to value: {e}"
                        )))
                    }
                }
            }
            "camt.109.001.01" => {
                let header = match from_str::<bah_camt_109_001_01::BusinessApplicationHeaderV02>(
                    app_hdr_content,
                ) {
                    Ok(header) => header,
                    Err(e) => {
                        error!("Failed to parse header: {:?}", e);
                        return Err(DataflowError::Validation(format!(
                            "Failed to parse camt.109 header: {e}"
                        )));
                    }
                };
                match serde_json::to_value(header) {
                    Ok(value) => Ok(value),
                    Err(e) => {
                        error!("Failed to convert header: {:?}", e);
                        Err(DataflowError::Validation(format!(
                            "Failed to convert header to value: {e}"
                        )))
                    }
                }
            }
            "camt.052.001.08" => {
                let header = match from_str::<bah_camt_052_001_08::BusinessApplicationHeaderV02>(
                    app_hdr_content,
                ) {
                    Ok(header) => header,
                    Err(e) => {
                        error!("Failed to parse header: {:?}", e);
                        return Err(DataflowError::Validation(format!(
                            "Failed to parse camt.052 header: {e}"
                        )));
                    }
                };
                match serde_json::to_value(header) {
                    Ok(value) => Ok(value),
                    Err(e) => {
                        error!("Failed to convert header: {:?}", e);
                        Err(DataflowError::Validation(format!(
                            "Failed to convert header to value: {e}"
                        )))
                    }
                }
            }
            "camt.053.001.08" => {
                let header = match from_str::<bah_camt_053_001_08::BusinessApplicationHeaderV02>(
                    app_hdr_content,
                ) {
                    Ok(header) => header,
                    Err(e) => {
                        error!("Failed to parse header: {:?}", e);
                        return Err(DataflowError::Validation(format!(
                            "Failed to parse camt.053 header: {e}"
                        )));
                    }
                };
                match serde_json::to_value(header) {
                    Ok(value) => Ok(value),
                    Err(e) => {
                        error!("Failed to convert header: {:?}", e);
                        Err(DataflowError::Validation(format!(
                            "Failed to convert header to value: {e}"
                        )))
                    }
                }
            }
            "camt.029.001.09" => {
                let header = match from_str::<bah_camt_029_001::BusinessApplicationHeaderV02>(
                    app_hdr_content,
                ) {
                    Ok(header) => header,
                    Err(e) => {
                        error!("Failed to parse header: {:?}", e);
                        return Err(DataflowError::Validation(format!(
                            "Failed to parse camt.029 header: {e}"
                        )));
                    }
                };
                match serde_json::to_value(header) {
                    Ok(value) => Ok(value),
                    Err(e) => {
                        error!("Failed to convert header: {:?}", e);
                        Err(DataflowError::Validation(format!(
                            "Failed to convert header to value: {e}"
                        )))
                    }
                }
            }
            "camt.056.001.08" => {
                let header = match from_str::<bah_camt_056_001_08::BusinessApplicationHeaderV02>(
                    app_hdr_content,
                ) {
                    Ok(header) => header,
                    Err(e) => {
                        error!("Failed to parse header: {:?}", e);
                        return Err(DataflowError::Validation(format!(
                            "Failed to parse camt.056 header: {e}"
                        )));
                    }
                };
                match serde_json::to_value(header) {
                    Ok(value) => Ok(value),
                    Err(e) => {
                        error!("Failed to convert header: {:?}", e);
                        Err(DataflowError::Validation(format!(
                            "Failed to convert header to value: {e}"
                        )))
                    }
                }
            }
            "camt.057.001.06" => {
                let header = match from_str::<bah_camt_057_001_06::BusinessApplicationHeaderV02>(
                    app_hdr_content,
                ) {
                    Ok(header) => header,
                    Err(e) => {
                        error!("Failed to parse header: {:?}", e);
                        return Err(DataflowError::Validation(format!(
                            "Failed to parse camt.057 header: {e}"
                        )));
                    }
                };
                match serde_json::to_value(header) {
                    Ok(value) => Ok(value),
                    Err(e) => {
                        error!("Failed to convert header: {:?}", e);
                        Err(DataflowError::Validation(format!(
                            "Failed to convert header to value: {e}"
                        )))
                    }
                }
            }
            "camt.058.001.08" => {
                let header = match from_str::<bah_camt_058_001_08::BusinessApplicationHeaderV02>(
                    app_hdr_content,
                ) {
                    Ok(header) => header,
                    Err(e) => {
                        error!("Failed to parse header: {:?}", e);
                        return Err(DataflowError::Validation(format!(
                            "Failed to parse camt.058 header: {e}"
                        )));
                    }
                };
                match serde_json::to_value(header) {
                    Ok(value) => Ok(value),
                    Err(e) => {
                        error!("Failed to convert header: {:?}", e);
                        Err(DataflowError::Validation(format!(
                            "Failed to convert header to value: {e}"
                        )))
                    }
                }
            }
            "camt.105.001.02" => {
                let header = match from_str::<bah_camt_105_001_02::BusinessApplicationHeaderV02>(
                    app_hdr_content,
                ) {
                    Ok(header) => header,
                    Err(e) => {
                        error!("Failed to parse camt.105 header: {:?}", e);
                        return Err(DataflowError::Validation(format!(
                            "Failed to parse camt.105 header: {e}"
                        )));
                    }
                };
                match serde_json::to_value(header) {
                    Ok(value) => Ok(value),
                    Err(e) => {
                        error!("Failed to convert header: {:?}", e);
                        Err(DataflowError::Validation(format!(
                            "Failed to convert header to value: {e}"
                        )))
                    }
                }
            }
            "admi.024.001.01" => {
                let header = match from_str::<bah_admi_024_001_01::BusinessApplicationHeaderV02>(
                    app_hdr_content,
                ) {
                    Ok(header) => header,
                    Err(e) => {
                        error!("Failed to parse admi.024 header: {:?}", e);
                        return Err(DataflowError::Validation(format!(
                            "Failed to parse admi.024 header: {e}"
                        )));
                    }
                };
                match serde_json::to_value(header) {
                    Ok(value) => Ok(value),
                    Err(e) => {
                        error!("Failed to convert header: {:?}", e);
                        Err(DataflowError::Validation(format!(
                            "Failed to convert header to value: {e}"
                        )))
                    }
                }
            }
            _ => {
                let header = match from_str::<bah_pacs_008_001_08::BusinessApplicationHeaderV02>(
                    app_hdr_content,
                ) {
                    Ok(header) => header,
                    Err(e) => {
                        return Err(DataflowError::Validation(format!(
                            "Failed to parse header: {e}"
                        )));
                    }
                };
                match serde_json::to_value(header) {
                    Ok(value) => Ok(value),
                    Err(e) => {
                        error!("Failed to convert header: {:?}", e);
                        Err(DataflowError::Validation(format!(
                            "Failed to convert header to value: {e}"
                        )))
                    }
                }
            }
        }
    }

    pub fn parse_document(message_type: &str, document_content: &str) -> Result<Value> {
        info!("Parsing document for message type: {:?}", message_type);
        match message_type {
            "pacs.008.001.08" => {
                let document = match from_str::<pacs_008_001_08::FIToFICustomerCreditTransferV08>(
                    document_content,
                ) {
                    Ok(document) => document,
                    Err(e) => {
                        error!("Failed to parse document: {:?}", e);
                        return Err(DataflowError::Validation(format!(
                            "Failed to parse document: {e}"
                        )));
                    }
                };
                match serde_json::to_value(document) {
                    Ok(value) => Ok(value),
                    Err(e) => Err(DataflowError::Validation(format!(
                        "Failed to convert document to value: {e}"
                    ))),
                }
            }
            "pacs.004.001.09" => {
                let document = match from_str::<pacs_004_001_09::PaymentReturnV09>(document_content)
                {
                    Ok(document) => document,
                    Err(e) => {
                        error!("Failed to parse document: {:?}", e);
                        return Err(DataflowError::Validation(format!(
                            "Failed to parse document: {e}"
                        )));
                    }
                };
                match serde_json::to_value(document) {
                    Ok(value) => Ok(value),
                    Err(e) => Err(DataflowError::Validation(format!(
                        "Failed to convert document to value: {e}"
                    ))),
                }
            }
            "pacs.009.001.08" => {
                let document = match from_str::<
                    pacs_009_001_08::FinancialInstitutionCreditTransferV08,
                >(document_content)
                {
                    Ok(document) => document,
                    Err(e) => {
                        error!("Failed to parse document: {:?}", e);
                        return Err(DataflowError::Validation(format!(
                            "Failed to parse document: {e}"
                        )));
                    }
                };
                match serde_json::to_value(document) {
                    Ok(value) => Ok(value),
                    Err(e) => Err(DataflowError::Validation(format!(
                        "Failed to convert document to value: {e}"
                    ))),
                }
            }
            "pacs.002.001.10" => {
                let document = match from_str::<pacs_002_001_10::FIToFIPaymentStatusReportV10>(
                    document_content,
                ) {
                    Ok(document) => document,
                    Err(e) => {
                        error!("Failed to parse pacs.002 document: {:?}", e);
                        return Err(DataflowError::Validation(format!(
                            "Failed to parse pacs.002 document: {e}"
                        )));
                    }
                };
                match serde_json::to_value(document) {
                    Ok(value) => Ok(value),
                    Err(e) => Err(DataflowError::Validation(format!(
                        "Failed to convert pacs.002 document to value: {e}"
                    ))),
                }
            }
            "camt.107.001.01" => {
                let document = match from_str::<camt_107_001_01::ChequePresentmentNotificationV01>(
                    document_content,
                ) {
                    Ok(document) => document,
                    Err(e) => {
                        error!("Failed to parse camt.107 document: {:?}", e);
                        return Err(DataflowError::Validation(format!(
                            "Failed to parse camt.107 document: {e}"
                        )));
                    }
                };
                match serde_json::to_value(document) {
                    Ok(value) => Ok(value),
                    Err(e) => Err(DataflowError::Validation(format!(
                        "Failed to convert camt.107 document to value: {e}"
                    ))),
                }
            }
            "camt.108.001.01" => {
                let document = match from_str::<camt_108_001_01::ChequeCancellationOrStopRequestV01>(
                    document_content,
                ) {
                    Ok(document) => document,
                    Err(e) => {
                        error!("Failed to parse camt.108 document: {:?}", e);
                        return Err(DataflowError::Validation(format!(
                            "Failed to parse camt.108 document: {e}"
                        )));
                    }
                };
                match serde_json::to_value(document) {
                    Ok(value) => Ok(value),
                    Err(e) => Err(DataflowError::Validation(format!(
                        "Failed to convert camt.108 document to value: {e}"
                    ))),
                }
            }
            "camt.109.001.01" => {
                let document = match from_str::<camt_109_001_01::ChequeCancellationOrStopReportV01>(
                    document_content,
                ) {
                    Ok(document) => document,
                    Err(e) => {
                        error!("Failed to parse camt.109 document: {:?}", e);
                        return Err(DataflowError::Validation(format!(
                            "Failed to parse camt.109 document: {e}"
                        )));
                    }
                };
                match serde_json::to_value(document) {
                    Ok(value) => Ok(value),
                    Err(e) => Err(DataflowError::Validation(format!(
                        "Failed to convert camt.109 document to value: {e}"
                    ))),
                }
            }
            "camt.052.001.08" => {
                let document = match from_str::<camt_052_001_08::BankToCustomerAccountReportV08>(
                    document_content,
                ) {
                    Ok(document) => document,
                    Err(e) => {
                        error!("Failed to parse camt.052 document: {:?}", e);
                        error!(
                            "Document content sample (first 1000 chars): {}",
                            &document_content[..document_content.len().min(1000)]
                        );

                        // Try to provide more specific error location information
                        let error_msg = Self::analyze_xml_parsing_error(
                            "camt.052",
                            &e.to_string(),
                            document_content,
                        );
                        return Err(DataflowError::Validation(format!(
                            "Failed to parse camt.052 document: {error_msg}"
                        )));
                    }
                };
                match serde_json::to_value(document) {
                    Ok(value) => Ok(value),
                    Err(e) => Err(DataflowError::Validation(format!(
                        "Failed to convert camt.052 document to value: {e}"
                    ))),
                }
            }
            "camt.053.001.08" => {
                let document =
                    match from_str::<camt_053_001_08::BankToCustomerStatementV08>(document_content)
                    {
                        Ok(document) => document,
                        Err(e) => {
                            error!("Failed to parse camt.053 document: {:?}", e);
                            error!(
                                "Document content sample (first 1000 chars): {}",
                                &document_content[..document_content.len().min(1000)]
                            );

                            // Try to provide more specific error location information
                            let error_msg = Self::analyze_xml_parsing_error(
                                "camt.053",
                                &e.to_string(),
                                document_content,
                            );
                            return Err(DataflowError::Validation(format!(
                                "Failed to parse camt.053 document: {error_msg}"
                            )));
                        }
                    };
                match serde_json::to_value(document) {
                    Ok(value) => Ok(value),
                    Err(e) => Err(DataflowError::Validation(format!(
                        "Failed to convert camt.053 document to value: {e}"
                    ))),
                }
            }
            "admi.024.001.01" => {
                let document = match from_str::<admi_024_001_01::NotificationOfCorrespondenceV01>(
                    document_content,
                ) {
                    Ok(document) => document,
                    Err(e) => {
                        error!("Failed to parse admi.024 document: {:?}", e);
                        return Err(DataflowError::Validation(format!(
                            "Failed to parse admi.024 document: {e}"
                        )));
                    }
                };
                match serde_json::to_value(document) {
                    Ok(value) => Ok(value),
                    Err(e) => Err(DataflowError::Validation(format!(
                        "Failed to convert admi.024 document to value: {e}"
                    ))),
                }
            }
            "camt.029.001.09" => {
                let document = match from_str::<camt_029_001_09::ResolutionOfInvestigationV09>(
                    document_content,
                ) {
                    Ok(document) => document,
                    Err(e) => {
                        error!("Failed to parse camt.029 document: {:?}", e);
                        return Err(DataflowError::Validation(format!(
                            "Failed to parse camt.029 document: {e}"
                        )));
                    }
                };
                match serde_json::to_value(document) {
                    Ok(value) => Ok(value),
                    Err(e) => Err(DataflowError::Validation(format!(
                        "Failed to convert camt.029 document to value: {e}"
                    ))),
                }
            }
            "camt.056.001.08" => {
                let document = match from_str::<camt_056_001_08::FIToFIPaymentCancellationRequestV08>(
                    document_content,
                ) {
                    Ok(document) => document,
                    Err(e) => {
                        error!("Failed to parse camt.056 document: {:?}", e);
                        return Err(DataflowError::Validation(format!(
                            "Failed to parse camt.056 document: {e}"
                        )));
                    }
                };
                match serde_json::to_value(document) {
                    Ok(value) => Ok(value),
                    Err(e) => Err(DataflowError::Validation(format!(
                        "Failed to convert camt.056 document to value: {e}"
                    ))),
                }
            }
            "camt.057.001.06" => {
                // Extract the NtfctnToRcv content from the Document wrapper
                let ntfctn_content = if let Some(start) = document_content.find("<NtfctnToRcv")
                    && let Some(end) = document_content.find("</NtfctnToRcv>")
                {
                    let end_pos = end + "</NtfctnToRcv>".len();
                    &document_content[start..end_pos]
                } else {
                    document_content
                };

                let document =
                    match from_str::<camt_057_001_06::NotificationToReceiveV06>(ntfctn_content) {
                        Ok(document) => document,
                        Err(e) => {
                            error!("Failed to parse camt.057 document: {:?}", e);
                            return Err(DataflowError::Validation(format!(
                                "Failed to parse camt.057 document: {e}"
                            )));
                        }
                    };
                match serde_json::to_value(document) {
                    Ok(value) => Ok(value),
                    Err(e) => Err(DataflowError::Validation(format!(
                        "Failed to convert camt.057 document to value: {e}"
                    ))),
                }
            }
            "camt.058.001.08" => {
                // Extract the NtfctnToRcvCxlAdvc content from the Document wrapper
                let ntfctn_content = if let Some(start) =
                    document_content.find("<NtfctnToRcvCxlAdvc")
                    && let Some(end) = document_content.find("</NtfctnToRcvCxlAdvc>")
                {
                    let end_pos = end + "</NtfctnToRcvCxlAdvc>".len();
                    &document_content[start..end_pos]
                } else {
                    document_content
                };

                let document = match from_str::<
                    camt_058_001_08::NotificationToReceiveCancellationAdviceV08,
                >(ntfctn_content)
                {
                    Ok(document) => document,
                    Err(e) => {
                        error!("Failed to parse camt.058 document: {:?}", e);
                        return Err(DataflowError::Validation(format!(
                            "Failed to parse camt.058 document: {e}"
                        )));
                    }
                };
                match serde_json::to_value(document) {
                    Ok(value) => Ok(value),
                    Err(e) => Err(DataflowError::Validation(format!(
                        "Failed to convert camt.058 document to value: {e}"
                    ))),
                }
            }
            "camt.105.001.02" => {
                // Extract the ChrgsPaymtNtfctn content from the Document wrapper
                let chrgs_content = if let Some(start) = document_content.find("<ChrgsPaymtNtfctn")
                    && let Some(end) = document_content.find("</ChrgsPaymtNtfctn>")
                {
                    let end_pos = end + "</ChrgsPaymtNtfctn>".len();
                    &document_content[start..end_pos]
                } else {
                    document_content
                };

                let document =
                    match from_str::<camt_105_001_02::ChargesPaymentNotificationV02>(chrgs_content)
                    {
                        Ok(document) => document,
                        Err(e) => {
                            error!("Failed to parse camt.105 document: {:?}", e);
                            return Err(DataflowError::Validation(format!(
                                "Failed to parse camt.105 document: {e}"
                            )));
                        }
                    };
                match serde_json::to_value(document) {
                    Ok(value) => Ok(value),
                    Err(e) => Err(DataflowError::Validation(format!(
                        "Failed to convert camt.105 document to value: {e}"
                    ))),
                }
            }
            _ => Err(DataflowError::Validation(
                "Unknown message type".to_string(),
            )),
        }
    }

    /// Analyze XML parsing error and provide detailed location information
    fn analyze_xml_parsing_error(message_type: &str, error_msg: &str, xml_content: &str) -> String {
        let mut analysis = format!("Error in {message_type}: {error_msg}");

        // Check for missing field errors
        if error_msg.contains("missing field")
            && let Some(field_start) = error_msg.find("missing field `")
            && let Some(field_end) = error_msg[field_start + 15..].find("`")
        {
            let missing_field = &error_msg[field_start + 15..field_start + 15 + field_end];
            analysis.push_str(&format!(
                "\n\nMissing field analysis for '{missing_field}':"
            ));

            // Search for structural context where this field should be
            analysis.push_str(&Self::find_missing_field_context(
                missing_field,
                xml_content,
            ));
        }

        // Check for XML structure issues
        if error_msg.contains("invalid type") || error_msg.contains("expected") {
            analysis.push_str("\n\nXML Structure Analysis:");
            analysis.push_str(&Self::analyze_xml_structure(xml_content));
        }

        analysis
    }

    /// Find context around where a missing field should be located
    fn find_missing_field_context(missing_field: &str, xml_content: &str) -> String {
        let mut context = String::new();

        match missing_field {
            "Amt" => {
                context.push_str("\n- Searching for elements that should contain 'Amt' fields...");

                // Check for elements that typically require Amt fields
                let amt_elements = [
                    "<Bal>",
                    "<Ntry>",
                    "<TtlNetNtry>",
                    "<AmtDtls>",
                    "<TxDtls>",
                    "<InstdAmt>",
                    "<TxAmt>",
                    "<IntrBkSttlmAmt>",
                    "<ChrgBr>",
                    "<ExchgRate>",
                ];
                for element in amt_elements {
                    let element_name = element.trim_start_matches('<').trim_end_matches('>');
                    let mut element_count = 0;
                    let mut start_pos = 0;

                    while let Some(pos) = xml_content[start_pos..].find(element) {
                        element_count += 1;
                        let absolute_pos = start_pos + pos;

                        // Find the end of this element
                        let end_tag = format!("</{element_name}>");

                        if let Some(end_pos) = xml_content[absolute_pos..].find(&end_tag) {
                            let element_content =
                                &xml_content[absolute_pos..absolute_pos + end_pos + end_tag.len()];
                            let has_amt = element_content.contains("<Amt");

                            context.push_str(&format!(
                                "\n  {element_name} #{element_count}: Has Amt field: {has_amt}"
                            ));

                            // If no Amt field found, show the element content
                            if !has_amt {
                                let preview = if element_content.len() > 200 {
                                    format!("{}...", &element_content[..200])
                                } else {
                                    element_content.to_string()
                                };
                                context.push_str(&format!("\n    Content: {preview}"));
                            }

                            start_pos = absolute_pos + end_pos + end_tag.len();
                        } else {
                            start_pos = absolute_pos + element.len();
                        }
                    }

                    if element_count > 0 {
                        context.push_str(&format!(
                            "\n- Found {element_count} {element_name} elements"
                        ));
                    }
                }
            }
            _ => {
                context.push_str(&format!(
                    "\n- Searching for '{missing_field}' field in XML..."
                ));
                if xml_content.contains(&format!("<{missing_field}>")) {
                    context.push_str(" [FOUND as element]");
                } else if xml_content.contains(&format!("{missing_field}=")) {
                    context.push_str(" [FOUND as attribute]");
                } else {
                    context.push_str(" [NOT FOUND]");
                }
            }
        }

        context
    }

    /// Analyze overall XML structure for common issues
    fn analyze_xml_structure(xml_content: &str) -> String {
        let mut analysis = String::new();

        // Count major elements
        let elements_to_count = [
            ("GrpHdr", "<GrpHdr>"),
            ("Rpt", "<Rpt>"),
            ("Bal", "<Bal>"),
            ("Ntry", "<Ntry>"),
            ("TxsSummry", "<TxsSummry>"),
            ("TtlNtries", "<TtlNtries>"),
            ("TtlCdtNtries", "<TtlCdtNtries>"),
            ("TtlDbtNtries", "<TtlDbtNtries>"),
        ];

        for (name, tag) in elements_to_count {
            let count = xml_content.matches(tag).count();
            analysis.push_str(&format!("\n- {name} elements: {count}"));
        }

        // Check for common XML issues
        let unclosed_tags = xml_content.matches('<').count()
            - xml_content.matches("</").count()
            - xml_content.matches("/>").count();
        if unclosed_tags != 1 {
            // Should be 1 for the root element
            analysis.push_str(&format!(
                "\n- WARNING: Potential unclosed tags detected (difference: {unclosed_tags})"
            ));
        }

        // Check for required attributes
        let amt_with_ccy = xml_content.matches("<Amt Ccy=").count();
        let amt_without_ccy = xml_content.matches("<Amt>").count();
        analysis.push_str(&format!(
            "\n- Amt elements with Ccy attribute: {amt_with_ccy}"
        ));
        analysis.push_str(&format!(
            "\n- Amt elements without Ccy attribute: {amt_without_ccy}"
        ));

        analysis
    }
}
