use axum::{
    extract::{Extension, Query},
    Json,
};
use sqlx::SqlitePool;
use validator::Validate;

use crate::dto::inventory_dto::AtpItem;
use crate::dto::inventory_dto::AtpQuery;
use crate::error::AppError;
use crate::response::ApiResponse;
use crate::services::inventory_query_service::InventoryQueryService;

pub async fn check_atp_handler(
    Extension(pool): Extension<SqlitePool>,
    Query(query): Query<AtpQuery>,
) -> Result<Json<ApiResponse<Vec<AtpItem>>>, AppError> {
    query.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let items = InventoryQueryService::check_atp(&pool, &query).await?;
    Ok(ApiResponse::ok(items))
}
