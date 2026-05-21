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

// ━━━ CreateInventoryLog (helper struct, not a model) ━━━

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

// ━━━ CheckInitItem (helper struct) ━━━

#[derive(Debug, Clone)]
pub struct CheckInitItem {
    pub pipe_type: String,
    pub pipe_id: i64,
    pub expected_status: String,
}

// ━━━ LocationRepo ━━━

pub struct LocationRepo;

impl LocationRepo {
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
             LIMIT {} OFFSET {}",
            where_clause, page_size, offset
        );

        let items = sqlx::query_as::<_, Location>(&list_sql)
            .fetch_all(pool)
            .await?;

        Ok((items, total.0 as u64))
    }

    pub async fn increment_used(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE locations SET used_count = used_count + 1, updated_at = datetime('now') \
             WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn decrement_used(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE locations SET used_count = MAX(used_count - 1, 0), updated_at = datetime('now') \
             WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }
}

// ━━━ InboundRepo ━━━

pub struct InboundRepo;

impl InboundRepo {
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
               handled_by, handled_at, created_at, updated_at, deleted_at",
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

    pub async fn find_by_id(
        pool: &SqlitePool,
        id: i64,
    ) -> Result<Option<InboundRecord>, sqlx::Error> {
        sqlx::query_as::<_, InboundRecord>(
            "SELECT id, inbound_no, inbound_type, order_id, supplier_id, notes, approval_status, \
             handled_by, handled_at, created_at, updated_at, deleted_at \
             FROM inbound_records WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

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

        if let Some(ref q) = filter.q {
            if !q.is_empty() {
                conditions.push(format!(
                    "inbound_no LIKE '%{}%'",
                    q.replace('\'', "''")
                ));
            }
        }
        if let Some(ref inbound_type) = filter.inbound_type {
            conditions.push(format!(
                "inbound_type = '{}'",
                inbound_type.replace('\'', "''")
            ));
        }
        if let Some(ref approval_status) = filter.approval_status {
            conditions.push(format!(
                "approval_status = '{}'",
                approval_status.replace('\'', "''")
            ));
        }
        if let Some(order_id) = filter.order_id {
            conditions.push(format!("order_id = {}", order_id));
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
        let total: (i64,) = sqlx::query_as(&count_sql).fetch_one(pool).await?;

        let list_sql = format!(
            "SELECT id, inbound_no, inbound_type, order_id, supplier_id, notes, approval_status, \
             handled_by, handled_at, created_at, updated_at, deleted_at \
             FROM inbound_records WHERE {} \
             ORDER BY {} {} LIMIT {} OFFSET {}",
            where_clause, sort_by, sort_order, page_size, offset
        );

        let items = sqlx::query_as::<_, InboundRecord>(&list_sql)
            .fetch_all(pool)
            .await?;

        Ok((items, total.0 as u64))
    }

    pub async fn update_status(pool: &SqlitePool, id: i64, status: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE inbound_records SET approval_status = ?, updated_at = datetime('now') \
             WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(status)
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }

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

// ━━━ OutboundRepo ━━━

pub struct OutboundRepo;

impl OutboundRepo {
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
               handled_by, handled_at, created_at, updated_at, deleted_at",
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

    pub async fn find_by_id(
        pool: &SqlitePool,
        id: i64,
    ) -> Result<Option<OutboundRecord>, sqlx::Error> {
        sqlx::query_as::<_, OutboundRecord>(
            "SELECT id, outbound_no, outbound_type, order_id, customer_id, notes, approval_status, \
             handled_by, handled_at, created_at, updated_at, deleted_at \
             FROM outbound_records WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

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

        if let Some(ref q) = filter.q {
            if !q.is_empty() {
                conditions.push(format!(
                    "outbound_no LIKE '%{}%'",
                    q.replace('\'', "''")
                ));
            }
        }
        if let Some(ref outbound_type) = filter.outbound_type {
            conditions.push(format!(
                "outbound_type = '{}'",
                outbound_type.replace('\'', "''")
            ));
        }
        if let Some(ref approval_status) = filter.approval_status {
            conditions.push(format!(
                "approval_status = '{}'",
                approval_status.replace('\'', "''")
            ));
        }
        if let Some(order_id) = filter.order_id {
            conditions.push(format!("order_id = {}", order_id));
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
        let total: (i64,) = sqlx::query_as(&count_sql).fetch_one(pool).await?;

        let list_sql = format!(
            "SELECT id, outbound_no, outbound_type, order_id, customer_id, notes, approval_status, \
             handled_by, handled_at, created_at, updated_at, deleted_at \
             FROM outbound_records WHERE {} \
             ORDER BY {} {} LIMIT {} OFFSET {}",
            where_clause, sort_by, sort_order, page_size, offset
        );

        let items = sqlx::query_as::<_, OutboundRecord>(&list_sql)
            .fetch_all(pool)
            .await?;

        Ok((items, total.0 as u64))
    }

    pub async fn update_status(
        pool: &SqlitePool,
        id: i64,
        status: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE outbound_records SET approval_status = ?, updated_at = datetime('now') \
             WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(status)
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }

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

// ━━━ InventoryLogRepo ━━━

pub struct InventoryLogRepo;

impl InventoryLogRepo {
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

        if let Some(ref pipe_type) = filter.pipe_type {
            conditions.push(format!(
                "pipe_type = '{}'",
                pipe_type.replace('\'', "''")
            ));
        }
        if let Some(location_id) = filter.location_id {
            conditions.push(format!(
                "(from_location_id = {} OR to_location_id = {})",
                location_id, location_id
            ));
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
        let total: (i64,) = sqlx::query_as(&count_sql).fetch_one(pool).await?;

        let list_sql = format!(
            "SELECT id, pipe_type, pipe_id, change_type, ref_type, ref_id, \
             from_location_id, to_location_id, notes, created_by, created_at \
             FROM inventory_logs WHERE {} \
             ORDER BY created_at DESC LIMIT {} OFFSET {}",
            where_clause, page_size, offset
        );

        let items = sqlx::query_as::<_, InventoryLog>(&list_sql)
            .fetch_all(pool)
            .await?;

        Ok((items, total.0 as u64))
    }

    pub async fn find_by_pipe(
        pool: &SqlitePool,
        pipe_type: &str,
        pipe_id: i64,
    ) -> Result<Vec<InventoryLog>, sqlx::Error> {
        sqlx::query_as::<_, InventoryLog>(
            "SELECT id, pipe_type, pipe_id, change_type, ref_type, ref_id, \
             from_location_id, to_location_id, notes, created_by, created_at \
             FROM inventory_logs WHERE pipe_type = ? AND pipe_id = ? \
             ORDER BY created_at ASC",
        )
        .bind(pipe_type)
        .bind(pipe_id)
        .fetch_all(pool)
        .await
    }
}

// ━━━ CheckRepo ━━━

pub struct CheckRepo;

impl CheckRepo {
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

    pub async fn list(
        pool: &SqlitePool,
        params: &PaginationParams,
    ) -> Result<(Vec<InventoryCheckRecord>, u64), sqlx::Error> {
        let page_size = params.page_size();
        let offset = params.offset();

        let count_sql = "SELECT COUNT(*) as cnt FROM inventory_check_records WHERE deleted_at IS NULL";
        let total: (i64,) = sqlx::query_as(count_sql).fetch_one(pool).await?;

        let list_sql = format!(
            "SELECT id, check_no, location_id, status, notes, created_by, \
             created_at, updated_at, deleted_at \
             FROM inventory_check_records WHERE deleted_at IS NULL \
             ORDER BY created_at DESC LIMIT {} OFFSET {}",
            page_size, offset
        );

        let items = sqlx::query_as::<_, InventoryCheckRecord>(&list_sql)
            .fetch_all(pool)
            .await?;

        Ok((items, total.0 as u64))
    }

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

    pub async fn update_status(pool: &SqlitePool, id: i64, status: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE inventory_check_records SET status = ?, updated_at = datetime('now') \
             WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(status)
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }
}
