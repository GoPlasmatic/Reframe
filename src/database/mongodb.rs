use super::{DatabaseClient, DatabaseConfig, PublishMode};
use async_trait::async_trait;
use dataflow_rs::engine::message::Message;
use mongodb::{
    Client, Database,
    bson::{Document, doc},
    options::ClientOptions,
};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tracing::{debug, error, info};

// Import GraphQL types for query methods
use crate::graphql::types::{MessageConnection, MessageFilter};

// Import conversion utilities
use super::conversion::document_to_transformation_message;
use crate::package_manager::PackageManager;
use crate::utils::json_to_bson;

/// MongoDB implementation of DatabaseClient
pub struct MongoDBClient {
    database: Database,
    collection_name: String,
    publish_mode: PublishMode,
    package_manager: Arc<RwLock<PackageManager>>,
}

impl MongoDBClient {
    /// Create a new MongoDB client from generic database configuration
    pub async fn new(
        config: &DatabaseConfig,
        package_manager: Arc<RwLock<PackageManager>>,
    ) -> Result<Self, String> {
        info!("🔌 Connecting to MongoDB: {}", config.database_name);

        // Parse connection URI and configure client options
        let mut client_options = ClientOptions::parse(&config.connection_uri)
            .await
            .map_err(|e| format!("Failed to parse MongoDB URI: {}", e))?;

        // Set connection timeout
        let timeout_duration = Duration::from_millis(config.connection_timeout_ms);
        client_options.server_selection_timeout = Some(timeout_duration);

        // Configure connection pool
        if let Some(ref mut pool_options) = client_options.max_pool_size {
            *pool_options = config.max_pool_size;
        } else {
            client_options.max_pool_size = Some(config.max_pool_size);
        }

        if let Some(ref mut pool_options) = client_options.min_pool_size {
            *pool_options = config.min_pool_size;
        } else {
            client_options.min_pool_size = Some(config.min_pool_size);
        }

        // Set max idle time for connections
        let max_idle_time = Duration::from_millis(config.max_idle_time_ms);
        client_options.max_idle_time = Some(max_idle_time);

        // Create client
        let client = Client::with_options(client_options)
            .map_err(|e| format!("Failed to create MongoDB client: {}", e))?;

        // Get database handle
        let database = client.database(&config.database_name);

        // Verify connection with ping
        database
            .run_command(doc! { "ping": 1 })
            .await
            .map_err(|e| format!("Failed to connect to MongoDB: {}", e))?;

        info!(
            "✅ MongoDB connected: {} (collection: {})",
            config.database_name, config.collection_name
        );

        Ok(Self {
            database,
            collection_name: config.collection_name.clone(),
            publish_mode: config.publish_mode.clone(),
            package_manager,
        })
    }

    /// Serialize a dataflow-rs Message to BSON document
    fn message_to_document(message: &Message) -> Result<Document, String> {
        // Log custom fields in context before serialization
        if let Some(custom_fields) = message.context.get("custom_fields") {
            debug!(
                message_id = %message.id,
                custom_fields_present = !custom_fields.is_null(),
                "Message serialized with custom_fields"
            );
        } else {
            debug!(
                message_id = %message.id,
                "Message serialized WITHOUT custom_fields in context"
            );
        }

        // Convert Message to JSON Value
        let json_value = serde_json::to_value(message)
            .map_err(|e| format!("Failed to serialize message to JSON: {}", e))?;

        // Convert JSON Value to BSON Document
        let document = mongodb::bson::to_document(&json_value)
            .map_err(|e| format!("Failed to convert JSON to BSON: {}", e))?;

        Ok(document)
    }

    /// Internal implementation of message publishing
    async fn publish_message_internal(&self, message: &Message) -> Result<(), String> {
        let collection = self.database.collection::<Document>(&self.collection_name);

        // Convert message to BSON document
        let mut document = Self::message_to_document(message)?;

        // Add timestamp if not present
        if !document.contains_key("timestamp") {
            document.insert("timestamp", mongodb::bson::DateTime::now());
        }

        // Insert document
        collection
            .insert_one(document)
            .await
            .map_err(|e| format!("Failed to insert document: {}", e))?;

        debug!(
            message_id = %message.id,
            "Published message to MongoDB"
        );

        Ok(())
    }

    // ========== GraphQL Query Methods ==========

