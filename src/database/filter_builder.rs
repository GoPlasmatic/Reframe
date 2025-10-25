//! MongoDB Filter Builder
//!
//! Converts GraphQL filter types to MongoDB query documents

use crate::database::constants::fields;
use crate::database::constants::paths;
use crate::database::field_utils::normalize_field_path;
use crate::graphql::types::{
    DateFilter, FilterValue, MessageFilter, NumberFilter, PathFilter,
    StringFilter as PathStringFilter,
};
use crate::utils::json_to_bson;
use mongodb::bson::{Document, doc};

/// Build MongoDB query document from MessageFilter
///
/// Converts GraphQL filters to MongoDB query syntax
pub fn build_mongodb_filter(filter: &MessageFilter) -> Result<Document, String> {
    let mut query = Document::new();

    // Message type filter
    if let Some(ref msg_type) = filter.message_type {
        query.insert(fields::MESSAGE_TYPE_HINT, msg_type.clone());
    }

    // Direction filter
    if let Some(ref direction) = filter.direction {
        query.insert(fields::DIRECTION, direction.clone());
    }

    // Success filter (inverted to has_errors)
    if let Some(success) = filter.success {
        if !success {
            query.insert(fields::ERRORS, doc! { "$exists": true, "$ne": [] });
        } else {
            query.insert(
                "$or",
                vec![
                    doc! { fields::ERRORS: { "$exists": false } },
                    doc! { fields::ERRORS: { "$size": 0 } },
                ],
            );
        }
    }

    // Date range filter
    if filter.date_from.is_some() || filter.date_to.is_some() {
        let mut ts_doc = Document::new();
        if let Some(date_from) = filter.date_from {
            ts_doc.insert("$gte", mongodb::bson::DateTime::from_chrono(date_from));
        }
        if let Some(date_to) = filter.date_to {
            ts_doc.insert("$lte", mongodb::bson::DateTime::from_chrono(date_to));
        }
        if !ts_doc.is_empty() {
            query.insert(fields::TIMESTAMP, ts_doc);
        }
    }

    // Search filter (text search)
    if let Some(ref search) = filter.search {
        query.insert("$text", doc! { "$search": search.clone() });
    }

    // Package ID filter
    if let Some(ref pkg_id) = filter.package_id {
        query.insert(fields::PACKAGE_ID, pkg_id.clone());
    }

    // Custom field filters
    if let Some(ref custom_filters) = filter.custom_field_filters {
        apply_custom_field_filters(&mut query, custom_filters)?;
    }

    // Path filters (dot notation)
    if let Some(ref path_filters) = filter.path {
        apply_path_filters(&mut query, path_filters)?;
    }

    Ok(query)
}

/// Apply custom field filters to query
fn apply_custom_field_filters(
    query: &mut Document,
    custom_filters: &serde_json::Value,
) -> Result<(), String> {
    // Custom field filters apply to context.custom_fields.{field_name}
    if let Some(obj) = custom_filters.as_object() {
        for (field_name, filter_value) in obj {
            let field_path = paths::custom_field(field_name);

            if let Some(filter_obj) = filter_value.as_object() {
                let mut field_doc = Document::new();

                // Handle different filter operators
                if let Some(eq) = filter_obj.get("eq") {
                    query.insert(&field_path, json_to_bson(eq)?);
                } else if let Some(ne) = filter_obj.get("ne") {
                    field_doc.insert("$ne", json_to_bson(ne)?);
                    query.insert(&field_path, field_doc);
                } else if let Some(gte) = filter_obj.get("gte") {
                    field_doc.insert("$gte", json_to_bson(gte)?);
                    query.insert(&field_path, field_doc);
                } else if let Some(lte) = filter_obj.get("lte") {
                    field_doc.insert("$lte", json_to_bson(lte)?);
                    query.insert(&field_path, field_doc);
                } else if let Some(gt) = filter_obj.get("gt") {
                    field_doc.insert("$gt", json_to_bson(gt)?);
                    query.insert(&field_path, field_doc);
                } else if let Some(lt) = filter_obj.get("lt") {
                    field_doc.insert("$lt", json_to_bson(lt)?);
                    query.insert(&field_path, field_doc);
                } else if let Some(exists) = filter_obj.get("exists") {
                    field_doc.insert("$exists", json_to_bson(exists)?);
                    query.insert(&field_path, field_doc);
                }
            } else {
                // Direct value comparison
                query.insert(&field_path, json_to_bson(filter_value)?);
            }
        }
    }

    Ok(())
}

