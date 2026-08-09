use axum::{extract::{Extension, Path, Query}, Json};
use sqlx::SqlitePool;

use crate::dto::common::PaginationParams;
use crate::dto::inventory_dto::InventoryFilter;
use crate::error::AppError;
use crate::models::inventory::InventoryLog;
use crate::response::{ApiResponse, PaginatedResponse};
use crate::inventory::inventory_query_service::InventoryQueryService;
use crate::inventory::trace_service::TraceService;

pub async fn list_inventory_handler(
    Extension(pool): Extension<SqlitePool>,
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
    Extension(pool): Extension<SqlitePool>,
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
    Extension(pool): Extension<SqlitePool>,
) -> Result<Json<ApiResponse<crate::dto::inventory_dto::InventoryStatistics>>, AppError> {
    let stats = InventoryQueryService::inventory_statistics(&pool).await?;
    Ok(ApiResponse::ok(stats))
}

/// GET /api/v1/trace/items/{item_id} — full lifecycle trace for one item.
pub async fn trace_item_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(item_id): Path<i64>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let result = TraceService::trace_item_lifecycle(&pool, item_id).await?;
    Ok(ApiResponse::ok(result))
}

/// GET /api/v1/trace/order/{order_type}/{order_id} — inbound/outbound records
/// linked to a purchase/sales order.
pub async fn trace_order_handler(
    Extension(pool): Extension<SqlitePool>,
    axum::extract::Path((order_type, order_id)): axum::extract::Path<(String, i64)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let result = TraceService::trace_by_order(&pool, &order_type, order_id).await?;
    Ok(ApiResponse::ok(result))
}
