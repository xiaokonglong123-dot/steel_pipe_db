use sqlx::SqlitePool;
use crate::domain::{Supplier, PurchaseOrder, PurchaseOrderItem};
use crate::error::{AppError, AppResult};

// ── Supplier ────────────────────────────────────────────────────────────────

pub struct SupplierRepo;

impl SupplierRepo {
    pub async fn create(pool: &SqlitePool, s: &Supplier) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO suppliers (id, name, contact_person, phone, email, address, cert_info, is_active)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&s.id).bind(&s.name)
        .bind(&s.contact_person).bind(&s.phone).bind(&s.email)
        .bind(&s.address).bind(&s.cert_info).bind(s.is_active)
        .execute(pool).await?;
        Ok(())
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> AppResult<Supplier> {
        let s = sqlx::query_as::<_, Supplier>(
            "SELECT id, name, contact_person, phone, email, address, cert_info, is_active, created_at, updated_at
             FROM suppliers WHERE id = ?"
        )
        .bind(id).fetch_optional(pool).await?
        .ok_or_else(|| AppError::NotFound("Supplier not found".into()))?;
        Ok(s)
    }

    pub async fn list(
        pool: &SqlitePool,
        search: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> AppResult<Vec<Supplier>> {
        let offset = (page - 1) * page_size;
        if let Some(keyword) = search.filter(|s| !s.is_empty()) {
            let pattern = format!("%{}%", keyword);
            let rows = sqlx::query_as::<_, Supplier>(
                "SELECT id, name, contact_person, phone, email, address, cert_info, is_active, created_at, updated_at
                 FROM suppliers
                 WHERE name LIKE ? OR contact_person LIKE ? OR phone LIKE ?
                 ORDER BY created_at DESC LIMIT ? OFFSET ?"
            )
            .bind(&pattern).bind(&pattern).bind(&pattern)
            .bind(page_size).bind(offset)
            .fetch_all(pool).await?;
            Ok(rows)
        } else {
            let rows = sqlx::query_as::<_, Supplier>(
                "SELECT id, name, contact_person, phone, email, address, cert_info, is_active, created_at, updated_at
                 FROM suppliers ORDER BY created_at DESC LIMIT ? OFFSET ?"
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
                "SELECT COUNT(*) FROM suppliers WHERE name LIKE ? OR contact_person LIKE ? OR phone LIKE ?"
            )
            .bind(&pattern).bind(&pattern).bind(&pattern)
            .fetch_one(pool).await?;
            Ok(count.0)
        } else {
            let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM suppliers")
                .fetch_one(pool).await?;
            Ok(count.0)
        }
    }

    pub async fn update(pool: &SqlitePool, s: &Supplier) -> AppResult<()> {
        let affected = sqlx::query(
            "UPDATE suppliers SET name = ?, contact_person = ?, phone = ?, email = ?, address = ?, cert_info = ?, is_active = ?, updated_at = datetime('now')
             WHERE id = ?"
        )
        .bind(&s.name).bind(&s.contact_person).bind(&s.phone)
        .bind(&s.email).bind(&s.address).bind(&s.cert_info)
        .bind(s.is_active).bind(&s.id)
        .execute(pool).await?.rows_affected();
        if affected == 0 {
            return Err(AppError::NotFound("Supplier not found".into()));
        }
        Ok(())
    }

    pub async fn delete(pool: &SqlitePool, id: &str) -> AppResult<()> {
        let affected = sqlx::query("DELETE FROM suppliers WHERE id = ?")
            .bind(id).execute(pool).await?.rows_affected();
        if affected == 0 {
            return Err(AppError::NotFound("Supplier not found".into()));
        }
        Ok(())
    }
}

// ── Purchase Order ──────────────────────────────────────────────────────────

pub struct PurchaseOrderRepo;

