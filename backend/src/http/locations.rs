//! Warehouse & Location HTTP handlers — 仓库/库位 CRUD
//!
//! 对齐 http/catalog.rs：`Extension(pool): Extension<SqlitePool>`，DTO 解析后调 service。

use axum::extract::{Extension, Path, Query};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::repos::location_repo::{LocationFilter, WarehouseFilter};
use crate::response::{ApiResponse, PaginatedResponse};
use crate::services::location_service;

// —— Warehouse DTOs ——

#[derive(Deserialize)]
pub struct CreateWarehouseRequest {
    pub code: String,
    pub name: String,
    pub address: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateWarehouseRequest {
    pub code: String,
    pub name: String,
    pub address: Option<String>,
}

#[derive(Deserialize)]
pub struct WarehouseFilterQuery {
    pub code: Option<String>,
    pub name: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

// —— Location DTOs ——

#[derive(Deserialize)]
pub struct CreateLocationRequest {
    pub warehouse_id: Option<i64>,
    pub code: String,
    pub name: String,
}

#[derive(Deserialize)]
pub struct UpdateLocationRequest {
    pub warehouse_id: Option<i64>,
    pub code: String,
    pub name: String,
}

#[derive(Deserialize)]
pub struct LocationFilterQuery {
    pub warehouse_id: Option<i64>,
    pub code: Option<String>,
    pub name: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

// —— Warehouse handlers ——

pub async fn create_warehouse(
    Extension(pool): Extension<SqlitePool>,
    Json(req): Json<CreateWarehouseRequest>,
) -> Result<impl IntoResponse, AppError> {
    let wh =
        location_service::create_warehouse(&pool, &req.code, &req.name, req.address.as_deref())
            .await?;
    Ok((StatusCode::CREATED, Json(ApiResponse::ok(wh))))
}

pub async fn list_warehouses(
    Extension(pool): Extension<SqlitePool>,
    Query(q): Query<WarehouseFilterQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(20).clamp(1, 200);
    let filter = WarehouseFilter {
        code: q.code.as_deref(),
        name: q.name.as_deref(),
    };
    let (rows, total) = location_service::list_warehouses(&pool, &filter, page, page_size).await?;
    Ok(Json(PaginatedResponse::ok(rows, total, page, page_size)))
}

pub async fn get_warehouse(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let wh = location_service::get_warehouse(&pool, id).await?;
    Ok(Json(ApiResponse::ok(wh)))
}

pub async fn update_warehouse(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateWarehouseRequest>,
) -> Result<impl IntoResponse, AppError> {
    let wh =
        location_service::update_warehouse(&pool, id, &req.code, &req.name, req.address.as_deref())
            .await?;
    Ok((StatusCode::OK, Json(ApiResponse::ok(wh))))
}

pub async fn delete_warehouse(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    location_service::delete_warehouse(&pool, id).await?;
    Ok((
        StatusCode::OK,
        Json(ApiResponse::ok(serde_json::json!({ "id": id }))),
    ))
}

// —— Location handlers ——

pub async fn create_location(
    Extension(pool): Extension<SqlitePool>,
    Json(req): Json<CreateLocationRequest>,
) -> Result<impl IntoResponse, AppError> {
    let loc =
        location_service::create_location(&pool, req.warehouse_id, &req.code, &req.name).await?;
    Ok((StatusCode::CREATED, Json(ApiResponse::ok(loc))))
}

pub async fn list_locations(
    Extension(pool): Extension<SqlitePool>,
    Query(q): Query<LocationFilterQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(20).clamp(1, 200);
    let filter = LocationFilter {
        warehouse_id: q.warehouse_id,
        code: q.code.as_deref(),
        name: q.name.as_deref(),
    };
    let (rows, total) = location_service::list_locations(&pool, &filter, page, page_size).await?;
    Ok(Json(PaginatedResponse::ok(rows, total, page, page_size)))
}

pub async fn get_location(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let loc = location_service::get_location(&pool, id).await?;
    Ok(Json(ApiResponse::ok(loc)))
}

pub async fn update_location(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateLocationRequest>,
) -> Result<impl IntoResponse, AppError> {
    let loc = location_service::update_location(&pool, id, req.warehouse_id, &req.code, &req.name)
        .await?;
    Ok((StatusCode::OK, Json(ApiResponse::ok(loc))))
}

pub async fn delete_location(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    location_service::delete_location(&pool, id).await?;
    Ok((
        StatusCode::OK,
        Json(ApiResponse::ok(serde_json::json!({ "id": id }))),
    ))
}
