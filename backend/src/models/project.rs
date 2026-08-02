//! Project row models — mirror `034_create_projects.sql`.

use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct Project {
    pub id: i64,
    pub tenant_id: i64,
    pub project_no: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub manager_id: Option<i64>,
    pub budget: rust_decimal::Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct WbsElement {
    pub id: i64,
    pub tenant_id: i64,
    pub project_id: i64,
    pub parent_id: Option<i64>,
    pub code: String,
    pub name: String,
    pub sort_order: i32,
    pub weight_pct: Option<rust_decimal::Decimal>,
    pub progress_pct: rust_decimal::Decimal,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub assignee_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ProjectTransaction {
    pub id: i64,
    pub tenant_id: i64,
    pub project_id: i64,
    pub tx_type: String,
    pub amount: rust_decimal::Decimal,
    pub description: Option<String>,
    pub tx_date: NaiveDate,
    pub created_by: Option<i64>,
    pub created_at: DateTime<Utc>,
}

/// Budget vs actual summary for a project.
#[derive(Debug, Serialize, FromRow)]
pub struct ProjectFinancials {
    pub project_id: i64,
    pub budget: rust_decimal::Decimal,
    pub expense_total: rust_decimal::Decimal,
    pub revenue_total: rust_decimal::Decimal,
    pub remaining: rust_decimal::Decimal,
}
