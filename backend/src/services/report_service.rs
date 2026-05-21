use sqlx::SqlitePool;

use crate::error::AppError;
use crate::repositories::report_repo::ReportRepo;

pub struct ReportService;

impl ReportService {
    pub async fn inventory_summary(pool: &SqlitePool) -> Result<serde_json::Value, AppError> {
        let by_status = ReportRepo::inventory_by_status(pool).await?;
        let by_grade = ReportRepo::inventory_by_grade(pool).await?;
        let by_type = ReportRepo::inventory_by_type(pool).await?;
        let location_occupancy = ReportRepo::location_occupancy(pool).await?;

        Ok(serde_json::json!({
            "by_status": by_status,
            "by_grade": by_grade,
            "by_type": by_type,
            "location_occupancy": location_occupancy,
        }))
    }

    pub async fn order_report(
        pool: &SqlitePool,
        order_type: &str,
        period: &str,
    ) -> Result<serde_json::Value, AppError> {
        let period = if period.is_empty() { "monthly" } else { period };

        let (orders, top_entities, status_dist, label, detail_key) = match order_type {
            "sales" => {
                let orders = ReportRepo::sales_order_report(pool, period).await?;
                let top = ReportRepo::top_customers(pool, 10).await?;
                let status = ReportRepo::order_status_distribution(pool, "sales_orders").await?;
                (orders, serde_json::json!(top), status, "sales", "top_customers")
            }
            _ => {
                let orders = ReportRepo::purchase_order_report(pool, period).await?;
                let top = ReportRepo::top_suppliers(pool, 10).await?;
                let status = ReportRepo::order_status_distribution(pool, "purchase_orders").await?;
                (orders, serde_json::json!(top), status, "purchase", "top_suppliers")
            }
        };

        Ok(serde_json::json!({
            "type": label,
            "period": period,
            "orders": orders,
            "status_distribution": status_dist,
            (detail_key): top_entities,
        }))
    }

    pub async fn quality_report(pool: &SqlitePool) -> Result<serde_json::Value, AppError> {
        let by_grade = ReportRepo::quality_pass_fail_by_grade(pool).await?;
        let by_month = ReportRepo::quality_certs_by_month(pool).await?;

        Ok(serde_json::json!({
            "by_grade": by_grade,
            "by_month": by_month,
        }))
    }

    pub async fn dashboard(pool: &SqlitePool) -> Result<serde_json::Value, AppError> {
        let stock = ReportRepo::total_stock(pool).await?;
        let inbound_count = ReportRepo::inbound_count_30d(pool).await?;
        let outbound_count = ReportRepo::outbound_count_30d(pool).await?;
        let recent_inbound = ReportRepo::recent_inbound(pool, 30, 10).await?;
        let recent_outbound = ReportRepo::recent_outbound(pool, 30, 10).await?;
        let pending = ReportRepo::pending_approval_count(pool).await?;
        let pending_list = ReportRepo::pending_approvals(pool).await?;
        let recent_failures = ReportRepo::recent_quality_failures(pool, 10).await?;

        Ok(serde_json::json!({
            "total_stock": stock,
            "inbound_30d": inbound_count,
            "outbound_30d": outbound_count,
            "recent_inbound": recent_inbound,
            "recent_outbound": recent_outbound,
            "pending_approvals": pending,
            "pending_approval_list": pending_list,
            "recent_quality_failures": recent_failures,
        }))
    }
}
