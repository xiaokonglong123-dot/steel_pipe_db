use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::cache::CacheManager;
use crate::dto::common::PaginationParams;
use crate::dto::inventory_dto::{CreateLocationRequest, UpdateLocationRequest};
use validator::Validate;

use crate::error::AppError;
use crate::models::inventory::Location;
use crate::response::{ApiResponse, PaginatedResponse};
use crate::inventory::location_service::LocationService;

#[derive(Deserialize)]
pub struct LocationListQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub active_only: Option<bool>,
}

pub async fn list_locations_handler(
    Extension(pool): Extension<SqlitePool>,
    Extension(cache): Extension<CacheManager>,
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
    // Cache key must include page/page_size: the cached payload is the result
    // of the *paginated* query (current page's items + total). Without the
    // pagination in the key, a page-2 request would serve page-1 data.
    let cache_key = format!(
        "locations:{}:p{}:s{}",
        if active_only { "active" } else { "all" },
        page,
        page_size
    );

    if let Some(cached_json) = cache.locations.get(&cache_key).await {
        if let Ok(cached) = serde_json::from_value::<(Vec<Location>, u64)>(cached_json) {
            let (items, total) = cached;
            return Ok(PaginatedResponse::ok(items, total, page, page_size));
        }
    }

    let (items, total) = LocationService::list_locations(&pool, &pagination, active_only).await?;
    cache.locations.insert(cache_key, serde_json::to_value((&items, total)).map_err(AppError::from)?).await;
    Ok(PaginatedResponse::ok(items, total, page, page_size))
}

pub async fn create_location_handler(
    Extension(pool): Extension<SqlitePool>,
    Extension(cache): Extension<CacheManager>,
    Json(req): Json<CreateLocationRequest>,
) -> Result<axum::response::Response, AppError> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    let location = LocationService::create_location(&pool, &cache, &req).await?;
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
    Extension(cache): Extension<CacheManager>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateLocationRequest>,
) -> Result<Json<ApiResponse<Location>>, AppError> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    let location = LocationService::update_location(&pool, &cache, id, &req).await?;
    Ok(ApiResponse::ok(location))
}

pub async fn delete_location_handler(
    Extension(pool): Extension<SqlitePool>,
    Extension(cache): Extension<CacheManager>,
    Path(id): Path<i64>,
) -> Result<axum::response::Response, AppError> {
    LocationService::delete_location(&pool, &cache, id).await?;
    Ok((StatusCode::NO_CONTENT, ()).into_response())
}
