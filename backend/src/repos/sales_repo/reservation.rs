//! sales_repo.reservation — 预留 reservations (ATP)

use sqlx::Executor;
use sqlx::SqlitePool;
use rust_decimal::Decimal;
use crate::domain::money::parse_amount;
use crate::domain::quantity::serialize_qty_str;
use crate::error::{AppError, ErrorCode};


#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct ReservationRow {
    pub id: i64,
    pub item_id: i64,
    #[serde(serialize_with = "serialize_qty_str")]
    pub quantity: String,
    pub order_type: String,
    pub order_id: i64,
    pub status: String,
    pub created_by: Option<i64>,
    pub created_at: String,
    pub released_at: Option<String>,
}


// —— Reservations (ATP) ——

/// 创建预留行（status='active'）。泛型化以支持事务。返回新行 id。
pub async fn insert_reservation<'e, E>(
    executor: E,
    item_id: i64,
    quantity: Decimal,
    order_id: i64,
    created_by: Option<i64>,
) -> Result<i64, AppError>
where
    E: Executor<'e, Database = sqlx::Sqlite>,
{
    let result = sqlx::query(
        "INSERT INTO reservations (item_id, quantity, order_type, order_id, status, created_by)
         VALUES (?, ?, 'sales', ?, 'active', ?)",
    )
    .bind(item_id)
    .bind(quantity.to_string())
    .bind(order_id)
    .bind(created_by)
    .execute(executor)
    .await?;
    Ok(result.last_insert_rowid())
}


/// 释放预留（status='released', released_at=now）。泛型化以支持事务。
pub async fn release_reservation_tx<'e, E>(executor: E, reservation_id: i64) -> Result<(), AppError>
where
    E: Executor<'e, Database = sqlx::Sqlite>,
{
    let result = sqlx::query(
        "UPDATE reservations SET status = 'released', released_at = datetime('now')
         WHERE id = ? AND status = 'active'",
    )
    .bind(reservation_id)
    .execute(executor)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::new(ErrorCode::NotFound, "预留记录未找到或已释放"));
    }
    Ok(())
}


pub async fn cancel_reservation(pool: &SqlitePool, reservation_id: i64) -> Result<(), AppError> {
    let result = sqlx::query(
        "UPDATE reservations SET status = 'cancelled'
         WHERE id = ? AND status = 'active'",
    )
    .bind(reservation_id)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::new(ErrorCode::NotFound, "预留记录未找到或已取消"));
    }
    Ok(())
}


/// 释放某销售订单的所有 active 预留（事务版）。用于 cancel 订单时回收预留。
pub async fn release_reservations_for_order_tx<'e, E>(
    executor: E,
    order_id: i64,
) -> Result<i64, AppError>
where
    E: Executor<'e, Database = sqlx::Sqlite>,
{
    let result = sqlx::query(
        "UPDATE reservations SET status = 'released', released_at = datetime('now')
         WHERE order_id = ? AND order_type = 'sales' AND status = 'active'",
    )
    .bind(order_id)
    .execute(executor)
    .await?;
    Ok(result.rows_affected() as i64)
}


pub async fn list_active_reservations_for_item(
    pool: &SqlitePool,
    item_id: i64,
) -> Result<Vec<ReservationRow>, AppError> {
    let rows = sqlx::query_as::<_, ReservationRow>(
        "SELECT id, item_id, quantity, order_type, order_id, status, created_by,
                created_at, released_at
         FROM reservations WHERE item_id = ? AND status = 'active'
         ORDER BY id",
    )
    .bind(item_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}


/// 汇总某 item 的 active 预留总量（TEXT，Rust 层 Decimal 累计，不做 SQL SUM on TEXT）
pub async fn sum_active_reservations_for_item(
    pool: &SqlitePool,
    item_id: i64,
) -> Result<Decimal, AppError> {
    let rows = sqlx::query_scalar::<_, String>(
        "SELECT quantity FROM reservations
         WHERE item_id = ? AND status = 'active'",
    )
    .bind(item_id)
    .fetch_all(pool)
    .await?;
    let mut acc = Decimal::ZERO;
    for r in rows {
        acc += parse_amount(&r)?;
    }
    Ok(acc)
}


/// 列出某销售订单的所有 active 预留（事务版）
pub async fn list_active_reservations_for_order_tx<'e, E>(
    executor: E,
    order_id: i64,
) -> Result<Vec<ReservationRow>, AppError>
where
    E: Executor<'e, Database = sqlx::Sqlite>,
{
    let rows = sqlx::query_as::<_, ReservationRow>(
        "SELECT id, item_id, quantity, order_type, order_id, status, created_by,
                created_at, released_at
         FROM reservations WHERE order_id = ? AND order_type = 'sales' AND status = 'active'
         ORDER BY id",
    )
    .bind(order_id)
    .fetch_all(executor)
    .await?;
    Ok(rows)
}


/// 汇总某商品的 active 预留数量（TEXT，Rust 层 Decimal 累计，不做 SQL SUM on TEXT）。
/// 不含当前正在提交的订单。
pub async fn sum_active_reserved_quantity_for_item(
    pool: &SqlitePool,
    item_id: i64,
) -> Result<Decimal, AppError> {
    let rows = sqlx::query_scalar::<_, String>(
        "SELECT quantity FROM reservations
         WHERE item_id = ? AND status = 'active'",
    )
    .bind(item_id)
    .fetch_all(pool)
    .await?;
    let mut acc = Decimal::ZERO;
    for r in rows {
        acc += parse_amount(&r)?;
    }
    Ok(acc)
}


// —— Inventory balance helpers (for ATP) ——

/// 汇总某商品在所有库位的库存余额（TEXT，Rust 层 Decimal 累计，不做 SQL SUM on TEXT）。
pub async fn sum_balance_for_item(pool: &SqlitePool, item_id: i64) -> Result<Decimal, AppError> {
    let rows = sqlx::query_scalar::<_, String>(
        "SELECT quantity FROM inventory WHERE item_id = ?",
    )
    .bind(item_id)
    .fetch_all(pool)
    .await?;
    let mut acc = Decimal::ZERO;
    for r in rows {
        acc += parse_amount(&r)?;
    }
    Ok(acc)
}
