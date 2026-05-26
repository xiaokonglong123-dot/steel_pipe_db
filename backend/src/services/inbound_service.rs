use chrono::Utc;
use sqlx::SqlitePool;

use crate::dto::inventory_dto::{
    BatchCreateInboundRequest, CreateInboundRecordRequest, InboundFilter,
};
use crate::error::AppError;
use crate::models::inventory::{InboundItem, InboundRecord};
use crate::repositories::inventory_repo::InboundRepo;

pub struct InboundService;

impl InboundService {
    fn generate_no(prefix: &str) -> String {
        let now = Utc::now();
        let date_str = now.format("%Y%m%d").to_string();
        let serial = uuid::Uuid::new_v4().to_string();
        let short_serial = &serial[..8];
        format!("{}-{}-{}", prefix, date_str, short_serial)
    }

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
            for item in &dto.pipes {
                Self::execute_inbound(pool, record.id, item).await?;
            }
        }

        Ok(record)
    }

    async fn execute_inbound(
        pool: &SqlitePool,
        record_id: i64,
        item: &crate::dto::inventory_dto::InboundPipeItem,
    ) -> Result<(), AppError> {
        let mut tx = pool.begin().await.map_err(AppError::from)?;
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        match item.pipe_type.as_str() {
            "seamless" | "casing" | "tubing" => {
                sqlx::query(
                    "UPDATE seamless_pipes SET status = 'in_stock', updated_at = ? WHERE id = ? AND deleted_at IS NULL",
                )
                .bind(&now)
                .bind(item.pipe_id)
                .execute(&mut *tx)
                .await
                .map_err(AppError::from)?;
            }
            "screen" | "screened" => {
                sqlx::query(
                    "UPDATE screen_pipes SET status = 'in_stock', updated_at = ? WHERE id = ? AND deleted_at IS NULL",
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
        .bind("inbound")
        .bind(Some("inbound"))
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

    pub async fn approve_inbound(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
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

        sqlx::query(
            "UPDATE inbound_records SET approval_status = 'approved', \
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
                        "UPDATE seamless_pipes SET status = 'in_stock', updated_at = ? WHERE id = ? AND deleted_at IS NULL",
                    )
                    .bind(&now)
                    .bind(item.pipe_id)
                    .execute(&mut *tx)
                    .await
                    .map_err(AppError::from)?;
                }
                "screen" | "screened" => {
                    sqlx::query(
                        "UPDATE screen_pipes SET status = 'in_stock', updated_at = ? WHERE id = ? AND deleted_at IS NULL",
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

    pub async fn list_inbound_records(
        pool: &SqlitePool,
        filter: &InboundFilter,
    ) -> Result<(Vec<InboundRecord>, u64), AppError> {
        InboundRepo::list(pool, filter)
            .await
            .map_err(AppError::from)
    }

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

    pub async fn list_inbound_items(
        pool: &SqlitePool,
        inbound_id: i64,
    ) -> Result<Vec<InboundItem>, AppError> {
        InboundRepo::find_items(pool, inbound_id)
            .await
            .map_err(AppError::from)
    }

    pub async fn batch_create_inbound(
        pool: &SqlitePool,
        dto: &BatchCreateInboundRequest,
    ) -> Result<Vec<InboundRecord>, AppError> {
        if dto.records.is_empty() {
            return Err(AppError::Validation("At least one inbound record is required".into()));
        }

        let mut results = Vec::new();
        for record_dto in &dto.records {
            let record = Self::create_inbound(pool, record_dto).await?;
            results.push(record);
        }

        Ok(results)
    }
}
