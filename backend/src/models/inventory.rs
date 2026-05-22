use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Location {
    pub id: i64,
    pub zone_code: String,
    pub shelf_code: String,
    pub level_code: String,
    pub full_code: String,
    pub description: Option<String>,
    pub capacity: Option<i64>,
    pub used_count: i64,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InboundRecord {
    pub id: i64,
    pub inbound_no: String,
    pub inbound_type: String,
    pub order_id: Option<i64>,
    pub supplier_id: Option<i64>,
    pub notes: Option<String>,
    pub approval_status: String,
    pub rejection_reason: Option<String>,
    pub handled_by: Option<i64>,
    pub handled_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InboundItem {
    pub id: i64,
    pub inbound_id: i64,
    pub pipe_type: String,
    pub pipe_id: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OutboundRecord {
    pub id: i64,
    pub outbound_no: String,
    pub outbound_type: String,
    pub order_id: Option<i64>,
    pub customer_id: Option<i64>,
    pub notes: Option<String>,
    pub approval_status: String,
    pub rejection_reason: Option<String>,
    pub handled_by: Option<i64>,
    pub handled_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OutboundItem {
    pub id: i64,
    pub outbound_id: i64,
    pub pipe_type: String,
    pub pipe_id: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InventoryLog {
    pub id: i64,
    pub pipe_type: String,
    pub pipe_id: i64,
    pub change_type: String,
    pub ref_type: Option<String>,
    pub ref_id: Option<i64>,
    pub from_location_id: Option<i64>,
    pub to_location_id: Option<i64>,
    pub notes: Option<String>,
    pub created_by: Option<i64>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InventoryCheckRecord {
    pub id: i64,
    pub check_no: String,
    pub location_id: Option<i64>,
    pub status: String,
    pub notes: Option<String>,
    pub created_by: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InventoryCheckItem {
    pub id: i64,
    pub check_id: i64,
    pub pipe_type: String,
    pub pipe_id: i64,
    pub expected_status: String,
    pub found_status: Option<String>,
    pub is_match: Option<bool>,
    pub notes: Option<String>,
    pub created_at: String,
}
