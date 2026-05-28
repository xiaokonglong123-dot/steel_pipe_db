use chrono::Utc;
use sqlx::{SqlitePool, Transaction};
use sqlx::sqlite::Sqlite;

use crate::domain::pipe::PipeType;
use crate::dto::inventory_dto::{
    BatchCreateInboundRequest, CreateInboundRecordRequest, InboundFilter,
};
use crate::error::AppError;
use crate::models::inventory::{InboundItem, InboundRecord};
use crate::repositories::inventory_repo::InboundRepo;

/// Inbound service — handles purchase, production, and return stock-in with create/approve/execute/query.
/// Auto-approved inbound kicks off stock changes right away; pending ones need a separate `approve_inbound` call.
pub struct InboundService;

impl InboundService {
    fn generate_no(prefix: &str) -> String {
        let now = Utc::now();
        let date_str = now.format("%Y%m%d").to_string();
        let serial = uuid::Uuid::new_v4().to_string();
        let short_serial = &serial[..8];
        format!("{}-{}-{}", prefix, date_str, short_serial)
    }

    /// Creates an inbound record. Needs at least one pipe item.
    /// If `auto_approved`, applies all stock changes in a single transaction
    /// (updates pipe status + writes logs) to ensure atomicity.
    ///
    /// # Errors
    /// - `AppError::Validation` — pipe items list is empty
    pub async fn create_inbound(
        pool: &SqlitePool,
        dto: &CreateInboundRecordRequest,
    ) -> Result<InboundRecord, AppError> {
        if dto.pipes.is_empty() {
            return Err(AppError::Validation("At least one pipe is required".into()));
        }

        let inbound_no = Self::generate_no("IN");

        let record = InboundRepo::create_with_items(pool, dto, &inbound_no)
            .await
            .map_err(AppError::from)?;

        if record.approval_status == "auto_approved" {
            Self::execute_inbound_batch(pool, record.id, &dto.pipes).await?;
        }

        Ok(record)
    }

    async fn execute_inbound_batch_inner(
        tx: &mut Transaction<'_, Sqlite>,
        record_id: i64,
        items: &[crate::dto::inventory_dto::InboundPipeItem],
    ) -> Result<(), AppError> {
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        for item in items {
            let pipe_type = PipeType::from_pipe_type_str(&item.pipe_type)
                .ok_or_else(|| AppError::Validation(format!("Unknown pipe_type: {}", item.pipe_type)))?;

            match pipe_type {
                PipeType::Seamless => {
                    let result = sqlx::query(
                        "UPDATE seamless_pipes SET status = 'in_stock', updated_at = ? WHERE id = ? AND deleted_at IS NULL",
                    )
                    .bind(&now)
                    .bind(item.pipe_id)
                    .execute(&mut **tx)
                    .await
                    .map_err(AppError::from)?;
                    if result.rows_affected() == 0 {
                        return Err(AppError::NotFound(format!(
                            "Seamless pipe id={} not found for inbound execution", item.pipe_id
                        )));
                    }
                }
                PipeType::Screen => {
                    let result = sqlx::query(
                        "UPDATE screen_pipes SET status = 'in_stock', updated_at = ? WHERE id = ? AND deleted_at IS NULL",
                    )
                    .bind(&now)
                    .bind(item.pipe_id)
                    .execute(&mut **tx)
                    .await
                    .map_err(AppError::from)?;
                    if result.rows_affected() == 0 {
                        return Err(AppError::NotFound(format!(
                            "Screen pipe id={} not found for inbound execution", item.pipe_id
                        )));
                    }
                }
            }

            sqlx::query(
                "INSERT INTO inventory_logs (pipe_type, pipe_id, change_type, ref_type, ref_id, \
                 from_location_id, to_location_id, notes, created_by) \
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&item.pipe_type)
            .bind(item.pipe_id)
            .bind("inbound")
            .bind(Some("inbound"))
            .bind(Some(record_id))
            .bind(None::<i64>)
            .bind(None::<i64>)
            .bind(None::<String>)
            .bind(None::<i64>)
            .execute(&mut **tx)
            .await
            .map_err(AppError::from)?;
        }

        Ok(())
    }

    /// Applies inbound stock changes for all pipe items in a single transaction.
    /// If any item fails, the entire batch is rolled back.
    async fn execute_inbound_batch(
        pool: &SqlitePool,
        record_id: i64,
        items: &[crate::dto::inventory_dto::InboundPipeItem],
    ) -> Result<(), AppError> {
        let mut tx = pool.begin().await.map_err(AppError::from)?;
        Self::execute_inbound_batch_inner(&mut tx, record_id, items).await?;
        tx.commit().await.map_err(AppError::from)?;
        Ok(())
    }

    async fn create_inbound_inner(
        tx: &mut Transaction<'_, Sqlite>,
        dto: &CreateInboundRecordRequest,
        inbound_no: &str,
    ) -> Result<InboundRecord, AppError> {
        let record = sqlx::query_as::<_, InboundRecord>(
            "INSERT INTO inbound_records (inbound_no, inbound_type, order_id, supplier_id, notes, approval_status) \
             VALUES (?, ?, ?, ?, ?, ?) \
             RETURNING id, inbound_no, inbound_type, order_id, supplier_id, notes, approval_status, \
               rejection_reason, approval_reason, handled_by, handled_at, created_at, updated_at, deleted_at",
        )
        .bind(inbound_no)
        .bind(&dto.inbound_type)
        .bind(dto.order_id)
        .bind(dto.supplier_id)
        .bind(&dto.notes)
        .bind(if dto.inbound_type == "purchase" {
            "auto_approved"
        } else {
            "pending"
        })
        .fetch_one(&mut **tx)
        .await
        .map_err(AppError::from)?;

        for item in &dto.pipes {
            sqlx::query(
                "INSERT INTO inbound_items (inbound_id, pipe_type, pipe_id) VALUES (?, ?, ?)",
            )
            .bind(record.id)
            .bind(&item.pipe_type)
            .bind(item.pipe_id)
            .execute(&mut **tx)
            .await
            .map_err(AppError::from)?;
        }

        Ok(record)
    }

    /// Approves a pending inbound record and applies the stock changes (pipe status + log).
    /// Inbound record must be in `pending` state.
    ///
    /// # Errors
    /// - `AppError::NotFound` — record doesn't exist or was deleted
    /// - `AppError::Validation` — current status does not allow approval
    pub async fn approve_inbound(
        pool: &SqlitePool,
        id: i64,
        approval_reason: Option<&str>,
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
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let result = sqlx::query(
            "UPDATE inbound_records SET approval_status = 'approved', \
             rejection_reason = NULL, approval_reason = ?, handled_by = ?, handled_at = datetime('now'), updated_at = datetime('now') \
             WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(approval_reason)
        .bind(Option::<i64>::None)
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(AppError::from)?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!(
                "Inbound record id={} not found or was deleted during approval", id
            )));
        }

