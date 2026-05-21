// 报表入口：库存汇总、订单统计、质量统计、驾驶舱看板

use axum::extract::{Extension, Query};
use axum::Json;
use sqlx::SqlitePool;

use crate::dto::report_dto::OrderReportQuery;
use crate::error::AppError;
use crate::response::ApiResponse;
use crate::services::report_service::ReportService;

pub async fn inventory_summary_handler(
    Extension(pool): Extension<SqlitePool>,
) -> Result<Json<serde_json::Value>, AppError> {
    let data = ReportService::inventory_summary(&pool).await?;
    Ok(Json(serde_json::json!({ "success": true, "data": data })))
}

pub async fn order_report_handler(
    Extension(pool): Extension<SqlitePool>,
    Query(query): Query<OrderReportQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let order_type = query.r#type.as_deref().unwrap_or("purchase");
    let period = query.period.as_deref().unwrap_or("monthly");

    // 前端传参校验：只允许 purchase/sales 两种订单类型
    // period 只支持月/季/年三种统计粒度
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

pub async fn quality_report_handler(
    Extension(pool): Extension<SqlitePool>,
) -> Result<Json<serde_json::Value>, AppError> {
    let data = ReportService::quality_report(&pool).await?;
    Ok(Json(serde_json::json!({ "success": true, "data": data })))
}

pub async fn dashboard_handler(
    Extension(pool): Extension<SqlitePool>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let data = ReportService::dashboard(&pool).await?;
    Ok(ApiResponse::ok(data))
}
