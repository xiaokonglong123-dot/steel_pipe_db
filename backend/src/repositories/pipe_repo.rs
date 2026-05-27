use sqlx::{QueryBuilder, Sqlite, SqlitePool};

use crate::dto::common::PaginationParams;
use crate::dto::pipe_dto::{
    CreateScreenPipeRequest, CreateSeamlessPipeRequest, PipeFilterParams,
    UpdateScreenPipeRequest, UpdateSeamlessPipeRequest,
};
use crate::models::screen_pipe::ScreenPipe;
use crate::models::seamless_pipe::SeamlessPipe;

/// CRUD for `seamless_pipes` table. All queries filter `deleted_at IS NULL`.
pub struct SeamlessPipeRepo;

impl SeamlessPipeRepo {
    /// INSERT into `seamless_pipes`. Returns the newly created row with generated `id`.
    pub async fn create(
        pool: &SqlitePool,
        dto: &CreateSeamlessPipeRequest,
    ) -> Result<SeamlessPipe, sqlx::Error> {
        sqlx::query_as::<_, SeamlessPipe>(
            "INSERT INTO seamless_pipes (pipe_number, batch_number, pipe_type, grade, od, wt, \
             length, weight_per_unit, end_type, coupling_type, coupling_od, coupling_length, \
             heat_number, serial_number, manufacturer, production_date, cert_number, \
             location_id, notes) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) \
             RETURNING id, pipe_number, batch_number, pipe_type, grade, od, wt, length, \
               weight_per_unit, end_type, coupling_type, coupling_od, coupling_length, \
               heat_number, serial_number, manufacturer, production_date, cert_number, \
               location_id, status, notes, created_at, updated_at, deleted_at",
        )
        .bind(&dto.pipe_number)
        .bind(&dto.batch_number)
        .bind(&dto.pipe_type)
        .bind(&dto.grade)
        .bind(dto.od)
        .bind(dto.wt)
        .bind(dto.length)
        .bind(dto.weight_per_unit)
        .bind(&dto.end_type)
        .bind(&dto.coupling_type)
        .bind(dto.coupling_od)
        .bind(dto.coupling_length)
        .bind(&dto.heat_number)
        .bind(&dto.serial_number)
        .bind(&dto.manufacturer)
        .bind(&dto.production_date)
        .bind(&dto.cert_number)
        .bind(None::<i64>)
        .bind(&dto.notes)
        .fetch_one(pool)
        .await
    }

    /// UPDATE `seamless_pipes` by id. All non-None fields in `UpdateSeamlessPipeRequest` are
    /// conditionally set via `COALESCE`. Returns the updated row.
    pub async fn update(
        pool: &SqlitePool,
        id: i64,
        dto: &UpdateSeamlessPipeRequest,
    ) -> Result<SeamlessPipe, sqlx::Error> {
        let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new(
            "UPDATE seamless_pipes SET updated_at = datetime('now')",
        );

        if let Some(ref val) = dto.batch_number {
            builder.push(", batch_number = ");
            builder.push_bind(val);
        }
        if let Some(ref val) = dto.pipe_type {
            builder.push(", pipe_type = ");
            builder.push_bind(val);
        }
        if let Some(ref val) = dto.grade {
            builder.push(", grade = ");
            builder.push_bind(val);
        }
        if let Some(val) = dto.od {
            builder.push(", od = ");
            builder.push_bind(val);
        }
        if let Some(val) = dto.wt {
            builder.push(", wt = ");
            builder.push_bind(val);
        }
        if let Some(val) = dto.length {
            builder.push(", length = ");
            builder.push_bind(val);
        }
        if let Some(val) = dto.weight_per_unit {
            builder.push(", weight_per_unit = ");
            builder.push_bind(val);
        }
        if let Some(ref val) = dto.end_type {
            builder.push(", end_type = ");
            builder.push_bind(val);
        }
        if let Some(ref val) = dto.coupling_type {
            builder.push(", coupling_type = ");
            builder.push_bind(val);
        }
        if let Some(val) = dto.coupling_od {
            builder.push(", coupling_od = ");
            builder.push_bind(val);
        }
        if let Some(val) = dto.coupling_length {
            builder.push(", coupling_length = ");
            builder.push_bind(val);
        }
        if let Some(ref val) = dto.heat_number {
            builder.push(", heat_number = ");
            builder.push_bind(val);
        }
        if let Some(ref val) = dto.serial_number {
            builder.push(", serial_number = ");
            builder.push_bind(val);
        }
        if let Some(ref val) = dto.manufacturer {
            builder.push(", manufacturer = ");
            builder.push_bind(val);
        }
        if let Some(ref val) = dto.production_date {
            builder.push(", production_date = ");
            builder.push_bind(val);
        }
        if let Some(ref val) = dto.cert_number {
            builder.push(", cert_number = ");
            builder.push_bind(val);
        }
        if let Some(ref val) = dto.notes {
            builder.push(", notes = ");
            builder.push_bind(val);
        }

        builder.push(" WHERE id = ");
        builder.push_bind(id);
        builder.push(" AND deleted_at IS NULL RETURNING id, pipe_number, batch_number, \
            pipe_type, grade, od, wt, length, weight_per_unit, end_type, coupling_type, \
            coupling_od, coupling_length, heat_number, serial_number, manufacturer, \
            production_date, cert_number, location_id, status, notes, created_at, \
            updated_at, deleted_at");

        builder.build_query_as::<SeamlessPipe>().fetch_one(pool).await
    }

