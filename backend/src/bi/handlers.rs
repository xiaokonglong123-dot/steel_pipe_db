//! BI handlers — aggregate analytics endpoints.

use axum::extract::{Extension, Query};
use axum::Json;
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::bi::services::{
    BiService, FinanceSummary, InventoryValueRow, SalesTrendRow, SupplierPerfRow,
};
use crate::error::AppError;
use crate::handlers::auth_handler::AuthenticatedUser;
use crate::response::ApiResponse;

#[derive(Debug, Deserialize)]
pub struct TrendFilter {
    pub months: Option<i32>,
}

pub async fn sales_trend(
    Extension(pool): Extension<SqlitePool>,
    _user: AuthenticatedUser,
    Query(f): Query<TrendFilter>,
) -> Result<Json<ApiResponse<Vec<SalesTrendRow>>>, AppError> {
    Ok(ApiResponse::ok(BiService::sales_trend(&pool, 1, f.months.unwrap_or(12)).await?))
}

pub async fn inventory_value(
    Extension(pool): Extension<SqlitePool>,
    _user: AuthenticatedUser,
) -> Result<Json<ApiResponse<Vec<InventoryValueRow>>>, AppError> {
    Ok(ApiResponse::ok(BiService::inventory_value(&pool).await?))
}

pub async fn finance_summary(
    Extension(pool): Extension<SqlitePool>,
    _user: AuthenticatedUser,
) -> Result<Json<ApiResponse<FinanceSummary>>, AppError> {
    Ok(ApiResponse::ok(BiService::finance_summary(&pool, 1).await?))
}

pub async fn supplier_performance(
    Extension(pool): Extension<SqlitePool>,
    _user: AuthenticatedUser,
) -> Result<Json<ApiResponse<Vec<SupplierPerfRow>>>, AppError> {
    Ok(ApiResponse::ok(BiService::supplier_performance(&pool, 1).await?))
}
