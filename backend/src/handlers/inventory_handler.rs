use axum::{
    extract::{Extension, Path, Query},
    Json,
};
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::dto::common::PaginationParams;
use crate::dto::inventory_dto::{
    ApproveRequest, CreateCheckRequest, CreateInboundRecordRequest, CreateLocationRequest,
    CreateOutboundRecordRequest, InboundFilter, InventoryFilter, OutboundFilter, RejectRequest,
    SubmitCheckItemRequest, UpdateLocationRequest,
};
use crate::error::AppError;
use crate::models::inventory::{
    InboundRecord, InventoryCheckItem, InventoryCheckRecord, InventoryLog, Location,
    OutboundRecord,
};
use crate::response::{ApiResponse, PaginatedResponse};
use crate::services::inventory_service::InventoryService;
use crate::services::trace_service::TraceService;

#[derive(Deserialize)]
pub struct LocationListQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub active_only: Option<bool>,
}

#[derive(Deserialize)]
pub struct HeatNumberQuery {
    pub heat_number: String,
}

#[derive(Deserialize)]
pub struct CheckListQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

// ━━━ Inbound Handlers ━━━

pub async fn create_inbound_handler(
    Extension(pool): Extension<SqlitePool>,
    Json(req): Json<CreateInboundRecordRequest>,
) -> Result<Json<ApiResponse<InboundRecord>>, AppError> {
    let record = InventoryService::create_inbound(&pool, &req).await?;
    Ok(ApiResponse::ok(record))
}

pub async fn list_inbound_handler(
    Extension(pool): Extension<SqlitePool>,
    Query(filter): Query<InboundFilter>,
) -> Result<Json<PaginatedResponse<InboundRecord>>, AppError> {
    let pagination = PaginationParams {
        page: filter.page,
        page_size: filter.page_size,
        sort_by: filter.sort_by.clone(),
        sort_order: filter.sort_order.clone(),
    };
    let page = pagination.page();
    let page_size = pagination.page_size();

    let (items, total) = InventoryService::list_inbound_records(&pool, &filter).await?;

    Ok(PaginatedResponse::ok(items, total, page, page_size))
}

pub async fn get_inbound_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, AppError> {
    let (record, items) = InventoryService::get_inbound_record(&pool, id).await?;
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "record": record,
            "items": items,
        }
    })))
}

pub async fn approve_inbound_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
    Json(_req): Json<ApproveRequest>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    InventoryService::approve_inbound(&pool, id).await?;
    Ok(ApiResponse::ok("Inbound approved".into()))
}

pub async fn reject_inbound_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
    Json(req): Json<RejectRequest>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    if req.reason.trim().is_empty() {
        return Err(AppError::Validation("Rejection reason is required".into()));
    }
    InventoryService::reject_inbound(&pool, id, &req.reason).await?;
    Ok(ApiResponse::ok("Inbound rejected".into()))
}

pub async fn delete_inbound_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    InventoryService::delete_inbound(&pool, id).await?;
    Ok(ApiResponse::ok("Inbound record deleted".into()))
}

// ━━━ Outbound Handlers ━━━

pub async fn create_outbound_handler(
    Extension(pool): Extension<SqlitePool>,
    Json(req): Json<CreateOutboundRecordRequest>,
) -> Result<Json<ApiResponse<OutboundRecord>>, AppError> {
    let record = InventoryService::create_outbound(&pool, &req).await?;
    Ok(ApiResponse::ok(record))
}

pub async fn list_outbound_handler(
    Extension(pool): Extension<SqlitePool>,
    Query(filter): Query<OutboundFilter>,
) -> Result<Json<PaginatedResponse<OutboundRecord>>, AppError> {
    let pagination = PaginationParams {
        page: filter.page,
        page_size: filter.page_size,
        sort_by: filter.sort_by.clone(),
        sort_order: filter.sort_order.clone(),
    };
    let page = pagination.page();
    let page_size = pagination.page_size();

    let (items, total) = InventoryService::list_outbound_records(&pool, &filter).await?;

    Ok(PaginatedResponse::ok(items, total, page, page_size))
}

pub async fn get_outbound_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, AppError> {
    let (record, items) = InventoryService::get_outbound_record(&pool, id).await?;
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "record": record,
            "items": items,
        }
    })))
}

pub async fn approve_outbound_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
    Json(_req): Json<ApproveRequest>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    InventoryService::approve_outbound(&pool, id).await?;
    Ok(ApiResponse::ok("Outbound approved".into()))
}

pub async fn reject_outbound_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
    Json(req): Json<RejectRequest>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    if req.reason.trim().is_empty() {
        return Err(AppError::Validation("Rejection reason is required".into()));
    }
    InventoryService::reject_outbound(&pool, id, &req.reason).await?;
    Ok(ApiResponse::ok("Outbound rejected".into()))
}

pub async fn delete_outbound_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    InventoryService::delete_outbound(&pool, id).await?;
    Ok(ApiResponse::ok("Outbound record deleted".into()))
}

// ━━━ Inventory Handlers ━━━

