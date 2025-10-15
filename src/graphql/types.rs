use async_graphql::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Transformation message stored in the audit database
/// This matches the dataflow-rs Message structure
#[derive(SimpleObject, Serialize, Deserialize, Clone, Debug)]
pub struct TransformationMessage {
    /// Unique message ID (UUID)
    pub id: String,

    /// Original payload (input message)
    pub payload: serde_json::Value,

    /// Unified context containing data, metadata, and temp_data
    pub context: serde_json::Value,

    /// Audit trail of workflow and task executions
    pub audit_trail: Vec<AuditTrailEntry>,

    /// Errors that occurred during processing
    pub errors: Vec<ErrorInfoEntry>,

    /// Timestamp when stored (added by MongoDB)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<DateTime<Utc>>,
}

/// Audit trail entry showing workflow/task execution
#[derive(SimpleObject, Serialize, Deserialize, Clone, Debug)]
pub struct AuditTrailEntry {
    /// Workflow ID that executed
    pub workflow_id: String,

    /// Task ID within the workflow
    pub task_id: String,

    /// When this task executed
    pub timestamp: DateTime<Utc>,

    /// Changes made by this task
    pub changes: Vec<ChangeEntry>,

    /// Status code (0 = success, non-zero = error)
    pub status: i64,
}

/// Change made by a workflow task
#[derive(SimpleObject, Serialize, Deserialize, Clone, Debug)]
pub struct ChangeEntry {
    /// JSONPath to the changed field
    pub path: String,

    /// Value before the change
    pub old_value: serde_json::Value,

    /// Value after the change
    pub new_value: serde_json::Value,
}

/// Error information from message processing
#[derive(SimpleObject, Serialize, Deserialize, Clone, Debug)]
pub struct ErrorInfoEntry {
    /// Error code (e.g., "VALIDATION_ERROR", "WORKFLOW_ERROR")
    pub code: String,

    /// Human-readable error message
    pub message: String,

    /// Optional path to error location
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// Workflow ID where error occurred
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow_id: Option<String>,

    /// Task ID where error occurred
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,

    /// Error timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,

    /// Whether retry was attempted
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_attempted: Option<bool>,

    /// Number of retries
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_count: Option<i64>,
}

/// Filter criteria for querying messages
#[derive(InputObject, Debug, Clone)]
pub struct MessageFilter {
    /// Filter by message type (e.g., MT103, pacs.008)
    pub message_type: Option<String>,

    /// Filter by transformation direction (outgoing or incoming)
    pub direction: Option<String>,

    /// Filter by success status
    pub success: Option<bool>,

    /// Filter messages from this date onwards
    pub date_from: Option<DateTime<Utc>>,

    /// Filter messages up to this date
    pub date_to: Option<DateTime<Utc>>,

    /// Text search across message content
    pub search: Option<String>,
}

/// Paginated response for messages query
#[derive(SimpleObject, Debug, Clone)]
pub struct MessageConnection {
    /// List of transformation messages
    pub messages: Vec<TransformationMessage>,

    /// Total number of messages matching the filter
    pub total_count: i64,

    /// Whether there are more pages available
    pub has_next_page: bool,
}

/// Statistics about transformation messages
#[derive(SimpleObject, Debug, Clone)]
pub struct MessageStats {
    /// Total number of messages
    pub total_messages: i64,

    /// Number of successful transformations
    pub success_count: i64,

    /// Number of failed transformations
    pub failure_count: i64,

    /// Success rate as a percentage (0-100)
    pub success_rate: f64,

    /// Average processing time in milliseconds
    pub average_processing_time_ms: f64,

    /// Breakdown by message type
    pub message_type_breakdown: Vec<MessageTypeCount>,
}

/// Count of messages by type
#[derive(SimpleObject, Debug, Clone)]
pub struct MessageTypeCount {
    /// Message type (e.g., MT103, pacs.008)
    pub message_type: String,

    /// Number of messages of this type
    pub count: i64,
}
