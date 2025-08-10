use axum::{
    Router,
    routing::{get, post},
};
use tracing::info;
use tracing_subscriber::EnvFilter;

// Module declarations
mod engine;
mod handlers;
mod helper;
mod mx_sample_generator;
mod openapi;
mod parse_mt;
mod parse_mx;
mod publish_mt;
mod publish_mx;
mod sample_generator;
mod types;

// Import public items from modules
use engine::initialize_engines;
use handlers::{
    generate_sample, health_check, reload_workflows, transform_mt_to_mx, transform_mx_to_mt,
    validate_mt, validate_mx,
};
use openapi::swagger_ui;

#[tokio::main]
async fn main() {
    // Initialize logging
    initialize_logging();

    // Initialize scenario paths for sample generation
    initialize_scenario_paths();

    info!("🚀 Starting Reframe Bidirectional Transformation Service");

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
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("🌐 Server running on http://0.0.0.0:3000");
    info!("📡 Forward endpoint: POST /transform/mt-to-mx");
    info!("📡 Reverse endpoint: POST /transform/mx-to-mt");
    info!("🔧 Sample generation: POST /generate/sample (supports MT and MX)");
    info!("🔍 MT validation: POST /validate/mt");
    info!("🔍 MX validation: POST /validate/mx");
    info!("🔄 Workflow reload: POST /admin/reload-workflows");
    info!("🏥 Health check: GET /health");
    info!("📚 API Documentation: GET /swagger-ui");
    info!("📋 OpenAPI Spec: GET /api-docs/openapi.json");

    axum::serve(listener, app).await.unwrap();
}

fn initialize_logging() {
    // Simple tracing initialization to avoid conflicts
    if std::env::var("RUST_LOG").is_err() {
        unsafe {
            std::env::set_var("RUST_LOG", "reframe=debug,info");
        }
    }

    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_env_filter(EnvFilter::from_default_env())
        .try_init()
        .ok(); // Ignore errors if already initialized
}

fn initialize_scenario_paths() {
    // Set environment variables for scenario paths
    // These will be used by swift-mt-message and mx-message libraries
    unsafe {
        std::env::set_var("SWIFT_SCENARIO_PATH", "scenarios/SwiftMTMessage");
        std::env::set_var("MX_SCENARIO_PATH", "scenarios/MXMessage");
    }
    info!("📁 Scenario paths configured:");
    info!("   SWIFT: scenarios/SwiftMTMessage");
    info!("   MX: scenarios/MXMessage");
}
