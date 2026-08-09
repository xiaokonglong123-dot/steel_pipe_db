use chrono::{DateTime, Utc};
use sqlx::SqlitePool;

use crate::error::AppError;

/// Aggregation queries for dashboard and reports. Returns `serde_json::Value`.
///
/// All reports are item-based for the generic ERP: inventory reports aggregate
/// the `items` master + `inventory_logs` movements; quality reports aggregate
/// the manufacturing inspections/NCRs (kept from the pipe era as generic QC).
pub struct ReportRepo;

impl ReportRepo {
    /// Count of items by `status`, plus the total item count.
    pub async fn inventory_by_status(
        pool: &SqlitePool,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let rows: Vec<(String, i64)> = sqlx::query_as(
            "SELECT status, COUNT(*) as cnt FROM items \
             WHERE deleted_at IS NULL GROUP BY status ORDER BY cnt DESC",
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        let total: i64 = rows.iter().map(|(_, cnt)| cnt).sum();

        let mut result: Vec<serde_json::Value> = Vec::new();
        result.insert(0, serde_json::json!({"status": "total", "count": total}));
        for (status, cnt) in rows {
            result.push(serde_json::json!({"status": status, "count": cnt}));
        }

        Ok(result)
    }

    /// Count of items grouped by category.
    pub async fn inventory_by_category(
        pool: &SqlitePool,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let rows: Vec<(Option<String>, i64)> = sqlx::query_as(
            "SELECT category, COUNT(*) as cnt FROM items \
             WHERE deleted_at IS NULL GROUP BY category ORDER BY cnt DESC",
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        Ok(rows
            .into_iter()
            .map(|(category, cnt)| {
                serde_json::json!({"category": category.unwrap_or_else(|| "uncategorized".into()), "count": cnt})
            })
            .collect())
    }

    /// Stock movement totals by `change_type` from `inventory_logs` (inbound/outbound/transfer/check_adjust).
    pub async fn inventory_by_type(pool: &SqlitePool) -> Result<Vec<serde_json::Value>, AppError> {
        let rows: Vec<(String, f64)> = sqlx::query_as(
            "SELECT change_type, CAST(COALESCE(SUM(quantity), 0.0) AS REAL) as total_quantity \
             FROM inventory_logs \
             GROUP BY change_type ORDER BY total_quantity DESC",
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        Ok(rows
            .into_iter()
            .map(|(change_type, total)| {
                serde_json::json!({"change_type": change_type, "total_quantity": total})
            })
            .collect())
    }

    /// Location occupancy stats — full_code, max_capacity, current_usage, available, occupancy_pct.
    pub async fn location_occupancy(pool: &SqlitePool) -> Result<Vec<serde_json::Value>, AppError> {
        let rows: Vec<(String, i64, i64, i64, i64)> = sqlx::query_as(
            "SELECT l.full_code, COALESCE(l.capacity, 0) as capacity, l.used_count, \
             (COALESCE(l.capacity, 0) - l.used_count) as available, \
             CASE WHEN l.capacity > 0 THEN \
             CAST(ROUND(l.used_count * 100.0 / l.capacity) AS INTEGER) \
             ELSE 0 END as occupancy_pct \
             FROM locations l              WHERE l.is_active = 1 AND l.deleted_at IS NULL ORDER BY l.full_code",
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        Ok(rows
            .into_iter()
            .map(|(code, cap, usage, avail, pct)| {
                serde_json::json!({
                    "location": code,
                    "max_capacity": cap,
                    "current_usage": usage,
                    "available": avail,
                    "occupancy_pct": pct.to_string(),
                })
            })
            .collect())
    }

    fn period_group_expr(date_trunc: &str) -> String {
        match date_trunc {
            "monthly" => "strftime('%Y-%m', order_date)".to_string(),
            "quarterly" => {
                "strftime('%Y', order_date) || '-Q' || \
                 CAST(((CAST(strftime('%m', order_date) AS INTEGER) + 2) / 3) AS TEXT)"
                    .to_string()
            }
            "yearly" => "strftime('%Y', order_date)".to_string(),
            _ => "strftime('%Y-%m', order_date)".to_string(),
        }
    }

    /// Aggregated purchase orders by period (monthly/quarterly/yearly). Returns order_count and total_amount.
    pub async fn purchase_order_report(
        pool: &SqlitePool,
        date_trunc: &str,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let group_expr = Self::period_group_expr(date_trunc);

        let sql = format!(
            "SELECT {} as period, COUNT(*) as order_count, \
             CAST(COALESCE(SUM(total_amount), 0.0) AS REAL) as total_amount \
             FROM purchase_orders WHERE deleted_at IS NULL \
             GROUP BY period ORDER BY period DESC LIMIT 24",
            group_expr
        );

        let rows: Vec<(String, i64, f64)> = sqlx::query_as(&sql)
            .fetch_all(pool)
            .await
            .map_err(AppError::from)?;

        Ok(rows
            .into_iter()
            .map(|(period, cnt, amount)| {
                serde_json::json!({
                    "period": period,
                    "order_count": cnt,
                    "total_amount": amount,
                })
            })
            .collect())
    }

    /// Aggregated sales orders by period (monthly/quarterly/yearly). Returns order_count and total_amount.
    pub async fn sales_order_report(
        pool: &SqlitePool,
        date_trunc: &str,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let group_expr = Self::period_group_expr(date_trunc);

        let sql = format!(
            "SELECT {} as period, COUNT(*) as order_count, \
             CAST(COALESCE(SUM(total_amount), 0.0) AS REAL) as total_amount \
             FROM sales_orders WHERE deleted_at IS NULL \
             GROUP BY period ORDER BY period DESC LIMIT 24",
            group_expr
        );

        let rows: Vec<(String, i64, f64)> = sqlx::query_as(&sql)
            .fetch_all(pool)
            .await
            .map_err(AppError::from)?;

        Ok(rows
            .into_iter()
            .map(|(period, cnt, amount)| {
                serde_json::json!({
                    "period": period,
                    "order_count": cnt,
                    "total_amount": amount,
                })
            })
            .collect())
    }

    /// Count of purchase/sales orders grouped by status.
    pub async fn order_status_distribution(
        pool: &SqlitePool,
        table: &str,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        // Whitelist table names to prevent SQL injection
        let allowed_tables = ["purchase_orders", "sales_orders"];
        if !allowed_tables.contains(&table) {
            return Err(AppError::BadRequest(format!("Invalid table: {}", table)));
        }
        let sql = format!(
            "SELECT status, COUNT(*) as cnt FROM {} \
             WHERE deleted_at IS NULL GROUP BY status ORDER BY cnt DESC",
            table
        );

        let rows: Vec<(String, i64)> = sqlx::query_as(&sql)
            .fetch_all(pool)
            .await
            .map_err(AppError::from)?;

        Ok(rows
            .into_iter()
            .map(|(status, cnt)| serde_json::json!({"status": status, "count": cnt}))
            .collect())
    }

    /// Top N suppliers by total purchase amount.
    pub async fn top_suppliers(
        pool: &SqlitePool,
        limit: i64,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let rows: Vec<(String, i64, f64)> = sqlx::query_as(
            "SELECT s.name, COUNT(*) as order_count, CAST(COALESCE(SUM(po.total_amount), 0.0) AS REAL) as total_amount \
             FROM purchase_orders po JOIN suppliers s ON po.supplier_id = s.id \
             WHERE po.deleted_at IS NULL AND s.deleted_at IS NULL \
             GROUP BY s.id, s.name ORDER BY total_amount DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        Ok(rows
            .into_iter()
            .map(|(name, cnt, amount)| {
                serde_json::json!({
                    "supplier": name,
                    "order_count": cnt,
                    "total_amount": amount,
                })
            })
            .collect())
    }

    /// Top N customers by total sales amount.
    pub async fn top_customers(
        pool: &SqlitePool,
        limit: i64,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let rows: Vec<(String, i64, f64)> = sqlx::query_as(
            "SELECT c.name, COUNT(*) as order_count, CAST(COALESCE(SUM(so.total_amount), 0.0) AS REAL) as total_amount \
             FROM sales_orders so JOIN customers c ON so.customer_id = c.id \
             WHERE so.deleted_at IS NULL AND c.deleted_at IS NULL \
             GROUP BY c.id, c.name ORDER BY total_amount DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        Ok(rows
            .into_iter()
            .map(|(name, cnt, amount)| {
                serde_json::json!({
                    "customer": name,
                    "order_count": cnt,
                    "total_amount": amount,
                })
            })
            .collect())
    }

    /// Pass/fail counts by item SKU from manufacturing inspections. Includes pass_rate percentage.
    pub async fn quality_pass_fail_by_grade(
        pool: &SqlitePool,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let rows: Vec<(Option<String>, i64, i64)> = sqlx::query_as(
            "SELECT i.sku, \
             SUM(CASE WHEN mi.result = 'pass' THEN 1 ELSE 0 END) as pass_count, \
             SUM(CASE WHEN mi.result = 'fail' THEN 1 ELSE 0 END) as fail_count \
             FROM mfg_inspections mi LEFT JOIN items i ON i.id = mi.item_id \
             GROUP BY i.sku \
             ORDER BY pass_count DESC",
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        Ok(rows
            .into_iter()
            .map(|(sku, pass, fail)| {
                let total = pass + fail;
                let pass_rate = if total > 0 {
                    format!("{:.1}%", pass as f64 * 100.0 / total as f64)
                } else {
                    "N/A".into()
                };
                serde_json::json!({
                    "sku": sku.unwrap_or_else(|| "unknown".into()),
                    "pass_count": pass,
                    "fail_count": fail,
                    "total": total,
                    "pass_rate": pass_rate,
                })
            })
            .collect())
    }

    /// Inspections grouped by month (last 12). Returns total, passed, failed, pass_rate.
    pub async fn inspections_by_month(
        pool: &SqlitePool,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let rows: Vec<(String, i64, i64, i64)> = sqlx::query_as(
            "SELECT strftime('%Y-%m', inspected_at) as month, \
             COUNT(*) as total, \
             SUM(CASE WHEN result = 'pass' THEN 1 ELSE 0 END) as passed, \
             SUM(CASE WHEN result = 'fail' THEN 1 ELSE 0 END) as failed \
             FROM mfg_inspections WHERE inspected_at IS NOT NULL \
             GROUP BY month ORDER BY month DESC LIMIT 12",
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        Ok(rows
            .into_iter()
            .map(|(month, total, passed, failed)| {
                let pass_rate = if total > 0 {
                    format!("{:.1}%", passed as f64 * 100.0 / total as f64)
                } else {
                    "0%".into()
                };
                serde_json::json!({
                    "month": month,
                    "total": total,
                    "passed": passed,
                    "failed": failed,
                    "pass_rate": pass_rate,
                })
            })
            .collect())
    }

    /// Total active items in the item master.
    pub async fn total_stock(pool: &SqlitePool) -> Result<i64, AppError> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM items WHERE deleted_at IS NULL AND status = 'active'",
        )
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;
        Ok(row.0)
    }

    /// Recent inbound records within N days. Returns record_no, type, status, created_at.
    pub async fn recent_inbound(
        pool: &SqlitePool,
        days: i64,
        limit: i64,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let rows: Vec<(String, String, String, String)> = sqlx::query_as(
            "SELECT ir.inbound_no, ir.inbound_type, ir.approval_status, ir.created_at \
             FROM inbound_records ir \
             WHERE ir.created_at >= datetime('now', ?) AND ir.deleted_at IS NULL \
             ORDER BY ir.created_at DESC LIMIT ?",
        )
        .bind(format!("-{} days", days))
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        Ok(rows
            .into_iter()
            .map(|(no, ty, status, at)| {
                serde_json::json!({
                    "record_no": no,
                    "type": ty,
                    "approval_status": status,
                    "created_at": at,
                })
            })
            .collect())
    }

    /// Recent outbound records within N days. Returns record_no, type, status, created_at.
    pub async fn recent_outbound(
        pool: &SqlitePool,
        days: i64,
        limit: i64,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let rows: Vec<(String, String, String, String)> = sqlx::query_as(
            "SELECT orr.outbound_no, orr.outbound_type, orr.approval_status, orr.created_at \
             FROM outbound_records orr \
             WHERE orr.created_at >= datetime('now', ?) AND orr.deleted_at IS NULL \
             ORDER BY orr.created_at DESC LIMIT ?",
        )
        .bind(format!("-{} days", days))
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        Ok(rows
            .into_iter()
            .map(|(no, ty, status, at)| {
                serde_json::json!({
                    "record_no": no,
                    "type": ty,
                    "approval_status": status,
                    "created_at": at,
                })
            })
            .collect())
    }

    /// Count of inbound records in the last 30 days.
    pub async fn inbound_count_30d(pool: &SqlitePool) -> Result<i64, AppError> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM inbound_records \
             WHERE created_at >= datetime('now', '-30 days') AND deleted_at IS NULL",
        )
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;
        Ok(row.0)
    }

    /// Count of outbound records in the last 30 days.
    pub async fn outbound_count_30d(pool: &SqlitePool) -> Result<i64, AppError> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM outbound_records \
             WHERE created_at >= datetime('now', '-30 days') AND deleted_at IS NULL",
        )
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;
        Ok(row.0)
    }

