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
    obj = obj.description(format!("Custom fields for package: {}", package_name));

    // Add each field from api_config.json
    for def in definitions {
        let field_name_camel = to_camel_case(&def.name);
        let field_type = map_field_type(&def.field_type);

        let description = format!(
            "{}\n\nType: {}, Storage: {:?}",
            def.description, def.field_type, def.storage
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
pub fn build_transformation_message_type(packages: &HashMap<String, PackageInfo>) -> Object {
    let mut obj = Object::new("TransformationMessage");
    obj = obj.description("Transformation message stored in the audit database");

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
            .description("Unique message ID (UUID)"),
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
            .description("Package ID used for this transformation"),
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
            .description("Timestamp when stored (RFC3339 format)"),
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
            .description("Original payload (input message) as JSON string"),
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
            .description("Unified context containing data and metadata"),
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
            .description("Audit trail of workflow and task executions"),
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
            .description("Errors that occurred during processing"),
        );

    // Add package-specific custom field accessors
    for (package_id, package) in packages {
        if package.custom_fields.is_empty() {
            continue;
        }

        let type_name = format!("{}Fields", to_pascal_case(package_id));
        let accessor_name = build_accessor_name(package_id);
        let pkg_id = package_id.clone();

        let field = Field::new(&accessor_name, TypeRef::named(&type_name), move |ctx| {
            let package_id_filter = pkg_id.clone();
            FieldFuture::new(async move {
                let msg = ctx
                    .parent_value
                    .try_downcast_ref::<TransformationMessage>()?;

                // Only return data if message's packageId matches
                if msg.package_id.as_ref() != Some(&package_id_filter) {
                    return Ok(None);
                }

                // Parse custom_fields JSON to HashMap
                let custom_fields_map: Option<HashMap<String, JsonValue>> = msg
                    .custom_fields
                    .as_ref()
                    .and_then(|json| serde_json::from_value(json.clone()).ok());

                Ok(custom_fields_map.map(|map| FieldValue::owned_any(map)))
            })
        })
        .description(format!("Custom fields for {} package", package.name));

        obj = obj.field(field);
    }

    obj
}

/// Build Context type with data and metadata fields
pub fn build_context_type() -> Object {
    Object::new("Context")
        .description("Unified context containing data and metadata")
        .field(
            Field::new("data", TypeRef::named_nn("JSON"), |ctx| {
                FieldFuture::new(async move {
                    let context = ctx
                        .parent_value
                        .try_downcast_ref::<serde_json::Map<String, JsonValue>>()?;
                    Ok(context.get("data").and_then(json_to_graphql_value))
                })
            })
            .description("Data field as JSON object"),
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
            .description("Metadata field as JSON object"),
        )
}

/// Build AuditTrailEntry type
pub fn build_audit_trail_entry_type() -> Object {
    use super::types::AuditTrailEntry;

    Object::new("AuditTrailEntry")
        .description("Audit trail entry showing workflow/task execution")
        .field(
            Field::new("workflowId", TypeRef::named_nn(TypeRef::STRING), |ctx| {
                FieldFuture::new(async move {
                    let entry = ctx.parent_value.try_downcast_ref::<AuditTrailEntry>()?;
                    Ok(Some(GqlValue::String(entry.workflow_id.clone())))
                })
            })
            .description("Workflow ID that executed"),
        )
        .field(
            Field::new("taskId", TypeRef::named_nn(TypeRef::STRING), |ctx| {
                FieldFuture::new(async move {
                    let entry = ctx.parent_value.try_downcast_ref::<AuditTrailEntry>()?;
                    Ok(Some(GqlValue::String(entry.task_id.clone())))
                })
            })
            .description("Task ID within the workflow"),
        )
        .field(
            Field::new("timestamp", TypeRef::named_nn("DateTime"), |ctx| {
                FieldFuture::new(async move {
                    let entry = ctx.parent_value.try_downcast_ref::<AuditTrailEntry>()?;
                    Ok(Some(GqlValue::String(entry.timestamp.to_rfc3339())))
                })
            })
            .description("When this task executed (RFC3339 format)"),
        )
        .field(
            Field::new("status", TypeRef::named_nn(TypeRef::INT), |ctx| {
                FieldFuture::new(async move {
                    let entry = ctx.parent_value.try_downcast_ref::<AuditTrailEntry>()?;
                    Ok(Some(GqlValue::from(entry.status)))
                })
            })
            .description("Status code (0 = success, non-zero = error)"),
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
            .description("Changes made by this task"),
        )
}

/// Build ErrorInfoEntry type
pub fn build_error_info_entry_type() -> Object {
    use super::types::ErrorInfoEntry;

    Object::new("ErrorInfoEntry")
        .description("Error information from message processing")
        .field(
            Field::new("code", TypeRef::named_nn(TypeRef::STRING), |ctx| {
                FieldFuture::new(async move {
                    let entry = ctx.parent_value.try_downcast_ref::<ErrorInfoEntry>()?;
                    Ok(Some(GqlValue::String(entry.code.clone())))
                })
            })
            .description("Error code (e.g., VALIDATION_ERROR, WORKFLOW_ERROR)"),
        )
        .field(
            Field::new("message", TypeRef::named_nn(TypeRef::STRING), |ctx| {
                FieldFuture::new(async move {
                    let entry = ctx.parent_value.try_downcast_ref::<ErrorInfoEntry>()?;
                    Ok(Some(GqlValue::String(entry.message.clone())))
                })
            })
            .description("Human-readable error message"),
        )
        .field(
            Field::new("path", TypeRef::named(TypeRef::STRING), |ctx| {
                FieldFuture::new(async move {
                    let entry = ctx.parent_value.try_downcast_ref::<ErrorInfoEntry>()?;
                    Ok(entry.path.as_ref().map(|s| GqlValue::String(s.clone())))
                })
            })
            .description("Optional path to error location"),
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
            .description("Workflow ID where error occurred"),
        )
        .field(
            Field::new("taskId", TypeRef::named(TypeRef::STRING), |ctx| {
                FieldFuture::new(async move {
                    let entry = ctx.parent_value.try_downcast_ref::<ErrorInfoEntry>()?;
                    Ok(entry.task_id.as_ref().map(|s| GqlValue::String(s.clone())))
                })
            })
            .description("Task ID where error occurred"),
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
            .description("Error timestamp (RFC3339 format)"),
        )
        .field(
            Field::new("retryAttempted", TypeRef::named(TypeRef::BOOLEAN), |ctx| {
                FieldFuture::new(async move {
                    let entry = ctx.parent_value.try_downcast_ref::<ErrorInfoEntry>()?;
                    Ok(entry.retry_attempted.map(GqlValue::Boolean))
                })
            })
            .description("Whether retry was attempted"),
        )
        .field(
            Field::new("retryCount", TypeRef::named(TypeRef::INT), |ctx| {
                FieldFuture::new(async move {
                    let entry = ctx.parent_value.try_downcast_ref::<ErrorInfoEntry>()?;
                    Ok(entry.retry_count.map(GqlValue::from))
                })
            })
            .description("Number of retries"),
        )
}

/// Build MessageConnection type for paginated results
pub fn build_message_connection_type() -> Object {
    use super::types::MessageConnection;

    Object::new("MessageConnection")
        .description("Paginated response for messages query")
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
        ))
        .field(Field::new(
            "totalCount",
            TypeRef::named_nn(TypeRef::INT),
            |ctx| {
                FieldFuture::new(async move {
                    let conn = ctx.parent_value.try_downcast_ref::<MessageConnection>()?;
                    Ok(Some(GqlValue::from(conn.total_count)))
                })
            },
        ))
        .field(Field::new(
            "hasNextPage",
            TypeRef::named_nn(TypeRef::BOOLEAN),
            |ctx| {
                FieldFuture::new(async move {
                    let conn = ctx.parent_value.try_downcast_ref::<MessageConnection>()?;
                    Ok(Some(GqlValue::Boolean(conn.has_next_page)))
                })
            },
        ))
}

