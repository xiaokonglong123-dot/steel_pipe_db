use sqlx::PgPool;

use crate::dto::common::PaginationParams;
use crate::dto::quality_dto::{
    CreateAttachmentRequest, CreateQualityCertRequest, QualityCertFilterParams,
    UpdateQualityCertRequest,
};
use crate::error::AppError;
use crate::models::quality::{Api5ctGradeRef, PipeAttachment, QualityCert};
use crate::repositories::quality_repo::{Api5ctGradeRefRepo, PipeAttachmentRepo, QualityCertRepo};

fn validate_result(result: &str) -> Result<(), AppError> {
    match result {
        "pass" | "fail" | "pending" => Ok(()),
        _ => Err(AppError::Validation(format!(
            "Invalid result '{}'. Must be 'pass', 'fail', or 'pending'",
            result
        ))),
    }
}

/// Quality service — manages API 5CT pipe QC certs (CRUD), read-only API 5CT grade
/// reference queries, and pipe attachment upload/delete.
pub struct QualityService;

impl QualityService {
    fn generate_cert_number(pipe_type: &str, id: i64) -> String {
        format!("QC-{}-{}", pipe_type, id)
    }

    // ━━━ Quality Cert ━━━

    /// Create a QC cert — auto-generates cert number (`QC-{pipe_type}-{id}`).
    /// Validates that `result` is `pass`, `fail`, or `pending`.
    ///
    /// # Errors
    /// - `AppError::Validation` — invalid result value
    /// - `AppError::Database` — DB write failure
    pub async fn create_cert(
        pool: &PgPool,
        dto: &CreateQualityCertRequest,
    ) -> Result<QualityCert, AppError> {
        let result = dto.result.as_deref().unwrap_or("pending");
        validate_result(result)?;

        // Reject duplicate: one active cert per (pipe_type, pipe_id).
        if QualityCertRepo::find_active_by_pipe(pool, &dto.pipe_type, dto.pipe_id)
            .await?
            .is_some()
        {
            return Err(AppError::BadRequest(format!(
                "Quality cert already exists for pipe_type={} pipe_id={}",
                dto.pipe_type, dto.pipe_id
            )));
        }

        let placeholder = format!("tmp-{}", uuid::Uuid::new_v4());

        let adjusted = CreateQualityCertRequest {
            pipe_type: dto.pipe_type.clone(),
            pipe_id: dto.pipe_id,
            cert_date: dto.cert_date.clone(),
            result: Some(result.to_string()),
            inspector: dto.inspector.clone(),
            inspection_body: dto.inspection_body.clone(),
            notes: dto.notes.clone(),
            cert_number: Some(placeholder),
        };

        let cert = QualityCertRepo::create(pool, &adjusted).await?;

        let cert_number = Self::generate_cert_number(&cert.pipe_type, cert.id);

        QualityCertRepo::update_cert_number(pool, cert.id, &cert_number)
            .await
            .map_err(AppError::from)?;

        QualityCertRepo::find_by_id(pool, cert.id)
            .await?
            .ok_or_else(|| AppError::Internal("Failed to retrieve created cert".into()))
    }

    /// Update a QC cert — validates `result` if provided.
    ///
    /// # Errors
    /// - `AppError::QualityCertNotFound` — ID not found or deleted
    /// - `AppError::Validation` — invalid result value
    pub async fn update_cert(
        pool: &PgPool,
        id: i64,
        dto: &UpdateQualityCertRequest,
    ) -> Result<QualityCert, AppError> {
        QualityCertRepo::find_by_id(pool, id)
            .await?
            .ok_or_else(|| {
                AppError::QualityCertNotFound(format!("Quality cert id={} not found", id))
            })?;

        if let Some(ref result) = dto.result {
            validate_result(result)?;
        }

        QualityCertRepo::update(pool, id, dto)
            .await
            .map_err(AppError::from)
    }

    /// Soft-delete a QC cert. Idempotent check prevents double deletion.
    ///
    /// # Errors
    /// - `AppError::QualityCertNotFound` — ID not found or already deleted
    pub async fn delete_cert(pool: &PgPool, id: i64) -> Result<(), AppError> {
        let existing = QualityCertRepo::find_by_id(pool, id)
            .await?
            .ok_or_else(|| {
                AppError::QualityCertNotFound(format!("Quality cert id={} not found", id))
            })?;

        if existing.deleted_at.is_some() {
            return Err(AppError::QualityCertNotFound(format!(
                "Quality cert id={} has been deleted",
                id
            )));
        }

        QualityCertRepo::delete(pool, id)
            .await
            .map_err(AppError::from)
    }

    /// Get a QC cert by ID.
    ///
    /// # Errors
    /// - `AppError::QualityCertNotFound` — ID not found or deleted
    pub async fn get_cert(pool: &PgPool, id: i64) -> Result<QualityCert, AppError> {
        QualityCertRepo::find_by_id(pool, id).await?.ok_or_else(|| {
            AppError::QualityCertNotFound(format!("Quality cert id={} not found", id))
        })
    }

    /// Paginated QC cert listing with filters for pipe type, result, date range, etc.
    pub async fn list_certs(
        pool: &PgPool,
        filter: &QualityCertFilterParams,
        params: &PaginationParams,
    ) -> Result<(Vec<QualityCert>, u64), AppError> {
        QualityCertRepo::list(pool, filter, params)
            .await
            .map_err(AppError::from)
    }

    // ━━━ API 5CT Grade Ref (read-only) ━━━

    /// Query API 5CT reference data by grade code.
    ///
    /// # Errors
    /// - `AppError::NotFound` — grade not found
    pub async fn get_grade(pool: &PgPool, grade: &str) -> Result<Api5ctGradeRef, AppError> {
        Api5ctGradeRefRepo::find_by_grade(pool, grade)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Grade '{}' not found", grade)))
    }

    /// List all API 5CT grade reference data (read-only).
    pub async fn list_grades(pool: &PgPool) -> Result<Vec<Api5ctGradeRef>, AppError> {
        Api5ctGradeRefRepo::list_all(pool)
            .await
            .map_err(AppError::from)
    }

    // ━━━ Pipe Attachment ━━━

    /// Create a pipe attachment record — saves file metadata (type, path, description) to DB.
    pub async fn create_attachment(
        pool: &PgPool,
        dto: &CreateAttachmentRequest,
    ) -> Result<PipeAttachment, AppError> {
        PipeAttachmentRepo::create(pool, dto)
            .await
            .map_err(AppError::from)
    }

    /// Soft-delete an attachment record.
    ///
    /// # Errors
    /// - `AppError::AttachmentNotFound` — ID not found or deleted
    pub async fn delete_attachment(pool: &PgPool, id: i64) -> Result<(), AppError> {
        PipeAttachmentRepo::find_by_id(pool, id)
            .await?
            .ok_or_else(|| {
                AppError::AttachmentNotFound(format!("Attachment id={} not found", id))
            })?;

        PipeAttachmentRepo::delete(pool, id)
            .await
            .map_err(AppError::from)
    }

    /// List all attachments for a pipe by type and ID.
    pub async fn list_attachments(
        pool: &PgPool,
        pipe_type: &str,
        pipe_id: i64,
    ) -> Result<Vec<PipeAttachment>, AppError> {
        PipeAttachmentRepo::list_by_pipe(pool, pipe_type, pipe_id)
            .await
            .map_err(AppError::from)
    }
}
