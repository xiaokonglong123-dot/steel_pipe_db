//! 出库单层 — outbound_records / outbound_items

use sqlx::Executor;
use sqlx::SqlitePool;
use rust_decimal::Decimal;
use crate::domain::quantity::serialize_qty_str;
use crate::error::{AppError, ErrorCode};

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct OutboundOrderRow {
    pub id: i64,
    pub record_no: String,
    pub outbound_type: String,
    pub order_id: Option<i64>,
    pub customer_id: Option<i64>,
    pub status: String,
    pub notes: Option<String>,
    pub created_by: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct OutboundOrderItemRow {
    pub id: i64,
    pub record_id: i64,
    pub item_id: i64,
    pub location_id: Option<i64>,
    #[serde(serialize_with = "serialize_qty_str")]
    pub quantity: String,
    pub notes: Option<String>,
    pub created_at: String,
}

// —— Filter structs ——

#[derive(Debug, Clone, Default)]
pub struct OutboundOrderFilter {
    pub status: Option<String>,
    pub outbound_type: Option<String>,
}

// —— Outbound orders (mirror inbound) ——

pub async fn create_outbound_order(
    pool: &SqlitePool,
    record_no: &str,
    outbound_type: &str,
    order_id: Option<i64>,
    customer_id: Option<i64>,
    created_by: Option<i64>,
    notes: Option<&str>,
) -> Result<OutboundOrderRow, AppError> {
    let result = sqlx::query(
        "INSERT INTO outbound_records
            (record_no, outbound_type, order_id, customer_id, status, notes, created_by)
         VALUES (?, ?, ?, ?, 'draft', ?, ?)",
    )
    .bind(record_no)
    .bind(outbound_type)
    .bind(order_id)
    .bind(customer_id)
    .bind(notes)
    .bind(created_by)
    .execute(pool)
    .await?;
    let id = result.last_insert_rowid();
    get_outbound_order_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::Internal, "出库单创建后读取失败"))
}

pub async fn get_outbound_order_by_id(
    pool: &SqlitePool,
    id: i64,
) -> Result<Option<OutboundOrderRow>, AppError> {
    let row = sqlx::query_as::<_, OutboundOrderRow>(
        "SELECT id, record_no, outbound_type, order_id, customer_id, status, notes,
                created_by, created_at, updated_at, deleted_at
         FROM outbound_records WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn insert_outbound_item(
    pool: &SqlitePool,
    record_id: i64,
    item_id: i64,
    location_id: Option<i64>,
    quantity: Decimal,
    notes: Option<&str>,
) -> Result<i64, AppError> {
    let result = sqlx::query(
        "INSERT INTO outbound_items (record_id, item_id, location_id, quantity, notes)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(record_id)
    .bind(item_id)
    .bind(location_id)
    .bind(quantity.to_string())
    .bind(notes)
    .execute(pool)
    .await?;
    Ok(result.last_insert_rowid())
}

pub async fn list_outbound_items_for_order(
    pool: &SqlitePool,
    record_id: i64,
) -> Result<Vec<OutboundOrderItemRow>, AppError> {
    let rows = sqlx::query_as::<_, OutboundOrderItemRow>(
        "SELECT id, record_id, item_id, location_id, quantity, notes, created_at
         FROM outbound_items WHERE record_id = ? ORDER BY id",
    )
    .bind(record_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn list_outbound_orders(
    pool: &SqlitePool,
    filter: &OutboundOrderFilter,
    page: i64,
    page_size: i64,
) -> Result<(Vec<OutboundOrderRow>, i64), AppError> {
    let mut where_clauses: Vec<&'static str> = vec!["deleted_at IS NULL"];
    let mut count_sql =
        String::from("SELECT COUNT(*) FROM outbound_records WHERE deleted_at IS NULL");
    let mut list_sql = String::from(
        "SELECT id, record_no, outbound_type, order_id, customer_id, status, notes,
                created_by, created_at, updated_at, deleted_at
         FROM outbound_records WHERE deleted_at IS NULL",
    );

    if filter.status.is_some() {
        where_clauses.push("status = ?");
    }
    if filter.outbound_type.is_some() {
        where_clauses.push("outbound_type = ?");
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
    if let Some(v) = &filter.status {
        count_q = count_q.bind(v);
    }
    if let Some(v) = &filter.outbound_type {
        count_q = count_q.bind(v);
    }
    let total: i64 = count_q.fetch_one(pool).await?;

    let offset = (page - 1).max(0) * page_size;
    let mut list_q = sqlx::query_as::<_, OutboundOrderRow>(&list_sql);
    if let Some(v) = &filter.status {
        list_q = list_q.bind(v);
    }
    if let Some(v) = &filter.outbound_type {
        list_q = list_q.bind(v);
    }
    list_q = list_q.bind(page_size).bind(offset);
    let rows = list_q.fetch_all(pool).await?;
    Ok((rows, total))
}

pub async fn update_outbound_status_tx<'e, E>(
    executor: E,
    id: i64,
    status: &str,
) -> Result<(), AppError>
where
    E: Executor<'e, Database = sqlx::Sqlite>,
{
    let result = sqlx::query(
        "UPDATE outbound_records SET status = ?, updated_at = datetime('now')
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(status)
    .bind(id)
    .execute(executor)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::new(ErrorCode::OrderNotFound, "出库单未找到"));
    }
    Ok(())
}

pub async fn list_outbound_items_for_order_tx<'e, E>(
    executor: E,
    record_id: i64,
) -> Result<Vec<OutboundOrderItemRow>, AppError>
where
    E: Executor<'e, Database = sqlx::Sqlite>,
{
    let rows = sqlx::query_as::<_, OutboundOrderItemRow>(
        "SELECT id, record_id, item_id, location_id, quantity, notes, created_at
         FROM outbound_items WHERE record_id = ? ORDER BY id",
    )
    .bind(record_id)
    .fetch_all(executor)
    .await?;
    Ok(rows)
}
