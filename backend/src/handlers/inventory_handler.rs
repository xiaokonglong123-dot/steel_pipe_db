use axum::{
    extract::{Extension, Path, Query},
    Json,
};
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::dto::common::PaginationParams;
use crate::dto::inventory_dto::{
    ApproveRequest, AssignLocationRequest, BatchCreateInboundRequest, CreateCheckRequest,
    CreateInboundRecordRequest, CreateLocationRequest, CreateOutboundRecordRequest, InboundFilter,
    InventoryFilter, OutboundFilter, RejectRequest, SubmitCheckItemRequest,
    TransferLocationRequest, UpdateLocationRequest,
};
use validator::Validate;

use crate::error::AppError;
use crate::middleware::auth::AuthContext;
use crate::models::inventory::{
    InboundItem, InboundRecord, InventoryCheckItem, InventoryCheckRecord, InventoryLog, Location,
    OutboundItem, OutboundRecord,
};
use crate::response::{ApiResponse, PaginatedResponse};
use crate::services::check_service::CheckService;
use crate::services::inbound_service::InboundService;
use crate::services::inventory_query_service::InventoryQueryService;
use crate::services::location_service::LocationService;
use crate::services::outbound_service::OutboundService;
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

/// POST `/api/v1/inbound-records` — Create an inbound record
///
/// Creates a new inbound record (purchase, production, return, or transfer).
/// Validates the request body. Warehouse/admin role required.
pub async fn create_inbound_handler(
    Extension(pool): Extension<SqlitePool>,
    Json(req): Json<CreateInboundRecordRequest>,
) -> Result<Json<ApiResponse<InboundRecord>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let record = InboundService::create_inbound(&pool, &req).await?;
    Ok(ApiResponse::ok(record))
}

/// GET `/api/v1/inbound-records` — Paginated list of inbound records
///
/// Returns paginated inbound records, filterable by type, date range, status, etc.
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

    let (items, total) = InboundService::list_inbound_records(&pool, &filter).await?;

    Ok(PaginatedResponse::ok(items, total, page, page_size))
}

/// GET `/api/v1/inbound-records/{id}` — Get inbound record details
///
/// Returns the inbound record header plus its line items. Returns 404 if not found.
pub async fn get_inbound_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<crate::dto::inventory_dto::InboundRecordDetail>>, AppError> {
    let (record, items) = InboundService::get_inbound_record(&pool, id).await?;
    Ok(ApiResponse::ok(crate::dto::inventory_dto::InboundRecordDetail { record, items }))
}

/// PUT `/api/v1/inbound-records/{id}/approve` — Approve an inbound record
///
/// Approves an inbound record, updating stock quantities accordingly.
/// Warehouse/admin role required. Returns 404 if record not found.
pub async fn approve_inbound_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<ApproveRequest>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    InboundService::approve_inbound(&pool, id, auth.user_id, req.reason.as_deref()).await?;
    Ok(ApiResponse::ok("Inbound approved".into()))
}

/// PUT `/api/v1/inbound-records/{id}/reject` — Reject an inbound record
///
/// Rejects an inbound record with a reason. Returns 404 if record not found.
pub async fn reject_inbound_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
    Json(req): Json<RejectRequest>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    InboundService::reject_inbound(&pool, id, &req.reason).await?;
    Ok(ApiResponse::ok("Inbound rejected".into()))
}

/// DELETE `/api/v1/inbound-records/{id}` — Delete an inbound record
///
/// Soft-deletes an inbound record. Returns 404 if not found.
pub async fn delete_inbound_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    InboundService::delete_inbound(&pool, id).await?;
    Ok(ApiResponse::ok("Inbound record deleted".into()))
}

// ━━━ Outbound Handlers ━━━

/// POST `/api/v1/outbound-records` — Create an outbound record
///
/// Creates a new outbound record (sales, scrapped, or transfer).
/// Validates the request body. Warehouse/admin role required.
pub async fn create_outbound_handler(
    Extension(pool): Extension<SqlitePool>,
    Json(req): Json<CreateOutboundRecordRequest>,
) -> Result<Json<ApiResponse<OutboundRecord>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let record = OutboundService::create_outbound(&pool, &req).await?;
    Ok(ApiResponse::ok(record))
}

/// GET `/api/v1/outbound-records` — Paginated list of outbound records
///
/// Returns paginated outbound records, filterable by type, date range, status, etc.
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

    let (items, total) = OutboundService::list_outbound_records(&pool, &filter).await?;

    Ok(PaginatedResponse::ok(items, total, page, page_size))
}

/// GET `/api/v1/outbound-records/{id}` — Get outbound record details
///
/// Returns the outbound record header plus its line items. Returns 404 if not found.
pub async fn get_outbound_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<crate::dto::inventory_dto::OutboundRecordDetail>>, AppError> {
    let (record, items) = OutboundService::get_outbound_record(&pool, id).await?;
    Ok(ApiResponse::ok(crate::dto::inventory_dto::OutboundRecordDetail { record, items }))
}

