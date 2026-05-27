use sqlx::{QueryBuilder, Sqlite, SqlitePool};

use crate::dto::common::PaginationParams;
use crate::dto::inventory_dto::{
    CreateCheckRequest, CreateLocationRequest, InboundFilter, InventoryFilter, OutboundFilter,
    UpdateLocationRequest,
};
use crate::models::inventory::{
    InboundItem, InboundRecord, InventoryCheckItem, InventoryCheckRecord, InventoryLog, Location,
    OutboundItem, OutboundRecord,
};

/// Helper struct for inserting into `inventory_logs` — not a DB model.
#[derive(Debug, Clone)]
pub struct CreateInventoryLog {
    pub pipe_type: String,
    pub pipe_id: i64,
    pub change_type: String,
    pub ref_type: Option<String>,
    pub ref_id: Option<i64>,
    pub from_location_id: Option<i64>,
    pub to_location_id: Option<i64>,
    pub notes: Option<String>,
    pub created_by: Option<i64>,
}

/// Helper struct for seeding inventory check items — not a DB model.
#[derive(Debug, Clone)]
pub struct CheckInitItem {
    pub pipe_type: String,
    pub pipe_id: i64,
    pub expected_status: String,
}

/// ATP (Available-to-Promise) queries across `seamless_pipes` and `screen_pipes`.
pub struct InventoryRepo;

impl InventoryRepo {
    /// UNION query across both `seamless_pipes` and `screen_pipes` to compute available-to-promise
    /// stock grouped by `pipe_type`, `grade`, and `location_id`. Supports optional filters.
    pub async fn find_atp(
        pool: &SqlitePool,
        pipe_type: &Option<String>,
        grade: &Option<String>,
        location_id: &Option<i64>,
    ) -> Result<Vec<(String, String, i64, Option<i64>)>, sqlx::Error> {
        let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new(
            "SELECT pipe_type, grade, SUM(cnt) as quantity, location_id FROM ( \
             SELECT pipe_type, grade, COUNT(*) as cnt, location_id \
             FROM seamless_pipes WHERE status = 'in_stock' AND deleted_at IS NULL",
        );

        if let Some(ref pt) = pipe_type {
            builder.push(" AND pipe_type = ");
            builder.push_bind(pt);
        }
        if let Some(ref g) = grade {
            builder.push(" AND grade = ");
            builder.push_bind(g);
        }
        if let Some(loc) = location_id {
            builder.push(" AND location_id = ");
            builder.push_bind(loc);
        }

        builder.push(
            " GROUP BY pipe_type, grade, location_id \
             UNION ALL \
             SELECT screen_type as pipe_type, base_grade as grade, COUNT(*) as cnt, location_id \
             FROM screen_pipes WHERE status = 'in_stock' AND deleted_at IS NULL",
        );

        if let Some(ref pt) = pipe_type {
            builder.push(" AND screen_type = ");
            builder.push_bind(pt);
        }
        if let Some(ref g) = grade {
            builder.push(" AND base_grade = ");
            builder.push_bind(g);
        }
        if let Some(loc) = location_id {
            builder.push(" AND location_id = ");
            builder.push_bind(loc);
        }

        builder.push(
            " GROUP BY screen_type, base_grade, location_id \
             ) GROUP BY pipe_type, grade, location_id ORDER BY pipe_type, grade",
        );

        builder
            .build_query_as::<(String, String, i64, Option<i64>)>()
.fetch_all(pool)
        .await
    }

    /// Sums `COUNT(*)` of `in_stock` pipes from both `seamless_pipes` and `screen_pipes`.
    pub async fn get_total_in_stock(pool: &SqlitePool) -> Result<i64, sqlx::Error> {
        let (seamless,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM seamless_pipes WHERE status = 'in_stock' AND deleted_at IS NULL",
        )
        .fetch_one(pool)
        .await?;

        let (screen,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM screen_pipes WHERE status = 'in_stock' AND deleted_at IS NULL",
        )
        .fetch_one(pool)
        .await?;

        Ok(seamless + screen)
    }

