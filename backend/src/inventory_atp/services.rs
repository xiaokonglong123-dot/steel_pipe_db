//! Inventory ATP services — reservations, internal transfers (two-location
//! stock movement in one transaction), cycle count sessions.

use sqlx::PgPool;

use crate::dto::inventory_atp_dto::{
    CompleteCountSessionRequest, CreateCountTemplateRequest, CreateReservationRequest,
    CreateTransferRequest,
};
use crate::error::AppError;
use crate::inventory_atp::repos::{AtpSlotRepo, CountRepo, TransferRepo};
use crate::models::inventory_atp::{
    AtpOverviewRow, AtpSlot, CountSession, CountTemplate, InternalTransfer,
};

pub struct InventoryAtpService;

impl InventoryAtpService {
    // -----------------------------------------------------------------------
    // ATP
    // -----------------------------------------------------------------------

    pub async fn reserve(
        pool: &PgPool,
        tenant_id: i64,
        dto: &CreateReservationRequest,
    ) -> Result<AtpSlot, AppError> {
        if dto.quantity <= rust_decimal::Decimal::ZERO {
            return Err(AppError::Validation("Reservation quantity must be positive".into()));
        }
        // Guard: cannot reserve more than currently available.
        let overview = AtpSlotRepo::pipe_atp(pool, tenant_id, &dto.pipe_type, dto.pipe_number.as_deref().unwrap_or(""))
            .await
            .map_err(AppError::from)?;
        if dto.quantity > overview.available {
            return Err(AppError::Validation(format!(
                "Insufficient available stock: {} requested, {} available",
                dto.quantity, overview.available
            )));
        }
        AtpSlotRepo::reserve(
            pool,
            tenant_id,
            &dto.pipe_type,
            dto.pipe_number.as_deref(),
            dto.quantity,
            dto.sales_order_id,
        )
        .await
        .map_err(AppError::from)
    }

    pub async fn release(pool: &PgPool, tenant_id: i64, reservation_id: i64) -> Result<AtpSlot, AppError> {
        AtpSlotRepo::release(pool, tenant_id, reservation_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Reservation not found: {}", reservation_id)))
    }

    pub async fn overview(pool: &PgPool, tenant_id: i64) -> Result<Vec<AtpOverviewRow>, AppError> {
        AtpSlotRepo::overview(pool, tenant_id).await.map_err(AppError::from)
    }

    pub async fn pipe_atp(pool: &PgPool, tenant_id: i64, pipe_type: &str, pipe_number: &str) -> Result<AtpOverviewRow, AppError> {
        AtpSlotRepo::pipe_atp(pool, tenant_id, pipe_type, pipe_number)
            .await
            .map_err(AppError::from)
    }

    // -----------------------------------------------------------------------
    // Internal transfers
    // -----------------------------------------------------------------------

    /// Move stock between locations atomically: update the pipes' location_id
    /// from source to destination. Inventory is per-pipe (one row per pipe in
    /// the pipes tables), so a transfer moves pipes, not quantities.
    pub async fn create_transfer(
        pool: &PgPool,
        tenant_id: i64,
        created_by: Option<i64>,
        dto: &CreateTransferRequest,
    ) -> Result<InternalTransfer, AppError> {
        if dto.from_location_id == dto.to_location_id {
            return Err(AppError::Validation("Source and destination locations must differ".into()));
        }
        if dto.quantity <= rust_decimal::Decimal::ZERO {
            return Err(AppError::Validation("Transfer quantity must be positive".into()));
        }
        // Locations must exist.
        let loc_ok: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM locations WHERE id IN ($1, $2)",
        )
        .bind(dto.from_location_id)
        .bind(dto.to_location_id)
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;
        if loc_ok != 2 {
            return Err(AppError::Validation("One or both locations not found".into()));
        }
        // The pipe must exist and be in stock at the source location.
        // PG has no UPDATE ... LIMIT — verify count first, then update.
        let pipe_number = dto.pipe_number.clone().unwrap_or_default();
        let table = pipe_table(&dto.pipe_number.as_deref().unwrap_or("seamless"));
        let available: i64 = sqlx::query_scalar(&format!(
            "SELECT COUNT(*) FROM {table} \
             WHERE pipe_number = $1 AND status = 'in_stock' AND location_id = $2"
        ))
        .bind(&pipe_number)
        .bind(dto.from_location_id)
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;
        let needed = dto.quantity.round().to_string().parse::<i64>().unwrap_or(1).max(1);
        if available < needed {
            return Err(AppError::Validation(format!(
                "Pipe '{}': {} available at source location {}, {} requested",
                pipe_number, available, dto.from_location_id, needed
            )));
        }
        sqlx::query(&format!(
            "UPDATE {table} SET location_id = $2, updated_at = NOW() \
             WHERE pipe_number = $1 AND status = 'in_stock' AND location_id = $3"
        ))
        .bind(&pipe_number)
        .bind(dto.to_location_id)
        .bind(dto.from_location_id)
        .execute(pool)
        .await
        .map_err(AppError::from)?;

        let transfer_no = format!("TR-{}-{}", chrono::Utc::now().format("%Y%m%d"), seq(pool, "internal_transfers").await?);
        let transfer = TransferRepo::create(
            pool, tenant_id, &transfer_no, dto.from_location_id, dto.to_location_id,
            dto.pipe_id, Some(&pipe_number), dto.quantity, created_by, dto.notes.as_deref(),
        )
        .await
        .map_err(AppError::from)?;
        Ok(transfer)
    }

