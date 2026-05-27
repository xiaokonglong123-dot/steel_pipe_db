
use axum::extract::{Extension, Query};
use axum::Json;
use sqlx::SqlitePool;
use validator::Validate;

use crate::dto::report_dto::OrderReportQuery;
use crate::error::AppError;
use crate::response::ApiResponse;
use crate::services::report_service::ReportService;

/// GET `/api/v1/reports/inventory-summary` — Inventory summary report
///
/// Returns aggregated inventory data by pipe type, grade, and location.
pub async fn inventory_summary_handler(
    Extension(pool): Extension<SqlitePool>,
) -> Result<Json<serde_json::Value>, AppError> {
    let data = ReportService::inventory_summary(&pool).await?;
    Ok(Json(serde_json::json!({ "success": true, "data": data })))
}

/// GET `/api/v1/reports/orders` — Order report (purchase/sales)
///
/// Returns aggregated order data grouped by period (monthly/quarterly/yearly).
/// Query param `type` must be "purchase" or "sales".
pub async fn order_report_handler(
    Extension(pool): Extension<SqlitePool>,
    Query(query): Query<OrderReportQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    query.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let order_type = query.r#type.as_deref().unwrap_or("purchase");
    let period = query.period.as_deref().unwrap_or("monthly");

    if order_type != "purchase" && order_type != "sales" {
        return Err(AppError::Validation(
            "type must be 'purchase' or 'sales'".into(),
        ));
    }
    if !["monthly", "quarterly", "yearly"].contains(&period) {
        return Err(AppError::Validation(
            "period must be 'monthly', 'quarterly', or 'yearly'".into(),
        ));
    }

    let data = ReportService::order_report(&pool, order_type, period).await?;
    Ok(Json(serde_json::json!({ "success": true, "data": data })))
}

/// GET `/api/v1/reports/quality` — Quality inspection report
///
/// Returns aggregated quality inspection pass/fail statistics.
pub async fn quality_report_handler(
    Extension(pool): Extension<SqlitePool>,
) -> Result<Json<serde_json::Value>, AppError> {
    let data = ReportService::quality_report(&pool).await?;
    Ok(Json(serde_json::json!({ "success": true, "data": data })))
}

/// GET `/api/v1/reports/dashboard` — Dashboard key metrics
///
/// Returns key metrics for the main dashboard (inventory, orders, quality KPIs).
pub async fn dashboard_handler(
    Extension(pool): Extension<SqlitePool>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let data = ReportService::dashboard(&pool).await?;
    Ok(ApiResponse::ok(data))
}