    /// GROUP BY `grade`/`base_grade` across both pipe tables. Returns JSON objects with
    /// `grade`, `count`, and `pipe_type` fields.
    pub async fn get_count_by_grade(pool: &SqlitePool) -> Result<Vec<serde_json::Value>, sqlx::Error> {
        let seamless: Vec<(String, i64)> = sqlx::query_as(
            "SELECT grade, COUNT(*) as cnt FROM seamless_pipes \
             WHERE status = 'in_stock' AND deleted_at IS NULL GROUP BY grade ORDER BY grade",
        )
        .fetch_all(pool)
        .await?;

        let screen: Vec<(String, i64)> = sqlx::query_as(
            "SELECT base_grade as grade, COUNT(*) as cnt FROM screen_pipes \
             WHERE status = 'in_stock' AND deleted_at IS NULL GROUP BY base_grade ORDER BY base_grade",
        )
        .fetch_all(pool)
        .await?;

        let mut result = Vec::new();
        for (grade, cnt) in seamless {
            result.push(serde_json::json!({"grade": grade, "count": cnt, "pipe_type": "seamless"}));
        }
        for (grade, cnt) in screen {
            result.push(serde_json::json!({"grade": grade, "count": cnt, "pipe_type": "screen"}));
        }
        Ok(result)
    }

    /// GROUP BY `location_id` across both pipe tables. Returns JSON objects with
    /// `location_id`, `count`, and `pipe_type` fields.
    pub async fn get_count_by_location(pool: &SqlitePool) -> Result<Vec<serde_json::Value>, sqlx::Error> {
        let seamless: Vec<(Option<i64>, i64)> = sqlx::query_as(
            "SELECT location_id, COUNT(*) as cnt FROM seamless_pipes \
             WHERE status = 'in_stock' AND deleted_at IS NULL GROUP BY location_id",
        )
        .fetch_all(pool)
        .await?;

        let screen: Vec<(Option<i64>, i64)> = sqlx::query_as(
            "SELECT location_id, COUNT(*) as cnt FROM screen_pipes \
             WHERE status = 'in_stock' AND deleted_at IS NULL GROUP BY location_id",
        )
        .fetch_all(pool)
        .await?;

        let mut result = Vec::new();
        for (loc_id, cnt) in seamless {
            result.push(serde_json::json!({"location_id": loc_id, "count": cnt, "pipe_type": "seamless"}));
        }
        for (loc_id, cnt) in screen {
            result.push(serde_json::json!({"location_id": loc_id, "count": cnt, "pipe_type": "screen"}));
        }
        Ok(result)
    }

    /// Updates `location_id` on either `seamless_pipes` or `screen_pipes` depending on
    /// `pipe_type`. No-op if `pipe_type` is neither seamless nor screen.
    pub async fn update_pipe_location(
        pool: &SqlitePool,
        pipe_type: &str,
        pipe_id: i64,
        location_id: i64,
    ) -> Result<(), sqlx::Error> {
        match pipe_type {
            "seamless" | "casing" | "tubing" => {
                sqlx::query(
                    "UPDATE seamless_pipes SET location_id = ?, updated_at = datetime('now') \
                     WHERE id = ? AND deleted_at IS NULL",
                )
                .bind(location_id)
                .bind(pipe_id)
                .execute(pool)
                .await?;
            }
            "screen" | "screened" => {
                sqlx::query(
                    "UPDATE screen_pipes SET location_id = ?, updated_at = datetime('now') \
                     WHERE id = ? AND deleted_at IS NULL",
                )
                .bind(location_id)
                .bind(pipe_id)
                .execute(pool)
                .await?;
            }
            _ => {}
        }
        Ok(())
    }

    /// Returns `location_id` for a given pipe (seamless or screen). Returns `None` if not found.
    pub async fn get_pipe_location_id(
        pool: &SqlitePool,
        pipe_type: &str,
        pipe_id: i64,
    ) -> Result<Option<i64>, sqlx::Error> {
        match pipe_type {
            "seamless" | "casing" | "tubing" => {
                let row: Option<(Option<i64>,)> = sqlx::query_as(
                    "SELECT location_id FROM seamless_pipes WHERE id = ? AND deleted_at IS NULL",
                )
                .bind(pipe_id)
                .fetch_optional(pool)
                .await?;
                Ok(row.and_then(|r| r.0))
            }
            "screen" | "screened" => {
                let row: Option<(Option<i64>,)> = sqlx::query_as(
                    "SELECT location_id FROM screen_pipes WHERE id = ? AND deleted_at IS NULL",
                )
                .bind(pipe_id)
                .fetch_optional(pool)
                .await?;
                Ok(row.and_then(|r| r.0))
            }
            _ => Ok(None),
        }
    }
}

