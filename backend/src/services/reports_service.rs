//! Reports service — 应用层 Decimal 聚合 + CSV 序列化

use rust_decimal::Decimal;
use std::str::FromStr;

use crate::error::AppError;
use crate::repos::reports_repo::{
    self, FinanceSummaryRow, InboundOutboundRow, InventorySummaryRow, SalesTrendRow,
};
use sqlx::SqlitePool;

pub async fn inventory_summary(
    pool: &SqlitePool,
) -> Result<Vec<InventorySummaryRow>, AppError> {
    reports_repo::inventory_summary(pool).await
}

pub async fn inbound_outbound(
    pool: &SqlitePool,
    item_id: Option<i64>,
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> Result<Vec<InboundOutboundRow>, AppError> {
    reports_repo::inbound_outbound(pool, item_id, start_date, end_date).await
}

pub async fn sales_trend(
    pool: &SqlitePool,
    months: i64,
) -> Result<Vec<SalesTrendRow>, AppError> {
    let raw = reports_repo::sales_trend_raw(pool, months).await?;
    // 应用层 Decimal 累计总金额（ADR-002）
    let rows = raw
        .into_iter()
        .map(|(month, count, items)| {
            let total: Decimal = items
                .iter()
                .filter_map(|(_, amt)| Decimal::from_str(amt).ok())
                .sum();
            SalesTrendRow {
                month,
                order_count: count,
                total_amount: total.to_string(),
            }
        })
        .collect();
    Ok(rows)
}

pub async fn finance_summary(
    pool: &SqlitePool,
) -> Result<Vec<FinanceSummaryRow>, AppError> {
    reports_repo::finance_summary_raw(pool).await
}

// —— CSV 序列化 ——

pub fn inventory_summary_csv(rows: &[InventorySummaryRow]) -> String {
    let mut s = String::from("item_id,sku,name,category,total_qty,location_count\n");
    for r in rows {
        let cat = r.category.clone().unwrap_or_default();
        s.push_str(&format!(
            "{},{},{},{},{},{}\n",
            r.item_id, escape(&r.sku), escape(&r.name), escape(&cat),
            r.total_qty, r.location_count
        ));
    }
    s
}

pub fn inbound_outbound_csv(rows: &[InboundOutboundRow]) -> String {
    let mut s = String::from(
        "log_id,change_type,item_id,sku,name,quantity,location_id,ref_type,ref_id,created_at\n",
    );
    for r in rows {
        s.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{}\n",
            r.log_id, r.change_type, r.item_id,
            escape(&r.sku), escape(&r.name), r.quantity, r.location_id,
            r.ref_type.clone().unwrap_or_default(),
            r.ref_id.unwrap_or(0),
            r.created_at
        ));
    }
    s
}

pub fn sales_trend_csv(rows: &[SalesTrendRow]) -> String {
    let mut s = String::from("month,order_count,total_amount\n");
    for r in rows {
        s.push_str(&format!(
            "{},{},{}\n",
            r.month, r.order_count, r.total_amount
        ));
    }
    s
}

pub fn finance_summary_csv(rows: &[FinanceSummaryRow]) -> String {
    let mut s = String::from("account_id,account_code,account_name,account_type,total_debit,total_credit\n");
    for r in rows {
        s.push_str(&format!(
            "{},{},{},{},{},{}\n",
            r.account_id,
            escape(&r.account_code),
            escape(&r.account_name),
            r.account_type,
            r.total_debit,
            r.total_credit
        ));
    }
    s
}

fn escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

#[allow(dead_code)]
fn _unused() -> Result<(), AppError> {
    Err(AppError::validation("unused"))
}
