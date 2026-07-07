use sqlx::SqlitePool;

use crate::dto::common::PaginationParams;
use crate::dto::inventory_dto::CreateCheckRequest;
use crate::models::inventory::{InventoryCheckItem, InventoryCheckRecord};

use super::inventory_repo::CheckInitItem;

/// CRUD for inventory check records and items (`inventory_check_records` + `inventory_check_items`).
/// All queries filter `deleted_at IS NULL`.
pub struct CheckRepo;

impl CheckRepo {
    /// INSERT into `inventory_check_records` + `inventory_check_items` in a single transaction.
    /// Status starts as `in_progress`. Returns the created `InventoryCheckRecord`.
    pub async fn create(
        pool: &SqlitePool,
        dto: &CreateCheckRequest,
        check_no: &str,
        items: &[CheckInitItem],
    ) -> Result<InventoryCheckRecord, sqlx::Error> {
        let mut tx = pool.begin().await?;

        let record = sqlx::query_as::<_, InventoryCheckRecord>(
            "INSERT INTO inventory_check_records (check_no, location_id, status, notes) \
             VALUES (?, ?, 'in_progress', ?) \
             RETURNING id, check_no, location_id, status, notes, created_by, \
               created_at, updated_at, deleted_at",
        )
        .bind(check_no)
        .bind(dto.location_id)
        .bind(&dto.notes)
        .fetch_one(&mut *tx)
        .await?;

        for item in items {
            sqlx::query(
                "INSERT INTO inventory_check_items (check_id, pipe_type, pipe_id, expected_status) \
                 VALUES (?, ?, ?, ?)",
            )
            .bind(record.id)
            .bind(&item.pipe_type)
            .bind(item.pipe_id)
            .bind(&item.expected_status)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(record)
    }

    /// SELECT by primary key from `inventory_check_records`. Returns `None` if not found or soft-deleted.
    pub async fn find_by_id(
        pool: &SqlitePool,
        id: i64,
    ) -> Result<Option<InventoryCheckRecord>, sqlx::Error> {
        sqlx::query_as::<_, InventoryCheckRecord>(
            "SELECT id, check_no, location_id, status, notes, created_by, \
             created_at, updated_at, deleted_at \
             FROM inventory_check_records WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    /// SELECT all `InventoryCheckItem` rows for a given check.
    pub async fn get_check_items(
        pool: &SqlitePool,
        check_id: i64,
    ) -> Result<Vec<InventoryCheckItem>, sqlx::Error> {
        sqlx::query_as::<_, InventoryCheckItem>(
            "SELECT id, check_id, pipe_type, pipe_id, expected_status, found_status, \
             is_match, notes, created_at \
             FROM inventory_check_items WHERE check_id = ? ORDER BY id",
        )
        .bind(check_id)
        .fetch_all(pool)
        .await
    }

    /// Paginated SELECT from `inventory_check_records`. Returns `(items, total)`.
    pub async fn list(
        pool: &SqlitePool,
        params: &PaginationParams,
    ) -> Result<(Vec<InventoryCheckRecord>, u64), sqlx::Error> {
        let page_size = params.page_size();
        let offset = params.offset();

        let count_sql =
            "SELECT COUNT(*) as cnt FROM inventory_check_records WHERE deleted_at IS NULL";
        let total: (i64,) = sqlx::query_as(count_sql).fetch_one(pool).await?;

        let items = sqlx::query_as::<_, InventoryCheckRecord>(
            "SELECT id, check_no, location_id, status, notes, created_by, \
             created_at, updated_at, deleted_at \
             FROM inventory_check_records WHERE deleted_at IS NULL \
             ORDER BY created_at DESC LIMIT ? OFFSET ?",
        )
        .bind(page_size as i64)
        .bind(offset as i64)
        .fetch_all(pool)
        .await?;

        Ok((items, total.0 as u64))
    }

    /// UPDATE `status` on an inventory check record (e.g. `in_progress` → `completed`).
    pub async fn update_status(
        pool: &SqlitePool,
        check_id: i64,
        status: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE inventory_check_records SET status = ?, updated_at = datetime('now') \
             WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(status)
        .bind(check_id)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// COUNT of check items that are mismatched (`is_match` IS NULL or 0).
    pub async fn get_mismatch_count(pool: &SqlitePool, check_id: i64) -> Result<i64, sqlx::Error> {
        let (cnt,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM inventory_check_items \
             WHERE check_id = ? AND (is_match IS NULL OR is_match = 0)",
        )
        .bind(check_id)
        .fetch_one(pool)
        .await?;
        Ok(cnt)
    }

    /// UPDATE a single check item's `found_status` and compute `is_match`. Returns the updated item.
    pub async fn update_item_result(
        pool: &SqlitePool,
        check_id: i64,
        item_id: i64,
        found_status: &str,
        notes: &Option<String>,
    ) -> Result<InventoryCheckItem, sqlx::Error> {
        // Fetch expected_status to compute is_match correctly
        let existing: (String,) = sqlx::query_as(
            "SELECT expected_status FROM inventory_check_items WHERE id = ? AND check_id = ?",
        )
        .bind(item_id)
        .bind(check_id)
        .fetch_one(pool)
        .await?;
        let is_match = (found_status == existing.0.as_str()) as i64;
        sqlx::query_as::<_, InventoryCheckItem>(
            "UPDATE inventory_check_items SET found_status = ?, is_match = ?, notes = ? \
             WHERE id = ? AND check_id = ? \
             RETURNING id, check_id, pipe_type, pipe_id, expected_status, found_status, \
               is_match, notes, created_at",
        )
        .bind(found_status)
        .bind(is_match)
        .bind(notes)
        .bind(item_id)
        .bind(check_id)
        .fetch_one(pool)
        .await
    }
}
