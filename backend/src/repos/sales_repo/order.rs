//! sales_repo.order — 销售订单头/行

use sqlx::Executor;
use sqlx::SqlitePool;
use rust_decimal::Decimal;
use crate::domain::quantity::serialize_qty_str;
use crate::error::{AppError, ErrorCode};

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct SalesOrderRow {
    pub id: i64,
    pub order_no: String,
    pub customer_id: i64,
    pub customer_name: String,
    pub order_date: String,
    pub status: String,
    pub doc_status: i64,
    pub total_amount: String,
    pub currency: String,
    pub notes: Option<String>,
    pub created_by: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}


#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct SalesOrderItemRow {
    pub id: i64,
    pub order_id: i64,
    pub item_id: i64,
    pub item_name: String,
    #[serde(serialize_with = "serialize_qty_str")]
    pub quantity: String,
    #[serde(serialize_with = "serialize_qty_str")]
    pub shipped_qty: String,
    pub unit_price: Option<String>,
    pub total_price: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
}


// —— Filter ——

#[derive(Debug, Clone, Default)]
pub struct SalesOrderFilter {
    pub customer_id: Option<i64>,
    pub status: Option<String>,
    pub order_date_from: Option<String>,
    pub order_date_to: Option<String>,
    pub order_no: Option<String>,
}


// —— Sales orders ——

pub async fn find_by_id(pool: &SqlitePool, id: i64) -> Result<Option<SalesOrderRow>, AppError> {
    let row = sqlx::query_as::<_, SalesOrderRow>(
        "SELECT so.id, so.order_no, so.customer_id, c.name AS customer_name,
                so.order_date, so.status, so.doc_status,
                so.total_amount, so.currency, so.notes, so.created_by, so.created_at, so.updated_at, so.deleted_at
         FROM sales_orders so
         JOIN customers c ON c.id = so.customer_id
         WHERE so.id = ? AND so.deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}


pub async fn find_by_order_no(
    pool: &SqlitePool,
    order_no: &str,
) -> Result<Option<SalesOrderRow>, AppError> {
    let row = sqlx::query_as::<_, SalesOrderRow>(
        "SELECT so.id, so.order_no, so.customer_id, c.name AS customer_name,
                so.order_date, so.status, so.doc_status,
                so.total_amount, so.currency, so.notes, so.created_by, so.created_at, so.updated_at, so.deleted_at
         FROM sales_orders so
         JOIN customers c ON c.id = so.customer_id
         WHERE so.order_no = ? AND so.deleted_at IS NULL",
    )
    .bind(order_no)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}


/// 插入销售订单头（status / doc_status 由调用方传入）。泛型化以支持事务。
pub async fn insert_order<'e, E>(
    executor: E,
    order_no: &str,
    customer_id: i64,
    order_date: &str,
    status: &str,
    doc_status: i64,
    total_amount: &str,
    currency: &str,
    notes: Option<&str>,
    created_by: Option<i64>,
) -> Result<i64, AppError>
where
    E: Executor<'e, Database = sqlx::Sqlite>,
{
    let result = sqlx::query(
        "INSERT INTO sales_orders
            (order_no, customer_id, order_date, status, doc_status, total_amount,
             currency, notes, created_by)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(order_no)
    .bind(customer_id)
    .bind(order_date)
    .bind(status)
    .bind(doc_status)
    .bind(total_amount)
    .bind(currency)
    .bind(notes)
    .bind(created_by)
    .execute(executor)
    .await?;
    Ok(result.last_insert_rowid())
}


/// 插入销售订单明细行。泛型化以支持事务。
pub async fn insert_item<'e, E>(
    executor: E,
    order_id: i64,
    item_id: i64,
    quantity: Decimal,
    unit_price: Option<&str>,
    total_price: Option<&str>,
    notes: Option<&str>,
) -> Result<i64, AppError>
where
    E: Executor<'e, Database = sqlx::Sqlite>,
{
    let result = sqlx::query(
        "INSERT INTO sales_order_items
            (order_id, item_id, quantity, shipped_qty, unit_price, total_price, notes)
         VALUES (?, ?, ?, '0', ?, ?, ?)",
    )
    .bind(order_id)
    .bind(item_id)
    .bind(quantity.to_string())
    .bind(unit_price)
    .bind(total_price)
    .bind(notes)
    .execute(executor)
    .await?;
    Ok(result.last_insert_rowid())
}


