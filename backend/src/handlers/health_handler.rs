//! Health check handler — provides `/api/v1/health` endpoint for container
//! orchestration (Kubernetes liveness/readiness probes, Docker HEALTHCHECK).

use axum::{extract::Extension, Json};
use serde::Serialize;
use sqlx::SqlitePool;

use crate::response::ApiResponse;

/// Health check response payload.
#[derive(Debug, Serialize)]
pub struct HealthStatus {
    /// Overall service status: "ok" or "degraded".
    pub status: String,
    /// Database connectivity: "ok" or "error".
    pub database: String,
    /// Service version from Cargo.toml.
    pub version: String,
}

/// GET `/api/v1/health` — Health check endpoint.
///
/// Returns service health status including database connectivity.
/// Used by Kubernetes/Docker for liveness and readiness probes.
/// Does NOT require authentication.
pub async fn health_handler(
    Extension(pool): Extension<SqlitePool>,
) -> Json<ApiResponse<HealthStatus>> {
    let db_status = match sqlx::query("SELECT 1").execute(&pool).await {
        Ok(_) => "ok".to_string(),
        Err(e) => {
            tracing::error!("Health check DB query failed: {}", e);
            "error".to_string()
        }
    };

    let overall = if db_status == "ok" { "ok" } else { "degraded" };

    ApiResponse::ok(HealthStatus {
        status: overall.to_string(),
        database: db_status,
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}
