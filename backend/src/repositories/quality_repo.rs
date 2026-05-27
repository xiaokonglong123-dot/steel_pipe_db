use sqlx::{QueryBuilder, Sqlite, SqlitePool};

use crate::dto::common::PaginationParams;
use crate::dto::quality_dto::{
    CreateAttachmentRequest, CreateQualityCertRequest, QualityCertFilterParams,
    UpdateQualityCertRequest,
};
use crate::models::quality::{Api5ctGradeRef, PipeAttachment, QualityCert};

/// CRUD for `quality_certs`. All queries filter `deleted_at IS NULL`.
pub struct QualityCertRepo;

impl QualityCertRepo {
    /// INSERT a new quality certificate. Returns the created `QualityCert`.
    pub async fn create(
        pool: &SqlitePool,
        dto: &CreateQualityCertRequest,
    ) -> Result<QualityCert, sqlx::Error> {
        sqlx::query_as::<_, QualityCert>(
            "INSERT INTO quality_certs (cert_number, pipe_type, pipe_id, cert_date, result, \
             inspector, inspection_body, notes) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?) \
             RETURNING id, cert_number, pipe_type, pipe_id, cert_date, result, inspector, \
               inspection_body, notes, created_at, updated_at, deleted_at",
        )
        .bind(&dto.cert_number)
        .bind(&dto.pipe_type)
        .bind(dto.pipe_id)
        .bind(&dto.cert_date)
        .bind(&dto.result)
        .bind(&dto.inspector)
        .bind(&dto.inspection_body)
        .bind(&dto.notes)
        .fetch_one(pool)
        .await
    }

    /// Dynamic UPDATE of cert fields (cert_date, result, inspector, etc.). Only supplied fields change.
    /// Returns the updated `QualityCert`.
    pub async fn update(
        pool: &SqlitePool,
        id: i64,
        dto: &UpdateQualityCertRequest,
    ) -> Result<QualityCert, sqlx::Error> {
        let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new(
            "UPDATE quality_certs SET updated_at = datetime('now')",
        );

        if let Some(ref val) = dto.cert_date {
            builder.push(", cert_date = ");
            builder.push_bind(val);
        }
        if let Some(ref val) = dto.result {
            builder.push(", result = ");
            builder.push_bind(val);
        }
        if let Some(ref val) = dto.inspector {
            builder.push(", inspector = ");
            builder.push_bind(val);
        }
        if let Some(ref val) = dto.inspection_body {
            builder.push(", inspection_body = ");
            builder.push_bind(val);
        }
        if let Some(ref val) = dto.notes {
            builder.push(", notes = ");
            builder.push_bind(val);
        }

        builder.push(" WHERE id = ");
        builder.push_bind(id);
        builder.push(" AND deleted_at IS NULL");
        builder.push(" RETURNING id, cert_number, pipe_type, pipe_id, cert_date, result, \
            inspector, inspection_body, notes, created_at, updated_at, deleted_at");

        builder.build_query_as::<QualityCert>().fetch_one(pool).await
    }

