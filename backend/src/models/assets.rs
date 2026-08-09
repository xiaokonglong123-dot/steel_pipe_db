//! Fixed asset row models — mirror `035_create_assets.sql`.

use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct FixedAsset {
    pub id: i64,
    pub tenant_id: i64,
    pub asset_no: String,
    pub name: String,
    pub category: String,
    pub purchase_date: NaiveDate,
    pub purchase_cost: f64,
    pub salvage_value: f64,
    pub useful_life_months: i32,
    pub current_value: f64,
    pub status: String,
    pub location: Option<String>,
    pub department_id: Option<i64>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct DepreciationEntry {
    pub id: i64,
    pub asset_id: i64,
    pub period: String,
    pub amount: f64,
    pub created_at: DateTime<Utc>,
}
