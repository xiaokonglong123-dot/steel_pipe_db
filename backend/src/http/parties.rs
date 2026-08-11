//! Parties HTTP handlers — 供应商 / 客户 CRUD
//!
//! 对齐 http/catalog.rs：`Extension(pool): Extension<SqlitePool>`，DTO 解析+校验后调 service。
//! 权限映射：parties 复用 order.read / order.write（种子权限无独立 party.read/write；
//! 往来单位被订单引用，订单读写者需同步读写往来单位 — v2 P0 权宜方案）。

use axum::extract::{Extension, Path, Query};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::repos::parties_repo::{CustomerFilter, SupplierFilter};
use crate::response::{ApiResponse, PaginatedResponse};
use crate::services::parties_service;

// —— Supplier DTOs ——

#[derive(Deserialize)]
pub struct CreateSupplierRequest {
    pub code: String,
    pub name: String,
    pub contact: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateSupplierRequest {
    pub code: String,
    pub name: String,
    pub contact: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub status: Option<String>,
}

#[derive(Deserialize)]
pub struct SupplierFilterQuery {
    pub code: Option<String>,
    pub name: Option<String>,
    pub status: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

// —— Customer DTOs ——

#[derive(Deserialize)]
pub struct CreateCustomerRequest {
    pub code: String,
    pub name: String,
    pub contact: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateCustomerRequest {
    pub code: String,
    pub name: String,
    pub contact: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub status: Option<String>,
}

#[derive(Deserialize)]
pub struct CustomerFilterQuery {
    pub code: Option<String>,
    pub name: Option<String>,
    pub status: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

// —— Supplier handlers ——

pub async fn create_supplier(
    Extension(pool): Extension<SqlitePool>,
    Json(req): Json<CreateSupplierRequest>,
) -> Result<impl IntoResponse, AppError> {
    let row = parties_service::create_supplier(
        &pool,
        &req.code,
        &req.name,
        req.contact.as_deref(),
        req.phone.as_deref(),
        req.email.as_deref(),
        req.address.as_deref(),
    )
    .await?;
    Ok((StatusCode::CREATED, Json(ApiResponse::ok(row))))
}

pub async fn list_suppliers(
    Extension(pool): Extension<SqlitePool>,
    Query(q): Query<SupplierFilterQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(20).clamp(1, 200);
    let filter = SupplierFilter {
        code: q.code.as_deref(),
        name: q.name.as_deref(),
        status: q.status.as_deref(),
    };
    let (rows, total) = parties_service::list_suppliers(&pool, &filter, page, page_size).await?;
    Ok(Json(PaginatedResponse::ok(rows, total, page, page_size)))
}

pub async fn get_supplier(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let row = parties_service::get_supplier(&pool, id).await?;
    Ok(Json(ApiResponse::ok(row)))
}

pub async fn update_supplier(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateSupplierRequest>,
) -> Result<impl IntoResponse, AppError> {
    let status = req.status.as_deref().unwrap_or("active");
    let row = parties_service::update_supplier(
        &pool,
        id,
        &req.code,
        &req.name,
        req.contact.as_deref(),
        req.phone.as_deref(),
        req.email.as_deref(),
        req.address.as_deref(),
        status,
    )
    .await?;
    Ok((StatusCode::OK, Json(ApiResponse::ok(row))))
}

pub async fn delete_supplier(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    parties_service::delete_supplier(&pool, id).await?;
    Ok((
        StatusCode::OK,
        Json(ApiResponse::ok(serde_json::json!({ "id": id }))),
    ))
}

// —— Customer handlers ——

pub async fn create_customer(
    Extension(pool): Extension<SqlitePool>,
    Json(req): Json<CreateCustomerRequest>,
) -> Result<impl IntoResponse, AppError> {
    let row = parties_service::create_customer(
        &pool,
        &req.code,
        &req.name,
        req.contact.as_deref(),
        req.phone.as_deref(),
        req.email.as_deref(),
        req.address.as_deref(),
    )
    .await?;
    Ok((StatusCode::CREATED, Json(ApiResponse::ok(row))))
}

pub async fn list_customers(
    Extension(pool): Extension<SqlitePool>,
    Query(q): Query<CustomerFilterQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(20).clamp(1, 200);
    let filter = CustomerFilter {
        code: q.code.as_deref(),
        name: q.name.as_deref(),
        status: q.status.as_deref(),
    };
    let (rows, total) = parties_service::list_customers(&pool, &filter, page, page_size).await?;
    Ok(Json(PaginatedResponse::ok(rows, total, page, page_size)))
}

pub async fn get_customer(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let row = parties_service::get_customer(&pool, id).await?;
    Ok(Json(ApiResponse::ok(row)))
}

pub async fn update_customer(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateCustomerRequest>,
) -> Result<impl IntoResponse, AppError> {
    let status = req.status.as_deref().unwrap_or("active");
    let row = parties_service::update_customer(
        &pool,
        id,
        &req.code,
        &req.name,
        req.contact.as_deref(),
        req.phone.as_deref(),
        req.email.as_deref(),
        req.address.as_deref(),
        status,
    )
    .await?;
    Ok((StatusCode::OK, Json(ApiResponse::ok(row))))
}

pub async fn delete_customer(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    parties_service::delete_customer(&pool, id).await?;
    Ok((
        StatusCode::OK,
        Json(ApiResponse::ok(serde_json::json!({ "id": id }))),
    ))
}
