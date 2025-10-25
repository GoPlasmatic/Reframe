//! Dynamic GraphQL Schema Builder
//!
//! This module builds GraphQL types at runtime based on package configurations.
//! It uses async-graphql's dynamic schema features to create package-specific
//! custom field types that can be hot-reloaded when packages change.

use async_graphql::Value as GqlValue;
use async_graphql::dynamic::*;
use serde_json::Value as JsonValue;
use std::collections::HashMap;

use super::types::TransformationMessage;
use crate::custom_fields::CustomFieldDefinition;
use crate::package_manager::PackageInfo;
use crate::utils::{
    json_to_graphql_value, json_to_graphql_value_typed, to_camel_case, to_pascal_case,
};

/// Build JSON scalar type for arbitrary JSON values
pub fn build_json_scalar() -> Scalar {
    Scalar::new("JSON")
        .description("Arbitrary JSON value (object, array, string, number, boolean, or null)")
}

/// Build DateTime scalar type for RFC3339 timestamps
pub fn build_datetime_scalar() -> Scalar {
    Scalar::new("DateTime")
        .description("DateTime value in RFC3339 format (e.g., '2024-01-15T10:30:00Z')")
}

/// Convert api_config.json field type to GraphQL TypeRef
///
/// Supported types (all nullable):
/// - string → String
/// - number → Float
/// - int → Int
/// - boolean → Boolean
/// - datetime → DateTime (RFC3339 format)
pub fn map_field_type(field_type: &str) -> TypeRef {
    match field_type {
        "string" => TypeRef::named(TypeRef::STRING),
        "number" => TypeRef::named(TypeRef::FLOAT),
        "int" => TypeRef::named(TypeRef::INT),
        "boolean" => TypeRef::named(TypeRef::BOOLEAN),
        "datetime" => TypeRef::named("DateTime"), // Custom DateTime scalar
        _ => {
            // Unknown types default to String
            TypeRef::named(TypeRef::STRING)
        }
    }
}

/// Build a dynamic GraphQL Object type for a package's custom fields
///
/// Creates a type like "SwiftCbprMtMxFields" with fields for each
/// custom field definition in the package's api_config.json
///
/// # Arguments
/// * `package_id` - The package identifier (e.g., "swift-cbpr-mt-mx")
/// * `package_name` - Human-readable package name
/// * `definitions` - Custom field definitions from api_config.json
///
/// # Returns
/// A dynamic GraphQL Object type
pub fn build_package_custom_fields_type(
    package_id: &str,
    package_name: &str,
    definitions: &[CustomFieldDefinition],
) -> Object {
    let type_name = format!("{}Fields", to_pascal_case(package_id));

    let mut obj = Object::new(&type_name);
    obj = obj.description(format!(
        "Package-specific custom fields for {} - Computed using JSONLogic expressions during message transformation and validation",
        package_name
    ));

    // Add each field from api_config.json
    for def in definitions {
        let field_name_camel = to_camel_case(&def.name);
        let field_type = map_field_type(&def.field_type);

        // Use description from package config with additional metadata
        let description = format!(
            "{}\n\n📦 Package: {}\n🔧 Type: {}\n💾 Storage: {:?}\n\nThis field is computed from the transformation context using JSONLogic expressions defined in the package configuration.",
            def.description, package_name, def.field_type, def.storage
        );

        let def_name = def.name.clone();
        let def_type = def.field_type.clone(); // Capture field type for type-aware conversion
        let field = Field::new(&field_name_camel, field_type, move |ctx| {
            let field_name = def_name.clone();
            let field_type = def_type.clone();
            FieldFuture::new(async move {
                // Get custom fields data from parent
                let parent = ctx
                    .parent_value
                    .try_downcast_ref::<HashMap<String, JsonValue>>()?;

                // Extract field value with type-aware conversion
                let value = parent
                    .get(&field_name)
                    .and_then(|v| json_to_graphql_value_typed(v, &field_type));

                Ok(value)
            })
        })
        .description(&description);

        obj = obj.field(field);
    }

    obj
}

/// Build accessor name for a package's custom fields
///
/// Examples:
/// - "swift-cbpr-mt-mx" → "swiftCbprMtMxFields"
/// - "other-package" → "otherPackageFields"
pub fn build_accessor_name(package_id: &str) -> String {
    format!("{}Fields", to_camel_case(&package_id.replace("-", "_")))
}