/// Build MessageFilter input type
pub fn build_message_filter_type() -> InputObject {
    InputObject::new("MessageFilter")
        .description("Filter criteria for querying messages")
        .field(InputValue::new(
            "messageType",
            TypeRef::named(TypeRef::STRING),
        ))
        .field(InputValue::new(
            "direction",
            TypeRef::named(TypeRef::STRING),
        ))
        .field(InputValue::new("success", TypeRef::named(TypeRef::BOOLEAN)))
        .field(
            InputValue::new("dateFrom", TypeRef::named(TypeRef::STRING))
                .description("Filter messages from this date onwards (RFC3339 format)"),
        )
        .field(
            InputValue::new("dateTo", TypeRef::named(TypeRef::STRING))
                .description("Filter messages up to this date (RFC3339 format)"),
        )
        .field(InputValue::new("search", TypeRef::named(TypeRef::STRING)))
        .field(InputValue::new(
            "packageId",
            TypeRef::named(TypeRef::STRING),
        ))
        .field(
            InputValue::new("customFieldFilters", TypeRef::named(TypeRef::STRING))
                .description("Custom field filters as JSON string"),
        )
}

/// Build MessageStats type
pub fn build_message_stats_type() -> Object {
    use super::types::MessageStats;

    Object::new("MessageStats")
        .description("Statistics about transformation messages")
        .field(Field::new(
            "totalMessages",
            TypeRef::named_nn(TypeRef::INT),
            |ctx| {
                FieldFuture::new(async move {
                    let stats = ctx.parent_value.try_downcast_ref::<MessageStats>()?;
                    Ok(Some(GqlValue::from(stats.total_messages)))
                })
            },
        ))
        .field(Field::new(
            "successCount",
            TypeRef::named_nn(TypeRef::INT),
            |ctx| {
                FieldFuture::new(async move {
                    let stats = ctx.parent_value.try_downcast_ref::<MessageStats>()?;
                    Ok(Some(GqlValue::from(stats.success_count)))
                })
            },
        ))
        .field(Field::new(
            "failureCount",
            TypeRef::named_nn(TypeRef::INT),
            |ctx| {
                FieldFuture::new(async move {
                    let stats = ctx.parent_value.try_downcast_ref::<MessageStats>()?;
                    Ok(Some(GqlValue::from(stats.failure_count)))
                })
            },
        ))
        .field(Field::new(
            "successRate",
            TypeRef::named_nn(TypeRef::FLOAT),
            |ctx| {
                FieldFuture::new(async move {
                    let stats = ctx.parent_value.try_downcast_ref::<MessageStats>()?;
                    Ok(serde_json::Number::from_f64(stats.success_rate).map(GqlValue::Number))
                })
            },
        ))
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
        ))
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
        ))
}