    /// Find messages with filtering, sorting, and pagination
    pub async fn find_messages(
        &self,
        filter: Option<MessageFilter>,
        sort: Option<Vec<crate::graphql::aggregation_types::SortInput>>,
        limit: Option<i64>,
        offset: Option<i64>,
        recompute_custom_fields: bool,
    ) -> Result<MessageConnection, String> {
        let collection = self.database.collection::<Document>(&self.collection_name);

        // Build filter document
        let mut query = doc! {};

        if let Some(f) = filter {
            if let Some(msg_type) = f.message_type {
                query.insert("context.metadata.message_type_hint", msg_type);
            }
            if let Some(direction) = f.direction {
                query.insert("context.metadata.direction", direction);
            }
            if let Some(success) = f.success {
                // Fixed: Check errors array instead of non-existent "success" field
                if !success {
                    // User wants failures - show messages with errors
                    query.insert("errors", doc! { "$exists": true, "$ne": [] });
                } else {
                    // User wants successes - show messages without errors
                    query.insert(
                        "$or",
                        vec![
                            doc! { "errors": { "$exists": false } },
                            doc! { "errors": { "$size": 0 } },
                        ],
                    );
                }
            }
            if let Some(date_from) = f.date_from {
                query.insert(
                    "timestamp",
                    doc! { "$gte": mongodb::bson::DateTime::from_chrono(date_from) },
                );
            }
            if let Some(date_to) = f.date_to {
                let existing_timestamp = query.get("timestamp");
                match existing_timestamp {
                    Some(mongodb::bson::Bson::Document(ts_doc)) => {
                        let mut new_ts = ts_doc.clone();
                        new_ts.insert("$lte", mongodb::bson::DateTime::from_chrono(date_to));
                        query.insert("timestamp", new_ts);
                    }
                    None => {
                        query.insert(
                            "timestamp",
                            doc! { "$lte": mongodb::bson::DateTime::from_chrono(date_to) },
                        );
                    }
                    _ => {}
                }
            }
            if let Some(search) = f.search {
                // Text search requires a text index
                query.insert("$text", doc! { "$search": search });
            }
            let package_id_opt = f.package_id.clone();
            if let Some(ref package_id) = package_id_opt {
                query.insert("context.metadata.package_id", package_id.clone());
            }
            if let Some(custom_filters) = f.custom_field_filters {
                // Apply custom field filters (requires package_id)
                let pkg_id =
                    package_id_opt.ok_or("package_id required when using custom_field_filters")?;
                if let Err(e) = apply_custom_field_filters(&mut query, &custom_filters, &pkg_id) {
                    return Err(format!("Invalid custom field filters: {}", e));
                }
            }

            // Handle PathFilter for dynamic field access
            if let Some(path_filters) = f.path {
                for path_filter in path_filters {
                    let field_path = normalize_field_path(&path_filter.field);
                    apply_path_filter(&mut query, &field_path, &path_filter.value)?;
                }
            }
        }

        // Get total count
        let total_count = collection
            .count_documents(query.clone())
            .await
            .map_err(|e| format!("Failed to count documents: {}", e))?
            as i64;

        // Query with pagination
        let limit = limit.unwrap_or(50).min(1000);
        let offset = offset.unwrap_or(0);

        // Build sort document
        let sort_doc = if let Some(sort_inputs) = sort {
            build_sort_document(&sort_inputs)
        } else {
            // Default sort by timestamp descending
            doc! { "timestamp": -1 }
        };

        let mut cursor = collection
            .find(query)
            .sort(sort_doc)
            .skip(offset as u64)
            .limit(limit)
            .await
            .map_err(|e| format!("MongoDB query failed: {}", e))?;

        let mut messages = Vec::new();
        while cursor
            .advance()
            .await
            .map_err(|e| format!("Cursor error: {}", e))?
        {
            let doc = cursor
                .deserialize_current()
                .map_err(|e| format!("Failed to deserialize document: {}", e))?;
            let message = document_to_transformation_message(doc)?;
            messages.push(message);
        }

        // Compute runtime and hybrid custom fields at query time
        for message in &mut messages {
            if let Some(package_id) = &message.package_id {
                // Get package custom field definitions
                let pm = self.package_manager.read().unwrap();
                if let Some(package) = pm.get_package(package_id.as_str()) {
                    let custom_field_defs = package.custom_fields.clone();
                    drop(pm); // Release lock early

                    // Skip if package has no custom fields
                    if custom_field_defs.is_empty() {
                        continue;
                    }

                    // Compute runtime fields (and hybrid if recompute requested)
                    let computed = crate::custom_fields::compute_runtime_fields(
                        message,
                        &custom_field_defs,
                        recompute_custom_fields,
                    );

                    // Merge computed fields with existing custom_fields
                    if !computed.is_empty() {
                        if let Some(serde_json::Value::Object(ref mut obj)) = message.custom_fields
                        {
                            // Merge: computed fields override stored fields if recompute is true
                            for (key, value) in computed {
                                obj.insert(key, value);
                            }
                        } else {
                            // No existing custom fields, create new object
                            message.custom_fields = Some(
                                serde_json::to_value(&computed).unwrap_or(serde_json::json!({})),
                            );
                        }
                    }
                }
            }
        }

        let has_next_page = (offset + limit) < total_count;

        Ok(MessageConnection {
            messages,
            total_count,
            has_next_page,
        })
    }

