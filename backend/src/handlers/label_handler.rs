// 标签打印入口：管子标签、质量合格证标签、发货标签
// 标签生成为 HTML 格式，前端直接打印

use axum::{
    extract::{Extension, Path},
    Json,
};
use serde::Deserialize;
use sqlx::SqlitePool;

use validator::Validate;

use crate::dto::label_dto::{BatchLabelRequest, ShippingLabelRequest};
use crate::error::AppError;
use crate::response::ApiResponse;
use crate::services::label_service::LabelService;

#[derive(Deserialize)]
pub struct PipeLabelPath {
    pub pipe_type: String,
    pub pipe_id: i64,
}

pub async fn get_pipe_label_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(path): Path<PipeLabelPath>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    let html = LabelService::generate_pipe_label(&pool, &path.pipe_type, path.pipe_id).await?;
    Ok(ApiResponse::ok(html))
}

pub async fn create_batch_labels_handler(
    Extension(pool): Extension<SqlitePool>,
    Json(req): Json<BatchLabelRequest>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let html = LabelService::generate_batch_labels(&pool, &req).await?;
    Ok(ApiResponse::ok(html))
}

pub async fn get_quality_label_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(cert_id): Path<i64>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    let html = LabelService::generate_quality_tag(&pool, cert_id).await?;
    Ok(ApiResponse::ok(html))
}

pub async fn create_shipping_label_handler(
    Extension(pool): Extension<SqlitePool>,
    Json(req): Json<ShippingLabelRequest>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let html = LabelService::generate_shipping_label(&pool, &req).await?;
    Ok(ApiResponse::ok(html))
}
