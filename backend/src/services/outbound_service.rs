use chrono::Utc;
use sqlx::SqlitePool;

use crate::dto::inventory_dto::{CreateOutboundRecordRequest, OutboundFilter};
use crate::error::AppError;
use crate::models::inventory::{OutboundItem, OutboundRecord};
use crate::repositories::inventory_repo::OutboundRepo;
use crate::repositories::pipe_repo::{SeamlessPipeRepo, ScreenPipeRepo};

/// Outbound service — handles sales, scrapped, and transfer stock-out with create/approve/execute/query.
/// Mirror of inbound: `auto_approved` executes immediately, `pending` needs approval later.
pub struct OutboundService;

impl OutboundService {
    fn generate_no(prefix: &str) -> String {
        let now = Utc::now();
        let date_str = now.format("%Y%m%d").to_string();
        let serial = uuid::Uuid::new_v4().to_string();
        let short_serial = &serial[..8];
        format!("{}-{}-{}", prefix, date_str, short_serial)
    }

    /// Creates an outbound record. Needs at least one pipe; auto-checks every pipe is `in_stock`.
    /// If `auto_approved`, it immediately applies the stock changes.
    ///
    /// # Errors
    /// - `AppError::Validation` — pipe items list is empty
    /// - `AppError::NotFound` — pipe ID doesn't exist
    /// - `AppError::InsufficientStock` — pipe ain't `in_stock`
    pub async fn create_outbound(
        pool: &SqlitePool,
        dto: &CreateOutboundRecordRequest,
    ) -> Result<OutboundRecord, AppError> {
        if dto.pipes.is_empty() {
            return Err(AppError::Validation("At least one pipe is required".into()));
        }

        // Batch query all pipes to fix N+1 problem
        let seamless_ids: Vec<i64> = dto.pipes.iter()
            .filter(|item| matches!(item.pipe_type.as_str(), "seamless" | "casing" | "tubing"))
            .map(|item| item.pipe_id)
            .collect();
        let screen_ids: Vec<i64> = dto.pipes.iter()
            .filter(|item| matches!(item.pipe_type.as_str(), "screen" | "screened"))
            .map(|item| item.pipe_id)
            .collect();

        let seamless_pipes = SeamlessPipeRepo::find_by_ids(pool, &seamless_ids).await?;
        let screen_pipes = ScreenPipeRepo::find_by_ids(pool, &screen_ids).await?;

        let seamless_map: std::collections::HashMap<i64, _> = seamless_pipes.iter()
            .map(|p| (p.id, &p.status))
            .collect();
        let screen_map: std::collections::HashMap<i64, _> = screen_pipes.iter()
            .map(|p| (p.id, &p.status))
            .collect();

        for item in &dto.pipes {
            match item.pipe_type.as_str() {
                "seamless" | "casing" | "tubing" => {
                    let status = seamless_map.get(&item.pipe_id)
                        .ok_or_else(|| AppError::NotFound(format!(
                            "Seamless pipe id={} not found", item.pipe_id
                        )))?;
                    if status.as_str() != "in_stock" {
                        return Err(AppError::InsufficientStock);
                    }
                }
                "screen" | "screened" => {
                    let status = screen_map.get(&item.pipe_id)
                        .ok_or_else(|| AppError::NotFound(format!(
                            "Screen pipe id={} not found", item.pipe_id
                        )))?;
                    if status.as_str() != "in_stock" {
                        return Err(AppError::InsufficientStock);
                    }
                }
                _ => return Err(AppError::Validation(format!(
                    "Unknown pipe_type: {}", item.pipe_type
                ))),
            }
        }

        let outbound_no = Self::generate_no("OUT");

        let record = OutboundRepo::create_with_items(pool, dto, &outbound_no)
            .await
            .map_err(AppError::from)?;

        if record.approval_status == "auto_approved" {
            for item in &dto.pipes {
                Self::execute_outbound(pool, record.id, item).await?;
            }
        }

        Ok(record)
    }

