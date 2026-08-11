//! Reports handlers — GET 报表 + ?format=csv 可选 CSV 导出

use axum::extract::{Extension, Query};
use axum::http::header;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;

use crate::error::AppError;
use crate::response::ApiResponse;
use crate::services::reports_service;
use sqlx::SqlitePool;

#[derive(Deserialize)]
pub struct ReportParams {
    pub item_id: Option<i64>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub months: Option<i64>,
    pub format: Option<String>,
}

pub async fn inventory_summary_report(
    Extension(pool): Extension<SqlitePool>,
    Query(p): Query<ReportParams>,
) -> Result<Response, AppError> {
    let rows = reports_service::inventory_summary(&pool).await?;
    if p.format.as_deref() == Some("csv") {
        let csv = reports_service::inventory_summary_csv(&rows);
        Ok(csv_response(csv, "inventory_summary.csv"))
    } else {
        Ok(Json(ApiResponse::ok(rows)).into_response())
    }
}

pub async fn inbound_outbound_report(
    Extension(pool): Extension<SqlitePool>,
    Query(p): Query<ReportParams>,
) -> Result<Response, AppError> {
    let rows = reports_service::inbound_outbound(
        &pool,
        p.item_id,
        p.start_date.as_deref(),
        p.end_date.as_deref(),
    )
    .await?;
    if p.format.as_deref() == Some("csv") {
        let csv = reports_service::inbound_outbound_csv(&rows);
        Ok(csv_response(csv, "inbound_outbound.csv"))
    } else {
        Ok(Json(ApiResponse::ok(rows)).into_response())
    }
}

pub async fn sales_trend_report(
    Extension(pool): Extension<SqlitePool>,
    Query(p): Query<ReportParams>,
) -> Result<Response, AppError> {
    let months = p.months.unwrap_or(6);
    let rows = reports_service::sales_trend(&pool, months).await?;
    if p.format.as_deref() == Some("csv") {
        let csv = reports_service::sales_trend_csv(&rows);
        Ok(csv_response(csv, "sales_trend.csv"))
    } else {
        Ok(Json(ApiResponse::ok(rows)).into_response())
    }
}

pub async fn finance_summary_report(
    Extension(pool): Extension<SqlitePool>,
    Query(p): Query<ReportParams>,
) -> Result<Response, AppError> {
    let rows = reports_service::finance_summary(&pool).await?;
    if p.format.as_deref() == Some("csv") {
        let csv = reports_service::finance_summary_csv(&rows);
        Ok(csv_response(csv, "finance_summary.csv"))
    } else {
        Ok(Json(ApiResponse::ok(rows)).into_response())
    }
}

fn csv_response(content: String, filename: &str) -> Response {
    use axum::http::HeaderValue;
    let mut resp = Response::new(content.into());
    let headers = resp.headers_mut();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("text/csv; charset=utf-8"));
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename={filename}"))
            .unwrap_or_else(|_| HeaderValue::from_static("attachment")),
    );
    resp
}
