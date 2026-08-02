//! Inventory ATP HTTP handlers.

use axum::extract::{Extension, Path, Query};
use axum::Json;
use serde::Deserialize;
use sqlx::PgPool;

use crate::dto::inventory_atp_dto::{
    CompleteCountSessionRequest, CreateCountTemplateRequest, CreateReservationRequest,
    CreateTransferRequest,
};
use crate::error::AppError;
use crate::handlers::auth_handler::AuthenticatedUser;
use crate::inventory_atp::services::InventoryAtpService;
use crate::models::inventory_atp::{
    AtpOverviewRow, AtpSlot, CountSession, CountTemplate, InternalTransfer,
};
use crate::response::ApiResponse;

#[derive(Debug, Deserialize)]
pub struct PipeQuery {
    pub pipe_type: String,
    pub pipe_number: Option<String>,
}

// ATP
pub async fn reserve(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Json(p): Json<CreateReservationRequest>,
) -> Result<Json<ApiResponse<AtpSlot>>, AppError> {
    Ok(ApiResponse::ok(InventoryAtpService::reserve(&pool, user.0.tenant_id, &p).await?))
}

pub async fn release(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<AtpSlot>>, AppError> {
    Ok(ApiResponse::ok(InventoryAtpService::release(&pool, user.0.tenant_id, id).await?))
}

pub async fn overview(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
) -> Result<Json<ApiResponse<Vec<AtpOverviewRow>>>, AppError> {
    Ok(ApiResponse::ok(InventoryAtpService::overview(&pool, user.0.tenant_id).await?))
}

pub async fn pipe_atp(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Query(q): Query<PipeQuery>,
) -> Result<Json<ApiResponse<AtpOverviewRow>>, AppError> {
    let number = q.pipe_number.unwrap_or_default();
    Ok(ApiResponse::ok(InventoryAtpService::pipe_atp(&pool, user.0.tenant_id, &q.pipe_type, &number).await?))
}

// Internal transfers
pub async fn create_transfer(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Json(p): Json<CreateTransferRequest>,
) -> Result<Json<ApiResponse<InternalTransfer>>, AppError> {
    Ok(ApiResponse::ok(InventoryAtpService::create_transfer(&pool, user.0.tenant_id, Some(user.0.user_id), &p).await?))
}

pub async fn list_transfers(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
) -> Result<Json<ApiResponse<Vec<InternalTransfer>>>, AppError> {
    Ok(ApiResponse::ok(InventoryAtpService::list_transfers(&pool, user.0.tenant_id).await?))
}

// Cycle counting
pub async fn create_count_template(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Json(p): Json<CreateCountTemplateRequest>,
) -> Result<Json<ApiResponse<CountTemplate>>, AppError> {
    Ok(ApiResponse::ok(InventoryAtpService::create_count_template(&pool, user.0.tenant_id, &p).await?))
}

pub async fn list_count_templates(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
) -> Result<Json<ApiResponse<Vec<CountTemplate>>>, AppError> {
    Ok(ApiResponse::ok(InventoryAtpService::list_count_templates(&pool, user.0.tenant_id).await?))
}

pub async fn start_count_session(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Path(template_id): Path<i64>,
) -> Result<Json<ApiResponse<CountSession>>, AppError> {
    Ok(ApiResponse::ok(InventoryAtpService::start_count_session(&pool, user.0.tenant_id, template_id).await?))
}

pub async fn complete_count_session(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Json(p): Json<CompleteCountSessionRequest>,
) -> Result<Json<ApiResponse<CountSession>>, AppError> {
    Ok(ApiResponse::ok(InventoryAtpService::complete_count_session(&pool, user.0.tenant_id, &p).await?))
}

pub async fn list_count_sessions(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
) -> Result<Json<ApiResponse<Vec<CountSession>>>, AppError> {
    Ok(ApiResponse::ok(InventoryAtpService::list_count_sessions(&pool, user.0.tenant_id).await?))
}
