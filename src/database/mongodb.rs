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
use crate::graphql::types::{
    MessageConnection, MessageFilter, MessageStats, MessageTypeCount, TransformationMessage,
};

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

    /// Find a single message by ID
    pub async fn find_message_by_id(
        &self,
        id: &str,
    ) -> Result<Option<TransformationMessage>, String> {
        let collection = self.database.collection::<Document>(&self.collection_name);

        let filter = doc! { "id": id };
        let result = collection
            .find_one(filter)
            .await
            .map_err(|e| format!("MongoDB query failed: {}", e))?;

        match result {
            Some(doc) => {
                let message = document_to_transformation_message(doc)?;
                Ok(Some(message))
            }
            None => Ok(None),
        }
    }

    /// Find messages with filtering and pagination
    pub async fn find_messages(
        &self,
        filter: Option<MessageFilter>,
        limit: Option<i64>,
        offset: Option<i64>,
        recompute_custom_fields: bool,
    ) -> Result<MessageConnection, String> {
        let collection = self.database.collection::<Document>(&self.collection_name);

        // Build filter document
        let mut query = doc! {};

        if let Some(f) = filter {
            if let Some(msg_type) = f.message_type {
                query.insert("metadata.message_type_hint", msg_type);
            }
            if let Some(direction) = f.direction {
                query.insert("metadata.transformation_direction", direction);
            }
            if let Some(success) = f.success {
                query.insert("success", success);
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
            if let Some(package_id) = f.package_id {
                query.insert("context.metadata.package_id", package_id);
            }
            if let Some(custom_filters) = f.custom_field_filters {
                // Apply custom field filters
                if let Err(e) = apply_custom_field_filters(&mut query, &custom_filters) {
                    return Err(format!("Invalid custom field filters: {}", e));
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

        let mut cursor = collection
            .find(query)
            .sort(doc! { "timestamp": -1 })
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

    /// Search messages using text search
    pub async fn search_messages(
        &self,
        query: &str,
        limit: Option<i64>,
    ) -> Result<Vec<TransformationMessage>, String> {
        let collection = self.database.collection::<Document>(&self.collection_name);

        let search_filter = doc! { "$text": { "$search": query } };
        let limit = limit.unwrap_or(50).min(1000);

        let mut cursor = collection
            .find(search_filter)
            .limit(limit)
            .await
            .map_err(|e| format!("MongoDB search failed: {}", e))?;

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

        Ok(messages)
    }

    /// Get aggregated statistics
    pub async fn get_statistics(&self) -> Result<MessageStats, String> {
        let collection = self.database.collection::<Document>(&self.collection_name);

        // Total messages
        let total_messages = collection
            .count_documents(doc! {})
            .await
            .map_err(|e| format!("Failed to count documents: {}", e))?
            as i64;

        // Success/failure counts based on errors array
        // Success = errors array is empty or doesn't exist
        let success_count = collection
            .count_documents(doc! {
                "$or": [
                    { "errors": { "$exists": false } },
                    { "errors": { "$size": 0 } }
                ]
            })
            .await
            .map_err(|e| format!("Failed to count successes: {}", e))?
            as i64;

        let failure_count = total_messages - success_count;

        let success_rate = if total_messages > 0 {
            (success_count as f64 / total_messages as f64) * 100.0
        } else {
            0.0
        };

        // Average processing time from context.metadata.processing_time_ms
        let pipeline = vec![
            doc! {
                "$match": {
                    "context.metadata.processing_time_ms": { "$exists": true }
                }
            },
            doc! {
                "$group": {
                    "_id": null,
                    "avg_time": { "$avg": "$context.metadata.processing_time_ms" }
                }
            },
        ];

        let mut cursor = collection
            .aggregate(pipeline)
            .await
            .map_err(|e| format!("Aggregation failed: {}", e))?;

        let mut average_processing_time_ms = 0.0;
        if cursor
            .advance()
            .await
            .map_err(|e| format!("Cursor error: {}", e))?
        {
            let doc = cursor
                .deserialize_current()
                .map_err(|e| format!("Failed to deserialize: {}", e))?;
            if let Some(avg) = doc.get("avg_time") {
                average_processing_time_ms = avg.as_f64().unwrap_or(0.0);
            }
        }

        // Message type breakdown - aggregate by direction and message type
        let pipeline = vec![
            doc! {
                "$project": {
                    "direction": "$context.metadata.direction",
                    "mx_type": "$context.metadata.ISO20022_MX.message_type",
                    "mt_type": "$context.data.SwiftMT.message_type"
                }
            },
            doc! {
                "$project": {
                    "message_type": {
                        "$cond": {
                            "if": { "$eq": ["$direction", "outgoing"] },
                            "then": { "$concat": ["MT", { "$toString": "$mt_type" }, " → MX"] },
                            "else": {
                                "$cond": {
                                    "if": { "$ne": ["$mx_type", null] },
                                    "then": { "$concat": ["MX:", "$mx_type", " → MT"] },
                                    "else": "unknown"
                                }
                            }
                        }
                    }
                }
            },
            doc! {
                "$group": {
                    "_id": "$message_type",
                    "count": { "$sum": 1 }
                }
            },
            doc! {
                "$sort": { "count": -1 }
            },
        ];

        let mut cursor = collection
            .aggregate(pipeline)
            .await
            .map_err(|e| format!("Aggregation failed: {}", e))?;

        let mut message_type_breakdown = Vec::new();
        while cursor
            .advance()
            .await
            .map_err(|e| format!("Cursor error: {}", e))?
        {
            let doc = cursor
                .deserialize_current()
                .map_err(|e| format!("Failed to deserialize: {}", e))?;

            let message_type = doc
                .get("_id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            let count = doc.get("count").and_then(|v| v.as_i32()).unwrap_or(0) as i64;

            message_type_breakdown.push(MessageTypeCount {
                message_type,
                count,
            });
        }

        Ok(MessageStats {
            total_messages,
            success_count,
            failure_count,
            success_rate,
            average_processing_time_ms,
            message_type_breakdown,
        })
    }
}

/// Apply custom field filters to MongoDB query
///
/// Converts JSON filter format to MongoDB query syntax.
/// Supports operators: eq, ne, gt, gte, lt, lte, in, exists
///
/// Example input:
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
///   "context.custom_fields.transaction_risk_score": { "$gte": 70 },
///   "context.custom_fields.is_cross_border": true
/// }
/// ```
fn apply_custom_field_filters(
    query: &mut Document,
    filters: &serde_json::Value,
) -> Result<(), String> {
    let filters_obj = filters
        .as_object()
        .ok_or_else(|| "custom_field_filters must be an object".to_string())?;

    for (field_name, filter_value) in filters_obj {
        let db_field = format!("context.custom_fields.{}", field_name);

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
