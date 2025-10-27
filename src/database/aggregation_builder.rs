//! MongoDB Aggregation Pipeline Builder
//!
//! Converts GraphQL aggregation queries to MongoDB aggregation pipelines
//! with support for grouping, time bucketing, metrics, and transformations

use crate::database::field_utils::normalize_field_path;
use crate::graphql::aggregation_types::{
    AggFunction, DatePart, FieldTransform, GroupByInput, MetricInput, SortDirection, SortInput,
    TimeBucketInput,
};
use mongodb::bson::{Bson, Document, doc};
use std::collections::HashMap;

/// Build a nested Document structure from a dot-notation path
///
/// Converts a field path like "context.metadata.direction" with a value reference
/// into a nested Document structure:
/// { "context": { "metadata": { "direction": "$_id.__grp_context_metadata_direction" } } }
fn build_nested_document(path: &str, value: &str) -> Document {
    let parts: Vec<&str> = path.split('.').collect();

    if parts.len() == 1 {
        // No nesting needed
        return doc! { path: value };
    }

    // Build from innermost to outermost
    let mut current = doc! { parts[parts.len() - 1]: value };

    for i in (0..parts.len() - 1).rev() {
        current = doc! { parts[i]: current };
    }

    current
}

/// Merge two Documents recursively, combining nested structures
fn merge_documents(target: &mut Document, source: Document) {
    for (key, value) in source {
        if let Some(existing) = target.get_mut(&key) {
            // Key exists - try to merge if both are documents
            if let (Some(existing_doc), Some(value_doc)) =
                (existing.as_document_mut(), value.as_document())
            {
                // Both are documents - merge recursively
                merge_documents(existing_doc, value_doc.clone());
            } else {
                // Not both documents - replace
                *existing = value;
            }
        } else {
            // Key doesn't exist - insert
            target.insert(key, value);
        }
    }
}

/// Sanitize a field name to make it MongoDB-safe for use in $group._id keys
///
/// MongoDB doesn't allow dots (.) in field names used as keys in the $group._id object.
/// This function replaces dots with underscores and adds a prefix to avoid collisions.
///
/// # Examples
/// ```
/// sanitize_field_alias("context.metadata.direction") -> "__grp_context_metadata_direction"
/// sanitize_field_alias("timestamp") -> "__grp_timestamp"
/// ```
fn sanitize_field_alias(field: &str) -> String {
    format!("__grp_{}", field.replace(['.', '$'], "_"))
}