/// Build the TransformationMessage type with package-specific custom field accessors
///
/// This creates a dynamic GraphQL type that includes:
/// - Standard fields (id, packageId, timestamp, etc.)
/// - Package-specific custom field accessors (e.g., swiftCbprMtMxCustomFields)
///
/// Each package accessor returns null if the message doesn't belong to that package.
pub fn build_transformation_message_type(_packages: &HashMap<String, PackageInfo>) -> Object {
    let mut obj = Object::new("TransformationMessage");
    obj = obj.description("Represents a message transformation processed by Reframe - includes the original payload, transformation context, workflow execution audit trail, and any errors encountered during processing");

    // Add standard fields
    obj = obj
        .field(
            Field::new("id", TypeRef::named_nn(TypeRef::STRING), |ctx| {
                FieldFuture::new(async move {
                    let msg = ctx
                        .parent_value
                        .try_downcast_ref::<TransformationMessage>()?;
                    Ok(Some(GqlValue::String(msg.id.clone())))
                })
            })
            .description("Unique message identifier (UUID) - Generated when the message enters the transformation engine and used for tracking throughout the workflow pipeline"),
        )
        .field(
            Field::new("packageId", TypeRef::named(TypeRef::STRING), |ctx| {
                FieldFuture::new(async move {
                    let msg = ctx
                        .parent_value
                        .try_downcast_ref::<TransformationMessage>()?;
                    Ok(msg.package_id.as_ref().map(|s| GqlValue::String(s.clone())))
                })
            })
            .description("Identifier of the transformation package used to process this message (e.g., 'swift-cbpr-mt-mx') - Packages contain the workflow definitions, validation rules, and custom field logic"),
        )
        .field(
            Field::new("timestamp", TypeRef::named("DateTime"), |ctx| {
                FieldFuture::new(async move {
                    let msg = ctx
                        .parent_value
                        .try_downcast_ref::<TransformationMessage>()?;
                    Ok(msg.timestamp.map(|dt| GqlValue::String(dt.to_rfc3339())))
                })
            })
            .description("Timestamp when this message was stored in the audit database (RFC3339 format) - Reflects when the transformation completed, not when the original message was created"),
        )
        .field(
            Field::new("payload", TypeRef::named_nn(TypeRef::STRING), |ctx| {
                FieldFuture::new(async move {
                    let msg = ctx
                        .parent_value
                        .try_downcast_ref::<TransformationMessage>()?;
                    Ok(Some(GqlValue::String(
                        serde_json::to_string(&msg.payload).unwrap_or_default(),
                    )))
                })
            })
            .description("Original input message payload as received by the transformation engine (JSON string) - Can be MT (SWIFT), MX (ISO 20022 XML), or any other supported format depending on the transformation direction"),
        )
        .field(
            Field::new("context", TypeRef::named_nn("Context"), |ctx| {
                FieldFuture::new(async move {
                    let msg = ctx
                        .parent_value
                        .try_downcast_ref::<TransformationMessage>()?;
                    // Convert context JsonValue to Map for Context type resolver
                    if let JsonValue::Object(map) = &msg.context {
                        Ok(Some(FieldValue::owned_any(map.clone())))
                    } else {
                        Ok(None)
                    }
                })
            })
            .description("Unified transformation context containing parsed message data, metadata about the transformation (direction, message type, package info), and package-specific custom computed fields"),
        )
        .field(
            Field::new(
                "auditTrail",
                TypeRef::named_nn_list_nn("AuditTrailEntry"),
                |ctx| {
                    FieldFuture::new(async move {
                        let msg = ctx
                            .parent_value
                            .try_downcast_ref::<TransformationMessage>()?;
                        let trail: Vec<FieldValue> = msg
                            .audit_trail
                            .iter()
                            .map(|entry| FieldValue::owned_any(entry.clone()))
                            .collect();
                        Ok(Some(FieldValue::list(trail)))
                    })
                },
            )
            .description("Complete audit trail of all workflow tasks executed during transformation - Each entry shows the workflow/task ID, execution timestamp, changes made to the context, and success/failure status"),
        )
        .field(
            Field::new(
                "errors",
                TypeRef::named_nn_list_nn("ErrorInfoEntry"),
                |ctx| {
                    FieldFuture::new(async move {
                        let msg = ctx
                            .parent_value
                            .try_downcast_ref::<TransformationMessage>()?;
                        let errors: Vec<FieldValue> = msg
                            .errors
                            .iter()
                            .map(|err| FieldValue::owned_any(err.clone()))
                            .collect();
                        Ok(Some(FieldValue::list(errors)))
                    })
                },
            )
            .description("List of errors encountered during message processing - Empty array indicates successful transformation. Errors include validation failures, workflow execution errors, and parsing issues"),
        );

    // Note: Custom field accessors have been moved to Context type
    // Access via: message.context.swiftCbprMtMxFields instead of message.swiftCbprMtMxFields

    obj
}

