//! Aggregation types for GraphQL analytics queries
//!
//! This module defines the types needed for powerful aggregation queries
//! including grouping, time bucketing, metrics, and results.

use async_graphql::dynamic::*;
use serde::{Deserialize, Serialize};

// ========== Input Types ==========

/// Group by configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupByInput {
    pub field: String,
    pub r#as: Option<String>,
    pub time_bucket: Option<TimeBucketInput>,
    pub numeric_bucket: Option<NumericBucketInput>,
    pub transform: Option<FieldTransform>,
}

/// Time bucketing configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeBucketInput {
    pub interval: String,
    pub timezone: Option<String>,
    pub fill_gaps: Option<bool>,
}

/// Numeric bucketing configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NumericBucketInput {
    pub strategy: BucketStrategy,
    pub boundaries: Option<Vec<f64>>,
    pub count: Option<i32>,
    pub labels: Option<Vec<String>>,
}

/// Bucket strategy enum
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BucketStrategy {
    Fixed,
    Quantile,
}

/// Field transformation options
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldTransform {
    pub date_part: Option<DatePart>,
    pub to_lower: Option<bool>,
    pub to_upper: Option<bool>,
    pub round: Option<i32>,
}

/// Date part extraction enum
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DatePart {
    Year,
    Month,
    Day,
    Hour,
    Minute,
    DayOfWeek,
    DayOfYear,
    WeekOfYear,
    Quarter,
}

/// Metric calculation configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetricInput {
    pub function: AggFunction,
    pub field: Option<String>,
    pub r#as: String,
    // Filter removed - filtering is now done at the query level only
    pub percentile: Option<f64>,
}

/// Aggregation function enum
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AggFunction {
    Count,
    Sum,
    Avg,
    Min,
    Max,
    Median,
    Percentile,
    Stddev,
    Variance,
    CountDistinct,
    Collect,
    CollectDistinct,
    First,
    Last,
}

/// Sort configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SortInput {
    pub field: String,
    pub direction: SortDirection,
}

/// Sort direction enum
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SortDirection {
    Asc,
    Desc,
}

// ========== Output Types ==========

/// Aggregation query result
#[derive(Debug, Clone, Serialize)]
pub struct AggregationResult {
    pub data: Vec<DataPoint>,
    pub total_groups: i64,
    pub execution_time_ms: f64,
    pub metadata: AggregationMetadata,
}

/// Single data point in aggregation result
#[derive(Debug, Clone, Serialize)]
pub struct DataPoint {
    pub group: serde_json::Value,
    pub metrics: serde_json::Value,
}

/// Metadata about the aggregation query
#[derive(Debug, Clone, Serialize)]
pub struct AggregationMetadata {
    pub group_by_fields: Vec<String>,
    pub metric_fields: Vec<String>,
    pub total_messages: i64,
}

// ========== GraphQL Type Builders ==========

/// Build GroupByInput for GraphQL
pub fn build_group_by_input() -> InputObject {
    InputObject::new("GroupByInput")
        .description("Group by field configuration")
        .field(InputValue::new("field", TypeRef::named_nn(TypeRef::STRING)))
        .field(InputValue::new("as", TypeRef::named(TypeRef::STRING)))
        .field(InputValue::new(
            "timeBucket",
            TypeRef::named("TimeBucketInput"),
        ))
        .field(InputValue::new(
            "numericBucket",
            TypeRef::named("NumericBucketInput"),
        ))
        .field(InputValue::new(
            "transform",
            TypeRef::named("FieldTransform"),
        ))
}

/// Build TimeBucketInput for GraphQL
pub fn build_time_bucket_input() -> InputObject {
    InputObject::new("TimeBucketInput")
        .description("Time bucketing configuration (1s, 5m, 1h, 1d, 1w, 1M, 1y)")
        .field(InputValue::new(
            "interval",
            TypeRef::named_nn(TypeRef::STRING),
        ))
        .field(InputValue::new("timezone", TypeRef::named(TypeRef::STRING)))
        .field(InputValue::new(
            "fillGaps",
            TypeRef::named(TypeRef::BOOLEAN),
        ))
}

