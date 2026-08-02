//! Inventory ATP row models — mirror `031_create_inventory_atp.sql`.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct AtpSlot {
    pub id: i64,
    pub tenant_id: i64,
    pub pipe_type: String,
    pub pipe_number: Option<String>,
    pub quantity_reserved: rust_decimal::Decimal,
    pub sales_order_id: Option<i64>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub released_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct InternalTransfer {
    pub id: i64,
    pub tenant_id: i64,
    pub transfer_no: String,
    pub from_location_id: i64,
    pub to_location_id: i64,
    pub pipe_id: Option<i64>,
    pub pipe_number: Option<String>,
    pub quantity: rust_decimal::Decimal,
    pub transferred_at: DateTime<Utc>,
    pub status: String,
    pub created_by: Option<i64>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct CountTemplate {
    pub id: i64,
    pub tenant_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub location_ids: serde_json::Value,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct CountSession {
    pub id: i64,
    pub tenant_id: i64,
    pub template_id: i64,
    pub session_no: String,
    pub status: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub result_json: Option<serde_json::Value>,
}

/// ATP overview row per pipe type: on-hand minus reserved.
#[derive(Debug, Serialize, FromRow)]
pub struct AtpOverviewRow {
    pub pipe_type: String,
    pub on_hand: rust_decimal::Decimal,
    pub reserved: rust_decimal::Decimal,
    pub available: rust_decimal::Decimal,
}