/// Build Context type with data, metadata, and package-specific custom field accessors
pub fn build_context_type(packages: &HashMap<String, PackageInfo>) -> Object {
    let mut obj = Object::new("Context")
        .description("Unified transformation context object - Contains parsed message data, transformation metadata, and package-specific custom computed fields. This context is passed through the workflow pipeline and modified by each task.")
        .field(
            Field::new("data", TypeRef::named_nn("JSON"), |ctx| {
                FieldFuture::new(async move {
                    let context = ctx
                        .parent_value
                        .try_downcast_ref::<serde_json::Map<String, JsonValue>>()?;
                    Ok(context.get("data").and_then(json_to_graphql_value))
                })
            })
            .description("Parsed and transformed message data as a JSON object - Contains the structured representation of the message fields after parsing and any transformations applied by workflow tasks. Structure varies by message type and transformation direction."),
        )
        .field(
            Field::new("metadata", TypeRef::named_nn("JSON"), |ctx| {
                FieldFuture::new(async move {
                    let context = ctx
                        .parent_value
                        .try_downcast_ref::<serde_json::Map<String, JsonValue>>()?;
                    Ok(context.get("metadata").and_then(json_to_graphql_value))
                })
            })
            .description("Transformation metadata as a JSON object - Contains information about the transformation process including direction (outgoing/incoming), message_type_hint (e.g., MT103, pacs.008), package_id, and other processing metadata"),
        );

    // Add package-specific custom field accessors to Context
    for (package_id, package) in packages {
        if package.custom_fields.is_empty() {
            continue;
        }

        let type_name = format!("{}Fields", to_pascal_case(package_id));
        let accessor_name = build_accessor_name(package_id);
        let accessor_name_clone = accessor_name.clone();
        let package_name_clone = package.name.clone();

        let field = Field::new(&accessor_name, TypeRef::named(&type_name), move |ctx| {
            let accessor = accessor_name_clone.clone();
            FieldFuture::new(async move {
                let context = ctx
                    .parent_value
                    .try_downcast_ref::<serde_json::Map<String, JsonValue>>()?;

                let custom_fields_map: Option<HashMap<String, JsonValue>> = context
                    .get(&accessor)
                    .and_then(|json| serde_json::from_value(json.clone()).ok());

                Ok(custom_fields_map.map(FieldValue::owned_any))
            })
        })
        .description(format!(
            "Package-specific custom computed fields for {} - These fields are calculated using JSONLogic expressions defined in the package's api_config.json and provide domain-specific business logic derived from the transformation context (stored at context.{})",
            package_name_clone, accessor_name
        ));

        obj = obj.field(field);
    }

    obj
}

/// Build AuditTrailEntry type
pub fn build_audit_trail_entry_type() -> Object {
    use super::types::AuditTrailEntry;

    Object::new("AuditTrailEntry")
        .description("Represents a single step in the transformation workflow execution - Records which workflow task ran, when it executed, what changes it made to the context, and whether it succeeded or failed")
        .field(
            Field::new("workflowId", TypeRef::named_nn(TypeRef::STRING), |ctx| {
                FieldFuture::new(async move {
                    let entry = ctx.parent_value.try_downcast_ref::<AuditTrailEntry>()?;
                    Ok(Some(GqlValue::String(entry.workflow_id.clone())))
                })
            })
            .description("Identifier of the workflow that was executed (e.g., 'parse-mt', 'document-mapping', 'validation') - Workflows are defined in the transformation package and orchestrate the message transformation pipeline"),
        )
        .field(
            Field::new("taskId", TypeRef::named_nn(TypeRef::STRING), |ctx| {
                FieldFuture::new(async move {
                    let entry = ctx.parent_value.try_downcast_ref::<AuditTrailEntry>()?;
                    Ok(Some(GqlValue::String(entry.task_id.clone())))
                })
            })
            .description("Unique identifier of the specific task within the workflow - Tasks perform atomic operations like field mapping, validation, or data transformation using JSONLogic expressions"),
        )
        .field(
            Field::new("timestamp", TypeRef::named_nn("DateTime"), |ctx| {
                FieldFuture::new(async move {
                    let entry = ctx.parent_value.try_downcast_ref::<AuditTrailEntry>()?;
                    Ok(Some(GqlValue::String(entry.timestamp.to_rfc3339())))
                })
            })
            .description("Timestamp when this specific task was executed (RFC3339 format) - Useful for performance analysis and debugging transformation pipeline execution order"),
        )
        .field(
            Field::new("status", TypeRef::named_nn(TypeRef::INT), |ctx| {
                FieldFuture::new(async move {
                    let entry = ctx.parent_value.try_downcast_ref::<AuditTrailEntry>()?;
                    Ok(Some(GqlValue::from(entry.status)))
                })
            })
            .description("Task execution status code - 0 indicates successful execution, non-zero values indicate errors. Task failures may halt the transformation pipeline depending on error handling configuration"),
        )
        .field(
            Field::new("changes", TypeRef::named_nn_list_nn("ChangeEntry"), |ctx| {
                FieldFuture::new(async move {
                    let entry = ctx.parent_value.try_downcast_ref::<AuditTrailEntry>()?;
                    let changes: Vec<FieldValue> = entry
                        .changes
                        .iter()
                        .map(|change| FieldValue::owned_any(change.clone()))
                        .collect();
                    Ok(Some(FieldValue::list(changes)))
                })
            })
            .description("List of all changes made to the transformation context by this task - Each change records the JSONPath to the modified field, its value before the change, and the new value after the change"),
        )
}

