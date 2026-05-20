use sqlx::SqlitePool;
use crate::domain::{SeamlessPipe, ScreenPipe};
use crate::error::{AppResult, AppError};

// ─── Allowlist for ORDER BY columns (SQL injection prevention) ───────────────

const ALLOWED_SORT_COLUMNS: &[&str] = &[
    "created_at", "pipe_number", "grade", "status", "od", "wt", "length", "weight",
];

fn validate_sort_col(sort_by: &str) -> &'static str {
    ALLOWED_SORT_COLUMNS
        .iter()
        .find(|col| **col == sort_by)
        .copied()
        .unwrap_or("created_at")
}

// ─── SeamlessPipeRepo ─────────────────────────────────────────────────────────

pub struct SeamlessPipeRepo {
    pool: SqlitePool,
}

impl SeamlessPipeRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, pipe: &SeamlessPipe) -> AppResult<SeamlessPipe> {
        let result = sqlx::query_as::<_, SeamlessPipe>(
            "INSERT INTO seamless_pipes \
             (id, pipe_number, grade, od, wt, length, weight, \
              connection_type, heat_number, production_date, \
              status, location, notes, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15) \
             RETURNING *",
        )
        .bind(&pipe.id)
        .bind(&pipe.pipe_number)
        .bind(&pipe.grade)
        .bind(pipe.od)
        .bind(pipe.wt)
        .bind(pipe.length)
        .bind(pipe.weight)
        .bind(&pipe.connection_type)
        .bind(&pipe.heat_number)
        .bind(&pipe.production_date)
        .bind(&pipe.status)
        .bind(&pipe.location)
        .bind(&pipe.notes)
        .bind(&pipe.created_at)
        .bind(&pipe.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(AppError::from)?;
        Ok(result)
    }

    pub async fn find_by_id(&self, id: &str) -> AppResult<SeamlessPipe> {
        sqlx::query_as::<_, SeamlessPipe>(
            "SELECT * FROM seamless_pipes WHERE id = ?1 AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(format!("Seamless pipe {} not found", id)))
    }

    pub async fn find_by_pipe_number(&self, pipe_number: &str) -> AppResult<Option<SeamlessPipe>> {
        sqlx::query_as::<_, SeamlessPipe>(
            "SELECT * FROM seamless_pipes WHERE pipe_number = ?1 AND deleted_at IS NULL",
        )
        .bind(pipe_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::from)
    }

    pub async fn update(&self, id: &str, pipe: &SeamlessPipe) -> AppResult<SeamlessPipe> {
        let rows = sqlx::query_as::<_, SeamlessPipe>(
            "UPDATE seamless_pipes SET \
             pipe_number = ?1, grade = ?2, od = ?3, wt = ?4, \
             length = ?5, weight = ?6, \
             connection_type = ?7, heat_number = ?8, production_date = ?9, \
             status = ?10, location = ?11, notes = ?12, \
             updated_at = datetime('now') \
             WHERE id = ?13 AND deleted_at IS NULL \
             RETURNING *",
        )
        .bind(&pipe.pipe_number)
        .bind(&pipe.grade)
        .bind(pipe.od)
        .bind(pipe.wt)
        .bind(pipe.length)
        .bind(pipe.weight)
        .bind(&pipe.connection_type)
        .bind(&pipe.heat_number)
        .bind(&pipe.production_date)
        .bind(&pipe.status)
        .bind(&pipe.location)
        .bind(&pipe.notes)
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::from)?;

        rows.ok_or_else(|| AppError::NotFound(format!("Seamless pipe {} not found", id)))
    }

    pub async fn soft_delete(&self, id: &str) -> AppResult<()> {
        let result = sqlx::query(
            "UPDATE seamless_pipes SET deleted_at = datetime('now') \
             WHERE id = ?1 AND deleted_at IS NULL",
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(AppError::from)?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("Seamless pipe {} not found", id)));
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn list(
        &self,
        grade: Option<&str>,
        status: Option<&str>,
        search: Option<&str>,
        page: i64,
        page_size: i64,
        sort_by: &str,
        sort_order: &str,
    ) -> AppResult<Vec<SeamlessPipe>> {
        let sort_col = validate_sort_col(sort_by);
        let order = if sort_order.eq_ignore_ascii_case("asc") {
            "ASC"
        } else {
            "DESC"
        };
        let offset = (page - 1) * page_size;

        let sql = format!(
            "SELECT * FROM seamless_pipes \
             WHERE deleted_at IS NULL \
               AND (?1 IS NULL OR grade = ?1) \
               AND (?2 IS NULL OR status = ?2) \
               AND (?3 IS NULL OR pipe_number LIKE '%' || ?3 || '%') \
             ORDER BY {} {} \
             LIMIT ?4 OFFSET ?5",
            sort_col, order
        );

        sqlx::query_as::<_, SeamlessPipe>(&sql)
            .bind(grade)
            .bind(status)
            .bind(search)
            .bind(page_size)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(AppError::from)
    }

    pub async fn count(
        &self,
        grade: Option<&str>,
        status: Option<&str>,
        search: Option<&str>,
    ) -> AppResult<i64> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM seamless_pipes \
             WHERE deleted_at IS NULL \
               AND (?1 IS NULL OR grade = ?1) \
               AND (?2 IS NULL OR status = ?2) \
               AND (?3 IS NULL OR pipe_number LIKE '%' || ?3 || '%')",
        )
        .bind(grade)
        .bind(status)
        .bind(search)
        .fetch_one(&self.pool)
        .await
        .map_err(AppError::from)?;
        Ok(count)
    }
}

