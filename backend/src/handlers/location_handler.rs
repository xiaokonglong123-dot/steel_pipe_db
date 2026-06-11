use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::dto::common::PaginationParams;
use crate::dto::inventory_dto::{
    AssignLocationRequest, CreateLocationRequest, TransferLocationRequest, UpdateLocationRequest,
};
use validator::Validate;

use crate::error::AppError;
use crate::models::inventory::Location;
use crate::response::{ApiResponse, PaginatedResponse};
use crate::services::location_service::LocationService;

#[derive(Deserialize)]
pub struct LocationListQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub active_only: Option<bool>,
}

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

pub async fn create_location_handler(
    Extension(pool): Extension<SqlitePool>,
    Json(req): Json<CreateLocationRequest>,
) -> Result<axum::response::Response, AppError> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    let location = LocationService::create_location(&pool, &req).await?;
    Ok(ApiResponse::created(location))
}

pub async fn get_location_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<Location>>, AppError> {
    let location = LocationService::get_location(&pool, id).await?;
    Ok(ApiResponse::ok(location))
}

pub async fn update_location_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateLocationRequest>,
) -> Result<Json<ApiResponse<Location>>, AppError> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    let location = LocationService::update_location(&pool, id, &req).await?;
    Ok(ApiResponse::ok(location))
}

pub async fn delete_location_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<axum::response::Response, AppError> {
    LocationService::delete_location(&pool, id).await?;
    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

pub async fn assign_location_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(location_id): Path<i64>,
    Json(req): Json<AssignLocationRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    let result = LocationService::assign_location(&pool, location_id, &req).await?;
    Ok(ApiResponse::ok(result))
}

pub async fn transfer_location_handler(
    Extension(pool): Extension<SqlitePool>,
    Path((pipe_type, pipe_id)): Path<(String, i64)>,
    Json(req): Json<TransferLocationRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    let result = LocationService::transfer_location(&pool, &pipe_type, pipe_id, &req).await?;
    Ok(ApiResponse::ok(result))
}
