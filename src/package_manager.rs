use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::{debug, error, info, warn};

use crate::custom_fields::CustomFieldDefinition;

/// Package configuration loaded from reframe-package.json
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PackageConfig {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub license: String,
    #[serde(default)]
    pub engine_version: String,
    #[serde(default)]
    pub required_plugins: Vec<String>,
    pub workflows: HashMap<String, WorkflowConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scenarios: Option<ScenarioConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WorkflowConfig {
    pub path: String,
    pub description: String,
    #[serde(default)]
    pub entry_point: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScenarioConfig {
    pub path: String,
    pub description: String,
    #[serde(default)]
    pub entry_point: String,
}

/// Runtime package information
#[derive(Debug, Clone, Serialize)]
pub struct PackageInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: String,
    pub engine_version: String,
    pub required_plugins: Vec<String>,
    pub workflows: HashMap<String, WorkflowConfig>,
    pub scenarios: Option<ScenarioConfig>,
    pub enabled: bool,
    pub package_path: String,
    pub loaded_at: chrono::DateTime<chrono::Utc>,
    /// Custom field definitions from api_config.json
    pub custom_fields: Vec<CustomFieldDefinition>,
}

/// Reframe configuration file structure
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ReframeConfig {
    #[serde(default)]
    pub packages: Vec<PackageReference>,
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub api_docs: ApiDocsConfig,
    #[serde(default)]
    pub database: DatabaseConfigFile,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_format")]
    pub format: String,
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default)]
    pub show_target: bool,
    #[serde(default)]
    pub show_thread: bool,
    #[serde(default)]
    pub show_file_location: bool,
    #[serde(default)]
    pub show_span_events: bool,
    #[serde(default)]
    pub file_output: FileOutputConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FileOutputConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_log_directory")]
    pub directory: String,
    #[serde(default = "default_log_prefix")]
    pub file_prefix: String,
    #[serde(default = "default_log_rotation")]
    pub rotation: String,
}

impl Default for FileOutputConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            directory: default_log_directory(),
            file_prefix: default_log_prefix(),
            rotation: default_log_rotation(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            format: default_log_format(),
            level: default_log_level(),
            show_target: false,
            show_thread: false,
            show_file_location: false,
            show_span_events: false,
            file_output: FileOutputConfig::default(),
        }
    }
}

