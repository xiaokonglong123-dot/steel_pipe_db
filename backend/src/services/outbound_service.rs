use sqlx::SqlitePool;

use crate::domain::pipe::PipeType;
use crate::dto::inventory_dto::{
    CreateOutboundRecordRequest, OutboundFilter, OutboundPipeItem, UpdateOutboundRecordRequest,
};
use crate::error::AppError;
use crate::models::inventory::{OutboundItem, OutboundRecord};
use crate::models::screen_pipe::ScreenPipe;
use crate::models::seamless_pipe::SeamlessPipe;
use crate::models::welded_pipe::WeldedPipe;
use crate::repositories::generic_pipe_repo::GenericPipeRepo;
use crate::repositories::inventory_log_repo::InventoryLogRepo;
use crate::repositories::inventory_repo::{CreateInventoryLog, InventoryRepo};
use crate::repositories::location_repo::LocationRepo;
use crate::repositories::outbound_repo::OutboundRepo;
use crate::services::utils;

/// After outbound execution, refresh location used_counts for all affected pipes.
/// This ensures the `used_count` column stays consistent with actual stock.
async fn refresh_outbound_locations(
    pool: &SqlitePool,
    items: &[OutboundPipeItem],
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

/// Outbound service — handles sales, scrapped, and transfer stock-out with create/approve/execute/query.
/// Mirror of inbound: `auto_approved` executes immediately, `pending` needs approval later.
pub struct OutboundService;

impl OutboundService {
    /// Creates an outbound record. Needs at least one pipe; auto-checks every pipe is `in_stock`.
    /// If `auto_approved`, it immediately applies the stock changes.
    ///
    /// # Errors
    /// - `AppError::Validation` — pipe items list is empty
    /// - `AppError::NotFound` — pipe ID doesn't exist
    /// - `AppError::InsufficientStock` — pipe is not `in_stock`
    pub async fn create_outbound(
        pool: &SqlitePool,
        dto: &CreateOutboundRecordRequest,
    ) -> Result<OutboundRecord, AppError> {
        if dto.pipes.is_empty() {
            return Err(AppError::Validation("At least one pipe is required".into()));
        }

        // Batch query all pipes to fix N+1 problem
        let seamless_ids: Vec<i64> = dto
            .pipes
            .iter()
            .filter(|item| {
                PipeType::from_pipe_type_str(&item.pipe_type) == Some(PipeType::Seamless)
            })
            .map(|item| item.pipe_id)
            .collect();
        let screen_ids: Vec<i64> = dto
            .pipes
            .iter()
            .filter(|item| PipeType::from_pipe_type_str(&item.pipe_type) == Some(PipeType::Screen))
            .map(|item| item.pipe_id)
            .collect();
        let welded_ids: Vec<i64> = dto
            .pipes
            .iter()
            .filter(|item| PipeType::from_pipe_type_str(&item.pipe_type) == Some(PipeType::Welded))
            .map(|item| item.pipe_id)
            .collect();

        let seamless_pipes = GenericPipeRepo::<SeamlessPipe>::find_by_ids(pool, &seamless_ids).await?;
        let screen_pipes = GenericPipeRepo::<ScreenPipe>::find_by_ids(pool, &screen_ids).await?;
        let welded_pipes = GenericPipeRepo::<WeldedPipe>::find_by_ids(pool, &welded_ids).await?;

        let seamless_map: std::collections::HashMap<i64, _> =
            seamless_pipes.iter().map(|p| (p.id, &p.status)).collect();
        let screen_map: std::collections::HashMap<i64, _> =
            screen_pipes.iter().map(|p| (p.id, &p.status)).collect();
        let welded_map: std::collections::HashMap<i64, _> =
            welded_pipes.iter().map(|p| (p.id, &p.status)).collect();

        for item in &dto.pipes {
            let pipe_type = PipeType::from_pipe_type_str(&item.pipe_type).ok_or_else(|| {
                AppError::Validation(format!("Unknown pipe_type: {}", item.pipe_type))
            })?;

            match pipe_type {
                PipeType::Seamless => {
                    let status = seamless_map.get(&item.pipe_id).ok_or_else(|| {
                        AppError::NotFound(format!("Seamless pipe id={} not found", item.pipe_id))
                    })?;
                    if status.as_str() != "in_stock" {
                        return Err(AppError::InsufficientStock("Insufficient stock".into()));
                    }
                }
                PipeType::Screen => {
                    let status = screen_map.get(&item.pipe_id).ok_or_else(|| {
                        AppError::NotFound(format!("Screen pipe id={} not found", item.pipe_id))
                    })?;
                    if status.as_str() != "in_stock" {
                        return Err(AppError::InsufficientStock("Insufficient stock".into()));
                    }
                }
                PipeType::Welded => {
                    let status = welded_map.get(&item.pipe_id).ok_or_else(|| {
                        AppError::NotFound(format!("Welded pipe id={} not found", item.pipe_id))
                    })?;
                    if status.as_str() != "in_stock" {
                        return Err(AppError::InsufficientStock("Insufficient stock".into()));
                    }
                }
            }
        }

        let outbound_no = utils::generate_no("OUT");

        let record = OutboundRepo::create_with_items(pool, dto, &outbound_no)
            .await
            .map_err(AppError::from)?;

        if record.approval_status == "auto_approved" {
            Self::execute_outbound_batch(pool, record.id, &record.outbound_type, None, &dto.pipes)
                .await?;
        }

        Ok(record)
    }

    /// Applies outbound stock changes for all pipe items in a single transaction.
    /// If any item fails, the entire batch is rolled back.
    async fn execute_outbound_batch(
        pool: &SqlitePool,
        record_id: i64,
        outbound_type: &str,
        created_by: Option<i64>,
        items: &[crate::dto::inventory_dto::OutboundPipeItem],
    ) -> Result<(), AppError> {
        let mut tx = pool.begin().await.map_err(AppError::from)?;
        let next_status = if outbound_type == "scrapped" {
            "scrapped"
        } else {
            "outbound"
        };

        for item in items {
            let pipe_type = PipeType::from_pipe_type_str(&item.pipe_type).ok_or_else(|| {
                AppError::Validation(format!("Unknown pipe_type: {}", item.pipe_type))
            })?;

            match pipe_type {
                PipeType::Seamless | PipeType::Screen | PipeType::Welded => {
                    let affected = InventoryRepo::update_pipe_status_with_stock_check(
                        &mut *tx,
                        &item.pipe_type,
                        item.pipe_id,
                        next_status,
                    )
                    .await
                    .map_err(AppError::from)?;
                    if affected == 0 {
                        return Err(AppError::InsufficientStock("Insufficient stock".into()));
                    }
                }
            }

            InventoryLogRepo::create_in_transaction(
                &mut tx,
                &CreateInventoryLog {
                    pipe_type: item.pipe_type.clone(),
                    pipe_id: item.pipe_id,
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
        refresh_outbound_locations(pool, items).await?;
        Ok(())
    }

    /// Approves a pending outbound and deducts stock (pipe status → `outbound` + inventory log).
    /// Outbound must be in `pending` state.
    ///
    /// # Errors
    /// - `AppError::NotFound` — record doesn't exist or was deleted
    /// - `AppError::Validation` — current state won't allow approval
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

        let items = OutboundRepo::find_items(pool, id)
            .await
            .map_err(AppError::from)?;

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

        let next_status = if record.outbound_type == "scrapped" {
            "scrapped"
        } else {
            "outbound"
        };

        for item in &items {
            let pipe_type = PipeType::from_pipe_type_str(&item.pipe_type).ok_or_else(|| {
                AppError::Validation(format!("Unknown pipe_type: {}", item.pipe_type))
            })?;

            match pipe_type {
                PipeType::Seamless | PipeType::Screen | PipeType::Welded => {
                    let affected = InventoryRepo::update_pipe_status_with_stock_check(
                        &mut *tx,
                        &item.pipe_type,
                        item.pipe_id,
                        next_status,
                    )
                    .await
                    .map_err(AppError::from)?;
                    if affected == 0 {
                        return Err(AppError::InsufficientStock("Insufficient stock".into()));
                    }
                }
            }

            InventoryLogRepo::create_in_transaction(
                &mut tx,
                &CreateInventoryLog {
                    pipe_type: item.pipe_type.clone(),
                    pipe_id: item.pipe_id,
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

        // Increment delivered_quantity on linked sales order if order_id is present
        if let Some(order_id) = record.order_id {
            let mut count_by_type: std::collections::BTreeMap<String, i64> =
                std::collections::BTreeMap::new();
            for item in &items {
                *count_by_type.entry(item.pipe_type.clone()).or_insert(0) += 1;
            }
            for (pipe_type, count) in count_by_type {
                sqlx::query(
                    "UPDATE sales_order_items SET delivered_quantity = delivered_quantity + ? \
                     WHERE id = (SELECT id FROM sales_order_items \
                      WHERE order_id = ? AND pipe_type = ? AND delivered_quantity < quantity LIMIT 1)",
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
        refresh_outbound_locations(
            pool,
            &items
                .iter()
                .map(|i| OutboundPipeItem {
                    pipe_type: i.pipe_type.clone(),
                    pipe_id: i.pipe_id,
                })
                .collect::<Vec<_>>(),
        )
        .await?;
        Ok(())
    }

    /// Rejects a pending outbound — sets `rejected` and stores the reason. No stock changes.
    ///
    /// # Errors
    /// - `AppError::NotFound` — record not found
    /// - `AppError::Validation` — can't reject in this state
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

    /// Fetches an outbound record with all line items. Returns `(record, items)` tuple.
    ///
    /// # Errors
    /// - `AppError::NotFound` — record not found
    pub async fn get_outbound_record(
        pool: &SqlitePool,
        id: i64,
    ) -> Result<(OutboundRecord, Vec<OutboundItem>), AppError> {
        let record = OutboundRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::NotFound(format!("Outbound record id={} not found", id)))?;

        let items = OutboundRepo::find_items(pool, id)
            .await
            .map_err(AppError::from)?;

        Ok((record, items))
    }

    /// Paginated outbound records — filter by date, status, type, etc.
    /// Returns `(records, total_count)`.
    pub async fn list_outbound_records(
        pool: &SqlitePool,
        filter: &OutboundFilter,
    ) -> Result<(Vec<OutboundRecord>, u64), AppError> {
        OutboundRepo::list(pool, filter)
            .await
            .map_err(AppError::from)
    }

    /// Soft-deletes an outbound record. Only `auto_approved` or `rejected` ones can be deleted.
    ///
    /// # Errors
    /// - `AppError::NotFound` — record not found
    /// - `AppError::Validation` — current state doesn't allow deletion
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
    /// Only records with `auto_approved` or `rejected` status can be updated.
    ///
    /// # Errors
    /// - `AppError::NotFound` — record not found or was deleted
    /// - `AppError::Validation` — current status doesn't allow updates
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

        OutboundRepo::update(pool, id, dto)
            .await
            .map_err(AppError::from)
    }

    /// Gets all line items for a given outbound record.
    pub async fn list_outbound_items(
        pool: &SqlitePool,
        outbound_id: i64,
    ) -> Result<Vec<OutboundItem>, AppError> {
        OutboundRepo::find_items(pool, outbound_id)
            .await
            .map_err(AppError::from)
    }
}