/// Build ErrorInfoEntry type
pub fn build_error_info_entry_type() -> Object {
    use super::types::ErrorInfoEntry;

    Object::new("ErrorInfoEntry")
        .description("Detailed error information from message transformation or validation - Captures errors that occur during parsing, workflow execution, validation, or message generation")
        .field(
            Field::new("code", TypeRef::named_nn(TypeRef::STRING), |ctx| {
                FieldFuture::new(async move {
                    let entry = ctx.parent_value.try_downcast_ref::<ErrorInfoEntry>()?;
                    Ok(Some(GqlValue::String(entry.code.clone())))
                })
            })
            .description("Error category code identifying the type of error (e.g., VALIDATION_ERROR, WORKFLOW_ERROR, PARSE_ERROR, GENERATION_ERROR) - Used for programmatic error handling and filtering"),
        )
        .field(
            Field::new("message", TypeRef::named_nn(TypeRef::STRING), |ctx| {
                FieldFuture::new(async move {
                    let entry = ctx.parent_value.try_downcast_ref::<ErrorInfoEntry>()?;
                    Ok(Some(GqlValue::String(entry.message.clone())))
                })
            })
            .description("Human-readable error message describing what went wrong - Provides detailed context for debugging and user feedback"),
        )
        .field(
            Field::new("path", TypeRef::named(TypeRef::STRING), |ctx| {
                FieldFuture::new(async move {
                    let entry = ctx.parent_value.try_downcast_ref::<ErrorInfoEntry>()?;
                    Ok(entry.path.as_ref().map(|s| GqlValue::String(s.clone())))
                })
            })
            .description("Optional JSONPath or field path indicating where in the message the error occurred (e.g., 'context.data.Amount', 'payload.field_32A') - Useful for pinpointing validation or transformation errors"),
        )
        .field(
            Field::new("workflowId", TypeRef::named(TypeRef::STRING), |ctx| {
                FieldFuture::new(async move {
                    let entry = ctx.parent_value.try_downcast_ref::<ErrorInfoEntry>()?;
                    Ok(entry
                        .workflow_id
                        .as_ref()
                        .map(|s| GqlValue::String(s.clone())))
                })
            })
            .description("Identifier of the workflow that was executing when the error occurred - Helps identify which stage of the transformation pipeline failed"),
        )
        .field(
            Field::new("taskId", TypeRef::named(TypeRef::STRING), |ctx| {
                FieldFuture::new(async move {
                    let entry = ctx.parent_value.try_downcast_ref::<ErrorInfoEntry>()?;
                    Ok(entry.task_id.as_ref().map(|s| GqlValue::String(s.clone())))
                })
            })
            .description("Identifier of the specific task within the workflow where the error occurred - Enables precise error localization for debugging"),
        )
        .field(
            Field::new("timestamp", TypeRef::named("DateTime"), |ctx| {
                FieldFuture::new(async move {
                    let entry = ctx.parent_value.try_downcast_ref::<ErrorInfoEntry>()?;
                    Ok(entry
                        .timestamp
                        .as_ref()
                        .map(|s| GqlValue::String(s.clone())))
                })
            })
            .description("Timestamp when the error occurred (RFC3339 format) - Useful for correlating errors with audit trail entries and performance analysis"),
        )
        .field(
            Field::new("retryAttempted", TypeRef::named(TypeRef::BOOLEAN), |ctx| {
                FieldFuture::new(async move {
                    let entry = ctx.parent_value.try_downcast_ref::<ErrorInfoEntry>()?;
                    Ok(entry.retry_attempted.map(GqlValue::Boolean))
                })
            })
            .description("Indicates whether the transformation engine attempted to retry the failed operation - Depends on error handling configuration in the workflow"),
        )
        .field(
            Field::new("retryCount", TypeRef::named(TypeRef::INT), |ctx| {
                FieldFuture::new(async move {
                    let entry = ctx.parent_value.try_downcast_ref::<ErrorInfoEntry>()?;
                    Ok(entry.retry_count.map(GqlValue::from))
                })
            })
            .description("Number of retry attempts made before giving up - Zero or null indicates no retries were performed"),
        )
}

