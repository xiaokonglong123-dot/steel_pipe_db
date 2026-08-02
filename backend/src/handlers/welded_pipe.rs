use axum::{
    extract::{Extension, Path, Query},
    Json,
};
use sqlx::SqlitePool;
use validator::Validate;

use crate::cache_invalidator::CacheInvalidator;
use crate::dto::common::PaginationParams;
use crate::dto::pipe_dto::{CreateWeldedPipeRequest, PipeFilterParams, UpdateWeldedPipeRequest};
use crate::error::AppError;
use crate::models::welded_pipe::WeldedPipe;
use crate::response::{ApiResponse, PaginatedResponse};
use crate::services::pipe_service::PipeService;

/// GET `/api/v1/welded-pipes` — Paginated list of welded pipes
pub async fn list_welded_pipes_handler(
    Extension(pool): Extension<SqlitePool>,
    Query(filter): Query<PipeFilterParams>,
) -> Result<Json<PaginatedResponse<WeldedPipe>>, AppError> {
    let pagination = PaginationParams {
        page: Some(filter.page.unwrap_or(1)),
        page_size: Some(filter.page_size.unwrap_or(10)),
        sort_by: filter.sort_by.clone(),
        sort_order: filter.sort_order.clone(),
    };
    let page = pagination.page();
    let page_size = pagination.page_size();

    let (items, total) = PipeService::list_welded_pipes(&pool, &filter, &pagination).await?;

    Ok(PaginatedResponse::ok(items, total, page, page_size))
}

/// POST `/api/v1/welded-pipes` — Create a new welded pipe
pub async fn create_welded_pipe_handler(
    Extension(pool): Extension<SqlitePool>,
    Extension(cache): Extension<CacheInvalidator>,
    Json(req): Json<CreateWeldedPipeRequest>,
) -> Result<axum::response::Response, AppError> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    let pipe = PipeService::create_welded_pipe(&pool, &cache, &req).await?;
    Ok(ApiResponse::created(pipe))
}

/// GET `/api/v1/welded-pipes/{id}` — Get welded pipe by ID
pub async fn get_welded_pipe_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<WeldedPipe>>, AppError> {
    let pipe = PipeService::get_welded_pipe(&pool, id).await?;
    Ok(ApiResponse::ok(pipe))
}

/// PUT `/api/v1/welded-pipes/{id}` — Update a welded pipe
pub async fn update_welded_pipe_handler(
    Extension(pool): Extension<SqlitePool>,
    Extension(cache): Extension<CacheInvalidator>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateWeldedPipeRequest>,
) -> Result<Json<ApiResponse<WeldedPipe>>, AppError> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    let pipe = PipeService::update_welded_pipe(&pool, &cache, id, &req).await?;
    Ok(ApiResponse::ok(pipe))
}

/// DELETE `/api/v1/welded-pipes/{id}` — Soft-delete a welded pipe
pub async fn delete_welded_pipe_handler(
    Extension(pool): Extension<SqlitePool>,
    Extension(cache): Extension<CacheInvalidator>,
    Path(id): Path<i64>,
) -> Result<axum::response::Response, AppError> {
    PipeService::delete_welded_pipe(&pool, &cache, id).await?;
    Ok(crate::response::no_content())
}
