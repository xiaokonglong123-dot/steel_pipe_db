use sqlx::{QueryBuilder, Sqlite, SqlitePool};

use crate::dto::common::PaginationParams;
use crate::dto::purchase_dto::{
    CreatePurchaseItemRequest, CreatePurchaseOrderRequest, PurchaseOrderFilterParams,
    UpdatePurchaseItemRequest, UpdatePurchaseOrderRequest,
};
use crate::models::purchase_order::{PurchaseOrder, PurchaseOrderItem};

/// CRUD for `purchase_orders` and `purchase_order_items`. All queries filter `deleted_at IS NULL`.
pub struct PurchaseOrderRepo;

impl PurchaseOrderRepo {
    /// INSERT into `purchase_orders` + line items in a single transaction flow.
    /// Status starts as `draft`. Automatically computes `total_amount` from item prices.
    /// Returns the created `PurchaseOrder`.
    pub async fn create_with_items(
        pool: &SqlitePool,
        dto: &CreatePurchaseOrderRequest,
        order_no: &str,
    ) -> Result<PurchaseOrder, sqlx::Error> {
        let order = sqlx::query_as::<_, PurchaseOrder>(
            "INSERT INTO purchase_orders (order_no, supplier_id, order_date, status, \
             total_amount, notes) \
             VALUES (?, ?, ?, 'draft', 0, ?) \
             RETURNING id, order_no, supplier_id, order_date, status, total_amount, notes, \
               created_by, created_at, updated_at, deleted_at",
        )
        .bind(order_no)
        .bind(dto.supplier_id)
        .bind(&dto.order_date)
        .bind(&dto.notes)
        .fetch_one(pool)
        .await?;

        for item in &dto.items {
            Self::create_item(pool, order.id, item).await?;
        }

        let total: f64 = Self::sum_item_totals(pool, order.id).await?;
        if total > 0.0 {
            sqlx::query("UPDATE purchase_orders SET total_amount = ? WHERE id = ?")
                .bind(total)
                .bind(order.id)
                .execute(pool)
                .await?;
        }

        Ok(PurchaseOrder {
            total_amount: Some(if total > 0.0 { total } else { 0.0 }),
            ..order
        })
    }

    async fn create_item(
        pool: &SqlitePool,
        order_id: i64,
        dto: &CreatePurchaseItemRequest,
    ) -> Result<PurchaseOrderItem, sqlx::Error> {
        sqlx::query_as::<_, PurchaseOrderItem>(
            "INSERT INTO purchase_order_items (order_id, pipe_type, grade, od, wt, quantity, \
             received_quantity, unit_price, total_price, notes) \
             VALUES (?, ?, ?, ?, ?, ?, 0, ?, ?, ?) \
             RETURNING id, order_id, pipe_type, grade, od, wt, quantity, received_quantity, \
               unit_price, total_price, notes, created_at",
        )
        .bind(order_id)
        .bind(&dto.pipe_type)
        .bind(&dto.grade)
        .bind(dto.od)
        .bind(dto.wt)
        .bind(dto.quantity)
        .bind(dto.unit_price)
        .bind(dto.total_price)
        .bind(&dto.notes)
        .fetch_one(pool)
        .await
    }

    async fn sum_item_totals(pool: &SqlitePool, order_id: i64) -> Result<f64, sqlx::Error> {
        let row: (Option<f64>,) = sqlx::query_as(
            "SELECT COALESCE(SUM(total_price), 0) FROM purchase_order_items WHERE order_id = ?",
        )
        .bind(order_id)
        .fetch_one(pool)
        .await?;
        Ok(row.0.unwrap_or(0.0))
    }

    /// Dynamic UPDATE of order-level fields (`order_date`, `notes`). Only supplied fields change.
    /// Returns the updated `PurchaseOrder`.
    pub async fn update_order(
        pool: &SqlitePool,
        id: i64,
        dto: &UpdatePurchaseOrderRequest,
    ) -> Result<PurchaseOrder, sqlx::Error> {
        let mut builder: QueryBuilder<Sqlite> =
            QueryBuilder::new("UPDATE purchase_orders SET updated_at = datetime('now')");

        if let Some(ref val) = dto.order_date {
            builder.push(", order_date = ");
            builder.push_bind(val);
        }
        if let Some(ref val) = dto.notes {
            builder.push(", notes = ");
            builder.push_bind(val);
        }

        builder.push(" WHERE id = ");
        builder.push_bind(id);
        builder.push(
            " AND deleted_at IS NULL RETURNING id, order_no, supplier_id, order_date, status, \
             total_amount, notes, created_by, created_at, updated_at, deleted_at",
        );

        builder.build_query_as::<PurchaseOrder>().fetch_one(pool).await
    }

