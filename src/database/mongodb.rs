use super::{DatabaseClient, DatabaseConfig, PublishMode};
use async_trait::async_trait;
use dataflow_rs::engine::message::Message;
use mongodb::{
    bson::{doc, Document},
    options::ClientOptions,
    Client, Database,
};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info};

/// MongoDB implementation of DatabaseClient
pub struct MongoDBClient {
    database: Database,
    collection_name: String,
    publish_mode: PublishMode,
}

impl MongoDBClient {
    /// Create a new MongoDB client from generic database configuration
    pub async fn new(config: &DatabaseConfig) -> Result<Self, String> {
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
        })
    }

    /// Serialize a dataflow-rs Message to BSON document
    fn message_to_document(message: &Message) -> Result<Document, String> {
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
