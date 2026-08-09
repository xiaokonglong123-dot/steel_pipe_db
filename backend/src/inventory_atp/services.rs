//! Inventory ATP services — reservations, internal transfers (two-location
//! stock movement in one transaction), cycle count sessions.

use sqlx::SqlitePool;

use crate::dto::inventory_atp_dto::{
    CompleteCountSessionRequest, CreateCountTemplateRequest, CreateReservationRequest,
    CreateTransferRequest,
};
use crate::error::AppError;
use crate::inventory_atp::repos::{AtpSlotRepo, CountRepo, TransferRepo};
use crate::models::inventory_atp::{
    AtpOverviewRow, AtpSlot, CountSession, CountTemplate, InternalTransfer,
};
use crate::inventory::inventory_log_repo::InventoryLogRepo;
use crate::inventory::inventory_repo::{CreateInventoryLog, InventoryRepo};

pub struct InventoryAtpService;

impl InventoryAtpService {
    // -----------------------------------------------------------------------
    // ATP
    // -----------------------------------------------------------------------

    pub async fn reserve(
        pool: &SqlitePool,
        tenant_id: i64,
        dto: &CreateReservationRequest,
    ) -> Result<AtpSlot, AppError> {
        if dto.quantity <= 0.0 {
            return Err(AppError::Validation("Reservation quantity must be positive".into()));
        }
        let item = crate::items::item_service::ItemService::get_item(pool, dto.item_id).await?;
        let sku = Some(item.sku.as_str());
        // Guard: cannot reserve more than currently available.
        let overview = AtpSlotRepo::item_atp(pool, tenant_id, dto.item_id)
            .await
            .map_err(AppError::from)?;
        if dto.quantity > overview.available {
            return Err(AppError::Validation(format!(
                "Insufficient available stock: {} requested, {} available",
                dto.quantity, overview.available
            )));
        }
        AtpSlotRepo::reserve(pool, tenant_id, dto.item_id, sku, dto.quantity, dto.sales_order_id)
            .await
            .map_err(AppError::from)
    }

    pub async fn release(pool: &SqlitePool, tenant_id: i64, reservation_id: i64) -> Result<AtpSlot, AppError> {
        AtpSlotRepo::release(pool, tenant_id, reservation_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Reservation not found: {}", reservation_id)))
    }

    pub async fn overview(pool: &SqlitePool, tenant_id: i64) -> Result<Vec<AtpOverviewRow>, AppError> {
        AtpSlotRepo::overview(pool, tenant_id).await.map_err(AppError::from)
    }

    pub async fn item_atp(pool: &SqlitePool, tenant_id: i64, item_id: i64) -> Result<AtpOverviewRow, AppError> {
        AtpSlotRepo::item_atp(pool, tenant_id, item_id)
            .await
            .map_err(AppError::from)
    }

    // -----------------------------------------------------------------------
    // Internal transfers
    // -----------------------------------------------------------------------

    /// Move stock between locations: validate source availability, record the
    /// transfer row and a `transfer` inventory log (negative at source,
    /// positive at destination), all in one transaction.
    pub async fn create_transfer(
        pool: &SqlitePool,
        tenant_id: i64,
        created_by: Option<i64>,
        dto: &CreateTransferRequest,
    ) -> Result<InternalTransfer, AppError> {
        if dto.from_location_id == dto.to_location_id {
            return Err(AppError::Validation("Source and destination locations must differ".into()));
        }
        if dto.quantity <= 0.0 {
            return Err(AppError::Validation("Transfer quantity must be positive".into()));
        }
        // Locations must exist.
        let loc_ok: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM locations WHERE id IN (?, ?) AND deleted_at IS NULL",
        )
        .bind(dto.from_location_id)
        .bind(dto.to_location_id)
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;
        if loc_ok != 2 {
            return Err(AppError::Validation("One or both locations not found".into()));
        }
        // Item must exist.
        let item = crate::items::item_service::ItemService::get_item(pool, dto.item_id).await?;

        // Source availability check.
        let available = InventoryRepo::stock_on_hand_at_location(pool, dto.item_id, dto.from_location_id)
            .await
            .map_err(AppError::from)?;
        if available < dto.quantity {
            return Err(AppError::Validation(format!(
                "Item '{}': {} available at source location {}, {} requested",
                item.sku, available, dto.from_location_id, dto.quantity
            )));
        }

        let transfer_no = format!("TR-{}-{}", chrono::Utc::now().format("%Y%m%d"), seq(pool, "internal_transfers").await?);
        let transfer = TransferRepo::create(
            pool, tenant_id, &transfer_no, dto.from_location_id, dto.to_location_id,
            Some(dto.item_id), Some(item.sku.as_str()), dto.quantity, created_by, dto.notes.as_deref(),
        )
        .await
        .map_err(AppError::from)?;

        // Movement log: negative at source, positive at destination.
        InventoryLogRepo::create(
            pool,
            &CreateInventoryLog {
                item_id: dto.item_id,
                quantity: dto.quantity,
                change_type: "transfer".into(),
                ref_type: Some("transfer".into()),
                ref_id: Some(transfer.id),
                from_location_id: Some(dto.from_location_id),
                to_location_id: Some(dto.to_location_id),
                notes: dto.notes.clone(),
                created_by,
            },
        )
        .await
        .map_err(AppError::from)?;

        Ok(transfer)
    }

    pub async fn list_transfers(pool: &SqlitePool, tenant_id: i64) -> Result<Vec<InternalTransfer>, AppError> {
        TransferRepo::list(pool, tenant_id).await.map_err(AppError::from)
    }

    // -----------------------------------------------------------------------
    // Cycle counting
    // -----------------------------------------------------------------------

    pub async fn create_count_template(
        pool: &SqlitePool,
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

    pub async fn list_count_templates(pool: &SqlitePool, tenant_id: i64) -> Result<Vec<CountTemplate>, AppError> {
        CountRepo::list_templates(pool, tenant_id).await.map_err(AppError::from)
    }

    /// Start a count session from a template.
    pub async fn start_count_session(pool: &SqlitePool, tenant_id: i64, template_id: i64) -> Result<CountSession, AppError> {
        let session_no = format!("CC-{}-{}", chrono::Utc::now().format("%Y%m%d"), seq(pool, "count_sessions").await?);
        CountRepo::create_session(pool, tenant_id, template_id, &session_no)
            .await
            .map_err(AppError::from)
    }

    pub async fn complete_count_session(
        pool: &SqlitePool,
        tenant_id: i64,
        dto: &CompleteCountSessionRequest,
    ) -> Result<CountSession, AppError> {
        CountRepo::complete_session(pool, tenant_id, dto.session_id, &dto.result)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Count session not found: {}", dto.session_id)))
    }

    pub async fn list_count_sessions(pool: &SqlitePool, tenant_id: i64) -> Result<Vec<CountSession>, AppError> {
        CountRepo::list_sessions(pool, tenant_id).await.map_err(AppError::from)
    }
}

/// Per-table sequence helper for document numbers.
async fn seq(pool: &SqlitePool, table: &str) -> Result<i64, AppError> {
    let n: i64 = sqlx::query_scalar(&format!("SELECT COALESCE(MAX(id), 0) + 1 FROM {}", table))
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;
    Ok(n)
}