pub async fn list_orders(
    pool: &SqlitePool,
    filter: &SalesOrderFilter,
    page: i64,
    page_size: i64,
) -> Result<(Vec<SalesOrderRow>, i64), AppError> {
    let mut where_clauses: Vec<&'static str> = vec!["deleted_at IS NULL"];
    let mut count_sql = String::from("SELECT COUNT(*) FROM sales_orders so WHERE so.deleted_at IS NULL");
    let mut list_sql = String::from(
        "SELECT so.id, so.order_no, so.customer_id, c.name AS customer_name,
                so.order_date, so.status, so.doc_status,
                so.total_amount, so.currency, so.notes, so.created_by, so.created_at, so.updated_at, so.deleted_at
         FROM sales_orders so
         JOIN customers c ON c.id = so.customer_id
         WHERE so.deleted_at IS NULL",
    );

    if filter.customer_id.is_some() {
        where_clauses.push("so.customer_id = ?");
    }
    if filter.status.is_some() {
        where_clauses.push("so.status = ?");
    }
    if filter.order_date_from.is_some() {
        where_clauses.push("so.order_date >= ?");
    }
    if filter.order_date_to.is_some() {
        where_clauses.push("so.order_date <= ?");
    }
    if filter.order_no.is_some() {
        where_clauses.push("so.order_no LIKE ?");
    }

    if where_clauses.len() > 1 {
        let extra = where_clauses[1..].join(" AND ");
        count_sql.push_str(" AND ");
        count_sql.push_str(&extra);
        list_sql.push_str(" AND ");
        list_sql.push_str(&extra);
    }

    list_sql.push_str(" ORDER BY so.id DESC LIMIT ? OFFSET ?");

    let mut count_q = sqlx::query_scalar::<_, i64>(&count_sql);
    if let Some(v) = filter.customer_id {
        count_q = count_q.bind(v);
    }
    if let Some(v) = &filter.status {
        count_q = count_q.bind(v);
    }
    if let Some(v) = &filter.order_date_from {
        count_q = count_q.bind(v);
    }
    if let Some(v) = &filter.order_date_to {
        count_q = count_q.bind(v);
    }
    if let Some(v) = &filter.order_no {
        let like = format!("%{v}%");
        count_q = count_q.bind(like);
    }
    let total: i64 = count_q.fetch_one(pool).await?;

    let offset = (page - 1).max(0) * page_size;
    let mut list_q = sqlx::query_as::<_, SalesOrderRow>(&list_sql);
    if let Some(v) = filter.customer_id {
        list_q = list_q.bind(v);
    }
    if let Some(v) = &filter.status {
        list_q = list_q.bind(v);
    }
    if let Some(v) = &filter.order_date_from {
        list_q = list_q.bind(v);
    }
    if let Some(v) = &filter.order_date_to {
        list_q = list_q.bind(v);
    }
    if let Some(v) = &filter.order_no {
        let like = format!("%{v}%");
        list_q = list_q.bind(like);
    }
    list_q = list_q.bind(page_size).bind(offset);
    let rows = list_q.fetch_all(pool).await?;
    Ok((rows, total))
}


pub async fn list_items_for_order(
    pool: &SqlitePool,
    order_id: i64,
) -> Result<Vec<SalesOrderItemRow>, AppError> {
    let rows = sqlx::query_as::<_, SalesOrderItemRow>(
        "SELECT si.id, si.order_id, si.item_id, i.name AS item_name,
                si.quantity, si.shipped_qty, si.unit_price, si.total_price,
                si.notes, si.created_at
         FROM sales_order_items si
         JOIN items i ON i.id = si.item_id
         WHERE si.order_id = ? ORDER BY si.id",
    )
    .bind(order_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}


/// 列出销售订单明细（事务版）
pub async fn list_items_for_order_tx<'e, E>(
    executor: E,
    order_id: i64,
) -> Result<Vec<SalesOrderItemRow>, AppError>
where
    E: Executor<'e, Database = sqlx::Sqlite>,
{
    let rows = sqlx::query_as::<_, SalesOrderItemRow>(
        "SELECT si.id, si.order_id, si.item_id, i.name AS item_name,
                si.quantity, si.shipped_qty, si.unit_price, si.total_price,
                si.notes, si.created_at
         FROM sales_order_items si
         JOIN items i ON i.id = si.item_id
         WHERE si.order_id = ? ORDER BY si.id",
    )
    .bind(order_id)
    .fetch_all(executor)
    .await?;
    Ok(rows)
}


/// 更新订单状态（事务版）：status + doc_status
pub async fn update_status_tx<'e, E>(
    executor: E,
    id: i64,
    status: &str,
    doc_status: i64,
) -> Result<(), AppError>
where
    E: Executor<'e, Database = sqlx::Sqlite>,
{
    let result = sqlx::query(
        "UPDATE sales_orders SET status = ?, doc_status = ?, updated_at = datetime('now')
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(status)
    .bind(doc_status)
    .bind(id)
    .execute(executor)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::new(ErrorCode::OrderNotFound, "销售订单未找到"));
    }
    Ok(())
}


/// 更新订单头字段（仅 draft 状态可调用；service 层负责状态校验）
pub async fn update_order(
    pool: &SqlitePool,
    id: i64,
    customer_id: i64,
    order_date: &str,
    total_amount: &str,
    currency: &str,
    notes: Option<&str>,
) -> Result<(), AppError> {
    let result = sqlx::query(
        "UPDATE sales_orders SET customer_id = ?, order_date = ?, total_amount = ?,
           currency = ?, notes = ?, updated_at = datetime('now')
         WHERE id = ? AND deleted_at IS NULL AND status = 'draft'",
    )
    .bind(customer_id)
    .bind(order_date)
    .bind(total_amount)
    .bind(currency)
    .bind(notes)
    .bind(id)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::new(
            ErrorCode::OrderCannotModify,
            "销售订单不可编辑（不存在或非 draft 状态）",
        ));
    }
    Ok(())
}


/// 删除订单明细（用于 update_order 时全量重写明细）
pub async fn delete_items_for_order_tx<'e, E>(executor: E, order_id: i64) -> Result<(), AppError>
where
    E: Executor<'e, Database = sqlx::Sqlite>,
{
    sqlx::query("DELETE FROM sales_order_items WHERE order_id = ?")
        .bind(order_id)
        .execute(executor)
        .await?;
    Ok(())
}


/// 软删除订单（仅 draft 状态可删除）
pub async fn soft_delete_order(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
    let result = sqlx::query(
        "UPDATE sales_orders SET deleted_at = datetime('now')
         WHERE id = ? AND deleted_at IS NULL AND status = 'draft'",
    )
    .bind(id)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::new(
            ErrorCode::OrderCannotModify,
            "销售订单不可删除（不存在或非 draft 状态）",
        ));
    }
    Ok(())
}
