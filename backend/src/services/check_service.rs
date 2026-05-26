use chrono::Utc;
use sqlx::SqlitePool;

use crate::dto::common::PaginationParams;
use crate::dto::inventory_dto::{CreateCheckRequest, SubmitCheckItemRequest};
use crate::error::AppError;
use crate::models::inventory::{InventoryCheckItem, InventoryCheckRecord};
use crate::repositories::inventory_repo::{CheckInitItem, CheckRepo};

pub struct CheckService;

impl CheckService {
    fn generate_no(prefix: &str) -> String {
        let now = Utc::now();
        let date_str = now.format("%Y%m%d").to_string();
        let serial = uuid::Uuid::new_v4().to_string();
        let short_serial = &serial[..8];
        format!("{}-{}-{}", prefix, date_str, short_serial)
    }

    pub async fn create_check(
        pool: &SqlitePool,
        dto: &CreateCheckRequest,
    ) -> Result<InventoryCheckRecord, AppError> {
        let check_no = Self::generate_no("CHK");

        let mut items: Vec<CheckInitItem> = Vec::new();

        let seamless_pipes: Vec<(i64,)> = sqlx::query_as(
            "SELECT id FROM seamless_pipes WHERE status = 'in_stock' AND deleted_at IS NULL",
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        for (id,) in seamless_pipes {
            items.push(CheckInitItem {
                pipe_type: "seamless".into(),
                pipe_id: id,
                expected_status: "in_stock".into(),
            });
        }

        let screen_pipes: Vec<(i64,)> = sqlx::query_as(
            "SELECT id FROM screen_pipes WHERE status = 'in_stock' AND deleted_at IS NULL",
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        for (id,) in screen_pipes {
            items.push(CheckInitItem {
                pipe_type: "screen".into(),
                pipe_id: id,
                expected_status: "in_stock".into(),
            });
        }

        CheckRepo::create(pool, dto, &check_no, &items)
            .await
            .map_err(AppError::from)
    }

    pub async fn get_check_detail(
        pool: &SqlitePool,
        id: i64,
    ) -> Result<(InventoryCheckRecord, Vec<InventoryCheckItem>), AppError> {
        let record = CheckRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::NotFound(format!("Check record id={} not found", id)))?;

        let items = CheckRepo::get_check_items(pool, id)
            .await
            .map_err(AppError::from)?;

        Ok((record, items))
    }

    pub async fn list_checks(
        pool: &SqlitePool,
        params: &PaginationParams,
    ) -> Result<(Vec<InventoryCheckRecord>, u64), AppError> {
        CheckRepo::list(pool, params)
            .await
            .map_err(AppError::from)
    }

    pub async fn submit_check_item(
        pool: &SqlitePool,
        check_id: i64,
        item_id: i64,
        dto: &SubmitCheckItemRequest,
    ) -> Result<InventoryCheckItem, AppError> {
        let record = CheckRepo::find_by_id(pool, check_id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::NotFound(format!("Check record id={} not found", check_id)))?;

        if record.status != "in_progress" {
            return Err(AppError::Validation(format!(
                "Check id={} is not in progress (status: {})",
                check_id, record.status
            )));
        }

        CheckRepo::update_item_result(pool, check_id, item_id, &dto.found_status, &dto.notes)
            .await
            .map_err(AppError::from)
    }

    pub async fn complete_check(
        pool: &SqlitePool,
        check_id: i64,
    ) -> Result<serde_json::Value, AppError> {
        let record = CheckRepo::find_by_id(pool, check_id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::NotFound(format!("Check record id={} not found", check_id)))?;

        if record.status != "in_progress" {
            return Err(AppError::Validation(format!(
                "Cannot complete check with status '{}'. Only in_progress checks can be completed.",
                record.status
            )));
        }

        CheckRepo::update_status(pool, check_id, "completed")
            .await
            .map_err(AppError::from)?;

        let mismatch_count = CheckRepo::get_mismatch_count(pool, check_id)
            .await
            .map_err(AppError::from)?;

        Ok(serde_json::json!({
            "check_id": check_id,
            "status": "completed",
            "mismatch_count": mismatch_count,
            "message": format!("Check completed with {} mismatches", mismatch_count),
        }))
    }
}