/// Build MessageConnection type for paginated results
pub fn build_message_connection_type() -> Object {
    use super::types::MessageConnection;

    Object::new("MessageConnection")
        .description("Paginated list response for transformation messages query - Returned when querying messages in list mode (without aggregation metrics)")
        .field(Field::new(
            "messages",
            TypeRef::named_nn_list_nn("TransformationMessage"),
            |ctx| {
                FieldFuture::new(async move {
                    let conn = ctx.parent_value.try_downcast_ref::<MessageConnection>()?;
                    let messages: Vec<FieldValue> = conn
                        .messages
                        .iter()
                        .map(|m| FieldValue::owned_any(m.clone()))
                        .collect();
                    Ok(Some(FieldValue::list(messages)))
                })
            },
        ).description("Array of transformation messages matching the filter criteria - Limited by the 'limit' parameter (default 50, max 1000)"))
        .field(Field::new(
            "totalCount",
            TypeRef::named_nn(TypeRef::INT),
            |ctx| {
                FieldFuture::new(async move {
                    let conn = ctx.parent_value.try_downcast_ref::<MessageConnection>()?;
                    Ok(Some(GqlValue::from(conn.total_count)))
                })
            },
        ).description("Total number of messages matching the filter criteria across all pages - Use this with 'limit' and 'offset' for pagination calculations"))
        .field(Field::new(
            "hasNextPage",
            TypeRef::named_nn(TypeRef::BOOLEAN),
            |ctx| {
                FieldFuture::new(async move {
                    let conn = ctx.parent_value.try_downcast_ref::<MessageConnection>()?;
                    Ok(Some(GqlValue::Boolean(conn.has_next_page)))
                })
            },
        ).description("Indicates whether more results are available beyond the current page - True if (offset + limit) < totalCount"))
}

/// Build MessageFilter input type
pub fn build_message_filter_type() -> InputObject {
    InputObject::new("MessageFilter")
        .description("Filter criteria for querying transformation messages - Supports filtering by message type, direction, success status, date range, text search, package, custom fields, and dynamic path-based filtering")
        .field(
            InputValue::new("messageType", TypeRef::named(TypeRef::STRING))
                .description("Filter by message type identifier (e.g., 'MT103', 'pacs.008', 'MT202', 'camt.052') - Matches the message_type_hint stored in context.metadata")
        )
        .field(
            InputValue::new("direction", TypeRef::named(TypeRef::STRING))
                .description("Filter by transformation direction: 'outgoing' for MT→MX transformations or 'incoming' for MX→MT transformations")
        )
        .field(
            InputValue::new("success", TypeRef::named(TypeRef::BOOLEAN))
                .description("Filter by transformation success status: true for successful transformations (no errors), false for failed transformations (has errors)")
        )
        .field(
            InputValue::new("dateFrom", TypeRef::named(TypeRef::STRING))
                .description("Filter messages from this timestamp onwards (RFC3339 format, e.g., '2024-01-15T10:30:00Z') - Filters based on when the message was stored in the audit database")
        )
        .field(
            InputValue::new("dateTo", TypeRef::named(TypeRef::STRING))
                .description("Filter messages up to this timestamp (RFC3339 format) - Filters based on when the message was stored in the audit database")
        )
        .field(
            InputValue::new("search", TypeRef::named(TypeRef::STRING))
                .description("Text search across message content - Searches in payload, context data, and metadata fields. Uses MongoDB text search capabilities for efficient full-text queries")
        )
        .field(
            InputValue::new("packageId", TypeRef::named(TypeRef::STRING))
                .description("Filter by transformation package identifier (e.g., 'swift-cbpr-mt-mx') - Useful when multiple transformation packages are configured")
        )
        .field(
            InputValue::new("customFieldFilters", TypeRef::named(TypeRef::STRING))
                .description("Filter by package-specific custom computed fields as a JSON string - Example: '{\"transaction_risk_score\": {\"gte\": 70}, \"is_cross_border\": true}' - Custom fields must be defined in the package's api_config.json")
        )
        .field(
            InputValue::new("path", TypeRef::named_list("PathFilter"))
                .description("Advanced dynamic field filtering using dot-notation paths - Allows filtering on any field in the message structure (e.g., context.metadata.direction, errors) - More flexible than predefined filter fields")
        )
}

