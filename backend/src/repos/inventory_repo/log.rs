//! 库存日志层 — inventory_logs 事件追加与查询

use sqlx::Executor;
use sqlx::SqlitePool;
use rust_decimal::Decimal;
use crate::domain::quantity::serialize_qty_str;
use crate::error::AppError;

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

#[derive(Debug, Clone, Default)]
pub struct InventoryLogFilter {
    pub item_id: Option<i64>,
    pub location_id: Option<i64>,
    pub change_type: Option<String>,
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
