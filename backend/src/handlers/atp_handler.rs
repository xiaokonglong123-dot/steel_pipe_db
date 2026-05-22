use axum::{
    extract::{Extension, Query},
    Json,
};
use sqlx::SqlitePool;

use crate::dto::inventory_dto::AtpItem;
use crate::dto::inventory_dto::AtpQuery;
use crate::error::AppError;
use crate::response::ApiResponse;
use crate::services::inventory_service::InventoryService;

pub async fn check_atp_handler(
    Extension(pool): Extension<SqlitePool>,
    Query(query): Query<AtpQuery>,
) -> Result<Json<ApiResponse<Vec<AtpItem>>>, AppError> {
    let items = InventoryService::check_atp(&pool, &query).await?;
    Ok(ApiResponse::ok(items))
}
