//! Manufacturing row models — mirror `032_create_manufacturing.sql`.

use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct Bom {
    pub id: i64,
    pub tenant_id: i64,
    pub name: String,
    pub product_type: String,
    pub version: i32,
    pub is_active: bool,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct BomItem {
    pub id: i64,
    pub bom_id: i64,
    pub material: String,
    pub quantity: f64,
    pub unit: String,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct WorkOrder {
    pub id: i64,
    pub tenant_id: i64,
    pub wo_no: String,
    pub bom_id: Option<i64>,
    pub product_type: String,
    pub quantity: f64,
    pub status: String,
    pub current_step: i32,
    pub assigned_to: Option<i64>,
    pub due_date: Option<NaiveDate>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct WorkOrderStep {
    pub id: i64,
    pub work_order_id: i64,
    pub step_index: i32,
    pub step_name: String,
    pub status: String,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct Inspection {
    pub id: i64,
    pub tenant_id: i64,
    pub work_order_id: Option<i64>,
    pub item_id: Option<i64>,
    pub inspection_type: String,
    pub result: String,
    pub inspector: Option<i64>,
    pub notes: Option<String>,
    pub inspected_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct Ncr {
    pub id: i64,
    pub tenant_id: i64,
    pub ncr_no: String,
    pub work_order_id: Option<i64>,
    pub item_id: Option<i64>,
    pub description: String,
    pub severity: String,
    pub disposition: Option<String>,
    pub status: String,
    pub created_by: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}