/// PUT `/api/v1/outbound-records/{id}/approve` — Approve an outbound record
///
/// Approves an outbound record, deducting stock quantities accordingly.
/// Warehouse/admin role required. Returns 404 if record not found.
pub async fn approve_outbound_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<ApproveRequest>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    OutboundService::approve_outbound(&pool, id, auth.user_id, req.reason.as_deref()).await?;
    Ok(ApiResponse::ok("Outbound approved".into()))
}

/// PUT `/api/v1/outbound-records/{id}/reject` — Reject an outbound record
///
/// Rejects an outbound record with a reason. Returns 404 if record not found.
pub async fn reject_outbound_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
    Json(req): Json<RejectRequest>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    OutboundService::reject_outbound(&pool, id, &req.reason).await?;
    Ok(ApiResponse::ok("Outbound rejected".into()))
}

/// DELETE `/api/v1/outbound-records/{id}` — Delete an outbound record
///
/// Soft-deletes an outbound record. Returns 404 if not found.
pub async fn delete_outbound_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    OutboundService::delete_outbound(&pool, id).await?;
    Ok(ApiResponse::ok("Outbound record deleted".into()))
}

// ━━━ Inventory Handlers ━━━

/// GET `/api/v1/inventory` — Paginated stock list
///
/// Returns paginated current stock by pipe spec, filterable by pipe type, grade, etc.
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

/// GET `/api/v1/inventory/logs` — Paginated inventory change log
///
/// Returns paginated inventory audit trail, filterable by pipe, date range, etc.
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

// ━━━ Location Handlers ━━━

/// GET `/api/v1/locations` — Paginated list of locations
///
/// Returns paginated storage locations, with optional `active_only` filter.
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
    let (items, total) = LocationService::list_locations(&pool, &pagination, active_only).await?;

    Ok(PaginatedResponse::ok(items, total, page, page_size))
}

/// POST `/api/v1/locations` — Create a new location
///
/// Creates a new storage location with area, row, shelf identifiers.
/// Validates the request body. Warehouse/admin role required.
pub async fn create_location_handler(
    Extension(pool): Extension<SqlitePool>,
    Json(req): Json<CreateLocationRequest>,
) -> Result<Json<ApiResponse<Location>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let location = LocationService::create_location(&pool, &req).await?;
    Ok(ApiResponse::ok(location))
}

/// GET `/api/v1/locations/{id}` — Get location details
///
/// Returns a single storage location by ID. Returns 404 if not found.
pub async fn get_location_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<Location>>, AppError> {
    let location = LocationService::get_location(&pool, id).await?;
    Ok(ApiResponse::ok(location))
}

/// PUT `/api/v1/locations/{id}` — Update a location
///
/// Updates storage location fields (capacity, active status, etc.).
/// Validates the request body. Returns 404 if not found.
pub async fn update_location_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateLocationRequest>,
) -> Result<Json<ApiResponse<Location>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let location = LocationService::update_location(&pool, id, &req).await?;
    Ok(ApiResponse::ok(location))
}

/// DELETE `/api/v1/locations/{id}` — Delete a location
///
/// Soft-deletes a storage location. Returns 404 if not found.
pub async fn delete_location_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    LocationService::delete_location(&pool, id).await?;
    Ok(ApiResponse::ok("Location deleted".into()))
}

// ━━━ Check Handlers ━━━

/// POST `/api/v1/inventory/checks` — Create a check task
///
/// Creates a new inventory check record. Warehouse/admin role required.
/// Validates the request body.
pub async fn create_check_handler(
    Extension(pool): Extension<SqlitePool>,
    Json(req): Json<CreateCheckRequest>,
) -> Result<Json<ApiResponse<InventoryCheckRecord>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let record = CheckService::create_check(&pool, &req).await?;
    Ok(ApiResponse::ok(record))
}

/// GET `/api/v1/inventory/checks` — Paginated list of check tasks
///
/// Returns a paginated list of inventory check records.
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

    let (items, total) = CheckService::list_checks(&pool, &pagination).await?;

    Ok(PaginatedResponse::ok(items, total, page, page_size))
}

/// GET `/api/v1/inventory/checks/{id}` — Get check task details
///
/// Returns the check record plus its checked items. Returns 404 if not found.
pub async fn get_check_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<crate::dto::inventory_dto::CheckRecordDetail>>, AppError> {
    let (record, items) = CheckService::get_check_detail(&pool, id).await?;
    Ok(ApiResponse::ok(crate::dto::inventory_dto::CheckRecordDetail { record, items }))
}

/// POST `/api/v1/inventory/checks/{check_id}/items/{item_id}/submit` — Submit a check item
///
/// Submit the actual quantity counted for a specific check item.
/// Validates the request body. QC/admin role required.
pub async fn submit_check_item_handler(
    Extension(pool): Extension<SqlitePool>,
    Path((check_id, item_id)): Path<(i64, i64)>,
    Json(req): Json<SubmitCheckItemRequest>,
) -> Result<Json<ApiResponse<InventoryCheckItem>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let item = CheckService::submit_check_item(&pool, check_id, item_id, &req).await?;
    Ok(ApiResponse::ok(item))
}

