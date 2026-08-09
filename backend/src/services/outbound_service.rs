use sqlx::SqlitePool;

use crate::dto::inventory_dto::{
    CreateOutboundRecordRequest, OutboundFilter, OutboundItemRequest, UpdateOutboundRecordRequest,
};
use crate::error::AppError;
use crate::models::inventory::{OutboundItem, OutboundRecord};
use crate::repositories::inventory_log_repo::InventoryLogRepo;
use crate::repositories::inventory_repo::{CreateInventoryLog, InventoryRepo};
use crate::repositories::outbound_repo::OutboundRepo;
use crate::services::utils;

/// Outbound service — handles sales, scrapped, and transfer stock-out with
/// create/approve/execute/query. Mirror of inbound: `auto_approved` executes
/// immediately, `pending` needs approval later.
pub struct OutboundService;

impl OutboundService {
    /// Creates an outbound record. Needs at least one line item; checks every
    /// item has sufficient on-hand stock. If `auto_approved`, immediately
    /// applies the stock changes.
    pub async fn create_outbound(
        pool: &SqlitePool,
        dto: &CreateOutboundRecordRequest,
    ) -> Result<OutboundRecord, AppError> {
        if dto.items.is_empty() {
            return Err(AppError::Validation("At least one item is required".into()));
        }
        Self::validate_stock(pool, &dto.items).await?;

        let outbound_no = utils::generate_no("OUT");

        let record = OutboundRepo::create_with_items(pool, dto, &outbound_no)
            .await
            .map_err(AppError::from)?;

        if record.approval_status == "auto_approved" {
            Self::execute_outbound_batch(pool, record.id, &record.outbound_type, None, &dto.items)
                .await?;
        }

        Ok(record)
    }

