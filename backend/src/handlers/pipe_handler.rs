use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use sqlx::SqlitePool;
use validator::Validate;

use crate::cache_invalidator::CacheInvalidator;
use crate::domain::pipe::PipeModel;
use crate::dto::common::PaginationParams;
use crate::dto::pipe_dto::{
    BatchCreatePipeRequest, CreateScreenPipeRequest, CreateSeamlessPipeRequest, PipeFilterParams,
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

/// Generic handler factory for pipe types implementing `PipeModel`.
///
/// This module provides reusable handler functions that can operate on any
/// pipe type (SeamlessPipe, ScreenPipe, WeldedPipe, etc.) without duplicating
/// the HTTP layer logic.
pub mod generic {
    use super::*;

    /// Generic handler for listing pipes of type P.
    pub async fn list_pipes_handler<P>(
        Extension(pool): Extension<SqlitePool>,
        Query(filter): Query<PipeFilterParams>,
    ) -> Result<Json<PaginatedResponse<P>>, AppError>
    where
        P: PipeModel + Send + Sync + 'static,
    {
        let pagination = PaginationParams {
            page: Some(filter.page.unwrap_or(1)),
            page_size: Some(filter.page_size.unwrap_or(10)),
            sort_by: filter.sort_by.clone(),
            sort_order: filter.sort_order.clone(),
        };
        let page = pagination.page();
        let page_size = pagination.page_size();

        let (items, total) = PipeService::list_pipes::<P>(&pool, &filter, &pagination).await?;

        Ok(PaginatedResponse::ok(items, total, page, page_size))
    }

    /// Generic handler for creating a pipe of type P.
    pub async fn create_pipe_handler<P>(
        Extension(pool): Extension<SqlitePool>,
        Extension(cache): Extension<CacheInvalidator>,
        Json(req): Json<P::CreateDto>,
    ) -> Result<axum::response::Response, AppError>
    where
        P: PipeModel + Send + Sync + 'static,
        P::CreateDto: Validate,
    {
        req.validate()
            .map_err(|e| AppError::Validation(e.to_string()))?;
        let pipe = PipeService::create_pipe::<P, _>(&pool, &cache, &req).await?;
        Ok(ApiResponse::created(pipe))
    }

    /// Generic handler for getting a pipe of type P by ID.
    pub async fn get_pipe_handler<P>(
        Extension(pool): Extension<SqlitePool>,
        Path(id): Path<i64>,
    ) -> Result<Json<ApiResponse<P>>, AppError>
    where
        P: PipeModel + Send + Sync + 'static,
    {
        let pipe = PipeService::get_pipe::<P>(&pool, id).await?;
        Ok(ApiResponse::ok(pipe))
    }

    /// Generic handler for updating a pipe of type P.
    pub async fn update_pipe_handler<P>(
        Extension(pool): Extension<SqlitePool>,
        Extension(cache): Extension<CacheInvalidator>,
        Path(id): Path<i64>,
        Json(req): Json<P::UpdateDto>,
    ) -> Result<Json<ApiResponse<P>>, AppError>
    where
        P: PipeModel + Send + Sync + 'static,
        P::UpdateDto: Validate,
    {
        req.validate()
            .map_err(|e| AppError::Validation(e.to_string()))?;
        let pipe = PipeService::update_pipe::<P, _>(&pool, &cache, id, &req).await?;
        Ok(ApiResponse::ok(pipe))
    }

    /// Generic handler for deleting a pipe of type P.
    pub async fn delete_pipe_handler<P>(
        Extension(pool): Extension<SqlitePool>,
        Extension(cache): Extension<CacheInvalidator>,
        Path(id): Path<i64>,
    ) -> Result<axum::response::Response, AppError>
    where
        P: PipeModel + Send + Sync + 'static,
    {
        PipeService::delete_pipe::<P, _>(&pool, &cache, id).await?;
        Ok((StatusCode::NO_CONTENT, ()).into_response())
    }
}

// ━━━ Seamless Pipe Handlers ━━━

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
        page: Some(filter.page.unwrap_or(1)),
        page_size: Some(filter.page_size.unwrap_or(10)),
        sort_by: filter.sort_by.clone(),
        sort_order: filter.sort_order.clone(),
    };
    let page = pagination.page();
    let page_size = pagination.page_size();

    let (items, total) = PipeService::list_seamless_pipes(&pool, &filter, &pagination).await?;

    Ok(PaginatedResponse::ok(items, total, page, page_size))
}

