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
    pub budget: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateWbsRequest {
    pub parent_id: Option<i64>,
    pub code: String,
    pub name: String,
    pub weight_pct: Option<f64>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub assignee_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWbsProgressRequest {
    pub progress_pct: f64,
}

#[derive(Debug, Deserialize)]
pub struct CreateTransactionRequest {
    pub tx_type: String,
    pub amount: f64,
    pub description: Option<String>,
    pub tx_date: Option<NaiveDate>,
}
