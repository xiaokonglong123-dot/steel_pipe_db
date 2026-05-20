use sqlx::SqlitePool;
use uuid::Uuid;

use crate::domain::quality::{CreateQualityCertDto, QualityCertFilter, TraceResult, UpdateQualityCertDto};
use crate::domain::{Api5ctGradeRef, PipeAttachment, QualityCert};
use crate::error::{AppError, AppResult};
use crate::repository::quality_repo::{AttachmentRepo, GradeRefRepo, QualityCertRepo};

pub struct QualityService;

impl QualityService {
    pub async fn create_cert(pool: &SqlitePool, dto: CreateQualityCertDto) -> AppResult<QualityCert> {
        let pipe_type = dto.pipe_type.trim().to_lowercase();
        if pipe_type != "seamless" && pipe_type != "screen" {
            return Err(AppError::BadRequest("pipe_type must be 'seamless' or 'screen'".into()));
        }

        let result = dto.result.as_deref().unwrap_or("pending");
        if !["pass", "fail", "pending"].contains(&result) {
            return Err(AppError::BadRequest("result must be 'pass', 'fail', or 'pending'".into()));
        }

        let pipe_exists = QualityCertRepo::pipe_exists(pool, &pipe_type, &dto.pipe_id).await?;
        if !pipe_exists {
            return Err(AppError::NotFound(format!(
                "{} pipe with id '{}' not found",
                pipe_type, dto.pipe_id
            )));
        }

        let id = Uuid::new_v4().to_string();
        let inspect_date = dto.inspect_date.unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%d").to_string());
        let inspector = dto.inspector.unwrap_or_default();

        QualityCertRepo::create(
            pool,
            &id,
            &dto.cert_no,
            &pipe_type,
            &dto.pipe_id,
            &inspect_date,
            &inspector,
            dto.agency.as_deref(),
            result,
            dto.items_json.as_deref(),
            dto.notes.as_deref(),
        )
        .await
    }

    pub async fn update_cert(pool: &SqlitePool, id: &str, dto: UpdateQualityCertDto) -> AppResult<QualityCert> {
        QualityCertRepo::find_by_id(pool, id).await?;

        if let Some(ref result) = dto.result {
            if !["pass", "fail", "pending"].contains(&result.as_str()) {
                return Err(AppError::BadRequest("result must be 'pass', 'fail', or 'pending'".into()));
            }
        }

        QualityCertRepo::update(
            pool,
            id,
            dto.cert_no.as_deref(),
            dto.inspect_date.as_deref(),
            dto.inspector.as_deref(),
            dto.agency.as_deref(),
            dto.result.as_deref(),
            dto.items_json.as_deref(),
            dto.notes.as_deref(),
        )
        .await
    }

    pub async fn get_cert(pool: &SqlitePool, id: &str) -> AppResult<QualityCert> {
        QualityCertRepo::find_by_id(pool, id).await
    }

    pub async fn list_certs(pool: &SqlitePool, filter: QualityCertFilter) -> AppResult<(Vec<QualityCert>, i64)> {
        let page = filter.page.unwrap_or(1).max(1);
        let page_size = filter.page_size.unwrap_or(20).clamp(1, 100);

        QualityCertRepo::list(
            pool,
            filter.pipe_type.as_deref(),
            filter.result.as_deref(),
            filter.date_from.as_deref(),
            filter.date_to.as_deref(),
            page,
            page_size,
        )
        .await
    }

    pub async fn trace_by_heat_number(pool: &SqlitePool, heat_no: &str) -> AppResult<Vec<TraceResult>> {
        let raw = QualityCertRepo::find_by_heat_number(pool, heat_no).await?;

        // Group by pipe
        let mut grouped: Vec<TraceResult> = Vec::new();
        for (pipe_id, pipe_number, grade, cert) in raw {
            if let Some(entry) = grouped.iter_mut().find(|e| e.pipe_id == pipe_id) {
                entry.certs.push(cert);
            } else {
                grouped.push(TraceResult {
                    pipe_type: String::new(), // filled below with first cert's pipe_type
                    pipe_id,
                    pipe_number,
                    grade,
                    heat_number: Some(heat_no.to_string()),
                    certs: vec![cert],
                });
            }
        }
        // Fill pipe_type from cert if available
        for entry in grouped.iter_mut() {
            if let Some(first_cert) = entry.certs.first() {
                entry.pipe_type = first_cert.pipe_type.clone();
            }
        }
        Ok(grouped)
    }

    pub async fn trace_by_pipe_number(pool: &SqlitePool, pipe_no: &str) -> AppResult<Vec<TraceResult>> {
        let pipe_info = QualityCertRepo::find_pipe_by_number(pool, pipe_no).await?;

        let (pipe_type, pipe_id, grade) = match pipe_info {
            Some(info) => info,
            None => return Ok(Vec::new()),
        };

        let certs = QualityCertRepo::find_by_pipe(pool, &pipe_type, &pipe_id).await?;

        Ok(vec![TraceResult {
            pipe_type,
            pipe_id,
            pipe_number: pipe_no.to_string(),
            grade,
            heat_number: None,
            certs,
        }])
    }

    pub async fn get_api_5ct_reference(pool: &SqlitePool, grade: &str) -> AppResult<Api5ctGradeRef> {
        GradeRefRepo::get_by_grade(pool, grade).await
    }

    pub async fn list_api_5ct_grades(pool: &SqlitePool) -> AppResult<Vec<Api5ctGradeRef>> {
        GradeRefRepo::list_all(pool).await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn attach_file(
        pool: &SqlitePool,
        pipe_type: &str,
        pipe_id: &str,
        file_name: &str,
        file_path: &str,
        file_size: i64,
        mime_type: &str,
        uploaded_by: &str,
    ) -> AppResult<PipeAttachment> {
        let pt = pipe_type.trim().to_lowercase();
        if pt != "seamless" && pt != "screen" {
            return Err(AppError::BadRequest("pipe_type must be 'seamless' or 'screen'".into()));
        }

        let pipe_exists = QualityCertRepo::pipe_exists(pool, &pt, pipe_id).await?;
        if !pipe_exists {
            return Err(AppError::NotFound(format!("{} pipe with id '{}' not found", pt, pipe_id)));
        }

        let id = Uuid::new_v4().to_string();
        AttachmentRepo::create(pool, &id, &pt, pipe_id, file_name, file_path, file_size, mime_type, uploaded_by).await
    }

    pub async fn delete_attachment(pool: &SqlitePool, id: &str) -> AppResult<PipeAttachment> {
        let attachment = AttachmentRepo::find_by_id(pool, id).await?;

        if let Err(e) = tokio::fs::remove_file(&attachment.file_path).await {
            tracing::warn!("Failed to delete attachment file '{}': {}", attachment.file_path, e);
        }

        AttachmentRepo::delete(pool, id).await
    }

    pub async fn find_attachments_by_pipe(
        pool: &SqlitePool,
        pipe_type: &str,
        pipe_id: &str,
    ) -> AppResult<Vec<PipeAttachment>> {
        AttachmentRepo::find_by_pipe(pool, pipe_type, pipe_id).await
    }
}
