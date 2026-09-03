//! 库存查询 — 库存列表视图

use sqlx::SqlitePool;
use crate::domain::quantity::serialize_qty_str;
use crate::error::AppError;

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