fn default_log_format() -> String {
    "compact".to_string()
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_directory() -> String {
    "./logs".to_string()
}

fn default_log_prefix() -> String {
    "reframe".to_string()
}

fn default_log_rotation() -> String {
    "daily".to_string()
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApiDocsConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_api_title")]
    pub title: String,
    #[serde(default = "default_api_version")]
    pub version: String,
    #[serde(default = "default_api_description")]
    pub description: String,
    #[serde(default)]
    pub contact: ContactInfo,
    #[serde(default)]
    pub license: LicenseInfo,
    #[serde(default = "default_external_docs_url")]
    pub external_docs_url: String,
    #[serde(default = "default_server_url")]
    pub server_url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ContactInfo {
    #[serde(default = "default_contact_name")]
    pub name: String,
    #[serde(default = "default_contact_email")]
    pub email: String,
}

impl Default for ContactInfo {
    fn default() -> Self {
        Self {
            name: default_contact_name(),
            email: default_contact_email(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LicenseInfo {
    #[serde(default = "default_license_name")]
    pub name: String,
    #[serde(default = "default_license_identifier")]
    pub identifier: String,
    #[serde(default = "default_license_url")]
    pub url: String,
}

impl Default for LicenseInfo {
    fn default() -> Self {
        Self {
            name: default_license_name(),
            identifier: default_license_identifier(),
            url: default_license_url(),
        }
    }
}

impl Default for ApiDocsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            title: default_api_title(),
            version: default_api_version(),
            description: default_api_description(),
            contact: ContactInfo::default(),
            license: LicenseInfo::default(),
            external_docs_url: default_external_docs_url(),
            server_url: default_server_url(),
        }
    }
}

fn default_api_title() -> String {
    "Reframe API".to_string()
}

fn default_api_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

fn default_api_description() -> String {
    "A Universal Message Transformation Engine".to_string()
}

fn default_contact_name() -> String {
    "Plasmatic Team".to_string()
}

fn default_contact_email() -> String {
    "enquires@goplasmatic.io".to_string()
}

fn default_license_name() -> String {
    "Apache 2.0".to_string()
}

fn default_license_identifier() -> String {
    "Apache-2.0".to_string()
}

fn default_license_url() -> String {
    "https://opensource.org/license/apache-2-0".to_string()
}

fn default_external_docs_url() -> String {
    "https://sandbox.goplasmatic.io".to_string()
}

fn default_server_url() -> String {
    "http://localhost:3000".to_string()
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PackageReference {
    pub path: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default)]
    pub runtime: RuntimeConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RuntimeConfig {
    /// Number of Tokio worker threads ("auto" for CPU count, or specific number)
    #[serde(default = "default_tokio_threads")]
    pub tokio_worker_threads: String,
    #[serde(default = "default_thread_name_prefix")]
    pub thread_name_prefix: String,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            tokio_worker_threads: default_tokio_threads(),
            thread_name_prefix: default_thread_name_prefix(),
        }
    }
}

fn default_tokio_threads() -> String {
    "auto".to_string()
}

fn default_thread_name_prefix() -> String {
    "reframe-worker".to_string()
}

fn default_true() -> bool {
    true
}

fn default_port() -> u16 {
    3000
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: default_port(),
            host: default_host(),
            runtime: RuntimeConfig::default(),
        }
    }
}