    /// Execute aggregation query
    ///
    /// Builds and executes a MongoDB aggregation pipeline based on the provided
    /// filter, groupBy, metrics, sort, and limit parameters.
    pub async fn aggregate(
        &self,
        filter: Option<MessageFilter>,
        group_by: &[crate::graphql::aggregation_types::GroupByInput],
        metrics: &[crate::graphql::aggregation_types::MetricInput],
        sort: Option<&[crate::graphql::aggregation_types::SortInput]>,
        limit: Option<i64>,
    ) -> Result<crate::graphql::aggregation_types::AggregationResult, String> {
        use super::aggregation_builder;
        use super::filter_builder;
        use crate::graphql::aggregation_types::{
            AggregationMetadata, AggregationResult, DataPoint,
        };
        use crate::utils::bson_to_json;

        let collection = self.database.collection::<Document>(&self.collection_name);

        // Build filter query
        let filter_query = if let Some(f) = filter {
            filter_builder::build_mongodb_filter(&f)?
        } else {
            Document::new()
        };

        // Build aggregation pipeline
        let pipeline = aggregation_builder::build_aggregation_pipeline(
            filter_query,
            group_by,
            metrics,
            sort,
            limit,
        )?;

        debug!("Aggregation pipeline: {:?}", pipeline);

        // Execute aggregation
        let mut cursor = collection
            .aggregate(pipeline)
            .await
            .map_err(|e| format!("Aggregation execution failed: {}", e))?;

        // Collect results
        let mut data = Vec::new();
        while cursor
            .advance()
            .await
            .map_err(|e| format!("Cursor error: {}", e))?
        {
            let doc = cursor
                .deserialize_current()
                .map_err(|e| format!("Deserialization error: {}", e))?;

            let group_val = doc
                .get("group")
                .and_then(|v| bson_to_json(v).ok())
                .unwrap_or(serde_json::json!({}));

            let metrics_val = doc
                .get("metrics")
                .and_then(|v| bson_to_json(v).ok())
                .unwrap_or(serde_json::json!({}));

            data.push(DataPoint {
                group: group_val,
                metrics: metrics_val,
            });
        }

        let total_groups = data.len() as i64;

        Ok(AggregationResult {
            data,
            total_groups,
            execution_time_ms: 0.0, // Set by GraphQL resolver
            metadata: AggregationMetadata {
                group_by_fields: group_by
                    .iter()
                    .map(|g| g.r#as.as_ref().unwrap_or(&g.field).clone())
                    .collect(),
                metric_fields: metrics.iter().map(|m| m.r#as.clone()).collect(),
                total_messages: 0, // Optional: add separate count query if needed
            },
        })
    }
}

/// Apply custom field filters to MongoDB query using package-specific accessor
///
/// Converts JSON filter format to MongoDB query syntax.
/// Supports operators: eq, ne, gt, gte, lt, lte, in, exists
///
/// Example input (for package "swift-cbpr-mt-mx"):
/// ```json
/// {
///   "transaction_risk_score": { "gte": 70 },
///   "is_cross_border": true
/// }
/// ```
///
/// Becomes MongoDB query:
/// ```json
/// {
///   "context.swiftCbprMtMxFields.transaction_risk_score": { "$gte": 70 },
///   "context.swiftCbprMtMxFields.is_cross_border": true
/// }
/// ```
fn apply_custom_field_filters(
    query: &mut Document,
    filters: &serde_json::Value,
    package_id: &str,
) -> Result<(), String> {
    use crate::graphql::dynamic_schema::build_accessor_name;

    let filters_obj = filters
        .as_object()
        .ok_or_else(|| "custom_field_filters must be an object".to_string())?;

    // Build package-specific accessor name
    let accessor_name = build_accessor_name(package_id);

    for (field_name, filter_value) in filters_obj {
        let db_field = format!("context.{}.{}", accessor_name, field_name);

        match filter_value {
            // Direct value comparison: {"field": value}
            serde_json::Value::String(s) => {
                query.insert(db_field, s.clone());
            }
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    query.insert(db_field, i);
                } else if let Some(f) = n.as_f64() {
                    query.insert(db_field, f);
                }
            }
            serde_json::Value::Bool(b) => {
                query.insert(db_field, *b);
            }

            // Operator-based filter: {"field": {"gte": 70}}
            serde_json::Value::Object(operators) => {
                let mut filter_doc = Document::new();

                for (op, val) in operators {
                    let mongo_op = match op.as_str() {
                        "eq" => "$eq",
                        "ne" => "$ne",
                        "gt" => "$gt",
                        "gte" => "$gte",
                        "lt" => "$lt",
                        "lte" => "$lte",
                        "in" => "$in",
                        "exists" => "$exists",
                        _ => return Err(format!("Unknown operator: {}", op)),
                    };

                    // Convert JSON value to BSON
                    let bson_val = json_to_bson(val)
                        .map_err(|e| format!("Failed to convert filter value: {}", e))?;
                    filter_doc.insert(mongo_op, bson_val);
                }

                query.insert(db_field, filter_doc);
            }

            _ => {
                return Err(format!("Invalid filter value for field '{}'", field_name));
            }
        }
    }

