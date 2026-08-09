//! Item (商品) HTTP handlers — /api/v1/items CRUD + SKU search.

use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sqlx::SqlitePool;
use validator::Validate;

use crate::dto::item_dto::{CreateItemRequest, ItemFilter, ItemSkuQuery, UpdateItemRequest};
use crate::error::AppError;
use crate::models::item::Item;
use crate::response::{ApiResponse, PaginatedResponse};
use crate::items::item_service::ItemService;

pub async fn list_items_handler(
    Extension(pool): Extension<SqlitePool>,
    Query(filter): Query<ItemFilter>,
) -> Result<Json<PaginatedResponse<Item>>, AppError> {
    let page = filter.page.unwrap_or(1).max(1);
    let page_size = filter.page_size.unwrap_or(20).clamp(1, 100);

    let (items, total) = ItemService::list_items(&pool, &filter).await?;
    Ok(PaginatedResponse::ok(items, total, page, page_size))
}

pub async fn create_item_handler(
    Extension(pool): Extension<SqlitePool>,
    Json(req): Json<CreateItemRequest>,
) -> Result<axum::response::Response, AppError> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    let item = ItemService::create_item(&pool, &req).await?;
    Ok(ApiResponse::created(item))
}

pub async fn get_item_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<Item>>, AppError> {
    let item = ItemService::get_item(&pool, id).await?;
    Ok(ApiResponse::ok(item))
}

pub async fn update_item_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateItemRequest>,
) -> Result<Json<ApiResponse<Item>>, AppError> {
    let item = ItemService::update_item(&pool, id, &req).await?;
    Ok(ApiResponse::ok(item))
}

pub async fn delete_item_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<axum::response::Response, AppError> {
    ItemService::delete_item(&pool, id).await?;
    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

/// GET /api/v1/items/search?sku=… — partial SKU match.
pub async fn search_items_handler(
    Extension(pool): Extension<SqlitePool>,
    Query(query): Query<ItemSkuQuery>,
) -> Result<Json<ApiResponse<Vec<Item>>>, AppError> {
    if query.sku.trim().is_empty() {
        return Err(AppError::Validation("SKU query is required".into()));
    }
    let items = ItemService::search_by_sku(&pool, &query).await?;
    Ok(ApiResponse::ok(items))
}
