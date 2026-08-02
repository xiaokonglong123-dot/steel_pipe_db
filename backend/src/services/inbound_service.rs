use sqlx::Postgres;
use sqlx::{PgPool, Transaction};

use crate::domain::pipe::PipeType;
use crate::dto::inventory_dto::{
    BatchCreateInboundRequest, CreateInboundRecordRequest, InboundFilter, InboundPipeItem,
    UpdateInboundRecordRequest,
};
use crate::error::AppError;
use crate::models::inventory::{InboundItem, InboundRecord};
use crate::models::screen_pipe::ScreenPipe;
use crate::models::seamless_pipe::SeamlessPipe;
use crate::repositories::inbound_repo::InboundRepo;
use crate::repositories::inventory_log_repo::InventoryLogRepo;
use crate::repositories::inventory_repo::{CreateInventoryLog, InventoryRepo};
use crate::repositories::location_repo::LocationRepo;
use crate::services::pipe_helpers::PipeHelpers;
use crate::services::utils;

/// After inbound execution, refresh location used_counts for all affected pipes.
/// This ensures the `used_count` column stays consistent with actual stock.
async fn refresh_inbound_locations(
    pool: &PgPool,
    items: &[InboundPipeItem],
) -> Result<(), AppError> {
    let mut location_ids = std::collections::BTreeSet::new();
    for item in items {
        // Validate pipe type
        PipeType::from_pipe_type_str(&item.pipe_type).ok_or_else(|| {
            AppError::Validation(format!("Unknown pipe_type: {}", item.pipe_type))
        })?;
        if let Some(loc_id) =
            InventoryRepo::get_pipe_location_id(pool, &item.pipe_type, item.pipe_id)
                .await
                .map_err(AppError::from)?
        {
            location_ids.insert(loc_id);
        }
    }
    for loc_id in location_ids {
        LocationRepo::refresh_used_count(pool, loc_id)
            .await
            .map_err(AppError::from)?;
    }
    Ok(())
}

const VALID_INBOUND_PIPE_STATUSES: &[&str] = &["new", "outbound", "scrapped"];

/// Inbound service — handles purchase, production, and return stock-in with create/approve/execute/query.
/// Auto-approved inbound kicks off stock changes right away; pending ones need a separate `approve_inbound` call.
pub struct InboundService;

impl InboundService {
    pub async fn create_inbound(
        pool: &PgPool,
        dto: &CreateInboundRecordRequest,
    ) -> Result<InboundRecord, AppError> {
        if dto.pipes.is_empty() {
            return Err(AppError::Validation("At least one pipe is required".into()));
        }

        for item in &dto.pipes {
            let pipe_type = PipeType::from_pipe_type_str(&item.pipe_type).ok_or_else(|| {
                AppError::Validation(format!("Unknown pipe_type: {}", item.pipe_type))
            })?;
            match pipe_type {
                PipeType::Seamless => PipeHelpers::validate_pipes_for_inbound::<SeamlessPipe>(pool, &[item.pipe_id]).await?,
                PipeType::Screen => PipeHelpers::validate_pipes_for_inbound::<ScreenPipe>(pool, &[item.pipe_id]).await?,
                PipeType::Welded => PipeHelpers::validate_pipes_for_inbound::<crate::models::welded_pipe::WeldedPipe>(pool, &[item.pipe_id]).await?,
            }
        }

        let inbound_no = utils::generate_no("IN");

        // Header + items + (auto-approved) stock changes all in ONE transaction:
        // no orphan record is left behind if execution fails, and the guarded
        // status update prevents concurrent double-inbound of the same pipe.
        let mut tx = pool.begin().await.map_err(AppError::from)?;

        let record = InboundRepo::create_inner(&mut tx, dto, &inbound_no)
            .await
            .map_err(AppError::from)?;

        if record.approval_status == "auto_approved" {
            Self::execute_inbound_batch_inner(&mut tx, record.id, &dto.pipes).await?;
        }

        tx.commit().await.map_err(AppError::from)?;

        refresh_inbound_locations(pool, &dto.pipes).await?;

        Ok(record)
    }

