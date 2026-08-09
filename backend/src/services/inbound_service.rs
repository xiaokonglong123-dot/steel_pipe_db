use sqlx::{Sqlite, SqlitePool, Transaction};

use crate::dto::inventory_dto::{
    BatchCreateInboundRequest, CreateInboundRecordRequest, InboundFilter, InboundItemRequest,
    UpdateInboundRecordRequest,
};
use crate::error::AppError;
use crate::models::inventory::{InboundItem, InboundRecord};
use crate::repositories::inbound_repo::InboundRepo;
use crate::repositories::inventory_log_repo::InventoryLogRepo;
use crate::repositories::inventory_repo::CreateInventoryLog;
use crate::services::utils;

/// Validate that every item in an inbound request exists and is active.
async fn validate_items(pool: &SqlitePool, items: &[InboundItemRequest]) -> Result<(), AppError> {
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
    }
    Ok(())
}

/// Inbound service — handles purchase, production, and return stock-in with
/// create/approve/execute/query. Auto-approved inbound kicks off stock changes
/// right away; pending ones need a separate `approve_inbound` call.
pub struct InboundService;

impl InboundService {
    pub async fn create_inbound(
        pool: &SqlitePool,
        dto: &CreateInboundRecordRequest,
    ) -> Result<InboundRecord, AppError> {
        if dto.items.is_empty() {
            return Err(AppError::Validation("At least one item is required".into()));
        }
        validate_items(pool, &dto.items).await?;

        let inbound_no = utils::generate_no("IN");

        // Header + items + (auto-approved) stock changes all in ONE transaction.
        let mut tx = pool.begin().await.map_err(AppError::from)?;

        let record = InboundRepo::create_inner(&mut tx, dto, &inbound_no)
            .await
            .map_err(AppError::from)?;

        if record.approval_status == "auto_approved" {
            Self::execute_inbound_batch_inner(&mut tx, record.id, &dto.items).await?;
        }

        tx.commit().await.map_err(AppError::from)?;

        Ok(record)
    }

    /// Applies inbound stock changes: inserts an `inventory_logs` row per line
    /// item with a positive (in) quantity.
    async fn execute_inbound_batch_inner(
        tx: &mut Transaction<'_, Sqlite>,
        record_id: i64,
        items: &[InboundItemRequest],
    ) -> Result<(), AppError> {
        for item in items {
            InventoryLogRepo::create_in_transaction(
                tx,
                &CreateInventoryLog {
                    item_id: item.item_id,
                    quantity: item.quantity,
                    change_type: "inbound".into(),
                    ref_type: Some("inbound".into()),
                    ref_id: Some(record_id),
                    from_location_id: None,
                    to_location_id: None,
                    notes: None,
                    created_by: None,
                },
            )
            .await
            .map_err(AppError::from)?;
        }
        Ok(())
    }

    pub async fn approve_inbound(
        pool: &SqlitePool,
        id: i64,
        approval_reason: Option<&str>,
        handled_by: Option<i64>,
    ) -> Result<(), AppError> {
        let record = InboundRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::NotFound(format!("Inbound record id={} not found", id)))?;

        if record.deleted_at.is_some() {
            return Err(AppError::NotFound(format!(
                "Inbound record id={} has been deleted",
                id
            )));
        }

        if record.approval_status != "pending" {
            return Err(AppError::Validation(format!(
                "Cannot approve inbound with status '{}'",
                record.approval_status
            )));
        }

        let items = InboundRepo::find_items(pool, id).await.map_err(AppError::from)?;

        let mut tx = pool.begin().await.map_err(AppError::from)?;

        let affected = InboundRepo::approve(&mut tx, id, approval_reason, handled_by)
            .await
            .map_err(AppError::from)?;

        if affected == 0 {
            return Err(AppError::NotFound(format!(
                "Inbound record id={} not found or was deleted during approval",
                id
            )));
        }

