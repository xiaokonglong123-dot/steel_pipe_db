//! 库存余额层 — inventory 物化表读写 (transactional-aware)

use sqlx::{Executor, Sqlite};
use sqlx::SqlitePool;
use sqlx::Transaction;
use rust_decimal::Decimal;
use crate::domain::money::parse_amount;
use crate::domain::quantity::serialize_qty_str;
use crate::error::AppError;

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct InventoryRow {
    pub id: i64,
    pub item_id: i64,
    pub location_id: i64,
    #[serde(serialize_with = "serialize_qty_str")]
    pub quantity: String,
    pub created_at: String,
    pub updated_at: String,
}

// —— Inventory balances (transactional-aware) ——

/// 读取单条库存记录
pub async fn get_inventory(
    pool: &SqlitePool,
    item_id: i64,
    location_id: i64,
) -> Result<Option<InventoryRow>, AppError> {
    let row = sqlx::query_as::<_, InventoryRow>(
        "SELECT id, item_id, location_id, quantity, created_at, updated_at
         FROM inventory WHERE item_id = ? AND location_id = ?",
    )
    .bind(item_id)
    .bind(location_id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// 读取某 (item, location) 的库存余量，无记录返回 0
pub async fn get_balance_for_item_at_location(
    pool: &SqlitePool,
    item_id: i64,
    location_id: i64,
) -> Result<Decimal, AppError> {
    let qty: Option<String> =
        sqlx::query_scalar("SELECT quantity FROM inventory WHERE item_id = ? AND location_id = ?")
            .bind(item_id)
            .bind(location_id)
            .fetch_optional(pool)
            .await?;
    match qty {
        Some(s) => parse_amount(&s),
        None => Ok(Decimal::ZERO),
    }
}

/// 任意 location 的库存余额合计（跨库位）
/// 任意 location 的库存余额合计（跨库位）。数量为 TEXT，不在 SQL 做 SUM；改为 Rust 层累计。
pub async fn get_balance_for_item(
    pool: &SqlitePool,
    item_id: i64,
) -> Result<Decimal, AppError> {
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

/// 原子增量更新（Decimal 读改写）。quantity 为 TEXT，无法用 SQL 算术，
/// 改为在事务内读现值 → Decimal 加法 → 写回。
/// 必须传 Service 层事务以满足并发正确；服务层一律传 `&mut tx`。
pub async fn upsert_inventory_increment(
    tx: &mut Transaction<'_, Sqlite>,
    item_id: i64,
    location_id: i64,
    delta: Decimal,
) -> Result<(), AppError> {
    let cur: Option<String> =
        sqlx::query_scalar("SELECT quantity FROM inventory WHERE item_id = ? AND location_id = ?")
            .bind(item_id)
            .bind(location_id)
            .fetch_optional(&mut **tx)
            .await?;
    let new_qty = match cur {
        Some(s) => parse_amount(&s)? + delta,
        None => delta,
    };
    sqlx::query(
        "INSERT INTO inventory (item_id, location_id, quantity)
         VALUES (?, ?, ?)
         ON CONFLICT(item_id, location_id) DO UPDATE SET
             quantity = excluded.quantity,
             updated_at = datetime('now')",
    )
    .bind(item_id)
    .bind(location_id)
    .bind(new_qty.to_string())
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// 原子减量（delta 传正数，内部减去）。当余量不足时，调用方应在 service 层提前校验。
pub async fn upsert_inventory_decrement(
    tx: &mut Transaction<'_, Sqlite>,
    item_id: i64,
    location_id: i64,
    delta: Decimal,
) -> Result<(), AppError> {
    upsert_inventory_increment(tx, item_id, location_id, -delta).await
}

/// 在事务中读取余量，供 post_outbound 在提交前做库存足额校验（避免并发下超卖）。
pub async fn get_balance_for_item_at_location_tx<'e, E>(
    executor: E,
    item_id: i64,
    location_id: i64,
) -> Result<Decimal, AppError>
where
    E: Executor<'e, Database = sqlx::Sqlite>,
{
    let qty: Option<String> =
        sqlx::query_scalar("SELECT quantity FROM inventory WHERE item_id = ? AND location_id = ?")
            .bind(item_id)
            .bind(location_id)
            .fetch_optional(executor)
            .await?;
    match qty {
        Some(s) => parse_amount(&s),
        None => Ok(Decimal::ZERO),
    }
}
