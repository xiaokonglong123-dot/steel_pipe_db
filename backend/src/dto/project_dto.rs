//! Project DTOs.

use chrono::NaiveDate;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub manager_id: Option<i64>,
    pub budget: Option<rust_decimal::Decimal>,
}

#[derive(Debug, Deserialize)]
pub struct CreateWbsRequest {
    pub parent_id: Option<i64>,
    pub code: String,
    pub name: String,
    pub weight_pct: Option<rust_decimal::Decimal>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub assignee_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWbsProgressRequest {
    pub progress_pct: rust_decimal::Decimal,
}

#[derive(Debug, Deserialize)]
pub struct CreateTransactionRequest {
    pub tx_type: String,
    pub amount: rust_decimal::Decimal,
    pub description: Option<String>,
    pub tx_date: Option<NaiveDate>,
}
