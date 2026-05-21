use axum::{
    extract::{Extension, Path, Query},
    Json,
};
use serde::Deserialize;
use sqlx::SqlitePool;

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

pub async fn create_seamless_pipe_handler(
    Extension(pool): Extension<SqlitePool>,
    Json(req): Json<CreateSeamlessPipeRequest>,
) -> Result<Json<ApiResponse<SeamlessPipe>>, AppError> {
    let pipe = PipeService::create_seamless_pipe(&pool, &req).await?;
    Ok(ApiResponse::ok(pipe))
}

pub async fn get_seamless_pipe_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<SeamlessPipe>>, AppError> {
    let pipe = PipeService::get_seamless_pipe(&pool, id).await?;
    Ok(ApiResponse::ok(pipe))
}

pub async fn update_seamless_pipe_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateSeamlessPipeRequest>,
) -> Result<Json<ApiResponse<SeamlessPipe>>, AppError> {
    let pipe = PipeService::update_seamless_pipe(&pool, id, &req).await?;
    Ok(ApiResponse::ok(pipe))
}

pub async fn delete_seamless_pipe_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    PipeService::delete_seamless_pipe(&pool, id).await?;
    Ok(ApiResponse::ok("Seamless pipe deleted successfully".into()))
}

// ━━━ Screen Pipe Handlers ━━━

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

pub async fn create_screen_pipe_handler(
    Extension(pool): Extension<SqlitePool>,
    Json(req): Json<CreateScreenPipeRequest>,
) -> Result<Json<ApiResponse<ScreenPipe>>, AppError> {
    let pipe = PipeService::create_screen_pipe(&pool, &req).await?;
    Ok(ApiResponse::ok(pipe))
}

pub async fn get_screen_pipe_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<ScreenPipe>>, AppError> {
    let pipe = PipeService::get_screen_pipe(&pool, id).await?;
    Ok(ApiResponse::ok(pipe))
}

pub async fn update_screen_pipe_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateScreenPipeRequest>,
) -> Result<Json<ApiResponse<ScreenPipe>>, AppError> {
    let pipe = PipeService::update_screen_pipe(&pool, id, &req).await?;
    Ok(ApiResponse::ok(pipe))
}

pub async fn delete_screen_pipe_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    PipeService::delete_screen_pipe(&pool, id).await?;
    Ok(ApiResponse::ok("Screen pipe deleted successfully".into()))
}

// ━━━ Search Handler ━━━

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
