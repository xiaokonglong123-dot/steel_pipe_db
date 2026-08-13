//! Inventory 数据访问 — inventory / inventory_logs / inbound_records / inbound_items /
//! outbound_records / outbound_items 表（004_inventory.sql）。
//!
//! 纯 SQL（sqlx），无业务逻辑。事务控制由 service 层 `pool.begin()` 负责；
//! 本 repo 中需要参与事务的函数对 `sqlx::Executor` 泛型化，可接收 `&SqlitePool` 或
//! `&mut sqlx::Transaction`。其余纯读函数只接 `&SqlitePool`。

use sqlx::{Executor, Sqlite, SqlitePool, Transaction};
use rust_decimal::Decimal;

use crate::domain::money::parse_amount;
use crate::domain::quantity::serialize_qty_str;
use crate::error::{AppError, ErrorCode};

// —— Row structs ——

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

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct InventoryLogRow {
    pub id: i64,
    pub item_id: i64,
    pub location_id: Option<i64>,
    pub change_type: String,
    #[serde(serialize_with = "serialize_qty_str")]
    pub quantity: String,
    pub ref_type: Option<String>,
    pub ref_id: Option<i64>,
    pub notes: Option<String>,
    pub created_by: Option<i64>,
    pub created_at: String,
}

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
pub struct InboundOrderFilter {
    pub status: Option<String>,
    pub inbound_type: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct OutboundOrderFilter {
    pub status: Option<String>,
    pub outbound_type: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct InventoryLogFilter {
    pub item_id: Option<i64>,
    pub location_id: Option<i64>,
    pub change_type: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct StockFilter {
    pub item_id: Option<i64>,
    pub location_id: Option<i64>,
    pub warehouse_id: Option<i64>,
}

/// 聚合后的库存行（用于 /stock 列表展示）。聚合在 Rust 层完成，避免对 quantity 做 SQL SUM。
#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct StockRow {
    pub item_id: i64,
    pub location_id: Option<i64>,
    pub warehouse_id: Option<i64>,
    #[serde(serialize_with = "serialize_qty_str")]
    pub quantity: String,
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

// —— Logs ——

/// 插入一条 inventory_log。`balance_after` 由调用方在 service 层根据刚发生的增量算出后传入，
/// 写入 notes 字段（schema 无独立列；存为 JSON 片段 `balance_after=<n>`）。
pub async fn insert_log<'e, E>(
    executor: E,
    item_id: i64,
    location_id: Option<i64>,
    change_type: &str,
    quantity: Decimal,
    ref_type: Option<&str>,
    ref_id: Option<i64>,
    notes: Option<&str>,
    created_by: Option<i64>,
) -> Result<i64, AppError>
where
    E: Executor<'e, Database = sqlx::Sqlite>,
{
    let result = sqlx::query(
        "INSERT INTO inventory_logs
            (item_id, location_id, change_type, quantity, ref_type, ref_id, notes, created_by)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(item_id)
    .bind(location_id)
    .bind(change_type)
    .bind(quantity.to_string())
    .bind(ref_type)
    .bind(ref_id)
    .bind(notes)
    .bind(created_by)
    .execute(executor)
    .await?;
    Ok(result.last_insert_rowid())
}

pub async fn list_logs(
    pool: &SqlitePool,
    filter: &InventoryLogFilter,
    page: i64,
    page_size: i64,
) -> Result<(Vec<InventoryLogRow>, i64), AppError> {
    let mut where_clauses: Vec<&'static str> = Vec::new();
    let mut count_sql = String::from("SELECT COUNT(*) FROM inventory_logs");
    let mut list_sql = String::from(
        "SELECT id, item_id, location_id, change_type, quantity, ref_type, ref_id,
                notes, created_by, created_at
         FROM inventory_logs",
    );

    if filter.item_id.is_some() {
        where_clauses.push("item_id = ?");
    }
    if filter.location_id.is_some() {
        where_clauses.push("location_id = ?");
    }
    if filter.change_type.is_some() {
        where_clauses.push("change_type = ?");
    }

    if !where_clauses.is_empty() {
        let extra = where_clauses.join(" AND ");
        count_sql.push_str(" WHERE ");
        count_sql.push_str(&extra);
        list_sql.push_str(" WHERE ");
        list_sql.push_str(&extra);
    }

    list_sql.push_str(" ORDER BY id DESC LIMIT ? OFFSET ?");

    let mut count_q = sqlx::query_scalar::<_, i64>(&count_sql);
    if let Some(v) = filter.item_id {
        count_q = count_q.bind(v);
    }
    if let Some(v) = filter.location_id {
        count_q = count_q.bind(v);
    }
    if let Some(v) = &filter.change_type {
        count_q = count_q.bind(v);
    }
    let total: i64 = count_q.fetch_one(pool).await?;

    let offset = (page - 1).max(0) * page_size;
    let mut list_q = sqlx::query_as::<_, InventoryLogRow>(&list_sql);
    if let Some(v) = filter.item_id {
        list_q = list_q.bind(v);
    }
    if let Some(v) = filter.location_id {
        list_q = list_q.bind(v);
    }
    if let Some(v) = &filter.change_type {
        list_q = list_q.bind(v);
    }
    list_q = list_q.bind(page_size).bind(offset);
    let rows = list_q.fetch_all(pool).await?;
    Ok((rows, total))
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

// —— Stock listing ——

/// 列出库存行，可选过滤 item_id / location_id / warehouse_id（warehouse_id 到 locations 表 JOIN）。
/// 不对 quantity 做 SQL SUM；分页在 inventory 表行级完成；item 维度聚合由调用方按需在 Rust 层做。
pub async fn list_stock(
    pool: &SqlitePool,
    filter: &StockFilter,
    page: i64,
    page_size: i64,
) -> Result<(Vec<StockRow>, i64), AppError> {
    let mut where_clauses: Vec<&'static str> = Vec::new();
    let mut count_sql = String::from(
        "SELECT COUNT(*) FROM inventory inv
         LEFT JOIN locations loc ON loc.id = inv.location_id AND loc.deleted_at IS NULL",
    );
    let mut list_sql = String::from(
        "SELECT inv.item_id, inv.location_id, loc.warehouse_id AS warehouse_id, inv.quantity
         FROM inventory inv
         LEFT JOIN locations loc ON loc.id = inv.location_id AND loc.deleted_at IS NULL",
    );

    if filter.item_id.is_some() {
        where_clauses.push("inv.item_id = ?");
    }
    if filter.location_id.is_some() {
        where_clauses.push("inv.location_id = ?");
    }
    if filter.warehouse_id.is_some() {
        where_clauses.push("loc.warehouse_id = ?");
    }

    if !where_clauses.is_empty() {
        let extra = where_clauses.join(" AND ");
        count_sql.push_str(" WHERE ");
        count_sql.push_str(&extra);
        list_sql.push_str(" WHERE ");
        list_sql.push_str(&extra);
    }

    list_sql.push_str(" ORDER BY inv.item_id DESC, inv.location_id ASC LIMIT ? OFFSET ?");

    let mut count_q = sqlx::query_scalar::<_, i64>(&count_sql);
    if let Some(v) = filter.item_id {
        count_q = count_q.bind(v);
    }
    if let Some(v) = filter.location_id {
        count_q = count_q.bind(v);
    }
    if let Some(v) = filter.warehouse_id {
        count_q = count_q.bind(v);
    }
    let total: i64 = count_q.fetch_one(pool).await?;

    let offset = (page - 1).max(0) * page_size;
    let mut list_q = sqlx::query_as::<_, StockRow>(&list_sql);
    if let Some(v) = filter.item_id {
        list_q = list_q.bind(v);
    }
    if let Some(v) = filter.location_id {
        list_q = list_q.bind(v);
    }
    if let Some(v) = filter.warehouse_id {
        list_q = list_q.bind(v);
    }
    list_q = list_q.bind(page_size).bind(offset);
    let rows = list_q.fetch_all(pool).await?;
    Ok((rows, total))
}
