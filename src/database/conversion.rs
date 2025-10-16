//! MongoDB BSON to GraphQL type conversions
//!
//! This module handles the conversion between MongoDB BSON documents
//! and GraphQL types, keeping the conversion logic separate from database operations.
use crate::graphql::types::{AuditTrailEntry, ChangeEntry, ErrorInfoEntry, TransformationMessage};
use mongodb::bson::Document;

/// Convert BSON document to TransformationMessage
///
/// Extracts and converts all fields from a MongoDB document into a GraphQL-compatible
/// TransformationMessage structure.
pub fn document_to_transformation_message(doc: Document) -> Result<TransformationMessage, String> {
    let id = extract_id(&doc)?;
    let payload = extract_payload(&doc)?;
    let context = extract_context(&doc)?;
    let audit_trail = extract_audit_trail(&doc);
    let errors = extract_errors(&doc);
    let timestamp = extract_timestamp(&doc);

    Ok(TransformationMessage {
        id,
        payload,
        context,
        audit_trail,
        errors,
        timestamp,
    })
}

/// Extract message ID from document
fn extract_id(doc: &Document) -> Result<String, String> {
    doc.get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing id field".to_string())
        .map(String::from)
}

/// Extract payload from document and convert to JSON
fn extract_payload(doc: &Document) -> Result<serde_json::Value, String> {
    let payload_bson = doc
        .get("payload")
        .ok_or_else(|| "Missing payload field".to_string())?;

    bson_to_json(payload_bson)
}

/// Extract context from document and convert to JSON
fn extract_context(doc: &Document) -> Result<serde_json::Value, String> {
    let context_bson = doc
        .get("context")
        .ok_or_else(|| "Missing context field".to_string())?;

    bson_to_json(context_bson)
}

/// Extract audit trail entries from document
fn extract_audit_trail(doc: &Document) -> Vec<AuditTrailEntry> {
    doc.get("audit_trail")
        .and_then(|v| v.as_array())
        .map(|trail_bson| trail_bson.iter().filter_map(parse_audit_entry).collect())
        .unwrap_or_default()
}

/// Parse a single audit trail entry from BSON
fn parse_audit_entry(entry: &mongodb::bson::Bson) -> Option<AuditTrailEntry> {
    let entry_doc = entry.as_document()?;

    let workflow_id = entry_doc.get("workflow_id")?.as_str()?.to_string();
    let task_id = entry_doc.get("task_id")?.as_str()?.to_string();
    let timestamp_str = entry_doc.get("timestamp")?.as_str()?;
    let timestamp = chrono::DateTime::parse_from_rfc3339(timestamp_str)
        .ok()?
        .with_timezone(&chrono::Utc);
    let status = entry_doc.get("status")?.as_i64()?;

    let changes = extract_changes(entry_doc);

    Some(AuditTrailEntry {
        workflow_id,
        task_id,
        timestamp,
        changes,
        status,
    })
}

/// Extract changes from an audit trail entry
fn extract_changes(entry_doc: &Document) -> Vec<ChangeEntry> {
    entry_doc
        .get("changes")
        .and_then(|v| v.as_array())
        .map(|changes_array| {
            changes_array
                .iter()
                .filter_map(parse_change_entry)
                .collect()
        })
        .unwrap_or_default()
}

/// Parse a single change entry from BSON
fn parse_change_entry(change: &mongodb::bson::Bson) -> Option<ChangeEntry> {
    let change_doc = change.as_document()?;
    let path = change_doc.get("path")?.as_str()?.to_string();

    let old_value = change_doc
        .get("old_value")
        .and_then(|v| bson_to_json(v).ok())
        .unwrap_or(serde_json::Value::Null);

    let new_value = change_doc
        .get("new_value")
        .and_then(|v| bson_to_json(v).ok())
        .unwrap_or(serde_json::Value::Null);

    Some(ChangeEntry {
        path,
        old_value,
        new_value,
    })
}

/// Extract error entries from document
fn extract_errors(doc: &Document) -> Vec<ErrorInfoEntry> {
    doc.get("errors")
        .and_then(|v| v.as_array())
        .map(|errors_bson| errors_bson.iter().filter_map(parse_error_entry).collect())
        .unwrap_or_default()
}

/// Parse a single error entry from BSON
fn parse_error_entry(error: &mongodb::bson::Bson) -> Option<ErrorInfoEntry> {
    let error_doc = error.as_document()?;

    let code = error_doc.get("code")?.as_str()?.to_string();
    let message = error_doc.get("message")?.as_str()?.to_string();
    let path = error_doc
        .get("path")
        .and_then(|v| v.as_str())
        .map(String::from);
    let workflow_id = error_doc
        .get("workflow_id")
        .and_then(|v| v.as_str())
        .map(String::from);
    let task_id = error_doc
        .get("task_id")
        .and_then(|v| v.as_str())
        .map(String::from);
    let timestamp = error_doc
        .get("timestamp")
        .and_then(|v| v.as_str())
        .map(String::from);
    let retry_attempted = error_doc.get("retry_attempted").and_then(|v| v.as_bool());
    let retry_count = error_doc.get("retry_count").and_then(|v| v.as_i64());

    Some(ErrorInfoEntry {
        code,
        message,
        path,
        workflow_id,
        task_id,
        timestamp,
        retry_attempted,
        retry_count,
    })
}

/// Extract timestamp from document
fn extract_timestamp(doc: &Document) -> Option<chrono::DateTime<chrono::Utc>> {
    doc.get("timestamp")
        .and_then(|v| v.as_datetime())
        .map(|dt| dt.to_chrono())
}

/// Convert BSON value to JSON value
///
/// This is a helper function that handles the conversion between MongoDB's BSON
/// format and standard JSON values.
fn bson_to_json(bson: &mongodb::bson::Bson) -> Result<serde_json::Value, String> {
    mongodb::bson::to_bson(bson)
        .ok()
        .and_then(|b| serde_json::to_value(&b).ok())
        .ok_or_else(|| "Failed to convert BSON to JSON".to_string())
}