impl PurchaseOrderRepo {
    /// Creates a purchase order and its items in a single transaction.
    pub async fn create(pool: &SqlitePool, order: &PurchaseOrder, items: &[PurchaseOrderItem]) -> AppResult<()> {
        let mut tx = pool.begin().await?;
        sqlx::query(
            "INSERT INTO purchase_orders (id, order_no, supplier_id, status, total_amount, notes, operator_id)
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&order.id).bind(&order.order_no)
        .bind(&order.supplier_id).bind(&order.status)
        .bind(order.total_amount).bind(&order.notes)
        .bind(&order.operator_id)
        .execute(&mut *tx).await?;

        for item in items {
            sqlx::query(
                "INSERT INTO purchase_order_items (id, order_id, pipe_type, grade, od, wt, quantity, received_quantity, unit_price, subtotal)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(&item.id).bind(&item.order_id)
            .bind(&item.pipe_type).bind(&item.grade)
            .bind(item.od).bind(item.wt)
            .bind(item.quantity).bind(item.received_quantity)
            .bind(item.unit_price).bind(item.subtotal)
            .execute(&mut *tx).await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn update_status(pool: &SqlitePool, id: &str, status: &str) -> AppResult<()> {
        let affected = sqlx::query(
            "UPDATE purchase_orders SET status = ?, updated_at = datetime('now') WHERE id = ?"
        )
        .bind(status).bind(id)
        .execute(pool).await?.rows_affected();
        if affected == 0 {
            return Err(AppError::NotFound("Purchase order not found".into()));
        }
        Ok(())
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> AppResult<(PurchaseOrder, Vec<PurchaseOrderItem>)> {
        let order = sqlx::query_as::<_, PurchaseOrder>(
            "SELECT id, order_no, supplier_id, status, total_amount, notes, operator_id, created_at, updated_at
             FROM purchase_orders WHERE id = ?"
        )
        .bind(id).fetch_optional(pool).await?
        .ok_or_else(|| AppError::NotFound("Purchase order not found".into()))?;

        let items = sqlx::query_as::<_, PurchaseOrderItem>(
            "SELECT id, order_id, pipe_type, grade, od, wt, quantity, received_quantity, unit_price, subtotal
             FROM purchase_order_items WHERE order_id = ?"
        )
        .bind(id).fetch_all(pool).await?;

        Ok((order, items))
    }

    pub async fn list(
        pool: &SqlitePool,
        status: Option<&str>,
        supplier_id: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> AppResult<Vec<(PurchaseOrder, Option<String>, Option<String>)>> {
        let offset = (page - 1) * page_size;
        let mut sql = String::from(
            "SELECT po.id, po.order_no, po.supplier_id, po.status, po.total_amount, po.notes, po.operator_id, po.created_at, po.updated_at,
                    s.name as supplier_name,
                    u.display_name as operator_name
             FROM purchase_orders po
             LEFT JOIN suppliers s ON s.id = po.supplier_id
             LEFT JOIN users u ON u.id = po.operator_id
             WHERE 1=1"
        );
        let mut params: Vec<String> = Vec::new();

        if let Some(s) = status.filter(|s| !s.is_empty()) {
            params.push(format!("po.status = '{}'", s.replace('\'', "''")));
        }
        if let Some(sid) = supplier_id.filter(|s| !s.is_empty()) {
            params.push(format!("po.supplier_id = '{}'", sid.replace('\'', "''")));
        }
        if !params.is_empty() {
            sql.push_str(" AND ");
            sql.push_str(&params.join(" AND "));
        }
        sql.push_str(" ORDER BY po.created_at DESC LIMIT ? OFFSET ?");

        #[derive(sqlx::FromRow)]
        struct PoRow {
            id: String,
            order_no: String,
            supplier_id: String,
            status: String,
            total_amount: f64,
            notes: Option<String>,
            operator_id: String,
            created_at: String,
            updated_at: String,
            supplier_name: Option<String>,
            operator_name: Option<String>,
        }

        let mut q = sqlx::query_as::<_, PoRow>(&sql);
        q = q.bind(page_size).bind(offset);

        let rows = q.fetch_all(pool).await?;
        let result = rows.into_iter().map(|r| {
            let po = PurchaseOrder {
                id: r.id,
                order_no: r.order_no,
                supplier_id: r.supplier_id,
                status: r.status,
                total_amount: r.total_amount,
                notes: r.notes,
                operator_id: r.operator_id,
                created_at: r.created_at,
                updated_at: r.updated_at,
            };
            (po, r.supplier_name, r.operator_name)
        }).collect();
        Ok(result)
    }

    pub async fn count(pool: &SqlitePool, status: Option<&str>, supplier_id: Option<&str>) -> AppResult<i64> {
        let mut sql = String::from("SELECT COUNT(*) FROM purchase_orders po WHERE 1=1");
        let mut params: Vec<String> = Vec::new();

        if let Some(s) = status.filter(|s| !s.is_empty()) {
            params.push(format!("po.status = '{}'", s.replace('\'', "''")));
        }
        if let Some(sid) = supplier_id.filter(|s| !s.is_empty()) {
            params.push(format!("po.supplier_id = '{}'", sid.replace('\'', "''")));
        }
        if !params.is_empty() {
            sql.push_str(" AND ");
            sql.push_str(&params.join(" AND "));
        }

        let count: (i64,) = sqlx::query_as(&sql).fetch_one(pool).await?;
        Ok(count.0)
    }

    pub async fn update_received_quantity(pool: &SqlitePool, item_id: &str, qty: i32) -> AppResult<()> {
        let affected = sqlx::query(
            "UPDATE purchase_order_items SET received_quantity = ? WHERE id = ?"
        )
        .bind(qty).bind(item_id)
        .execute(pool).await?.rows_affected();
        if affected == 0 {
            return Err(AppError::NotFound("Purchase order item not found".into()));
        }
        Ok(())
    }
}

// ── Purchase Order Items ────────────────────────────────────────────────────

pub struct PurchaseOrderItemRepo;

impl PurchaseOrderItemRepo {
    pub async fn batch_create(pool: &SqlitePool, items: &[PurchaseOrderItem]) -> AppResult<()> {
        let mut tx = pool.begin().await?;
        for item in items {
            sqlx::query(
                "INSERT INTO purchase_order_items (id, order_id, pipe_type, grade, od, wt, quantity, received_quantity, unit_price, subtotal)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(&item.id).bind(&item.order_id)
            .bind(&item.pipe_type).bind(&item.grade)
            .bind(item.od).bind(item.wt)
            .bind(item.quantity).bind(item.received_quantity)
            .bind(item.unit_price).bind(item.subtotal)
            .execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn find_by_order(pool: &SqlitePool, order_id: &str) -> AppResult<Vec<PurchaseOrderItem>> {
        let items = sqlx::query_as::<_, PurchaseOrderItem>(
            "SELECT id, order_id, pipe_type, grade, od, wt, quantity, received_quantity, unit_price, subtotal
             FROM purchase_order_items WHERE order_id = ?"
        )
        .bind(order_id).fetch_all(pool).await?;
        Ok(items)
    }
}
