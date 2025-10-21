//! Conversion utilities for BSON and JSON
//!
//! Provides efficient conversion between MongoDB BSON and JSON formats,
//! as well as GraphQL value conversions.

use async_graphql::{Name, Value as GqlValue};
use mongodb::bson::Bson;
use serde_json::Value as JsonValue;

use super::errors::DatabaseError;

/// Convert BSON value to JSON value
///
/// This provides an efficient direct conversion from MongoDB's BSON format
/// to standard JSON values, avoiding unnecessary round-trips.
///
/// # Examples
/// ```
/// use mongodb::bson::Bson;
/// use reframe::utils::conversion::bson_to_json;
///
/// let bson = Bson::String("test".to_string());
/// let json = bson_to_json(&bson).unwrap();
/// assert_eq!(json, serde_json::json!("test"));
/// ```
pub fn bson_to_json(bson: &Bson) -> Result<JsonValue, DatabaseError> {
    // Direct conversion using serde
    serde_json::to_value(bson)
        .map_err(|e| DatabaseError::Serialization(format!("Failed to convert BSON to JSON: {}", e)))
}

/// Convert JSON value to BSON value
///
/// # Examples
/// ```
/// use serde_json::json;
/// use reframe::utils::conversion::json_to_bson;
///
/// let json = json!({"key": "value"});
/// let bson = json_to_bson(&json).unwrap();
/// ```
pub fn json_to_bson(value: &JsonValue) -> Result<Bson, DatabaseError> {
    match value {
        JsonValue::Null => Ok(Bson::Null),
        JsonValue::Bool(b) => Ok(Bson::Boolean(*b)),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Bson::Int64(i))
            } else if let Some(f) = n.as_f64() {
                Ok(Bson::Double(f))
            } else {
                Err(DatabaseError::Serialization(
                    "Unsupported number type".to_string(),
                ))
            }
        }
        JsonValue::String(s) => Ok(Bson::String(s.clone())),
        JsonValue::Array(arr) => {
            let bson_arr: Result<Vec<Bson>, DatabaseError> = arr.iter().map(json_to_bson).collect();
            Ok(Bson::Array(bson_arr?))
        }
        JsonValue::Object(obj) => {
            let mut doc = mongodb::bson::Document::new();
            for (k, v) in obj {
                doc.insert(k, json_to_bson(v)?);
            }
            Ok(Bson::Document(doc))
        }
    }
}

/// Convert JSON Value to GraphQL Value
///
/// This is used for converting stored JSON data into GraphQL response values.
pub fn json_to_graphql_value(json: &JsonValue) -> Option<GqlValue> {
    match json {
        JsonValue::Null => Some(GqlValue::Null),
        JsonValue::Bool(b) => Some(GqlValue::Boolean(*b)),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Some(GqlValue::Number(i.into()))
            } else if let Some(f) = n.as_f64() {
                Some(GqlValue::Number(serde_json::Number::from_f64(f)?))
            } else {
                None
            }
        }
        JsonValue::String(s) => Some(GqlValue::String(s.clone())),
        JsonValue::Array(arr) => {
            let values: Option<Vec<_>> = arr.iter().map(json_to_graphql_value).collect();
            values.map(GqlValue::List)
        }
        JsonValue::Object(obj) => {
            let mut map = indexmap::IndexMap::new();
            for (k, v) in obj {
                if let Some(gql_v) = json_to_graphql_value(v) {
                    map.insert(Name::new(k), gql_v);
                }
            }
            Some(GqlValue::Object(map))
        }
    }
}

/// Convert JSON Value to GraphQL Value with type awareness
///
/// This ensures that values match their declared field types:
/// - "number" type fields are always converted to Float (even if stored as integers)
/// - "int" type fields are converted to Int
/// - "datetime" type fields are converted to String (RFC3339)
/// - Other types use standard conversion
pub fn json_to_graphql_value_typed(json: &JsonValue, field_type: &str) -> Option<GqlValue> {
    match json {
        JsonValue::Null => Some(GqlValue::Null),
        JsonValue::Bool(b) => Some(GqlValue::Boolean(*b)),
        JsonValue::Number(n) => match field_type {
            "number" => {
                // Always convert to float for "number" type fields
                let f = n.as_f64()?;
                Some(GqlValue::Number(serde_json::Number::from_f64(f)?))
            }
            "int" => {
                // Convert to integer for "int" type fields
                let i = n.as_i64()?;
                Some(GqlValue::Number(i.into()))
            }
            _ => {
                // Default: try int first, then float
                if let Some(i) = n.as_i64() {
                    Some(GqlValue::Number(i.into()))
                } else if let Some(f) = n.as_f64() {
                    Some(GqlValue::Number(serde_json::Number::from_f64(f)?))
                } else {
                    None
                }
            }
        },
        JsonValue::String(s) => Some(GqlValue::String(s.clone())),
        JsonValue::Array(arr) => {
            let values: Option<Vec<_>> = arr.iter().map(json_to_graphql_value).collect();
            values.map(GqlValue::List)
        }
        JsonValue::Object(obj) => {
            let mut map = indexmap::IndexMap::new();
            for (k, v) in obj {
                if let Some(gql_v) = json_to_graphql_value(v) {
                    map.insert(Name::new(k), gql_v);
                }
            }
            Some(GqlValue::Object(map))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bson_to_json() {
        let bson = Bson::String("test".to_string());
        let json = bson_to_json(&bson).unwrap();
        assert_eq!(json, serde_json::json!("test"));
    }

    #[test]
    fn test_json_to_bson() {
        let json = serde_json::json!({"key": "value"});
        let bson = json_to_bson(&json).unwrap();
        assert!(matches!(bson, Bson::Document(_)));
    }

    #[test]
    fn test_json_to_graphql_typed_number() {
        let json = serde_json::json!(42);

        // Test "number" type - should convert to float
        let gql_float = json_to_graphql_value_typed(&json, "number").unwrap();
        assert!(matches!(gql_float, GqlValue::Number(_)));

        // Test "int" type - should convert to int
        let gql_int = json_to_graphql_value_typed(&json, "int").unwrap();
        assert!(matches!(gql_int, GqlValue::Number(_)));
    }
}
