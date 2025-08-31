use axum::{
    Router,
    routing::{get, post},
    middleware,
};
use tracing::info;

// Module declarations
mod engine;
mod logging;
mod handlers;
mod helper;
mod mt_generator;
mod mx_generator;
mod openapi;
mod parse_mt;
mod parse_mx;
mod publish_mt;
mod publish_mx;
mod sample_generator;
mod scenario_loader;
mod types;
mod validation_helpers;

// Import public items from modules
use engine::initialize_engines;
use handlers::{
    generate_sample, health_check, reload_workflows, transform_mt_to_mx, transform_mx_to_mt,
    validate_mt, validate_mx, correlation_middleware,
};
use logging::{init_logging, log_system_info, LogConfig};
use openapi::swagger_ui;

#[tokio::main]
async fn main() {
    // Initialize professional logging system
    let log_config = LogConfig {
        format: if std::env::var("LOG_FORMAT").as_deref() == Ok("json") {
            logging::LogFormat::Json
        } else if cfg!(debug_assertions) {
            logging::LogFormat::Pretty
        } else {
            logging::LogFormat::Compact
        },
        ..Default::default()
    };
    
    if let Err(e) = init_logging(log_config) {
        // Use eprintln here since logging isn't initialized yet
        eprintln!("Failed to initialize logging: {}", e);
    }

    // Log system information
    log_system_info(env!("CARGO_PKG_VERSION"));

    // Initialize scenario paths for sample generation
    initialize_scenario_paths();

    info!("Service initialization started");
    
    // Log concurrency configuration
    let concurrency = std::env::var("ENGINE_CONCURRENCY")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(16);
    info!("Engine concurrency level: {} (set ENGINE_CONCURRENCY env var to change)", concurrency);

    // Initialize dual engines
    let app_state = initialize_engines().await;

    // Build router with new endpoints
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/transform/mt-to-mx", post(transform_mt_to_mx))
        .route("/transform/mx-to-mt", post(transform_mx_to_mt))
        .route("/generate/sample", post(generate_sample))
        .route("/validate/mt", post(validate_mt))
        .route("/validate/mx", post(validate_mx))
        .route("/admin/reload-workflows", post(reload_workflows))
        .merge(swagger_ui())
        .layer(middleware::from_fn(correlation_middleware))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    
    info!("Service started successfully");
    info!("Listening on: http://0.0.0.0:3000");
    info!("Available endpoints:");
    info!("  POST /transform/mt-to-mx    - MT to ISO 20022 transformation");
    info!("  POST /transform/mx-to-mt    - ISO 20022 to MT transformation");
    info!("  POST /generate/sample       - Generate sample messages");
    info!("  POST /validate/mt           - Validate MT messages");
    info!("  POST /validate/mx           - Validate ISO 20022 messages");
    info!("  POST /admin/reload-workflows - Reload workflow configurations");
    info!("  GET  /health                - Health check endpoint");
    info!("  GET  /swagger-ui            - API documentation");
    info!("  GET  /api-docs/openapi.json - OpenAPI specification");

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}


fn initialize_scenario_paths() {
    // Set environment variables for scenario paths
    // These will be used by swift-mt-message and mx-message libraries
    unsafe {
        std::env::set_var("SWIFT_SCENARIO_PATH", "scenarios/SwiftMTMessage");
        std::env::set_var("MX_SCENARIO_PATH", "scenarios/MXMessage");
    }
    tracing::debug!("Scenario paths configured");
    tracing::debug!("  SWIFT: scenarios/SwiftMTMessage");
    tracing::debug!("  MX: scenarios/MXMessage");
}
