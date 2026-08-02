//! Fixed asset DTOs.

use chrono::NaiveDate;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateAssetRequest {
    pub name: String,
    pub category: Option<String>,
    pub purchase_date: NaiveDate,
    pub purchase_cost: rust_decimal::Decimal,
    pub salvage_value: Option<rust_decimal::Decimal>,
    pub useful_life_months: Option<i32>,
    pub location: Option<String>,
    pub department_id: Option<i64>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAssetRequest {
    pub name: Option<String>,
    pub location: Option<String>,
    pub department_id: Option<i64>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DepreciateRequest {
    pub period: String,   // 'YYYY-MM'
}