/// Build MessageTypeCount type
pub fn build_message_type_count_type() -> Object {
    use super::types::MessageTypeCount;

    Object::new("MessageTypeCount")
        .description("Count of messages by type")
        .field(Field::new(
            "messageType",
            TypeRef::named_nn(TypeRef::STRING),
            |ctx| {
                FieldFuture::new(async move {
                    let mtc = ctx.parent_value.try_downcast_ref::<MessageTypeCount>()?;
                    Ok(Some(GqlValue::String(mtc.message_type.clone())))
                })
            },
        ))
        .field(Field::new(
            "count",
            TypeRef::named_nn(TypeRef::INT),
            |ctx| {
                FieldFuture::new(async move {
                    let mtc = ctx.parent_value.try_downcast_ref::<MessageTypeCount>()?;
                    Ok(Some(GqlValue::from(mtc.count)))
                })
            },
        ))
}

/// Build ChangeEntry type
pub fn build_change_entry_type() -> Object {
    use super::types::ChangeEntry;

    Object::new("ChangeEntry")
        .description("Change made by a workflow task")
        .field(
            Field::new("path", TypeRef::named_nn(TypeRef::STRING), |ctx| {
                FieldFuture::new(async move {
                    let entry = ctx.parent_value.try_downcast_ref::<ChangeEntry>()?;
                    Ok(Some(GqlValue::String(entry.path.clone())))
                })
            })
            .description("JSONPath to the changed field"),
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
            .description("Value before the change (JSON string)"),
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
            .description("Value after the change (JSON string)"),
        )
}
