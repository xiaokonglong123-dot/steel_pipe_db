use axum::{
    extract::{Extension, Path, Query},
    Json,
};
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::dto::common::PaginationParams;
use crate::dto::inventory_dto::{CreateCheckRequest, SubmitCheckItemRequest};
use validator::Validate;

use crate::error::AppError;
use crate::models::inventory::InventoryCheckRecord;
use crate::response::{ApiResponse, PaginatedResponse};
use crate::inventory::check_service::CheckService;

#[derive(Deserialize)]
pub struct CheckListQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

pub async fn create_check_handler(
    Extension(pool): Extension<SqlitePool>,
    Json(req): Json<CreateCheckRequest>,
) -> Result<axum::response::Response, AppError> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    let record = CheckService::create_check(&pool, &req).await?;
    Ok(ApiResponse::created(record))
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

    let (items, total) = CheckService::list_checks(&pool, &pagination).await?;

    Ok(PaginatedResponse::ok(items, total, page, page_size))
}

pub async fn get_check_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<crate::dto::inventory_dto::CheckRecordDetail>>, AppError> {
    let (record, items) = CheckService::get_check_detail(&pool, id).await?;
    Ok(ApiResponse::ok(
        crate::dto::inventory_dto::CheckRecordDetail { record, items },
    ))
}

pub async fn submit_check_item_handler(
    Extension(pool): Extension<SqlitePool>,
    Path((check_id, item_id)): Path<(i64, i64)>,
    Json(req): Json<SubmitCheckItemRequest>,
) -> Result<axum::response::Response, AppError> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    let item = CheckService::submit_check_item(&pool, check_id, item_id, &req).await?;
    Ok(ApiResponse::created(item))
}

pub async fn complete_check_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let result = CheckService::complete_check(&pool, id).await?;
    Ok(ApiResponse::ok(result))
}
