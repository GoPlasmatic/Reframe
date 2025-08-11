use mx_message::parse_result::{ErrorCollector, ParserConfig as MxParserConfig};
use mx_message::validation::Validate;
use serde::de::DeserializeOwned;
use tracing::{debug, info};

use crate::types::{ErrorType, ReframeError};

/// Helper function to collect validation errors from MX ErrorCollector
pub fn collect_mx_validation_errors(collector: ErrorCollector, errors: &mut Vec<ReframeError>) {
    for error in collector.errors() {
        debug!(
            "Validation error: code={}, message={}, field={:?}, path={:?}",
            error.code, error.message, error.field, error.path
        );
        errors.push(ReframeError {
            error_type: if error.code >= 9000 {
                ErrorType::ParserError
            } else {
                ErrorType::BusinessValidationError
            },
            code: format!("MX_ERROR_{}", error.code),
            message: error.message.clone(),
            field: error.field.clone(),
            location: error.path.clone(),
            details: None,
        });
    }
}

/// Generic function to validate MX header
pub fn validate_mx_header_generic<T>(
    app_hdr_content: &Option<String>,
    message_type: &str,
    mx_config: &MxParserConfig,
    collector: &mut ErrorCollector,
    errors: &mut Vec<ReframeError>,
) where
    T: DeserializeOwned + Validate,
{
    if let Some(app_hdr) = app_hdr_content {
        debug!("Parsing and validating {} header", message_type);
        match quick_xml::de::from_str::<T>(app_hdr) {
            Ok(header) => {
                debug!("Successfully parsed header, now validating...");
                header.validate("AppHdr", mx_config, collector);
                debug!(
                    "Header validation complete, found {} errors so far",
                    collector.error_count()
                );
            }
            Err(parse_err) => {
                debug!("Header parse failed with error: {}", parse_err);
                errors.push(ReframeError {
                    error_type: ErrorType::ParserError,
                    code: "MX_HEADER_PARSE_ERROR".to_string(),
                    message: format!("Failed to parse {message_type} header: {parse_err}"),
                    field: None,
                    location: Some("AppHdr".to_string()),
                    details: None,
                });
            }
        }
    }
}

/// Generic function to validate MX document (takes ownership of collector)
pub fn validate_mx_document_generic<T>(
    doc_content: &str,
    message_type: &str,
    mx_config: &MxParserConfig,
    mut collector: ErrorCollector,
    errors: &mut Vec<ReframeError>,
) where
    T: DeserializeOwned + Validate,
{
    debug!("Parsing and validating {} document", message_type);
    match quick_xml::de::from_str::<T>(doc_content) {
        Ok(document) => {
            debug!("Successfully parsed {}, now validating...", message_type);
            document.validate("Document", mx_config, &mut collector);
            debug!(
                "Document validation complete, total {} errors",
                collector.error_count()
            );
            collect_mx_validation_errors(collector, errors);
        }
        Err(parse_err) => {
            info!("MX Validation: Parse failed with error: {}", parse_err);
            errors.push(ReframeError {
                error_type: ErrorType::ParserError,
                code: "MX_PARSE_ERROR".to_string(),
                message: format!("Failed to parse {message_type} document: {parse_err}"),
                field: None,
                location: None,
                details: None,
            });
        }
    }
}

/// Macro to simplify MX message validation
#[macro_export]
macro_rules! validate_mx_message {
    ($message_type:expr, $app_hdr:expr, $doc_content:expr, $mx_config:expr, $errors:expr, {
        header: $header_type:ty,
        document: $document_type:ty
    }) => {{
        use $crate::validation_helpers::{
            validate_mx_document_generic, validate_mx_header_generic,
        };

        // Create a new collector for this validation
        let mut collector = mx_message::parse_result::ErrorCollector::new();

        // Validate header if present
        validate_mx_header_generic::<$header_type>(
            $app_hdr,
            $message_type,
            $mx_config,
            &mut collector,
            $errors,
        );

        // Validate document (passes ownership of collector)
        validate_mx_document_generic::<$document_type>(
            $doc_content,
            $message_type,
            $mx_config,
            collector,
            $errors,
        );
    }};
}

/// Helper function to perform MT business validation
pub fn perform_mt_business_validation(message_type: &str, errors: &mut Vec<ReframeError>) {
    // For now, just add an info message that business validation is not yet implemented
    // This can be expanded in the future with actual business rule validation
    errors.push(ReframeError {
        error_type: ErrorType::Info,
        code: "MT_VALIDATION_LIMITED".to_string(),
        message: format!(
            "Business validation not yet implemented for message type: {message_type}"
        ),
        field: None,
        location: None,
        details: None,
    });
}