    /// SELECT by primary key. Returns `None` if soft-deleted or missing.
    pub async fn find_by_id(
        pool: &SqlitePool,
        id: i64,
    ) -> Result<Option<QualityCert>, sqlx::Error> {
        sqlx::query_as::<_, QualityCert>(
            "SELECT id, cert_number, pipe_type, pipe_id, cert_date, result, inspector, \
             inspection_body, notes, created_at, updated_at, deleted_at \
             FROM quality_certs WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    /// Soft-delete: sets `deleted_at` and `updated_at`.
    pub async fn delete(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE quality_certs SET deleted_at = datetime('now'), \
             updated_at = datetime('now') WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Paginated SELECT with dynamic filters (pipe_type, pipe_id, result).
    /// Supports sorting by cert_number, pipe_type, result, cert_date, inspector. Returns `(items, total)`.
    pub async fn list(
        pool: &SqlitePool,
        filter: &QualityCertFilterParams,
        params: &PaginationParams,
    ) -> Result<(Vec<QualityCert>, u64), sqlx::Error> {
        let page_size = params.page_size();
        let offset = params.offset();

        let mut conditions: Vec<String> = vec!["deleted_at IS NULL".into()];
        let mut bind_values: Vec<String> = Vec::new();

        if let Some(ref pipe_type) = filter.pipe_type {
            conditions.push("pipe_type = ?".into());
            bind_values.push(pipe_type.clone());
        }
        if let Some(pipe_id) = filter.pipe_id {
            conditions.push("pipe_id = ?".into());
            bind_values.push(pipe_id.to_string());
        }
        if let Some(ref result) = filter.result {
            conditions.push("result = ?".into());
            bind_values.push(result.clone());
        }

        let where_clause = conditions.join(" AND ");

        let sort_by = match params.sort_by.as_deref() {
            Some("cert_number") => "cert_number",
            Some("pipe_type") => "pipe_type",
            Some("result") => "result",
            Some("cert_date") => "cert_date",
            Some("inspector") => "inspector",
            _ => "created_at",
        };
        let sort_order = params.sort_order_sql();

        let count_sql = format!(
            "SELECT COUNT(*) as cnt FROM quality_certs WHERE {}",
            where_clause
        );
        let mut count_q = sqlx::query_as::<_, (i64,)>(&count_sql);
        for val in &bind_values {
            count_q = count_q.bind(val.as_str());
        }
        let total: (i64,) = count_q.fetch_one(pool).await?;

        let list_sql = format!(
            "SELECT id, cert_number, pipe_type, pipe_id, cert_date, result, inspector, \
             inspection_body, notes, created_at, updated_at, deleted_at \
             FROM quality_certs WHERE {} \
             ORDER BY {} {} LIMIT ? OFFSET ?",
            where_clause, sort_by, sort_order
        );
        let mut list_q = sqlx::query_as::<_, QualityCert>(&list_sql);
        for val in &bind_values {
            list_q = list_q.bind(val.as_str());
        }
        let items = list_q
            .bind(page_size as i64)
            .bind(offset as i64)
            .fetch_all(pool)
            .await?;

        Ok((items, total.0 as u64))
    }
}

/// Reference data: API 5CT grade mechanical properties (read-only).
pub struct Api5ctGradeRefRepo;

impl Api5ctGradeRefRepo {
    /// SELECT a grade by its name. Returns `None` if not found.
    pub async fn find_by_grade(
        pool: &SqlitePool,
        grade: &str,
    ) -> Result<Option<Api5ctGradeRef>, sqlx::Error> {
        sqlx::query_as::<_, Api5ctGradeRef>(
            "SELECT id, grade, yield_strength_min, yield_strength_max, tensile_strength_min, \
             hardness_max, carbon_content_max, manganese_content_max, phosphorus_content_max, \
             sulfur_content_max, notes \
             FROM api_5ct_grade_ref WHERE grade = ?",
        )
        .bind(grade)
        .fetch_optional(pool)
        .await
    }

    /// SELECT all grades ordered by `grade` name.
    pub async fn list_all(
        pool: &SqlitePool,
    ) -> Result<Vec<Api5ctGradeRef>, sqlx::Error> {
        sqlx::query_as::<_, Api5ctGradeRef>(
            "SELECT id, grade, yield_strength_min, yield_strength_max, tensile_strength_min, \
             hardness_max, carbon_content_max, manganese_content_max, phosphorus_content_max, \
             sulfur_content_max, notes \
             FROM api_5ct_grade_ref ORDER BY grade",
        )
        .fetch_all(pool)
        .await
    }
}

/// CRUD for `pipe_attachments` (file metadata).
pub struct PipeAttachmentRepo;

impl PipeAttachmentRepo {
    /// INSERT a new attachment record. Returns the created `PipeAttachment`.
    pub async fn create(
        pool: &SqlitePool,
        dto: &CreateAttachmentRequest,
    ) -> Result<PipeAttachment, sqlx::Error> {
        sqlx::query_as::<_, PipeAttachment>(
            "INSERT INTO pipe_attachments (pipe_type, pipe_id, file_name, file_path, \
             file_size, content_type, uploaded_by) \
             VALUES (?, ?, ?, ?, ?, ?, ?) \
             RETURNING id, pipe_type, pipe_id, file_name, file_path, file_size, content_type, \
               uploaded_by, created_at",
        )
        .bind(&dto.pipe_type)
        .bind(dto.pipe_id)
        .bind(&dto.file_name)
        .bind(&dto.file_path)
        .bind(dto.file_size)
        .bind(&dto.content_type)
        .bind(dto.uploaded_by)
        .fetch_one(pool)
        .await
    }

    /// SELECT by primary key. Returns `None` if not found.
    pub async fn find_by_id(
        pool: &SqlitePool,
        id: i64,
    ) -> Result<Option<PipeAttachment>, sqlx::Error> {
        sqlx::query_as::<_, PipeAttachment>(
            "SELECT id, pipe_type, pipe_id, file_name, file_path, file_size, content_type, \
             uploaded_by, created_at \
             FROM pipe_attachments WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    /// Hard DELETE from `pipe_attachments`.
    pub async fn delete(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM pipe_attachments WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// SELECT attachments for a pipe (by `pipe_type` + `pipe_id`), newest first.
    pub async fn list_by_pipe(
        pool: &SqlitePool,
        pipe_type: &str,
        pipe_id: i64,
    ) -> Result<Vec<PipeAttachment>, sqlx::Error> {
        sqlx::query_as::<_, PipeAttachment>(
            "SELECT id, pipe_type, pipe_id, file_name, file_path, file_size, content_type, \
             uploaded_by, created_at \
             FROM pipe_attachments WHERE pipe_type = ? AND pipe_id = ? \
             ORDER BY created_at DESC",
        )
        .bind(pipe_type)
        .bind(pipe_id)
        .fetch_all(pool)
        .await
    }
}
