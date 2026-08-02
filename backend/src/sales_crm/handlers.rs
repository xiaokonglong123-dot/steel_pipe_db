//! Sales CRM HTTP handlers.

use axum::extract::{Extension, Path, Query};
use axum::Json;
use serde::Deserialize;
use sqlx::PgPool;

use crate::dto::sales_crm_dto::{CreateSalesQuoteRequest, CreateShipmentRequest, UpdateShipmentStatusRequest};
use crate::error::AppError;
use crate::handlers::auth_handler::AuthenticatedUser;
use crate::models::sales_crm::{CustomerCredit, SalesQuote, SalesShipment};
use crate::response::ApiResponse;
use crate::sales_crm::services::SalesCrmService;

#[derive(Debug, Deserialize)]
pub struct IdFilter {
    pub sales_order_id: Option<i64>,
    pub customer_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct StatusRequest {
    pub status: String,
}

// Shipments
pub async fn list_shipments(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Query(f): Query<IdFilter>,
) -> Result<Json<ApiResponse<Vec<SalesShipment>>>, AppError> {
    Ok(ApiResponse::ok(SalesCrmService::list_shipments(&pool, user.0.tenant_id, f.sales_order_id).await?))
}

pub async fn create_shipment(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Json(p): Json<CreateShipmentRequest>,
) -> Result<Json<ApiResponse<SalesShipment>>, AppError> {
    Ok(ApiResponse::ok(SalesCrmService::create_shipment(&pool, user.0.tenant_id, Some(user.0.user_id), &p).await?))
}

pub async fn update_shipment_status(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Path(id): Path<i64>,
    Json(p): Json<UpdateShipmentStatusRequest>,
) -> Result<Json<ApiResponse<SalesShipment>>, AppError> {
    Ok(ApiResponse::ok(SalesCrmService::update_shipment_status(&pool, user.0.tenant_id, id, &p.status).await?))
}

// Quotes
pub async fn list_quotes(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Query(f): Query<IdFilter>,
) -> Result<Json<ApiResponse<Vec<SalesQuote>>>, AppError> {
    Ok(ApiResponse::ok(SalesCrmService::list_quotes(&pool, user.0.tenant_id, f.customer_id).await?))
}

pub async fn get_quote(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<SalesQuote>>, AppError> {
    Ok(ApiResponse::ok(SalesCrmService::get_quote(&pool, user.0.tenant_id, id).await?))
}

pub async fn create_quote(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Json(p): Json<CreateSalesQuoteRequest>,
) -> Result<Json<ApiResponse<SalesQuote>>, AppError> {
    Ok(ApiResponse::ok(SalesCrmService::create_quote(&pool, user.0.tenant_id, &p).await?))
}

pub async fn update_quote_status(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Path(id): Path<i64>,
    Json(p): Json<StatusRequest>,
) -> Result<Json<ApiResponse<SalesQuote>>, AppError> {
    Ok(ApiResponse::ok(SalesCrmService::update_quote_status(&pool, user.0.tenant_id, id, &p.status).await?))
}

/// Convert a confirmed quote into a sales order.
pub async fn convert_quote(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let order_id = SalesCrmService::convert_quote(&pool, user.0.tenant_id, id).await?;
    Ok(ApiResponse::ok(serde_json::json!({ "order_id": order_id })))
}

// Credit
pub async fn customer_credit(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Path(customer_id): Path<i64>,
) -> Result<Json<ApiResponse<CustomerCredit>>, AppError> {
    Ok(ApiResponse::ok(SalesCrmService::customer_credit(&pool, user.0.tenant_id, customer_id).await?))
}
