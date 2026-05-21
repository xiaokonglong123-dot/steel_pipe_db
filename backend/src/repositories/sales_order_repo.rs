use sqlx::{QueryBuilder, Sqlite, SqlitePool};

use crate::dto::common::PaginationParams;
use crate::dto::sales_dto::{
    CreateSalesItemRequest, CreateSalesOrderRequest, SalesOrderFilterParams,
    UpdateSalesItemRequest, UpdateSalesOrderRequest,
};
use crate::models::sales_order::{SalesOrder, SalesOrderItem};

pub struct SalesOrderRepo;

impl SalesOrderRepo {
    pub async fn create_with_items(
        pool: &SqlitePool,
        dto: &CreateSalesOrderRequest,
        order_no: &str,
    ) -> Result<SalesOrder, sqlx::Error> {
        let order = sqlx::query_as::<_, SalesOrder>(
            "INSERT INTO sales_orders (order_no, customer_id, order_date, status, \
             total_amount, notes) \
             VALUES (?, ?, ?, 'draft', 0, ?) \
             RETURNING id, order_no, customer_id, order_date, status, total_amount, notes, \
               created_by, created_at, updated_at, deleted_at",
        )
        .bind(order_no)
        .bind(dto.customer_id)
        .bind(&dto.order_date)
        .bind(&dto.notes)
        .fetch_one(pool)
        .await?;

        for item in &dto.items {
            Self::create_item(pool, order.id, item).await?;
        }

        let total: f64 = Self::sum_item_totals(pool, order.id).await?;
        if total > 0.0 {
            sqlx::query("UPDATE sales_orders SET total_amount = ? WHERE id = ?")
                .bind(total)
                .bind(order.id)
                .execute(pool)
                .await?;
        }

        Ok(SalesOrder {
            total_amount: Some(if total > 0.0 { total } else { 0.0 }),
            ..order
        })
    }

    async fn create_item(
        pool: &SqlitePool,
        order_id: i64,
        dto: &CreateSalesItemRequest,
    ) -> Result<SalesOrderItem, sqlx::Error> {
        sqlx::query_as::<_, SalesOrderItem>(
            "INSERT INTO sales_order_items (order_id, pipe_type, grade, od, wt, quantity, \
             delivered_quantity, unit_price, total_price, notes) \
             VALUES (?, ?, ?, ?, ?, ?, 0, ?, ?, ?) \
             RETURNING id, order_id, pipe_type, grade, od, wt, quantity, delivered_quantity, \
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
            "SELECT COALESCE(SUM(total_price), 0) FROM sales_order_items WHERE order_id = ?",
        )
        .bind(order_id)
        .fetch_one(pool)
        .await?;
        Ok(row.0.unwrap_or(0.0))
    }

    pub async fn update_order(
        pool: &SqlitePool,
        id: i64,
        dto: &UpdateSalesOrderRequest,
    ) -> Result<SalesOrder, sqlx::Error> {
        let mut builder: QueryBuilder<Sqlite> =
            QueryBuilder::new("UPDATE sales_orders SET updated_at = datetime('now')");

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
            " AND deleted_at IS NULL RETURNING id, order_no, customer_id, order_date, status, \
             total_amount, notes, created_by, created_at, updated_at, deleted_at",
        );

