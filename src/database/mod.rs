pub mod conversion;
pub mod mongodb;

// Re-export MongoDBClient for GraphQL
pub use mongodb::MongoDBClient;

use async_trait::async_trait;
use dataflow_rs::engine::message::Message;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Supported database types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseType {
    #[serde(rename = "mongodb")]
    MongoDB,
    // Future support:
    // PostgreSQL,
    // MySQL,
    // Cassandra,
}

/// Publishing mode determines how messages are persisted
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PublishMode {
    /// Fire-and-forget: Spawn async task (fastest, no error handling)
    Async,
    /// Wait for acknowledgement (slower, reliable)
    Sync,
}

/// Generic database client trait
///
/// All database implementations must implement this trait to provide
/// a consistent interface for message persistence and querying
#[async_trait]
pub trait DatabaseClient: Send + Sync {
    /// Publish a transformation message to the database
    ///
    /// This method serializes the dataflow-rs Message object and stores it.
    /// The message ID is used as the primary key/partition key.
    fn publish_message(self: Arc<Self>, message: &Message);

    /// Health check: verify database connectivity
    async fn ping(&self) -> Result<(), String>;
}

/// Generic database configuration
///
/// This structure is database-agnostic and can be used to configure
/// any supported database backend (MongoDB, PostgreSQL, etc.)
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Database type (mongodb, postgresql, etc.)
    pub db_type: DatabaseType,

    /// Connection URI
    pub connection_uri: String,

    /// Database name
    pub database_name: String,

    /// Collection/table name for storing transformation messages
    pub collection_name: String,

    /// Connection timeout in milliseconds
    pub connection_timeout_ms: u64,

    /// Maximum connection pool size
    pub max_pool_size: u32,

    /// Minimum connection pool size
    pub min_pool_size: u32,

    /// Maximum idle time for pooled connections (milliseconds)
    pub max_idle_time_ms: u64,

    /// Publishing mode (async or sync)
    pub publish_mode: PublishMode,

    /// Enable/disable persistence for transform operations
    pub persist_transform: bool,

    /// Enable/disable persistence for validate operations
    pub persist_validate: bool,

    /// Enable/disable persistence for generate operations
    pub persist_generate: bool,
}

impl DatabaseConfig {
    /// Load database configuration from config file and environment variables
    pub fn load() -> Self {
        // Load config from file
        let config_content = if std::path::Path::new("reframe.config.json").exists() {
            std::fs::read_to_string("reframe.config.json").ok()
        } else {
            None
        };

        let config: Option<serde_json::Value> =
            config_content.and_then(|c| serde_json::from_str(&c).ok());

        let db_config = config
            .as_ref()
            .and_then(|c| c.get("database"))
            .and_then(|v| v.as_object());

        // Get database type
        let db_type_str = std::env::var("DB_TYPE")
            .ok()
            .or_else(|| db_config.and_then(|d| d.get("type")?.as_str().map(String::from)))
            .unwrap_or_else(|| "mongodb".to_string());

        let db_type = match db_type_str.to_lowercase().as_str() {
            "mongodb" => DatabaseType::MongoDB,
            _ => panic!("Unsupported database type: {}", db_type_str),
        };

        // Get connection settings
        let connection = db_config
            .and_then(|d| d.get("connection"))
            .and_then(|v| v.as_object());

        let connection_uri = std::env::var("DB_URI")
            .ok()
            .or_else(|| connection.and_then(|c| c.get("uri")?.as_str().map(String::from)))
            .unwrap_or_else(|| "mongodb://localhost:27017".to_string());

        let connection_timeout_ms = std::env::var("DB_CONNECTION_TIMEOUT_MS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .or_else(|| connection.and_then(|c| c.get("timeout_ms")?.as_u64()))
            .unwrap_or(5000);

        // Get pool settings
        let pool = connection
            .and_then(|c| c.get("pool"))
            .and_then(|v| v.as_object());

        let max_pool_size = std::env::var("DB_POOL_MAX_SIZE")
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .or_else(|| pool.and_then(|p| p.get("max_size")?.as_u64().map(|v| v as u32)))
            .unwrap_or(10);

        let min_pool_size = std::env::var("DB_POOL_MIN_SIZE")
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .or_else(|| pool.and_then(|p| p.get("min_size")?.as_u64().map(|v| v as u32)))
            .unwrap_or(2);

        let max_idle_time_ms = std::env::var("DB_MAX_IDLE_TIME_MS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .or_else(|| pool.and_then(|p| p.get("max_idle_time_ms")?.as_u64()))
            .unwrap_or(60000);

        // Get storage settings
        let storage = db_config
            .and_then(|d| d.get("storage"))
            .and_then(|v| v.as_object());

        let database_name = std::env::var("DB_NAME")
            .ok()
            .or_else(|| storage.and_then(|s| s.get("database")?.as_str().map(String::from)))
            .unwrap_or_else(|| "reframe".to_string());

        let collection_name = std::env::var("DB_COLLECTION")
            .ok()
            .or_else(|| storage.and_then(|s| s.get("collection")?.as_str().map(String::from)))
            .unwrap_or_else(|| "reframe_audit".to_string());

        // Get options
        let options = db_config
            .and_then(|d| d.get("options"))
            .and_then(|v| v.as_object());

        let publish_mode_str = std::env::var("DB_PUBLISH_MODE")
            .ok()
            .or_else(|| options.and_then(|o| o.get("publish_mode")?.as_str().map(String::from)))
            .unwrap_or_else(|| "async".to_string());

        let publish_mode = match publish_mode_str.as_str() {
            "sync" => PublishMode::Sync,
            _ => PublishMode::Async,
        };

        let persist_transform = std::env::var("DB_PERSIST_TRANSFORM")
            .ok()
            .and_then(|s| s.parse::<bool>().ok())
            .or_else(|| options.and_then(|o| o.get("persist_transform")?.as_bool()))
            .unwrap_or(true);

        let persist_validate = std::env::var("DB_PERSIST_VALIDATE")
            .ok()
            .and_then(|s| s.parse::<bool>().ok())
            .or_else(|| options.and_then(|o| o.get("persist_validate")?.as_bool()))
            .unwrap_or(false);

        let persist_generate = std::env::var("DB_PERSIST_GENERATE")
            .ok()
            .and_then(|s| s.parse::<bool>().ok())
            .or_else(|| options.and_then(|o| o.get("persist_generate")?.as_bool()))
            .unwrap_or(false);

        Self {
            db_type,
            connection_uri,
            database_name,
            collection_name,
            connection_timeout_ms,
            max_pool_size,
            min_pool_size,
            max_idle_time_ms,
            publish_mode,
            persist_transform,
            persist_validate,
            persist_generate,
        }
    }
}

/// Factory function to create a database client based on configuration
///
/// This allows easy extension to support additional database types
/// without modifying consumer code
pub async fn create_database_client(
    config: &DatabaseConfig,
) -> Result<Arc<dyn DatabaseClient>, String> {
    match config.db_type {
        DatabaseType::MongoDB => {
            let client = mongodb::MongoDBClient::new(config).await?;
            Ok(Arc::new(client))
        } // Future implementations:
          // DatabaseType::PostgreSQL => {
          //     let client = postgresql::PostgreSQLClient::new(config).await?;
          //     Ok(Arc::new(client))
          // }
    }
}
