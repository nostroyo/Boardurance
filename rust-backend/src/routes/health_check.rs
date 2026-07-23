use axum::{extract::State, http::StatusCode, response::Json};
use mongodb::{bson::doc, Database};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub message: String,
    /// Build identifier, sourced from the crate version at compile time.
    pub version: String,
}

impl HealthResponse {
    /// Body for the healthy path. Shared by the handler and its unit test so the
    /// test guards exactly what the handler emits (the deploy contract).
    fn healthy() -> Self {
        Self {
            status: "ok".to_string(),
            message: "Service is healthy and database is connected".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    /// Body for the degraded path (server up, database unreachable).
    fn degraded() -> Self {
        Self {
            status: "degraded".to_string(),
            message: "Service is running but database is not available".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
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
            Ok(Json(HealthResponse::healthy()))
        }
        Err(e) => {
            tracing::warn!("Health check failed - database connection error: {}", e);
            Ok(Json(HealthResponse::degraded()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // These build the response via the SAME constructors the handler uses, so a
    // regression in the handler's healthy/degraded body is caught here in the
    // fast lane (no DB). They guard the version wiring and the deploy contract:
    // the Render deploy workflow greps the healthy body for `"status":"ok"`.
    #[test]
    fn healthy_body_carries_crate_version_and_preserves_status_contract() {
        let body =
            serde_json::to_string(&HealthResponse::healthy()).expect("serialize HealthResponse");

        // `status` serializes first, so the compact body starts with the exact
        // substring the deploy poll greps for.
        assert!(
            body.starts_with(r#"{"status":"ok""#),
            "deploy contract substring missing or not first in body: {body}"
        );
        assert!(
            body.contains(&format!(r#""version":"{}""#, env!("CARGO_PKG_VERSION"))),
            "version field missing from body: {body}"
        );
    }

    #[test]
    fn degraded_body_reports_degraded_and_still_carries_version() {
        let response = HealthResponse::degraded();
        assert_eq!(response.status, "degraded");
        assert_eq!(response.version, env!("CARGO_PKG_VERSION"));

        // The degraded body must NOT satisfy the deploy poll's healthy grep.
        let body = serde_json::to_string(&response).expect("serialize HealthResponse");
        assert!(!body.contains(r#""status":"ok""#));
    }
}
