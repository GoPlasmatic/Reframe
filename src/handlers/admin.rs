use axum::{Json, extract::State, http::StatusCode};
use std::time::Instant;
use tracing::{error, info, instrument};

use super::types::{
    ConfigInfo, DatabaseStatus, EngineStatus, HealthResponse, PackageDetails, PackagesResponse,
    ReloadResponse, WorkflowInfo, WorkflowsInfo,
};
use crate::engine::reload_engines;
use crate::types::AppState;

// Health check endpoint
#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse),
    )
)]
pub async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    // Async Engine is always healthy if it exists
    let cpu_count = num_cpus::get();

    let transform_status =
        "healthy (Unified bidirectional, Async Engine, Tokio runtime)".to_string();
    let generation_status = "healthy (Async Engine, Tokio runtime)".to_string();
    let validation_status = "healthy (Async Engine, Tokio runtime)".to_string();

    // Check database status
    let database_status = if let Some(ref client) = state.database_client {
        match client.ping().await {
            Ok(_) => DatabaseStatus {
                enabled: true,
                status: "connected".to_string(),
                message: Some(format!(
                    "Persistence: transform={}, validate={}, generate={}",
                    state.database_config.persist_transform,
                    state.database_config.persist_validate,
                    state.database_config.persist_generate
                )),
            },
            Err(e) => DatabaseStatus {
                enabled: true,
                status: "error".to_string(),
                message: Some(format!("Connection failed: {}", e)),
            },
        }
    } else {
        DatabaseStatus {
            enabled: false,
            status: "disabled".to_string(),
            message: Some("Database persistence is disabled".to_string()),
        }
    };

    Json(HealthResponse {
        status: "running".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        engines: EngineStatus {
            transform: transform_status,
            generation: generation_status,
            validation: validation_status,
        },
        database: database_status,
        config: Some(ConfigInfo {
            thread_count: cpu_count,
        }),
    })
}

#[utoipa::path(
    post,
    path = "/admin/reload-workflows",
    tag = "admin",
    responses(
        (status = 200, description = "Workflows reloaded successfully", body = ReloadResponse),
        (status = 500, description = "Failed to reload workflows", body = ReloadResponse)
    )
)]
#[instrument(skip(state))]
pub async fn reload_workflows(
    State(state): State<AppState>,
) -> Result<Json<ReloadResponse>, StatusCode> {
    let start_time = Instant::now();

    info!("🔄 Processing workflow reload request");

    match reload_engines(&state).await {
        Ok(()) => {
            let reload_time = start_time.elapsed().as_millis() as u64;
            info!(
                "✅ Workflow reload completed successfully in {}ms",
                reload_time
            );

            Ok(Json(ReloadResponse {
                success: true,
                message: format!("Workflows reloaded successfully in {reload_time}ms"),
                timestamp: chrono::Utc::now().to_rfc3339(),
                error: None,
            }))
        }
        Err(e) => {
            let reload_time = start_time.elapsed().as_millis() as u64;
            error!(
                reload_time_ms = reload_time,
                error = %e,
                "Workflow reload failed"
            );

            Ok(Json(ReloadResponse {
                success: false,
                message: format!("Workflow reload failed after {reload_time}ms"),
                timestamp: chrono::Utc::now().to_rfc3339(),
                error: Some(e.to_string()),
            }))
        }
    }
}

/// List all loaded packages with their details
#[utoipa::path(
    get,
    path = "/api/packages",
    tag = "packages",
    responses(
        (status = 200, description = "Packages listed successfully", body = PackagesResponse),
    )
)]
pub async fn list_packages(State(state): State<AppState>) -> Json<PackagesResponse> {
    info!("📦 Processing package list request");

    let pm = state.package_manager.read().unwrap();
    let packages = pm.get_packages();

    let package_details: Vec<PackageDetails> = packages
        .values()
        .map(|pkg| {
            // Build workflow info
            let transform_info = pkg
                .workflows
                .get("transform")
                .map(|w| WorkflowInfo {
                    available: true,
                    description: w.description.clone(),
                })
                .unwrap_or_else(|| WorkflowInfo {
                    available: false,
                    description: "Not available".to_string(),
                });

            let generate_info = pkg
                .workflows
                .get("generate")
                .map(|w| WorkflowInfo {
                    available: true,
                    description: w.description.clone(),
                })
                .unwrap_or_else(|| WorkflowInfo {
                    available: false,
                    description: "Not available".to_string(),
                });

            let validate_info = pkg
                .workflows
                .get("validate")
                .map(|w| WorkflowInfo {
                    available: true,
                    description: w.description.clone(),
                })
                .unwrap_or_else(|| WorkflowInfo {
                    available: false,
                    description: "Not available".to_string(),
                });

            PackageDetails {
                id: pkg.id.clone(),
                name: pkg.name.clone(),
                version: pkg.version.clone(),
                description: pkg.description.clone(),
                author: pkg.author.clone(),
                license: pkg.license.clone(),
                engine_version: pkg.engine_version.clone(),
                status: if pkg.enabled { "loaded" } else { "disabled" }.to_string(),
                workflows: WorkflowsInfo {
                    transform: transform_info,
                    generate: generate_info,
                    validate: validate_info,
                },
                loaded_at: pkg.loaded_at.to_rfc3339(),
            }
        })
        .collect();

    info!("✅ Listed {} package(s)", package_details.len());

    Json(PackagesResponse {
        success: true,
        packages: package_details,
    })
}