/// CRUD for `locations` table (warehouse bin locations). All queries filter `deleted_at IS NULL`.
pub struct LocationRepo;

impl LocationRepo {
    /// INSERT into `locations`. Returns the newly created row with generated `id`.
    pub async fn create(
        pool: &SqlitePool,
        dto: &CreateLocationRequest,
        full_code: &str,
    ) -> Result<Location, sqlx::Error> {
        sqlx::query_as::<_, Location>(
            "INSERT INTO locations (zone_code, shelf_code, level_code, full_code, description, capacity) \
             VALUES (?, ?, ?, ?, ?, ?) \
             RETURNING id, zone_code, shelf_code, level_code, full_code, description, capacity, \
               used_count, is_active, created_at, updated_at, deleted_at",
        )
        .bind(&dto.zone_code)
        .bind(&dto.shelf_code)
        .bind(&dto.level_code)
        .bind(full_code)
        .bind(&dto.description)
        .bind(dto.capacity)
        .fetch_one(pool)
        .await
    }

    /// UPDATE `locations` by id. Supports optional `description`, `capacity`, `is_active` fields.
    /// Returns the updated row.
    pub async fn update(
        pool: &SqlitePool,
        id: i64,
        dto: &UpdateLocationRequest,
    ) -> Result<Location, sqlx::Error> {
        let mut builder: QueryBuilder<Sqlite> =
            QueryBuilder::new("UPDATE locations SET updated_at = datetime('now')");

        if let Some(ref val) = dto.description {
            builder.push(", description = ");
            builder.push_bind(val);
        }
        if let Some(val) = dto.capacity {
            builder.push(", capacity = ");
            builder.push_bind(val);
        }
        if let Some(val) = dto.is_active {
            builder.push(", is_active = ");
            builder.push_bind(val);
        }

        builder.push(" WHERE id = ");
        builder.push_bind(id);
        builder.push(
            " AND deleted_at IS NULL RETURNING id, zone_code, shelf_code, level_code, \
             full_code, description, capacity, used_count, is_active, created_at, \
             updated_at, deleted_at",
        );

        builder.build_query_as::<Location>().fetch_one(pool).await
    }

