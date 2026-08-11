//! Inventory HTTP handlers — 入库/出库/库存/日志
//!
//! 对齐 http/catalog.rs：`Extension(pool): Extension<SqlitePool>`，DTO 解析+校验后调 service。
//! POST /inbounds / POST /outbounds / POST /{id}/post 走 `stock.write`；
//! GET 类走 `stock.read`（路由在 http/mod.rs 分 sub-router 绑定）。

use axum::extract::{Extension, Path, Query};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::repos::inventory_repo::{
    InboundOrderFilter, InventoryLogFilter, OutboundOrderFilter, StockFilter,
};
use crate::response::{ApiResponse, PaginatedResponse};
use crate::services::inventory_service;

// —— DTOs ——

#[derive(Deserialize)]
pub struct InboundItemDto {
    pub item_id: i64,
    pub location_id: i64,
    pub quantity: f64,
    pub notes: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateInboundRequest {
    pub inbound_type: String,
    #[serde(default)]
    pub order_id: Option<i64>,
    #[serde(default)]
    pub supplier_id: Option<i64>,
    #[serde(default)]
    pub notes: Option<String>,
    pub items: Vec<InboundItemDto>,
}

#[derive(Deserialize)]
pub struct OutboundItemDto {
    pub item_id: i64,
    pub location_id: i64,
    pub quantity: f64,
    pub notes: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateOutboundRequest {
    pub outbound_type: String,
    #[serde(default)]
    pub order_id: Option<i64>,
    #[serde(default)]
    pub customer_id: Option<i64>,
    #[serde(default)]
    pub notes: Option<String>,
    pub items: Vec<OutboundItemDto>,
}

#[derive(Deserialize)]
pub struct InboundListQuery {
    pub status: Option<String>,
    #[serde(rename = "type")]
    pub inbound_type: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Deserialize)]
pub struct OutboundListQuery {
    pub status: Option<String>,
    #[serde(rename = "type")]
    pub outbound_type: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Deserialize)]
pub struct StockListQuery {
    pub item_id: Option<i64>,
    pub location_id: Option<i64>,
    pub warehouse_id: Option<i64>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Deserialize)]
pub struct LogListQuery {
    pub item_id: Option<i64>,
    pub location_id: Option<i64>,
    pub change_type: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Deserialize)]
pub struct CreateCheckRequest {
    pub location_id: i64,
    pub scope: String,
}

#[derive(Deserialize)]
pub struct RecordActualQtyRequest {
    pub detail_id: i64,
    pub actual_qty: f64,
}

// —— Inbound handlers ——

pub async fn create_inbound(
    Extension(pool): Extension<SqlitePool>,
    user: AuthUser,
    Json(req): Json<CreateInboundRequest>,
) -> Result<impl IntoResponse, AppError> {
    let dto = inventory_service::CreateInboundRequest {
        inbound_type: req.inbound_type,
        order_id: req.order_id,
        supplier_id: req.supplier_id,
        notes: req.notes,
        items: req
            .items
            .into_iter()
            .map(|i| inventory_service::CreateInboundItemInput {
                item_id: i.item_id,
                location_id: i.location_id,
                quantity: i.quantity,
                notes: i.notes,
            })
            .collect(),
    };
    let order = inventory_service::create_inbound(&pool, &dto, &user).await?;
    Ok((StatusCode::CREATED, Json(ApiResponse::ok(order))))
}

pub async fn list_inbounds(
    Extension(pool): Extension<SqlitePool>,
    Query(q): Query<InboundListQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(20).clamp(1, 200);
    let filter = InboundOrderFilter {
        status: q.status,
        inbound_type: q.inbound_type,
    };
    let (rows, total) = inventory_service::list_inbounds(&pool, &filter, page, page_size).await?;
    Ok(Json(PaginatedResponse::ok(rows, total, page, page_size)))
}

pub async fn get_inbound(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let with_items = inventory_service::get_inbound_with_items(&pool, id).await?;
    match with_items {
        Some((order, items)) => Ok(Json(ApiResponse::ok(serde_json::json!({
            "order": order,
            "items": items,
        })))),
        None => Err(AppError::new(
            crate::error::ErrorCode::OrderNotFound,
            "入库单未找到",
        )),
    }
}

pub async fn post_inbound(
    Extension(pool): Extension<SqlitePool>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let order = inventory_service::post_inbound(&pool, id, &user).await?;
    Ok(Json(ApiResponse::ok(order)))
}

// —— Outbound handlers ——

