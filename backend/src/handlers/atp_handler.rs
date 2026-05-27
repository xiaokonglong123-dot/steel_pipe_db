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

/// GET `/api/v1/atp/check` — ATP inventory availability check
///
/// Checks available-to-promise inventory for a set of pipe specs.
/// Used before sales order approval to ensure sufficient stock.
pub async fn check_atp_handler(
    Extension(pool): Extension<SqlitePool>,
    Query(query): Query<AtpQuery>,
) -> Result<Json<ApiResponse<Vec<AtpItem>>>, AppError> {
    query.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let items = InventoryQueryService::check_atp(&pool, &query).await?;
    Ok(ApiResponse::ok(items))
}
