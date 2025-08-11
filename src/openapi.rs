use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use utoipa::openapi::ServerBuilder;

use crate::types::{
    DebugInfo, EngineStatus, ErrorType, HealthResponse, ReframeError, ReloadResponse,
    ResponseMetadata, SampleDebugInfo, SampleGenerationOptions, SampleGenerationRequest,
    SampleGenerationResponse, TransformationOptions, TransformationRequest, TransformationResponse,
    ValidationOptions, ValidationRequest, ValidationResponse,
};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Reframe API",
        version = "3.0.0",
        description = "Enterprise-grade bidirectional SWIFT MT ↔ ISO 20022 transformation service",
        contact(
            name = "Plasmatic Team",
            email = "enquires@goplasmatic.io"
        ),
        license(
            name = "Apache 2.0",
            identifier = "Apache-2.0",
            url = "https://opensource.org/license/apache-2-0"
        )
    ),
    paths(
        crate::handlers::health_check,
        crate::handlers::transform_mt_to_mx,
        crate::handlers::transform_mx_to_mt,
        crate::handlers::generate_sample,
        crate::handlers::validate_mt,
        crate::handlers::validate_mx,
        crate::handlers::reload_workflows,
    ),
    components(
        schemas(
            // Request types
            TransformationRequest,
            TransformationOptions,
            SampleGenerationRequest,
            SampleGenerationOptions,
            ValidationRequest,
            ValidationOptions,

            // Response types
            HealthResponse,
            TransformationResponse,
            SampleGenerationResponse,
            ValidationResponse,
            ReloadResponse,

            // Shared types
            ReframeError,
            ErrorType,
            EngineStatus,
            DebugInfo,
            SampleDebugInfo,
            ResponseMetadata,
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "transformation", description = "Message transformation endpoints"),
        (name = "validation", description = "Message validation endpoints"),
        (name = "generation", description = "Sample message generation endpoints"),
        (name = "admin", description = "Administrative endpoints")
    ),
    external_docs(url = "https://sandbox.goplasmatic.io", description = "Full API documentation")
)]
pub struct ApiDoc;

impl ApiDoc {
    pub fn with_server() -> utoipa::openapi::OpenApi {
        let mut doc = Self::openapi();
        
        // Get server URL from environment variable, default to localhost
        let server_url = std::env::var("API_SERVER_URL")
            .unwrap_or_else(|_| "http://localhost:3000".to_string());
        
        let description = if server_url.contains("localhost") {
            "Local development server"
        } else {
            "API server"
        };
        
        doc.servers = Some(vec![
            ServerBuilder::new()
                .url(server_url)
                .description(Some(description.to_string()))
                .build()
        ]);
        
        doc
    }
}

pub fn swagger_ui() -> SwaggerUi {
    SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::with_server())
}