// ─── ScreenPipeRepo ───────────────────────────────────────────────────────────

pub struct ScreenPipeRepo {
    pool: SqlitePool,
}

impl ScreenPipeRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, pipe: &ScreenPipe) -> AppResult<ScreenPipe> {
        let result = sqlx::query_as::<_, ScreenPipe>(
            "INSERT INTO screen_pipes \
             (id, pipe_number, grade, od, wt, length, weight, \
              screen_type, slot_width, open_area, \
              connection_type, heat_number, production_date, \
              status, location, notes, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18) \
             RETURNING *",
        )
        .bind(&pipe.id)
        .bind(&pipe.pipe_number)
        .bind(&pipe.grade)
        .bind(pipe.od)
        .bind(pipe.wt)
        .bind(pipe.length)
        .bind(pipe.weight)
        .bind(&pipe.screen_type)
        .bind(pipe.slot_width)
        .bind(pipe.open_area)
        .bind(&pipe.connection_type)
        .bind(&pipe.heat_number)
        .bind(&pipe.production_date)
        .bind(&pipe.status)
        .bind(&pipe.location)
        .bind(&pipe.notes)
        .bind(&pipe.created_at)
        .bind(&pipe.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(AppError::from)?;
        Ok(result)
    }

    pub async fn find_by_id(&self, id: &str) -> AppResult<ScreenPipe> {
        sqlx::query_as::<_, ScreenPipe>(
            "SELECT * FROM screen_pipes WHERE id = ?1 AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(format!("Screen pipe {} not found", id)))
    }

    pub async fn find_by_pipe_number(&self, pipe_number: &str) -> AppResult<Option<ScreenPipe>> {
        sqlx::query_as::<_, ScreenPipe>(
            "SELECT * FROM screen_pipes WHERE pipe_number = ?1 AND deleted_at IS NULL",
        )
        .bind(pipe_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::from)
    }

    pub async fn update(&self, id: &str, pipe: &ScreenPipe) -> AppResult<ScreenPipe> {
        let rows = sqlx::query_as::<_, ScreenPipe>(
            "UPDATE screen_pipes SET \
             pipe_number = ?1, grade = ?2, od = ?3, wt = ?4, \
             length = ?5, weight = ?6, \
             screen_type = ?7, slot_width = ?8, open_area = ?9, \
             connection_type = ?10, heat_number = ?11, production_date = ?12, \
             status = ?13, location = ?14, notes = ?15, \
             updated_at = datetime('now') \
             WHERE id = ?16 AND deleted_at IS NULL \
             RETURNING *",
        )
        .bind(&pipe.pipe_number)
        .bind(&pipe.grade)
        .bind(pipe.od)
        .bind(pipe.wt)
        .bind(pipe.length)
        .bind(pipe.weight)
        .bind(&pipe.screen_type)
        .bind(pipe.slot_width)
        .bind(pipe.open_area)
        .bind(&pipe.connection_type)
        .bind(&pipe.heat_number)
        .bind(&pipe.production_date)
        .bind(&pipe.status)
        .bind(&pipe.location)
        .bind(&pipe.notes)
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::from)?;

        rows.ok_or_else(|| AppError::NotFound(format!("Screen pipe {} not found", id)))
    }

    pub async fn soft_delete(&self, id: &str) -> AppResult<()> {
        let result = sqlx::query(
            "UPDATE screen_pipes SET deleted_at = datetime('now') \
             WHERE id = ?1 AND deleted_at IS NULL",
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(AppError::from)?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("Screen pipe {} not found", id)));
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn list(
        &self,
        grade: Option<&str>,
        status: Option<&str>,
        search: Option<&str>,
        page: i64,
        page_size: i64,
        sort_by: &str,
        sort_order: &str,
    ) -> AppResult<Vec<ScreenPipe>> {
        let sort_col = validate_sort_col(sort_by);
        let order = if sort_order.eq_ignore_ascii_case("asc") {
            "ASC"
        } else {
            "DESC"
        };
        let offset = (page - 1) * page_size;

        let sql = format!(
            "SELECT * FROM screen_pipes \
             WHERE deleted_at IS NULL \
               AND (?1 IS NULL OR grade = ?1) \
               AND (?2 IS NULL OR status = ?2) \
               AND (?3 IS NULL OR pipe_number LIKE '%' || ?3 || '%') \
             ORDER BY {} {} \
             LIMIT ?4 OFFSET ?5",
            sort_col, order
        );

        sqlx::query_as::<_, ScreenPipe>(&sql)
            .bind(grade)
            .bind(status)
            .bind(search)
            .bind(page_size)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(AppError::from)
    }

    pub async fn count(
        &self,
        grade: Option<&str>,
        status: Option<&str>,
        search: Option<&str>,
    ) -> AppResult<i64> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM screen_pipes \
             WHERE deleted_at IS NULL \
               AND (?1 IS NULL OR grade = ?1) \
               AND (?2 IS NULL OR status = ?2) \
               AND (?3 IS NULL OR pipe_number LIKE '%' || ?3 || '%')",
        )
        .bind(grade)
        .bind(status)
        .bind(search)
        .fetch_one(&self.pool)
        .await
        .map_err(AppError::from)?;
        Ok(count)
    }
}
