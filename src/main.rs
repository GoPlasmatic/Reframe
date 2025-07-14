use axum::{
    Router,
    routing::{get, post},
};
use tower_http::services::ServeDir;
use tracing::info;
use tracing_subscriber::EnvFilter;

// Module declarations
mod engine;
mod handlers;
mod helper;
mod parse_mt;
mod parse_mx;
mod publish_mt;
mod publish_mx;
mod sample_generator;
mod types;

// Import public items from modules
use engine::initialize_engines;
use handlers::{generate_mt_sample, health_check, reload_workflows, transform_mt_to_mx, transform_mx_to_mt};

#[tokio::main]
async fn main() {
    // Initialize logging
    initialize_logging();

    info!("🚀 Starting Reframe Bidirectional Transformation Service");

    // Initialize dual engines
    let app_state = initialize_engines().await;

    // Build router with new endpoints
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/transform/mt-to-mx", post(transform_mt_to_mx))
        .route("/transform/mx-to-mt", post(transform_mx_to_mt))
        .route("/generate/mt-sample", post(generate_mt_sample))
        .route("/admin/reload-workflows", post(reload_workflows))
        .nest_service("/", ServeDir::new("static"))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("🌐 Server running on http://0.0.0.0:3000");
    info!("📡 Forward endpoint: POST /transform/mt-to-mx");
    info!("📡 Reverse endpoint: POST /transform/mx-to-mt");
    info!("🔧 Sample generation: POST /generate/mt-sample");
    info!("🔄 Workflow reload: POST /admin/reload-workflows");
    info!("🏥 Health check: GET /health");

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
