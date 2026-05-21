use axum::{
    extract::{Extension, Path, Query},
    Json,
};
use serde_json;
use sqlx::SqlitePool;

use validator::Validate;

use crate::dto::common::PaginationParams;
use crate::dto::purchase_dto::{
    CreatePurchaseOrderRequest, PurchaseOrderFilterParams, PurchaseOrderStatusTransitionRequest,
    UpdatePurchaseItemRequest, UpdatePurchaseOrderRequest,
};
use crate::error::AppError;
use crate::models::purchase_order::PurchaseOrder;
use crate::response::{ApiResponse, PaginatedResponse};
use crate::services::purchase_sales_service::PurchaseSalesService;

pub async fn list_purchase_orders_handler(
    Extension(pool): Extension<SqlitePool>,
    Query(filter): Query<PurchaseOrderFilterParams>,
) -> Result<Json<PaginatedResponse<PurchaseOrder>>, AppError> {
    let pagination = PaginationParams {
        page: filter.page,
        page_size: filter.page_size,
        sort_by: filter.sort_by.clone(),
        sort_order: filter.sort_order.clone(),
    };
    let page = pagination.page();
    let page_size = pagination.page_size();

    let (items, total) =
        PurchaseSalesService::list_purchase_orders(&pool, &filter, &pagination).await?;

    Ok(PaginatedResponse::ok(items, total, page, page_size))
}

pub async fn create_purchase_order_handler(
    Extension(pool): Extension<SqlitePool>,
    Json(req): Json<CreatePurchaseOrderRequest>,
) -> Result<Json<ApiResponse<PurchaseOrder>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let order = PurchaseSalesService::create_purchase_order(&pool, &req).await?;
    Ok(ApiResponse::ok(order))
}

pub async fn get_purchase_order_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, AppError> {
    let (order, items) = PurchaseSalesService::get_purchase_order(&pool, id).await?;
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "order": order,
            "items": items,
        }
    })))
}

pub async fn update_purchase_order_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
    Json(req): Json<UpdatePurchaseOrderRequest>,
) -> Result<Json<ApiResponse<PurchaseOrder>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let order = PurchaseSalesService::update_purchase_order(&pool, id, &req).await?;
    Ok(ApiResponse::ok(order))
}

pub async fn delete_purchase_order_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    PurchaseSalesService::delete_purchase_order(&pool, id).await?;
    Ok(ApiResponse::ok("Purchase order deleted successfully".into()))
}

pub async fn transition_purchase_order_status_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
    Json(req): Json<PurchaseOrderStatusTransitionRequest>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    PurchaseSalesService::transition_purchase_status(&pool, id, &req).await?;
    Ok(ApiResponse::ok(format!(
        "Purchase order status changed to '{}'",
        req.status
    )))
}

pub async fn update_purchase_item_handler(
    Extension(pool): Extension<SqlitePool>,
    Path((order_id, item_id)): Path<(i64, i64)>,
    Json(req): Json<UpdatePurchaseItemRequest>,
) -> Result<Json<ApiResponse<PurchaseOrder>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let (order, _item) =
        PurchaseSalesService::update_purchase_item(&pool, order_id, item_id, &req).await?;
    Ok(ApiResponse::ok(order))
}

pub async fn delete_purchase_item_handler(
    Extension(pool): Extension<SqlitePool>,
    Path((order_id, item_id)): Path<(i64, i64)>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    PurchaseSalesService::delete_purchase_item(&pool, order_id, item_id).await?;
    Ok(ApiResponse::ok("Purchase order item deleted".into()))
}