    async fn execute_outbound(
        pool: &SqlitePool,
        record_id: i64,
        item: &crate::dto::inventory_dto::OutboundPipeItem,
    ) -> Result<(), AppError> {
        let mut tx = pool.begin().await.map_err(AppError::from)?;
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        match item.pipe_type.as_str() {
            "seamless" | "casing" | "tubing" => {
                sqlx::query(
                    "UPDATE seamless_pipes SET status = 'outbound', updated_at = ? WHERE id = ? AND deleted_at IS NULL",
                )
                .bind(&now)
                .bind(item.pipe_id)
                .execute(&mut *tx)
                .await
                .map_err(AppError::from)?;
            }
            "screen" | "screened" => {
                sqlx::query(
                    "UPDATE screen_pipes SET status = 'outbound', updated_at = ? WHERE id = ? AND deleted_at IS NULL",
                )
                .bind(&now)
                .bind(item.pipe_id)
                .execute(&mut *tx)
                .await
                .map_err(AppError::from)?;
            }
            _ => {
                return Err(AppError::Validation(format!(
                    "Unknown pipe_type: {}",
                    item.pipe_type
                )));
            }
        }

        sqlx::query(
            "INSERT INTO inventory_logs (pipe_type, pipe_id, change_type, ref_type, ref_id, \
             from_location_id, to_location_id, notes, created_by) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&item.pipe_type)
        .bind(item.pipe_id)
        .bind("outbound")
        .bind(Some("outbound"))
        .bind(Some(record_id))
        .bind(None::<i64>)
        .bind(None::<i64>)
        .bind(None::<String>)
        .bind(None::<i64>)
        .execute(&mut *tx)
        .await
        .map_err(AppError::from)?;

        tx.commit().await.map_err(AppError::from)?;
        Ok(())
    }

    /// Approves a pending outbound and deducts stock (pipe status → `outbound` + inventory log).
    /// Outbound must be in `pending` state.
    ///
    /// # Errors
    /// - `AppError::NotFound` — record doesn't exist or was deleted
    /// - `AppError::Validation` — current state won't allow approval
    pub async fn approve_outbound(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
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
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        sqlx::query(
            "UPDATE outbound_records SET approval_status = 'approved', \
             rejection_reason = NULL, updated_at = datetime('now') \
             WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(AppError::from)?;

        for item in &items {
            match item.pipe_type.as_str() {
                "seamless" | "casing" | "tubing" => {
                    sqlx::query(
                        "UPDATE seamless_pipes SET status = 'outbound', updated_at = ? WHERE id = ? AND deleted_at IS NULL",
                    )
                    .bind(&now)
                    .bind(item.pipe_id)
                    .execute(&mut *tx)
                    .await
                    .map_err(AppError::from)?;
                }
                "screen" | "screened" => {
                    sqlx::query(
                        "UPDATE screen_pipes SET status = 'outbound', updated_at = ? WHERE id = ? AND deleted_at IS NULL",
                    )
                    .bind(&now)
                    .bind(item.pipe_id)
                    .execute(&mut *tx)
                    .await
                    .map_err(AppError::from)?;
                }
                _ => {
                    return Err(AppError::Validation(format!(
                        "Unknown pipe_type: {}",
                        item.pipe_type
                    )));
                }
            }

            sqlx::query(
                "INSERT INTO inventory_logs (pipe_type, pipe_id, change_type, ref_type, ref_id, \
                 from_location_id, to_location_id, notes, created_by) \
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&item.pipe_type)
            .bind(item.pipe_id)
            .bind("outbound")
            .bind(Some("outbound"))
            .bind(Some(id))
            .bind(Option::<i64>::None)
            .bind(Option::<i64>::None)
            .bind(Option::<String>::None)
            .bind(Option::<i64>::None)
            .execute(&mut *tx)
            .await
            .map_err(AppError::from)?;
        }

        tx.commit().await.map_err(AppError::from)?;
        Ok(())
    }

    /// Rejects a pending outbound — sets `rejected` and stores the reason. No stock changes.
    ///
    /// # Errors
    /// - `AppError::NotFound` — record not found
    /// - `AppError::Validation` — can't reject in this state
    pub async fn reject_outbound(
        pool: &SqlitePool,
        id: i64,
        reason: &str,
    ) -> Result<(), AppError> {
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

        OutboundRepo::delete(pool, id)
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