        for item in &items {
            let pipe_type = PipeType::from_pipe_type_str(&item.pipe_type)
                .ok_or_else(|| AppError::Validation(format!("Unknown pipe_type: {}", item.pipe_type)))?;

            match pipe_type {
                PipeType::Seamless => {
                    let result = sqlx::query(
                        "UPDATE seamless_pipes SET status = 'in_stock', updated_at = ? WHERE id = ? AND deleted_at IS NULL",
                    )
                    .bind(&now)
                    .bind(item.pipe_id)
                    .execute(&mut *tx)
                    .await
                    .map_err(AppError::from)?;
                    if result.rows_affected() == 0 {
                        return Err(AppError::NotFound(format!(
                            "Seamless pipe id={} not found during inbound approval", item.pipe_id
                        )));
                    }
                }
                PipeType::Screen => {
                    let result = sqlx::query(
                        "UPDATE screen_pipes SET status = 'in_stock', updated_at = ? WHERE id = ? AND deleted_at IS NULL",
                    )
                    .bind(&now)
                    .bind(item.pipe_id)
                    .execute(&mut *tx)
                    .await
                    .map_err(AppError::from)?;
                    if result.rows_affected() == 0 {
                        return Err(AppError::NotFound(format!(
                            "Screen pipe id={} not found during inbound approval", item.pipe_id
                        )));
                    }
                }
            }

            sqlx::query(
                "INSERT INTO inventory_logs (pipe_type, pipe_id, change_type, ref_type, ref_id, \
                 from_location_id, to_location_id, notes, created_by) \
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&item.pipe_type)
            .bind(item.pipe_id)
            .bind("inbound")
            .bind(Some("inbound"))
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

    /// Rejects a pending inbound — sets status to `rejected` and saves the reason. No stock changes happen.
    ///
    /// # Errors
    /// - `AppError::NotFound` — record not found
    /// - `AppError::Validation` — can't reject in the current state
    pub async fn reject_inbound(
        pool: &SqlitePool,
        id: i64,
        reason: &str,
    ) -> Result<(), AppError> {
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
        pool: &SqlitePool,
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
        pool: &SqlitePool,
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

        InboundRepo::delete(pool, id)
            .await
            .map_err(AppError::from)
    }

    /// Gets all line items for a given inbound record.
    pub async fn list_inbound_items(
        pool: &SqlitePool,
        inbound_id: i64,
    ) -> Result<Vec<InboundItem>, AppError> {
        InboundRepo::find_items(pool, inbound_id)
            .await
            .map_err(AppError::from)
    }

    /// Batch-creates inbound records inside a single transaction.
    /// If any record fails, the entire batch is rolled back.
    ///
    /// # Errors
    /// - `AppError::Validation` — record list is empty
    pub async fn batch_create_inbound(
        pool: &SqlitePool,
        dto: &BatchCreateInboundRequest,
    ) -> Result<Vec<InboundRecord>, AppError> {
        if dto.records.is_empty() {
            return Err(AppError::Validation("At least one inbound record is required".into()));
        }

        let mut tx = pool.begin().await.map_err(AppError::from)?;
        let mut results = Vec::with_capacity(dto.records.len());

        for record_dto in &dto.records {
            let inbound_no = Self::generate_no("IN");
            let record = Self::create_inbound_inner(&mut tx, record_dto, &inbound_no).await?;
            if record.approval_status == "auto_approved" {
                Self::execute_inbound_batch_inner(&mut tx, record.id, &record_dto.pipes).await?;
            }
            results.push(record);
        }

        tx.commit().await.map_err(AppError::from)?;
        Ok(results)
    }
}
