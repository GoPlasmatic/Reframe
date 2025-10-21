use arc_swap::ArcSwap;
use dataflow_rs::{AsyncFunctionHandler, Engine, Workflow};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, info, warn};

use crate::types::AppState;

// Import plugin registration functions
use mx_message::plugin::register_mx_functions;
use swift_mt_message::plugin::register_swift_mt_functions;

// Import detection module
use crate::detection;

// Re-use package config types from package_manager
use crate::package_manager::PackageConfig;

/// Load package configuration from reframe-package.json
fn load_package_config(package_path: &str) -> Result<PackageConfig, Box<dyn std::error::Error>> {
    let config_path = format!("{}/reframe-package.json", package_path);
    debug!("Loading package configuration from: {}", config_path);

    let config_content = fs::read_to_string(&config_path)?;
    let config: PackageConfig = serde_json::from_str(&config_content)?;

    info!("Loaded package: {} v{}", config.name, config.version);
    Ok(config)
}

/// Helper function to register all plugin functions
///
/// This is cached and reused across all engines to avoid redundant registration
fn register_all_functions() -> HashMap<String, Box<dyn AsyncFunctionHandler + Send + Sync>> {
    let mut custom_functions: HashMap<String, Box<dyn AsyncFunctionHandler + Send + Sync>> =
        HashMap::new();

    // Register generic detection function
    debug!("Registering generic detection function");
    custom_functions.insert("detect".to_string(), Box::new(detection::Detect));

    // Register all MT plugin functions
    let mt_functions = register_swift_mt_functions();
    for (name, handler) in mt_functions {
        debug!("Registering MT function: {}", name);
        custom_functions.insert(name.to_string(), handler);
    }

    // Register all MX plugin functions
    let mx_functions = register_mx_functions();
    for (name, handler) in mx_functions {
        debug!("Registering MX function: {}", name);
        custom_functions.insert(name.to_string(), handler);
    }

    custom_functions
}

/// Generic engine initialization function for any workflow type
///
/// This consolidates the common logic for initializing transform, generation, and validation engines
///
/// # Arguments
/// * `config` - Package configuration
/// * `package_path` - Path to the package directory
/// * `workflow_type` - Type of workflow ("transform", "generate", "validate")
///
/// # Returns
/// The initialized engine or an error
async fn initialize_engine_for_workflow(
    config: &PackageConfig,
    package_path: &str,
    workflow_type: &str,
) -> Result<Arc<Engine>, Box<dyn std::error::Error>> {
    debug!(
        "Initializing {} engine for package {}",
        workflow_type, config.name
    );

    // Get workflow configuration
    let workflow_config = config.workflows.get(workflow_type).ok_or_else(|| {
        format!(
            "{} workflow configuration not found in package",
            workflow_type
        )
    })?;

    let workflow_path = format!("{}/{}", package_path, workflow_config.path);

    // Load workflows
    let workflows = load_workflows(&workflow_path).await?;
    let workflow_count = workflows.len();

    // Register custom functions for this engine
    // Note: Function handlers contain trait objects that can't be cloned,
    // so we need to register separately for each engine
    let custom_functions = register_all_functions();
    let function_count = custom_functions.len();

    // Create engine with workflows and custom functions
    let engine = Engine::new(workflows, Some(custom_functions));

    info!(
        "✓ {} engine: {} workflows, {} functions",
        workflow_type.to_string().to_uppercase(),
        workflow_count,
        function_count
    );

    Ok(Arc::new(engine))
}

async fn load_workflows(workflow_dir: &str) -> Result<Vec<Workflow>, Box<dyn std::error::Error>> {
    debug!(directory = %workflow_dir, "Loading workflow configurations");

    let mut workflows = Vec::new();
    let index_path = format!("{workflow_dir}/index.json");

    if !Path::new(&index_path).exists() {
        warn!(
            "No index.json found in {}, skipping workflow loading",
            workflow_dir
        );
        return Ok(workflows);
    }

    let index_content = fs::read_to_string(&index_path)?;
    let index: Value = serde_json::from_str(&index_content)?;

    if let Some(workflow_entries) = index.get("workflows").and_then(|w| w.as_array()) {
        for workflow_entry in workflow_entries {
            if let Some(path) = workflow_entry.get("path").and_then(|p| p.as_str()) {
                let full_path = format!("{workflow_dir}/{path}");
                if Path::new(&full_path).exists() {
                    let workflow_content = fs::read_to_string(&full_path)?;
                    let workflow: Workflow = serde_json::from_str(&workflow_content)?;
                    workflows.push(workflow);
                    debug!(workflow = %path, "Loaded workflow file");
                } else {
                    warn!(file = %full_path, "Workflow file not found");
                }
            }
        }
    }

    Ok(workflows)
}

