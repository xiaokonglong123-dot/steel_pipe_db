use axum::{
    extract::{Extension, Path, Query},
    Json,
};
use serde::Deserialize;
use sqlx::SqlitePool;

use validator::Validate;

use crate::dto::common::PaginationParams;
use crate::dto::pipe_dto::{
    CreateScreenPipeRequest, CreateSeamlessPipeRequest, PipeFilterParams,
    PipeSearchResult, UpdateScreenPipeRequest, UpdateSeamlessPipeRequest,
};
use crate::error::AppError;
use crate::models::screen_pipe::ScreenPipe;
use crate::models::seamless_pipe::SeamlessPipe;
use crate::response::{ApiResponse, PaginatedResponse};
use crate::services::pipe_service::PipeService;

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

// ━━━ Seamless Pipe Handlers ━━━

/// GET `/api/v1/seamless-pipes` — Paginated list of seamless pipes
///
/// Returns a paginated list of seamless pipes, filterable by spec, grade, heat number, etc.
/// Supports sorting and pagination via query params.
pub async fn list_seamless_pipes_handler(
    Extension(pool): Extension<SqlitePool>,
    Query(filter): Query<PipeFilterParams>,
) -> Result<Json<PaginatedResponse<SeamlessPipe>>, AppError> {
    let pagination = PaginationParams {
        page: filter.page,
        page_size: filter.page_size,
        sort_by: filter.sort_by.clone(),
        sort_order: filter.sort_order.clone(),
    };
    let page = pagination.page();
    let page_size = pagination.page_size();

    let (items, total) =
        PipeService::list_seamless_pipes(&pool, &filter, &pagination).await?;

    Ok(PaginatedResponse::ok(items, total, page, page_size))
}

/// POST `/api/v1/seamless-pipes` — Whip up a new seamless pipe
///
/// Creates a new seamless pipe record with API 5CT specs (grade, heat treatment, threading, etc.).
/// Validates the request body. Warehouse/admin role required.
/// Returns 400 on validation error.
pub async fn create_seamless_pipe_handler(
    Extension(pool): Extension<SqlitePool>,
    Json(req): Json<CreateSeamlessPipeRequest>,
) -> Result<Json<ApiResponse<SeamlessPipe>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let pipe = PipeService::create_seamless_pipe(&pool, &req).await?;
    Ok(ApiResponse::ok(pipe))
}

/// GET `/api/v1/seamless-pipes/{id}` — Grab seamless pipe deets by ID
///
/// Returns a single seamless pipe record by its ID.
/// Returns 404 if not found.
pub async fn get_seamless_pipe_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<SeamlessPipe>>, AppError> {
    let pipe = PipeService::get_seamless_pipe(&pool, id).await?;
    Ok(ApiResponse::ok(pipe))
}

/// PUT `/api/v1/seamless-pipes/{id}` — Update a seamless pipe
///
/// Updates an existing seamless pipe record with partial fields.
/// Validates the request body. Returns 404 if not found.
pub async fn update_seamless_pipe_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateSeamlessPipeRequest>,
) -> Result<Json<ApiResponse<SeamlessPipe>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let pipe = PipeService::update_seamless_pipe(&pool, id, &req).await?;
    Ok(ApiResponse::ok(pipe))
}

/// DELETE `/api/v1/seamless-pipes/{id}` — Soft-delete a seamless pipe
///
/// Soft-deletes a seamless pipe record. Returns 404 if not found.
pub async fn delete_seamless_pipe_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    PipeService::delete_seamless_pipe(&pool, id).await?;
    Ok(ApiResponse::ok("Seamless pipe deleted successfully".into()))
}

// ━━━ Screen Pipe Handlers ━━━

/// GET `/api/v1/screen-pipes` — Paginated list of screen pipes
///
/// Returns a paginated list of screen pipes, filterable by spec, grade, etc.
/// Supports sorting and pagination via query params.
pub async fn list_screen_pipes_handler(
    Extension(pool): Extension<SqlitePool>,
    Query(filter): Query<PipeFilterParams>,
) -> Result<Json<PaginatedResponse<ScreenPipe>>, AppError> {
    let pagination = PaginationParams {
        page: filter.page,
        page_size: filter.page_size,
        sort_by: filter.sort_by.clone(),
        sort_order: filter.sort_order.clone(),
    };
    let page = pagination.page();
    let page_size = pagination.page_size();

    let (items, total) = PipeService::list_screen_pipes(&pool, &filter, &pagination).await?;

    Ok(PaginatedResponse::ok(items, total, page, page_size))
}

/// POST `/api/v1/screen-pipes` — Create a new screen pipe
///
/// Creates a new screen pipe record with specs (slot width, wire type, etc.).
/// Validates the request body. Warehouse/admin role required.
pub async fn create_screen_pipe_handler(
    Extension(pool): Extension<SqlitePool>,
    Json(req): Json<CreateScreenPipeRequest>,
) -> Result<Json<ApiResponse<ScreenPipe>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let pipe = PipeService::create_screen_pipe(&pool, &req).await?;
    Ok(ApiResponse::ok(pipe))
}

/// GET `/api/v1/screen-pipes/{id}` — Get screen pipe by ID
///
/// Returns a single screen pipe record by its ID. Returns 404 if not found.
pub async fn get_screen_pipe_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<ScreenPipe>>, AppError> {
    let pipe = PipeService::get_screen_pipe(&pool, id).await?;
    Ok(ApiResponse::ok(pipe))
}

/// PUT `/api/v1/screen-pipes/{id}` — Update a screen pipe
///
/// Updates an existing screen pipe record. Validates request body. Returns 404 if not found.
pub async fn update_screen_pipe_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateScreenPipeRequest>,
) -> Result<Json<ApiResponse<ScreenPipe>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let pipe = PipeService::update_screen_pipe(&pool, id, &req).await?;
    Ok(ApiResponse::ok(pipe))
}

/// DELETE `/api/v1/screen-pipes/{id}` — Soft-delete a screen pipe
///
/// Soft-deletes a screen pipe record. Returns 404 if not found.
pub async fn delete_screen_pipe_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    PipeService::delete_screen_pipe(&pool, id).await?;
    Ok(ApiResponse::ok("Screen pipe deleted successfully".into()))
}

// ━━━ Search Handler ━━━

/// GET `/api/v1/pipes/search` — Search all pipes (seamless + screen)
///
/// Searches both seamless and screen pipes by keyword query `q`.
/// Searches across pipe number, heat number, grade, and other fields.
/// Returns 400 if the search query is empty.
pub async fn search_pipes_handler(
    Extension(pool): Extension<SqlitePool>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<ApiResponse<Vec<PipeSearchResult>>>, AppError> {
    if query.q.trim().is_empty() {
        return Err(AppError::Validation("Search query is required".into()));
    }
    let results = PipeService::search_pipes(&pool, &query.q).await?;
    Ok(ApiResponse::ok(results))
}
