use std::sync::Arc;

use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::error::AppResult;
use crate::handler::ok_response;
use crate::AppState;

// ── Query parameter structs ──────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct DateRangeQuery {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

// ── Report: Stock Summary ────────────────────────────────────────────────────

pub async fn stock_summary(
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<impl serde::Serialize>> {
    let seamless_total: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM seamless_pipes WHERE deleted_at IS NULL")
            .fetch_one(&state.db)
            .await?;

    let screen_total: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM screen_pipes WHERE deleted_at IS NULL")
            .fetch_one(&state.db)
            .await?;

    let in_stock_total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM (
            SELECT id FROM seamless_pipes WHERE deleted_at IS NULL AND status='in_stock'
            UNION ALL
            SELECT id FROM screen_pipes WHERE deleted_at IS NULL AND status='in_stock'
        )",
    )
    .fetch_one(&state.db)
    .await?;

    #[derive(Debug, serde::Serialize, sqlx::FromRow)]
    struct StatusCount {
        status: String,
        count: i64,
    }
    let seamless_status: Vec<StatusCount> = sqlx::query_as(
        "SELECT status, COUNT(*) as count FROM seamless_pipes WHERE deleted_at IS NULL GROUP BY status",
    )
    .fetch_all(&state.db)
    .await?;

    let screen_status: Vec<StatusCount> = sqlx::query_as(
        "SELECT status, COUNT(*) as count FROM screen_pipes WHERE deleted_at IS NULL GROUP BY status",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(ok_response(json!({
        "total_in_stock": in_stock_total,
        "total_seamless": seamless_total,
        "total_screen": screen_total,
        "status_breakdown": {
            "seamless": seamless_status,
            "screen": screen_status,
        }
    })))
}

// ── Report: Stock by Grade ───────────────────────────────────────────────────

pub async fn stock_by_grade(
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<impl serde::Serialize>> {
    let grades: Vec<(String,)> = sqlx::query_as(
        "SELECT DISTINCT grade FROM (
            SELECT grade FROM seamless_pipes WHERE deleted_at IS NULL AND status='in_stock'
            UNION
            SELECT grade FROM screen_pipes WHERE deleted_at IS NULL AND status='in_stock'
        ) ORDER BY grade",
    )
    .fetch_all(&state.db)
    .await?;

    let mut result = Vec::new();
    for (grade,) in grades {
        let seamless_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM seamless_pipes WHERE deleted_at IS NULL AND status='in_stock' AND grade=?",
        )
        .bind(&grade)
        .fetch_one(&state.db)
        .await?;

        let screen_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM screen_pipes WHERE deleted_at IS NULL AND status='in_stock' AND grade=?",
        )
        .bind(&grade)
        .fetch_one(&state.db)
        .await?;

        result.push(json!({
            "grade": grade,
            "seamless_count": seamless_count,
            "screen_count": screen_count,
            "total": seamless_count + screen_count,
        }));
    }

    Ok(ok_response(json!({
        "grades": result
    })))
}

// ── Report: Stock by Location ────────────────────────────────────────────────

