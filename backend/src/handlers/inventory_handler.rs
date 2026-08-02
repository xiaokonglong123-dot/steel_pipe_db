use axum::{extract::{Extension, Query}, Json};
use serde::Deserialize;
use sqlx::PgPool;

use crate::dto::common::PaginationParams;
use crate::dto::inventory_dto::InventoryFilter;
use crate::error::AppError;
use crate::models::inventory::InventoryLog;
use crate::response::{ApiResponse, PaginatedResponse};
use crate::services::inventory_query_service::InventoryQueryService;
use crate::services::trace_service::TraceService;

#[derive(Deserialize)]
pub struct HeatNumberQuery {
    pub heat_number: String,
}

pub async fn list_inventory_handler(
    Extension(pool): Extension<PgPool>,
    Query(filter): Query<InventoryFilter>,
) -> Result<Json<PaginatedResponse<crate::dto::inventory_dto::StockItem>>, AppError> {
    let pagination = PaginationParams {
        page: filter.page,
        page_size: filter.page_size,
        sort_by: None,
        sort_order: None,
    };
    let page = pagination.page();
    let page_size = pagination.page_size();

    let (items, total) = InventoryQueryService::list_inventory(&pool, &filter).await?;

    Ok(PaginatedResponse::ok(items, total, page, page_size))
}

pub async fn list_inventory_logs_handler(
    Extension(pool): Extension<PgPool>,
    Query(filter): Query<InventoryFilter>,
) -> Result<Json<PaginatedResponse<InventoryLog>>, AppError> {
    let pagination = PaginationParams {
        page: filter.page,
        page_size: filter.page_size,
        sort_by: None,
        sort_order: None,
    };
    let page = pagination.page();
    let page_size = pagination.page_size();

    let (items, total) = InventoryQueryService::list_inventory_logs(&pool, &filter).await?;

    Ok(PaginatedResponse::ok(items, total, page, page_size))
}

pub async fn inventory_statistics_handler(
    Extension(pool): Extension<PgPool>,
) -> Result<Json<ApiResponse<crate::dto::inventory_dto::InventoryStatistics>>, AppError> {
    let stats = InventoryQueryService::inventory_statistics(&pool).await?;
    Ok(ApiResponse::ok(stats))
}

pub async fn trace_pipe_handler(
    Extension(pool): Extension<PgPool>,
    axum::extract::Path((pipe_type, pipe_id)): axum::extract::Path<(String, i64)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let result = TraceService::trace_pipe_lifecycle(&pool, &pipe_type, pipe_id).await?;
    Ok(ApiResponse::ok(result))
}

pub async fn trace_heat_handler(
    Extension(pool): Extension<PgPool>,
    Query(query): Query<HeatNumberQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    if query.heat_number.trim().is_empty() {
        return Err(AppError::Validation("Heat number is required".into()));
    }
    let results = TraceService::trace_by_heat_number(&pool, &query.heat_number).await?;
    Ok(ApiResponse::ok(serde_json::Value::Array(results)))
}

pub async fn trace_order_handler(
    Extension(pool): Extension<PgPool>,
    axum::extract::Path((order_type, order_id)): axum::extract::Path<(String, i64)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let result = TraceService::trace_by_order(&pool, &order_type, order_id).await?;
    Ok(ApiResponse::ok(result))
}