/// Build PathFilter input type
pub fn build_path_filter_type() -> InputObject {
    InputObject::new("PathFilter")
        .description("Dynamic path-based filter for querying any field in the message structure - Enables flexible filtering without requiring predefined filter fields")
        .field(
            InputValue::new("field", TypeRef::named_nn(TypeRef::STRING))
                .description("Dot-notation path to the field to filter on (e.g., 'context.metadata.direction', 'context.data.Amount', 'errors', 'timestamp') - Supports nested field access for complex queries")
        )
        .field(
            InputValue::new("value", TypeRef::named_nn("FilterValue"))
                .description("Filter value and operator - Supports string, number, boolean, date comparisons, and existence checks - Type should match the field being filtered")
        )
}

/// Build FilterValue input type
pub fn build_filter_value_type() -> InputObject {
    InputObject::new("FilterValue")
        .description("Polymorphic filter value supporting different data types and comparison operators - Specify exactly one of the available filter types based on the field being filtered")
        .field(
            InputValue::new("string", TypeRef::named("StringFilter"))
                .description("String comparison filters - Use for text fields, supports equality, contains, regex, and array membership checks")
        )
        .field(
            InputValue::new("number", TypeRef::named("NumberFilter"))
                .description("Numeric comparison filters - Use for integer or float fields, supports equality, comparison operators (gt, gte, lt, lte), and range checks")
        )
        .field(
            InputValue::new("boolean", TypeRef::named(TypeRef::BOOLEAN))
                .description("Boolean value filter - Use for true/false fields, performs exact equality match")
        )
        .field(
            InputValue::new("date", TypeRef::named("DateFilter"))
                .description("Date/timestamp comparison filters - Use for DateTime fields, supports before, after, and between range queries (RFC3339 format)")
        )
        .field(
            InputValue::new("exists", TypeRef::named(TypeRef::BOOLEAN))
                .description("Field existence check - true to filter for messages where the field exists (not null), false to filter for messages where the field is missing or null")
        )
}

/// Build StringFilter input type
pub fn build_string_filter_type() -> InputObject {
    InputObject::new("StringFilter")
        .description("String comparison operators for text field filtering - Specify exactly one operator per filter")
        .field(
            InputValue::new("eq", TypeRef::named(TypeRef::STRING))
                .description("Exact equality match - Field value must exactly match this string (case-sensitive)")
        )
        .field(
            InputValue::new("ne", TypeRef::named(TypeRef::STRING))
                .description("Not equal - Field value must not match this string (case-sensitive)")
        )
        .field(
            InputValue::new("in", TypeRef::named_list(TypeRef::STRING))
                .description("Array membership - Field value must match one of the strings in the provided array (e.g., ['MT103', 'MT202'])")
        )
        .field(
            InputValue::new("contains", TypeRef::named(TypeRef::STRING))
                .description("Substring match - Field value must contain this substring (case-sensitive) - Useful for partial text matching")
        )
        .field(
            InputValue::new("regex", TypeRef::named(TypeRef::STRING))
                .description("Regular expression match - Field value must match this regex pattern (e.g., '^MT[0-9]{3}$') - Supports MongoDB regex syntax")
        )
}

