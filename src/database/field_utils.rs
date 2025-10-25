//! Field path normalization utilities
//!
//! This module provides utilities for normalizing field paths used in MongoDB queries.
//! It handles the conversion between GraphQL field paths and MongoDB document paths,
//! ensuring consistent behavior across filtering and aggregation operations.

use super::constants::ROOT_FIELDS;

/// Normalize field path for MongoDB queries
///
/// Converts GraphQL field paths to MongoDB document paths with the following rules:
/// - Root-level fields (id, timestamp, errors, etc.) are returned as-is
/// - GraphQL accessor names (e.g., "swiftCbprMtMxFields.xxx") are prefixed with "context."
/// - Fields already prefixed with "context." are returned as-is
/// - All other fields are prefixed with "context."
///
/// # Examples
///
/// ```
/// // Root-level field (no prefix)
/// assert_eq!(normalize_field_path("timestamp"), "timestamp");
/// assert_eq!(normalize_field_path("id"), "id");
/// assert_eq!(normalize_field_path("errors"), "errors");
///
/// // GraphQL accessor (add context prefix)
/// assert_eq!(
///     normalize_field_path("swiftCbprMtMxFields.direction"),
///     "context.swiftCbprMtMxFields.direction"
/// );
///
/// // Already normalized (return as-is)
/// assert_eq!(
///     normalize_field_path("context.metadata.package_id"),
///     "context.metadata.package_id"
/// );
///
/// // Regular field (add context prefix)
/// assert_eq!(
///     normalize_field_path("metadata.package_id"),
///     "context.metadata.package_id"
/// );
/// ```
pub fn normalize_field_path(field: &str) -> String {
    // Check if this is a root-level field
    if is_root_field(field) {
        return field.to_string();
    }

    // Check if this is a GraphQL accessor name (e.g., "swiftCbprMtMxFields.direction")
    // These are dynamically generated accessor names ending with "Fields"
    if let Some(accessor_path) = normalize_graphql_accessor(field) {
        return accessor_path;
    }

    // Apply normal normalization
    if field.starts_with("context.") {
        // Already has context prefix, return as-is
        field.to_string()
    } else {
        // Add context prefix
        format!("context.{}", field)
    }
}

/// Check if a field is a root-level field
///
/// Root-level fields are stored at the document root in MongoDB,
/// not under the "context" object.
fn is_root_field(field: &str) -> bool {
    for &root_field in ROOT_FIELDS {
        if field == root_field || field.starts_with(&format!("{}.", root_field)) {
            return true;
        }
    }
    false
}

/// Normalize a GraphQL accessor field path
///
/// GraphQL accessor names are dynamically generated based on package IDs
/// (e.g., "swiftCbprMtMxFields"). These need to be prefixed with "context."
/// but should not have additional prefixes added.
///
/// Returns Some(path) if this is a GraphQL accessor, None otherwise.
fn normalize_graphql_accessor(field: &str) -> Option<String> {
    if let Some(dot_pos) = field.find('.') {
        let first_component = &field[..dot_pos];
        let rest = &field[dot_pos + 1..];

        // If first component ends with "Fields", it's a GraphQL accessor
        // Exception: "customFields" is not a GraphQL accessor
        if first_component.ends_with("Fields") && first_component != "customFields" {
            return Some(format!("context.{}.{}", first_component, rest));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_root_fields() {
        assert_eq!(normalize_field_path("id"), "id");
        assert_eq!(normalize_field_path("_id"), "_id");
        assert_eq!(normalize_field_path("timestamp"), "timestamp");
        assert_eq!(normalize_field_path("errors"), "errors");
        assert_eq!(normalize_field_path("audit_trail"), "audit_trail");
        assert_eq!(normalize_field_path("payload"), "payload");
    }

    #[test]
    fn test_normalize_context_fields() {
        assert_eq!(
            normalize_field_path("metadata.package_id"),
            "context.metadata.package_id"
        );
        assert_eq!(normalize_field_path("data.amount"), "context.data.amount");
    }

    #[test]
    fn test_normalize_already_prefixed() {
        assert_eq!(
            normalize_field_path("context.metadata.package_id"),
            "context.metadata.package_id"
        );
        assert_eq!(
            normalize_field_path("context.data.amount"),
            "context.data.amount"
        );
    }

    #[test]
    fn test_normalize_graphql_accessor() {
        assert_eq!(
            normalize_field_path("swiftCbprMtMxFields.direction"),
            "context.swiftCbprMtMxFields.direction"
        );
        assert_eq!(
            normalize_field_path("swiftCbprMtMxFields.transaction_risk_score"),
            "context.swiftCbprMtMxFields.transaction_risk_score"
        );
    }

    #[test]
    fn test_normalize_custom_fields_not_accessor() {
        // customFields should be treated as a regular field, not a GraphQL accessor
        assert_eq!(
            normalize_field_path("customFields.field1"),
            "context.customFields.field1"
        );
    }
}
