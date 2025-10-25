use async_graphql::Error;
use async_graphql::dynamic::{self, Field, FieldFuture, FieldValue, Object, Schema, TypeRef};
use std::sync::Arc;

use super::types::*;
use super::{aggregation_types, dynamic_schema};
use crate::database::mongodb::MongoDBClient;
use crate::package_manager::PackageManager;

/// The complete GraphQL schema for Reframe audit API (fully dynamic)
pub type ReframeSchema = dynamic::Schema;

/// Build the Query type with the unified messages query
///
/// This creates a dynamic Query type with a single unified resolver:
/// - messages(filter, sort, limit, offset, groupBy, metrics): MessageResult
///   Returns either MessageConnection (list mode) or AggregationResult (aggregate mode)
fn build_query_type() -> Object {
    Object::new("Query")
        // messages(filter, sort, limit, offset, aggregate): MessageResult (unified query)
        .field(
            Field::new("messages", TypeRef::named_nn("MessageResult"), |ctx| {
                FieldFuture::new(async move {
                    use crate::graphql::aggregation_types;

                    // Get filter
                    let filter = ctx.args.try_get("filter").ok().and_then(|v| {
                        serde_json::from_value::<MessageFilter>(v.deserialize().ok()?).ok()
                    });

                    // Check if this is an aggregation query
                    let group_by: Option<Vec<aggregation_types::GroupByInput>> =
                        ctx.args.try_get("groupBy").ok().and_then(|v| {
                            let deserialized = v.deserialize().ok()?;
                            serde_json::from_value(deserialized).ok()
                        });

                    let metrics: Option<Vec<aggregation_types::MetricInput>> =
                        ctx.args.try_get("metrics").ok().and_then(|v| {
                            let deserialized = v.deserialize().ok()?;
                            serde_json::from_value(deserialized).ok()
                        });

                    // Determine mode: aggregate if metrics are provided
                    let is_aggregate_mode = metrics.is_some();

                    let db_client = ctx.data::<Arc<MongoDBClient>>()?;

                    if is_aggregate_mode {
                        // AGGREGATE MODE
                        let metrics = metrics
                            .ok_or_else(|| Error::new("metrics required for aggregation"))?;
                        let group_by = group_by.unwrap_or_default();

                        let sort = ctx.args.try_get("sort").ok().and_then(|v| {
                            serde_json::from_value::<Vec<aggregation_types::SortInput>>(
                                v.deserialize().ok()?,
                            )
                            .ok()
                        });

                        let limit = ctx
                            .args
                            .try_get("limit")
                            .and_then(|v| v.i64())
                            .ok()
                            .map(|l| l.min(1000));

                        let result = db_client
                            .aggregate(filter, &group_by, &metrics, sort.as_deref(), limit)
                            .await
                            .map_err(|e| Error::new(format!("Aggregation error: {}", e)))?;

                        // Return AggregationResult wrapped with type info for union
                        Ok(Some(FieldValue::with_type(
                            FieldValue::owned_any(result),
                            "AggregationResult",
                        )))
                    } else {
                        // LIST MODE (default)
                        let sort = ctx.args.try_get("sort").ok().and_then(|v| {
                            serde_json::from_value::<
                                Vec<crate::graphql::aggregation_types::SortInput>,
                            >(v.deserialize().ok()?)
                            .ok()
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

                        let connection = db_client
                            .find_messages(filter, sort, Some(limit), Some(offset), recompute)
                            .await
                            .map_err(|e| Error::new(format!("Database error: {}", e)))?;

                        // Return MessageConnection wrapped with type info for union
                        Ok(Some(FieldValue::with_type(
                            FieldValue::owned_any(connection),
                            "MessageConnection",
                        )))
                    }
                })
            })
            .argument(dynamic::InputValue::new(
                "filter",
                TypeRef::named("MessageFilter"),
            ))
            .argument(dynamic::InputValue::new(
                "sort",
                TypeRef::named_list("SortInput"),
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
            )
            // Aggregation arguments
            .argument(dynamic::InputValue::new(
                "groupBy",
                TypeRef::named_list("GroupByInput"),
            ))
            .argument(dynamic::InputValue::new(
                "metrics",
                TypeRef::named_list("MetricInput"),
            )),
        )
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

    // Register other dynamic types needed (pass packages for context custom fields)
    schema_builder = schema_builder
        .register(dynamic_schema::build_context_type(pm.get_packages()))
        .register(dynamic_schema::build_message_connection_type())
        .register(dynamic_schema::build_message_result_type()) // Unified query result type
        .register(dynamic_schema::build_audit_trail_entry_type())
        .register(dynamic_schema::build_error_info_entry_type())
        .register(dynamic_schema::build_message_stats_type())
        .register(dynamic_schema::build_message_type_count_type())
        .register(dynamic_schema::build_change_entry_type())
        .register(dynamic_schema::build_message_filter_type())
        // Register new PathFilter types
        .register(dynamic_schema::build_path_filter_type())
        .register(dynamic_schema::build_filter_value_type())
        .register(dynamic_schema::build_string_filter_type())
        .register(dynamic_schema::build_number_filter_type())
        .register(dynamic_schema::build_date_filter_type());

    // Register aggregation input types
    schema_builder = schema_builder
        .register(aggregation_types::build_group_by_input())
        .register(aggregation_types::build_time_bucket_input())
        .register(aggregation_types::build_numeric_bucket_input())
        .register(aggregation_types::build_bucket_strategy_enum())
        .register(aggregation_types::build_field_transform_input())
        .register(aggregation_types::build_date_part_enum())
        .register(aggregation_types::build_metric_input())
        .register(aggregation_types::build_agg_function_enum())
        .register(aggregation_types::build_sort_input())
        .register(aggregation_types::build_sort_direction_enum());

    // Register aggregation output types
    schema_builder = schema_builder
        .register(aggregation_types::build_aggregation_result_type())
        .register(aggregation_types::build_data_point_type())
        .register(aggregation_types::build_aggregation_metadata_type());

    // Add context data
    schema_builder = schema_builder
        .data(database_client)
        .data(package_manager.clone());

    schema_builder
        .finish()
        .expect("Failed to build GraphQL schema")
}