/// Build MongoDB aggregation pipeline from aggregation inputs
///
/// Creates a multi-stage pipeline that:
/// 1. Filters documents ($match)
/// 2. Adds computed fields for grouping ($addFields)
/// 3. Groups and calculates metrics ($group)
/// 4. Projects to final shape ($project)
/// 5. Sorts results ($sort)
/// 6. Limits results ($limit)
pub fn build_aggregation_pipeline(
    filter_query: Document,
    group_by: &[GroupByInput],
    metrics: &[MetricInput],
    sort: Option<&[SortInput]>,
    limit: Option<i64>,
) -> Result<Vec<Document>, String> {
    let mut pipeline = Vec::new();

    // Stage 1: Match (filter)
    if !filter_query.is_empty() {
        pipeline.push(doc! { "$match": filter_query });
    }

    // Build mapping of original field names to sanitized aliases
    // This is needed because MongoDB $group._id keys cannot contain dots
    let mut field_mapping: HashMap<String, String> = HashMap::new();

    for group_by_input in group_by {
        let original_field = group_by_input
            .r#as
            .as_ref()
            .unwrap_or(&group_by_input.field);
        let sanitized_alias = sanitize_field_alias(original_field);
        field_mapping.insert(original_field.clone(), sanitized_alias);
    }

    // Stage 2: Add computed fields for grouping
    let mut add_fields = Document::new();

    for group_by_input in group_by {
        let field_path = normalize_field_path(&group_by_input.field);
        let original_field = group_by_input
            .r#as
            .as_ref()
            .unwrap_or(&group_by_input.field);
        let sanitized_alias = field_mapping.get(original_field).unwrap();

        if let Some(ref time_bucket) = group_by_input.time_bucket {
            add_fields.insert(
                sanitized_alias,
                build_time_bucket_expression(&field_path, time_bucket)?,
            );
        } else if let Some(ref transform) = group_by_input.transform {
            add_fields.insert(
                sanitized_alias,
                build_transform_expression(&field_path, transform)?,
            );
        } else {
            // Just add a reference to the field
            add_fields.insert(sanitized_alias, format!("${}", field_path));
        }
    }

    if !add_fields.is_empty() {
        pipeline.push(doc! { "$addFields": add_fields });
    }

    // Stage 3: Group
    let mut group_doc = Document::new();
    let mut group_id = Document::new();

    for group_by_input in group_by {
        let original_field = group_by_input
            .r#as
            .as_ref()
            .unwrap_or(&group_by_input.field);
        let sanitized_alias = field_mapping.get(original_field).unwrap();
        // Use sanitized alias as both key and value reference
        group_id.insert(sanitized_alias, format!("${}", sanitized_alias));
    }

    if group_id.is_empty() {
        group_doc.insert("_id", Bson::Null);
    } else {
        group_doc.insert("_id", group_id);
    }

    // Add metrics
    for metric in metrics {
        let metric_expr = build_metric_expression(metric)?;
        group_doc.insert(&metric.r#as, metric_expr);
    }

    pipeline.push(doc! { "$group": group_doc });

    // Stage 4: Sort (before projection, using sanitized field names)
    // This must happen before the final projection because MongoDB can't sort
    // by field names containing dots when used as object keys
    if let Some(sort_inputs) = sort {
        let mut sort_doc = Document::new();
        for sort_input in sort_inputs {
            let direction = match sort_input.direction {
                SortDirection::Asc => 1,
                SortDirection::Desc => -1,
            };

            // Check if sorting by group field or metric
            if group_by
                .iter()
                .any(|g| g.r#as.as_ref().unwrap_or(&g.field) == &sort_input.field)
            {
                // Use sanitized alias for sorting
                let sanitized_alias = field_mapping.get(&sort_input.field).unwrap();
                sort_doc.insert(format!("_id.{}", sanitized_alias), direction);
            } else {
                sort_doc.insert(&sort_input.field, direction);
            }
        }
        pipeline.push(doc! { "$sort": sort_doc });
    }

    // Stage 5: Project to final shape
    // Build nested Document structures to properly handle fields with overlapping paths
    let mut project_doc = Document::new();
    project_doc.insert("_id", 0);

    if !group_by.is_empty() {
        // Build group object with properly nested structures
        // For fields like "context.metadata.direction" and "context.metadata.message_type_hint",
        // we need to manually build the nested structure and merge them
        let mut group_obj = Document::new();
        for group_by_input in group_by {
            let original_field = group_by_input
                .r#as
                .as_ref()
                .unwrap_or(&group_by_input.field);
            let sanitized_alias = field_mapping.get(original_field).unwrap();
            // Build nested document for this field
            let nested_doc =
                build_nested_document(original_field, &format!("$_id.{}", sanitized_alias));
            // Merge into group_obj
            merge_documents(&mut group_obj, nested_doc);
        }
        project_doc.insert("group", group_obj);
    } else {
        project_doc.insert("group", doc! {});
    }

    let mut metrics_obj = Document::new();
    for metric in metrics {
        metrics_obj.insert(&metric.r#as, format!("${}", metric.r#as));
    }
    project_doc.insert("metrics", metrics_obj);

    pipeline.push(doc! { "$project": project_doc });

    // Stage 6: Limit
    if let Some(limit_val) = limit {
        pipeline.push(doc! { "$limit": limit_val });
    }

    Ok(pipeline)
}

/// Build time bucket expression for MongoDB
///
/// Uses MongoDB's $dateTrunc operator for single units (1m, 1h, 1d)
/// or manual bucketing for custom intervals (5m, 15m, 30m, etc.)
fn build_time_bucket_expression(
    field: &str,
    time_bucket: &TimeBucketInput,
) -> Result<Bson, String> {
    let (amount, unit) = parse_interval_with_amount(&time_bucket.interval)?;
    let timezone = time_bucket.timezone.as_deref().unwrap_or("UTC");

    // For single units (1m, 1h, etc.), use $dateTrunc (more efficient)
    if amount == 1 {
        Ok(Bson::Document(doc! {
            "$dateTrunc": {
                "date": format!("${}", field),
                "unit": unit,
                "timezone": timezone
            }
        }))
    } else {
        // For custom intervals (5m, 15m, etc.), use manual bucketing
        // Calculate interval in milliseconds
        let interval_ms = match unit.as_str() {
            "second" => amount * 1000,
            "minute" => amount * 60 * 1000,
            "hour" => amount * 60 * 60 * 1000,
            "day" => amount * 24 * 60 * 60 * 1000,
            _ => {
                return Err(format!(
                    "Unsupported unit for custom intervals: {}. Only s, m, h, d are supported for custom intervals.",
                    unit
                ));
            }
        };

        // Manual bucketing:
        // 1. Convert date to milliseconds since epoch ($toLong)
        // 2. Divide by interval_ms and floor to get bucket number
        // 3. Multiply back by interval_ms to get bucket start time
        // 4. Convert back to date ($toDate)
        Ok(Bson::Document(doc! {
            "$toDate": {
                "$multiply": [
                    {
                        "$floor": {
                            "$divide": [
                                { "$toLong": format!("${}", field) },
                                interval_ms
                            ]
                        }
                    },
                    interval_ms
                ]
            }
        }))
    }
}

/// Parse interval with amount (e.g., "5m" -> (5, "minute"))
///
/// Supports formats:
/// - "5m" -> (5, "minute")
/// - "1h" -> (1, "hour")
/// - "15m" -> (15, "minute")
/// - "m" -> (1, "minute")  // defaults to 1
fn parse_interval_with_amount(interval: &str) -> Result<(i64, String), String> {
    if interval.is_empty() {
        return Err("Empty interval".to_string());
    }

    let unit_char = interval.chars().last().unwrap();
    let amount_str = &interval[..interval.len() - 1];

    let amount: i64 = if amount_str.is_empty() {
        1 // Default to 1 if no number specified
    } else {
        amount_str
            .parse()
            .map_err(|_| format!("Invalid interval amount: {}", amount_str))?
    };

    // Convert single-char unit to MongoDB date unit name
    let unit = match unit_char {
        's' => "second",
        'm' => "minute",
        'h' => "hour",
        'd' => "day",
        'w' => "week",
        'M' => "month",
        'y' => "year",
        _ => {
            return Err(format!(
                "Invalid interval unit: {}. Supported units: s, m, h, d, w, M, y",
                unit_char
            ));
        }
    };

    Ok((amount, unit.to_string()))
}

/// Build field transformation expression
fn build_transform_expression(field: &str, transform: &FieldTransform) -> Result<Bson, String> {
    if let Some(ref date_part) = transform.date_part {
        return build_date_part_expression(field, date_part);
    }

    if let Some(true) = transform.to_lower {
        return Ok(Bson::Document(doc! { "$toLower": format!("${}", field) }));
    }

    if let Some(true) = transform.to_upper {
        return Ok(Bson::Document(doc! { "$toUpper": format!("${}", field) }));
    }

    if let Some(decimals) = transform.round {
        return Ok(Bson::Document(doc! {
            "$round": [format!("${}", field), decimals]
        }));
    }

    Err("No valid transformation specified".to_string())
}

/// Build date part extraction expression
fn build_date_part_expression(field: &str, date_part: &DatePart) -> Result<Bson, String> {
    let expr = match date_part {
        DatePart::Year => doc! { "$year": format!("${}", field) },
        DatePart::Month => doc! { "$month": format!("${}", field) },
        DatePart::Day => doc! { "$dayOfMonth": format!("${}", field) },
        DatePart::Hour => doc! { "$hour": format!("${}", field) },
        DatePart::Minute => doc! { "$minute": format!("${}", field) },
        DatePart::DayOfWeek => doc! { "$dayOfWeek": format!("${}", field) },
        DatePart::DayOfYear => doc! { "$dayOfYear": format!("${}", field) },
        DatePart::WeekOfYear => doc! { "$week": format!("${}", field) },
        DatePart::Quarter => {
            // Quarter = ceil(month / 3)
            doc! {
                "$ceil": {
                    "$divide": [
                        { "$month": format!("${}", field) },
                        3
                    ]
                }
            }
        }
    };

    Ok(Bson::Document(expr))
}

/// Build metric aggregation expression
fn build_metric_expression(metric: &MetricInput) -> Result<Bson, String> {
    let field_path = if let Some(ref field) = metric.field {
        normalize_field_path(field)
    } else {
        "$$ROOT".to_string()
    };

    let expr = match metric.function {
        AggFunction::Count => doc! { "$sum": 1 },
        AggFunction::Sum => doc! { "$sum": format!("${}", field_path) },
        AggFunction::Avg => doc! { "$avg": format!("${}", field_path) },
        AggFunction::Min => doc! { "$min": format!("${}", field_path) },
        AggFunction::Max => doc! { "$max": format!("${}", field_path) },
        AggFunction::Median => {
            // MongoDB 5.0+ $percentile operator
            doc! {
                "$percentile": {
                    "input": format!("${}", field_path),
                    "p": [0.5],
                    "method": "approximate"
                }
            }
        }
        AggFunction::Percentile => {
            let p = metric
                .percentile
                .ok_or("Percentile value required for PERCENTILE function")?;
            doc! {
                "$percentile": {
                    "input": format!("${}", field_path),
                    "p": [p / 100.0],
                    "method": "approximate"
                }
            }
        }
        AggFunction::Stddev => doc! { "$stdDevPop": format!("${}", field_path) },
        AggFunction::Variance => {
            // MongoDB doesn't have built-in variance, but we can compute it
            // Variance = stdDevPop^2
            doc! { "$pow": [{ "$stdDevPop": format!("${}", field_path) }, 2] }
        }
        AggFunction::CountDistinct => {
            // Use $addToSet then $size in a follow-up stage
            // For now, return the set
            doc! { "$addToSet": format!("${}", field_path) }
        }
        AggFunction::Collect => doc! { "$push": format!("${}", field_path) },
        AggFunction::CollectDistinct => doc! { "$addToSet": format!("${}", field_path) },
        AggFunction::First => doc! { "$first": format!("${}", field_path) },
        AggFunction::Last => doc! { "$last": format!("${}", field_path) },
    };

    Ok(Bson::Document(expr))
}
