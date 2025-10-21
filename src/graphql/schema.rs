use async_graphql::*;
use std::sync::Arc;

use super::types::*;
use crate::database::mongodb::MongoDBClient;

/// Root query object for the GraphQL API
pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// Get a single transformation message by its ID
    ///
    /// # Arguments
    /// * `id` - The unique message ID
    ///
    /// # Returns
    /// The message if found, or None if not found
    async fn message(
        &self,
        ctx: &Context<'_>,
        id: String,
    ) -> Result<Option<TransformationMessage>> {
        let db_client = ctx.data::<Arc<MongoDBClient>>()?;
        db_client
            .find_message_by_id(&id)
            .await
            .map_err(|e| Error::new(format!("Database error: {}", e)))
    }

    /// List transformation messages with optional filtering and pagination
    ///
    /// # Arguments
    /// * `filter` - Optional filter criteria (message_type, direction, success, date ranges, search, package_id, custom_field_filters)
    /// * `limit` - Maximum number of messages to return (default: 50, max: 1000)
    /// * `offset` - Number of messages to skip for pagination (default: 0)
    /// * `recompute_custom_fields` - If true, recompute custom fields with latest logic for hybrid storage fields (default: false)
    ///
    /// # Returns
    /// A paginated list of messages with total count and pagination info
    async fn messages(
        &self,
        ctx: &Context<'_>,
        filter: Option<MessageFilter>,
        #[graphql(default = 50)] limit: i64,
        #[graphql(default = 0)] offset: i64,
        #[graphql(default = false)] recompute_custom_fields: bool,
    ) -> Result<MessageConnection> {
        let db_client = ctx.data::<Arc<MongoDBClient>>()?;
        let limit = limit.min(1000); // Enforce maximum limit

        db_client
            .find_messages(filter, Some(limit), Some(offset), recompute_custom_fields)
            .await
            .map_err(|e| Error::new(format!("Database error: {}", e)))
    }

    /// Search transformation messages by text query
    ///
    /// Performs full-text search across message content fields.
    ///
    /// # Arguments
    /// * `query` - Text to search for in message content
    /// * `limit` - Maximum number of results to return (default: 50, max: 1000)
    ///
    /// # Returns
    /// A list of matching messages
    async fn search_messages(
        &self,
        ctx: &Context<'_>,
        query: String,
        #[graphql(default = 50)] limit: i64,
    ) -> Result<Vec<TransformationMessage>> {
        let db_client = ctx.data::<Arc<MongoDBClient>>()?;
        let limit = limit.min(1000); // Enforce maximum limit

        db_client
            .search_messages(&query, Some(limit))
            .await
            .map_err(|e| Error::new(format!("Database error: {}", e)))
    }

    /// Get aggregated statistics about transformation messages
    ///
    /// # Returns
    /// Statistics including total messages, success rate, processing time,
    /// and breakdown by message type
    async fn stats(&self, ctx: &Context<'_>) -> Result<MessageStats> {
        let db_client = ctx.data::<Arc<MongoDBClient>>()?;
        db_client
            .get_statistics()
            .await
            .map_err(|e| Error::new(format!("Database error: {}", e)))
    }
}

/// The complete GraphQL schema for Reframe audit API
pub type ReframeSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

/// Create the GraphQL schema with the database client
///
/// # Arguments
/// * `database_client` - The MongoDB client for querying audit data
///
/// # Returns
/// A configured GraphQL schema ready to execute queries
pub fn create_schema(database_client: Arc<MongoDBClient>) -> ReframeSchema {
    Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .data(database_client)
        .finish()
}
