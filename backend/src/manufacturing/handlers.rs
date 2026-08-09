//! Manufacturing HTTP handlers.

use axum::extract::{Extension, Path, Query};
use axum::response::Response;
use axum::Json;
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::dto::manufacturing_dto::{
    CreateBomRequest, CreateInspectionRequest, CreateNcrRequest, CreateWorkOrderRequest,
    ResolveNcrRequest,
};
use crate::error::AppError;
use crate::handlers::auth_handler::AuthenticatedUser;
use crate::manufacturing::services::ManufacturingService;
use crate::models::manufacturing::{Bom, BomItem, Inspection, Ncr, WorkOrder, WorkOrderStep};
use crate::response::ApiResponse;

#[derive(Debug, Deserialize)]
pub struct StatusFilter {
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct WoFilter {
    pub work_order_id: Option<i64>,
}

// BOMs
pub async fn list_boms(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
) -> Result<Json<ApiResponse<Vec<Bom>>>, AppError> {
    Ok(ApiResponse::ok(ManufacturingService::list_boms(&pool, user.0.tenant_id).await?))
}

pub async fn get_bom(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<BomDetail>>, AppError> {
    let (bom, items) = ManufacturingService::get_bom(&pool, user.0.tenant_id, id).await?;
    Ok(ApiResponse::ok(BomDetail { bom, items }))
}

#[derive(serde::Serialize)]
pub struct BomDetail {
    pub bom: Bom,
    pub items: Vec<BomItem>,
}

pub async fn create_bom(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
    Json(p): Json<CreateBomRequest>,
) -> Result<Response, AppError> {
    Ok(ApiResponse::created(ManufacturingService::create_bom(&pool, user.0.tenant_id, &p).await?))
}

// Work orders
pub async fn list_work_orders(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
    Query(f): Query<StatusFilter>,
) -> Result<Json<ApiResponse<Vec<WorkOrder>>>, AppError> {
    Ok(ApiResponse::ok(ManufacturingService::list_work_orders(&pool, user.0.tenant_id, f.status.as_deref()).await?))
}

pub async fn get_work_order(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<WorkOrderDetail>>, AppError> {
    let (wo, steps) = ManufacturingService::get_work_order(&pool, user.0.tenant_id, id).await?;
    Ok(ApiResponse::ok(WorkOrderDetail { work_order: wo, steps }))
}

#[derive(serde::Serialize)]
pub struct WorkOrderDetail {
    pub work_order: WorkOrder,
    pub steps: Vec<WorkOrderStep>,
}

pub async fn create_work_order(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
    Json(p): Json<CreateWorkOrderRequest>,
) -> Result<Response, AppError> {
    Ok(ApiResponse::created(ManufacturingService::create_work_order(&pool, user.0.tenant_id, &p).await?))
}

pub async fn start_work_order(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<WorkOrder>>, AppError> {
    Ok(ApiResponse::ok(ManufacturingService::start_work_order(&pool, user.0.tenant_id, id).await?))
}

pub async fn complete_step(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<WorkOrder>>, AppError> {
    Ok(ApiResponse::ok(ManufacturingService::complete_step(&pool, user.0.tenant_id, id).await?))
}

// Inspections
pub async fn list_inspections(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
    Query(f): Query<WoFilter>,
) -> Result<Json<ApiResponse<Vec<Inspection>>>, AppError> {
    Ok(ApiResponse::ok(ManufacturingService::list_inspections(&pool, user.0.tenant_id, f.work_order_id).await?))
}

pub async fn create_inspection(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
    Json(p): Json<CreateInspectionRequest>,
) -> Result<Response, AppError> {
    Ok(ApiResponse::created(ManufacturingService::create_inspection(&pool, user.0.tenant_id, &p, Some(user.0.user_id)).await?))
}

// NCRs
pub async fn list_ncrs(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
    Query(f): Query<StatusFilter>,
) -> Result<Json<ApiResponse<Vec<Ncr>>>, AppError> {
    Ok(ApiResponse::ok(ManufacturingService::list_ncrs(&pool, user.0.tenant_id, f.status.as_deref()).await?))
}

pub async fn create_ncr(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
    Json(p): Json<CreateNcrRequest>,
) -> Result<Response, AppError> {
    Ok(ApiResponse::created(ManufacturingService::create_ncr(&pool, user.0.tenant_id, &p, Some(user.0.user_id)).await?))
}

pub async fn resolve_ncr(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
    Path(id): Path<i64>,
    Json(p): Json<ResolveNcrRequest>,
) -> Result<Json<ApiResponse<Ncr>>, AppError> {
    Ok(ApiResponse::ok(ManufacturingService::resolve_ncr(&pool, user.0.tenant_id, id, &p).await?))
}
