use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sqlx::SqlitePool;

use crate::dto::common::PaginationParams;
use crate::dto::inventory_dto::{
    ApproveRequest, BatchCreateInboundRequest, CreateInboundRecordRequest, InboundFilter,
    RejectRequest, UpdateInboundRecordRequest,
};
use validator::Validate;

use crate::error::AppError;
use crate::middleware::auth::AuthContext;
use crate::models::inventory::{InboundItem, InboundRecord};
use crate::response::{ApiResponse, PaginatedResponse};
use crate::services::inbound_service::InboundService;

pub async fn create_inbound_handler(
    Extension(pool): Extension<SqlitePool>,
    Json(req): Json<CreateInboundRecordRequest>,
) -> Result<axum::response::Response, AppError> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    let record = InboundService::create_inbound(&pool, &req).await?;
    Ok(ApiResponse::created(record))
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

    let (items, total) = InboundService::list_inbound_records(&pool, &filter).await?;

    Ok(PaginatedResponse::ok(items, total, page, page_size))
}

pub async fn get_inbound_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<crate::dto::inventory_dto::InboundRecordDetail>>, AppError> {
    let (record, items) = InboundService::get_inbound_record(&pool, id).await?;
    Ok(ApiResponse::ok(
        crate::dto::inventory_dto::InboundRecordDetail { record, items },
    ))
}

pub async fn approve_inbound_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<ApproveRequest>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    InboundService::approve_inbound(&pool, id, req.reason.as_deref(), Some(auth.user_id)).await?;
    Ok(ApiResponse::ok("Inbound approved".into()))
}

pub async fn reject_inbound_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
    Json(req): Json<RejectRequest>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    InboundService::reject_inbound(&pool, id, &req.reason).await?;
    Ok(ApiResponse::ok("Inbound rejected".into()))
}

pub async fn update_inbound_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateInboundRecordRequest>,
) -> Result<Json<ApiResponse<InboundRecord>>, AppError> {
    let record = InboundService::update_inbound(&pool, id, &req).await?;
    Ok(ApiResponse::ok(record))
}

pub async fn delete_inbound_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<axum::response::Response, AppError> {
    InboundService::delete_inbound(&pool, id).await?;
    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

pub async fn list_inbound_items_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<Vec<InboundItem>>>, AppError> {
    let items = InboundService::list_inbound_items(&pool, id).await?;
    Ok(ApiResponse::ok(items))
}

pub async fn batch_create_inbound_handler(
    Extension(pool): Extension<SqlitePool>,
    Json(req): Json<BatchCreateInboundRequest>,
) -> Result<axum::response::Response, AppError> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    let records = InboundService::batch_create_inbound(&pool, &req).await?;
    Ok(ApiResponse::created(records))
}