    Ok(())
}

/// Normalize field path for MongoDB queries
/// Adds "context." prefix to fields that need it
fn normalize_field_path(field: &str) -> String {
    // Root-level fields that don't need context prefix
    const ROOT_FIELDS: &[&str] = &[
        "timestamp",
        "id",
        "_id",
        "errors",
        "audit_trail",
        "package_id",
    ];

    // Check if this is a root field
    if ROOT_FIELDS
        .iter()
        .any(|&rf| field == rf || field.starts_with(&format!("{}.", rf)))
    {
        return field.to_string();
    }

    // If already has context prefix, return as-is
    if field.starts_with("context.") {
        return field.to_string();
    }

    // Add context prefix
    format!("context.{}", field)
}

/// Apply PathFilter to MongoDB query
fn apply_path_filter(
    query: &mut Document,
    field_path: &str,
    filter_value: &crate::graphql::types::FilterValue,
) -> Result<(), String> {
    use mongodb::bson::{Bson, doc};

    // Check for field existence filter
    if let Some(exists) = filter_value.exists {
        query.insert(field_path, doc! { "$exists": exists });
        return Ok(());
    }

    // Handle boolean filter
    if let Some(boolean) = filter_value.boolean {
        query.insert(field_path, boolean);
        return Ok(());
    }

    // Handle string filter
    if let Some(string_filter) = &filter_value.string {
        let mut filter_doc = Document::new();

        if let Some(eq) = &string_filter.eq {
            query.insert(field_path, eq.clone());
            return Ok(());
        }
        if let Some(ne) = &string_filter.ne {
            filter_doc.insert("$ne", ne.clone());
        }
        if let Some(in_vals) = &string_filter.r#in {
            let vals: Vec<Bson> = in_vals.iter().map(|s| Bson::String(s.clone())).collect();
            filter_doc.insert("$in", vals);
        }
        if let Some(contains) = &string_filter.contains {
            // Use regex for substring match (case-insensitive)
            filter_doc.insert("$regex", contains.clone());
            filter_doc.insert("$options", "i");
        }
        if let Some(regex) = &string_filter.regex {
            filter_doc.insert("$regex", regex.clone());
        }

        if !filter_doc.is_empty() {
            query.insert(field_path, filter_doc);
        }
        return Ok(());
    }

    // Handle number filter
    if let Some(number_filter) = &filter_value.number {
        let mut filter_doc = Document::new();

        if let Some(eq) = number_filter.eq {
            query.insert(field_path, eq);
            return Ok(());
        }
        if let Some(ne) = number_filter.ne {
            filter_doc.insert("$ne", ne);
        }
        if let Some(gt) = number_filter.gt {
            filter_doc.insert("$gt", gt);
        }
        if let Some(gte) = number_filter.gte {
            filter_doc.insert("$gte", gte);
        }
        if let Some(lt) = number_filter.lt {
            filter_doc.insert("$lt", lt);
        }
        if let Some(lte) = number_filter.lte {
            filter_doc.insert("$lte", lte);
        }
        if let Some(between) = &number_filter.between {
            if between.len() == 2 {
                filter_doc.insert("$gte", between[0]);
                filter_doc.insert("$lte", between[1]);
            } else {
                return Err("between requires exactly 2 values [min, max]".to_string());
            }
        }

        if !filter_doc.is_empty() {
            query.insert(field_path, filter_doc);
        }
        return Ok(());
    }

    // Handle date filter
    if let Some(date_filter) = &filter_value.date {
        let mut filter_doc = Document::new();

        if let Some(after) = date_filter.after {
            filter_doc.insert("$gt", mongodb::bson::DateTime::from_chrono(after));
        }
        if let Some(before) = date_filter.before {
            filter_doc.insert("$lt", mongodb::bson::DateTime::from_chrono(before));
        }
        if let Some(between) = &date_filter.between {
            if between.len() == 2 {
                filter_doc.insert("$gte", mongodb::bson::DateTime::from_chrono(between[0]));
                filter_doc.insert("$lte", mongodb::bson::DateTime::from_chrono(between[1]));
            } else {
                return Err("between requires exactly 2 dates [start, end]".to_string());
            }
        }

        if !filter_doc.is_empty() {
            query.insert(field_path, filter_doc);
        }
        return Ok(());
    }

    Err("No valid filter value provided".to_string())
}

