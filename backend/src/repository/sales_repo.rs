use sqlx::SqlitePool;
use crate::domain::{Customer, SalesOrder, SalesOrderItem};
use crate::error::{AppError, AppResult};

// ── Customer ────────────────────────────────────────────────────────────────

pub struct CustomerRepo;

impl CustomerRepo {
    pub async fn create(pool: &SqlitePool, c: &Customer) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO customers (id, name, contact_person, phone, email, address, is_active)
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&c.id).bind(&c.name)
        .bind(&c.contact_person).bind(&c.phone).bind(&c.email)
        .bind(&c.address).bind(c.is_active)
        .execute(pool).await?;
        Ok(())
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> AppResult<Customer> {
        let c = sqlx::query_as::<_, Customer>(
            "SELECT id, name, contact_person, phone, email, address, is_active, created_at, updated_at
             FROM customers WHERE id = ?"
        )
        .bind(id).fetch_optional(pool).await?
        .ok_or_else(|| AppError::NotFound("Customer not found".into()))?;
        Ok(c)
    }

    pub async fn list(
        pool: &SqlitePool,
        search: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> AppResult<Vec<Customer>> {
        let offset = (page - 1) * page_size;
        if let Some(keyword) = search.filter(|s| !s.is_empty()) {
            let pattern = format!("%{}%", keyword);
            let rows = sqlx::query_as::<_, Customer>(
                "SELECT id, name, contact_person, phone, email, address, is_active, created_at, updated_at
                 FROM customers
                 WHERE name LIKE ? OR contact_person LIKE ? OR phone LIKE ?
                 ORDER BY created_at DESC LIMIT ? OFFSET ?"
            )
            .bind(&pattern).bind(&pattern).bind(&pattern)
            .bind(page_size).bind(offset)
            .fetch_all(pool).await?;
            Ok(rows)
        } else {
            let rows = sqlx::query_as::<_, Customer>(
                "SELECT id, name, contact_person, phone, email, address, is_active, created_at, updated_at
                 FROM customers ORDER BY created_at DESC LIMIT ? OFFSET ?"
            )
            .bind(page_size).bind(offset)
            .fetch_all(pool).await?;
            Ok(rows)
        }
    }

    pub async fn count(pool: &SqlitePool, search: Option<&str>) -> AppResult<i64> {
        if let Some(keyword) = search.filter(|s| !s.is_empty()) {
            let pattern = format!("%{}%", keyword);
            let count: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM customers WHERE name LIKE ? OR contact_person LIKE ? OR phone LIKE ?"
            )
            .bind(&pattern).bind(&pattern).bind(&pattern)
            .fetch_one(pool).await?;
            Ok(count.0)
        } else {
            let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM customers")
                .fetch_one(pool).await?;
            Ok(count.0)
        }
    }

    pub async fn update(pool: &SqlitePool, c: &Customer) -> AppResult<()> {
        let affected = sqlx::query(
            "UPDATE customers SET name = ?, contact_person = ?, phone = ?, email = ?, address = ?, is_active = ?, updated_at = datetime('now')
             WHERE id = ?"
        )
        .bind(&c.name).bind(&c.contact_person).bind(&c.phone)
        .bind(&c.email).bind(&c.address)
        .bind(c.is_active).bind(&c.id)
        .execute(pool).await?.rows_affected();
        if affected == 0 {
            return Err(AppError::NotFound("Customer not found".into()));
        }
        Ok(())
    }

    pub async fn delete(pool: &SqlitePool, id: &str) -> AppResult<()> {
        let affected = sqlx::query("DELETE FROM customers WHERE id = ?")
            .bind(id).execute(pool).await?.rows_affected();
        if affected == 0 {
            return Err(AppError::NotFound("Customer not found".into()));
        }
        Ok(())
    }
}

// ── Sales Order ─────────────────────────────────────────────────────────────

pub struct SalesOrderRepo;