    async fn execute_inbound_batch_inner(
        tx: &mut Transaction<'_, Postgres>,
        record_id: i64,
        items: &[crate::dto::inventory_dto::InboundPipeItem],
    ) -> Result<(), AppError> {
        for item in items {
            let table = match PipeType::from_pipe_type_str(&item.pipe_type) {
                Some(PipeType::Seamless) => "seamless_pipes",
                Some(PipeType::Screen) => "screen_pipes",
                Some(PipeType::Welded) => "welded_pipes",
                None => {
                    return Err(AppError::Validation(format!(
                        "Unknown pipe_type: {}",
                        item.pipe_type
                    )));
                }
            };
            // Guarded update: a pipe already `in_stock` cannot be inbound again.
            // `affected == 0` means the pipe was deleted or its status changed
            // concurrently — roll back instead of double-counting stock.
            let sql = format!(
                "UPDATE {} SET status = 'in_stock', updated_at = NOW() \
                 WHERE id = ? AND deleted_at IS NULL AND status != 'in_stock'",
                table
            );
            let result = sqlx::query(&sql)
                .bind(item.pipe_id)
                .execute(&mut **tx)
                .await
                .map_err(AppError::from)?;
            if result.rows_affected() == 0 {
                return Err(AppError::PipeStatusConflict(format!(
                    "Pipe id={} (type={}) status changed concurrently, expected non-in_stock status not matched",
                    item.pipe_id, item.pipe_type
                )));
            }

            InventoryLogRepo::create_in_transaction(
                &mut *tx,
                &CreateInventoryLog {
                    pipe_type: item.pipe_type.clone(),
                    pipe_id: item.pipe_id,
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

    async fn create_inbound_inner(
        tx: &mut Transaction<'_, Postgres>,
        dto: &CreateInboundRecordRequest,
        inbound_no: &str,
    ) -> Result<InboundRecord, AppError> {
        InboundRepo::create_inner(tx, dto, inbound_no)
            .await
            .map_err(AppError::from)
    }

    pub async fn approve_inbound(
        pool: &PgPool,
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

        let items = InboundRepo::find_items(pool, id)
            .await
            .map_err(AppError::from)?;

        for item in &items {
            let pipe_type = PipeType::from_pipe_type_str(&item.pipe_type).ok_or_else(|| {
                AppError::Validation(format!("Unknown pipe_type: {}", item.pipe_type))
            })?;
            match pipe_type {
                PipeType::Seamless => PipeHelpers::validate_pipes_for_inbound::<SeamlessPipe>(pool, &[item.pipe_id]).await?,
                PipeType::Screen => PipeHelpers::validate_pipes_for_inbound::<ScreenPipe>(pool, &[item.pipe_id]).await?,
                PipeType::Welded => PipeHelpers::validate_pipes_for_inbound::<crate::models::welded_pipe::WeldedPipe>(pool, &[item.pipe_id]).await?,
            }
        }

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
            let table = match PipeType::from_pipe_type_str(&item.pipe_type) {
                Some(PipeType::Seamless) => "seamless_pipes",
                Some(PipeType::Screen) => "screen_pipes",
                Some(PipeType::Welded) => "welded_pipes",
                None => {
                    return Err(AppError::Validation(format!(
                        "Unknown pipe_type: {}",
                        item.pipe_type
                    )));
                }
            };
            let sql = format!(
                "UPDATE {} SET status = 'in_stock', updated_at = NOW() \
                 WHERE id = ? AND deleted_at IS NULL AND status != 'in_stock'",
                table
            );
            let result = sqlx::query(&sql)
                .bind(item.pipe_id)
                .execute(&mut *tx)
                .await
                .map_err(AppError::from)?;
            if result.rows_affected() == 0 {
                return Err(AppError::PipeStatusConflict(format!(
                    "Pipe id={} (type={}) status changed concurrently, expected pre-status not matched",
                    item.pipe_id, item.pipe_type
                )));
            }

            InventoryLogRepo::create_in_transaction(
                &mut tx,
                &CreateInventoryLog {
                    pipe_type: item.pipe_type.clone(),
                    pipe_id: item.pipe_id,
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

        // FIX 3: Increment received_quantity on linked purchase order
        if let Some(order_id) = record.order_id {
            let mut count_by_type: std::collections::BTreeMap<String, i64> =
                std::collections::BTreeMap::new();
            for item in &items {
                *count_by_type.entry(item.pipe_type.clone()).or_insert(0) += 1;
            }
            for (pipe_type, count) in count_by_type {
                sqlx::query(
                    "UPDATE purchase_order_items SET received_quantity = received_quantity + ? \
                     WHERE id = (SELECT id FROM purchase_order_items \
                      WHERE order_id = ? AND pipe_type = ? AND received_quantity < quantity LIMIT 1)",
                )
                .bind(count)
                .bind(order_id)
                .bind(&pipe_type)
                .execute(&mut *tx)
                .await
                .map_err(AppError::from)?;
            }
        }

        tx.commit().await.map_err(AppError::from)?;
        refresh_inbound_locations(pool, &items.iter().map(|i| InboundPipeItem {
            pipe_type: i.pipe_type.clone(),
            pipe_id: i.pipe_id,
        }).collect::<Vec<_>>()).await?;
        Ok(())
    }

    pub async fn reject_inbound(pool: &PgPool, id: i64, reason: &str) -> Result<(), AppError> {
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
    ///
    /// # Errors
    /// - `AppError::NotFound` — record not found
    pub async fn get_inbound_record(
        pool: &PgPool,
        id: i64,
    ) -> Result<(InboundRecord, Vec<InboundItem>), AppError> {
        let record = InboundRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::NotFound(format!("Inbound record id={} not found", id)))?;

        let items = InboundRepo::find_items(pool, id)
            .await
            .map_err(AppError::from)?;

        Ok((record, items))
    }

    /// Paginated inbound records — filter by date, status, type, whatever.
    /// Returns `(records, total_count)`.
    pub async fn list_inbound_records(
        pool: &PgPool,
        filter: &InboundFilter,
    ) -> Result<(Vec<InboundRecord>, u64), AppError> {
        InboundRepo::list(pool, filter)
            .await
            .map_err(AppError::from)
    }

    /// Soft-deletes an inbound record. Only `auto_approved` or `rejected` ones are fair game.
    ///
    /// # Errors
    /// - `AppError::NotFound` — record not found
    /// - `AppError::Validation` — current status won't let you delete
    pub async fn delete_inbound(pool: &PgPool, id: i64) -> Result<(), AppError> {
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
    /// Only records with `auto_approved` or `rejected` status can be updated.
    ///
    /// # Errors
    /// - `AppError::NotFound` — record not found or was deleted
    /// - `AppError::Validation` — current status doesn't allow updates
    pub async fn update_inbound(
        pool: &PgPool,
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

        InboundRepo::update(pool, id, dto)
            .await
            .map_err(AppError::from)
    }

    /// Gets all line items for a given inbound record.
    pub async fn list_inbound_items(
        pool: &PgPool,
        inbound_id: i64,
    ) -> Result<Vec<InboundItem>, AppError> {
        InboundRepo::find_items(pool, inbound_id)
            .await
            .map_err(AppError::from)
    }

    pub async fn batch_create_inbound(
        pool: &PgPool,
        dto: &BatchCreateInboundRequest,
    ) -> Result<Vec<InboundRecord>, AppError> {
        if dto.records.is_empty() {
            return Err(AppError::Validation(
                "At least one inbound record is required".into(),
            ));
        }

        let mut tx = pool.begin().await.map_err(AppError::from)?;
        let mut results = Vec::with_capacity(dto.records.len());

        for record_dto in &dto.records {
            let inbound_no = utils::generate_no("IN");
            let record = Self::create_inbound_inner(&mut tx, record_dto, &inbound_no).await?;
            if record.approval_status == "auto_approved" {
                Self::execute_inbound_batch_inner(&mut tx, record.id, &record_dto.pipes).await?;
            }
            results.push(record);
        }

        tx.commit().await.map_err(AppError::from)?;

        // Refresh location counts after batch inbound execution (all types)
        for record_dto in &dto.records {
            refresh_inbound_locations(pool, &record_dto.pipes).await?;
        }

        Ok(results)
    }
}
