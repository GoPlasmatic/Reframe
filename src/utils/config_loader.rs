//! Configuration loading utilities
//!
//! Provides helpers for loading configuration from environment variables
//! and JSON config files, with fallback to defaults.

use serde_json::Value as JsonValue;

/// Helper struct for loading configuration values with environment variable
/// and JSON config fallbacks
pub struct ConfigLoader<'a> {
    config: Option<&'a JsonValue>,
}

impl<'a> ConfigLoader<'a> {
    /// Create a new ConfigLoader with optional JSON config
    pub fn new(config: Option<&'a JsonValue>) -> Self {
        Self { config }
    }

    /// Get a string configuration value
    ///
    /// Priority: ENV_VAR > JSON path > default
    ///
    /// # Arguments
    /// * `env_var` - Environment variable name
    /// * `json_path` - Path to config value in JSON (e.g., &["database", "connection", "uri"])
    /// * `default` - Default value if not found
    pub fn get_string(&self, env_var: &str, json_path: &[&str], default: &str) -> String {
        std::env::var(env_var).ok().unwrap_or_else(|| {
            self.get_json_string(json_path)
                .unwrap_or_else(|| default.to_string())
        })
    }

    /// Get a u64 configuration value
    ///
    /// Priority: ENV_VAR > JSON path > default
    pub fn get_u64(&self, env_var: &str, json_path: &[&str], default: u64) -> u64 {
        std::env::var(env_var)
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .or_else(|| self.get_json_u64(json_path))
            .unwrap_or(default)
    }

    /// Get a u32 configuration value
    ///
    /// Priority: ENV_VAR > JSON path > default
    pub fn get_u32(&self, env_var: &str, json_path: &[&str], default: u32) -> u32 {
        std::env::var(env_var)
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .or_else(|| self.get_json_u64(json_path).map(|v| v as u32))
            .unwrap_or(default)
    }

    /// Get a boolean configuration value
    ///
    /// Priority: ENV_VAR > JSON path > default
    pub fn get_bool(&self, env_var: &str, json_path: &[&str], default: bool) -> bool {
        std::env::var(env_var)
            .ok()
            .and_then(|s| s.parse::<bool>().ok())
            .or_else(|| self.get_json_bool(json_path))
            .unwrap_or(default)
    }

    /// Get string value from JSON config following a path
    fn get_json_string(&self, path: &[&str]) -> Option<String> {
        self.navigate_json_path(path)
            .and_then(|v| v.as_str())
            .map(String::from)
    }

    /// Get u64 value from JSON config following a path
    fn get_json_u64(&self, path: &[&str]) -> Option<u64> {
        self.navigate_json_path(path).and_then(|v| v.as_u64())
    }

    /// Get bool value from JSON config following a path
    fn get_json_bool(&self, path: &[&str]) -> Option<bool> {
        self.navigate_json_path(path).and_then(|v| v.as_bool())
    }

    /// Navigate to a nested path in the JSON config
    fn navigate_json_path(&self, path: &[&str]) -> Option<&JsonValue> {
        let mut current = self.config?;

        for &segment in path {
            current = current.get(segment)?;
        }

        Some(current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_get_string_from_json() {
        let config = json!({
            "database": {
                "connection": {
                    "uri": "mongodb://localhost"
                }
            }
        });

        let loader = ConfigLoader::new(Some(&config));
        let result = loader.get_string(
            "NON_EXISTENT",
            &["database", "connection", "uri"],
            "default",
        );
        assert_eq!(result, "mongodb://localhost");
    }

    #[test]
    fn test_get_string_default() {
        let loader = ConfigLoader::new(None);
        let result = loader.get_string("NON_EXISTENT", &["missing", "path"], "default_value");
        assert_eq!(result, "default_value");
    }

    #[test]
    fn test_get_u64() {
        let config = json!({
            "database": {
                "pool": {
                    "max_size": 100
                }
            }
        });

        let loader = ConfigLoader::new(Some(&config));
        let result = loader.get_u64("NON_EXISTENT", &["database", "pool", "max_size"], 10);
        assert_eq!(result, 100);
    }

    #[test]
    fn test_get_bool() {
        let config = json!({
            "options": {
                "persist_transform": true
            }
        });

        let loader = ConfigLoader::new(Some(&config));
        let result = loader.get_bool("NON_EXISTENT", &["options", "persist_transform"], false);
        assert_eq!(result, true);
    }
}
