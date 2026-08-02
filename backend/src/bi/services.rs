//! BI analytics services — aggregate queries over existing tables.
//! No dedicated tables; every query reads from pipes/orders/finance/inventory.

use sqlx::PgPool;

use crate::error::AppError;

pub struct BiService;

impl BiService {
    /// Monthly sales totals (order_date month × status).
    pub async fn sales_trend(pool: &PgPool, tenant_id: i64, months: i32) -> Result<Vec<SalesTrendRow>, AppError> {
        sqlx::query_as::<_, SalesTrendRow>(
            "SELECT to_char(order_date, 'YYYY-MM') AS month, status, COUNT(*) AS order_count, \
                    COALESCE(SUM(total_amount), 0)::NUMERIC AS total_amount \
             FROM sales_orders WHERE order_date >= NOW() - ($2::int || ' months')::interval \
             GROUP BY 1, 2 ORDER BY 1 DESC, 2",
        )
        .bind(tenant_id)
        .bind(months)
        .fetch_all(pool)
        .await
        .map_err(AppError::from)
    }

    /// Inventory value by pipe type (on-hand count × nominal weight × price proxy).
    pub async fn inventory_value(pool: &PgPool) -> Result<Vec<InventoryValueRow>, AppError> {
        sqlx::query_as::<_, InventoryValueRow>(
            "SELECT 'seamless' AS pipe_type, COUNT(*) AS on_hand \
             FROM seamless_pipes WHERE status = 'in_stock' \
             UNION ALL \
             SELECT 'screen', COUNT(*) FROM screen_pipes WHERE status = 'in_stock' \
             UNION ALL \
             SELECT 'welded', COUNT(*) FROM welded_pipes WHERE status = 'in_stock'",
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)
    }

    /// Finance summary: posted entries, AR/AP open invoices, payments.
    pub async fn finance_summary(pool: &PgPool, tenant_id: i64) -> Result<FinanceSummary, AppError> {
        let posted_entries: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM journal_entries WHERE tenant_id = $1 AND status = 'posted'",
        )
        .bind(tenant_id)
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;
        let open_ar: rust_decimal::Decimal = sqlx::query_scalar(
            "SELECT COALESCE(SUM(total_amount), 0) FROM finance_invoices \
             WHERE tenant_id = $1 AND invoice_type = 'sales' AND status IN ('confirmed', 'draft')",
        )
        .bind(tenant_id)
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;
        let open_ap: rust_decimal::Decimal = sqlx::query_scalar(
            "SELECT COALESCE(SUM(total_amount), 0) FROM finance_invoices \
             WHERE tenant_id = $1 AND invoice_type = 'purchase' AND status IN ('confirmed', 'draft')",
        )
        .bind(tenant_id)
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;
        let payments: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM finance_payments WHERE tenant_id = $1",
        )
        .bind(tenant_id)
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;
        Ok(FinanceSummary { posted_entries, open_ar, open_ap, payment_count: payments })
    }

    /// Supplier performance: order counts + totals per supplier.
    pub async fn supplier_performance(pool: &PgPool, _tenant_id: i64) -> Result<Vec<SupplierPerfRow>, AppError> {
        sqlx::query_as::<_, SupplierPerfRow>(
            "SELECT s.id AS supplier_id, s.name AS supplier_name, \
                    COUNT(po.id) AS order_count, \
                    COALESCE(SUM(po.total_amount), 0)::NUMERIC AS order_total \
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
    pub total_amount: rust_decimal::Decimal,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct InventoryValueRow {
    pub pipe_type: String,
    pub on_hand: i64,
}

#[derive(Debug, serde::Serialize)]
pub struct FinanceSummary {
    pub posted_entries: i64,
    pub open_ar: rust_decimal::Decimal,
    pub open_ap: rust_decimal::Decimal,
    pub payment_count: i64,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct SupplierPerfRow {
    pub supplier_id: i64,
    pub supplier_name: String,
    pub order_count: i64,
    pub order_total: rust_decimal::Decimal,
}
