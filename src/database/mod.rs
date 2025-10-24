pub mod aggregation_builder;
pub mod conversion;
pub mod filter_builder;
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

        // Get database config section
        let db_config_section = config.as_ref().and_then(|c| c.get("database"));

        // Create ConfigLoader
        let loader = crate::utils::ConfigLoader::new(db_config_section);

        // Get database type
        let db_type_str = loader.get_string("DB_TYPE", &["type"], "mongodb");
        let db_type = match db_type_str.to_lowercase().as_str() {
            "mongodb" => DatabaseType::MongoDB,
            _ => panic!("Unsupported database type: {}", db_type_str),
        };

        // Get connection settings using ConfigLoader
        let connection_uri = loader.get_string(
            "DB_URI",
            &["connection", "uri"],
            "mongodb://localhost:27017",
        );

        let connection_timeout_ms = loader.get_u64(
            "DB_CONNECTION_TIMEOUT_MS",
            &["connection", "timeout_ms"],
            5000,
        );

        // Get pool settings
        let max_pool_size =
            loader.get_u32("DB_POOL_MAX_SIZE", &["connection", "pool", "max_size"], 10);
        let min_pool_size =
            loader.get_u32("DB_POOL_MIN_SIZE", &["connection", "pool", "min_size"], 2);
        let max_idle_time_ms = loader.get_u64(
            "DB_MAX_IDLE_TIME_MS",
            &["connection", "pool", "max_idle_time_ms"],
            60000,
        );

        // Get storage settings
        let database_name = loader.get_string("DB_NAME", &["storage", "database"], "reframe");
        let collection_name =
            loader.get_string("DB_COLLECTION", &["storage", "collection"], "reframe_audit");

        // Get publish mode
        let publish_mode_str =
            loader.get_string("DB_PUBLISH_MODE", &["options", "publish_mode"], "async");
        let publish_mode = match publish_mode_str.as_str() {
            "sync" => PublishMode::Sync,
            _ => PublishMode::Async,
        };

        // Get persistence options
        let persist_transform = loader.get_bool(
            "DB_PERSIST_TRANSFORM",
            &["options", "persist_transform"],
            true,
        );
        let persist_validate = loader.get_bool(
            "DB_PERSIST_VALIDATE",
            &["options", "persist_validate"],
            false,
        );
        let persist_generate = loader.get_bool(
            "DB_PERSIST_GENERATE",
            &["options", "persist_generate"],
            false,
        );

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
    package_manager: std::sync::Arc<std::sync::RwLock<crate::package_manager::PackageManager>>,
) -> Result<Arc<dyn DatabaseClient>, String> {
    match config.db_type {
        DatabaseType::MongoDB => {
            let client = mongodb::MongoDBClient::new(config, package_manager).await?;
            Ok(Arc::new(client))
        } // Future implementations:
          // DatabaseType::PostgreSQL => {
          //     let client = postgresql::PostgreSQLClient::new(config).await?;
          //     Ok(Arc::new(client))
          // }
    }
}
