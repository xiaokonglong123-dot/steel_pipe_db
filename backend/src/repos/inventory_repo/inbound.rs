//! 入库单层 — inbound_records / inbound_items

use sqlx::Executor;
use sqlx::SqlitePool;
use rust_decimal::Decimal;
use crate::domain::quantity::serialize_qty_str;
use crate::error::{AppError, ErrorCode};

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct InboundOrderRow {
    pub id: i64,
    pub record_no: String,
    pub inbound_type: String,
    pub order_id: Option<i64>,
    pub supplier_id: Option<i64>,
    pub status: String,
    pub notes: Option<String>,
    pub created_by: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct InboundOrderItemRow {
    pub id: i64,
    pub record_id: i64,
    pub item_id: i64,
    pub location_id: Option<i64>,
    #[serde(serialize_with = "serialize_qty_str")]
    pub quantity: String,
    pub notes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Default)]
pub struct InboundOrderFilter {
    pub status: Option<String>,
    pub inbound_type: Option<String>,
}

// —— Inbound orders ——

pub async fn create_inbound_order(
    pool: &SqlitePool,
    record_no: &str,
    inbound_type: &str,
    order_id: Option<i64>,
    supplier_id: Option<i64>,
    created_by: Option<i64>,
    notes: Option<&str>,
) -> Result<InboundOrderRow, AppError> {
    let result = sqlx::query(
        "INSERT INTO inbound_records
            (record_no, inbound_type, order_id, supplier_id, status, notes, created_by)
         VALUES (?, ?, ?, ?, 'draft', ?, ?)",
    )
    .bind(record_no)
    .bind(inbound_type)
    .bind(order_id)
    .bind(supplier_id)
    .bind(notes)
    .bind(created_by)
    .execute(pool)
    .await?;
    let id = result.last_insert_rowid();
    get_inbound_order_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::Internal, "入库单创建后读取失败"))
}

pub async fn get_inbound_order_by_id(
    pool: &SqlitePool,
    id: i64,
) -> Result<Option<InboundOrderRow>, AppError> {
    let row = sqlx::query_as::<_, InboundOrderRow>(
        "SELECT id, record_no, inbound_type, order_id, supplier_id, status, notes,
                created_by, created_at, updated_at, deleted_at
         FROM inbound_records WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn insert_inbound_item(
    pool: &SqlitePool,
    record_id: i64,
    item_id: i64,
    location_id: Option<i64>,
    quantity: Decimal,
    notes: Option<&str>,
) -> Result<i64, AppError> {
    let result = sqlx::query(
        "INSERT INTO inbound_items (record_id, item_id, location_id, quantity, notes)
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

pub async fn list_inbound_items_for_order(
    pool: &SqlitePool,
    record_id: i64,
) -> Result<Vec<InboundOrderItemRow>, AppError> {
    let rows = sqlx::query_as::<_, InboundOrderItemRow>(
        "SELECT id, record_id, item_id, location_id, quantity, notes, created_at
         FROM inbound_items WHERE record_id = ? ORDER BY id",
    )
    .bind(record_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn list_inbound_orders(
    pool: &SqlitePool,
    filter: &InboundOrderFilter,
    page: i64,
    page_size: i64,
) -> Result<(Vec<InboundOrderRow>, i64), AppError> {
    let mut where_clauses: Vec<&'static str> = vec!["deleted_at IS NULL"];
    let mut count_sql =
        String::from("SELECT COUNT(*) FROM inbound_records WHERE deleted_at IS NULL");
    let mut list_sql = String::from(
        "SELECT id, record_no, inbound_type, order_id, supplier_id, status, notes,
                created_by, created_at, updated_at, deleted_at
         FROM inbound_records WHERE deleted_at IS NULL",
    );

    if filter.status.is_some() {
        where_clauses.push("status = ?");
    }
    if filter.inbound_type.is_some() {
        where_clauses.push("inbound_type = ?");
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
    if let Some(v) = &filter.inbound_type {
        count_q = count_q.bind(v);
    }
    let total: i64 = count_q.fetch_one(pool).await?;

    let offset = (page - 1).max(0) * page_size;
    let mut list_q = sqlx::query_as::<_, InboundOrderRow>(&list_sql);
    if let Some(v) = &filter.status {
        list_q = list_q.bind(v);
    }
    if let Some(v) = &filter.inbound_type {
        list_q = list_q.bind(v);
    }
    list_q = list_q.bind(page_size).bind(offset);
    let rows = list_q.fetch_all(pool).await?;
    Ok((rows, total))
}

/// 在事务执行器上更新入库单状态（service 层 post_inbound 在提交前调用）
pub async fn update_inbound_status_tx<'e, E>(
    executor: E,
    id: i64,
    status: &str,
) -> Result<(), AppError>
where
    E: Executor<'e, Database = sqlx::Sqlite>,
{
    let result = sqlx::query(
        "UPDATE inbound_records SET status = ?, updated_at = datetime('now')
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(status)
    .bind(id)
    .execute(executor)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::new(ErrorCode::OrderNotFound, "入库单未找到"));
    }
    Ok(())
}

pub async fn list_inbound_items_for_order_tx<'e, E>(
    executor: E,
    record_id: i64,
) -> Result<Vec<InboundOrderItemRow>, AppError>
where
    E: Executor<'e, Database = sqlx::Sqlite>,
{
    let rows = sqlx::query_as::<_, InboundOrderItemRow>(
        "SELECT id, record_id, item_id, location_id, quantity, notes, created_at
         FROM inbound_items WHERE record_id = ? ORDER BY id",
    )
    .bind(record_id)
    .fetch_all(executor)
    .await?;
    Ok(rows)
}
