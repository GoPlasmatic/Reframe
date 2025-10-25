//! MongoDB field path constants
//!
//! This module provides constants for MongoDB field paths used throughout the application.
//! Centralizing these constants reduces the risk of typos and makes it easier to maintain
//! the database schema.

/// MongoDB field paths for transformation messages
pub mod fields {
    // Root-level fields (stored at document root, not under context)
    pub const TIMESTAMP: &str = "timestamp";
    pub const ERRORS: &str = "errors";

    // Context-level fields
    pub const MESSAGE_TYPE_HINT: &str = "context.metadata.message_type_hint";
    pub const DIRECTION: &str = "context.metadata.direction";
    pub const PACKAGE_ID: &str = "context.metadata.package_id";

    // Custom fields
    pub const CUSTOM_FIELDS: &str = "context.custom_fields";
}

/// Root-level fields that should NOT be prefixed with "context."
/// These fields are stored at the document root in MongoDB
pub const ROOT_FIELDS: &[&str] = &["id", "_id", "timestamp", "errors", "audit_trail", "payload"];

/// Helper functions for building field paths
pub mod paths {
    use super::fields;

    /// Build a custom field path
    ///
    /// # Example
    /// ```
    /// let path = custom_field("transaction_risk_score");
    /// assert_eq!(path, "context.custom_fields.transaction_risk_score");
    /// ```
    pub fn custom_field(field_name: &str) -> String {
        format!("{}.{}", fields::CUSTOM_FIELDS, field_name)
    }
}
