// Simple test to verify the server setup without MongoDB
use axum::{
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::Serialize;
use tower_http::cors::CorsLayer;
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

#[derive(Serialize, ToSchema)]
struct SimpleResponse {
    message: String,
    status: String,
}

#[derive(OpenApi)]
#[openapi(
    paths(simple_health_check),
    components(schemas(SimpleResponse)),
    tags((name = "test", description = "Simple test endpoints"))
)]
struct ApiDoc;

/// Simple health check that doesn't require database
#[utoipa::path(
    get,
    path = "/simple-health",
    responses(
        (status = 200, description = "Simple health check", body = SimpleResponse)
    ),
    tag = "test"
)]
async fn simple_health_check() -> Result<Json<SimpleResponse>, StatusCode> {
    Ok(Json(SimpleResponse {
        message: "Rust backend with Axum, Swagger, and MongoDB setup is working!".to_string(),
        status: "ok".to_string(),
    }))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Build our application with routes
    let app = Router::new()
        .route("/simple-health", get(simple_health_check))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(CorsLayer::permissive());

    // Run the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await?;
    tracing::info!("Simple test server running on http://0.0.0.0:3001");
    tracing::info!("Swagger UI available at http://0.0.0.0:3001/swagger-ui");
    tracing::info!("Test endpoint: http://0.0.0.0:3001/simple-health");
    
    axum::serve(listener, app).await?;

    Ok(())
}