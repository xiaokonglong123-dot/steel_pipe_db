use sqlx::SqlitePool;

use crate::dto::common::PaginationParams;
use crate::dto::inventory_dto::{CreateCheckRequest, SubmitCheckItemRequest};
use crate::error::AppError;
use crate::models::inventory::{InventoryCheckItem, InventoryCheckRecord};
use crate::inventory::check_repo::CheckRepo;
use crate::inventory::inventory_repo::{CheckInitItem, InventoryRepo};
use crate::utils;

/// Inventory check service — create check orders, submit results per item, complete the full workflow.
/// On creation, it auto-initializes all items with positive on-hand stock as pending check items.
pub struct CheckService;

impl CheckService {
    /// Creates a check order. Auto-scans all stocked items into check items and generates a CHK-prefixed number.
    pub async fn create_check(
        pool: &SqlitePool,
        dto: &CreateCheckRequest,
    ) -> Result<InventoryCheckRecord, AppError> {
        let check_no = utils::generate_no("CHK");

        let stock = InventoryRepo::list_stock(pool, dto.location_id)
            .await
            .map_err(AppError::from)?;

        let items: Vec<CheckInitItem> = stock
            .into_iter()
            .map(|(item_id, expected_quantity)| CheckInitItem {
                item_id,
                expected_quantity,
            })
            .collect();

        CheckRepo::create(pool, dto, &check_no, &items)
            .await
            .map_err(AppError::from)
    }

    /// Gets a check record with all its items. Returns `(record, items)` tuple.
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

    /// Paginated list of check records.
    pub async fn list_checks(
        pool: &SqlitePool,
        params: &PaginationParams,
    ) -> Result<(Vec<InventoryCheckRecord>, u64), AppError> {
        CheckRepo::list(pool, params).await.map_err(AppError::from)
    }

    /// Submits the actual result for a check item (`found_quantity` + `notes`).
    /// Only works on `in_progress` checks.
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

        CheckRepo::update_item_result(pool, check_id, item_id, dto.found_quantity, &dto.notes)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "Check item id={} not found in check id={}",
                    item_id, check_id
                ))
            })
    }

    /// Completes a check — sets status to `completed` and returns the mismatch count.
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