pub async fn stock_by_location(
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<impl serde::Serialize>> {
    #[derive(Debug, serde::Serialize, sqlx::FromRow)]
    struct LocationCount {
        location: String,
        count: i64,
    }

    let locations: Vec<LocationCount> = sqlx::query_as(
        "SELECT location, COUNT(*) as count FROM (
            SELECT location FROM seamless_pipes WHERE deleted_at IS NULL AND location IS NOT NULL AND location != ''
            UNION ALL
            SELECT location FROM screen_pipes WHERE deleted_at IS NULL AND location IS NOT NULL AND location != ''
        ) GROUP BY location ORDER BY count DESC",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(ok_response(json!({ "locations": locations })))
}

// ── Report: Inbound Summary ──────────────────────────────────────────────────

pub async fn inbound_summary(
    State(state): State<Arc<AppState>>,
    Query(range): Query<DateRangeQuery>,
) -> AppResult<Json<impl serde::Serialize>> {
    let (where_clause, start, end) = build_date_filter(&range, "i");

    let total_inbound: i64 = {
        let sql = format!("SELECT COUNT(*) FROM inbound_records i WHERE 1=1 {where_clause}");
        sqlx::query_scalar::<_, i64>(&sql)
            .fetch_one(&state.db)
            .await?
    };

    #[derive(Debug, serde::Serialize, sqlx::FromRow)]
    struct TypeCount {
        inbound_type: String,
        count: i64,
    }
    let by_type: Vec<TypeCount> = sqlx::query_as(&format!(
        "SELECT inbound_type, COUNT(*) as count FROM inbound_records i WHERE 1=1 {where_clause} GROUP BY inbound_type"
    ))
    .fetch_all(&state.db)
    .await?;

    let daily_trend: Vec<Value> = fetch_daily_trend(
        &state.db,
        "inbound_records",
        &start,
        &end,
    )
    .await?;

    Ok(ok_response(json!({
        "total_inbound": total_inbound,
        "by_type": by_type,
        "daily_trend": daily_trend,
    })))
}

// ── Report: Outbound Summary ─────────────────────────────────────────────────

pub async fn outbound_summary(
    State(state): State<Arc<AppState>>,
    Query(range): Query<DateRangeQuery>,
) -> AppResult<Json<impl serde::Serialize>> {
    let (where_clause, start, end) = build_date_filter(&range, "o");

    let total_outbound: i64 = sqlx::query_scalar(&format!(
        "SELECT COUNT(*) FROM outbound_records o WHERE 1=1 {where_clause}"
    ))
    .fetch_one(&state.db)
    .await?;

    #[derive(Debug, serde::Serialize, sqlx::FromRow)]
    struct TypeCount {
        outbound_type: String,
        count: i64,
    }
    let by_type: Vec<TypeCount> = sqlx::query_as(&format!(
        "SELECT outbound_type, COUNT(*) as count FROM outbound_records o WHERE 1=1 {where_clause} GROUP BY outbound_type"
    ))
    .fetch_all(&state.db)
    .await?;

    let daily_trend: Vec<Value> = fetch_daily_trend(
        &state.db,
        "outbound_records",
        &start,
        &end,
    )
    .await?;

    Ok(ok_response(json!({
        "total_outbound": total_outbound,
        "by_type": by_type,
        "daily_trend": daily_trend,
    })))
}

// ── Report: Monthly Flow ─────────────────────────────────────────────────────

pub async fn monthly_flow(
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<impl serde::Serialize>> {
    #[derive(Debug, serde::Serialize, sqlx::FromRow)]
    struct MonthlyFlow {
        month: String,
        inbound_count: i64,
        outbound_count: i64,
    }

    let items: Vec<MonthlyFlow> = sqlx::query_as(
        "SELECT
            COALESCE(i.month, o.month) AS month,
            COALESCE(i.cnt, 0) AS inbound_count,
            COALESCE(o.cnt, 0) AS outbound_count
        FROM (
            SELECT strftime('%Y-%m', created_at) AS month, COUNT(*) AS cnt
            FROM inbound_records GROUP BY month
        ) i
        FULL OUTER JOIN (
            SELECT strftime('%Y-%m', created_at) AS month, COUNT(*) AS cnt
            FROM outbound_records GROUP BY month
        ) o ON i.month = o.month
        ORDER BY month DESC
        LIMIT 24",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(ok_response(json!({ "monthly_flow": items })))
}

// ── Report: Purchase Summary ─────────────────────────────────────────────────

pub async fn purchase_summary(
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<impl serde::Serialize>> {
    #[derive(Debug, serde::Serialize, sqlx::FromRow)]
    struct SupplierSummary {
        supplier_id: String,
        supplier_name: String,
        total_amount: f64,
        order_count: i64,
    }
    let by_supplier: Vec<SupplierSummary> = sqlx::query_as(
        "SELECT s.id AS supplier_id, s.name AS supplier_name,
                COALESCE(SUM(p.total_amount), 0) AS total_amount,
                COUNT(p.id) AS order_count
         FROM suppliers s
         LEFT JOIN purchase_orders p ON p.supplier_id = s.id
         GROUP BY s.id, s.name
         ORDER BY total_amount DESC",
    )
    .fetch_all(&state.db)
    .await?;

    #[derive(Debug, serde::Serialize, sqlx::FromRow)]
    struct StatusSummary {
        status: String,
        count: i64,
        total_amount: f64,
    }
    let by_status: Vec<StatusSummary> = sqlx::query_as(
        "SELECT status, COUNT(*) AS count, COALESCE(SUM(total_amount), 0) AS total_amount
         FROM purchase_orders GROUP BY status ORDER BY status",
    )
    .fetch_all(&state.db)
    .await?;

    let monthly_trend: Vec<Value> = sqlx::query_as::<_, (String, i64, f64)>(
        "SELECT strftime('%Y-%m', created_at) AS month,
                COUNT(*) AS order_count,
                COALESCE(SUM(total_amount), 0) AS total_amount
         FROM purchase_orders
         GROUP BY month ORDER BY month DESC LIMIT 24",
    )
    .fetch_all(&state.db)
    .await?
    .into_iter()
    .map(|(month, count, amount)| json!({"month": month, "order_count": count, "total_amount": amount}))
    .collect();

    Ok(ok_response(json!({
        "by_supplier": by_supplier,
        "by_status": by_status,
        "monthly_trend": monthly_trend,
    })))
}

// ── Report: Sales Summary ────────────────────────────────────────────────────

pub async fn sales_summary(
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<impl serde::Serialize>> {
    #[derive(Debug, serde::Serialize, sqlx::FromRow)]
    struct CustomerSummary {
        customer_id: String,
        customer_name: String,
        total_amount: f64,
        order_count: i64,
    }
    let by_customer: Vec<CustomerSummary> = sqlx::query_as(
        "SELECT c.id AS customer_id, c.name AS customer_name,
                COALESCE(SUM(s.total_amount), 0) AS total_amount,
                COUNT(s.id) AS order_count
         FROM customers c
         LEFT JOIN sales_orders s ON s.customer_id = c.id
         GROUP BY c.id, c.name
         ORDER BY total_amount DESC",
    )
    .fetch_all(&state.db)
    .await?;

    #[derive(Debug, serde::Serialize, sqlx::FromRow)]
    struct StatusSummary {
        status: String,
        count: i64,
        total_amount: f64,
    }
    let by_status: Vec<StatusSummary> = sqlx::query_as(
        "SELECT status, COUNT(*) AS count, COALESCE(SUM(total_amount), 0) AS total_amount
         FROM sales_orders GROUP BY status ORDER BY status",
    )
    .fetch_all(&state.db)
    .await?;

    let monthly_trend: Vec<Value> = sqlx::query_as::<_, (String, i64, f64)>(
        "SELECT strftime('%Y-%m', created_at) AS month,
                COUNT(*) AS order_count,
                COALESCE(SUM(total_amount), 0) AS total_amount
         FROM sales_orders
         GROUP BY month ORDER BY month DESC LIMIT 24",
    )
    .fetch_all(&state.db)
    .await?
    .into_iter()
    .map(|(month, count, amount)| json!({"month": month, "order_count": count, "total_amount": amount}))
    .collect();

    Ok(ok_response(json!({
        "by_customer": by_customer,
        "by_status": by_status,
        "monthly_trend": monthly_trend,
    })))
}

// ── Report: Financial Monthly ────────────────────────────────────────────────

pub async fn financial_monthly(
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<impl serde::Serialize>> {
    #[derive(Debug, serde::Serialize, sqlx::FromRow)]
    struct MonthlyFinance {
        month: String,
        purchase_amount: f64,
        sales_amount: f64,
        gross_profit: f64,
    }

    let items: Vec<MonthlyFinance> = sqlx::query_as(
        "SELECT
            COALESCE(p.month, s.month) AS month,
            COALESCE(p.total_amount, 0) AS purchase_amount,
            COALESCE(s.total_amount, 0) AS sales_amount,
            COALESCE(s.total_amount, 0) - COALESCE(p.total_amount, 0) AS gross_profit
        FROM (
            SELECT strftime('%Y-%m', created_at) AS month,
                   COALESCE(SUM(total_amount), 0) AS total_amount
            FROM purchase_orders
            WHERE status != 'draft'
            GROUP BY month
        ) p
        FULL OUTER JOIN (
            SELECT strftime('%Y-%m', created_at) AS month,
                   COALESCE(SUM(total_amount), 0) AS total_amount
            FROM sales_orders
            WHERE status != 'draft'
            GROUP BY month
        ) s ON p.month = s.month
        ORDER BY month DESC
        LIMIT 24",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(ok_response(json!({ "financial_monthly": items })))
}

// ── Helper functions ─────────────────────────────────────────────────────────

/// Build a WHERE clause for date range filtering. Returns (clause, start, end).
/// `alias` is the table alias used in the query (e.g., "i" for inbound_records i).
fn build_date_filter(range: &DateRangeQuery, alias: &str) -> (String, Option<String>, Option<String>) {
    let start = range.start_date.as_ref().filter(|s| !s.is_empty());
    let end = range.end_date.as_ref().filter(|s| !s.is_empty());

    let mut clauses = Vec::new();
    if start.is_some() {
        clauses.push(format!("AND {alias}.created_at >= ?"));
    }
    if end.is_some() {
        clauses.push(format!("AND {alias}.created_at <= ?"));
    }
    (clauses.join(" "), start.cloned(), end.cloned())
}

async fn fetch_daily_trend(
    pool: &sqlx::SqlitePool,
    table: &str,
    start: &Option<String>,
    end: &Option<String>,
) -> Result<Vec<Value>, sqlx::Error> {
    let mut sql = format!(
        "SELECT DATE(created_at) AS day, COUNT(*) AS count FROM {table} WHERE 1=1"
    );

    if start.is_some() {
        sql.push_str(" AND created_at >= ?");
    }
    if end.is_some() {
        sql.push_str(" AND created_at <= ?");
    }
    sql.push_str(" GROUP BY day ORDER BY day DESC LIMIT 30");

    let mut query = sqlx::query_as::<_, (String, i64)>(&sql);
    if let Some(s) = start {
        query = query.bind(s);
    }
    if let Some(e) = end {
        query = query.bind(e);
    }

    let rows = query.fetch_all(pool).await?;
    Ok(rows
        .into_iter()
        .map(|(day, count)| json!({"date": day, "count": count}))
        .collect())
}
