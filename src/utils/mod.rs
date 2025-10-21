//! Utility modules for common functionality across the codebase

pub mod config_loader;
pub mod conversion;
pub mod errors;
pub mod string_case;

// Re-export commonly used items
pub use config_loader::ConfigLoader;
pub use conversion::{
    bson_to_json, json_to_bson, json_to_graphql_value, json_to_graphql_value_typed,
};
pub use errors::{DatabaseError, DatabaseResult};
pub use string_case::{to_camel_case, to_pascal_case};
