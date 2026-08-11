//! Reports repo — 报表查询。
//!
//! ADR-002 注意：所有金额汇总**不在 SQL 做 SUM on TEXT**。
//! 这里返回 raw rows，service 层用 Decimal 累计。
//! GROUP BY 仅用于分组维度（item.category、strftime('%Y-%m')），SUM 限于非金额的整数量
//! （如 record_count、journal_line_count）。

use serde::Serialize;
use sqlx::{sqlite::SqliteRow, Row, SqlitePool};

use crate::error::{AppError, ErrorCode};

// —— 行类型 ——

#[derive(Debug, Clone, Serialize)]
pub struct InventorySummaryRow {
    pub item_id: i64,
    pub sku: String,
    pub name: String,
    pub category: Option<String>,
    pub total_qty: f64,
    pub location_count: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct InboundOutboundRow {
    pub log_id: i64,
    pub change_type: String,
    pub item_id: i64,
    pub sku: String,
    pub name: String,
    pub quantity: f64,
    pub location_id: i64,
    pub ref_type: Option<String>,
    pub ref_id: Option<i64>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SalesTrendRow {
    pub month: String,
    pub order_count: i64,
    /// 总金额（应用层 Decimal 累计得到的字符串）
    pub total_amount: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FinanceSummaryRow {
    pub account_id: i64,
    pub account_code: String,
    pub account_name: String,
    pub account_type: String,
    pub total_debit: String,
    pub total_credit: String,
}

// —— 查询 ——

pub async fn inventory_summary(pool: &SqlitePool) -> Result<Vec<InventorySummaryRow>, AppError> {
    let rows = sqlx::query(
        "SELECT i.id, i.sku, i.name, i.category,
                COALESCE(SUM(inv.quantity), 0.0) AS total_qty,
                COUNT(DISTINCT inv.location_id) AS location_count
         FROM items i
         LEFT JOIN inventory inv ON inv.item_id = i.id
         WHERE i.deleted_at IS NULL
         GROUP BY i.id, i.sku, i.name, i.category
         ORDER BY i.sku",
    )
    .try_map(|row: SqliteRow| {
        Ok(InventorySummaryRow {
            item_id: row.try_get("id")?,
            sku: row.try_get("sku")?,
            name: row.try_get("name")?,
            category: row.try_get("category")?,
            total_qty: row.try_get::<f64, _>("total_qty")?,
            location_count: row.try_get("location_count")?,
        })
    })
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn inbound_outbound(
    pool: &SqlitePool,
    item_id: Option<i64>,
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> Result<Vec<InboundOutboundRow>, AppError> {
    let mut sql = String::from(
        "SELECT il.id AS log_id, il.change_type, il.item_id, i.sku, i.name,
                il.quantity, il.location_id, il.ref_type, il.ref_id, il.created_at
         FROM inventory_logs il
         JOIN items i ON i.id = il.item_id
         WHERE 1=1",
    );
    let binds: Vec<String> = Vec::new();
    if item_id.is_some() {
        sql.push_str(" AND il.item_id = ?");
    }
    if start_date.is_some() {
        sql.push_str(" AND il.created_at >= ?");
    }
    if end_date.is_some() {
        sql.push_str(" AND il.created_at <= ?");
    }
    sql.push_str(" ORDER BY il.created_at DESC LIMIT 1000");

    let mut q = sqlx::query(&sql);
    if let Some(iid) = item_id {
        q = q.bind(iid);
    }
    if let Some(s) = start_date {
        q = q.bind(s);
    }
    if let Some(e) = end_date {
        q = q.bind(e);
    }
    let _ = binds;

    let rows = q
        .try_map(|row: SqliteRow| {
            Ok(InboundOutboundRow {
                log_id: row.try_get("log_id")?,
                change_type: row.try_get("change_type")?,
                item_id: row.try_get("item_id")?,
                sku: row.try_get("sku")?,
                name: row.try_get("name")?,
                quantity: row.try_get("quantity")?,
                location_id: row.try_get("location_id")?,
                ref_type: row.try_get("ref_type")?,
                ref_id: row.try_get("ref_id")?,
                created_at: row.try_get("created_at")?,
            })
        })
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

/// 按月分组销售订单 — 返回 (month, order_count, [order_ids + amount_text]) 给 service 层做 Decimal 累计
pub async fn sales_trend_raw(
    pool: &SqlitePool,
    months: i64,
) -> Result<Vec<(String, i64, Vec<(i64, String)>)>, AppError> {
    let rows = sqlx::query(
        "SELECT strftime('%Y-%m', so.order_date) AS month,
                COUNT(so.id) AS order_count,
                so.id AS order_id,
                so.total_amount
         FROM sales_orders so
         WHERE so.order_date >= date('now', ?)
           AND so.deleted_at IS NULL
         ORDER BY month DESC, so.id",
    )
    .bind(format!("-{months} months"))
    .try_map(|row: SqliteRow| {
        Ok((
            row.try_get::<String, _>("month")?,
            row.try_get::<i64, _>("order_count")?,
            row.try_get::<i64, _>("order_id")?,
            row.try_get::<String, _>("total_amount")?,
        ))
    })
    .fetch_all(pool)
    .await?;

    // 按 month 聚合
    use std::collections::BTreeMap;
    let mut groups: BTreeMap<String, (i64, Vec<(i64, String)>)> = BTreeMap::new();
    for (month, count, order_id, amount) in rows {
        let entry = groups.entry(month).or_insert((0, Vec::new()));
        entry.0 += count;
        entry.1.push((order_id, amount));
    }
    // reverse to descending
    let mut result: Vec<(String, i64, Vec<(i64, String)>)> =
        groups.into_iter().map(|(m, (c, items))| (m, c, items)).collect();
    result.sort_by(|a, b| b.0.cmp(&a.0));
    Ok(result)
}

/// 财务汇总 — 返回 finance accounts + 每个账户所有 posted journal entry lines 的 raw 借贷
pub async fn finance_summary_raw(
    pool: &SqlitePool,
) -> Result<Vec<FinanceSummaryRow>, AppError> {
    // 先取所有 accounts + posted lines
    let accounts = sqlx::query("SELECT id, code, name, account_type FROM accounts ORDER BY code")
        .try_map(|row: SqliteRow| {
            Ok((
                row.try_get::<i64, _>("id")?,
                row.try_get::<String, _>("code")?,
                row.try_get::<String, _>("name")?,
                row.try_get::<String, _>("account_type")?,
            ))
        })
        .fetch_all(pool)
        .await?;

    // 取 posted journal_entry_lines
    let lines = sqlx::query(
        "SELECT jel.account_id, jel.debit, jel.credit
         FROM journal_entry_lines jel
         JOIN journal_entries je ON je.id = jel.entry_id
         WHERE je.status = 'posted'",
    )
    .try_map(|row: SqliteRow| {
        Ok((
            row.try_get::<i64, _>("account_id")?,
            row.try_get::<String, _>("debit")?,
            row.try_get::<String, _>("credit")?,
        ))
    })
    .fetch_all(pool)
    .await?;

    // 应用层聚合（per ADR-002 — NOT SQL SUM on TEXT）
    use rust_decimal::Decimal;
    use std::collections::BTreeMap;
    use std::str::FromStr;
    let mut totals: BTreeMap<i64, (Decimal, Decimal)> = BTreeMap::new();
    for (account_id, debit_s, credit_s) in lines {
        let d = Decimal::from_str(&debit_s).unwrap_or(Decimal::ZERO);
        let c = Decimal::from_str(&credit_s).unwrap_or(Decimal::ZERO);
        let entry = totals.entry(account_id).or_insert((Decimal::ZERO, Decimal::ZERO));
        entry.0 += d;
        entry.1 += c;
    }

    let rows = accounts
        .into_iter()
        .map(|(id, code, name, account_type)| {
            let (debit_total, credit_total) = totals.get(&id).copied().unwrap_or((Decimal::ZERO, Decimal::ZERO));
            FinanceSummaryRow {
                account_id: id,
                account_code: code,
                account_name: name,
                account_type,
                total_debit: debit_total.to_string(),
                total_credit: credit_total.to_string(),
            }
        })
        .collect();
    Ok(rows)
}

#[allow(dead_code)]
fn _unused() -> Result<(), AppError> {
    Err(AppError::new(ErrorCode::Internal, "unused"))
}
