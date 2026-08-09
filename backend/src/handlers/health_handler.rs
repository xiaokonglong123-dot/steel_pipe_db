//! Health check handlers — provides `/api/v1/health` and `/api/v1/health/ready` endpoints
//! for container orchestration (Kubernetes liveness/readiness probes, Docker HEALTHCHECK).

use std::sync::OnceLock;
use std::time::Instant;

use axum::{extract::Extension, Json};
use chrono::Utc;
use serde::Serialize;
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::response::ApiResponse;

/// Server start time — set once on first request, used for uptime calculation.
static STARTED_AT: OnceLock<Instant> = OnceLock::new();

fn started_at() -> Instant {
    *STARTED_AT.get_or_init(Instant::now)
}

/// Health check response payload.
#[derive(Debug, Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub database: String,
    pub version: String,
    pub timestamp: String,
    pub uptime_seconds: u64,
}

/// Readiness check response payload.
#[derive(Debug, Serialize)]
pub struct ReadinessStatus {
    pub status: String,
    pub timestamp: String,
    pub uptime_seconds: u64,
    pub version: String,
}

/// GET `/api/v1/health` — Liveness check.
///
/// Returns service health status including database connectivity.
/// Used by Kubernetes/Docker for liveness probes.
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
        timestamp: Utc::now().to_rfc3339(),
        uptime_seconds: started_at().elapsed().as_secs(),
    })
}

/// GET `/api/v1/health/ready` — Readiness check.
///
/// Verifies the service can handle traffic by testing the database connection.
/// Returns 503 if the database is unreachable.
/// Used by Kubernetes for readiness probes.
pub async fn readiness_handler(
    Extension(pool): Extension<SqlitePool>,
) -> Result<Json<ApiResponse<ReadinessStatus>>, AppError> {
    sqlx::query("SELECT 1")
        .execute(&pool)
        .await
        .map_err(|e| {
            tracing::error!("Readiness check DB query failed: {}", e);
            AppError::Internal(format!("Database unreachable: {}", e))
        })?;

    Ok(ApiResponse::ok(ReadinessStatus {
        status: "ok".to_string(),
        timestamp: Utc::now().to_rfc3339(),
        uptime_seconds: started_at().elapsed().as_secs(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    }))
}