/// POST `/api/v1/seamless-pipes` — Whip up a new seamless pipe
///
/// Creates a new seamless pipe record with API 5CT specs (grade, heat treatment, threading, etc.).
/// Validates the request body. Warehouse/admin role required.
/// Returns 400 on validation error.
pub async fn create_seamless_pipe_handler(
    Extension(pool): Extension<SqlitePool>,
    Extension(cache): Extension<CacheInvalidator>,
    Json(req): Json<CreateSeamlessPipeRequest>,
) -> Result<axum::response::Response, AppError> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    let pipe = PipeService::create_seamless_pipe(&pool, &cache, &req).await?;
    Ok(ApiResponse::created(pipe))
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
    Extension(cache): Extension<CacheInvalidator>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateSeamlessPipeRequest>,
) -> Result<Json<ApiResponse<SeamlessPipe>>, AppError> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    let pipe = PipeService::update_seamless_pipe(&pool, &cache, id, &req).await?;
    Ok(ApiResponse::ok(pipe))
}

/// DELETE `/api/v1/seamless-pipes/{id}` — Soft-delete a seamless pipe
///
/// Soft-deletes a seamless pipe record. Returns 404 if not found.
pub async fn delete_seamless_pipe_handler(
    Extension(pool): Extension<SqlitePool>,
    Extension(cache): Extension<CacheInvalidator>,
    Path(id): Path<i64>,
) -> Result<axum::response::Response, AppError> {
    PipeService::delete_seamless_pipe(&pool, &cache, id).await?;
    Ok((StatusCode::NO_CONTENT, ()).into_response())
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
        page: Some(filter.page.unwrap_or(1)),
        page_size: Some(filter.page_size.unwrap_or(10)),
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
    Extension(cache): Extension<CacheInvalidator>,
    Json(req): Json<CreateScreenPipeRequest>,
) -> Result<axum::response::Response, AppError> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    let pipe = PipeService::create_screen_pipe(&pool, &cache, &req).await?;
    Ok(ApiResponse::created(pipe))
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
    Extension(cache): Extension<CacheInvalidator>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateScreenPipeRequest>,
) -> Result<Json<ApiResponse<ScreenPipe>>, AppError> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    let pipe = PipeService::update_screen_pipe(&pool, &cache, id, &req).await?;
    Ok(ApiResponse::ok(pipe))
}

/// DELETE `/api/v1/screen-pipes/{id}` — Soft-delete a screen pipe
///
/// Soft-deletes a screen pipe record. Returns 404 if not found.
pub async fn delete_screen_pipe_handler(
    Extension(pool): Extension<SqlitePool>,
    Extension(cache): Extension<CacheInvalidator>,
    Path(id): Path<i64>,
) -> Result<axum::response::Response, AppError> {
    PipeService::delete_screen_pipe(&pool, &cache, id).await?;
    Ok((StatusCode::NO_CONTENT, ()).into_response())
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

pub async fn batch_create_pipes_handler(
    Extension(pool): Extension<SqlitePool>,
    Extension(cache): Extension<CacheInvalidator>,
    Json(req): Json<BatchCreatePipeRequest>,
) -> Result<axum::response::Response, AppError> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    let pipe_ids = PipeService::batch_create_pipes(&pool, &cache, &req).await?;
    Ok(ApiResponse::created(pipe_ids))
}