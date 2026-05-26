use axum::{
    extract::{Extension, Path, Query},
    Json,
};
use serde::Deserialize;
use sqlx::SqlitePool;

use validator::Validate;

use crate::dto::common::PaginationParams;
use crate::dto::quality_dto::{
    CreateAttachmentRequest, CreateQualityCertRequest, QualityCertFilterParams,
    UpdateQualityCertRequest,
};
use crate::error::AppError;
use crate::models::quality::{Api5ctGradeRef, PipeAttachment, QualityCert};
use crate::response::{ApiResponse, PaginatedResponse};
use crate::services::quality_service::QualityService;

#[derive(Deserialize, Default)]
pub struct AttachmentListQuery {
    pub pipe_type: Option<String>,
    pub pipe_id: Option<i64>,
    pub cert_id: Option<i64>,
}

#[derive(Deserialize)]
pub struct GradeQuery {
    pub grade: String,
}

// ━━━ Quality Cert Handlers ━━━

pub async fn list_certs_handler(
    Extension(pool): Extension<SqlitePool>,
    Query(filter): Query<QualityCertFilterParams>,
) -> Result<Json<PaginatedResponse<QualityCert>>, AppError> {
    let pagination = PaginationParams {
        page: filter.page,
        page_size: filter.page_size,
        sort_by: filter.sort_by.clone(),
        sort_order: filter.sort_order.clone(),
    };
    let page = pagination.page();
    let page_size = pagination.page_size();

    let (items, total) = QualityService::list_certs(&pool, &filter, &pagination).await?;

    Ok(PaginatedResponse::ok(items, total, page, page_size))
}

pub async fn create_cert_handler(
    Extension(pool): Extension<SqlitePool>,
    Json(req): Json<CreateQualityCertRequest>,
) -> Result<Json<ApiResponse<QualityCert>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let cert = QualityService::create_cert(&pool, &req).await?;
    Ok(ApiResponse::ok(cert))
}

pub async fn get_cert_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<QualityCert>>, AppError> {
    let cert = QualityService::get_cert(&pool, id).await?;
    Ok(ApiResponse::ok(cert))
}

pub async fn update_cert_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateQualityCertRequest>,
) -> Result<Json<ApiResponse<QualityCert>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let cert = QualityService::update_cert(&pool, id, &req).await?;
    Ok(ApiResponse::ok(cert))
}

pub async fn delete_cert_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    QualityService::delete_cert(&pool, id).await?;
    Ok(ApiResponse::ok("Quality cert deleted successfully".into()))
}

// ━━━ API 5CT Grade Ref Handlers ━━━

pub async fn get_grade_handler(
    Extension(pool): Extension<SqlitePool>,
    Query(query): Query<GradeQuery>,
) -> Result<Json<ApiResponse<Api5ctGradeRef>>, AppError> {
    let grade = QualityService::get_grade(&pool, &query.grade).await?;
    Ok(ApiResponse::ok(grade))
}

pub async fn list_grades_handler(
    Extension(pool): Extension<SqlitePool>,
) -> Result<Json<ApiResponse<Vec<Api5ctGradeRef>>>, AppError> {
    let grades = QualityService::list_grades(&pool).await?;
    Ok(ApiResponse::ok(grades))
}

// ━━━ Pipe Attachment Handlers ━━━

pub async fn create_attachment_handler(
    Extension(pool): Extension<SqlitePool>,
    Json(req): Json<CreateAttachmentRequest>,
) -> Result<Json<ApiResponse<PipeAttachment>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let attachment = QualityService::create_attachment(&pool, &req).await?;
    Ok(ApiResponse::ok(attachment))
}

pub async fn delete_attachment_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    QualityService::delete_attachment(&pool, id).await?;
    Ok(ApiResponse::ok("Attachment deleted successfully".into()))
}

pub async fn list_attachments_handler(
    Extension(pool): Extension<SqlitePool>,
    Query(query): Query<AttachmentListQuery>,
) -> Result<Json<ApiResponse<Vec<PipeAttachment>>>, AppError> {
    let (pipe_type, pipe_id) = if let Some(cert_id) = query.cert_id {
        let cert = QualityService::get_cert(&pool, cert_id).await?;
        (cert.pipe_type, cert.pipe_id)
    } else if let Some(ref pipe_type) = query.pipe_type {
        let pipe_id = query
            .pipe_id
            .ok_or_else(|| AppError::Validation("pipe_id is required when pipe_type is provided".into()))?;
        (pipe_type.clone(), pipe_id)
    } else {
        return Err(AppError::Validation(
            "Either cert_id or pipe_type+pipe_id must be provided".into(),
        ));
    };
    let attachments = QualityService::list_attachments(&pool, &pipe_type, pipe_id).await?;
    Ok(ApiResponse::ok(attachments))
}
