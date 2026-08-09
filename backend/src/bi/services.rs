//! BI analytics services — aggregate queries over existing tables.
//! No dedicated tables; every query reads from items/inventory/orders/finance.

use sqlx::SqlitePool;

use crate::error::AppError;

pub struct BiService;

impl BiService {
    /// Monthly sales totals (order_date month × status).
    ///
    /// `tenant_id` is unused: `sales_orders` has no tenant column (single-tenant
    /// system — see docs/refactor-issue-list-2026-08-09.md). Kept in the signature
    /// to avoid churn; removed when the tenant cleanup lands.
    pub async fn sales_trend(pool: &SqlitePool, tenant_id: i64, months: i32) -> Result<Vec<SalesTrendRow>, AppError> {
        let _ = tenant_id;
        sqlx::query_as::<_, SalesTrendRow>(
            "SELECT strftime('%Y-%m', order_date) AS month, status, COUNT(*) AS order_count, \
                    CAST(COALESCE(SUM(total_amount), 0.0) AS REAL) AS total_amount \
             FROM sales_orders WHERE order_date >= datetime('now', '-' || ? || ' months') \
             GROUP BY 1, 2 ORDER BY 1 DESC, 2",
        )
        .bind(months)
        .fetch_all(pool)
        .await
        .map_err(AppError::from)
    }

    /// Inventory value by item (net on-hand quantity from inventory movements).
    pub async fn inventory_value(pool: &SqlitePool) -> Result<Vec<InventoryValueRow>, AppError> {
        sqlx::query_as::<_, InventoryValueRow>(
            "SELECT i.sku, i.name, \
                    CAST(COALESCE(SUM(CASE WHEN l.change_type = 'inbound' THEN l.quantity \
                                      WHEN l.change_type = 'outbound' THEN -l.quantity \
                                      ELSE 0 END), 0.0) AS REAL) AS on_hand \
             FROM items i \
             LEFT JOIN inventory_logs l ON l.item_id = i.id \
             WHERE i.deleted_at IS NULL \
             GROUP BY i.id, i.sku, i.name \
             ORDER BY on_hand DESC LIMIT 50",
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)
    }

    /// Finance summary: posted entries, AR/AP open invoices, payments.
    pub async fn finance_summary(pool: &SqlitePool, tenant_id: i64) -> Result<FinanceSummary, AppError> {
        let posted_entries: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM journal_entries WHERE tenant_id = ? AND status = 'posted'",
        )
        .bind(tenant_id)
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;
        let open_ar: f64 = sqlx::query_scalar(
            "SELECT CAST(COALESCE(SUM(total_amount), 0.0) AS REAL) FROM finance_invoices \
             WHERE tenant_id = ? AND invoice_type = 'sales' AND status IN ('confirmed', 'draft')",
        )
        .bind(tenant_id)
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;
        let open_ap: f64 = sqlx::query_scalar(
            "SELECT CAST(COALESCE(SUM(total_amount), 0.0) AS REAL) FROM finance_invoices \
             WHERE tenant_id = ? AND invoice_type = 'purchase' AND status IN ('confirmed', 'draft')",
        )
        .bind(tenant_id)
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;
        let payments: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM finance_payments WHERE tenant_id = ?",
        )
        .bind(tenant_id)
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;
        Ok(FinanceSummary { posted_entries, open_ar, open_ap, payment_count: payments })
    }

    /// Supplier performance: order counts + totals per supplier.
    pub async fn supplier_performance(pool: &SqlitePool, _tenant_id: i64) -> Result<Vec<SupplierPerfRow>, AppError> {
        sqlx::query_as::<_, SupplierPerfRow>(
            "SELECT s.id AS supplier_id, s.name AS supplier_name, \
                    COUNT(po.id) AS order_count, \
                    CAST(COALESCE(SUM(po.total_amount), 0.0) AS REAL) AS order_total \
             FROM suppliers s \
             LEFT JOIN purchase_orders po ON po.supplier_id = s.id AND po.deleted_at IS NULL \
             WHERE s.deleted_at IS NULL \
             GROUP BY s.id, s.name ORDER BY order_total DESC LIMIT 20",
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)
    }
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct SalesTrendRow {
    pub month: String,
    pub status: String,
    pub order_count: i64,
    pub total_amount: f64,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct InventoryValueRow {
    pub sku: String,
    pub name: String,
    pub on_hand: f64,
}

#[derive(Debug, serde::Serialize)]
pub struct FinanceSummary {
    pub posted_entries: i64,
    pub open_ar: f64,
    pub open_ap: f64,
    pub payment_count: i64,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct SupplierPerfRow {
    pub supplier_id: i64,
    pub supplier_name: String,
    pub order_count: i64,
    pub order_total: f64,
}
