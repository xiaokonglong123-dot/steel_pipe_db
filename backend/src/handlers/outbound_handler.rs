use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sqlx::SqlitePool;

use crate::dto::common::PaginationParams;
use crate::dto::inventory_dto::{
    ApproveRequest, CreateOutboundRecordRequest, OutboundFilter, RejectRequest,
    UpdateOutboundRecordRequest,
};
use validator::Validate;

use crate::error::AppError;
use crate::middleware::auth::AuthContext;
use crate::models::inventory::{OutboundItem, OutboundRecord};
use crate::response::{ApiResponse, PaginatedResponse};
use crate::services::outbound_service::OutboundService;

pub async fn create_outbound_handler(
    Extension(pool): Extension<SqlitePool>,
    Json(req): Json<CreateOutboundRecordRequest>,
) -> Result<axum::response::Response, AppError> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    let record = OutboundService::create_outbound(&pool, &req).await?;
    Ok(ApiResponse::created(record))
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

    let (items, total) = OutboundService::list_outbound_records(&pool, &filter).await?;

    Ok(PaginatedResponse::ok(items, total, page, page_size))
}

pub async fn get_outbound_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<crate::dto::inventory_dto::OutboundRecordDetail>>, AppError> {
    let (record, items) = OutboundService::get_outbound_record(&pool, id).await?;
    Ok(ApiResponse::ok(
        crate::dto::inventory_dto::OutboundRecordDetail { record, items },
    ))
}

pub async fn approve_outbound_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<ApproveRequest>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    OutboundService::approve_outbound(&pool, id, req.reason.as_deref(), Some(auth.user_id)).await?;
    Ok(ApiResponse::ok("Outbound approved".into()))
}

pub async fn reject_outbound_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
    Json(req): Json<RejectRequest>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    OutboundService::reject_outbound(&pool, id, &req.reason).await?;
    Ok(ApiResponse::ok("Outbound rejected".into()))
}

pub async fn update_outbound_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateOutboundRecordRequest>,
) -> Result<Json<ApiResponse<OutboundRecord>>, AppError> {
    let record = OutboundService::update_outbound(&pool, id, &req).await?;
    Ok(ApiResponse::ok(record))
}

pub async fn delete_outbound_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<axum::response::Response, AppError> {
    OutboundService::delete_outbound(&pool, id).await?;
    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

pub async fn list_outbound_items_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<Vec<OutboundItem>>>, AppError> {
    let items = OutboundService::list_outbound_items(&pool, id).await?;
    Ok(ApiResponse::ok(items))
}
