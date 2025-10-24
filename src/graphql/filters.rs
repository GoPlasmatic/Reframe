//! Enhanced filter system with nested AND/OR/NOT logic
//!
//! This module provides a comprehensive filtering system for GraphQL queries
//! with support for nested logical operations and JSON path filtering.

use async_graphql::dynamic::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Enhanced message filter with nested logical operations
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EnhancedMessageFilter {
    pub timestamp: Option<TimeRangeFilter>,
    pub package_id: Option<StringFilter>,
    pub message_type: Option<StringFilter>,
    pub direction: Option<StringFilter>,
    pub has_errors: Option<bool>,
    pub search: Option<String>,
    pub custom_fields: Option<JSONPathFilter>,
    pub context: Option<JSONPathFilter>,

    // Nested logic
    pub and: Option<Vec<EnhancedMessageFilter>>,
    pub or: Option<Vec<EnhancedMessageFilter>>,
    pub not: Option<Box<EnhancedMessageFilter>>,
}

/// String filter with various comparison operators
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StringFilter {
    pub eq: Option<String>,
    pub ne: Option<String>,
    pub r#in: Option<Vec<String>>,
    pub not_in: Option<Vec<String>>,
    pub regex: Option<String>,
    pub contains: Option<String>,
    pub starts_with: Option<String>,
    pub ends_with: Option<String>,
}

/// Time range filter for timestamp comparisons
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TimeRangeFilter {
    pub gte: Option<DateTime<Utc>>,
    pub lte: Option<DateTime<Utc>>,
    pub gt: Option<DateTime<Utc>>,
    pub lt: Option<DateTime<Utc>>,
}

/// JSON path filter for filtering nested JSON fields
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JSONPathFilter {
    pub path: String,
    pub eq: Option<serde_json::Value>,
    pub ne: Option<serde_json::Value>,
    pub gt: Option<f64>,
    pub gte: Option<f64>,
    pub lt: Option<f64>,
    pub lte: Option<f64>,
    pub r#in: Option<Vec<serde_json::Value>>,
    pub not_in: Option<Vec<serde_json::Value>>,
    pub exists: Option<bool>,
}

// GraphQL Input Object builders

/// Build EnhancedMessageFilter input type for GraphQL
pub fn build_enhanced_message_filter_input() -> InputObject {
    InputObject::new("EnhancedMessageFilter")
        .description("Enhanced filter with nested AND/OR/NOT logic")
        .field(InputValue::new(
            "timestamp",
            TypeRef::named("TimeRangeFilter"),
        ))
        .field(InputValue::new("packageId", TypeRef::named("StringFilter")))
        .field(InputValue::new(
            "messageType",
            TypeRef::named("StringFilter"),
        ))
        .field(InputValue::new("direction", TypeRef::named("StringFilter")))
        .field(InputValue::new(
            "hasErrors",
            TypeRef::named(TypeRef::BOOLEAN),
        ))
        .field(InputValue::new("search", TypeRef::named(TypeRef::STRING)))
        .field(InputValue::new(
            "customFields",
            TypeRef::named("JSONPathFilter"),
        ))
        .field(InputValue::new("context", TypeRef::named("JSONPathFilter")))
        .field(InputValue::new(
            "and",
            TypeRef::named_list("EnhancedMessageFilter"),
        ))
        .field(InputValue::new(
            "or",
            TypeRef::named_list("EnhancedMessageFilter"),
        ))
        .field(InputValue::new(
            "not",
            TypeRef::named("EnhancedMessageFilter"),
        ))
}

/// Build StringFilter input type for GraphQL
pub fn build_string_filter_input() -> InputObject {
    InputObject::new("StringFilter")
        .description("String comparison operators")
        .field(InputValue::new("eq", TypeRef::named(TypeRef::STRING)))
        .field(InputValue::new("ne", TypeRef::named(TypeRef::STRING)))
        .field(InputValue::new("in", TypeRef::named_list(TypeRef::STRING)))
        .field(InputValue::new(
            "notIn",
            TypeRef::named_list(TypeRef::STRING),
        ))
        .field(InputValue::new("regex", TypeRef::named(TypeRef::STRING)))
        .field(InputValue::new("contains", TypeRef::named(TypeRef::STRING)))
        .field(InputValue::new(
            "startsWith",
            TypeRef::named(TypeRef::STRING),
        ))
        .field(InputValue::new("endsWith", TypeRef::named(TypeRef::STRING)))
}

/// Build TimeRangeFilter input type for GraphQL
pub fn build_time_range_filter_input() -> InputObject {
    InputObject::new("TimeRangeFilter")
        .description("Time range comparison operators")
        .field(InputValue::new("gte", TypeRef::named("DateTime")))
        .field(InputValue::new("lte", TypeRef::named("DateTime")))
        .field(InputValue::new("gt", TypeRef::named("DateTime")))
        .field(InputValue::new("lt", TypeRef::named("DateTime")))
}

/// Build JSONPathFilter input type for GraphQL
pub fn build_json_path_filter_input() -> InputObject {
    InputObject::new("JSONPathFilter")
        .description("JSON path filter for nested field access")
        .field(InputValue::new("path", TypeRef::named_nn(TypeRef::STRING)))
        .field(InputValue::new("eq", TypeRef::named("JSON")))
        .field(InputValue::new("ne", TypeRef::named("JSON")))
        .field(InputValue::new("gt", TypeRef::named(TypeRef::FLOAT)))
        .field(InputValue::new("gte", TypeRef::named(TypeRef::FLOAT)))
        .field(InputValue::new("lt", TypeRef::named(TypeRef::FLOAT)))
        .field(InputValue::new("lte", TypeRef::named(TypeRef::FLOAT)))
        .field(InputValue::new("in", TypeRef::named_list("JSON")))
        .field(InputValue::new("notIn", TypeRef::named_list("JSON")))
        .field(InputValue::new("exists", TypeRef::named(TypeRef::BOOLEAN)))
}