        builder.build_query_as::<SalesOrder>().fetch_one(pool).await
    }

    pub async fn find_by_id(
        pool: &SqlitePool,
        id: i64,
    ) -> Result<Option<SalesOrder>, sqlx::Error> {
        sqlx::query_as::<_, SalesOrder>(
            "SELECT id, order_no, customer_id, order_date, status, total_amount, notes, \
             created_by, created_at, updated_at, deleted_at \
             FROM sales_orders WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_order_no(
        pool: &SqlitePool,
        order_no: &str,
    ) -> Result<Option<SalesOrder>, sqlx::Error> {
        sqlx::query_as::<_, SalesOrder>(
            "SELECT id, order_no, customer_id, order_date, status, total_amount, notes, \
             created_by, created_at, updated_at, deleted_at \
             FROM sales_orders WHERE order_no = ? AND deleted_at IS NULL",
        )
        .bind(order_no)
        .fetch_optional(pool)
        .await
    }

    pub async fn find_items(
        pool: &SqlitePool,
        order_id: i64,
    ) -> Result<Vec<SalesOrderItem>, sqlx::Error> {
        sqlx::query_as::<_, SalesOrderItem>(
            "SELECT id, order_id, pipe_type, grade, od, wt, quantity, delivered_quantity, \
             unit_price, total_price, notes, created_at \
             FROM sales_order_items WHERE order_id = ? ORDER BY id ASC",
        )
        .bind(order_id)
        .fetch_all(pool)
        .await
    }

    pub async fn update_status(
        pool: &SqlitePool,
        id: i64,
        status: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE sales_orders SET status = ?, updated_at = datetime('now') \
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
            "UPDATE sales_orders SET deleted_at = datetime('now'), \
             updated_at = datetime('now') WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn list(
        pool: &SqlitePool,
        filter: &SalesOrderFilterParams,
        params: &PaginationParams,
    ) -> Result<(Vec<SalesOrder>, u64), sqlx::Error> {
        let page_size = params.page_size();
        let offset = params.offset();

        let mut conditions: Vec<String> = vec!["so.deleted_at IS NULL".into()];
        let mut bind_values: Vec<String> = Vec::new();

        if let Some(ref q) = filter.q {
            if !q.is_empty() {
                conditions.push("(so.order_no LIKE ? OR c.name LIKE ?)".into());
                let pattern = format!("%{}%", q);
                bind_values.push(pattern.clone());
                bind_values.push(pattern);
            }
        }
        if let Some(ref status) = filter.status {
            conditions.push("so.status = ?".into());
            bind_values.push(status.clone());
        }
        if let Some(customer_id) = filter.customer_id {
            conditions.push("so.customer_id = ?".into());
            bind_values.push(customer_id.to_string());
        }
        if let Some(ref from) = filter.order_date_from {
            conditions.push("so.order_date >= ?".into());
            bind_values.push(from.clone());
        }
        if let Some(ref to) = filter.order_date_to {
            conditions.push("so.order_date <= ?".into());
            bind_values.push(to.clone());
        }

        let where_clause = conditions.join(" AND ");

        let sort_by = match params.sort_by.as_deref() {
            Some("order_no") => "so.order_no",
            Some("order_date") => "so.order_date",
            Some("status") => "so.status",
            Some("total_amount") => "so.total_amount",
            _ => "so.created_at",
        };
        let sort_order = params.sort_order_sql();

        let count_sql = format!(
            "SELECT COUNT(*) as cnt FROM sales_orders so \
             LEFT JOIN customers c ON c.id = so.customer_id WHERE {}",
            where_clause
        );
        let mut count_q = sqlx::query_as::<_, (i64,)>(&count_sql);
        for val in &bind_values {
            count_q = count_q.bind(val.as_str());
        }
        let total: (i64,) = count_q.fetch_one(pool).await?;

        let list_sql = format!(
            "SELECT so.id, so.order_no, so.customer_id, so.order_date, so.status, \
             so.total_amount, so.notes, so.created_by, so.created_at, so.updated_at, so.deleted_at \
             FROM sales_orders so \
             LEFT JOIN customers c ON c.id = so.customer_id \
             WHERE {} ORDER BY {} {} LIMIT ? OFFSET ?",
            where_clause, sort_by, sort_order
        );
        let mut list_q = sqlx::query_as::<_, SalesOrder>(&list_sql);
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

    pub async fn update_item(
        pool: &SqlitePool,
        item_id: i64,
        dto: &UpdateSalesItemRequest,
    ) -> Result<SalesOrderItem, sqlx::Error> {
        let mut builder: QueryBuilder<Sqlite> =
            QueryBuilder::new("UPDATE sales_order_items SET");

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
        set_field_opt!(dto.total_price, "total_price");
        set_field!(dto.notes, "notes");

        if first {
            return Err(sqlx::Error::Protocol("No fields to update".into()));
        }

        builder.push(" WHERE id = ");
        builder.push_bind(item_id);
        builder.push(
            " RETURNING id, order_id, pipe_type, grade, od, wt, quantity, delivered_quantity, \
             unit_price, total_price, notes, created_at",
        );

        builder.build_query_as::<SalesOrderItem>().fetch_one(pool).await
    }

    pub async fn delete_item(pool: &SqlitePool, item_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM sales_order_items WHERE id = ?")
            .bind(item_id)
            .execute(pool)
            .await?;
        Ok(())
    }
}