/// Apply path filters (dot notation) to query
fn apply_path_filters(query: &mut Document, path_filters: &[PathFilter]) -> Result<(), String> {
    for path_filter in path_filters {
        let field = normalize_field_path(&path_filter.field);
        apply_filter_value(&mut *query, &field, &path_filter.value)?;
    }

    Ok(())
}

/// Apply a FilterValue to the query
fn apply_filter_value(
    query: &mut Document,
    field: &str,
    value: &FilterValue,
) -> Result<(), String> {
    // String filter
    if let Some(ref string_filter) = value.string {
        apply_path_string_filter(query, field, string_filter)?;
    }

    // Number filter
    if let Some(ref number_filter) = value.number {
        apply_number_filter(query, field, number_filter)?;
    }

    // Boolean filter
    if let Some(boolean_value) = value.boolean {
        query.insert(field, boolean_value);
    }

    // Date filter
    if let Some(ref date_filter) = value.date {
        apply_date_filter(query, field, date_filter)?;
    }

    // Exists filter
    if let Some(exists) = value.exists {
        query.insert(field, doc! { "$exists": exists });
    }

    Ok(())
}

/// Apply string filter to query
fn apply_path_string_filter(
    query: &mut Document,
    field: &str,
    filter: &PathStringFilter,
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
    if let Some(ref contains) = filter.contains {
        query.insert(field, doc! { "$regex": contains.clone(), "$options": "i" });
    }
    if let Some(ref regex) = filter.regex {
        query.insert(field, doc! { "$regex": regex.clone() });
    }
    Ok(())
}

/// Apply number filter to query
fn apply_number_filter(
    query: &mut Document,
    field: &str,
    filter: &NumberFilter,
) -> Result<(), String> {
    let mut field_doc = Document::new();

    if let Some(eq) = filter.eq {
        query.insert(field, eq);
        return Ok(());
    }

    if let Some(ne) = filter.ne {
        field_doc.insert("$ne", ne);
    }
    if let Some(gt) = filter.gt {
        field_doc.insert("$gt", gt);
    }
    if let Some(gte) = filter.gte {
        field_doc.insert("$gte", gte);
    }
    if let Some(lt) = filter.lt {
        field_doc.insert("$lt", lt);
    }
    if let Some(lte) = filter.lte {
        field_doc.insert("$lte", lte);
    }
    if let Some(ref between) = filter.between
        && between.len() == 2
    {
        field_doc.insert("$gte", between[0]);
        field_doc.insert("$lte", between[1]);
    }

    if !field_doc.is_empty() {
        query.insert(field, field_doc);
    }

    Ok(())
}

/// Apply date filter to query
fn apply_date_filter(query: &mut Document, field: &str, filter: &DateFilter) -> Result<(), String> {
    let mut field_doc = Document::new();

    if let Some(after) = filter.after {
        field_doc.insert("$gt", mongodb::bson::DateTime::from_chrono(after));
    }
    if let Some(before) = filter.before {
        field_doc.insert("$lt", mongodb::bson::DateTime::from_chrono(before));
    }
    if let Some(ref between) = filter.between
        && between.len() == 2
    {
        field_doc.insert("$gte", mongodb::bson::DateTime::from_chrono(between[0]));
        field_doc.insert("$lte", mongodb::bson::DateTime::from_chrono(between[1]));
    }

    if !field_doc.is_empty() {
        query.insert(field, field_doc);
    }

    Ok(())
}