    /// Pending inbound/outbound records and pending purchase/sales orders (up to 20 each).
    pub async fn pending_approvals(pool: &SqlitePool) -> Result<Vec<serde_json::Value>, AppError> {
        let mut result: Vec<serde_json::Value> = Vec::new();

        let inbound: Vec<(i64, String, String)> = sqlx::query_as(
            "SELECT id, inbound_no, 'inbound' as ref_type FROM inbound_records \
             WHERE approval_status = 'pending' AND deleted_at IS NULL ORDER BY created_at DESC LIMIT 20",
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        for (id, no, ref_type) in inbound {
            result.push(serde_json::json!({
                "id": id,
                "reference_no": no,
                "reference_type": ref_type,
            }));
        }

        let outbound: Vec<(i64, String, String)> = sqlx::query_as(
            "SELECT id, outbound_no, 'outbound' as ref_type FROM outbound_records \
             WHERE approval_status = 'pending' AND deleted_at IS NULL ORDER BY created_at DESC LIMIT 20",
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        for (id, no, ref_type) in outbound {
            result.push(serde_json::json!({
                "id": id,
                "reference_no": no,
                "reference_type": ref_type,
            }));
        }

        let purchase: Vec<(i64, String, String)> = sqlx::query_as(
            "SELECT id, order_no, 'purchase_order' as ref_type FROM purchase_orders \
             WHERE status = 'pending' AND deleted_at IS NULL ORDER BY created_at DESC LIMIT 20",
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        for (id, no, ref_type) in purchase {
            result.push(serde_json::json!({
                "id": id,
                "reference_no": no,
                "reference_type": ref_type,
            }));
        }

        let sales: Vec<(i64, String, String)> = sqlx::query_as(
            "SELECT id, order_no, 'sales_order' as ref_type FROM sales_orders \
             WHERE status = 'pending' AND deleted_at IS NULL ORDER BY created_at DESC LIMIT 20",
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        for (id, no, ref_type) in sales {
            result.push(serde_json::json!({
                "id": id,
                "reference_no": no,
                "reference_type": ref_type,
            }));
        }

        Ok(result)
    }

    /// Recent open NCRs (quality non-conformances), up to `limit`.
    pub async fn recent_quality_failures(
        pool: &SqlitePool,
        limit: i64,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let rows: Vec<(String, String, Option<i64>, DateTime<Utc>, Option<String>)> = sqlx::query_as(
            "SELECT ncr_no, description, item_id, created_at, disposition \
             FROM mfg_ncrs \
             WHERE status = 'open' \
             ORDER BY created_at DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        Ok(rows
            .into_iter()
            .map(|(ncr_no, description, item_id, created_at, disposition)| {
                serde_json::json!({
                    "ncr_no": ncr_no,
                    "description": description,
                    "item_id": item_id.map(|v| v.to_string()),
                    "created_at": created_at,
                    "disposition": disposition,
                })
            })
            .collect())
    }

    /// Sum of pending inbound + pending outbound records.
    pub async fn pending_approval_count(pool: &SqlitePool) -> Result<i64, AppError> {
        let ib: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM inbound_records WHERE approval_status = 'pending' AND deleted_at IS NULL",
        )
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;

        let ob: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM outbound_records WHERE approval_status = 'pending' AND deleted_at IS NULL",
        )
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;

        let po: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM purchase_orders WHERE status = 'pending' AND deleted_at IS NULL",
        )
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;

        let so: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM sales_orders WHERE status = 'pending' AND deleted_at IS NULL",
        )
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;

        Ok(ib.0 + ob.0 + po.0 + so.0)
    }
}
