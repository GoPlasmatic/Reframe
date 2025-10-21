use axum::{
    Extension, Router, middleware,
    response::{Html, IntoResponse},
    routing::{get, post},
};
use tracing::info;

// GraphQL imports
use async_graphql::http::{GraphQLPlaygroundConfig, playground_source};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};

// Module declarations
mod ascii_art;
mod config_helpers;
mod custom_fields;
mod database;
mod detection;
mod engine;
mod graphql;
mod handlers;
mod logging;
mod openapi;
mod package_manager;
mod types;
mod utils;

// Import public items from modules
use ascii_art::display_ascii_art;
use engine::initialize_engines;
use handlers::{
    correlation_middleware, generate_sample, health_check, list_packages, reload_workflows,
    transform, validate,
};
use logging::{LogConfig, init_logging, log_system_info};
use openapi::swagger_ui_with_config;

// GraphQL handler functions
async fn graphql_handler(
    schema: Extension<graphql::ReframeSchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

async fn graphql_playground() -> impl IntoResponse {
    Html(playground_source(GraphQLPlaygroundConfig::new("/graphql")))
}

fn main() {
    // Load configuration to determine runtime settings
    // Configuration is loaded from reframe.config.json (or built-in defaults if not found)
    let temp_config =
        crate::package_manager::PackageManager::new(None).expect("Failed to load configuration");
    let runtime_config = &temp_config.get_server_config().runtime;

    // Configure Tokio worker threads using helper (handles env override and config)
    let tokio_threads = config_helpers::calculate_tokio_threads(runtime_config);
    let thread_name_prefix = config_helpers::get_thread_name_prefix(runtime_config);

    // Build Tokio runtime with configured worker threads
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(tokio_threads)
        .enable_all()
        .thread_name(thread_name_prefix)
        .build()
        .expect("Failed to create Tokio runtime");

    // Run the async main function
    runtime.block_on(async_main());
}

async fn async_main() {
    // Display ASCII art at startup
    display_ascii_art();

    // Load configuration early to use it for logging setup
    let config_manager =
        crate::package_manager::PackageManager::new(None).expect("Failed to load configuration");

    let logging_config = config_manager.get_logging_config();

    // Initialize logging system from configuration
    // Environment variables (RUST_LOG, LOG_FORMAT) override config file settings
    let log_format = config_helpers::parse_log_format(&logging_config.format);

    let log_config = LogConfig {
        format: log_format,
        level: std::env::var("RUST_LOG").unwrap_or_else(|_| logging_config.level.clone()),
        show_target: logging_config.show_target,
        show_thread: logging_config.show_thread,
        show_file_location: logging_config.show_file_location,
        show_span_events: logging_config.show_span_events,
        file_output: if logging_config.file_output.enabled {
            Some(logging::FileOutputConfig {
                directory: logging_config.file_output.directory.clone(),
                file_prefix: logging_config.file_output.file_prefix.clone(),
                rotation: match logging_config.file_output.rotation.as_str() {
                    "hourly" => tracing_appender::rolling::Rotation::HOURLY,
                    "daily" => tracing_appender::rolling::Rotation::DAILY,
                    "never" => tracing_appender::rolling::Rotation::NEVER,
                    _ => tracing_appender::rolling::Rotation::DAILY,
                },
            })
        } else {
            None
        },
    };

    if let Err(e) = init_logging(log_config) {
        // Use eprintln here since logging isn't initialized yet
        eprintln!("Failed to initialize logging: {}", e);
    }

    // Log system information
    log_system_info(env!("CARGO_PKG_VERSION"));

    info!("Service initialization started");

    // Initialize all engines from configured packages
    let app_state = initialize_engines().await;

    // Get server and API documentation configuration from loaded config
    let (server_config, api_docs_config) = {
        let pm = app_state.package_manager.read().unwrap();
        (
            pm.get_server_config().clone(),
            pm.get_api_docs_config().clone(),
        )
    };

    // Build base router with core endpoints
    let mut app = Router::new()
        .route("/health", get(health_check))
        .route("/api/transform", post(transform))
        .route("/api/generate", post(generate_sample))
        .route("/api/validate", post(validate))
        .route("/api/packages", get(list_packages))
        .route("/admin/reload-workflows", post(reload_workflows));

    // Add GraphQL routes if MongoDB client is available
    let graphql_enabled = if let Some(ref mongo_client) = app_state.mongodb_client {
        info!("🔍 Initializing GraphQL API");

        // Create GraphQL schema with MongoDB client and package manager
        let schema =
            graphql::create_schema(mongo_client.clone(), app_state.package_manager.clone());

        // Add GraphQL routes and schema extension
        app = app
            .route("/graphql", post(graphql_handler))
            .route("/graphql/playground", get(graphql_playground))
            .layer(Extension(schema));

        true
    } else {
        false
    };

    // Complete the router
    let app = app
        .merge(swagger_ui_with_config(&api_docs_config))
        .layer(middleware::from_fn(correlation_middleware))
        .with_state(app_state);

    // Bind server to configured host and port
    // Environment variables (REFRAME_HOST, REFRAME_PORT) override config file
    let host = std::env::var("REFRAME_HOST").unwrap_or_else(|_| server_config.host.clone());
    let port = std::env::var("REFRAME_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(server_config.port);

    let bind_address = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&bind_address)
        .await
        .unwrap_or_else(|_| panic!("Failed to bind to {}", bind_address));

    info!("Service started successfully");
    info!("Listening on: http://{}", bind_address);

    // Get tokio threads for logging (reuse config helper)
    let tokio_threads = config_helpers::calculate_tokio_threads(&server_config.runtime);

    info!("🚀 Runtime Configuration:");
    info!("  • Server: {}", bind_address);
    info!(
        "  • Tokio worker threads: {} ({})",
        tokio_threads,
        if std::env::var("TOKIO_WORKER_THREADS").is_ok() {
            "env override"
        } else if server_config.runtime.tokio_worker_threads == "auto" {
            "auto-detected"
        } else {
            "configured"
        }
    );
    info!("  • CPU cores available: {}", num_cpus::get());
    info!(
        "  • Configuration source: {}",
        if std::path::Path::new("reframe.config.json").exists() {
            "reframe.config.json"
        } else {
            "built-in defaults"
        }
    );
    info!("");
    info!("🔧 Engines:");
    info!("  • Transform: Unified bidirectional (MT ↔ MX)");
    info!("  • Generation: Sample message creation");
    info!("  • Validation: Message compliance checking");
    info!("");
    info!("📡 API Endpoints:");
    info!("  POST /api/transform          - Bidirectional transformation (MT ↔ MX)");
    info!("  POST /api/generate           - Generate sample messages");
    info!("  POST /api/validate           - Validate messages");
    info!("  GET  /api/packages           - List loaded packages");
    info!("  POST /admin/reload-workflows - Reload workflows");
    info!("  GET  /health                 - Health check");
    info!("  GET  /swagger-ui             - API documentation");
    info!("  GET  /api-docs/openapi.json  - OpenAPI spec");

    if graphql_enabled {
        info!("");
        info!("🔍 GraphQL API:");
        info!("  POST /graphql                - GraphQL query endpoint");
        info!("  GET  /graphql/playground     - Interactive GraphQL Playground");
    }

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
