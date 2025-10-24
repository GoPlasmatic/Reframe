//! MongoDB Aggregation Pipeline Builder
//!
//! Converts GraphQL aggregation queries to MongoDB aggregation pipelines
//! with support for grouping, time bucketing, metrics, and transformations

use crate::graphql::aggregation_types::{
    AggFunction, DatePart, FieldTransform, GroupByInput, MetricInput, SortDirection, SortInput,
    TimeBucketInput,
};
use mongodb::bson::{Bson, Document, doc};

/// Normalize field path for MongoDB queries
///
/// Converts GraphQL field paths to MongoDB document paths:
/// - "$.swiftCbprMtMxFields.xxx" → "context.swiftCbprMtMxFields.xxx" (GraphQL accessor)
/// - "$.context.swiftCbprMtMxFields.xxx" → "context.swiftCbprMtMxFields.xxx" (Full path)
/// - "context.swiftCbprMtMxFields.xxx" → "context.swiftCbprMtMxFields.xxx" (Already normalized)
/// - "metadata.package_id" → "context.metadata.package_id" (Add context prefix)
/// - "$.context.data.amount" → "context.data.amount" (Strip $ prefix)
fn normalize_field_path(field: &str) -> String {
    // Handle JSONPath expressions starting with $
    let field_without_jsonpath = if let Some(stripped) = field.strip_prefix("$.") {
        stripped // Strip "$."
    } else if let Some(stripped) = field.strip_prefix('$') {
        stripped // Strip "$"
    } else {
        field
    };

    // Check if this is a GraphQL accessor name (e.g., "swiftCbprMtMxFields.direction")
    // These are dynamically generated accessor names ending with "Fields"
    if let Some(dot_pos) = field_without_jsonpath.find('.') {
        let first_component = &field_without_jsonpath[..dot_pos];
        let rest = &field_without_jsonpath[dot_pos + 1..];

        // If first component ends with "Fields", it's a GraphQL accessor
        // Keep it: "swiftCbprMtMxFields.direction" → "context.swiftCbprMtMxFields.direction"
        if first_component.ends_with("Fields") && first_component != "customFields" {
            return format!("context.{}.{}", first_component, rest);
        }
    }

    // Now apply normal normalization
    if field_without_jsonpath.starts_with("context.") {
        // Already has context prefix, return as-is
        field_without_jsonpath.to_string()
    } else {
        // Add context prefix
        format!("context.{}", field_without_jsonpath)
    }
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

    // Stage 2: Add computed fields for grouping
    let mut add_fields = Document::new();

    for group_by_input in group_by {
        let field_path = normalize_field_path(&group_by_input.field);
        let alias = group_by_input
            .r#as
            .as_ref()
            .unwrap_or(&group_by_input.field);

        if let Some(ref time_bucket) = group_by_input.time_bucket {
            add_fields.insert(
                alias,
                build_time_bucket_expression(&field_path, time_bucket)?,
            );
        } else if let Some(ref transform) = group_by_input.transform {
            add_fields.insert(alias, build_transform_expression(&field_path, transform)?);
        } else {
            // Just add a reference to the field
            add_fields.insert(alias, format!("${}", field_path));
        }
    }

    if !add_fields.is_empty() {
        pipeline.push(doc! { "$addFields": add_fields });
    }

    // Stage 3: Group
    let mut group_doc = Document::new();
    let mut group_id = Document::new();

    for group_by_input in group_by {
        let alias = group_by_input
            .r#as
            .as_ref()
            .unwrap_or(&group_by_input.field);
        group_id.insert(alias, format!("${}", alias));
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

    // Stage 4: Project to final shape
    let mut project_doc = Document::new();
    project_doc.insert("_id", 0);

    if !group_by.is_empty() {
        project_doc.insert("group", "$_id");
    } else {
        project_doc.insert("group", doc! {});
    }

    let mut metrics_obj = Document::new();
    for metric in metrics {
        metrics_obj.insert(&metric.r#as, format!("${}", metric.r#as));
    }
    project_doc.insert("metrics", metrics_obj);

    pipeline.push(doc! { "$project": project_doc });

    // Stage 5: Sort
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
                sort_doc.insert(format!("group.{}", sort_input.field), direction);
            } else {
                sort_doc.insert(format!("metrics.{}", sort_input.field), direction);
            }
        }
        pipeline.push(doc! { "$sort": sort_doc });
    }

    // Stage 6: Limit
    if let Some(limit_val) = limit {
        pipeline.push(doc! { "$limit": limit_val });
    }

    Ok(pipeline)
}

/// Build time bucket expression for MongoDB
///
/// Uses MongoDB's $dateTrunc operator to bucket timestamps
fn build_time_bucket_expression(
    field: &str,
    time_bucket: &TimeBucketInput,
) -> Result<Bson, String> {
    let unit = parse_interval(&time_bucket.interval)?;
    let timezone = time_bucket.timezone.as_deref().unwrap_or("UTC");

    Ok(Bson::Document(doc! {
        "$dateTrunc": {
            "date": format!("${}", field),
            "unit": unit,
            "timezone": timezone
        }
    }))
}

/// Parse interval string to MongoDB date unit
fn parse_interval(interval: &str) -> Result<String, String> {
    // Extract number and unit
    if interval.is_empty() {
        return Err("Empty interval".to_string());
    }

    let unit_char = interval.chars().last().unwrap();

    // For MongoDB $dateTrunc, we support: second, minute, hour, day, week, month, year
    match unit_char {
        's' => Ok("second".to_string()),
        'm' => Ok("minute".to_string()),
        'h' => Ok("hour".to_string()),
        'd' => Ok("day".to_string()),
        'w' => Ok("week".to_string()),
        'M' => Ok("month".to_string()),
        'y' => Ok("year".to_string()),
        _ => Err(format!("Invalid interval unit: {}", unit_char)),
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_field_path_jsonpath() {
        // Test GraphQL accessor names (ending with "Fields")
        assert_eq!(
            normalize_field_path("$.swiftCbprMtMxFields.direction"),
            "context.swiftCbprMtMxFields.direction"
        );

        assert_eq!(
            normalize_field_path("$swiftCbprMtMxFields.direction"),
            "context.swiftCbprMtMxFields.direction"
        );

        assert_eq!(
            normalize_field_path("swiftCbprMtMxFields.amount"),
            "context.swiftCbprMtMxFields.amount"
        );

        // Test other accessor names
        assert_eq!(
            normalize_field_path("otherPackageFields.value"),
            "context.otherPackageFields.value"
        );

        // Test full path with context prefix
        assert_eq!(
            normalize_field_path("$.context.swiftCbprMtMxFields.direction"),
            "context.swiftCbprMtMxFields.direction"
        );

        // Test context path (unchanged)
        assert_eq!(
            normalize_field_path("context.data.amount"),
            "context.data.amount"
        );

        assert_eq!(
            normalize_field_path("$.context.data.amount"),
            "context.data.amount"
        );

        // Test plain field name - adds context prefix
        assert_eq!(
            normalize_field_path("metadata.package_id"),
            "context.metadata.package_id"
        );

        // Test with $ prefix on plain field
        assert_eq!(
            normalize_field_path("$.metadata.package_id"),
            "context.metadata.package_id"
        );
    }
}
