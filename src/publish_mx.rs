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
use tracing::{debug, error, info, instrument, warn};

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
        println!(
            "message_type: {}, message_method: {}",
            message_type, message_method
        );

        let result = if source_format.ends_with(".Header") {
            info!(source_format = %source_format, "Processing MT to MX header");
            handle_mt_to_mx_header(
                input_data.clone(),
                message,
                output_field_name,
                message_type,
                message_method,
            )
        } else if source_format.ends_with(".Document") {
            info!(source_format = %source_format, "Processing MT to MX document");
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
                info!(
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

// MT to MX header handler
#[instrument(skip(data, message), fields(output_field = output_field_name))]
fn handle_mt_to_mx_header(
    data: Value,
    message: &mut Message,
    output_field_name: &str,
    message_type: &str,
    message_method: &str,
) -> Result<(usize, Vec<Change>)> {
    let xml_string = match message_type {
        "103" => {
            if message_method == "reject" {
                debug!("Serializing MT103 REJT to MX header");
                serialize_mt_to_mx_header::<bah_pacs_002_001_10::BusinessApplicationHeaderV02>(
                    data,
                )?
            } else if message_method == "return" {
                debug!("Serializing MT103 RETN to MX header");
                serialize_mt_to_mx_header::<bah_pacs_004_001_09::BusinessApplicationHeaderV02>(
                    data,
                )?
            } else if message_method == "stp" {
                debug!("Serializing MT103 STP to MX header");
                serialize_mt_to_mx_header::<bah_pacs_008_001_08_stp::BusinessApplicationHeaderV02>(
                    data,
                )?
            } else {
                debug!("Serializing MT103 to MX header");
                serialize_mt_to_mx_header::<bah_pacs_008_001_08::BusinessApplicationHeaderV02>(
                    data,
                )?
            }
        }
        "202" => {
            if message_method == "reject" {
                debug!("Serializing MT202 REJT to MX header");
                serialize_mt_to_mx_header::<bah_pacs_002_001_10::BusinessApplicationHeaderV02>(
                    data,
                )?
            } else if message_method == "return" {
                debug!("Serializing MT202 RETN to MX header");
                serialize_mt_to_mx_header::<bah_pacs_004_001_09::BusinessApplicationHeaderV02>(
                    data,
                )?
            } else if message_method == "cover" {
                debug!("Serializing MT202 COVER to MX header");
                serialize_mt_to_mx_header::<bah_pacs_009_001_08_cov::BusinessApplicationHeaderV02>(
                    data,
                )?
            } else {
                debug!("Serializing MT202 to MX header");
                serialize_mt_to_mx_header::<bah_pacs_009_001_08::BusinessApplicationHeaderV02>(
                    data,
                )?
            }
        }
        "205" => {
            if message_method == "reject" {
                debug!("Serializing MT205 REJT to MX header");
                serialize_mt_to_mx_header::<bah_pacs_002_001_10::BusinessApplicationHeaderV02>(
                    data,
                )?
            } else if message_method == "return" {
                debug!("Serializing MT205 RETN to MX header");
                serialize_mt_to_mx_header::<bah_pacs_004_001_09::BusinessApplicationHeaderV02>(
                    data,
                )?
            } else if message_method == "cover" {
                debug!("Serializing MT205 COVER to MX header");
                serialize_mt_to_mx_header::<bah_pacs_009_001_08_cov::BusinessApplicationHeaderV02>(
                    data,
                )?
            } else {
                debug!("Serializing MT205 to MX header");
                serialize_mt_to_mx_header::<bah_pacs_009_001_08::BusinessApplicationHeaderV02>(
                    data,
                )?
            }
        }
        "900" | "910" => {
            debug!("Serializing MT900 to MX header");
            serialize_mt_to_mx_header::<bah_camt_054_001::BusinessApplicationHeaderV02>(data)?
        }
        "192" | "292" => {
            debug!("Serializing MT192/MT292 to MX header");
            serialize_mt_to_mx_header::<bah_camt_056_001_08::BusinessApplicationHeaderV02>(data)?
        }
        "196" | "296" => {
            debug!("Serializing MT196/MT296 to MX header");
            serialize_mt_to_mx_header::<bah_camt_029_001::BusinessApplicationHeaderV02>(data)?
        }
        _ => {
            error!("Invalid message type: {}", message_type);
            return Err(DataflowError::Validation(format!(
                "Invalid message type: {message_type}"
            )));
        }
    };

    let result_value = Value::String(xml_string);
    message.data[output_field_name] = result_value.clone();

    debug!(
        output_field = output_field_name,
        xml_length = result_value.as_str().map(|s| s.len()).unwrap_or(0),
        "MT to MX header serialization completed"
    );

    Ok((
        200,
        vec![Change {
            path: format!("data.{output_field_name}"),
            old_value: Value::Null,
            new_value: result_value,
        }],
    ))
}

// MT to MX document handler
#[instrument(skip(data, message), fields(output_field = output_field_name))]
fn handle_mt_to_mx_document(
    data: Value,
    message: &mut Message,
    output_field_name: &str,
    message_type: &str,
    message_method: &str,
) -> Result<(usize, Vec<Change>)> {
    let xml_string = match message_type {
        "103" => {
            if message_method == "reject" {
                serialize_mt_to_mx_document::<pacs_002_001_10::FIToFIPaymentStatusReportV10>(
                    data.get("FIToFIPmtStsRpt").unwrap().clone(),
                )?
            } else if message_method == "return" {
                serialize_mt_to_mx_document::<pacs_004_001_09::PaymentReturnV09>(
                    data.get("PmtRtr").unwrap().clone(),
                )?
            } else if message_method == "stp" {
                serialize_mt_to_mx_document::<pacs_008_001_08_stp::FIToFICustomerCreditTransferV08>(
                    data.get("FIToFICstmrCdtTrf").unwrap().clone(),
                )?
            } else {
                serialize_mt_to_mx_document::<pacs_008_001_08::FIToFICustomerCreditTransferV08>(
                    data.get("FIToFICstmrCdtTrf").unwrap().clone(),
                )?
            }
        }
        "202" => {
            if message_method == "reject" {
                serialize_mt_to_mx_document::<pacs_002_001_10::FIToFIPaymentStatusReportV10>(
                    data.get("FIToFIPmtStsRpt").unwrap().clone(),
                )?
            } else if message_method == "return" {
                serialize_mt_to_mx_document::<pacs_004_001_09::PaymentReturnV09>(
                    data.get("PmtRtr").unwrap().clone(),
                )?
            } else if message_method == "cover" {
                serialize_mt_to_mx_document::<
                    pacs_009_001_08_cov::FinancialInstitutionCreditTransferV08,
                >(data.get("FIToFICdtTrf").unwrap().clone())?
            } else {
                serialize_mt_to_mx_document::<pacs_009_001_08::FinancialInstitutionCreditTransferV08>(
                    data.get("FIToFICdtTrf").unwrap().clone(),
                )?
            }
        }
        "205" => {
            if message_method == "reject" {
                serialize_mt_to_mx_document::<pacs_002_001_10::FIToFIPaymentStatusReportV10>(
                    data.get("FIToFIPmtStsRpt").unwrap().clone(),
                )?
            } else if message_method == "return" {
                serialize_mt_to_mx_document::<pacs_004_001_09::PaymentReturnV09>(
                    data.get("PmtRtr").unwrap().clone(),
                )?
            } else if message_method == "cover" {
                serialize_mt_to_mx_document::<
                    pacs_009_001_08_cov::FinancialInstitutionCreditTransferV08,
                >(data.get("FIToFICdtTrf").unwrap().clone())?
            } else {
                serialize_mt_to_mx_document::<pacs_009_001_08::FinancialInstitutionCreditTransferV08>(
                    data.get("FIToFICdtTrf").unwrap().clone(),
                )?
            }
        }
        "900" | "910" => serialize_mt_to_mx_document::<
            camt_054_001_08::BankToCustomerDebitCreditNotificationV08,
        >(data.get("BkToCstmrDbtCdtNtfctn").unwrap().clone())?,
        "192" | "292" => serialize_mt_to_mx_document::<
            camt_056_001_08::FIToFIPaymentCancellationRequestV08,
        >(data.get("FIToFIPmtCxlReq").unwrap().clone())?,
        "196" | "296" => serialize_mt_to_mx_document::<
            camt_029_001_09::ResolutionOfInvestigationV09,
        >(data.get("RsltnOfInvstgtn").unwrap().clone())?,
        _ => {
            error!("Invalid message type: {}", message_type);
            return Err(DataflowError::Validation(format!(
                "Invalid message type: {message_type}"
            )));
        }
    };
    let result_value = Value::String(xml_string);
    message.data[output_field_name] = result_value.clone();

    debug!(
        output_field = output_field_name,
        xml_length = result_value.as_str().map(|s| s.len()).unwrap_or(0),
        "MT to MX document serialization completed"
    );

    Ok((
        200,
        vec![Change {
            path: format!("data.{output_field_name}"),
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
    println!("json_str: {}", json_str);
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