/// Initialize async engines for high-performance multi-threaded processing
///
/// Loads configuration, discovers packages, and initializes all three engines:
/// - Transform: Bidirectional MT ↔ MX transformations
/// - Generation: Sample message creation
/// - Validation: Message compliance checking
pub async fn initialize_engines() -> AppState {
    info!("⚙️  Initializing engines...");

    // Initialize PackageManager
    let mut package_manager = crate::package_manager::PackageManager::new(None)
        .expect("Failed to initialize package manager");

    // Discover and load packages
    package_manager
        .discover_packages()
        .expect("Failed to discover packages");

    // Get default package for engine initialization
    let default_package = package_manager
        .get_default_package()
        .expect("No packages loaded");

    let package_path = &default_package.package_path;
    debug!(
        "Loading engines from package: {} v{} ({})",
        default_package.name, default_package.version, package_path
    );

    // Load package config once (reuse across all engines)
    let config = load_package_config(package_path).expect("Failed to load package config");

    // Create all engines using shared config
    // Note: Each engine gets its own function registry as trait objects can't be cloned
    let transform_engine = initialize_engine_for_workflow(&config, package_path, "transform")
        .await
        .expect("Failed to initialize transform engine");

    let generation_engine = initialize_engine_for_workflow(&config, package_path, "generate")
        .await
        .expect("Failed to initialize generation engine");

    let validation_engine = initialize_engine_for_workflow(&config, package_path, "validate")
        .await
        .expect("Failed to initialize validation engine");

    info!("✅ All engines initialized successfully");

    // Load database configuration
    let db_config = crate::database::DatabaseConfig::load();

    // Create Arc<RwLock<PackageManager>> once for sharing
    let package_manager_arc = Arc::new(std::sync::RwLock::new(package_manager));

    // Initialize database client only if persistence is enabled for at least one operation
    let (database_client, mongodb_client) = if db_config.persist_transform
        || db_config.persist_validate
        || db_config.persist_generate
    {
        info!(
            "🔌 Initializing database client (persist_transform: {}, persist_validate: {}, persist_generate: {})",
            db_config.persist_transform, db_config.persist_validate, db_config.persist_generate
        );

        let client =
            crate::database::create_database_client(&db_config, Arc::clone(&package_manager_arc))
                .await
                .expect("Failed to initialize database client");

        info!(
            "✅ Database audit logging initialized (type: {:?}, database: {}.{})",
            db_config.db_type, db_config.database_name, db_config.collection_name
        );

        // If MongoDB, create a second reference for GraphQL
        let mongo_client = if matches!(db_config.db_type, crate::database::DatabaseType::MongoDB) {
            Some(Arc::new(
                crate::database::mongodb::MongoDBClient::new(
                    &db_config,
                    Arc::clone(&package_manager_arc),
                )
                .await
                .expect("Failed to create MongoDB client for GraphQL"),
            ))
        } else {
            None
        };

        (Some(client), mongo_client)
    } else {
        info!(
            "⚠️  Database persistence disabled (persist_transform: false, persist_validate: false, persist_generate: false)"
        );
        (None, None)
    };

    AppState {
        transform_engine: Arc::new(ArcSwap::from(transform_engine)),
        generation_engine: Arc::new(ArcSwap::from(generation_engine)),
        validation_engine: Arc::new(ArcSwap::from(validation_engine)),
        package_manager: package_manager_arc,
        database_client,
        mongodb_client,
        database_config: Arc::new(db_config),
    }
}

/// Reload all engines with updated workflow configurations from packages
///
/// This allows hot-reloading of workflow changes without restarting the service
pub async fn reload_engines(app_state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    info!("🔄 Reloading workflow configurations and packages...");

    // Reload packages
    {
        let mut pm = app_state.package_manager.write().unwrap();
        pm.reload_packages()?;
    }

    // Get default package for engine reloading
    let package_path = {
        let pm = app_state.package_manager.read().unwrap();
        let default_package = pm
            .get_default_package()
            .ok_or("No packages loaded after reload")?;
        default_package.package_path.clone()
    };

    // Load package config once (reuse across all engines)
    let config = load_package_config(&package_path)?;

    // Create new engines using shared config
    let new_transform_engine =
        initialize_engine_for_workflow(&config, &package_path, "transform").await?;

    let new_generation_engine =
        initialize_engine_for_workflow(&config, &package_path, "generate").await?;

    let new_validation_engine =
        initialize_engine_for_workflow(&config, &package_path, "validate").await?;

    // Atomically swap the engines
    app_state.transform_engine.store(new_transform_engine);
    app_state.generation_engine.store(new_generation_engine);
    app_state.validation_engine.store(new_validation_engine);

    info!("✅ Workflow reload completed - new requests will use updated workflows");

    Ok(())
}
