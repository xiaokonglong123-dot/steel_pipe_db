use sqlx::SqlitePool;

use crate::domain::common::LabelTemplate;
use crate::domain::labels::{LabelTemplateDto, PrintLog, UpdateLabelTemplateDto};
use crate::error::{AppError, AppResult};

pub struct LabelTemplateRepo {
    pool: SqlitePool,
}

impl LabelTemplateRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, dto: &LabelTemplateDto) -> AppResult<LabelTemplate> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let config = dto
            .config_json
            .clone()
            .unwrap_or_else(|| "{}".to_string());

        let result = sqlx::query_as::<_, LabelTemplate>(
            "INSERT INTO label_templates (id, name, width_mm, height_mm, config_json, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7) RETURNING *",
        )
        .bind(&id)
        .bind(&dto.name)
        .bind(dto.width_mm)
        .bind(dto.height_mm)
        .bind(&config)
        .bind(&now)
        .bind(&now)
        .fetch_one(&self.pool)
        .await
        .map_err(AppError::from)?;
        Ok(result)
    }

    pub async fn find_by_id(&self, id: &str) -> AppResult<LabelTemplate> {
        sqlx::query_as::<_, LabelTemplate>(
            "SELECT * FROM label_templates WHERE id = ?1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(format!("Label template {} not found", id)))
    }

    pub async fn update(&self, id: &str, dto: &UpdateLabelTemplateDto) -> AppResult<LabelTemplate> {
        let existing = self.find_by_id(id).await?;

        let name = dto.name.clone().unwrap_or(existing.name);
        let width_mm = dto.width_mm.unwrap_or(existing.width_mm);
        let height_mm = dto.height_mm.unwrap_or(existing.height_mm);
        let config_json = dto
            .config_json
            .clone()
            .unwrap_or(existing.config_json);

        let result = sqlx::query_as::<_, LabelTemplate>(
            "UPDATE label_templates SET \
             name = ?1, width_mm = ?2, height_mm = ?3, config_json = ?4, \
             updated_at = datetime('now') \
             WHERE id = ?5 RETURNING *",
        )
        .bind(&name)
        .bind(width_mm)
        .bind(height_mm)
        .bind(&config_json)
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::from)?;

        result.ok_or_else(|| AppError::NotFound(format!("Label template {} not found", id)))
    }

    pub async fn delete(&self, id: &str) -> AppResult<()> {
        let result = sqlx::query("DELETE FROM label_templates WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(AppError::from)?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("Label template {} not found", id)));
        }
        Ok(())
    }

    pub async fn list(&self) -> AppResult<Vec<LabelTemplate>> {
        sqlx::query_as::<_, LabelTemplate>(
            "SELECT * FROM label_templates ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::from)
    }
}

pub struct PrintLogRepo {
    pool: SqlitePool,
}

impl PrintLogRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn insert(
        &self,
        template_id: &str,
        template_name: &str,
        pipe_numbers: &[String],
        total_labels: i64,
        printed_by: &str,
    ) -> AppResult<PrintLog> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let json = serde_json::to_string(pipe_numbers)
            .map_err(|e| AppError::Internal(format!("Serialize error: {}", e)))?;

        let result = sqlx::query_as::<_, PrintLog>(
            "INSERT INTO print_logs \
             (id, template_id, template_name, pipe_numbers_json, total_labels, printed_by, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7) RETURNING *",
        )
        .bind(&id)
        .bind(template_id)
        .bind(template_name)
        .bind(&json)
        .bind(total_labels)
        .bind(printed_by)
        .bind(&now)
        .fetch_one(&self.pool)
        .await
        .map_err(AppError::from)?;
        Ok(result)
    }

    pub async fn list(&self, page: i64, page_size: i64) -> AppResult<(Vec<PrintLog>, i64)> {
        let offset = (page - 1) * page_size;

        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM print_logs")
            .fetch_one(&self.pool)
            .await
            .map_err(AppError::from)?;

        let logs = sqlx::query_as::<_, PrintLog>(
            "SELECT * FROM print_logs ORDER BY created_at DESC LIMIT ?1 OFFSET ?2",
        )
        .bind(page_size)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::from)?;

        Ok((logs, total))
    }
}