/// Build MongoDB sort document from SortInput
fn build_sort_document(sort_inputs: &[crate::graphql::aggregation_types::SortInput]) -> Document {
    use crate::graphql::aggregation_types::SortDirection;
    let mut sort_doc = Document::new();

    for sort_input in sort_inputs {
        // Normalize field path
        let field_path = normalize_field_path(&sort_input.field);

        // Convert SortDirection to MongoDB sort direction (1 for ASC, -1 for DESC)
        let direction = match sort_input.direction {
            SortDirection::Asc => 1,
            SortDirection::Desc => -1,
        };

        sort_doc.insert(field_path, direction);
    }

    // If no sort specified or empty, default to timestamp DESC
    if sort_doc.is_empty() {
        sort_doc.insert("timestamp", -1);
    }

    sort_doc
}

#[async_trait]
impl DatabaseClient for MongoDBClient {
    /// Publish a transformation message to MongoDB
    ///
    /// Depending on publish_mode:
    /// - Async: Spawns tokio task (fire-and-forget, no error handling)
    /// - Sync: Blocks until acknowledgement (errors logged but not propagated)
    fn publish_message(self: Arc<Self>, message: &Message) {
        let message_clone = message.clone();

        match self.publish_mode {
            PublishMode::Async => {
                // Fire-and-forget: spawn async task
                tokio::spawn(async move {
                    if let Err(e) = self.publish_message_internal(&message_clone).await {
                        error!(
                            error = %e,
                            message_id = %message_clone.id,
                            "Failed to publish message to MongoDB (async mode)"
                        );
                    }
                });
            }
            PublishMode::Sync => {
                // Block until acknowledgement
                tokio::spawn(async move {
                    if let Err(e) = self.publish_message_internal(&message_clone).await {
                        error!(
                            error = %e,
                            message_id = %message_clone.id,
                            "Failed to publish message to MongoDB (sync mode)"
                        );
                    }
                });
            }
        }
    }

    /// Health check: verify MongoDB connectivity
    async fn ping(&self) -> Result<(), String> {
        self.database
            .run_command(doc! { "ping": 1 })
            .await
            .map_err(|e| format!("MongoDB ping failed: {}", e))?;

        Ok(())
    }
}
