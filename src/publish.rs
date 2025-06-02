use async_trait::async_trait;
use dataflow_rs::engine::error::DataflowError;
use dataflow_rs::engine::{
    error::Result,
    message::{Change, Message},
    AsyncFunctionHandler,
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

        if output_format == "pacs.008.001.13" {
            // Try to deserialize directly into the FIToFICustomerCreditTransferV13 structure
            if let Some(fi_to_fi) = data.get("FIToFICstmrCdtTrf") {
                match serde_json::from_value::<
                    mx_message::pacs_008_001_13::FIToFICustomerCreditTransferV13,
                >(fi_to_fi.clone())
                {
                    Ok(pacs_data) => {
                        // Create the Document enum
                        let document =
                            mx_message::document::Document::FIToFICustomerCreditTransferV13(
                                Box::new(pacs_data),
                            );

                        // Serialize to XML
                        match xml_to_string(&document) {
                            Ok(xml_string) => {
                                message.data["result"] = Value::String(xml_string);

                                return Ok((
                                    200,
                                    vec![Change {
                                        path: "data.result".to_string(),
                                        old_value: Value::Null,
                                        new_value: message.data["result"].clone(),
                                    }],
                                ));
                            }
                            Err(e) => {
                                println!("XML serialization failed: {}", e);
                                return Err(DataflowError::Validation(format!(
                                    "XML serialization failed: {}",
                                    e
                                )));
                            }
                        }
                    }
                    Err(e) => {
                        println!(
                            "FIToFICustomerCreditTransferV13 deserialization failed: {}",
                            e
                        );
                        return Err(DataflowError::Validation(format!(
                            "FIToFICustomerCreditTransferV13 deserialization failed: {}",
                            e
                        )));
                    }
                }
            } else {
                Err(DataflowError::Validation(
                    "FIToFICustomerCreditTransferV13 not found in data".to_string(),
                ))
            }
        } else {
            Err(DataflowError::Validation(format!(
                "Unsupported format: {}",
                output_format
            )))
        }
    }
}