    pub async fn list_transfers(pool: &PgPool, tenant_id: i64) -> Result<Vec<InternalTransfer>, AppError> {
        TransferRepo::list(pool, tenant_id).await.map_err(AppError::from)
    }

    // -----------------------------------------------------------------------
    // Cycle counting
    // -----------------------------------------------------------------------

    pub async fn create_count_template(
        pool: &PgPool,
        tenant_id: i64,
        dto: &CreateCountTemplateRequest,
    ) -> Result<CountTemplate, AppError> {
        if dto.name.trim().is_empty() {
            return Err(AppError::Validation("Template name is required".into()));
        }
        if dto.location_ids.is_empty() {
            return Err(AppError::Validation("Template needs at least one location".into()));
        }
        CountRepo::create_template(
            pool,
            tenant_id,
            dto.name.trim(),
            dto.description.as_deref(),
            &serde_json::json!(dto.location_ids),
        )
        .await
        .map_err(AppError::from)
    }

    pub async fn list_count_templates(pool: &PgPool, tenant_id: i64) -> Result<Vec<CountTemplate>, AppError> {
        CountRepo::list_templates(pool, tenant_id).await.map_err(AppError::from)
    }

    /// Start a count session from a template.
    pub async fn start_count_session(pool: &PgPool, tenant_id: i64, template_id: i64) -> Result<CountSession, AppError> {
        let session_no = format!("CC-{}-{}", chrono::Utc::now().format("%Y%m%d"), seq(pool, "count_sessions").await?);
        CountRepo::create_session(pool, tenant_id, template_id, &session_no)
            .await
            .map_err(AppError::from)
    }

    pub async fn complete_count_session(
        pool: &PgPool,
        tenant_id: i64,
        dto: &CompleteCountSessionRequest,
    ) -> Result<CountSession, AppError> {
        CountRepo::complete_session(pool, tenant_id, dto.session_id, &dto.result)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Count session not found: {}", dto.session_id)))
    }

    pub async fn list_count_sessions(pool: &PgPool, tenant_id: i64) -> Result<Vec<CountSession>, AppError> {
        CountRepo::list_sessions(pool, tenant_id).await.map_err(AppError::from)
    }
}

/// Map pipe_type string to its pipes table name.
fn pipe_table(pipe_type: &str) -> &'static str {
    match pipe_type {
        "screen" => "screen_pipes",
        "welded" => "welded_pipes",
        _ => "seamless_pipes",
    }
}

/// Per-table sequence helper for document numbers.
async fn seq(pool: &PgPool, table: &str) -> Result<i64, AppError> {
    let n: i64 = sqlx::query_scalar(&format!("SELECT COALESCE(MAX(id), 0) + 1 FROM {}", table))
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;
    Ok(n)
}
