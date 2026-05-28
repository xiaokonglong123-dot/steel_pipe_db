use axum::{
    extract::{Extension, Path, Query},
    Json,
};
use sqlx::SqlitePool;

use validator::Validate;

use crate::dto::common::PaginationParams;
use crate::dto::purchase_dto::{
    ApproveOrderRequest, CreatePurchaseOrderRequest, LinkInboundRequest,
    PurchaseOrderDetailResponse, PurchaseOrderFilterParams,
    PurchaseOrderStatusTransitionRequest, RejectOrderRequest, UpdatePurchaseItemRequest,
    UpdatePurchaseOrderRequest,
};
use crate::error::AppError;
use crate::models::purchase_order::PurchaseOrder;
use crate::response::{ApiResponse, PaginatedResponse};
use crate::services::purchase_sales_service::PurchaseSalesService;

/// GET `/api/v1/purchase-orders` — Paginated list of purchase orders
///
/// Returns paginated purchase orders, filterable by status, supplier, date range, etc.
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

/// POST `/api/v1/purchase-orders` — Create a purchase order
///
/// Creates a new purchase order with supplier, items, and delivery info.
/// Validates the request body. Admin/procurement role required.
pub async fn create_purchase_order_handler(
    Extension(pool): Extension<SqlitePool>,
    Json(req): Json<CreatePurchaseOrderRequest>,
) -> Result<Json<ApiResponse<PurchaseOrder>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let order = PurchaseSalesService::create_purchase_order(&pool, &req).await?;
    Ok(ApiResponse::ok(order))
}

/// GET `/api/v1/purchase-orders/{id}` — Get purchase order details
///
/// Returns the purchase order header plus its line items in a standard ApiResponse envelope.
/// Returns 404 if not found.
pub async fn get_purchase_order_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<PurchaseOrderDetailResponse>>, AppError> {
    let (order, items) = PurchaseSalesService::get_purchase_order(&pool, id).await?;
    Ok(ApiResponse::ok(PurchaseOrderDetailResponse { order, items }))
}

/// PUT `/api/v1/purchase-orders/{id}` — Update a purchase order
///
/// Updates an existing purchase order (items, dates, terms, etc.).
/// Validates the request body. Returns 404 if not found.
pub async fn update_purchase_order_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
    Json(req): Json<UpdatePurchaseOrderRequest>,
) -> Result<Json<ApiResponse<PurchaseOrder>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let order = PurchaseSalesService::update_purchase_order(&pool, id, &req).await?;
    Ok(ApiResponse::ok(order))
}

/// DELETE `/api/v1/purchase-orders/{id}` — Delete a purchase order
///
/// Soft-deletes a purchase order. Returns 404 if not found.
pub async fn delete_purchase_order_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    PurchaseSalesService::delete_purchase_order(&pool, id).await?;
    Ok(ApiResponse::ok("Purchase order deleted successfully".into()))
}

/// PUT `/api/v1/purchase-orders/{id}/status` — Transition purchase order status
///
/// Transitions the purchase order to a new status (e.g., confirmed, received, closed).
/// Validates the status transition request. Returns 400 on invalid transition.
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

/// PUT `/api/v1/purchase-orders/{order_id}/items/{item_id}` — Update a PO line item
///
/// Updates a specific line item within a purchase order (quantity, price, spec, etc.).
/// Validates the request body. Returns 404 if order or item not found.
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

/// DELETE `/api/v1/purchase-orders/{order_id}/items/{item_id}` — Delete a PO line item
///
/// Removes a line item from a purchase order. Returns 404 if order or item not found.
pub async fn delete_purchase_item_handler(
    Extension(pool): Extension<SqlitePool>,
    Path((order_id, item_id)): Path<(i64, i64)>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    PurchaseSalesService::delete_purchase_item(&pool, order_id, item_id).await?;
    Ok(ApiResponse::ok("Purchase order item deleted".into()))
}

/// PUT `/api/v1/purchase-orders/{id}/approve` — Approve a purchase order
///
/// Approves a purchase order, moving it to approved status.
/// Admin/procurement role required. Returns 404 if not found.
pub async fn approve_purchase_order_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
    Json(dto): Json<ApproveOrderRequest>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    PurchaseSalesService::approve_purchase_order(&pool, id, &dto).await?;
    Ok(ApiResponse::ok("Purchase order approved".into()))
}

/// PUT `/api/v1/purchase-orders/{id}/reject` — Reject a purchase order
///
/// Rejects a purchase order with a reason. Returns 404 if not found.
pub async fn reject_purchase_order_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
    Json(dto): Json<RejectOrderRequest>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    PurchaseSalesService::reject_purchase_order(&pool, id, &dto).await?;
    Ok(ApiResponse::ok("Purchase order rejected".into()))
}

/// POST `/api/v1/purchase-orders/{order_id}/link-inbound` — Link inbound record to PO
///
/// Links an existing inbound record to a purchase order for traceability.
/// Returns 404 if order or inbound record not found.
pub async fn link_inbound_to_order_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(order_id): Path<i64>,
    Json(dto): Json<LinkInboundRequest>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    PurchaseSalesService::link_inbound_to_order(&pool, order_id, dto.inbound_record_id).await?;
    Ok(ApiResponse::ok("Inbound record linked to purchase order".into()))
}
