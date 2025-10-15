use utoipa::OpenApi;
use utoipa::openapi::ServerBuilder;
use utoipa_swagger_ui::SwaggerUi;

use crate::handlers::types::{
    ConfigInfo, EngineStatus, HealthResponse, PackageDetails, PackagesResponse, ReloadResponse,
    ResponseMetadata, SampleGenerationRequest, SampleGenerationResponse, TransformationRequest,
    TransformationResponse, ValidationRequest, ValidationResponse, WorkflowInfo, WorkflowsInfo,
};
use crate::types::{ErrorType, ReframeError};

// Note: OpenAPI macro uses compile-time values, so we keep defaults here
// but allow runtime override via with_config method
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Reframe API",
        version = "3.1.1",
        description = "A Universal Message Transformation Engine",
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
        crate::handlers::admin::health_check,
        crate::handlers::transform::transform,
        crate::handlers::validate::validate,
        crate::handlers::generate::generate_sample,
        crate::handlers::admin::list_packages,
        crate::handlers::admin::reload_workflows,
    ),
    components(
        schemas(
            // Request types
            TransformationRequest,
            SampleGenerationRequest,
            ValidationRequest,

            // Response types
            HealthResponse,
            TransformationResponse,
            SampleGenerationResponse,
            ValidationResponse,
            ReloadResponse,
            PackagesResponse,

            // Shared types
            ReframeError,
            ErrorType,
            EngineStatus,
            ConfigInfo,
            ResponseMetadata,
            PackageDetails,
            WorkflowsInfo,
            WorkflowInfo,
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "transformation", description = "Message transformation endpoints"),
        (name = "validation", description = "Message validation endpoints"),
        (name = "generation", description = "Sample message generation endpoints"),
        (name = "packages", description = "Package management endpoints"),
        (name = "admin", description = "Administrative endpoints")
    ),
    external_docs(url = "https://sandbox.goplasmatic.io", description = "Full API documentation")
)]
pub struct ApiDoc;

impl ApiDoc {
    pub fn with_config(
        api_config: &crate::package_manager::ApiDocsConfig,
    ) -> utoipa::openapi::OpenApi {
        let mut doc = Self::openapi();

        // Override with config values (env var takes precedence)
        let server_url =
            std::env::var("API_SERVER_URL").unwrap_or_else(|_| api_config.server_url.clone());

        let description = if server_url.contains("localhost") {
            "Local development server"
        } else {
            "API server"
        };

        doc.servers = Some(vec![
            ServerBuilder::new()
                .url(server_url)
                .description(Some(description.to_string()))
                .build(),
        ]);

        // Update info with config values
        doc.info.title = api_config.title.clone();
        doc.info.version = api_config.version.clone();
        doc.info.description = Some(api_config.description.clone());

        if let Some(contact) = &mut doc.info.contact {
            contact.name = Some(api_config.contact.name.clone());
            contact.email = Some(api_config.contact.email.clone());
        }

        if let Some(license) = &mut doc.info.license {
            license.name = api_config.license.name.clone();
            license.identifier = Some(api_config.license.identifier.clone());
        }

        // Set external docs
        doc.external_docs = Some(
            utoipa::openapi::external_docs::ExternalDocsBuilder::new()
                .url(api_config.external_docs_url.clone())
                .description(Some("Full API documentation".to_string()))
                .build(),
        );

        doc
    }
}

pub fn swagger_ui_with_config(api_config: &crate::package_manager::ApiDocsConfig) -> SwaggerUi {
    SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::with_config(api_config))
}