    /// Validates that every item exists (active) and has enough on-hand stock.
    async fn validate_stock(pool: &SqlitePool, items: &[OutboundItemRequest]) -> Result<(), AppError> {
        for item in items {
            if item.quantity <= 0.0 {
                return Err(AppError::Validation(format!(
                    "Item id={} quantity must be positive",
                    item.item_id
                )));
            }
            let exists: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM items WHERE id = ? AND deleted_at IS NULL AND status = 'active')",
            )
            .bind(item.item_id)
            .fetch_one(pool)
            .await
            .map_err(AppError::from)?;
            if !exists {
                return Err(AppError::NotFound(format!(
                    "Item id={} not found or inactive",
                    item.item_id
                )));
            }
            let on_hand = InventoryRepo::stock_on_hand(pool, item.item_id).await?;
            if on_hand < item.quantity {
                return Err(AppError::InsufficientStock(format!(
                    "Insufficient stock for item id={}: {} on hand, {} requested",
                    item.item_id, on_hand, item.quantity
                )));
            }
        }
        Ok(())
    }

    /// Applies outbound stock changes: inserts one `inventory_logs` row per line
    /// item with a negative (out) quantity.
    async fn execute_outbound_batch(
        pool: &SqlitePool,
        record_id: i64,
        _outbound_type: &str,
        created_by: Option<i64>,
        items: &[OutboundItemRequest],
    ) -> Result<(), AppError> {
        let mut tx = pool.begin().await.map_err(AppError::from)?;

        for item in items {
            InventoryLogRepo::create_in_transaction(
                &mut tx,
                &CreateInventoryLog {
                    item_id: item.item_id,
                    quantity: item.quantity,
                    change_type: "outbound".into(),
                    ref_type: Some("outbound".into()),
                    ref_id: Some(record_id),
                    from_location_id: None,
                    to_location_id: None,
                    notes: None,
                    created_by,
                },
            )
            .await
            .map_err(AppError::from)?;
        }

        tx.commit().await.map_err(AppError::from)?;
        Ok(())
    }

    /// Approves a pending outbound and deducts stock.
    pub async fn approve_outbound(
        pool: &SqlitePool,
        id: i64,
        approval_reason: Option<&str>,
        handled_by: Option<i64>,
    ) -> Result<(), AppError> {
        let record = OutboundRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::NotFound(format!("Outbound record id={} not found", id)))?;

        if record.deleted_at.is_some() {
            return Err(AppError::NotFound(format!(
                "Outbound record id={} has been deleted",
                id
            )));
        }

        if record.approval_status != "pending" {
            return Err(AppError::Validation(format!(
                "Cannot approve outbound with status '{}'",
                record.approval_status
            )));
        }

        let items = OutboundRepo::find_items(pool, id).await.map_err(AppError::from)?;

        // Guard against over-deduction: stock must still be sufficient.
        for item in &items {
            let on_hand = InventoryRepo::stock_on_hand(pool, item.item_id).await?;
            if on_hand < item.quantity {
                return Err(AppError::InsufficientStock(format!(
                    "Insufficient stock for item id={}: {} on hand, {} requested",
                    item.item_id, on_hand, item.quantity
                )));
            }
        }

        let mut tx = pool.begin().await.map_err(AppError::from)?;

        let affected = OutboundRepo::approve(&mut tx, id, approval_reason, handled_by)
            .await
            .map_err(AppError::from)?;
        if affected == 0 {
            return Err(AppError::Validation(format!(
                "Outbound record id={} was already processed or deleted during approval",
                id
            )));
        }

        for item in &items {
            InventoryLogRepo::create_in_transaction(
                &mut tx,
                &CreateInventoryLog {
                    item_id: item.item_id,
                    quantity: item.quantity,
                    change_type: "outbound".into(),
                    ref_type: Some("outbound".into()),
                    ref_id: Some(id),
                    from_location_id: None,
                    to_location_id: None,
                    notes: None,
                    created_by: handled_by,
                },
            )
            .await
            .map_err(AppError::from)?;
        }

        // Increment delivered_quantity on the linked sales order (if any).
        if let Some(order_id) = record.order_id {
            let mut count_by_item: std::collections::BTreeMap<i64, f64> =
                std::collections::BTreeMap::new();
            for item in &items {
                *count_by_item.entry(item.item_id).or_insert(0.0) += item.quantity;
            }
            for (item_id, qty) in count_by_item {
                sqlx::query(
                    "UPDATE sales_order_items SET delivered_quantity = delivered_quantity + ? \
                     WHERE id = (SELECT id FROM sales_order_items \
                      WHERE order_id = ? AND item_id = ? AND delivered_quantity < quantity LIMIT 1)",
                )
                .bind(qty)
                .bind(order_id)
                .bind(item_id)
                .execute(&mut *tx)
                .await
                .map_err(AppError::from)?;
            }
        }

        tx.commit().await.map_err(AppError::from)?;
        Ok(())
    }

    /// Rejects a pending outbound — sets `rejected` and stores the reason. No stock changes.
    pub async fn reject_outbound(pool: &SqlitePool, id: i64, reason: &str) -> Result<(), AppError> {
        let record = OutboundRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::NotFound(format!("Outbound record id={} not found", id)))?;

        if record.approval_status != "pending" {
            return Err(AppError::Validation(format!(
                "Cannot reject outbound with status '{}'",
                record.approval_status
            )));
        }

        OutboundRepo::update_status(pool, id, "rejected", Some(reason))
            .await
            .map_err(AppError::from)?;

        Ok(())
    }

    /// Fetches an outbound record with all its line items. Returns `(record, items)` tuple.
    pub async fn get_outbound_record(
        pool: &SqlitePool,
        id: i64,
    ) -> Result<(OutboundRecord, Vec<OutboundItem>), AppError> {
        let record = OutboundRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::NotFound(format!("Outbound record id={} not found", id)))?;

        let items = OutboundRepo::find_items(pool, id).await.map_err(AppError::from)?;

        Ok((record, items))
    }

    /// Paginated outbound records.
    pub async fn list_outbound_records(
        pool: &SqlitePool,
        filter: &OutboundFilter,
    ) -> Result<(Vec<OutboundRecord>, u64), AppError> {
        OutboundRepo::list(pool, filter).await.map_err(AppError::from)
    }

    /// Soft-deletes an outbound record. Only `auto_approved` or `rejected` ones can be deleted.
    pub async fn delete_outbound(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
        let record = OutboundRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::NotFound(format!("Outbound record id={} not found", id)))?;

        if record.approval_status != "auto_approved" && record.approval_status != "rejected" {
            return Err(AppError::Validation(format!(
                "Cannot delete outbound with status '{}'. Only auto-approved or rejected records can be deleted.",
                record.approval_status
            )));
        }

        OutboundRepo::delete(pool, id).await.map_err(AppError::from)
    }

    /// Updates editable fields on an outbound record.
    pub async fn update_outbound(
        pool: &SqlitePool,
        id: i64,
        dto: &UpdateOutboundRecordRequest,
    ) -> Result<OutboundRecord, AppError> {
        let record = OutboundRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::NotFound(format!("Outbound record id={} not found", id)))?;

        if record.deleted_at.is_some() {
            return Err(AppError::NotFound(format!(
                "Outbound record id={} has been deleted",
                id
            )));
        }

        if record.approval_status != "auto_approved" && record.approval_status != "rejected" {
            return Err(AppError::Validation(format!(
                "Cannot update outbound with status '{}'. Only auto-approved or rejected records can be updated.",
                record.approval_status
            )));
        }

        OutboundRepo::update(pool, id, dto).await.map_err(AppError::from)
    }

    /// Gets all line items for a given outbound record.
    pub async fn list_outbound_items(pool: &SqlitePool, outbound_id: i64) -> Result<Vec<OutboundItem>, AppError> {
        OutboundRepo::find_items(pool, outbound_id).await.map_err(AppError::from)
    }
}
