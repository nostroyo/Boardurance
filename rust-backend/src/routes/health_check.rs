use axum::{extract::State, http::StatusCode, response::Json};
use mongodb::{bson::doc, Database};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub message: String,
}

/// Health check endpoint
#[utoipa::path(
    get,
    path = "/health_check",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse)
    ),
    tag = "health"
)]
#[tracing::instrument(name = "Health check", skip(database))]
pub async fn health_check(
    State(database): State<Database>,
) -> Result<Json<HealthResponse>, StatusCode> {
    // For health check, we'll try to list collections which is a simple operation
    match database.list_collection_names(None).await {
        Ok(_) => {
            tracing::info!("Health check successful - database connected");
            Ok(Json(HealthResponse {
                status: "ok".to_string(),
                message: "Service is healthy and database is connected".to_string(),
            }))
        }
        Err(e) => {
            tracing::warn!("Health check failed - database connection error: {}", e);
            Ok(Json(HealthResponse {
                status: "degraded".to_string(),
                message: "Service is running but database is not available".to_string(),
            }))
        }
    }
}