/// Build NumberFilter input type
pub fn build_number_filter_type() -> InputObject {
    InputObject::new("NumberFilter")
        .description("Numeric comparison operators for integer and float field filtering - Specify exactly one operator per filter")
        .field(
            InputValue::new("eq", TypeRef::named(TypeRef::FLOAT))
                .description("Exact equality - Field value must exactly equal this number")
        )
        .field(
            InputValue::new("ne", TypeRef::named(TypeRef::FLOAT))
                .description("Not equal - Field value must not equal this number")
        )
        .field(
            InputValue::new("gt", TypeRef::named(TypeRef::FLOAT))
                .description("Greater than - Field value must be strictly greater than this number (exclusive)")
        )
        .field(
            InputValue::new("gte", TypeRef::named(TypeRef::FLOAT))
                .description("Greater than or equal - Field value must be greater than or equal to this number (inclusive)")
        )
        .field(
            InputValue::new("lt", TypeRef::named(TypeRef::FLOAT))
                .description("Less than - Field value must be strictly less than this number (exclusive)")
        )
        .field(
            InputValue::new("lte", TypeRef::named(TypeRef::FLOAT))
                .description("Less than or equal - Field value must be less than or equal to this number (inclusive)")
        )
        .field(
            InputValue::new("between", TypeRef::named_list(TypeRef::FLOAT))
                .description("Range filter [min, max] - Field value must be between min and max (inclusive) - Provide exactly 2 numbers in the array"),
        )
}

/// Build DateFilter input type
pub fn build_date_filter_type() -> InputObject {
    InputObject::new("DateFilter")
        .description("Date/timestamp comparison operators for DateTime field filtering - All dates must be in RFC3339 format (e.g., '2024-01-15T10:30:00Z') - Specify exactly one operator per filter")
        .field(
            InputValue::new("after", TypeRef::named(TypeRef::STRING))
                .description("After this timestamp (exclusive) - Field value must be strictly after this date/time - RFC3339 format required"),
        )
        .field(
            InputValue::new("before", TypeRef::named(TypeRef::STRING))
                .description("Before this timestamp (exclusive) - Field value must be strictly before this date/time - RFC3339 format required"),
        )
        .field(
            InputValue::new("between", TypeRef::named_list(TypeRef::STRING))
                .description("Date range filter [start, end] (inclusive) - Field value must be between start and end timestamps - Provide exactly 2 RFC3339 formatted strings in the array"),
        )
}

/// Build MessageResult union type for the unified messages query
///
/// This creates a GraphQL union type that can return either:
/// - MessageConnection (for list mode)
/// - AggregationResult (for aggregation mode)
pub fn build_message_result_type() -> Union {
    Union::new("MessageResult")
        .description("Union type for the unified messages query result - Returns MessageConnection (paginated list of messages) when querying in list mode, or AggregationResult (grouped/aggregated statistics) when querying with metrics. Use inline fragments (...on MessageConnection, ...on AggregationResult) to access the appropriate fields.")
        .possible_type("MessageConnection")
        .possible_type("AggregationResult")
}

/// Build MessageStats type
pub fn build_message_stats_type() -> Object {
    use super::types::MessageStats;

    Object::new("MessageStats")
        .description("Aggregate statistics about transformation messages processed by Reframe - Provides insights into transformation success rates, performance metrics, and message type distribution")
        .field(Field::new(
            "totalMessages",
            TypeRef::named_nn(TypeRef::INT),
            |ctx| {
                FieldFuture::new(async move {
                    let stats = ctx.parent_value.try_downcast_ref::<MessageStats>()?;
                    Ok(Some(GqlValue::from(stats.total_messages)))
                })
            },
        ).description("Total number of messages processed and stored in the audit database"))
        .field(Field::new(
            "successCount",
            TypeRef::named_nn(TypeRef::INT),
            |ctx| {
                FieldFuture::new(async move {
                    let stats = ctx.parent_value.try_downcast_ref::<MessageStats>()?;
                    Ok(Some(GqlValue::from(stats.success_count)))
                })
            },
        ).description("Number of messages that were successfully transformed without errors - Messages with empty errors array"))
        .field(Field::new(
            "failureCount",
            TypeRef::named_nn(TypeRef::INT),
            |ctx| {
                FieldFuture::new(async move {
                    let stats = ctx.parent_value.try_downcast_ref::<MessageStats>()?;
                    Ok(Some(GqlValue::from(stats.failure_count)))
                })
            },
        ).description("Number of messages that failed transformation - Messages with one or more errors. Includes parsing errors, validation failures, and workflow errors"))
        .field(Field::new(
            "successRate",
            TypeRef::named_nn(TypeRef::FLOAT),
            |ctx| {
                FieldFuture::new(async move {
                    let stats = ctx.parent_value.try_downcast_ref::<MessageStats>()?;
                    Ok(serde_json::Number::from_f64(stats.success_rate).map(GqlValue::Number))
                })
            },
        ).description("Success rate as a percentage (0.0 to 100.0) - Calculated as (successCount / totalMessages) * 100. Useful for monitoring transformation reliability"))
        .field(Field::new(
            "averageProcessingTimeMs",
            TypeRef::named_nn(TypeRef::FLOAT),
            |ctx| {
                FieldFuture::new(async move {
                    let stats = ctx.parent_value.try_downcast_ref::<MessageStats>()?;
                    Ok(
                        serde_json::Number::from_f64(stats.average_processing_time_ms)
                            .map(GqlValue::Number),
                    )
                })
            },
        ).description("Average processing time in milliseconds from message receipt to storage - Includes parsing, transformation, validation, and custom field computation time. Useful for performance monitoring"))
        .field(Field::new(
            "messageTypeBreakdown",
            TypeRef::named_nn_list_nn("MessageTypeCount"),
            |ctx| {
                FieldFuture::new(async move {
                    let stats = ctx.parent_value.try_downcast_ref::<MessageStats>()?;
                    let breakdown: Vec<FieldValue> = stats
                        .message_type_breakdown
                        .iter()
                        .map(|mtc| FieldValue::owned_any(mtc.clone()))
                        .collect();
                    Ok(Some(FieldValue::list(breakdown)))
                })
            },
        ).description("Distribution of messages by type (e.g., MT103, pacs.008) - Shows count for each message type processed. Useful for understanding message volume patterns"))
}