/// Build NumericBucketInput for GraphQL
pub fn build_numeric_bucket_input() -> InputObject {
    InputObject::new("NumericBucketInput")
        .description("Numeric bucketing configuration")
        .field(InputValue::new(
            "strategy",
            TypeRef::named_nn("BucketStrategy"),
        ))
        .field(InputValue::new(
            "boundaries",
            TypeRef::named_list(TypeRef::FLOAT),
        ))
        .field(InputValue::new("count", TypeRef::named(TypeRef::INT)))
        .field(InputValue::new(
            "labels",
            TypeRef::named_list(TypeRef::STRING),
        ))
}

/// Build BucketStrategy enum for GraphQL
pub fn build_bucket_strategy_enum() -> Enum {
    Enum::new("BucketStrategy")
        .item(EnumItem::new("FIXED"))
        .item(EnumItem::new("QUANTILE"))
}

/// Build FieldTransform for GraphQL
pub fn build_field_transform_input() -> InputObject {
    InputObject::new("FieldTransform")
        .description("Field transformation options")
        .field(InputValue::new("datePart", TypeRef::named("DatePart")))
        .field(InputValue::new("toLower", TypeRef::named(TypeRef::BOOLEAN)))
        .field(InputValue::new("toUpper", TypeRef::named(TypeRef::BOOLEAN)))
        .field(InputValue::new("round", TypeRef::named(TypeRef::INT)))
}

/// Build DatePart enum for GraphQL
pub fn build_date_part_enum() -> Enum {
    Enum::new("DatePart")
        .item(EnumItem::new("YEAR"))
        .item(EnumItem::new("MONTH"))
        .item(EnumItem::new("DAY"))
        .item(EnumItem::new("HOUR"))
        .item(EnumItem::new("MINUTE"))
        .item(EnumItem::new("DAY_OF_WEEK"))
        .item(EnumItem::new("DAY_OF_YEAR"))
        .item(EnumItem::new("WEEK_OF_YEAR"))
        .item(EnumItem::new("QUARTER"))
}

/// Build MetricInput for GraphQL
pub fn build_metric_input() -> InputObject {
    InputObject::new("MetricInput")
        .description("Metric calculation configuration")
        .field(InputValue::new(
            "function",
            TypeRef::named_nn("AggFunction"),
        ))
        .field(InputValue::new("field", TypeRef::named(TypeRef::STRING)))
        .field(InputValue::new("as", TypeRef::named_nn(TypeRef::STRING)))
        .field(InputValue::new(
            "percentile",
            TypeRef::named(TypeRef::FLOAT),
        ))
}

/// Build AggFunction enum for GraphQL
pub fn build_agg_function_enum() -> Enum {
    Enum::new("AggFunction")
        .item(EnumItem::new("COUNT"))
        .item(EnumItem::new("SUM"))
        .item(EnumItem::new("AVG"))
        .item(EnumItem::new("MIN"))
        .item(EnumItem::new("MAX"))
        .item(EnumItem::new("MEDIAN"))
        .item(EnumItem::new("PERCENTILE"))
        .item(EnumItem::new("STDDEV"))
        .item(EnumItem::new("VARIANCE"))
        .item(EnumItem::new("COUNT_DISTINCT"))
        .item(EnumItem::new("COLLECT"))
        .item(EnumItem::new("COLLECT_DISTINCT"))
        .item(EnumItem::new("FIRST"))
        .item(EnumItem::new("LAST"))
}

/// Build SortInput for GraphQL
pub fn build_sort_input() -> InputObject {
    InputObject::new("SortInput")
        .description("Sort configuration")
        .field(InputValue::new("field", TypeRef::named_nn(TypeRef::STRING)))
        .field(InputValue::new(
            "direction",
            TypeRef::named_nn("SortDirection"),
        ))
}

/// Build SortDirection enum for GraphQL
pub fn build_sort_direction_enum() -> Enum {
    Enum::new("SortDirection")
        .item(EnumItem::new("ASC"))
        .item(EnumItem::new("DESC"))
}

