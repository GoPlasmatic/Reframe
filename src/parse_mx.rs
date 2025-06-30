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
                .unwrap_or("")
                .to_string()
        };

        let payload = Helper::manual_unescape(&payload);

        let document_xmlns = Self::extract_document_xmlns(&payload);
        let app_hdr_content = Self::extract_app_hdr_content(&payload);
        let document_content = Self::extract_document_content(&payload);

        let message_type = Self::extract_message_type(document_xmlns, app_hdr_content.clone())?;
        info!("Message type: {:?}", message_type);

        let header = Self::parse_header(&message_type, &app_hdr_content.unwrap_or("".to_string()))
            .unwrap_or(Value::Null);
        let document =
            Self::parse_document(&message_type, &document_content.unwrap_or("".to_string()))
                .unwrap_or(Value::Null);

        let parsed_result = json!({
            "header": header,
            "document": document,
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
    fn extract_message_type(
        document_xmlns: Option<String>,
        app_hdr_content: Option<String>,
    ) -> Result<String> {
        let message_type = if let Some(xmlns) = document_xmlns {
            match xmlns.split(":").last() {
                Some(message_type) => message_type.to_string(),
                None => {
                    return Err(DataflowError::Validation(
                        "Message type not found".to_string(),
                    ));
                }
            }
        } else if let Some(app_hdr_content) = app_hdr_content {
            match Self::parse_header("", &app_hdr_content) {
                Ok(header) => match header.get("MsgDefIdr") {
                    Some(value) => Helper::manual_unescape(value.to_string().as_str()),
                    None => {
                        return Err(DataflowError::Validation(
                            "MsgDefIdr not found in header".to_string(),
                        ));
                    }
                },
                Err(e) => {
                    return Err(DataflowError::Validation(format!(
                        "Failed to parse header: {e}"
                    )));
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
    fn extract_document_xmlns(xml: &str) -> Option<String> {
        // Find the Document element start tag
        if let Some(start) = xml.find("<Document") {
            // Find the end of the opening tag
            if let Some(end) = xml[start..].find(">") {
                let doc_tag = &xml[start..start + end + 1];

                // Look for xmlns attribute
                if let Some(xmlns_start) = doc_tag.find("xmlns=\"") {
                    if let Some(xmlns_end) = doc_tag[xmlns_start + 7..].find("\"") {
                        return Some(
                            doc_tag[xmlns_start + 7..xmlns_start + 7 + xmlns_end].to_string(),
                        );
                    }
                }
            }
        }
        None
    }

    /// Extract AppHdr content including the wrapper element
    fn extract_app_hdr_content(xml: &str) -> Option<String> {
        if let Some(start) = xml.find("<AppHdr") {
            if let Some(end) = xml.find("</AppHdr>") {
                let end_pos = end + "</AppHdr>".len();
                return Some(xml[start..end_pos].to_string());
            }
        }
        None
    }

    /// Extract Document inner content (without the Document wrapper)
    fn extract_document_content(xml: &str) -> Option<String> {
        // Find Document opening tag
        if let Some(start) = xml.find("<Document") {
            // Find end of opening tag
            if let Some(tag_end) = xml[start..].find(">") {
                let content_start = start + tag_end + 1;

                // Find closing tag
                if let Some(end) = xml.find("</Document>") {
                    return Some(xml[content_start..end].to_string());
                }
            }
        }
        None
    }

    fn parse_header(message_type: &str, app_hdr_content: &str) -> Result<Value> {
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

    fn parse_document(message_type: &str, document_content: &str) -> Result<Value> {
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
            _ => Err(DataflowError::Validation(
                "Unknown message type".to_string(),
            )),
        }
    }
}