    /// SELECT by primary key from `seamless_pipes`. Returns `None` if not found or soft-deleted.
    pub async fn find_by_id(
        pool: &SqlitePool,
        id: i64,
    ) -> Result<Option<SeamlessPipe>, sqlx::Error> {
        sqlx::query_as::<_, SeamlessPipe>(
            "SELECT id, pipe_number, batch_number, pipe_type, grade, od, wt, length, \
             weight_per_unit, end_type, coupling_type, coupling_od, coupling_length, \
             heat_number, serial_number, manufacturer, production_date, cert_number, \
             location_id, status, notes, created_at, updated_at, deleted_at \
             FROM seamless_pipes WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    /// SELECT multiple rows by a list of primary keys. Returns only non-deleted rows.
    /// If `ids` is empty, returns an empty `Vec` without hitting the DB.
    pub async fn find_by_ids(
        pool: &SqlitePool,
        ids: &[i64],
    ) -> Result<Vec<SeamlessPipe>, sqlx::Error> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();
        let query = format!(
            "SELECT id, pipe_number, batch_number, pipe_type, grade, od, wt, length, \
             weight_per_unit, end_type, coupling_type, coupling_od, coupling_length, \
             heat_number, serial_number, manufacturer, production_date, cert_number, \
             location_id, status, notes, created_at, updated_at, deleted_at \
             FROM seamless_pipes WHERE id IN ({}) AND deleted_at IS NULL",
            placeholders.join(",")
        );
        let mut q = sqlx::query_as::<_, SeamlessPipe>(&query);
        for id in ids {
            q = q.bind(id);
        }
        q.fetch_all(pool).await
    }

    /// SELECT by unique `pipe_number`. Returns `None` if not found or soft-deleted.
    pub async fn find_by_pipe_number(
        pool: &SqlitePool,
        pipe_number: &str,
    ) -> Result<Option<SeamlessPipe>, sqlx::Error> {
        sqlx::query_as::<_, SeamlessPipe>(
            "SELECT id, pipe_number, batch_number, pipe_type, grade, od, wt, length, \
             weight_per_unit, end_type, coupling_type, coupling_od, coupling_length, \
             heat_number, serial_number, manufacturer, production_date, cert_number, \
             location_id, status, notes, created_at, updated_at, deleted_at \
             FROM seamless_pipes WHERE pipe_number = ? AND deleted_at IS NULL",
        )
        .bind(pipe_number)
        .fetch_optional(pool)
        .await
    }

    /// Soft-delete by setting `deleted_at` timestamp. No-op if already deleted.
    pub async fn delete(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE seamless_pipes SET deleted_at = datetime('now'), \
             updated_at = datetime('now') WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Paginated SELECT with optional filters (`pipe_type`, `grade`, `status`, keyword search
    /// across `pipe_number`/`batch_number`/`heat_number`/`serial_number`). Returns `(items, total)`.
    pub async fn list(
        pool: &SqlitePool,
        filter: &PipeFilterParams,
        params: &PaginationParams,
    ) -> Result<(Vec<SeamlessPipe>, u64), sqlx::Error> {
        let page_size = params.page_size();
        let offset = params.offset();

        // Build WHERE conditions
        let mut conditions: Vec<String> = vec!["deleted_at IS NULL".into()];
        let mut bind_values: Vec<String> = Vec::new();

        if let Some(ref q) = filter.q {
            if !q.is_empty() {
                conditions.push("(pipe_number LIKE ? OR batch_number LIKE ? OR grade LIKE ?)".into());
                let pattern = format!("%{}%", q);
                bind_values.push(pattern.clone());
                bind_values.push(pattern.clone());
                bind_values.push(pattern);
            }
        }
        if let Some(ref grade) = filter.grade {
            conditions.push("grade = ?".into());
            bind_values.push(grade.clone());
        }
        if let Some(ref pipe_type) = filter.pipe_type {
            conditions.push("pipe_type = ?".into());
            bind_values.push(pipe_type.clone());
        }
        if let Some(ref status) = filter.status {
            conditions.push("status = ?".into());
            bind_values.push(status.clone());
        }
        if let Some(od_min) = filter.od_min {
            conditions.push("od >= ?".into());
            bind_values.push(od_min.to_string());
        }
        if let Some(od_max) = filter.od_max {
            conditions.push("od <= ?".into());
            bind_values.push(od_max.to_string());
        }
        if let Some(wt_min) = filter.wt_min {
            conditions.push("wt >= ?".into());
            bind_values.push(wt_min.to_string());
        }
        if let Some(wt_max) = filter.wt_max {
            conditions.push("wt <= ?".into());
            bind_values.push(wt_max.to_string());
        }
        if let Some(location_id) = filter.location_id {
            conditions.push("location_id = ?".into());
            bind_values.push(location_id.to_string());
        }
        if let Some(ref manufacturer) = filter.manufacturer {
            conditions.push("manufacturer = ?".into());
            bind_values.push(manufacturer.clone());
        }

        let where_clause = conditions.join(" AND ");

        // Sort
        let sort_by = match params.sort_by.as_deref() {
            Some("pipe_number") => "pipe_number",
            Some("grade") => "grade",
            Some("od") => "od",
            Some("wt") => "wt",
            Some("status") => "status",
            Some("manufacturer") => "manufacturer",
            Some("production_date") => "production_date",
            _ => "created_at",
        };
        let sort_order = params.sort_order_sql();

        // Count
        let count_sql = format!(
            "SELECT COUNT(*) as cnt FROM seamless_pipes WHERE {}",
            where_clause
        );
        let mut count_q = sqlx::query_as::<_, (i64,)>(&count_sql);
        for val in &bind_values {
            count_q = count_q.bind(val.as_str());
        }
        let total: (i64,) = count_q.fetch_one(pool).await?;

        // Data
        let list_sql = format!(
            "SELECT id, pipe_number, batch_number, pipe_type, grade, od, wt, length, \
             weight_per_unit, end_type, coupling_type, coupling_od, coupling_length, \
             heat_number, serial_number, manufacturer, production_date, cert_number, \
             location_id, status, notes, created_at, updated_at, deleted_at \
             FROM seamless_pipes WHERE {} \
             ORDER BY {} {} LIMIT ? OFFSET ?",
            where_clause, sort_by, sort_order
        );
        let mut list_q = sqlx::query_as::<_, SeamlessPipe>(&list_sql);
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

    /// Full-text LIKE search across `pipe_number` and `batch_number`. Returns up to 50 results.
    pub async fn search(
        pool: &SqlitePool,
        query: &str,
    ) -> Result<Vec<SeamlessPipe>, sqlx::Error> {
        let like = format!("%{}%", query);
        sqlx::query_as::<_, SeamlessPipe>(
            "SELECT id, pipe_number, batch_number, pipe_type, grade, od, wt, length, \
             weight_per_unit, end_type, coupling_type, coupling_od, coupling_length, \
             heat_number, serial_number, manufacturer, production_date, cert_number, \
             location_id, status, notes, created_at, updated_at, deleted_at \
             FROM seamless_pipes \
             WHERE deleted_at IS NULL AND (pipe_number LIKE ? OR batch_number LIKE ?) \
             ORDER BY created_at DESC LIMIT 50",
        )
        .bind(&like)
        .bind(&like)
        .fetch_all(pool)
        .await
    }
}

/// CRUD for `screen_pipes` table. All queries filter `deleted_at IS NULL`.
pub struct ScreenPipeRepo;

impl ScreenPipeRepo {
    /// INSERT into `screen_pipes`. Returns the newly created row with generated `id`.
    pub async fn create(
        pool: &SqlitePool,
        dto: &CreateScreenPipeRequest,
    ) -> Result<ScreenPipe, sqlx::Error> {
        sqlx::query_as::<_, ScreenPipe>(
            "INSERT INTO screen_pipes (pipe_number, batch_number, screen_type, slot_size, \
             filtration_grade, base_od, base_wt, base_grade, base_end_type, length, \
             weight_per_unit, heat_number, serial_number, manufacturer, production_date, \
             cert_number, location_id, notes) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) \
             RETURNING id, pipe_number, batch_number, screen_type, slot_size, \
               filtration_grade, base_od, base_wt, base_grade, base_end_type, length, \
               weight_per_unit, heat_number, serial_number, manufacturer, production_date, \
               cert_number, location_id, status, notes, created_at, updated_at, deleted_at",
        )
        .bind(&dto.pipe_number)
        .bind(&dto.batch_number)
        .bind(&dto.screen_type)
        .bind(dto.slot_size)
        .bind(&dto.filtration_grade)
        .bind(dto.base_od)
        .bind(dto.base_wt)
        .bind(&dto.base_grade)
        .bind(&dto.base_end_type)
        .bind(dto.length)
        .bind(dto.weight_per_unit)
        .bind(&dto.heat_number)
        .bind(&dto.serial_number)
        .bind(&dto.manufacturer)
        .bind(&dto.production_date)
        .bind(&dto.cert_number)
        .bind(None::<i64>)
        .bind(&dto.notes)
        .fetch_one(pool)
        .await
    }

    /// UPDATE `screen_pipes` by id. All non-None fields in `UpdateScreenPipeRequest` are
    /// conditionally set via `QueryBuilder`. Returns the updated row.
    pub async fn update(
        pool: &SqlitePool,
        id: i64,
        dto: &UpdateScreenPipeRequest,
    ) -> Result<ScreenPipe, sqlx::Error> {
        let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new(
            "UPDATE screen_pipes SET updated_at = datetime('now')",
        );

        if let Some(ref val) = dto.batch_number {
            builder.push(", batch_number = ");
            builder.push_bind(val);
        }
        if let Some(ref val) = dto.screen_type {
            builder.push(", screen_type = ");
            builder.push_bind(val);
        }
        if let Some(val) = dto.slot_size {
            builder.push(", slot_size = ");
            builder.push_bind(val);
        }
        if let Some(ref val) = dto.filtration_grade {
            builder.push(", filtration_grade = ");
            builder.push_bind(val);
        }
        if let Some(val) = dto.base_od {
            builder.push(", base_od = ");
            builder.push_bind(val);
        }
        if let Some(val) = dto.base_wt {
            builder.push(", base_wt = ");
            builder.push_bind(val);
        }
        if let Some(ref val) = dto.base_grade {
            builder.push(", base_grade = ");
            builder.push_bind(val);
        }
        if let Some(ref val) = dto.base_end_type {
            builder.push(", base_end_type = ");
            builder.push_bind(val);
        }
        if let Some(val) = dto.length {
            builder.push(", length = ");
            builder.push_bind(val);
        }
        if let Some(val) = dto.weight_per_unit {
            builder.push(", weight_per_unit = ");
            builder.push_bind(val);
        }
        if let Some(ref val) = dto.heat_number {
            builder.push(", heat_number = ");
            builder.push_bind(val);
        }
        if let Some(ref val) = dto.serial_number {
            builder.push(", serial_number = ");
            builder.push_bind(val);
        }
        if let Some(ref val) = dto.manufacturer {
            builder.push(", manufacturer = ");
            builder.push_bind(val);
        }
        if let Some(ref val) = dto.production_date {
            builder.push(", production_date = ");
            builder.push_bind(val);
        }
        if let Some(ref val) = dto.cert_number {
            builder.push(", cert_number = ");
            builder.push_bind(val);
        }
        if let Some(ref val) = dto.notes {
            builder.push(", notes = ");
            builder.push_bind(val);
        }

        builder.push(" WHERE id = ");
        builder.push_bind(id);
        builder.push(" AND deleted_at IS NULL RETURNING id, pipe_number, batch_number, \
            screen_type, slot_size, filtration_grade, base_od, base_wt, base_grade, \
            base_end_type, length, weight_per_unit, heat_number, serial_number, \
            manufacturer, production_date, cert_number, location_id, status, notes, \
            created_at, updated_at, deleted_at");

        builder.build_query_as::<ScreenPipe>().fetch_one(pool).await
    }

    /// SELECT by primary key from `screen_pipes`. Returns `None` if not found or soft-deleted.
    pub async fn find_by_id(
        pool: &SqlitePool,
        id: i64,
    ) -> Result<Option<ScreenPipe>, sqlx::Error> {
        sqlx::query_as::<_, ScreenPipe>(
            "SELECT id, pipe_number, batch_number, screen_type, slot_size, \
             filtration_grade, base_od, base_wt, base_grade, base_end_type, length, \
             weight_per_unit, heat_number, serial_number, manufacturer, production_date, \
             cert_number, location_id, status, notes, created_at, updated_at, deleted_at \
             FROM screen_pipes WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    /// SELECT multiple rows by a list of primary keys. Returns only non-deleted rows.
    /// If `ids` is empty, returns an empty `Vec` without hitting the DB.
    pub async fn find_by_ids(
        pool: &SqlitePool,
        ids: &[i64],
    ) -> Result<Vec<ScreenPipe>, sqlx::Error> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();
        let query = format!(
            "SELECT id, pipe_number, batch_number, screen_type, slot_size, \
             filtration_grade, base_od, base_wt, base_grade, base_end_type, length, \
             weight_per_unit, heat_number, serial_number, manufacturer, production_date, \
             cert_number, location_id, status, notes, created_at, updated_at, deleted_at \
             FROM screen_pipes WHERE id IN ({}) AND deleted_at IS NULL",
            placeholders.join(",")
        );
        let mut q = sqlx::query_as::<_, ScreenPipe>(&query);
        for id in ids {
            q = q.bind(id);
        }
        q.fetch_all(pool).await
    }

    /// SELECT by unique `pipe_number`. Returns `None` if not found or soft-deleted.
    pub async fn find_by_pipe_number(
        pool: &SqlitePool,
        pipe_number: &str,
    ) -> Result<Option<ScreenPipe>, sqlx::Error> {
        sqlx::query_as::<_, ScreenPipe>(
            "SELECT id, pipe_number, batch_number, screen_type, slot_size, \
             filtration_grade, base_od, base_wt, base_grade, base_end_type, length, \
             weight_per_unit, heat_number, serial_number, manufacturer, production_date, \
             cert_number, location_id, status, notes, created_at, updated_at, deleted_at \
             FROM screen_pipes WHERE pipe_number = ? AND deleted_at IS NULL",
        )
        .bind(pipe_number)
        .fetch_optional(pool)
        .await
    }

    /// Soft-delete by setting `deleted_at` timestamp. No-op if already deleted.
    pub async fn delete(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE screen_pipes SET deleted_at = datetime('now'), \
             updated_at = datetime('now') WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Paginated SELECT with optional filters (`q`, `grade` → `base_grade`, `pipe_type` → `screen_type`,
    /// `status`, `od_min`/`od_max` → `base_od`, `wt_min`/`wt_max` → `base_wt`, `location_id`, `manufacturer`).
    /// Returns `(items, total)`.
    pub async fn list(
        pool: &SqlitePool,
        filter: &PipeFilterParams,
        params: &PaginationParams,
    ) -> Result<(Vec<ScreenPipe>, u64), sqlx::Error> {
        let page_size = params.page_size();
        let offset = params.offset();

        let mut conditions: Vec<String> = vec!["deleted_at IS NULL".into()];
        let mut bind_values: Vec<String> = Vec::new();

        if let Some(ref q) = filter.q {
            if !q.is_empty() {
                conditions.push("(pipe_number LIKE ? OR batch_number LIKE ? OR base_grade LIKE ?)".into());
                let pattern = format!("%{}%", q);
                bind_values.push(pattern.clone());
                bind_values.push(pattern.clone());
                bind_values.push(pattern);
            }
        }
        if let Some(ref grade) = filter.grade {
            // For screen pipes, grade filters base_grade
            conditions.push("base_grade = ?".into());
            bind_values.push(grade.clone());
        }
        if let Some(ref pipe_type) = filter.pipe_type {
            // For screen pipes, pipe_type filters screen_type
            conditions.push("screen_type = ?".into());
            bind_values.push(pipe_type.clone());
        }
        if let Some(ref status) = filter.status {
            conditions.push("status = ?".into());
            bind_values.push(status.clone());
        }
        if let Some(od_min) = filter.od_min {
            conditions.push("base_od >= ?".into());
            bind_values.push(od_min.to_string());
        }
        if let Some(od_max) = filter.od_max {
            conditions.push("base_od <= ?".into());
            bind_values.push(od_max.to_string());
        }
        if let Some(wt_min) = filter.wt_min {
            conditions.push("base_wt >= ?".into());
            bind_values.push(wt_min.to_string());
        }
        if let Some(wt_max) = filter.wt_max {
            conditions.push("base_wt <= ?".into());
            bind_values.push(wt_max.to_string());
        }
        if let Some(location_id) = filter.location_id {
            conditions.push("location_id = ?".into());
            bind_values.push(location_id.to_string());
        }
        if let Some(ref manufacturer) = filter.manufacturer {
            conditions.push("manufacturer = ?".into());
            bind_values.push(manufacturer.clone());
        }

        let where_clause = conditions.join(" AND ");

        let sort_by = match params.sort_by.as_deref() {
            Some("pipe_number") => "pipe_number",
            Some("base_grade") => "base_grade",
            Some("base_od") => "base_od",
            Some("base_wt") => "base_wt",
            Some("status") => "status",
            Some("manufacturer") => "manufacturer",
            Some("production_date") => "production_date",
            _ => "created_at",
        };
        let sort_order = params.sort_order_sql();

        let count_sql = format!(
            "SELECT COUNT(*) as cnt FROM screen_pipes WHERE {}",
            where_clause
        );
        let mut count_q = sqlx::query_as::<_, (i64,)>(&count_sql);
        for val in &bind_values {
            count_q = count_q.bind(val.as_str());
        }
        let total: (i64,) = count_q.fetch_one(pool).await?;

        let list_sql = format!(
            "SELECT id, pipe_number, batch_number, screen_type, slot_size, \
             filtration_grade, base_od, base_wt, base_grade, base_end_type, length, \
             weight_per_unit, heat_number, serial_number, manufacturer, production_date, \
             cert_number, location_id, status, notes, created_at, updated_at, deleted_at \
             FROM screen_pipes WHERE {} \
             ORDER BY {} {} LIMIT ? OFFSET ?",
            where_clause, sort_by, sort_order
        );
        let mut list_q = sqlx::query_as::<_, ScreenPipe>(&list_sql);
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

    /// Full-text LIKE search across `pipe_number` and `batch_number`. Returns up to 50 results.
    pub async fn search(
        pool: &SqlitePool,
        query: &str,
    ) -> Result<Vec<ScreenPipe>, sqlx::Error> {
        let like = format!("%{}%", query);
        sqlx::query_as::<_, ScreenPipe>(
            "SELECT id, pipe_number, batch_number, screen_type, slot_size, \
             filtration_grade, base_od, base_wt, base_grade, base_end_type, length, \
             weight_per_unit, heat_number, serial_number, manufacturer, production_date, \
             cert_number, location_id, status, notes, created_at, updated_at, deleted_at \
             FROM screen_pipes \
             WHERE deleted_at IS NULL AND (pipe_number LIKE ? OR batch_number LIKE ?) \
             ORDER BY created_at DESC LIMIT 50",
        )
        .bind(&like)
        .bind(&like)
        .fetch_all(pool)
        .await
    }
}