impl SalesOrderRepo {
    pub async fn create(pool: &SqlitePool, order: &SalesOrder, items: &[SalesOrderItem]) -> AppResult<()> {
        let mut tx = pool.begin().await?;
        sqlx::query(
            "INSERT INTO sales_orders (id, order_no, customer_id, status, total_amount, notes, operator_id)
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&order.id).bind(&order.order_no)
        .bind(&order.customer_id).bind(&order.status)
        .bind(order.total_amount).bind(&order.notes)
        .bind(&order.operator_id)
        .execute(&mut *tx).await?;

        for item in items {
            sqlx::query(
                "INSERT INTO sales_order_items (id, order_id, pipe_type, grade, od, wt, quantity, delivered_quantity, unit_price, subtotal)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(&item.id).bind(&item.order_id)
            .bind(&item.pipe_type).bind(&item.grade)
            .bind(item.od).bind(item.wt)
            .bind(item.quantity).bind(item.delivered_quantity)
            .bind(item.unit_price).bind(item.subtotal)
            .execute(&mut *tx).await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn update_status(pool: &SqlitePool, id: &str, status: &str) -> AppResult<()> {
        let affected = sqlx::query(
            "UPDATE sales_orders SET status = ?, updated_at = datetime('now') WHERE id = ?"
        )
        .bind(status).bind(id)
        .execute(pool).await?.rows_affected();
        if affected == 0 {
            return Err(AppError::NotFound("Sales order not found".into()));
        }
        Ok(())
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> AppResult<(SalesOrder, Vec<SalesOrderItem>)> {
        let order = sqlx::query_as::<_, SalesOrder>(
            "SELECT id, order_no, customer_id, status, total_amount, notes, operator_id, created_at, updated_at
             FROM sales_orders WHERE id = ?"
        )
        .bind(id).fetch_optional(pool).await?
        .ok_or_else(|| AppError::NotFound("Sales order not found".into()))?;

        let items = sqlx::query_as::<_, SalesOrderItem>(
            "SELECT id, order_id, pipe_type, grade, od, wt, quantity, delivered_quantity, unit_price, subtotal
             FROM sales_order_items WHERE order_id = ?"
        )
        .bind(id).fetch_all(pool).await?;

        Ok((order, items))
    }

    pub async fn list(
        pool: &SqlitePool,
        status: Option<&str>,
        customer_id: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> AppResult<Vec<(SalesOrder, Option<String>, Option<String>)>> {
        let offset = (page - 1) * page_size;

        #[derive(sqlx::FromRow)]
        struct SoRow {
            id: String,
            order_no: String,
            customer_id: String,
            status: String,
            total_amount: f64,
            notes: Option<String>,
            operator_id: String,
            created_at: String,
            updated_at: String,
            customer_name: Option<String>,
            operator_name: Option<String>,
        }

        let mut sql = String::from(
            "SELECT so.id, so.order_no, so.customer_id, so.status, so.total_amount, so.notes, so.operator_id, so.created_at, so.updated_at,
                    c.name as customer_name,
                    u.display_name as operator_name
             FROM sales_orders so
             LEFT JOIN customers c ON c.id = so.customer_id
             LEFT JOIN users u ON u.id = so.operator_id
             WHERE 1=1"
        );
        let mut params: Vec<String> = Vec::new();

        if let Some(s) = status.filter(|s| !s.is_empty()) {
            params.push(format!("so.status = '{}'", s.replace('\'', "''")));
        }
        if let Some(cid) = customer_id.filter(|s| !s.is_empty()) {
            params.push(format!("so.customer_id = '{}'", cid.replace('\'', "''")));
        }
        if !params.is_empty() {
            sql.push_str(" AND ");
            sql.push_str(&params.join(" AND "));
        }
        sql.push_str(" ORDER BY so.created_at DESC LIMIT ? OFFSET ?");

        let mut q = sqlx::query_as::<_, SoRow>(&sql);
        q = q.bind(page_size).bind(offset);

        let rows = q.fetch_all(pool).await?;
        let result = rows.into_iter().map(|r| {
            let order = SalesOrder {
                id: r.id,
                order_no: r.order_no,
                customer_id: r.customer_id,
                status: r.status,
                total_amount: r.total_amount,
                notes: r.notes,
                operator_id: r.operator_id,
                created_at: r.created_at,
                updated_at: r.updated_at,
            };
            (order, r.customer_name, r.operator_name)
        }).collect();
        Ok(result)
    }

    pub async fn count(pool: &SqlitePool, status: Option<&str>, customer_id: Option<&str>) -> AppResult<i64> {
        let mut sql = String::from("SELECT COUNT(*) FROM sales_orders so WHERE 1=1");
        let mut params: Vec<String> = Vec::new();

        if let Some(s) = status.filter(|s| !s.is_empty()) {
            params.push(format!("so.status = '{}'", s.replace('\'', "''")));
        }
        if let Some(cid) = customer_id.filter(|s| !s.is_empty()) {
            params.push(format!("so.customer_id = '{}'", cid.replace('\'', "''")));
        }
        if !params.is_empty() {
            sql.push_str(" AND ");
            sql.push_str(&params.join(" AND "));
        }

        let count: (i64,) = sqlx::query_as(&sql).fetch_one(pool).await?;
        Ok(count.0)
    }

    pub async fn update_delivered_quantity(pool: &SqlitePool, item_id: &str, qty: i32) -> AppResult<()> {
        let affected = sqlx::query(
            "UPDATE sales_order_items SET delivered_quantity = ? WHERE id = ?"
        )
        .bind(qty).bind(item_id)
        .execute(pool).await?.rows_affected();
        if affected == 0 {
            return Err(AppError::NotFound("Sales order item not found".into()));
        }
        Ok(())
    }
}

// ── Sales Order Items ───────────────────────────────────────────────────────

pub struct SalesOrderItemRepo;

impl SalesOrderItemRepo {
    pub async fn batch_create(pool: &SqlitePool, items: &[SalesOrderItem]) -> AppResult<()> {
        let mut tx = pool.begin().await?;
        for item in items {
            sqlx::query(
                "INSERT INTO sales_order_items (id, order_id, pipe_type, grade, od, wt, quantity, delivered_quantity, unit_price, subtotal)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(&item.id).bind(&item.order_id)
            .bind(&item.pipe_type).bind(&item.grade)
            .bind(item.od).bind(item.wt)
            .bind(item.quantity).bind(item.delivered_quantity)
            .bind(item.unit_price).bind(item.subtotal)
            .execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn find_by_order(pool: &SqlitePool, order_id: &str) -> AppResult<Vec<SalesOrderItem>> {
        let items = sqlx::query_as::<_, SalesOrderItem>(
            "SELECT id, order_id, pipe_type, grade, od, wt, quantity, delivered_quantity, unit_price, subtotal
             FROM sales_order_items WHERE order_id = ?"
        )
        .bind(order_id).fetch_all(pool).await?;
        Ok(items)
    }
}
