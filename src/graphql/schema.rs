use async_graphql::Error;
use async_graphql::dynamic::{self, Field, FieldFuture, FieldValue, Object, Schema, TypeRef};
use std::sync::Arc;

use super::dynamic_schema;
use super::types::*;
use crate::database::mongodb::MongoDBClient;
use crate::package_manager::PackageManager;

/// The complete GraphQL schema for Reframe audit API (fully dynamic)
pub type ReframeSchema = dynamic::Schema;

/// Build the Query type with all query fields
///
/// This creates a dynamic Query type with resolvers for:
/// - message(id: String): TransformationMessage
/// - messages(filter, limit, offset, recompute_custom_fields): MessageConnection
/// - searchMessages(query, limit): [TransformationMessage]
/// - stats: MessageStats
fn build_query_type() -> Object {
    Object::new("Query")
        // message(id: String): TransformationMessage
        .field(
            Field::new("message", TypeRef::named("TransformationMessage"), |ctx| {
                FieldFuture::new(async move {
                    let id = ctx.args.try_get("id")?.string()?.to_string();
                    let db_client = ctx.data::<Arc<MongoDBClient>>()?;

                    let msg = db_client
                        .find_message_by_id(&id)
                        .await
                        .map_err(|e| Error::new(format!("Database error: {}", e)))?;

                    Ok(msg.map(FieldValue::owned_any))
                })
            })
            .argument(dynamic::InputValue::new(
                "id",
                TypeRef::named_nn(TypeRef::STRING),
            )),
        )
        // messages(filter, limit, offset, recompute_custom_fields): MessageConnection
        .field(
            Field::new("messages", TypeRef::named_nn("MessageConnection"), |ctx| {
                FieldFuture::new(async move {
                    let filter = ctx.args.try_get("filter").ok().and_then(|v| {
                        serde_json::from_value::<MessageFilter>(v.deserialize().ok()?).ok()
                    });
                    let limit = ctx
                        .args
                        .try_get("limit")
                        .and_then(|v| v.i64())
                        .unwrap_or(50)
                        .min(1000);
                    let offset = ctx
                        .args
                        .try_get("offset")
                        .and_then(|v| v.i64())
                        .unwrap_or(0);
                    let recompute = ctx
                        .args
                        .try_get("recomputeCustomFields")
                        .and_then(|v| v.boolean())
                        .unwrap_or(false);

                    let db_client = ctx.data::<Arc<MongoDBClient>>()?;

                    let connection = db_client
                        .find_messages(filter, Some(limit), Some(offset), recompute)
                        .await
                        .map_err(|e| Error::new(format!("Database error: {}", e)))?;

                    Ok(Some(FieldValue::owned_any(connection)))
                })
            })
            .argument(dynamic::InputValue::new(
                "filter",
                TypeRef::named("MessageFilter"),
            ))
            .argument(
                dynamic::InputValue::new("limit", TypeRef::named(TypeRef::INT)).default_value(50),
            )
            .argument(
                dynamic::InputValue::new("offset", TypeRef::named(TypeRef::INT)).default_value(0),
            )
            .argument(
                dynamic::InputValue::new("recomputeCustomFields", TypeRef::named(TypeRef::BOOLEAN))
                    .default_value(false),
            ),
        )
        // searchMessages(query: String!, limit: Int): [TransformationMessage]
        .field(
            Field::new(
                "searchMessages",
                TypeRef::named_nn_list_nn("TransformationMessage"),
                |ctx| {
                    FieldFuture::new(async move {
                        let query = ctx.args.try_get("query")?.string()?.to_string();
                        let limit = ctx
                            .args
                            .try_get("limit")
                            .and_then(|v| v.i64())
                            .unwrap_or(50)
                            .min(1000);

                        let db_client = ctx.data::<Arc<MongoDBClient>>()?;

                        let messages = db_client
                            .search_messages(&query, Some(limit))
                            .await
                            .map_err(|e| Error::new(format!("Database error: {}", e)))?;

                        let result: Vec<FieldValue> =
                            messages.into_iter().map(FieldValue::owned_any).collect();

                        Ok(Some(FieldValue::list(result)))
                    })
                },
            )
            .argument(dynamic::InputValue::new(
                "query",
                TypeRef::named_nn(TypeRef::STRING),
            ))
            .argument(
                dynamic::InputValue::new("limit", TypeRef::named(TypeRef::INT)).default_value(50),
            ),
        )
        // stats: MessageStats
        .field(Field::new(
            "stats",
            TypeRef::named_nn("MessageStats"),
            |ctx| {
                FieldFuture::new(async move {
                    let db_client = ctx.data::<Arc<MongoDBClient>>()?;

                    let stats = db_client
                        .get_statistics()
                        .await
                        .map_err(|e| Error::new(format!("Database error: {}", e)))?;

                    Ok(Some(FieldValue::owned_any(stats)))
                })
            },
        ))
}

/// Create the fully dynamic GraphQL schema
///
/// This creates a schema with:
/// - Dynamic Query type with resolver methods
/// - Dynamic TransformationMessage type with package-specific custom field accessors
/// - Dynamic custom field types for each package (e.g., SwiftCbprMtMxFields)
/// - Dynamic supporting types (MessageConnection, MessageStats, etc.)
///
/// # Arguments
/// * `database_client` - The MongoDB client for querying audit data
/// * `package_manager` - The package manager for accessing custom field definitions
///
/// # Returns
/// A configured GraphQL schema ready to execute queries
pub fn create_schema(
    database_client: Arc<MongoDBClient>,
    package_manager: Arc<std::sync::RwLock<PackageManager>>,
) -> ReframeSchema {
    let pm = package_manager.read().unwrap();

    // Build dynamic schema
    let mut schema_builder = Schema::build("Query", None, None);

    // Register Query type
    let query_type = build_query_type();
    schema_builder = schema_builder.register(query_type);

    // Build and register dynamic TransformationMessage type
    let message_type = dynamic_schema::build_transformation_message_type(pm.get_packages());
    schema_builder = schema_builder.register(message_type);

    // Register scalar types first (used by other types)
    schema_builder = schema_builder
        .register(dynamic_schema::build_json_scalar())
        .register(dynamic_schema::build_datetime_scalar());

    // Register dynamic custom field types for each package
    for (package_id, package) in pm.get_packages() {
        if !package.custom_fields.is_empty() {
            let fields_type = dynamic_schema::build_package_custom_fields_type(
                package_id,
                &package.name,
                &package.custom_fields,
            );
            schema_builder = schema_builder.register(fields_type);
        }
    }

    // Register other dynamic types needed
    schema_builder = schema_builder
        .register(dynamic_schema::build_context_type())
        .register(dynamic_schema::build_message_connection_type())
        .register(dynamic_schema::build_audit_trail_entry_type())
        .register(dynamic_schema::build_error_info_entry_type())
        .register(dynamic_schema::build_message_stats_type())
        .register(dynamic_schema::build_message_type_count_type())
        .register(dynamic_schema::build_change_entry_type())
        .register(dynamic_schema::build_message_filter_type());

    // Add context data
    schema_builder = schema_builder
        .data(database_client)
        .data(package_manager.clone());

    schema_builder
        .finish()
        .expect("Failed to build GraphQL schema")
}
