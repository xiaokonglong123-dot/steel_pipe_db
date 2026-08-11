//! Sales HTTP handlers — 销售订单 + ATP 预留
//!
//! 对齐 http/inventory.rs：`Extension(pool): Extension<SqlitePool>`，DTO 解析+校验后调 service。
//! POST/PUT/DELETE/submit/approve/reject/cancel 走 `order.write` / `order.approve`；
//! GET 类走 `order.read`（路由在 http/mod.rs 分 sub-router 绑定）。

use axum::extract::{Extension, Path, Query};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::repos::sales_repo::SalesOrderFilter;
use crate::response::{ApiResponse, PaginatedResponse};
use crate::services::sales_service;

// —— DTOs ——

#[derive(Deserialize)]
pub struct SalesOrderItemDto {
    pub item_id: i64,
    pub quantity: f64,
    pub unit_price: String,
    pub notes: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateSalesOrderRequest {
    pub customer_id: i64,
    #[serde(default)]
    pub order_date: Option<String>,
    #[serde(default)]
    pub currency: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    pub items: Vec<SalesOrderItemDto>,
}

#[derive(Deserialize)]
pub struct UpdateSalesOrderRequest {
    pub customer_id: i64,
    #[serde(default)]
    pub order_date: Option<String>,
    #[serde(default)]
    pub currency: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    pub items: Vec<SalesOrderItemDto>,
}

#[derive(Deserialize)]
pub struct SalesOrderListQuery {
    pub customer_id: Option<i64>,
    pub status: Option<String>,
    pub order_date_from: Option<String>,
    pub order_date_to: Option<String>,
    pub order_no: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Deserialize)]
pub struct ReservationListQuery {
    pub item_id: i64,
}

// —— Handlers ——

pub async fn create_order(
    Extension(pool): Extension<SqlitePool>,
    user: AuthUser,
    Json(req): Json<CreateSalesOrderRequest>,
) -> Result<impl IntoResponse, AppError> {
    let dto = sales_service::CreateSalesOrderRequest {
        customer_id: req.customer_id,
        order_date: req.order_date,
        currency: req.currency,
        notes: req.notes,
        items: req
            .items
            .into_iter()
            .map(|i| sales_service::CreateSalesOrderItemInput {
                item_id: i.item_id,
                quantity: i.quantity,
                unit_price: i.unit_price,
                notes: i.notes,
            })
            .collect(),
    };
    let order = sales_service::create_order(&pool, &dto, &user).await?;
    Ok((StatusCode::CREATED, Json(ApiResponse::ok(order))))
}

pub async fn list_orders(
    Extension(pool): Extension<SqlitePool>,
    Query(q): Query<SalesOrderListQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(20).clamp(1, 200);
    let filter = SalesOrderFilter {
        customer_id: q.customer_id,
        status: q.status,
        order_date_from: q.order_date_from,
        order_date_to: q.order_date_to,
        order_no: q.order_no,
    };
    let (rows, total) = sales_service::list_orders(&pool, &filter, page, page_size).await?;
    Ok(Json(PaginatedResponse::ok(rows, total, page, page_size)))
}

pub async fn get_order(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let with_items = sales_service::get_order_with_items(&pool, id).await?;
    match with_items {
        Some((order, items)) => Ok(Json(ApiResponse::ok(serde_json::json!({
            "order": order,
            "items": items,
        })))),
        None => Err(AppError::new(
            crate::error::ErrorCode::OrderNotFound,
            "销售订单未找到",
        )),
    }
}

pub async fn update_order(
    Extension(pool): Extension<SqlitePool>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(req): Json<UpdateSalesOrderRequest>,
) -> Result<impl IntoResponse, AppError> {
    let dto = sales_service::UpdateSalesOrderRequest {
        customer_id: req.customer_id,
        order_date: req.order_date,
        currency: req.currency,
        notes: req.notes,
        items: req
            .items
            .into_iter()
            .map(|i| sales_service::CreateSalesOrderItemInput {
                item_id: i.item_id,
                quantity: i.quantity,
                unit_price: i.unit_price,
                notes: i.notes,
            })
            .collect(),
    };
    let order = sales_service::update_order(&pool, id, &dto, &user).await?;
    Ok(Json(ApiResponse::ok(order)))
}

pub async fn delete_order(
    Extension(pool): Extension<SqlitePool>,
    _user: AuthUser,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    sales_service::delete_order(&pool, id).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "deleted": id }))))
}

pub async fn submit_order(
    Extension(pool): Extension<SqlitePool>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let order = sales_service::submit(&pool, id, &user).await?;
    Ok(Json(ApiResponse::ok(order)))
}

pub async fn approve_order(
    Extension(pool): Extension<SqlitePool>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let order = sales_service::approve(&pool, id, &user).await?;
    Ok(Json(ApiResponse::ok(order)))
}

pub async fn reject_order(
    Extension(pool): Extension<SqlitePool>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let order = sales_service::reject(&pool, id, &user).await?;
    Ok(Json(ApiResponse::ok(order)))
}

pub async fn cancel_order(
    Extension(pool): Extension<SqlitePool>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let order = sales_service::cancel(&pool, id, &user).await?;
    Ok(Json(ApiResponse::ok(order)))
}

pub async fn list_reservations(
    Extension(pool): Extension<SqlitePool>,
    Query(q): Query<ReservationListQuery>,
) -> Result<impl IntoResponse, AppError> {
    let rows = sales_service::list_active_reservations_for_item(&pool, q.item_id).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "items": rows }))))
}