/// Build AggregationResult type for GraphQL
pub fn build_aggregation_result_type() -> Object {
    Object::new("AggregationResult")
        .description("Result of an aggregation query")
        .field(Field::new(
            "data",
            TypeRef::named_nn_list_nn("DataPoint"),
            |ctx| {
                FieldFuture::new(async move {
                    let result = ctx.parent_value.try_downcast_ref::<AggregationResult>()?;
                    let data: Vec<FieldValue> = result
                        .data
                        .iter()
                        .map(|dp| FieldValue::owned_any(dp.clone()))
                        .collect();
                    Ok(Some(FieldValue::list(data)))
                })
            },
        ))
        .field(Field::new(
            "totalGroups",
            TypeRef::named_nn(TypeRef::INT),
            |ctx| {
                FieldFuture::new(async move {
                    let result = ctx.parent_value.try_downcast_ref::<AggregationResult>()?;
                    Ok(Some(async_graphql::Value::from(result.total_groups)))
                })
            },
        ))
        .field(Field::new(
            "executionTimeMs",
            TypeRef::named_nn(TypeRef::FLOAT),
            |ctx| {
                FieldFuture::new(async move {
                    let result = ctx.parent_value.try_downcast_ref::<AggregationResult>()?;
                    Ok(serde_json::Number::from_f64(result.execution_time_ms)
                        .map(async_graphql::Value::Number))
                })
            },
        ))
        .field(Field::new(
            "metadata",
            TypeRef::named_nn("AggregationMetadata"),
            |ctx| {
                FieldFuture::new(async move {
                    let result = ctx.parent_value.try_downcast_ref::<AggregationResult>()?;
                    Ok(Some(FieldValue::owned_any(result.metadata.clone())))
                })
            },
        ))
}

/// Build DataPoint type for GraphQL
pub fn build_data_point_type() -> Object {
    use crate::utils::json_to_graphql_value;

    Object::new("DataPoint")
        .description("Single data point with group and metrics")
        .field(Field::new("group", TypeRef::named_nn("JSON"), |ctx| {
            FieldFuture::new(async move {
                let dp = ctx.parent_value.try_downcast_ref::<DataPoint>()?;
                Ok(json_to_graphql_value(&dp.group))
            })
        }))
        .field(Field::new("metrics", TypeRef::named_nn("JSON"), |ctx| {
            FieldFuture::new(async move {
                let dp = ctx.parent_value.try_downcast_ref::<DataPoint>()?;
                Ok(json_to_graphql_value(&dp.metrics))
            })
        }))
}

/// Build AggregationMetadata type for GraphQL
pub fn build_aggregation_metadata_type() -> Object {
    Object::new("AggregationMetadata")
        .description("Metadata about the aggregation query")
        .field(Field::new(
            "groupByFields",
            TypeRef::named_nn_list_nn(TypeRef::STRING),
            |ctx| {
                FieldFuture::new(async move {
                    use async_graphql::Value as GqlValue;
                    let metadata = ctx.parent_value.try_downcast_ref::<AggregationMetadata>()?;
                    let fields: Vec<FieldValue> = metadata
                        .group_by_fields
                        .iter()
                        .map(|f| FieldValue::from(GqlValue::String(f.clone())))
                        .collect();
                    Ok(Some(FieldValue::list(fields)))
                })
            },
        ))
        .field(Field::new(
            "metricFields",
            TypeRef::named_nn_list_nn(TypeRef::STRING),
            |ctx| {
                FieldFuture::new(async move {
                    use async_graphql::Value as GqlValue;
                    let metadata = ctx.parent_value.try_downcast_ref::<AggregationMetadata>()?;
                    let fields: Vec<FieldValue> = metadata
                        .metric_fields
                        .iter()
                        .map(|f| FieldValue::from(GqlValue::String(f.clone())))
                        .collect();
                    Ok(Some(FieldValue::list(fields)))
                })
            },
        ))
        .field(Field::new(
            "totalMessages",
            TypeRef::named_nn(TypeRef::INT),
            |ctx| {
                FieldFuture::new(async move {
                    let metadata = ctx.parent_value.try_downcast_ref::<AggregationMetadata>()?;
                    Ok(Some(async_graphql::Value::from(metadata.total_messages)))
                })
            },
        ))
}
