//! Purchase 数据访问 — purchase_orders / purchase_order_items 表（005_purchasing.sql）
//!
//! 纯 SQL（sqlx），无业务逻辑、无事务控制（事务在 service 层）。
//! 金额以 TEXT 存储（rust_decimal canonical string，见 ADR-002）；不对 total_amount 做 SQL SUM，
//! 聚合由 service 层在 Rust app 层完成。

use sqlx::SqlitePool;

use crate::error::{AppError, ErrorCode};
use crate::services::purchase_service::{
    CreatePurchaseOrderRequest, PurchaseOrderItemInput, UpdatePurchaseOrderRequest,
};

// —— Row structs ——

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct PurchaseOrderRow {
    pub id: i64,
    pub order_no: String,
    pub supplier_id: i64,
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
pub struct PurchaseOrderItemRow {
    pub id: i64,
    pub order_id: i64,
    pub item_id: i64,
    pub quantity: f64,
    pub received_qty: f64,
    pub unit_price: Option<String>,
    pub total_price: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
}

// —— Filter struct ——

#[derive(Debug, Clone, Default)]
pub struct PurchaseOrderFilter {
    pub supplier_id: Option<i64>,
    pub status: Option<String>,
    pub order_date_from: Option<String>,
    pub order_date_to: Option<String>,
    pub order_no: Option<String>,
}

// —— Reads ——

pub async fn find_by_id(pool: &SqlitePool, id: i64) -> Result<Option<PurchaseOrderRow>, AppError> {
    let row = sqlx::query_as::<_, PurchaseOrderRow>(
        "SELECT id, order_no, supplier_id, order_date, status, doc_status, total_amount,
                currency, notes, created_by, created_at, updated_at, deleted_at
         FROM purchase_orders WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn find_by_order_no(
    pool: &SqlitePool,
    order_no: &str,
) -> Result<Option<PurchaseOrderRow>, AppError> {
    let row = sqlx::query_as::<_, PurchaseOrderRow>(
        "SELECT id, order_no, supplier_id, order_date, status, doc_status, total_amount,
                currency, notes, created_by, created_at, updated_at, deleted_at
         FROM purchase_orders WHERE order_no = ? AND deleted_at IS NULL",
    )
    .bind(order_no)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn list_items_for_order(
    pool: &SqlitePool,
    order_id: i64,
) -> Result<Vec<PurchaseOrderItemRow>, AppError> {
    let rows = sqlx::query_as::<_, PurchaseOrderItemRow>(
        "SELECT id, order_id, item_id, quantity, received_qty, unit_price, total_price,
                notes, created_at
         FROM purchase_order_items WHERE order_id = ? ORDER BY id",
    )
    .bind(order_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn list_orders(
    pool: &SqlitePool,
    filter: &PurchaseOrderFilter,
    page: i64,
    page_size: i64,
) -> Result<(Vec<PurchaseOrderRow>, i64), AppError> {
    let mut where_clauses: Vec<&'static str> = vec!["deleted_at IS NULL"];
    let mut count_sql =
        String::from("SELECT COUNT(*) FROM purchase_orders WHERE deleted_at IS NULL");
    let mut list_sql = String::from(
        "SELECT id, order_no, supplier_id, order_date, status, doc_status, total_amount,
                currency, notes, created_by, created_at, updated_at, deleted_at
         FROM purchase_orders WHERE deleted_at IS NULL",
    );

    if filter.supplier_id.is_some() {
        where_clauses.push("supplier_id = ?");
    }
    if filter.status.is_some() {
        where_clauses.push("status = ?");
    }
    if filter.order_date_from.is_some() {
        where_clauses.push("order_date >= ?");
    }
    if filter.order_date_to.is_some() {
        where_clauses.push("order_date <= ?");
    }
    if filter.order_no.is_some() {
        where_clauses.push("order_no LIKE ?");
    }

    if where_clauses.len() > 1 {
        let extra = where_clauses[1..].join(" AND ");
        count_sql.push_str(" AND ");
        count_sql.push_str(&extra);
        list_sql.push_str(" AND ");
        list_sql.push_str(&extra);
    }

    list_sql.push_str(" ORDER BY id DESC LIMIT ? OFFSET ?");

    let mut count_q = sqlx::query_scalar::<_, i64>(&count_sql);
    if let Some(v) = filter.supplier_id {
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
    let mut list_q = sqlx::query_as::<_, PurchaseOrderRow>(&list_sql);
    if let Some(v) = filter.supplier_id {
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

// —— Writes ——

pub async fn insert_order(
    pool: &SqlitePool,
    order_no: &str,
    dto: &CreatePurchaseOrderRequest,
    user_id: i64,
    total_amount_text: &str,
) -> Result<PurchaseOrderRow, AppError> {
    let currency = dto.currency.as_deref().unwrap_or("CNY");
    let result = sqlx::query(
        "INSERT INTO purchase_orders
            (order_no, supplier_id, order_date, status, doc_status, total_amount,
             currency, notes, created_by)
         VALUES (?, ?, ?, 'draft', 0, ?, ?, ?, ?)",
    )
    .bind(order_no)
    .bind(dto.supplier_id)
    .bind(&dto.order_date)
    .bind(total_amount_text)
    .bind(currency)
    .bind(dto.notes.as_deref())
    .bind(user_id)
    .execute(pool)
    .await?;
    let id = result.last_insert_rowid();
    find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::Internal, "采购订单创建后读取失败"))
}

pub async fn insert_item(
    pool: &SqlitePool,
    order_id: i64,
    item: &PurchaseOrderItemInput,
    total_price_text: &str,
) -> Result<PurchaseOrderItemRow, AppError> {
    let result = sqlx::query(
        "INSERT INTO purchase_order_items
            (order_id, item_id, quantity, received_qty, unit_price, total_price, notes)
         VALUES (?, ?, ?, 0, ?, ?, ?)",
    )
    .bind(order_id)
    .bind(item.item_id)
    .bind(item.quantity)
    .bind(item.unit_price.as_deref())
    .bind(total_price_text)
    .bind(item.notes.as_deref())
    .execute(pool)
    .await?;
    let id = result.last_insert_rowid();
    let row = sqlx::query_as::<_, PurchaseOrderItemRow>(
        "SELECT id, order_id, item_id, quantity, received_qty, unit_price, total_price,
                notes, created_at
         FROM purchase_order_items WHERE id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn update_status(
    pool: &SqlitePool,
    id: i64,
    status: &str,
    doc_status: Option<i64>,
) -> Result<(), AppError> {
    let result = match doc_status {
        Some(ds) => sqlx::query(
            "UPDATE purchase_orders SET status = ?, doc_status = ?, updated_at = datetime('now')
                 WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(status)
        .bind(ds)
        .bind(id)
        .execute(pool)
        .await?,
        None => {
            sqlx::query(
                "UPDATE purchase_orders SET status = ?, updated_at = datetime('now')
                 WHERE id = ? AND deleted_at IS NULL",
            )
            .bind(status)
            .bind(id)
            .execute(pool)
            .await?
        }
    };
    if result.rows_affected() == 0 {
        return Err(AppError::new(ErrorCode::OrderNotFound, "采购订单未找到"));
    }
    Ok(())
}

pub async fn update_order(
    pool: &SqlitePool,
    id: i64,
    dto: &UpdatePurchaseOrderRequest,
    total_amount_text: &str,
) -> Result<(), AppError> {
    let currency = dto.currency.as_deref().unwrap_or("CNY");
    let result = sqlx::query(
        "UPDATE purchase_orders SET supplier_id = ?, order_date = ?, total_amount = ?,
             currency = ?, notes = ?, updated_at = datetime('now')
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(dto.supplier_id)
    .bind(&dto.order_date)
    .bind(total_amount_text)
    .bind(currency)
    .bind(dto.notes.as_deref())
    .bind(id)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::new(ErrorCode::OrderNotFound, "采购订单未找到"));
    }
    Ok(())
}

/// 删除订单行（用于 update_order 时重建明细）
pub async fn delete_items_for_order(pool: &SqlitePool, order_id: i64) -> Result<(), AppError> {
    sqlx::query("DELETE FROM purchase_order_items WHERE order_id = ?")
        .bind(order_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn soft_delete_order(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
    let result = sqlx::query(
        "UPDATE purchase_orders SET deleted_at = datetime('now')
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(id)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::new(ErrorCode::OrderNotFound, "采购订单未找到"));
    }
    Ok(())
}
