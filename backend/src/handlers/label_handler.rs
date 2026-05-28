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

/// GET `/api/v1/labels/pipe/{pipe_type}/{pipe_id}` — Generate pipe label
///
/// Generates an HTML label for a specific pipe by type and ID.
pub async fn get_pipe_label_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(path): Path<PipeLabelPath>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    let html = LabelService::generate_pipe_label(&pool, &path.pipe_type, path.pipe_id).await?;
    Ok(ApiResponse::ok(html))
}

/// POST `/api/v1/labels/batch` — Batch generate labels
///
/// Generates HTML labels for multiple pipes in batch.
/// Validates request body.
pub async fn create_batch_labels_handler(
    Extension(pool): Extension<SqlitePool>,
    Json(req): Json<BatchLabelRequest>,
) -> Result<axum::response::Response, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let html = LabelService::generate_batch_labels(&pool, &req).await?;
    Ok(ApiResponse::created(html))
}

/// GET `/api/v1/labels/quality/{cert_id}` — Generate QC inspection tag
///
/// Generates an HTML quality inspection tag for a certificate.
pub async fn get_quality_label_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(cert_id): Path<i64>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    let html = LabelService::generate_quality_tag(&pool, cert_id).await?;
    Ok(ApiResponse::ok(html))
}

/// POST `/api/v1/labels/shipping` — Generate shipping label
///
/// Generates an HTML shipping label with customer and order info.
/// Validates request body.
pub async fn create_shipping_label_handler(
    Extension(pool): Extension<SqlitePool>,
    Json(req): Json<ShippingLabelRequest>,
) -> Result<axum::response::Response, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let html = LabelService::generate_shipping_label(&pool, &req).await?;
    Ok(ApiResponse::created(html))
}
