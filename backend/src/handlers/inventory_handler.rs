// 库存管理入口：入库/出库/仓位/盘点/追溯
// 核心业务 —— 钢管按根管理，每根都有独立状态和位置

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
use crate::models::inventory::{
    InboundItem, InboundRecord, InventoryCheckItem, InventoryCheckRecord, InventoryLog, Location,
    OutboundItem, OutboundRecord,
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
// 入库单：关联采购收货、生产入库、退货入库、移库入库
// 创建后需要审批，审批通过才真正增加库存

pub async fn create_inbound_handler(
    Extension(pool): Extension<SqlitePool>,
    Json(req): Json<CreateInboundRecordRequest>,
) -> Result<Json<ApiResponse<InboundRecord>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
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
    Json(req): Json<ApproveRequest>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    InventoryService::approve_inbound(&pool, id).await?;
    Ok(ApiResponse::ok("Inbound approved".into()))
}

pub async fn reject_inbound_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
    Json(req): Json<RejectRequest>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
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
// 出库单：关联销售发货、报废出库、移库出库
// 出库前会检查 ATP（可用库存），不允许超量出库

pub async fn create_outbound_handler(
    Extension(pool): Extension<SqlitePool>,
    Json(req): Json<CreateOutboundRecordRequest>,
) -> Result<Json<ApiResponse<OutboundRecord>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
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
    Json(req): Json<ApproveRequest>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    InventoryService::approve_outbound(&pool, id).await?;
    Ok(ApiResponse::ok("Outbound approved".into()))
}

pub async fn reject_outbound_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
    Json(req): Json<RejectRequest>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
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
// 库存查询：按规格汇总库存量，支持按批次/炉号/仓位过滤
// 库存日志：记录每根管子的出入库流水

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
// 仓位管理：仓库物理位置的增删改查，active_only 过滤停用仓位

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
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
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
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
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
// 库存盘点：按仓位/规格创建盘点任务，逐项提交实盘结果

pub async fn create_check_handler(
    Extension(pool): Extension<SqlitePool>,
    Json(req): Json<CreateCheckRequest>,
) -> Result<Json<ApiResponse<InventoryCheckRecord>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
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
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let item = InventoryService::submit_check_item(&pool, check_id, item_id, &req).await?;
    Ok(ApiResponse::ok(item))
}

// ━━━ Trace Handlers ━━━
// 追溯查询：按管子编号/炉号/订单号追溯全生命周期

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

// ━━━ Statistics ━━━

pub async fn inventory_statistics_handler(
    Extension(pool): Extension<SqlitePool>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let stats = InventoryService::inventory_statistics(&pool).await?;
    Ok(ApiResponse::ok(stats))
}

// ━━━ Inbound / Outbound Items ━━━

pub async fn list_inbound_items_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<Vec<InboundItem>>>, AppError> {
    let items = InventoryService::list_inbound_items(&pool, id).await?;
    Ok(ApiResponse::ok(items))
}

pub async fn list_outbound_items_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<Vec<OutboundItem>>>, AppError> {
    let items = InventoryService::list_outbound_items(&pool, id).await?;
    Ok(ApiResponse::ok(items))
}

// ━━━ Complete Check ━━━

pub async fn complete_check_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let result = InventoryService::complete_check(&pool, id).await?;
    Ok(ApiResponse::ok(result))
}

// ━━━ Assign Location ━━━

pub async fn assign_location_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(location_id): Path<i64>,
    Json(req): Json<AssignLocationRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let result = InventoryService::assign_location(&pool, location_id, &req).await?;
    Ok(ApiResponse::ok(result))
}

// ━━━ Transfer Location ━━━

pub async fn transfer_location_handler(
    Extension(pool): Extension<SqlitePool>,
    Path((pipe_type, pipe_id)): Path<(String, i64)>,
    Json(req): Json<TransferLocationRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let result = InventoryService::transfer_location(&pool, &pipe_type, pipe_id, &req).await?;
    Ok(ApiResponse::ok(result))
}

// ━━━ Batch Inbound ━━━

pub async fn batch_create_inbound_handler(
    Extension(pool): Extension<SqlitePool>,
    Json(req): Json<BatchCreateInboundRequest>,
) -> Result<Json<ApiResponse<Vec<InboundRecord>>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let records = InventoryService::batch_create_inbound(&pool, &req).await?;
    Ok(ApiResponse::ok(records))
}
