//! Custom error types for Reframe
//!
//! This module defines structured error types using thiserror for better error handling
//! and context throughout the application.

use thiserror::Error;

/// Top-level error type for all Reframe errors
#[derive(Error, Debug)]
pub enum ReframeError {
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),

    #[error("GraphQL error: {0}")]
    GraphQL(#[from] GraphQLError),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Conversion error: {0}")]
    Conversion(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Database-specific errors
#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Query failed: {0}")]
    QueryFailed(String),

    #[error("Document not found: {0}")]
    NotFound(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid field value: {field} - {reason}")]
    InvalidField { field: String, reason: String },

    #[error("MongoDB error: {0}")]
    MongoDB(#[from] mongodb::error::Error),

    #[error("BSON serialization error: {0}")]
    BsonSerialization(#[from] mongodb::bson::ser::Error),

    #[error("BSON deserialization error: {0}")]
    BsonDeserialization(#[from] mongodb::bson::de::Error),
}

/// GraphQL-specific errors
#[derive(Error, Debug)]
pub enum GraphQLError {
    #[error("Schema build error: {0}")]
    SchemaBuild(String),

    #[error("Type conversion error: {0}")]
    TypeConversion(String),

    #[error("Field resolution error: {0}")]
    FieldResolution(String),

    #[error("Invalid query: {0}")]
    InvalidQuery(String),
}

// Implement conversion to String for backward compatibility
impl From<DatabaseError> for String {
    fn from(err: DatabaseError) -> String {
        err.to_string()
    }
}

impl From<GraphQLError> for String {
    fn from(err: GraphQLError) -> String {
        err.to_string()
    }
}

impl From<ReframeError> for String {
    fn from(err: ReframeError) -> String {
        err.to_string()
    }
}

// Helper type alias
pub type DatabaseResult<T> = Result<T, DatabaseError>;
