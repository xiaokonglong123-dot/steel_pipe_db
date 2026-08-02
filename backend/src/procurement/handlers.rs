//! Procurement HTTP handlers.

use axum::extract::{Extension, Path, Query};
use axum::Json;
use serde::Deserialize;
use sqlx::PgPool;

use crate::dto::procurement_dto::{
    CreateQuoteRequest, CreateReceiptRequest, CreateRequisitionRequest, UpdateQuoteStatusRequest,
};
use crate::error::AppError;
use crate::handlers::auth_handler::AuthenticatedUser;
use crate::models::procurement::{
    PoReceipt, PoReceiptItem, PurchaseRequisition, SupplierQuote, SupplierScorecard,
};
use crate::procurement::services::ProcurementService;
use crate::response::ApiResponse;

#[derive(Debug, Deserialize)]
pub struct StatusFilter {
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct IdFilter {
    pub supplier_id: Option<i64>,
    pub purchase_order_id: Option<i64>,
}

// Requisitions
pub async fn list_requisitions(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Query(f): Query<StatusFilter>,
) -> Result<Json<ApiResponse<Vec<PurchaseRequisition>>>, AppError> {
    Ok(ApiResponse::ok(ProcurementService::list_requisitions(&pool, user.0.tenant_id, f.status.as_deref()).await?))
}

pub async fn get_requisition(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<PurchaseRequisition>>, AppError> {
    Ok(ApiResponse::ok(ProcurementService::get_requisition(&pool, user.0.tenant_id, id).await?))
}

pub async fn create_requisition(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Json(p): Json<CreateRequisitionRequest>,
) -> Result<Json<ApiResponse<PurchaseRequisition>>, AppError> {
    Ok(ApiResponse::ok(ProcurementService::create_requisition(&pool, user.0.tenant_id, Some(user.0.user_id), &p).await?))
}

pub async fn update_requisition_status(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Path(id): Path<i64>,
    Json(p): Json<StatusFilter>,
) -> Result<Json<ApiResponse<PurchaseRequisition>>, AppError> {
    let status = p.status.unwrap_or_default();
    Ok(ApiResponse::ok(ProcurementService::update_requisition_status(&pool, user.0.tenant_id, id, &status).await?))
}

// Goods receipts
pub async fn list_receipts(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Query(f): Query<IdFilter>,
) -> Result<Json<ApiResponse<Vec<PoReceipt>>>, AppError> {
    Ok(ApiResponse::ok(ProcurementService::list_receipts(&pool, user.0.tenant_id, f.purchase_order_id).await?))
}

pub async fn get_receipt(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<ReceiptDetail>>, AppError> {
    let (receipt, items) = ProcurementService::get_receipt(&pool, user.0.tenant_id, id).await?;
    Ok(ApiResponse::ok(ReceiptDetail { receipt, items }))
}

#[derive(serde::Serialize)]
pub struct ReceiptDetail {
    pub receipt: PoReceipt,
    pub items: Vec<PoReceiptItem>,
}

pub async fn create_receipt(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Json(p): Json<CreateReceiptRequest>,
) -> Result<Json<ApiResponse<PoReceipt>>, AppError> {
    Ok(ApiResponse::ok(ProcurementService::create_receipt(&pool, user.0.tenant_id, Some(user.0.user_id), &p).await?))
}

// Supplier quotes
pub async fn list_quotes(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Query(f): Query<IdFilter>,
) -> Result<Json<ApiResponse<Vec<SupplierQuote>>>, AppError> {
    Ok(ApiResponse::ok(ProcurementService::list_quotes(&pool, user.0.tenant_id, f.supplier_id).await?))
}

pub async fn create_quote(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Json(p): Json<CreateQuoteRequest>,
) -> Result<Json<ApiResponse<SupplierQuote>>, AppError> {
    Ok(ApiResponse::ok(ProcurementService::create_quote(&pool, user.0.tenant_id, &p).await?))
}

pub async fn update_quote_status(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Path(id): Path<i64>,
    Json(p): Json<UpdateQuoteStatusRequest>,
) -> Result<Json<ApiResponse<SupplierQuote>>, AppError> {
    Ok(ApiResponse::ok(ProcurementService::update_quote_status(&pool, user.0.tenant_id, id, &p).await?))
}

pub async fn supplier_scorecard(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Path(supplier_id): Path<i64>,
) -> Result<Json<ApiResponse<SupplierScorecard>>, AppError> {
    Ok(ApiResponse::ok(ProcurementService::supplier_scorecard(&pool, user.0.tenant_id, supplier_id).await?))
}