    /// SELECT by primary key. Returns `None` if soft-deleted or missing.
    pub async fn find_by_id(
        pool: &SqlitePool,
        id: i64,
    ) -> Result<Option<PurchaseOrder>, sqlx::Error> {
        sqlx::query_as::<_, PurchaseOrder>(
            "SELECT id, order_no, supplier_id, order_date, status, total_amount, notes, \
             created_by, created_at, updated_at, deleted_at \
             FROM purchase_orders WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    /// SELECT by unique `order_no`. Returns `None` if soft-deleted or missing.
    pub async fn find_by_order_no(
        pool: &SqlitePool,
        order_no: &str,
    ) -> Result<Option<PurchaseOrder>, sqlx::Error> {
        sqlx::query_as::<_, PurchaseOrder>(
            "SELECT id, order_no, supplier_id, order_date, status, total_amount, notes, \
             created_by, created_at, updated_at, deleted_at \
             FROM purchase_orders WHERE order_no = ? AND deleted_at IS NULL",
        )
        .bind(order_no)
        .fetch_optional(pool)
        .await
    }

    /// SELECT all line items for a purchase order, ordered by `id ASC`.
    pub async fn find_items(
        pool: &SqlitePool,
        order_id: i64,
    ) -> Result<Vec<PurchaseOrderItem>, sqlx::Error> {
        sqlx::query_as::<_, PurchaseOrderItem>(
            "SELECT id, order_id, pipe_type, grade, od, wt, quantity, received_quantity, \
             unit_price, total_price, notes, created_at \
             FROM purchase_order_items WHERE order_id = ? ORDER BY id ASC",
        )
        .bind(order_id)
        .fetch_all(pool)
        .await
    }

    /// UPDATE `status` (e.g. `draft` → `submitted`). Sets `updated_at` timestamp.
    pub async fn update_status(
        pool: &SqlitePool,
        id: i64,
        status: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE purchase_orders SET status = ?, updated_at = datetime('now') \
             WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(status)
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Soft-delete: sets `deleted_at`. Sets status to `rejected` and stores reason in `notes`.
    pub async fn reject(pool: &SqlitePool, id: i64, reason: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE purchase_orders SET status = 'rejected', notes = ?, updated_at = datetime('now') \
             WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(reason)
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Soft-delete: sets `deleted_at` and `updated_at`.
    pub async fn delete(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE purchase_orders SET deleted_at = datetime('now'), \
             updated_at = datetime('now') WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Paginated SELECT with dynamic filters (q, status, supplier_id, order_date range).
    /// Supports sorting by order_no, order_date, status, total_amount. JOINs suppliers for search.
    /// Returns `(items, total)`.
    pub async fn list(
        pool: &SqlitePool,
        filter: &PurchaseOrderFilterParams,
        params: &PaginationParams,
    ) -> Result<(Vec<PurchaseOrder>, u64), sqlx::Error> {
        let page_size = params.page_size();
        let offset = params.offset();

        let mut conditions: Vec<String> = vec!["po.deleted_at IS NULL".into()];
        let mut bind_values: Vec<String> = Vec::new();

        if let Some(ref q) = filter.q {
            if !q.is_empty() {
                conditions.push("(po.order_no LIKE ? OR s.name LIKE ?)".into());
                let pattern = format!("%{}%", q);
                bind_values.push(pattern.clone());
                bind_values.push(pattern);
            }
        }
        if let Some(ref status) = filter.status {
            conditions.push("po.status = ?".into());
            bind_values.push(status.clone());
        }
        if let Some(supplier_id) = filter.supplier_id {
            conditions.push("po.supplier_id = ?".into());
            bind_values.push(supplier_id.to_string());
        }
        if let Some(ref from) = filter.order_date_from {
            conditions.push("po.order_date >= ?".into());
            bind_values.push(from.clone());
        }
        if let Some(ref to) = filter.order_date_to {
            conditions.push("po.order_date <= ?".into());
            bind_values.push(to.clone());
        }

        let where_clause = conditions.join(" AND ");

        let sort_by = match params.sort_by.as_deref() {
            Some("order_no") => "po.order_no",
            Some("order_date") => "po.order_date",
            Some("status") => "po.status",
            Some("total_amount") => "po.total_amount",
            _ => "po.created_at",
        };
        let sort_order = params.sort_order_sql();

        let count_sql = format!(
            "SELECT COUNT(*) as cnt FROM purchase_orders po \
             LEFT JOIN suppliers s ON s.id = po.supplier_id WHERE {}",
            where_clause
        );
        let mut count_q = sqlx::query_as::<_, (i64,)>(&count_sql);
        for val in &bind_values {
            count_q = count_q.bind(val.as_str());
        }
        let total: (i64,) = count_q.fetch_one(pool).await?;

        let list_sql = format!(
            "SELECT po.id, po.order_no, po.supplier_id, po.order_date, po.status, \
             po.total_amount, po.notes, po.created_by, po.created_at, po.updated_at, po.deleted_at \
             FROM purchase_orders po \
             LEFT JOIN suppliers s ON s.id = po.supplier_id \
             WHERE {} ORDER BY {} {} LIMIT ? OFFSET ?",
            where_clause, sort_by, sort_order
        );
        let mut list_q = sqlx::query_as::<_, PurchaseOrder>(&list_sql);
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

    /// Dynamic UPDATE of item fields (pipe_type, grade, od, wt, quantity, unit_price, etc.).
    /// Only supplied fields are modified. Returns the updated `PurchaseOrderItem`.
    pub async fn update_item(
        pool: &SqlitePool,
        item_id: i64,
        dto: &UpdatePurchaseItemRequest,
    ) -> Result<PurchaseOrderItem, sqlx::Error> {
        let mut builder: QueryBuilder<Sqlite> =
            QueryBuilder::new("UPDATE purchase_order_items SET");

        let mut first = true;
        macro_rules! set_field {
            ($val:expr, $col:expr) => {
                if let Some(ref v) = $val {
                    if !first {
                        builder.push(",");
                    }
                    builder.push(format!(" {} = ", $col));
                    builder.push_bind(v);
                    first = false;
                }
            };
        }
        macro_rules! set_field_opt {
            ($val:expr, $col:expr) => {
                if let Some(v) = $val {
                    if !first {
                        builder.push(",");
                    }
                    builder.push(format!(" {} = ", $col));
                    builder.push_bind(v);
                    first = false;
                }
            };
        }

        set_field!(dto.pipe_type, "pipe_type");
        set_field!(dto.grade, "grade");
        set_field_opt!(dto.od, "od");
        set_field_opt!(dto.wt, "wt");
        set_field_opt!(dto.quantity, "quantity");
        set_field_opt!(dto.unit_price, "unit_price");
        // total_price is NOT set from client — recomputed below after UPDATE
        set_field!(dto.notes, "notes");

        if first {
            return Err(sqlx::Error::Protocol("No fields to update".into()));
        }

        builder.push(" WHERE id = ");
        builder.push_bind(item_id);
        builder.push(
            " RETURNING id, order_id, pipe_type, grade, od, wt, quantity, received_quantity, \
             unit_price, total_price, notes, created_at",
        );

        let item = builder.build_query_as::<PurchaseOrderItem>().fetch_one(pool).await?;

        // Recompute total_price server-side: quantity * unit_price
        // This runs after every update to ensure consistency even if only one field changed.
        if dto.quantity.is_some() || dto.unit_price.is_some() {
            let computed_total = item.quantity as f64 * item.unit_price.unwrap_or(0.0);
            sqlx::query("UPDATE purchase_order_items SET total_price = ? WHERE id = ?")
                .bind(computed_total)
                .bind(item.id)
                .execute(pool)
                .await?;
            // Re-fetch to get the updated total_price
            return sqlx::query_as::<_, PurchaseOrderItem>(
                "SELECT id, order_id, pipe_type, grade, od, wt, quantity, received_quantity,                  unit_price, total_price, notes, created_at                  FROM purchase_order_items WHERE id = ?"
            )
            .bind(item.id)
            .fetch_one(pool)
            .await;
        }

        Ok(item)
    }

    /// Hard DELETE from `purchase_order_items` (no soft-delete for items).
    pub async fn delete_item(pool: &SqlitePool, item_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM purchase_order_items WHERE id = ?")
            .bind(item_id)
            .execute(pool)
            .await?;
        Ok(())
    }
}
