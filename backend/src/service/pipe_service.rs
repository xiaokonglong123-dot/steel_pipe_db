use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::config::AppConfig;
use crate::domain::{InboundRecord, OutboundRecord, SeamlessPipe, ScreenPipe};
use crate::error::{AppError, AppResult};
use crate::repository::pipe_repo::{SeamlessPipeRepo, ScreenPipeRepo};

// ─── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateSeamlessPipeDto {
    pub pipe_number: Option<String>,
    pub grade: String,
    pub od: f64,
    pub wt: f64,
    pub length: f64,
    pub weight: f64,
    pub connection_type: Option<String>,
    pub heat_number: Option<String>,
    pub production_date: Option<String>,
    pub status: Option<String>,
    pub location: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSeamlessPipeDto {
    pub pipe_number: String,
    pub grade: String,
    pub od: f64,
    pub wt: f64,
    pub length: f64,
    pub weight: f64,
    pub connection_type: Option<String>,
    pub heat_number: Option<String>,
    pub production_date: Option<String>,
    pub status: String,
    pub location: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateScreenPipeDto {
    pub pipe_number: Option<String>,
    pub grade: String,
    pub od: f64,
    pub wt: f64,
    pub length: f64,
    pub weight: f64,
    pub screen_type: String,
    pub slot_width: Option<f64>,
    pub open_area: Option<f64>,
    pub connection_type: Option<String>,
    pub heat_number: Option<String>,
    pub production_date: Option<String>,
    pub status: Option<String>,
    pub location: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateScreenPipeDto {
    pub pipe_number: String,
    pub grade: String,
    pub od: f64,
    pub wt: f64,
    pub length: f64,
    pub weight: f64,
    pub screen_type: String,
    pub slot_width: Option<f64>,
    pub open_area: Option<f64>,
    pub connection_type: Option<String>,
    pub heat_number: Option<String>,
    pub production_date: Option<String>,
    pub status: String,
    pub location: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct PipeListFilter {
    pub grade: Option<String>,
    pub status: Option<String>,
    pub search: Option<String>,
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_page_size")]
    pub page_size: i64,
    #[serde(default = "default_sort_by")]
    pub sort_by: String,
    #[serde(default = "default_sort_order")]
    pub sort_order: String,
}

fn default_page() -> i64 {
    1
}
fn default_page_size() -> i64 {
    20
}
fn default_sort_by() -> String {
    "created_at".to_string()
}
fn default_sort_order() -> String {
    "desc".to_string()
}

#[derive(Debug, Serialize)]
pub struct TraceResult {
    pub pipe_type: String,
    pub pipe: serde_json::Value,
    pub inbound_records: Vec<InboundRecord>,
    pub outbound_records: Vec<OutboundRecord>,
}

// ─── PipeService ──────────────────────────────────────────────────────────────

pub struct PipeService {
    pool: SqlitePool,
    config: AppConfig,
}

impl PipeService {
    pub fn new(pool: SqlitePool, config: AppConfig) -> Self {
        Self { pool, config }
    }

    fn seamless_repo(&self) -> SeamlessPipeRepo {
        SeamlessPipeRepo::new(self.pool.clone())
    }

    fn screen_repo(&self) -> ScreenPipeRepo {
        ScreenPipeRepo::new(self.pool.clone())
    }

    // ── Seamless pipe operations ────────────────────────────────────────────────

    pub async fn create_seamless_pipe(
        &self,
        dto: CreateSeamlessPipeDto,
    ) -> AppResult<SeamlessPipe> {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let pipe_number = match dto.pipe_number {
            Some(ref n) if !n.trim().is_empty() => n.trim().to_string(),
            _ => self.generate_pipe_number(&dto.grade, dto.od, dto.wt, "seamless", dto.heat_number.as_deref()),
        };

        let pipe = SeamlessPipe {
            id: Uuid::new_v4().to_string(),
            pipe_number,
            grade: dto.grade,
            od: dto.od,
            wt: dto.wt,
            length: dto.length,
            weight: dto.weight,
            connection_type: dto.connection_type,
            heat_number: dto.heat_number,
            production_date: dto.production_date,
            status: dto.status.unwrap_or_else(|| "in_stock".to_string()),
            location: dto.location,
            notes: dto.notes,
            created_at: now.clone(),
            updated_at: now,
            deleted_at: None,
        };

        self.seamless_repo().create(&pipe).await
    }

    pub async fn update_seamless_pipe(
        &self,
        id: &str,
        dto: UpdateSeamlessPipeDto,
    ) -> AppResult<SeamlessPipe> {
        let existing = self.seamless_repo().find_by_id(id).await?;

        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let updated = SeamlessPipe {
            id: existing.id,
            pipe_number: dto.pipe_number,
            grade: dto.grade,
            od: dto.od,
            wt: dto.wt,
            length: dto.length,
            weight: dto.weight,
            connection_type: dto.connection_type,
            heat_number: dto.heat_number,
            production_date: dto.production_date,
            status: dto.status,
            location: dto.location,
            notes: dto.notes,
            created_at: existing.created_at,
            updated_at: now,
            deleted_at: None,
        };

        self.seamless_repo().update(id, &updated).await
    }

    pub async fn list_seamless_pipes(
        &self,
        filter: &PipeListFilter,
    ) -> AppResult<(Vec<SeamlessPipe>, i64)> {
        let pipes = self
            .seamless_repo()
            .list(
                filter.grade.as_deref(),
                filter.status.as_deref(),
                filter.search.as_deref(),
                filter.page,
                filter.page_size,
                &filter.sort_by,
                &filter.sort_order,
            )
            .await?;
        let total = self
            .seamless_repo()
            .count(
                filter.grade.as_deref(),
                filter.status.as_deref(),
                filter.search.as_deref(),
            )
            .await?;
        Ok((pipes, total))
    }

    pub async fn get_seamless_pipe(&self, id: &str) -> AppResult<SeamlessPipe> {
        self.seamless_repo().find_by_id(id).await
    }

    pub async fn delete_seamless_pipe(&self, id: &str) -> AppResult<()> {
        self.seamless_repo().soft_delete(id).await
    }

    // ── Screen pipe operations ──────────────────────────────────────────────────

    pub async fn create_screen_pipe(
        &self,
        dto: CreateScreenPipeDto,
    ) -> AppResult<ScreenPipe> {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let pipe_number = match dto.pipe_number {
            Some(ref n) if !n.trim().is_empty() => n.trim().to_string(),
            _ => self.generate_pipe_number(&dto.grade, dto.od, dto.wt, "screen", dto.heat_number.as_deref()),
        };

        let pipe = ScreenPipe {
            id: Uuid::new_v4().to_string(),
            pipe_number,
            grade: dto.grade,
            od: dto.od,
            wt: dto.wt,
            length: dto.length,
            weight: dto.weight,
            screen_type: dto.screen_type,
            slot_width: dto.slot_width,
            open_area: dto.open_area,
            connection_type: dto.connection_type,
            heat_number: dto.heat_number,
            production_date: dto.production_date,
            status: dto.status.unwrap_or_else(|| "in_stock".to_string()),
            location: dto.location,
            notes: dto.notes,
            created_at: now.clone(),
            updated_at: now,
            deleted_at: None,
        };

        self.screen_repo().create(&pipe).await
    }

    pub async fn update_screen_pipe(
        &self,
        id: &str,
        dto: UpdateScreenPipeDto,
    ) -> AppResult<ScreenPipe> {
        let existing = self.screen_repo().find_by_id(id).await?;

        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let updated = ScreenPipe {
            id: existing.id,
            pipe_number: dto.pipe_number,
            grade: dto.grade,
            od: dto.od,
            wt: dto.wt,
            length: dto.length,
            weight: dto.weight,
            screen_type: dto.screen_type,
            slot_width: dto.slot_width,
            open_area: dto.open_area,
            connection_type: dto.connection_type,
            heat_number: dto.heat_number,
            production_date: dto.production_date,
            status: dto.status,
            location: dto.location,
            notes: dto.notes,
            created_at: existing.created_at,
            updated_at: now,
            deleted_at: None,
        };

        self.screen_repo().update(id, &updated).await
    }

    pub async fn list_screen_pipes(
        &self,
        filter: &PipeListFilter,
    ) -> AppResult<(Vec<ScreenPipe>, i64)> {
        let pipes = self
            .screen_repo()
            .list(
                filter.grade.as_deref(),
                filter.status.as_deref(),
                filter.search.as_deref(),
                filter.page,
                filter.page_size,
                &filter.sort_by,
                &filter.sort_order,
            )
            .await?;
        let total = self
            .screen_repo()
            .count(
                filter.grade.as_deref(),
                filter.status.as_deref(),
                filter.search.as_deref(),
            )
            .await?;
        Ok((pipes, total))
    }

    pub async fn get_screen_pipe(&self, id: &str) -> AppResult<ScreenPipe> {
        self.screen_repo().find_by_id(id).await
    }

    pub async fn delete_screen_pipe(&self, id: &str) -> AppResult<()> {
        self.screen_repo().soft_delete(id).await
    }

    // ── Tracing ─────────────────────────────────────────────────────────────────

    pub async fn trace_by_pipe_number(
        &self,
        pipe_no: &str,
    ) -> AppResult<TraceResult> {
        // Search seamless pipes first
        if let Some(pipe) = self.seamless_repo().find_by_pipe_number(pipe_no).await? {
            let inbound = self
                .find_inbound_records("seamless", &pipe.id)
                .await?;
            let outbound = self
                .find_outbound_records("seamless", &pipe.id)
                .await?;
            return Ok(TraceResult {
                pipe_type: "seamless".to_string(),
                pipe: serde_json::to_value(&pipe)
                    .map_err(|e| AppError::Internal(format!("Serialize error: {}", e)))?,
                inbound_records: inbound,
                outbound_records: outbound,
            });
        }

        // Then search screen pipes
        if let Some(pipe) = self.screen_repo().find_by_pipe_number(pipe_no).await? {
            let inbound = self
                .find_inbound_records("screen", &pipe.id)
                .await?;
            let outbound = self
                .find_outbound_records("screen", &pipe.id)
                .await?;
            return Ok(TraceResult {
                pipe_type: "screen".to_string(),
                pipe: serde_json::to_value(&pipe)
                    .map_err(|e| AppError::Internal(format!("Serialize error: {}", e)))?,
                inbound_records: inbound,
                outbound_records: outbound,
            });
        }

        Err(AppError::NotFound(format!(
            "Pipe with number '{}' not found",
            pipe_no
        )))
    }

    pub async fn trace_by_heat_number(
        &self,
        heat_no: &str,
    ) -> AppResult<Vec<TraceResult>> {
        let mut results = Vec::new();

        // Search seamless pipes by heat number
        let seamless_pipes: Vec<SeamlessPipe> = sqlx::query_as(
            "SELECT * FROM seamless_pipes \
             WHERE heat_number = ?1 AND deleted_at IS NULL",
        )
        .bind(heat_no)
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::from)?;

        for pipe in &seamless_pipes {
            let inbound = self.find_inbound_records("seamless", &pipe.id).await?;
            let outbound = self.find_outbound_records("seamless", &pipe.id).await?;
            results.push(TraceResult {
                pipe_type: "seamless".to_string(),
                pipe: serde_json::to_value(pipe)
                    .map_err(|e| AppError::Internal(format!("Serialize error: {}", e)))?,
                inbound_records: inbound,
                outbound_records: outbound,
            });
        }

        // Search screen pipes by heat number
        let screen_pipes: Vec<ScreenPipe> = sqlx::query_as(
            "SELECT * FROM screen_pipes \
             WHERE heat_number = ?1 AND deleted_at IS NULL",
        )
        .bind(heat_no)
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::from)?;

        for pipe in &screen_pipes {
            let inbound = self.find_inbound_records("screen", &pipe.id).await?;
            let outbound = self.find_outbound_records("screen", &pipe.id).await?;
            results.push(TraceResult {
                pipe_type: "screen".to_string(),
                pipe: serde_json::to_value(pipe)
                    .map_err(|e| AppError::Internal(format!("Serialize error: {}", e)))?,
                inbound_records: inbound,
                outbound_records: outbound,
            });
        }

        if results.is_empty() {
            return Err(AppError::NotFound(format!(
                "No pipes found with heat number '{}'",
                heat_no
            )));
        }

        Ok(results)
    }

    async fn find_inbound_records(
        &self,
        pipe_type: &str,
        pipe_id: &str,
    ) -> AppResult<Vec<InboundRecord>> {
        // Join inbound_items with inbound_records to get full record details
        let records: Vec<InboundRecord> = sqlx::query_as(
            "SELECT ir.id, ir.inbound_no, ir.inbound_type, ir.supplier_id, \
                    ir.order_id, ir.operator_id, ir.total_items, ir.notes, ir.created_at \
             FROM inbound_records ir \
             INNER JOIN inbound_items ii ON ii.inbound_id = ir.id \
             WHERE ii.pipe_type = ?1 AND ii.pipe_id = ?2",
        )
        .bind(pipe_type)
        .bind(pipe_id)
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::from)?;
        Ok(records)
    }

    async fn find_outbound_records(
        &self,
        pipe_type: &str,
        pipe_id: &str,
    ) -> AppResult<Vec<OutboundRecord>> {
        let records: Vec<OutboundRecord> = sqlx::query_as(
            "SELECT orr.id, orr.outbound_no, orr.outbound_type, orr.customer_id, \
                    orr.order_id, orr.operator_id, orr.total_items, orr.notes, orr.created_at \
             FROM outbound_records orr \
             INNER JOIN outbound_items oi ON oi.outbound_id = orr.id \
             WHERE oi.pipe_type = ?1 AND oi.pipe_id = ?2",
        )
        .bind(pipe_type)
        .bind(pipe_id)
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::from)?;
        Ok(records)
    }

    // ── Pipe number generation (API 5CT format) ────────────────────────────────

    pub fn generate_pipe_number(
        &self,
        grade: &str,
        od: f64,
        wt: f64,
        pipe_type: &str,
        heat_number: Option<&str>,
    ) -> String {
        let type_code = match pipe_type {
            "seamless" => "SS",
            "screen" => "SC",
            _ => "XX",
        };
        let heat_code = heat_number.unwrap_or("XXXX");
        // Use a short random suffix as the sequence number
        let seq = (Uuid::new_v4().as_u128() % 1_000_000) as u32;
        format!(
            "{} {:.3}in\u{d7}{:.2}lb {}-{}-{:06}",
            grade, od, wt, type_code, heat_code, seq
        )
    }
}
