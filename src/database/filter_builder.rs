//! MongoDB Filter Builder
//!
//! Converts GraphQL filter types to MongoDB query documents
//! with support for nested AND/OR/NOT logic

use crate::graphql::filters::{
    EnhancedMessageFilter, JSONPathFilter, StringFilter, TimeRangeFilter,
};
use crate::utils::json_to_bson;
use mongodb::bson::{Document, doc};

/// Build MongoDB query document from EnhancedMessageFilter
///
/// Recursively converts GraphQL filters to MongoDB query syntax,
/// including nested logical operations (AND/OR/NOT)
pub fn build_mongodb_filter(filter: &EnhancedMessageFilter) -> Result<Document, String> {
    let mut query = Document::new();

    // Timestamp filter
    if let Some(ref ts) = filter.timestamp {
        apply_time_range_filter(&mut query, "timestamp", ts)?;
    }

    // Package ID filter
    if let Some(ref pkg) = filter.package_id {
        apply_string_filter(&mut query, "context.metadata.package_id", pkg)?;
    }

    // Message type filter
    if let Some(ref msg_type) = filter.message_type {
        apply_string_filter(&mut query, "context.metadata.message_type_hint", msg_type)?;
    }

    // Direction filter
    if let Some(ref direction) = filter.direction {
        apply_string_filter(&mut query, "context.metadata.direction", direction)?;
    }

    // Has errors filter
    if let Some(has_errors) = filter.has_errors {
        if has_errors {
            query.insert("errors", doc! { "$exists": true, "$ne": [] });
        } else {
            query.insert(
                "$or",
                vec![
                    doc! { "errors": { "$exists": false } },
                    doc! { "errors": { "$size": 0 } },
                ],
            );
        }
    }

    // Search filter (text search)
    if let Some(ref search) = filter.search {
        query.insert("$text", doc! { "$search": search.clone() });
    }

    // Custom fields filter
    if let Some(ref cf) = filter.custom_fields {
        apply_json_path_filter(
            &mut query,
            &format!("context.custom_fields.{}", cf.path),
            cf,
        )?;
    }

    // Context filter
    if let Some(ref ctx) = filter.context {
        apply_json_path_filter(&mut query, &format!("context.{}", ctx.path), ctx)?;
    }

    // Nested AND logic
    if let Some(ref and_filters) = filter.and {
        let and_queries: Result<Vec<Document>, String> =
            and_filters.iter().map(build_mongodb_filter).collect();
        query.insert("$and", and_queries?);
    }

    // Nested OR logic
    if let Some(ref or_filters) = filter.or {
        let or_queries: Result<Vec<Document>, String> =
            or_filters.iter().map(build_mongodb_filter).collect();
        query.insert("$or", or_queries?);
    }

    // NOT logic
    if let Some(ref not_filter) = filter.not {
        let not_query = build_mongodb_filter(not_filter)?;
        query.insert("$nor", vec![not_query]);
    }

    Ok(query)
}

/// Apply time range filter to query
fn apply_time_range_filter(
    query: &mut Document,
    field: &str,
    filter: &TimeRangeFilter,
) -> Result<(), String> {
    let mut ts_doc = Document::new();

    if let Some(gte) = filter.gte {
        ts_doc.insert("$gte", mongodb::bson::DateTime::from_chrono(gte));
    }
    if let Some(lte) = filter.lte {
        ts_doc.insert("$lte", mongodb::bson::DateTime::from_chrono(lte));
    }
    if let Some(gt) = filter.gt {
        ts_doc.insert("$gt", mongodb::bson::DateTime::from_chrono(gt));
    }
    if let Some(lt) = filter.lt {
        ts_doc.insert("$lt", mongodb::bson::DateTime::from_chrono(lt));
    }

    if !ts_doc.is_empty() {
        query.insert(field, ts_doc);
    }

    Ok(())
}

/// Apply string filter to query
fn apply_string_filter(
    query: &mut Document,
    field: &str,
    filter: &StringFilter,
) -> Result<(), String> {
    if let Some(ref eq) = filter.eq {
        query.insert(field, eq.clone());
    }
    if let Some(ref ne) = filter.ne {
        query.insert(field, doc! { "$ne": ne.clone() });
    }
    if let Some(ref in_vals) = filter.r#in {
        query.insert(field, doc! { "$in": in_vals.clone() });
    }
    if let Some(ref nin_vals) = filter.not_in {
        query.insert(field, doc! { "$nin": nin_vals.clone() });
    }
    if let Some(ref regex) = filter.regex {
        query.insert(field, doc! { "$regex": regex.clone() });
    }
    if let Some(ref contains) = filter.contains {
        query.insert(field, doc! { "$regex": contains.clone(), "$options": "i" });
    }
    if let Some(ref starts) = filter.starts_with {
        query.insert(field, doc! { "$regex": format!("^{}", starts) });
    }
    if let Some(ref ends) = filter.ends_with {
        query.insert(field, doc! { "$regex": format!("{}$", ends) });
    }
    Ok(())
}

/// Apply JSON path filter to query
fn apply_json_path_filter(
    query: &mut Document,
    field: &str,
    filter: &JSONPathFilter,
) -> Result<(), String> {
    if let Some(ref eq) = filter.eq {
        query.insert(field, json_to_bson(eq)?);
    }
    if let Some(ref ne) = filter.ne {
        query.insert(field, doc! { "$ne": json_to_bson(ne)? });
    }
    if let Some(gt) = filter.gt {
        query.insert(field, doc! { "$gt": gt });
    }
    if let Some(gte) = filter.gte {
        query.insert(field, doc! { "$gte": gte });
    }
    if let Some(lt) = filter.lt {
        query.insert(field, doc! { "$lt": lt });
    }
    if let Some(lte) = filter.lte {
        query.insert(field, doc! { "$lte": lte });
    }
    if let Some(ref in_vals) = filter.r#in {
        let bson_vals: Result<Vec<_>, _> = in_vals.iter().map(json_to_bson).collect();
        query.insert(field, doc! { "$in": bson_vals? });
    }
    if let Some(ref nin_vals) = filter.not_in {
        let bson_vals: Result<Vec<_>, _> = nin_vals.iter().map(json_to_bson).collect();
        query.insert(field, doc! { "$nin": bson_vals? });
    }
    if let Some(exists) = filter.exists {
        query.insert(field, doc! { "$exists": exists });
    }
    Ok(())
}