/// Build MessageTypeCount type
pub fn build_message_type_count_type() -> Object {
    use super::types::MessageTypeCount;

    Object::new("MessageTypeCount")
        .description("Message count statistics for a specific message type - Part of the messageTypeBreakdown in MessageStats")
        .field(Field::new(
            "messageType",
            TypeRef::named_nn(TypeRef::STRING),
            |ctx| {
                FieldFuture::new(async move {
                    let mtc = ctx.parent_value.try_downcast_ref::<MessageTypeCount>()?;
                    Ok(Some(GqlValue::String(mtc.message_type.clone())))
                })
            },
        ).description("Message type identifier (e.g., 'MT103', 'pacs.008', 'MT202', 'camt.052') - Corresponds to the message_type_hint in transformation metadata"))
        .field(Field::new(
            "count",
            TypeRef::named_nn(TypeRef::INT),
            |ctx| {
                FieldFuture::new(async move {
                    let mtc = ctx.parent_value.try_downcast_ref::<MessageTypeCount>()?;
                    Ok(Some(GqlValue::from(mtc.count)))
                })
            },
        ).description("Number of messages of this type that have been processed - Useful for understanding which message types are most common in your transformation pipeline"))
}

/// Build ChangeEntry type
pub fn build_change_entry_type() -> Object {
    use super::types::ChangeEntry;

    Object::new("ChangeEntry")
        .description("Represents a single field modification made by a workflow task during transformation - Tracks what changed, where it changed, and the before/after values for audit purposes")
        .field(
            Field::new("path", TypeRef::named_nn(TypeRef::STRING), |ctx| {
                FieldFuture::new(async move {
                    let entry = ctx.parent_value.try_downcast_ref::<ChangeEntry>()?;
                    Ok(Some(GqlValue::String(entry.path.clone())))
                })
            })
            .description("JSONPath location of the changed field within the transformation context (e.g., 'context.data.Amount', 'context.metadata.direction') - Used to precisely identify which field was modified"),
        )
        .field(
            Field::new("oldValue", TypeRef::named_nn(TypeRef::STRING), |ctx| {
                FieldFuture::new(async move {
                    let entry = ctx.parent_value.try_downcast_ref::<ChangeEntry>()?;
                    Ok(Some(GqlValue::String(
                        serde_json::to_string(&entry.old_value).unwrap_or_default(),
                    )))
                })
            })
            .description("Value of the field before the transformation task executed (JSON string) - Null for newly created fields. Useful for debugging and understanding data flow through the pipeline"),
        )
        .field(
            Field::new("newValue", TypeRef::named_nn(TypeRef::STRING), |ctx| {
                FieldFuture::new(async move {
                    let entry = ctx.parent_value.try_downcast_ref::<ChangeEntry>()?;
                    Ok(Some(GqlValue::String(
                        serde_json::to_string(&entry.new_value).unwrap_or_default(),
                    )))
                })
            })
            .description("Value of the field after the transformation task executed (JSON string) - Null if the field was deleted. Shows the result of the mapping or transformation logic"),
        )
}
