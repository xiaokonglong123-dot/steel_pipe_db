use axum::{
    extract::{Extension, Path, Query},
    Json,
};
use serde_json;
use sqlx::SqlitePool;

use crate::dto::common::PaginationParams;
use crate::dto::sales_dto::{
    CreateSalesOrderRequest, SalesOrderFilterParams, SalesOrderStatusTransitionRequest,
    UpdateSalesItemRequest, UpdateSalesOrderRequest,
};
use crate::error::AppError;
use crate::models::sales_order::SalesOrder;
use crate::response::{ApiResponse, PaginatedResponse};
use crate::services::purchase_sales_service::PurchaseSalesService;

pub async fn list_sales_orders_handler(
    Extension(pool): Extension<SqlitePool>,
    Query(filter): Query<SalesOrderFilterParams>,
) -> Result<Json<PaginatedResponse<SalesOrder>>, AppError> {
    let pagination = PaginationParams {
        page: filter.page,
        page_size: filter.page_size,
        sort_by: filter.sort_by.clone(),
        sort_order: filter.sort_order.clone(),
    };
    let page = pagination.page();
    let page_size = pagination.page_size();

    let (items, total) =
        PurchaseSalesService::list_sales_orders(&pool, &filter, &pagination).await?;

    Ok(PaginatedResponse::ok(items, total, page, page_size))
}

pub async fn create_sales_order_handler(
    Extension(pool): Extension<SqlitePool>,
    Json(req): Json<CreateSalesOrderRequest>,
) -> Result<Json<ApiResponse<SalesOrder>>, AppError> {
    let order = PurchaseSalesService::create_sales_order(&pool, &req).await?;
    Ok(ApiResponse::ok(order))
}

pub async fn get_sales_order_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, AppError> {
    let (order, items) = PurchaseSalesService::get_sales_order(&pool, id).await?;
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "order": order,
            "items": items,
        }
    })))
}

pub async fn update_sales_order_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateSalesOrderRequest>,
) -> Result<Json<ApiResponse<SalesOrder>>, AppError> {
    let order = PurchaseSalesService::update_sales_order(&pool, id, &req).await?;
    Ok(ApiResponse::ok(order))
}

pub async fn delete_sales_order_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    PurchaseSalesService::delete_sales_order(&pool, id).await?;
    Ok(ApiResponse::ok("Sales order deleted successfully".into()))
}

pub async fn transition_sales_order_status_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
    Json(req): Json<SalesOrderStatusTransitionRequest>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    PurchaseSalesService::transition_sales_status(&pool, id, &req).await?;
    Ok(ApiResponse::ok(format!(
        "Sales order status changed to '{}'",
        req.status
    )))
}

pub async fn update_sales_item_handler(
    Extension(pool): Extension<SqlitePool>,
    Path((order_id, item_id)): Path<(i64, i64)>,
    Json(req): Json<UpdateSalesItemRequest>,
) -> Result<Json<ApiResponse<SalesOrder>>, AppError> {
    let (order, _item) =
        PurchaseSalesService::update_sales_item(&pool, order_id, item_id, &req).await?;
    Ok(ApiResponse::ok(order))
}

pub async fn delete_sales_item_handler(
    Extension(pool): Extension<SqlitePool>,
    Path((order_id, item_id)): Path<(i64, i64)>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    PurchaseSalesService::delete_sales_item(&pool, order_id, item_id).await?;
    Ok(ApiResponse::ok("Sales order item deleted".into()))
}