pub async fn create_outbound(
    Extension(pool): Extension<SqlitePool>,
    user: AuthUser,
    Json(req): Json<CreateOutboundRequest>,
) -> Result<impl IntoResponse, AppError> {
    let dto = inventory_service::CreateOutboundRequest {
        outbound_type: req.outbound_type,
        order_id: req.order_id,
        customer_id: req.customer_id,
        notes: req.notes,
        items: req
            .items
            .into_iter()
            .map(|i| inventory_service::CreateOutboundItemInput {
                item_id: i.item_id,
                location_id: i.location_id,
                quantity: i.quantity,
                notes: i.notes,
            })
            .collect(),
    };
    let order = inventory_service::create_outbound(&pool, &dto, &user).await?;
    Ok((StatusCode::CREATED, Json(ApiResponse::ok(order))))
}

pub async fn list_outbounds(
    Extension(pool): Extension<SqlitePool>,
    Query(q): Query<OutboundListQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(20).clamp(1, 200);
    let filter = OutboundOrderFilter {
        status: q.status,
        outbound_type: q.outbound_type,
    };
    let (rows, total) = inventory_service::list_outbounds(&pool, &filter, page, page_size).await?;
    Ok(Json(PaginatedResponse::ok(rows, total, page, page_size)))
}

pub async fn get_outbound(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let with_items = inventory_service::get_outbound_with_items(&pool, id).await?;
    match with_items {
        Some((order, items)) => Ok(Json(ApiResponse::ok(serde_json::json!({
            "order": order,
            "items": items,
        })))),
        None => Err(AppError::new(
            crate::error::ErrorCode::OrderNotFound,
            "出库单未找到",
        )),
    }
}

pub async fn post_outbound(
    Extension(pool): Extension<SqlitePool>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let order = inventory_service::post_outbound(&pool, id, &user).await?;
    Ok(Json(ApiResponse::ok(order)))
}

// —— Stock / logs handlers ——

pub async fn list_stock(
    Extension(pool): Extension<SqlitePool>,
    Query(q): Query<StockListQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(20).clamp(1, 200);
    let filter = StockFilter {
        item_id: q.item_id,
        location_id: q.location_id,
        warehouse_id: q.warehouse_id,
    };
    let (rows, total) = inventory_service::list_stock(&pool, &filter, page, page_size).await?;
    Ok(Json(PaginatedResponse::ok(rows, total, page, page_size)))
}

pub async fn list_logs(
    Extension(pool): Extension<SqlitePool>,
    Query(q): Query<LogListQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(20).clamp(1, 200);
    let filter = InventoryLogFilter {
        item_id: q.item_id,
        location_id: q.location_id,
        change_type: q.change_type,
    };
    let (rows, total) = inventory_service::list_logs(&pool, &filter, page, page_size).await?;
    Ok(Json(PaginatedResponse::ok(rows, total, page, page_size)))
}

pub async fn create_check_session(
    Extension(pool): Extension<SqlitePool>,
    user: AuthUser,
    Json(req): Json<CreateCheckRequest>,
) -> Result<impl IntoResponse, AppError> {
    let input = inventory_service::CheckSessionCreateInput {
        location_id: req.location_id,
        scope: req.scope,
    };
    let session = inventory_service::create_check_session(&pool, &input, &user).await?;
    Ok((StatusCode::CREATED, Json(ApiResponse::created(session))))
}

pub async fn list_check_sessions(
    Extension(pool): Extension<SqlitePool>,
    Query(q): Query<InboundListQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(20).clamp(1, 200);
    let (rows, total) = inventory_service::list_check_sessions(&pool, page, page_size).await?;
    Ok(Json(PaginatedResponse::ok(rows, total, page, page_size)))
}

pub async fn get_check_session(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let (session, details) = inventory_service::get_check_session(&pool, id).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "session": session,
        "details": details,
    }))))
}

pub async fn record_actual_qty(
    Extension(pool): Extension<SqlitePool>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(req): Json<RecordActualQtyRequest>,
) -> Result<impl IntoResponse, AppError> {
    inventory_service::record_actual_qty(&pool, id, req.detail_id, req.actual_qty, &user).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "id": id }))))
}

pub async fn post_check_session(
    Extension(pool): Extension<SqlitePool>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let session = inventory_service::post_check_session(&pool, id, &user).await?;
    Ok(Json(ApiResponse::ok(session)))
}

/// ATP 可用量查询：GET /inventory/available?item_id=&location_id=
pub async fn get_available_qty(
    Extension(pool): Extension<SqlitePool>,
    Query(params): Query<AvailableQtyParams>,
) -> Result<impl IntoResponse, AppError> {
    let available = inventory_service::get_available_qty(
        &pool, params.item_id, params.location_id,
    ).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "item_id": params.item_id,
        "location_id": params.location_id,
        "available_qty": available,
    }))))
}

#[derive(serde::Deserialize)]
pub struct AvailableQtyParams {
    pub item_id: i64,
    pub location_id: Option<i64>,
}