/// Database configuration from config file
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfigFile {
    #[serde(rename = "type", default = "default_db_type")]
    pub db_type: String,
    #[serde(default)]
    pub connection: DatabaseConnectionConfig,
    #[serde(default)]
    pub storage: DatabaseStorageConfig,
    #[serde(default)]
    pub options: DatabaseOptionsConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConnectionConfig {
    #[serde(default = "default_db_uri")]
    pub uri: String,
    #[serde(default = "default_db_timeout")]
    pub timeout_ms: u64,
    #[serde(default)]
    pub pool: DatabasePoolConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabasePoolConfig {
    #[serde(default = "default_pool_max_size")]
    pub max_size: u32,
    #[serde(default = "default_pool_min_size")]
    pub min_size: u32,
    #[serde(default = "default_pool_max_idle_time")]
    pub max_idle_time_ms: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseStorageConfig {
    #[serde(default = "default_db_database")]
    pub database: String,
    #[serde(default = "default_db_collection")]
    pub collection: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseOptionsConfig {
    #[serde(default = "default_publish_mode")]
    pub publish_mode: String,
}

impl Default for DatabaseConfigFile {
    fn default() -> Self {
        Self {
            db_type: default_db_type(),
            connection: DatabaseConnectionConfig::default(),
            storage: DatabaseStorageConfig::default(),
            options: DatabaseOptionsConfig::default(),
        }
    }
}

impl Default for DatabaseConnectionConfig {
    fn default() -> Self {
        Self {
            uri: default_db_uri(),
            timeout_ms: default_db_timeout(),
            pool: DatabasePoolConfig::default(),
        }
    }
}

impl Default for DatabasePoolConfig {
    fn default() -> Self {
        Self {
            max_size: default_pool_max_size(),
            min_size: default_pool_min_size(),
            max_idle_time_ms: default_pool_max_idle_time(),
        }
    }
}

impl Default for DatabaseStorageConfig {
    fn default() -> Self {
        Self {
            database: default_db_database(),
            collection: default_db_collection(),
        }
    }
}

impl Default for DatabaseOptionsConfig {
    fn default() -> Self {
        Self {
            publish_mode: default_publish_mode(),
        }
    }
}

fn default_db_type() -> String {
    "mongodb".to_string()
}

fn default_db_uri() -> String {
    "mongodb://localhost:27017".to_string()
}

fn default_db_timeout() -> u64 {
    5000
}

fn default_pool_max_size() -> u32 {
    10
}

fn default_pool_min_size() -> u32 {
    2
}

fn default_pool_max_idle_time() -> u64 {
    60000
}

fn default_db_database() -> String {
    "reframe".to_string()
}

fn default_db_collection() -> String {
    "reframe_audit".to_string()
}

fn default_publish_mode() -> String {
    "async".to_string()
}

/// Package Manager - handles discovery, loading, and validation of packages
pub struct PackageManager {
    config: ReframeConfig,
    packages: HashMap<String, PackageInfo>,
}

impl PackageManager {
    /// Create a new PackageManager from a config file
    pub fn new(config_path: Option<&str>) -> Result<Self, Box<dyn std::error::Error>> {
        let config = if let Some(path) = config_path {
            Self::load_config(path)?
        } else {
            // Try default location
            if Path::new("reframe.config.json").exists() {
                Self::load_config("reframe.config.json")?
            } else {
                info!("No reframe.config.json found, using default configuration");
                Self::default_config()
            }
        };

        Ok(Self {
            config,
            packages: HashMap::new(),
        })
    }

    /// Load configuration from file
    fn load_config(path: &str) -> Result<ReframeConfig, Box<dyn std::error::Error>> {
        debug!("Loading configuration from: {}", path);
        let content = fs::read_to_string(path)?;
        let config: ReframeConfig = serde_json::from_str(&content)?;
        info!("Configuration loaded successfully from {}", path);
        Ok(config)
    }

    /// Create default configuration (uses environment variable or default path)
    fn default_config() -> ReframeConfig {
        let package_path = std::env::var("REFRAME_PACKAGE_PATH")
            .unwrap_or_else(|_| "../reframe-package-swift-cbpr".to_string());

        ReframeConfig {
            packages: vec![PackageReference {
                path: package_path,
                enabled: true,
            }],
            server: ServerConfig::default(),
            logging: LoggingConfig::default(),
            api_docs: ApiDocsConfig::default(),
            database: DatabaseConfigFile::default(),
        }
    }

    /// Discover and load all packages
    pub fn discover_packages(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Discovering packages...");

        for package_ref in &self.config.packages {
            if !package_ref.enabled {
                debug!("Skipping disabled package at: {}", package_ref.path);
                continue;
            }

            match self.load_package(&package_ref.path) {
                Ok(package_info) => {
                    info!(
                        "✅ Loaded package: {} v{} ({})",
                        package_info.name, package_info.version, package_info.id
                    );
                    self.packages.insert(package_info.id.clone(), package_info);
                }
                Err(e) => {
                    error!("Failed to load package from {}: {}", package_ref.path, e);
                    // Continue loading other packages
                }
            }
        }

        info!(
            "Package discovery complete. Loaded {} package(s)",
            self.packages.len()
        );
        Ok(())
    }

    /// Load a single package from a directory
    fn load_package(&self, package_path: &str) -> Result<PackageInfo, Box<dyn std::error::Error>> {
        let config_file = format!("{}/reframe-package.json", package_path);

        if !Path::new(&config_file).exists() {
            return Err(format!("Package configuration not found: {}", config_file).into());
        }

        debug!("Loading package from: {}", config_file);
        let content = fs::read_to_string(&config_file)?;
        let package_config: PackageConfig = serde_json::from_str(&content)?;

        // Validate package
        self.validate_package(&package_config)?;

        // Load custom field definitions from api_config.json (if it exists)
        let custom_fields = self.load_custom_fields(package_path, &package_config.id)?;

        // Create package info
        let package_info = PackageInfo {
            id: package_config.id,
            name: package_config.name,
            version: package_config.version,
            description: package_config.description,
            author: package_config.author,
            license: package_config.license,
            engine_version: package_config.engine_version,
            required_plugins: package_config.required_plugins,
            workflows: package_config.workflows,
            scenarios: package_config.scenarios,
            enabled: true,
            package_path: package_path.to_string(),
            loaded_at: chrono::Utc::now(),
            custom_fields,
        };

        Ok(package_info)
    }

    /// Validate package compatibility
    fn validate_package(&self, package: &PackageConfig) -> Result<(), Box<dyn std::error::Error>> {
        // Validate required fields
        if package.id.is_empty() {
            return Err("Package ID cannot be empty".into());
        }

        if package.name.is_empty() {
            return Err("Package name cannot be empty".into());
        }

        if package.version.is_empty() {
            return Err("Package version cannot be empty".into());
        }

        // Validate workflow definitions
        let required_workflows = ["transform", "generate", "validate"];
        for workflow_type in &required_workflows {
            if !package.workflows.contains_key(*workflow_type) {
                warn!(
                    "Package '{}' is missing recommended workflow: {}",
                    package.id, workflow_type
                );
            }
        }

        // TODO: Validate engine version compatibility (semver)
        // TODO: Validate plugin dependencies

        debug!("Package '{}' validation passed", package.id);
        Ok(())
    }

    /// Load custom field definitions from api_config.json
    fn load_custom_fields(
        &self,
        package_path: &str,
        package_id: &str,
    ) -> Result<Vec<CustomFieldDefinition>, Box<dyn std::error::Error>> {
        use crate::custom_fields::validate_custom_field_definitions;

        let api_config_path = format!("{}/api_config.json", package_path);

        // Check if api_config.json exists
        if !Path::new(&api_config_path).exists() {
            debug!(
                package_id = %package_id,
                "No api_config.json found, package has no custom fields"
            );
            return Ok(Vec::new());
        }

        // Read and parse api_config.json
        debug!(
            package_id = %package_id,
            path = %api_config_path,
            "Loading custom field definitions"
        );

        let content = fs::read_to_string(&api_config_path)
            .map_err(|e| format!("Failed to read api_config.json: {}", e))?;

        let api_config: crate::custom_fields::ApiConfig = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse api_config.json: {}", e))?;

        // Validate custom field definitions
        validate_custom_field_definitions(&api_config.custom_fields)?;

        info!(
            package_id = %package_id,
            field_count = api_config.custom_fields.len(),
            "✅ Loaded custom field definitions"
        );

        Ok(api_config.custom_fields)
    }

    /// Get all loaded packages
    pub fn get_packages(&self) -> &HashMap<String, PackageInfo> {
        &self.packages
    }

    /// Get a specific package by ID
    pub fn get_package(&self, id: &str) -> Option<&PackageInfo> {
        self.packages.get(id)
    }

    /// Get server configuration
    pub fn get_server_config(&self) -> &ServerConfig {
        &self.config.server
    }

    /// Get logging configuration
    pub fn get_logging_config(&self) -> &LoggingConfig {
        &self.config.logging
    }

    /// Get API docs configuration
    pub fn get_api_docs_config(&self) -> &ApiDocsConfig {
        &self.config.api_docs
    }

    /// Get default package (first enabled package, or by env var)
    pub fn get_default_package(&self) -> Option<&PackageInfo> {
        // Try to get from environment variable
        if let Ok(default_id) = std::env::var("REFRAME_DEFAULT_PACKAGE")
            && let Some(package) = self.packages.get(&default_id)
        {
            return Some(package);
        }

        // Return first package
        self.packages.values().next()
    }

    /// Reload all packages
    pub fn reload_packages(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Reloading all packages...");
        self.packages.clear();
        self.discover_packages()?;
        info!("All packages reloaded successfully");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ReframeConfig::default();
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.server.host, "0.0.0.0");
    }

    #[test]
    fn test_package_reference() {
        let package_ref = PackageReference {
            path: "/test/path".to_string(),
            enabled: true,
        };
        assert_eq!(package_ref.path, "/test/path");
        assert!(package_ref.enabled);
    }
}
