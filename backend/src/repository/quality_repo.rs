use sqlx::SqlitePool;

use crate::domain::{Api5ctGradeRef, PipeAttachment, QualityCert};
use crate::error::{AppError, AppResult};

// ── QualityCertRepo ──

pub struct QualityCertRepo;

impl QualityCertRepo {
    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        pool: &SqlitePool,
        id: &str,
        cert_no: &str,
        pipe_type: &str,
        pipe_id: &str,
        inspect_date: &str,
        inspector: &str,
        agency: Option<&str>,
        result: &str,
        items_json: Option<&str>,
        notes: Option<&str>,
    ) -> AppResult<QualityCert> {
        sqlx::query_as::<_, QualityCert>(
            "INSERT INTO quality_certs (id, cert_no, pipe_type, pipe_id, inspect_date, inspector, agency, result, items_json, notes)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             RETURNING *",
        )
        .bind(id)
        .bind(cert_no)
        .bind(pipe_type)
        .bind(pipe_id)
        .bind(inspect_date)
        .bind(inspector)
        .bind(agency)
        .bind(result)
        .bind(items_json)
        .bind(notes)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn update(
        pool: &SqlitePool,
        id: &str,
        cert_no: Option<&str>,
        inspect_date: Option<&str>,
        inspector: Option<&str>,
        agency: Option<&str>,
        result: Option<&str>,
        items_json: Option<&str>,
        notes: Option<&str>,
    ) -> AppResult<QualityCert> {
        // Build SET clause dynamically — only set non-None fields
        let mut sets: Vec<String> = Vec::new();

        let mut cert_no_param: Option<String> = None;
        let mut inspect_date_param: Option<String> = None;
        let mut inspector_param: Option<String> = None;
        let mut agency_param: Option<String> = None;
        let mut result_param: Option<String> = None;
        let mut items_json_param: Option<String> = None;
        let mut notes_param: Option<String> = None;

        if let Some(v) = cert_no {
            cert_no_param = Some(v.to_string());
            sets.push("cert_no = ?".to_string());
        }
        if let Some(v) = inspect_date {
            inspect_date_param = Some(v.to_string());
            sets.push("inspect_date = ?".to_string());
        }
        if let Some(v) = inspector {
            inspector_param = Some(v.to_string());
            sets.push("inspector = ?".to_string());
        }
        if let Some(v) = agency {
            agency_param = Some(v.to_string());
            sets.push("agency = ?".to_string());
        }
        if let Some(v) = result {
            result_param = Some(v.to_string());
            sets.push("result = ?".to_string());
        }
        if let Some(v) = items_json {
            items_json_param = Some(v.to_string());
            sets.push("items_json = ?".to_string());
        }
        if let Some(v) = notes {
            notes_param = Some(v.to_string());
            sets.push("notes = ?".to_string());
        }

        if sets.is_empty() {
            return Err(AppError::BadRequest("No fields to update".into()));
        }

        sets.push("updated_at = datetime('now')".to_string());

        let sql = format!(
            "UPDATE quality_certs SET {} WHERE id = ? RETURNING *",
            sets.join(", ")
        );

        let mut query = sqlx::query_as::<_, QualityCert>(&sql);

        if let Some(v) = cert_no_param {
            query = query.bind(v);
        }
        if let Some(v) = inspect_date_param {
            query = query.bind(v);
        }
        if let Some(v) = inspector_param {
            query = query.bind(v);
        }
        if let Some(v) = agency_param {
            query = query.bind(v);
        }
        if let Some(v) = result_param {
            query = query.bind(v);
        }
        if let Some(v) = items_json_param {
            query = query.bind(v);
        }
        if let Some(v) = notes_param {
            query = query.bind(v);
        }

        query.bind(id).fetch_optional(pool).await.map_err(AppError::Database)?.ok_or_else(|| AppError::NotFound(format!("Quality cert {} not found", id)))
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> AppResult<QualityCert> {
        sqlx::query_as::<_, QualityCert>("SELECT * FROM quality_certs WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(AppError::Database)?
            .ok_or_else(|| AppError::NotFound(format!("Quality cert {} not found", id)))
    }

    pub async fn list(
        pool: &SqlitePool,
        pipe_type: Option<&str>,
        result: Option<&str>,
        date_from: Option<&str>,
        date_to: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> AppResult<(Vec<QualityCert>, i64)> {
        let mut conditions: Vec<String> = Vec::new();
        let mut pipe_type_param: Option<String> = None;
        let mut result_param: Option<String> = None;
        let mut date_from_param: Option<String> = None;
        let mut date_to_param: Option<String> = None;

        if let Some(v) = pipe_type {
            pipe_type_param = Some(v.to_string());
            conditions.push("qc.pipe_type = ?".to_string());
        }
        if let Some(v) = result {
            result_param = Some(v.to_string());
            conditions.push("qc.result = ?".to_string());
        }
        if let Some(v) = date_from {
            date_from_param = Some(v.to_string());
            conditions.push("qc.inspect_date >= ?".to_string());
        }
        if let Some(v) = date_to {
            date_to_param = Some(v.to_string());
            conditions.push("qc.inspect_date <= ?".to_string());
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        // Count
        let count_sql = format!("SELECT COUNT(*) as cnt FROM quality_certs qc {}", where_clause);
        let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);

        if let Some(v) = pipe_type_param.as_ref() {
            count_query = count_query.bind(v);
        }
        if let Some(v) = result_param.as_ref() {
            count_query = count_query.bind(v);
        }
        if let Some(v) = date_from_param.as_ref() {
            count_query = count_query.bind(v);
        }
        if let Some(v) = date_to_param.as_ref() {
            count_query = count_query.bind(v);
        }

        let total = count_query.fetch_one(pool).await.unwrap_or(0);

        // Data
        let offset = (page - 1) * page_size;
        let data_sql = format!(
            "SELECT qc.* FROM quality_certs qc {} ORDER BY qc.created_at DESC LIMIT ? OFFSET ?",
            where_clause
        );
        let mut data_query = sqlx::query_as::<_, QualityCert>(&data_sql);

        if let Some(v) = pipe_type_param.as_ref() {
            data_query = data_query.bind(v);
        }
        if let Some(v) = result_param.as_ref() {
            data_query = data_query.bind(v);
        }
        if let Some(v) = date_from_param.as_ref() {
            data_query = data_query.bind(v);
        }
        if let Some(v) = date_to_param.as_ref() {
            data_query = data_query.bind(v);
        }

        data_query = data_query.bind(page_size).bind(offset);

        let rows = data_query.fetch_all(pool).await.map_err(AppError::Database)?;

        Ok((rows, total))
    }

    /// Find certs for a specific pipe (identified by pipe_type + pipe_id)
    pub async fn find_by_pipe(pool: &SqlitePool, pipe_type: &str, pipe_id: &str) -> AppResult<Vec<QualityCert>> {
        let rows = sqlx::query_as::<_, QualityCert>(
            "SELECT * FROM quality_certs WHERE pipe_type = ? AND pipe_id = ? ORDER BY created_at DESC",
        )
        .bind(pipe_type)
        .bind(pipe_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)?;
        Ok(rows)
    }

    /// Find certs by heat number — joins quality_certs with seamless_pipes / screen_pipes
    pub async fn find_by_heat_number(
        pool: &SqlitePool,
        heat_no: &str,
    ) -> AppResult<Vec<(String, String, String, QualityCert)>> {
        use sqlx::Row;

        let map_row = |row: sqlx::sqlite::SqliteRow| -> (String, String, String, QualityCert) {
            (
                row.get("pipe_id"),
                row.get("pipe_number"),
                row.get("grade"),
                QualityCert {
                    id: row.get("cert_id"),
                    cert_no: row.get("cert_no"),
                    pipe_type: row.get("pipe_type"),
                    pipe_id: row.get("cert_pipe_id"),
                    inspect_date: row.get("inspect_date"),
                    inspector: row.get("inspector"),
                    agency: row.get("agency"),
                    result: row.get("result"),
                    items_json: row.get("items_json"),
                    notes: row.get("notes"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                },
            )
        };

        // Seamless pipes
        let seamless: Vec<(String, String, String, QualityCert)> = sqlx::query(
            "SELECT sp.id AS pipe_id, sp.pipe_number, sp.grade,
                    qc.id AS cert_id, qc.cert_no, qc.pipe_type, qc.pipe_id AS cert_pipe_id,
                    qc.inspect_date, qc.inspector, qc.agency, qc.result,
                    qc.items_json, qc.notes, qc.created_at, qc.updated_at
             FROM quality_certs qc
             INNER JOIN seamless_pipes sp ON sp.id = qc.pipe_id AND qc.pipe_type = 'seamless'
             WHERE sp.heat_number = ? AND sp.deleted_at IS NULL
             ORDER BY qc.created_at DESC",
        )
        .bind(heat_no)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)?
        .into_iter()
        .map(map_row)
        .collect();

        // Screen pipes
        let screen: Vec<(String, String, String, QualityCert)> = sqlx::query(
            "SELECT sp.id AS pipe_id, sp.pipe_number, sp.grade,
                    qc.id AS cert_id, qc.cert_no, qc.pipe_type, qc.pipe_id AS cert_pipe_id,
                    qc.inspect_date, qc.inspector, qc.agency, qc.result,
                    qc.items_json, qc.notes, qc.created_at, qc.updated_at
             FROM quality_certs qc
             INNER JOIN screen_pipes sp ON sp.id = qc.pipe_id AND qc.pipe_type = 'screen'
             WHERE sp.heat_number = ? AND sp.deleted_at IS NULL
             ORDER BY qc.created_at DESC",
        )
        .bind(heat_no)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)?
        .into_iter()
        .map(map_row)
        .collect();

        let mut results = seamless;
        results.extend(screen);
        Ok(results)
    }

    /// Find pipe info by pipe_number (either seamless or screen)
    pub async fn find_pipe_by_number(
        pool: &SqlitePool,
        pipe_no: &str,
    ) -> AppResult<Option<(String, String, String)>> {
        // Check seamless
        let sp: Option<(String, String, String)> = sqlx::query_as::<_, (String, String, String)>(
            "SELECT id, pipe_number, grade FROM seamless_pipes WHERE pipe_number = ? AND deleted_at IS NULL",
        )
        .bind(pipe_no)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?;

        if let Some(row) = sp {
            return Ok(Some(("seamless".to_string(), row.0, row.2)));
        }

        // Check screen
        let sc: Option<(String, String, String)> = sqlx::query_as::<_, (String, String, String)>(
            "SELECT id, pipe_number, grade FROM screen_pipes WHERE pipe_number = ? AND deleted_at IS NULL",
        )
        .bind(pipe_no)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?;

        if let Some(row) = sc {
            return Ok(Some(("screen".to_string(), row.0, row.2)));
        }

        Ok(None)
    }

    /// Check if a pipe exists by type and id
    pub async fn pipe_exists(pool: &SqlitePool, pipe_type: &str, pipe_id: &str) -> AppResult<bool> {
        let table = match pipe_type {
            "seamless" => "seamless_pipes",
            "screen" => "screen_pipes",
            _ => return Err(AppError::BadRequest(format!("Invalid pipe_type: {}", pipe_type))),
        };
        let sql = format!("SELECT COUNT(*) FROM {} WHERE id = ? AND deleted_at IS NULL", table);
        let count: i64 = sqlx::query_scalar(&sql)
            .bind(pipe_id)
            .fetch_one(pool)
            .await
            .map_err(AppError::Database)?;
        Ok(count > 0)
    }
}

// ── GradeRefRepo ──

pub struct GradeRefRepo;

impl GradeRefRepo {
    pub async fn get_by_grade(pool: &SqlitePool, grade: &str) -> AppResult<Api5ctGradeRef> {
        sqlx::query_as::<_, Api5ctGradeRef>(
            "SELECT * FROM api_5ct_grade_ref WHERE grade = ?",
        )
        .bind(grade)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound(format!("Grade reference '{}' not found", grade)))
    }

    pub async fn list_all(pool: &SqlitePool) -> AppResult<Vec<Api5ctGradeRef>> {
        let rows = sqlx::query_as::<_, Api5ctGradeRef>(
            "SELECT * FROM api_5ct_grade_ref ORDER BY grade",
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)?;
        Ok(rows)
    }
}

// ── AttachmentRepo ──

pub struct AttachmentRepo;

impl AttachmentRepo {
    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        pool: &SqlitePool,
        id: &str,
        pipe_type: &str,
        pipe_id: &str,
        file_name: &str,
        file_path: &str,
        file_size: i64,
        mime_type: &str,
        uploaded_by: &str,
    ) -> AppResult<PipeAttachment> {
        sqlx::query_as::<_, PipeAttachment>(
            "INSERT INTO pipe_attachments (id, pipe_type, pipe_id, file_name, file_path, file_size, mime_type, uploaded_by)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)
             RETURNING *",
        )
        .bind(id)
        .bind(pipe_type)
        .bind(pipe_id)
        .bind(file_name)
        .bind(file_path)
        .bind(file_size)
        .bind(mime_type)
        .bind(uploaded_by)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)
    }

    pub async fn delete(pool: &SqlitePool, id: &str) -> AppResult<PipeAttachment> {
        sqlx::query_as::<_, PipeAttachment>(
            "DELETE FROM pipe_attachments WHERE id = ? RETURNING *",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound(format!("Attachment {} not found", id)))
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> AppResult<PipeAttachment> {
        sqlx::query_as::<_, PipeAttachment>(
            "SELECT * FROM pipe_attachments WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound(format!("Attachment {} not found", id)))
    }

    pub async fn find_by_pipe(pool: &SqlitePool, pipe_type: &str, pipe_id: &str) -> AppResult<Vec<PipeAttachment>> {
        let rows = sqlx::query_as::<_, PipeAttachment>(
            "SELECT * FROM pipe_attachments WHERE pipe_type = ? AND pipe_id = ? ORDER BY created_at DESC",
        )
        .bind(pipe_type)
        .bind(pipe_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)?;
        Ok(rows)
    }
}