    /// SELECT by primary key from `locations`. Returns `None` if not found or soft-deleted.
    pub async fn find_by_id(pool: &SqlitePool, id: i64) -> Result<Option<Location>, sqlx::Error> {
        sqlx::query_as::<_, Location>(
            "SELECT id, zone_code, shelf_code, level_code, full_code, description, capacity, \
             used_count, is_active, created_at, updated_at, deleted_at \
             FROM locations WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    /// SELECT by unique `full_code` (e.g. `A-01-01`). Returns `None` if not found or soft-deleted.
    pub async fn find_by_full_code(
        pool: &SqlitePool,
        code: &str,
    ) -> Result<Option<Location>, sqlx::Error> {
        sqlx::query_as::<_, Location>(
            "SELECT id, zone_code, shelf_code, level_code, full_code, description, capacity, \
             used_count, is_active, created_at, updated_at, deleted_at \
             FROM locations WHERE full_code = ? AND deleted_at IS NULL",
        )
        .bind(code)
        .fetch_optional(pool)
        .await
    }

    /// Soft-delete by setting `deleted_at` timestamp. No-op if already deleted.
    pub async fn delete(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE locations SET deleted_at = datetime('now'), \
             updated_at = datetime('now') WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Paginated SELECT from `locations`. Optionally filters to only active locations.
    /// Returns `(items, total)`.
    pub async fn list(
        pool: &SqlitePool,
        params: &PaginationParams,
        active_only: bool,
    ) -> Result<(Vec<Location>, u64), sqlx::Error> {
        let page_size = params.page_size();
        let offset = params.offset();

        let mut conditions: Vec<String> = vec!["deleted_at IS NULL".into()];
        if active_only {
            conditions.push("is_active = 1".into());
        }
        let where_clause = conditions.join(" AND ");

        let count_sql = format!("SELECT COUNT(*) as cnt FROM locations WHERE {}", where_clause);
        let total: (i64,) = sqlx::query_as(&count_sql).fetch_one(pool).await?;

        let list_sql = format!(
            "SELECT id, zone_code, shelf_code, level_code, full_code, description, capacity, \
             used_count, is_active, created_at, updated_at, deleted_at \
             FROM locations WHERE {} ORDER BY zone_code ASC, shelf_code ASC, level_code ASC \
             LIMIT ? OFFSET ?",
            where_clause
        );

        let items = sqlx::query_as::<_, Location>(&list_sql)
            .bind(page_size as i64)
            .bind(offset as i64)
            .fetch_all(pool)
            .await?;

        Ok((items, total.0 as u64))
    }

}

/// CRUD for `inbound_records` and `inbound_items`. All queries filter `deleted_at IS NULL`.
pub struct InboundRepo;

impl InboundRepo {
    /// INSERT into `inbound_records` + `inbound_items` in a single transaction.
    /// Purchase-type records start as `auto_approved`; others as `pending`.
    /// Returns the created `InboundRecord`.
    pub async fn create_with_items(
        pool: &SqlitePool,
        dto: &crate::dto::inventory_dto::CreateInboundRecordRequest,
        inbound_no: &str,
    ) -> Result<InboundRecord, sqlx::Error> {
        let mut tx = pool.begin().await?;

        let record = sqlx::query_as::<_, InboundRecord>(
            "INSERT INTO inbound_records (inbound_no, inbound_type, order_id, supplier_id, notes, approval_status) \
             VALUES (?, ?, ?, ?, ?, ?) \
             RETURNING id, inbound_no, inbound_type, order_id, supplier_id, notes, approval_status, \
               rejection_reason, handled_by, handled_at, created_at, updated_at, deleted_at",
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
        .fetch_one(&mut *tx)
        .await?;

        for item in &dto.pipes {
            sqlx::query(
                "INSERT INTO inbound_items (inbound_id, pipe_type, pipe_id) VALUES (?, ?, ?)",
            )
            .bind(record.id)
            .bind(&item.pipe_type)
            .bind(item.pipe_id)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(record)
    }

    /// SELECT by primary key from `inbound_records`. Returns `None` if not found or soft-deleted.
    pub async fn find_by_id(
        pool: &SqlitePool,
        id: i64,
    ) -> Result<Option<InboundRecord>, sqlx::Error> {
        sqlx::query_as::<_, InboundRecord>(
            "SELECT id, inbound_no, inbound_type, order_id, supplier_id, notes, approval_status, \
             rejection_reason, handled_by, handled_at, created_at, updated_at, deleted_at \
             FROM inbound_records WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    /// SELECT all `InboundItem` rows for a given inbound record.
    pub async fn find_items(
        pool: &SqlitePool,
        inbound_id: i64,
    ) -> Result<Vec<InboundItem>, sqlx::Error> {
        sqlx::query_as::<_, InboundItem>(
            "SELECT id, inbound_id, pipe_type, pipe_id, created_at \
             FROM inbound_items WHERE inbound_id = ? ORDER BY id",
        )
        .bind(inbound_id)
        .fetch_all(pool)
        .await
    }

    /// Paginated SELECT with optional filters (`q`, `inbound_type`, `approval_status`, `order_id`).
    /// Returns `(items, total)`.
    pub async fn list(
        pool: &SqlitePool,
        filter: &InboundFilter,
    ) -> Result<(Vec<InboundRecord>, u64), sqlx::Error> {
        let pagination = PaginationParams {
            page: filter.page,
            page_size: filter.page_size,
            sort_by: filter.sort_by.clone(),
            sort_order: filter.sort_order.clone(),
        };
        let page_size = pagination.page_size();
        let offset = pagination.offset();

        let mut conditions: Vec<String> = vec!["deleted_at IS NULL".into()];
        let mut bind_values: Vec<String> = Vec::new();

        if let Some(ref q) = filter.q {
            if !q.is_empty() {
                conditions.push("inbound_no LIKE ?".into());
                bind_values.push(format!("%{}%", q));
            }
        }
        if let Some(ref inbound_type) = filter.inbound_type {
            conditions.push("inbound_type = ?".into());
            bind_values.push(inbound_type.clone());
        }
        if let Some(ref approval_status) = filter.approval_status {
            conditions.push("approval_status = ?".into());
            bind_values.push(approval_status.clone());
        }
        if let Some(order_id) = filter.order_id {
            conditions.push("order_id = ?".into());
            bind_values.push(order_id.to_string());
        }

        let where_clause = conditions.join(" AND ");

        let sort_by = match pagination.sort_by.as_deref() {
            Some("inbound_no") => "inbound_no",
            Some("inbound_type") => "inbound_type",
            Some("approval_status") => "approval_status",
            _ => "created_at",
        };
        let sort_order = pagination.sort_order_sql();

        let count_sql = format!(
            "SELECT COUNT(*) as cnt FROM inbound_records WHERE {}",
            where_clause
        );
        let mut count_q = sqlx::query_as::<_, (i64,)>(&count_sql);
        for val in &bind_values {
            count_q = count_q.bind(val.as_str());
        }
        let total: (i64,) = count_q.fetch_one(pool).await?;

        let list_sql = format!(
            "SELECT id, inbound_no, inbound_type, order_id, supplier_id, notes, approval_status, \
             rejection_reason, handled_by, handled_at, created_at, updated_at, deleted_at \
             FROM inbound_records WHERE {} \
             ORDER BY {} {} LIMIT ? OFFSET ?",
            where_clause, sort_by, sort_order
        );
        let mut list_q = sqlx::query_as::<_, InboundRecord>(&list_sql);
        for val in &bind_values {
            list_q = list_q.bind(val.as_str());
        }
        let items = list_q
            .bind(page_size as i64)
            .bind(offset as i64)
            .fetch_all(pool)
            .await?;

        Ok((items, total.0 as u64))
    }

    /// UPDATE `approval_status` on an inbound record. Optionally sets `rejection_reason`.
    pub async fn update_status(
        pool: &SqlitePool,
        id: i64,
        status: &str,
        rejection_reason: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        if let Some(reason) = rejection_reason {
            sqlx::query(
                "UPDATE inbound_records SET approval_status = ?, rejection_reason = ?, \
                 updated_at = datetime('now') WHERE id = ? AND deleted_at IS NULL",
            )
            .bind(status)
            .bind(reason)
            .bind(id)
            .execute(pool)
            .await?;
        } else {
            sqlx::query(
                "UPDATE inbound_records SET approval_status = ?, \
                 rejection_reason = NULL, updated_at = datetime('now') \
                 WHERE id = ? AND deleted_at IS NULL",
            )
            .bind(status)
            .bind(id)
            .execute(pool)
            .await?;
        }
        Ok(())
    }

    /// Sets `order_id` on an inbound record to link it to a purchase order.
    pub async fn link_to_order(
        pool: &SqlitePool,
        inbound_id: i64,
        order_id: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE inbound_records SET order_id = ?, updated_at = datetime('now') \
             WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(order_id)
        .bind(inbound_id)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Soft-delete by setting `deleted_at` timestamp. No-op if already deleted.
    pub async fn delete(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE inbound_records SET deleted_at = datetime('now'), \
             updated_at = datetime('now') WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }
}

/// CRUD for `outbound_records` and `outbound_items`. All queries filter `deleted_at IS NULL`.
pub struct OutboundRepo;

impl OutboundRepo {
    /// INSERT into `outbound_records` + `outbound_items` in a single transaction.
    /// Sales-type records start as `auto_approved`; others as `pending`.
    /// Returns the created `OutboundRecord`.
    pub async fn create_with_items(
        pool: &SqlitePool,
        dto: &crate::dto::inventory_dto::CreateOutboundRecordRequest,
        outbound_no: &str,
    ) -> Result<OutboundRecord, sqlx::Error> {
        let mut tx = pool.begin().await?;

        let record = sqlx::query_as::<_, OutboundRecord>(
            "INSERT INTO outbound_records (outbound_no, outbound_type, order_id, customer_id, notes, approval_status) \
             VALUES (?, ?, ?, ?, ?, ?) \
             RETURNING id, outbound_no, outbound_type, order_id, customer_id, notes, approval_status, \
               rejection_reason, handled_by, handled_at, created_at, updated_at, deleted_at",
        )
        .bind(outbound_no)
        .bind(&dto.outbound_type)
        .bind(dto.order_id)
        .bind(dto.customer_id)
        .bind(&dto.notes)
        .bind(if dto.outbound_type == "sales" {
            "auto_approved"
        } else {
            "pending"
        })
        .fetch_one(&mut *tx)
        .await?;

        for item in &dto.pipes {
            sqlx::query(
                "INSERT INTO outbound_items (outbound_id, pipe_type, pipe_id) VALUES (?, ?, ?)",
            )
            .bind(record.id)
            .bind(&item.pipe_type)
            .bind(item.pipe_id)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(record)
    }

    /// SELECT by primary key from `outbound_records`. Returns `None` if not found or soft-deleted.
    pub async fn find_by_id(
        pool: &SqlitePool,
        id: i64,
    ) -> Result<Option<OutboundRecord>, sqlx::Error> {
        sqlx::query_as::<_, OutboundRecord>(
            "SELECT id, outbound_no, outbound_type, order_id, customer_id, notes, approval_status, \
             rejection_reason, handled_by, handled_at, created_at, updated_at, deleted_at \
             FROM outbound_records WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    /// SELECT all `OutboundItem` rows for a given outbound record.
    pub async fn find_items(
        pool: &SqlitePool,
        outbound_id: i64,
    ) -> Result<Vec<OutboundItem>, sqlx::Error> {
        sqlx::query_as::<_, OutboundItem>(
            "SELECT id, outbound_id, pipe_type, pipe_id, created_at \
             FROM outbound_items WHERE outbound_id = ? ORDER BY id",
        )
        .bind(outbound_id)
        .fetch_all(pool)
        .await
    }

    /// Paginated SELECT with optional filters (`q`, `outbound_type`, `approval_status`, `order_id`).
    /// Returns `(items, total)`.
    pub async fn list(
        pool: &SqlitePool,
        filter: &OutboundFilter,
    ) -> Result<(Vec<OutboundRecord>, u64), sqlx::Error> {
        let pagination = PaginationParams {
            page: filter.page,
            page_size: filter.page_size,
            sort_by: filter.sort_by.clone(),
            sort_order: filter.sort_order.clone(),
        };
        let page_size = pagination.page_size();
        let offset = pagination.offset();

        let mut conditions: Vec<String> = vec!["deleted_at IS NULL".into()];
        let mut bind_values: Vec<String> = Vec::new();

        if let Some(ref q) = filter.q {
            if !q.is_empty() {
                conditions.push("outbound_no LIKE ?".into());
                bind_values.push(format!("%{}%", q));
            }
        }
        if let Some(ref outbound_type) = filter.outbound_type {
            conditions.push("outbound_type = ?".into());
            bind_values.push(outbound_type.clone());
        }
        if let Some(ref approval_status) = filter.approval_status {
            conditions.push("approval_status = ?".into());
            bind_values.push(approval_status.clone());
        }
        if let Some(order_id) = filter.order_id {
            conditions.push("order_id = ?".into());
            bind_values.push(order_id.to_string());
        }

        let where_clause = conditions.join(" AND ");

        let sort_by = match pagination.sort_by.as_deref() {
            Some("outbound_no") => "outbound_no",
            Some("outbound_type") => "outbound_type",
            Some("approval_status") => "approval_status",
            _ => "created_at",
        };
        let sort_order = pagination.sort_order_sql();

        let count_sql = format!(
            "SELECT COUNT(*) as cnt FROM outbound_records WHERE {}",
            where_clause
        );
        let mut count_q = sqlx::query_as::<_, (i64,)>(&count_sql);
        for val in &bind_values {
            count_q = count_q.bind(val.as_str());
        }
        let total: (i64,) = count_q.fetch_one(pool).await?;

        let list_sql = format!(
            "SELECT id, outbound_no, outbound_type, order_id, customer_id, notes, approval_status, \
             rejection_reason, handled_by, handled_at, created_at, updated_at, deleted_at \
             FROM outbound_records WHERE {} \
             ORDER BY {} {} LIMIT ? OFFSET ?",
            where_clause, sort_by, sort_order
        );
        let mut list_q = sqlx::query_as::<_, OutboundRecord>(&list_sql);
        for val in &bind_values {
            list_q = list_q.bind(val.as_str());
        }
        let items = list_q
            .bind(page_size as i64)
            .bind(offset as i64)
            .fetch_all(pool)
            .await?;

        Ok((items, total.0 as u64))
    }

    /// UPDATE `approval_status` on an outbound record. Optionally sets `rejection_reason`.
    pub async fn update_status(
        pool: &SqlitePool,
        id: i64,
        status: &str,
        rejection_reason: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        if let Some(reason) = rejection_reason {
            sqlx::query(
                "UPDATE outbound_records SET approval_status = ?, rejection_reason = ?, \
                 updated_at = datetime('now') WHERE id = ? AND deleted_at IS NULL",
            )
            .bind(status)
            .bind(reason)
            .bind(id)
            .execute(pool)
            .await?;
        } else {
            sqlx::query(
                "UPDATE outbound_records SET approval_status = ?, \
                 rejection_reason = NULL, updated_at = datetime('now') \
                 WHERE id = ? AND deleted_at IS NULL",
            )
            .bind(status)
            .bind(id)
            .execute(pool)
            .await?;
        }
        Ok(())
    }

    /// Sets `order_id` on an outbound record to link it to a sales order.
    pub async fn link_to_order(
        pool: &SqlitePool,
        outbound_id: i64,
        order_id: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE outbound_records SET order_id = ?, updated_at = datetime('now') \
             WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(order_id)
        .bind(outbound_id)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Soft-delete by setting `deleted_at` timestamp. No-op if already deleted.
    pub async fn delete(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE outbound_records SET deleted_at = datetime('now'), \
             updated_at = datetime('now') WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }
}

/// INSERT + paginated SELECT for `inventory_logs` (pipe movement audit trail).
pub struct InventoryLogRepo;

impl InventoryLogRepo {
    /// INSERT a row into `inventory_logs`. Returns the newly created log entry with generated `id`.
    pub async fn create(
        pool: &SqlitePool,
        log: &CreateInventoryLog,
    ) -> Result<InventoryLog, sqlx::Error> {
        sqlx::query_as::<_, InventoryLog>(
            "INSERT INTO inventory_logs (pipe_type, pipe_id, change_type, ref_type, ref_id, \
             from_location_id, to_location_id, notes, created_by) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?) \
             RETURNING id, pipe_type, pipe_id, change_type, ref_type, ref_id, \
               from_location_id, to_location_id, notes, created_by, created_at",
        )
        .bind(&log.pipe_type)
        .bind(log.pipe_id)
        .bind(&log.change_type)
        .bind(&log.ref_type)
        .bind(log.ref_id)
        .bind(log.from_location_id)
        .bind(log.to_location_id)
        .bind(&log.notes)
        .bind(log.created_by)
        .fetch_one(pool)
        .await
    }

    /// Paginated SELECT from `inventory_logs` with optional filters (`pipe_type`, `location_id`).
    /// Returns `(items, total)`.
    pub async fn list(
        pool: &SqlitePool,
        filter: &InventoryFilter,
    ) -> Result<(Vec<InventoryLog>, u64), sqlx::Error> {
        let pagination = PaginationParams {
            page: filter.page,
            page_size: filter.page_size,
            sort_by: None,
            sort_order: None,
        };
        let page_size = pagination.page_size();
        let offset = pagination.offset();

        let mut conditions: Vec<String> = Vec::new();
        let mut bind_values: Vec<String> = Vec::new();

        if let Some(ref pipe_type) = filter.pipe_type {
            conditions.push("pipe_type = ?".into());
            bind_values.push(pipe_type.clone());
        }
        if let Some(location_id) = filter.location_id {
            conditions.push("(from_location_id = ? OR to_location_id = ?)".into());
            bind_values.push(location_id.to_string());
            bind_values.push(location_id.to_string());
        }

        let where_clause = if conditions.is_empty() {
            "1=1".to_string()
        } else {
            conditions.join(" AND ")
        };

        let count_sql = format!(
            "SELECT COUNT(*) as cnt FROM inventory_logs WHERE {}",
            where_clause
        );
        let mut count_q = sqlx::query_as::<_, (i64,)>(&count_sql);
        for val in &bind_values {
            count_q = count_q.bind(val.as_str());
        }
        let total: (i64,) = count_q.fetch_one(pool).await?;

        let list_sql = format!(
            "SELECT id, pipe_type, pipe_id, change_type, ref_type, ref_id, \
             from_location_id, to_location_id, notes, created_by, created_at \
             FROM inventory_logs WHERE {} \
             ORDER BY created_at DESC LIMIT ? OFFSET ?",
            where_clause
        );
        let mut list_q = sqlx::query_as::<_, InventoryLog>(&list_sql);
        for val in &bind_values {
            list_q = list_q.bind(val.as_str());
        }
        let items = list_q
            .bind(page_size as i64)
            .bind(offset as i64)
            .fetch_all(pool)
            .await?;

        Ok((items, total.0 as u64))
    }

}

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

        let count_sql = "SELECT COUNT(*) as cnt FROM inventory_check_records WHERE deleted_at IS NULL";
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
    pub async fn get_mismatch_count(
        pool: &SqlitePool,
        check_id: i64,
    ) -> Result<i64, sqlx::Error> {
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
        let is_match = (found_status == "in_stock") as i64;
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