        for item in &items {
            InventoryLogRepo::create_in_transaction(
                &mut tx,
                &CreateInventoryLog {
                    item_id: item.item_id,
                    quantity: item.quantity,
                    change_type: "inbound".into(),
                    ref_type: Some("inbound".into()),
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

        // Increment received_quantity on the linked purchase order (if any).
        if let Some(order_id) = record.order_id {
            let mut count_by_item: std::collections::BTreeMap<i64, f64> =
                std::collections::BTreeMap::new();
            for item in &items {
                *count_by_item.entry(item.item_id).or_insert(0.0) += item.quantity;
            }
            for (item_id, qty) in count_by_item {
                sqlx::query(
                    "UPDATE purchase_order_items SET received_quantity = received_quantity + ? \
                     WHERE id = (SELECT id FROM purchase_order_items \
                      WHERE order_id = ? AND item_id = ? AND received_quantity < quantity LIMIT 1)",
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

    pub async fn reject_inbound(pool: &SqlitePool, id: i64, reason: &str) -> Result<(), AppError> {
        let record = InboundRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::NotFound(format!("Inbound record id={} not found", id)))?;

        if record.approval_status != "pending" {
            return Err(AppError::Validation(format!(
                "Cannot reject inbound with status '{}'",
                record.approval_status
            )));
        }

        InboundRepo::update_status(pool, id, "rejected", Some(reason))
            .await
            .map_err(AppError::from)?;

        Ok(())
    }

    /// Fetches an inbound record with all its line items. Returns `(record, items)` tuple.
    pub async fn get_inbound_record(
        pool: &SqlitePool,
        id: i64,
    ) -> Result<(InboundRecord, Vec<InboundItem>), AppError> {
        let record = InboundRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::NotFound(format!("Inbound record id={} not found", id)))?;

        let items = InboundRepo::find_items(pool, id).await.map_err(AppError::from)?;

        Ok((record, items))
    }

    /// Paginated inbound records — filter by status, type, etc.
    pub async fn list_inbound_records(
        pool: &SqlitePool,
        filter: &InboundFilter,
    ) -> Result<(Vec<InboundRecord>, u64), AppError> {
        InboundRepo::list(pool, filter).await.map_err(AppError::from)
    }

    /// Soft-deletes an inbound record. Only `auto_approved` or `rejected` ones are fair game.
    pub async fn delete_inbound(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
        let record = InboundRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::NotFound(format!("Inbound record id={} not found", id)))?;

        if record.approval_status != "auto_approved" && record.approval_status != "rejected" {
            return Err(AppError::Validation(format!(
                "Cannot delete inbound with status '{}'. Only auto-approved or rejected records can be deleted.",
                record.approval_status
            )));
        }

        InboundRepo::delete(pool, id).await.map_err(AppError::from)
    }

    /// Updates editable fields on an inbound record.
    pub async fn update_inbound(
        pool: &SqlitePool,
        id: i64,
        dto: &UpdateInboundRecordRequest,
    ) -> Result<InboundRecord, AppError> {
        let record = InboundRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::NotFound(format!("Inbound record id={} not found", id)))?;

        if record.deleted_at.is_some() {
            return Err(AppError::NotFound(format!(
                "Inbound record id={} has been deleted",
                id
            )));
        }

        if record.approval_status != "auto_approved" && record.approval_status != "rejected" {
            return Err(AppError::Validation(format!(
                "Cannot update inbound with status '{}'. Only auto-approved or rejected records can be updated.",
                record.approval_status
            )));
        }

        InboundRepo::update(pool, id, dto).await.map_err(AppError::from)
    }

    /// Gets all line items for a given inbound record.
    pub async fn list_inbound_items(pool: &SqlitePool, inbound_id: i64) -> Result<Vec<InboundItem>, AppError> {
        InboundRepo::find_items(pool, inbound_id).await.map_err(AppError::from)
    }

    pub async fn batch_create_inbound(
        pool: &SqlitePool,
        dto: &BatchCreateInboundRequest,
    ) -> Result<Vec<InboundRecord>, AppError> {
        if dto.records.is_empty() {
            return Err(AppError::Validation(
                "At least one inbound record is required".into(),
            ));
        }

        for record_dto in &dto.records {
            validate_items(pool, &record_dto.items).await?;
        }

        let mut tx = pool.begin().await.map_err(AppError::from)?;
        let mut results = Vec::with_capacity(dto.records.len());

        for record_dto in &dto.records {
            let inbound_no = utils::generate_no("IN");
            let record = InboundRepo::create_inner(&mut tx, record_dto, &inbound_no)
                .await
                .map_err(AppError::from)?;
            if record.approval_status == "auto_approved" {
                Self::execute_inbound_batch_inner(&mut tx, record.id, &record_dto.items).await?;
            }
            results.push(record);
        }

        tx.commit().await.map_err(AppError::from)?;
        Ok(results)
    }
}