// ━━━ Trace Handlers ━━━

/// GET `/api/v1/trace/pipe/{pipe_type}/{pipe_id}` — Trace the pipe's full lifecycle
///
/// Returns the complete lifecycle trace for a pipe (inbound → outbound → quality).
/// Accepts pipe_type (seamless/screen) and pipe_id.
pub async fn trace_pipe_handler(
    Extension(pool): Extension<SqlitePool>,
    Path((pipe_type, pipe_id)): Path<(String, i64)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let result = TraceService::trace_pipe_lifecycle(&pool, &pipe_type, pipe_id).await?;
    Ok(ApiResponse::ok(result))
}

/// GET `/api/v1/trace/heat` — Trace by heat number
///
/// Returns all lifecycle events for pipes matching the given heat number.
/// Requires `heat_number` query parameter. Returns 400 if empty.
pub async fn trace_heat_handler(
    Extension(pool): Extension<SqlitePool>,
    Query(query): Query<HeatNumberQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    if query.heat_number.trim().is_empty() {
        return Err(AppError::Validation("Heat number is required".into()));
    }
    let results = TraceService::trace_by_heat_number(&pool, &query.heat_number).await?;
    Ok(ApiResponse::ok(serde_json::Value::Array(results)))
}

/// GET `/api/v1/trace/order/{order_type}/{order_id}` — Trace by order ID
///
/// Returns inventory movements for all pipes associated with a given order.
/// Accepts order_type (purchase/sales) and order_id.
pub async fn trace_order_handler(
    Extension(pool): Extension<SqlitePool>,
    Path((order_type, order_id)): Path<(String, i64)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let result = TraceService::trace_by_order(&pool, &order_type, order_id).await?;
    Ok(ApiResponse::ok(result))
}

// ━━━ Statistics ━━━

/// GET `/api/v1/inventory/statistics` — Inventory statistics overview
///
/// Returns aggregated inventory statistics (total quantity, value, counts by pipe type/grade, etc.).
pub async fn inventory_statistics_handler(
    Extension(pool): Extension<SqlitePool>,
) -> Result<Json<ApiResponse<crate::dto::inventory_dto::InventoryStatistics>>, AppError> {
    let stats = InventoryQueryService::inventory_statistics(&pool).await?;
    Ok(ApiResponse::ok(stats))
}

// ━━━ Inbound / Outbound Items ━━━

/// GET `/api/v1/inbound-records/{id}/items` — Inbound record line items
///
/// Returns all line items for a specific inbound record.
pub async fn list_inbound_items_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<Vec<InboundItem>>>, AppError> {
    let items = InboundService::list_inbound_items(&pool, id).await?;
    Ok(ApiResponse::ok(items))
}

/// GET `/api/v1/outbound-records/{id}/items` — Outbound record line items
///
/// Returns all line items for a specific outbound record.
pub async fn list_outbound_items_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<Vec<OutboundItem>>>, AppError> {
    let items = OutboundService::list_outbound_items(&pool, id).await?;
    Ok(ApiResponse::ok(items))
}

// ━━━ Complete Check ━━━

/// PUT `/api/v1/inventory/checks/{id}/complete` — Complete a check
///
/// Marks a check as completed, calculating variance from expected stock.
/// Returns the check summary with discrepancy details.
pub async fn complete_check_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let result = CheckService::complete_check(&pool, id).await?;
    Ok(ApiResponse::ok(result))
}

// ━━━ Assign Location ━━━

/// PUT `/api/v1/locations/{location_id}/assign` — Assign pipes to a location
///
/// Assigns pipes to a storage location. Validates the request body.
/// Warehouse/admin role required. Returns 404 if location not found.
pub async fn assign_location_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(location_id): Path<i64>,
    Json(req): Json<AssignLocationRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let result = LocationService::assign_location(&pool, location_id, &req).await?;
    Ok(ApiResponse::ok(result))
}

// ━━━ Transfer Location ━━━

/// PUT `/api/v1/locations/transfer` — Transfer pipe location
///
/// Transfers a pipe from its current location to a new location.
/// Validates the request body. Returns 404 if pipe or location not found.
pub async fn transfer_location_handler(
    Extension(pool): Extension<SqlitePool>,
    Path((pipe_type, pipe_id)): Path<(String, i64)>,
    Json(req): Json<TransferLocationRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let result = LocationService::transfer_location(&pool, &pipe_type, pipe_id, &req).await?;
    Ok(ApiResponse::ok(result))
}

// ━━━ Batch Inbound ━━━

/// POST `/api/v1/inbound-records/batch` — Batch create inbound records
///
/// Creates multiple inbound records in a single request for bulk operations.
/// Validates the request body. Warehouse/admin role required.
pub async fn batch_create_inbound_handler(
    Extension(pool): Extension<SqlitePool>,
    Json(req): Json<BatchCreateInboundRequest>,
) -> Result<Json<ApiResponse<Vec<InboundRecord>>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let records = InboundService::batch_create_inbound(&pool, &req).await?;
    Ok(ApiResponse::ok(records))
}