pub async fn list_inventory_handler(
    Extension(pool): Extension<SqlitePool>,
    Query(filter): Query<InventoryFilter>,
) -> Result<Json<PaginatedResponse<serde_json::Value>>, AppError> {
    let pagination = PaginationParams {
        page: filter.page,
        page_size: filter.page_size,
        sort_by: None,
        sort_order: None,
    };
    let page = pagination.page();
    let page_size = pagination.page_size();

    let (items, total) = InventoryService::list_inventory(&pool, &filter).await?;

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

    let (items, total) = InventoryService::list_inventory_logs(&pool, &filter).await?;

    Ok(PaginatedResponse::ok(items, total, page, page_size))
}

// ━━━ Location Handlers ━━━

pub async fn list_locations_handler(
    Extension(pool): Extension<SqlitePool>,
    Query(query): Query<LocationListQuery>,
) -> Result<Json<PaginatedResponse<Location>>, AppError> {
    let pagination = PaginationParams {
        page: query.page,
        page_size: query.page_size,
        sort_by: None,
        sort_order: None,
    };
    let page = pagination.page();
    let page_size = pagination.page_size();

    let active_only = query.active_only.unwrap_or(false);
    let (items, total) = InventoryService::list_locations(&pool, &pagination, active_only).await?;

    Ok(PaginatedResponse::ok(items, total, page, page_size))
}

pub async fn create_location_handler(
    Extension(pool): Extension<SqlitePool>,
    Json(req): Json<CreateLocationRequest>,
) -> Result<Json<ApiResponse<Location>>, AppError> {
    let location = InventoryService::create_location(&pool, &req).await?;
    Ok(ApiResponse::ok(location))
}

pub async fn get_location_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<Location>>, AppError> {
    let location = InventoryService::get_location(&pool, id).await?;
    Ok(ApiResponse::ok(location))
}

pub async fn update_location_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateLocationRequest>,
) -> Result<Json<ApiResponse<Location>>, AppError> {
    let location = InventoryService::update_location(&pool, id, &req).await?;
    Ok(ApiResponse::ok(location))
}

pub async fn delete_location_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    InventoryService::delete_location(&pool, id).await?;
    Ok(ApiResponse::ok("Location deleted".into()))
}

// ━━━ Check Handlers ━━━

pub async fn create_check_handler(
    Extension(pool): Extension<SqlitePool>,
    Json(req): Json<CreateCheckRequest>,
) -> Result<Json<ApiResponse<InventoryCheckRecord>>, AppError> {
    let record = InventoryService::create_check(&pool, &req).await?;
    Ok(ApiResponse::ok(record))
}

pub async fn list_checks_handler(
    Extension(pool): Extension<SqlitePool>,
    Query(query): Query<CheckListQuery>,
) -> Result<Json<PaginatedResponse<InventoryCheckRecord>>, AppError> {
    let pagination = PaginationParams {
        page: query.page,
        page_size: query.page_size,
        sort_by: None,
        sort_order: None,
    };
    let page = pagination.page();
    let page_size = pagination.page_size();

    let (items, total) = InventoryService::list_checks(&pool, &pagination).await?;

    Ok(PaginatedResponse::ok(items, total, page, page_size))
}

pub async fn get_check_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, AppError> {
    let (record, items) = InventoryService::get_check_detail(&pool, id).await?;
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "record": record,
            "items": items,
        }
    })))
}

pub async fn submit_check_item_handler(
    Extension(pool): Extension<SqlitePool>,
    Path((check_id, item_id)): Path<(i64, i64)>,
    Json(req): Json<SubmitCheckItemRequest>,
) -> Result<Json<ApiResponse<InventoryCheckItem>>, AppError> {
    let item = InventoryService::submit_check_item(&pool, check_id, item_id, &req).await?;
    Ok(ApiResponse::ok(item))
}

// ━━━ Trace Handlers ━━━

pub async fn trace_pipe_handler(
    Extension(pool): Extension<SqlitePool>,
    Path((pipe_type, pipe_id)): Path<(String, i64)>,
) -> Result<Json<serde_json::Value>, AppError> {
    let result = TraceService::trace_pipe_lifecycle(&pool, &pipe_type, pipe_id).await?;
    Ok(Json(serde_json::json!({ "success": true, "data": result })))
}

pub async fn trace_heat_handler(
    Extension(pool): Extension<SqlitePool>,
    Query(query): Query<HeatNumberQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    if query.heat_number.trim().is_empty() {
        return Err(AppError::Validation("Heat number is required".into()));
    }
    let results = TraceService::trace_by_heat_number(&pool, &query.heat_number).await?;
    Ok(Json(serde_json::json!({ "success": true, "data": results })))
}

pub async fn trace_order_handler(
    Extension(pool): Extension<SqlitePool>,
    Path((order_type, order_id)): Path<(String, i64)>,
) -> Result<Json<serde_json::Value>, AppError> {
    let result = TraceService::trace_by_order(&pool, &order_type, order_id).await?;
    Ok(Json(serde_json::json!({ "success": true, "data": result })))
}
