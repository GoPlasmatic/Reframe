use async_trait::async_trait;
use dataflow_rs::engine::error::DataflowError;
use dataflow_rs::engine::{
    AsyncFunctionHandler,
    error::Result,
    message::{Change, Message},
};
use serde_json::Value;
use swift_mt_message::SwiftMessage;
use swift_mt_message::messages::MT103;
use tracing::{debug, error, instrument};

pub struct PublishMT;

#[async_trait]
impl AsyncFunctionHandler for PublishMT {
    #[instrument(skip(self, message, input))]
    async fn execute(&self, message: &mut Message, input: &Value) -> Result<(usize, Vec<Change>)> {
        debug!("Starting MX to MT message publishing/conversion");

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

        // Extract data first to avoid borrow checker issues
        let input_data = message.data.get(input_field_name).cloned().ok_or_else(|| {
            error!(
                input_field = %input_field_name,
                available_fields = ?message.data.as_object().map(|obj| obj.keys().collect::<Vec<_>>()),
                "Input field not found in message data for MX to MT transformation"
            );
            DataflowError::Validation(format!(
                "Field {} not found in message data {}",
                input_field_name, message.data
            ))
        })?;

        debug!(
            data_type = ?input_data,
            "Processing MX to MT conversion data"
        );

        let json_str = input_data.to_string();
        let mt_message = if source_format == "pacs.008.001.08" {
            let data: SwiftMessage<MT103> = serde_json::from_str(&json_str).map_err(|e| {
                error!(error = ?e, "Failed to parse JSON string");
                DataflowError::Validation(format!("Failed to parse JSON string: {e}"))
            })?;
            data.to_mt_message()
        } else {
            "".to_string()
        };

        debug!(
            mt_message = %mt_message,
            "MT message published successfully"
        );

        let result_value = Value::String(mt_message.clone());
        message.data[output_field_name] = result_value.clone();

        Ok((
            200,
            vec![Change {
                path: format!("data.{output_field_name}"),
                old_value: Value::Null,
                new_value: Value::String(mt_message),
            }],
        ))
    }
}
