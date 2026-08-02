//! Threading HTTP handlers.

use axum::extract::{Extension, Query};
use axum::Json;
use serde::Deserialize;
use sqlx::PgPool;

use crate::dto::threading_dto::{
    CreateThreadingRecordRequest, DesignCheckRequest, ThreadCalcRequest,
};
use crate::error::AppError;
use crate::handlers::auth_handler::AuthenticatedUser;
use crate::models::threading::ThreadingRecord;
use crate::response::ApiResponse;
use crate::threading::services::{CalcResult, DesignCheckOutput, ThreadingService};

#[derive(Debug, Deserialize)]
pub struct PipeFilter {
    pub pipe_id: Option<i64>,
}

pub async fn create_record(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Json(p): Json<CreateThreadingRecordRequest>,
) -> Result<Json<ApiResponse<ThreadingRecord>>, AppError> {
    Ok(ApiResponse::ok(ThreadingService::create_record(&pool, user.0.tenant_id, &p, Some(user.0.user_id)).await?))
}

pub async fn list_records(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Query(f): Query<PipeFilter>,
) -> Result<Json<ApiResponse<Vec<ThreadingRecord>>>, AppError> {
    Ok(ApiResponse::ok(ThreadingService::list_records(&pool, user.0.tenant_id, f.pipe_id).await?))
}

pub async fn calc(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Json(p): Json<ThreadCalcRequest>,
) -> Result<Json<ApiResponse<CalcResult>>, AppError> {
    Ok(ApiResponse::ok(ThreadingService::calc(&pool, user.0.tenant_id, &p).await?))
}

pub async fn design_check(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Json(p): Json<DesignCheckRequest>,
) -> Result<Json<ApiResponse<DesignCheckOutput>>, AppError> {
    Ok(ApiResponse::ok(ThreadingService::design_check(&pool, user.0.tenant_id, &p).await?))
}
