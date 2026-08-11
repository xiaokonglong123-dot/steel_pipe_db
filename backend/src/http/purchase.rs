//! Purchase HTTP handlers — 采购订单 CRUD + 状态迁移
//!
//! 路由（在 http/mod.rs 分 sub-router 绑定）：
//!   order.read  → GET  /purchase-orders, /purchase-orders/{id}
//!   order.write → POST /purchase-orders, PUT /purchase-orders/{id},
//!                 DELETE /purchase-orders/{id}, POST /{id}/cancel
//!   order.approve → POST /{id}/submit, /{id}/approve, /{id}/reject

use axum::extract::{Extension, Path, Query};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::repos::purchase_repo::PurchaseOrderFilter;
use crate::response::{ApiResponse, PaginatedResponse};
use crate::services::purchase_service;
use crate::services::purchase_service::{CreatePurchaseOrderRequest, UpdatePurchaseOrderRequest};

// —— DTOs ——

#[derive(Deserialize)]
pub struct PurchaseItemDto {
    pub item_id: i64,
    pub quantity: f64,
    #[serde(default)]
    pub unit_price: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Deserialize)]
pub struct CreatePurchaseOrderDto {
    pub supplier_id: i64,
    pub order_date: String,
    #[serde(default)]
    pub currency: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    pub items: Vec<PurchaseItemDto>,
}

#[derive(Deserialize)]
pub struct UpdatePurchaseOrderDto {
    pub supplier_id: i64,
    pub order_date: String,
    #[serde(default)]
    pub currency: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    pub items: Vec<PurchaseItemDto>,
}

#[derive(Deserialize)]
pub struct PurchaseListQuery {
    #[serde(default)]
    pub supplier_id: Option<i64>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub order_date_from: Option<String>,
    #[serde(default)]
    pub order_date_to: Option<String>,
    #[serde(default)]
    pub order_no: Option<String>,
    #[serde(default)]
    pub page: Option<i64>,
    #[serde(default)]
    pub page_size: Option<i64>,
}

fn to_input(d: PurchaseItemDto) -> purchase_service::PurchaseOrderItemInput {
    purchase_service::PurchaseOrderItemInput {
        item_id: d.item_id,
        quantity: d.quantity,
        unit_price: d.unit_price,
        notes: d.notes,
    }
}

// —— Handlers ——

pub async fn create_purchase_order(
    Extension(pool): Extension<SqlitePool>,
    user: AuthUser,
    Json(req): Json<CreatePurchaseOrderDto>,
) -> Result<impl IntoResponse, AppError> {
    let dto = CreatePurchaseOrderRequest {
        supplier_id: req.supplier_id,
        order_date: req.order_date,
        currency: req.currency,
        notes: req.notes,
        items: req.items.into_iter().map(to_input).collect(),
    };
    let order = purchase_service::create_order(&pool, &dto, &user).await?;
    Ok((StatusCode::CREATED, Json(ApiResponse::ok(order))))
}

pub async fn list_purchase_orders(
    Extension(pool): Extension<SqlitePool>,
    Query(q): Query<PurchaseListQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(20).clamp(1, 200);
    let filter = PurchaseOrderFilter {
        supplier_id: q.supplier_id,
        status: q.status,
        order_date_from: q.order_date_from,
        order_date_to: q.order_date_to,
        order_no: q.order_no,
    };
    let (rows, total) = purchase_service::list_orders(&pool, &filter, page, page_size).await?;
    Ok(Json(PaginatedResponse::ok(rows, total, page, page_size)))
}

pub async fn get_purchase_order(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let (order, items) = purchase_service::get_order(&pool, id).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "order": order,
        "items": items,
    }))))
}

pub async fn update_purchase_order(
    Extension(pool): Extension<SqlitePool>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(req): Json<UpdatePurchaseOrderDto>,
) -> Result<impl IntoResponse, AppError> {
    let dto = UpdatePurchaseOrderRequest {
        supplier_id: req.supplier_id,
        order_date: req.order_date,
        currency: req.currency,
        notes: req.notes,
        items: req.items.into_iter().map(to_input).collect(),
    };
    purchase_service::update_order(&pool, id, &dto, &user).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "id": id }))))
}

pub async fn delete_purchase_order(
    Extension(pool): Extension<SqlitePool>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    purchase_service::delete_order(&pool, id, &user).await?;
    Ok((
        StatusCode::OK,
        Json(ApiResponse::ok(serde_json::json!({ "id": id }))),
    ))
}

pub async fn submit_purchase_order(
    Extension(pool): Extension<SqlitePool>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let order = purchase_service::submit(&pool, id, &user).await?;
    Ok(Json(ApiResponse::ok(order)))
}

pub async fn approve_purchase_order(
    Extension(pool): Extension<SqlitePool>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let order = purchase_service::approve(&pool, id, &user).await?;
    Ok(Json(ApiResponse::ok(order)))
}

pub async fn reject_purchase_order(
    Extension(pool): Extension<SqlitePool>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let order = purchase_service::reject(&pool, id, &user).await?;
    Ok(Json(ApiResponse::ok(order)))
}

pub async fn cancel_purchase_order(
    Extension(pool): Extension<SqlitePool>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let order = purchase_service::cancel(&pool, id, &user).await?;
    Ok(Json(ApiResponse::ok(order)))
}
