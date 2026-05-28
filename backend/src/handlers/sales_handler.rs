use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sqlx::SqlitePool;

use validator::Validate;

use crate::dto::common::PaginationParams;
use crate::dto::sales_dto::{
    ApproveOrderRequest, CreateSalesOrderRequest, LinkOutboundRequest, RejectOrderRequest,
    SalesOrderDetailResponse, SalesOrderFilterParams, SalesOrderStatusTransitionRequest,
    UpdateSalesItemRequest, UpdateSalesOrderRequest,
};
use crate::error::AppError;
use crate::models::sales_order::SalesOrder;
use crate::response::{ApiResponse, PaginatedResponse};
use crate::services::purchase_sales_service::PurchaseSalesService;

/// GET `/api/v1/sales-orders` — Paginated list of sales orders
///
/// Supports filtering by status, customer, date range, etc.
/// Returns paginated sales order results.
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

/// POST `/api/v1/sales-orders` — Create a sales order
///
/// Creates a new sales order with line items.
/// Validates request body. Returns the created order.
pub async fn create_sales_order_handler(
    Extension(pool): Extension<SqlitePool>,
    Json(req): Json<CreateSalesOrderRequest>,
) -> Result<axum::response::Response, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let order = PurchaseSalesService::create_sales_order(&pool, &req).await?;
    Ok(ApiResponse::created(order))
}

/// GET `/api/v1/sales-orders/{id}` — Get sales order details
///
/// Returns the sales order header plus its line items in a standard ApiResponse envelope.
/// Returns 404 if not found.
pub async fn get_sales_order_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<SalesOrderDetailResponse>>, AppError> {
    let (order, items) = PurchaseSalesService::get_sales_order(&pool, id).await?;
    Ok(ApiResponse::ok(SalesOrderDetailResponse { order, items }))
}

/// PUT `/api/v1/sales-orders/{id}` — Update a sales order
///
/// Updates an existing sales order (items, dates, terms, etc.).
/// Validates the request body. Returns 404 if not found.
pub async fn update_sales_order_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateSalesOrderRequest>,
) -> Result<Json<ApiResponse<SalesOrder>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let order = PurchaseSalesService::update_sales_order(&pool, id, &req).await?;
    Ok(ApiResponse::ok(order))
}

/// DELETE `/api/v1/sales-orders/{id}` — Delete a sales order
///
/// Soft-deletes a sales order. Returns 404 if not found.
pub async fn delete_sales_order_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<axum::response::Response, AppError> {
    PurchaseSalesService::delete_sales_order(&pool, id).await?;
    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

/// PUT `/api/v1/sales-orders/{id}/status` — Transition sales order status
///
/// Transitions the sales order status (e.g., confirmed, shipped, completed).
/// Validates the status transition. Returns 400 on invalid transition.
pub async fn transition_sales_order_status_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
    Json(req): Json<SalesOrderStatusTransitionRequest>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    PurchaseSalesService::transition_sales_status(&pool, id, &req).await?;
    Ok(ApiResponse::ok(format!(
        "Sales order status changed to '{}'",
        req.status
    )))
}

/// PUT `/api/v1/sales-orders/{order_id}/items/{item_id}` — Update a SO line item
///
/// Updates a specific line item within a sales order (quantity, price, spec, etc.).
/// Validates request body. Returns 404 if order or item not found.
pub async fn update_sales_item_handler(
    Extension(pool): Extension<SqlitePool>,
    Path((order_id, item_id)): Path<(i64, i64)>,
    Json(req): Json<UpdateSalesItemRequest>,
) -> Result<Json<ApiResponse<SalesOrder>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let (order, _item) =
        PurchaseSalesService::update_sales_item(&pool, order_id, item_id, &req).await?;
    Ok(ApiResponse::ok(order))
}

/// DELETE `/api/v1/sales-orders/{order_id}/items/{item_id}` — Delete a SO line item
///
/// Removes a line item from a sales order. Returns 404 if order or item not found.
pub async fn delete_sales_item_handler(
    Extension(pool): Extension<SqlitePool>,
    Path((order_id, item_id)): Path<(i64, i64)>,
) -> Result<axum::response::Response, AppError> {
    PurchaseSalesService::delete_sales_item(&pool, order_id, item_id).await?;
    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

/// PUT `/api/v1/sales-orders/{id}/approve` — Approve a sales order
///
/// Approves a sales order, typically after ATP check passes.
/// Admin/sales role required. Returns 404 if not found.
pub async fn approve_sales_order_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
    Json(dto): Json<ApproveOrderRequest>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    PurchaseSalesService::approve_sales_order(&pool, id, &dto).await?;
    Ok(ApiResponse::ok("Sales order approved".into()))
}

/// PUT `/api/v1/sales-orders/{id}/reject` — Reject a sales order
///
/// Rejects a sales order with a reason. Returns 404 if not found.
pub async fn reject_sales_order_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
    Json(dto): Json<RejectOrderRequest>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    PurchaseSalesService::reject_sales_order(&pool, id, &dto).await?;
    Ok(ApiResponse::ok("Sales order rejected".into()))
}

/// POST `/api/v1/sales-orders/{order_id}/link-outbound` — Link outbound record to SO
///
/// Links an existing outbound record to a sales order for traceability.
/// Returns 404 if order or outbound record not found.
pub async fn link_outbound_to_order_handler(
    Extension(pool): Extension<SqlitePool>,
    Path((order_id, outbound_id)): Path<(i64, i64)>,
) -> Result<axum::response::Response, AppError> {
    PurchaseSalesService::link_outbound_to_order(&pool, order_id, outbound_id).await?;
    Ok((StatusCode::NO_CONTENT, ()).into_response())
}
