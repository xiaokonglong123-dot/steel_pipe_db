use serde::{Deserialize, Serialize};
use validator::Validate;

// ━━━ Inbound DTOs ━━━

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct CreateInboundRecordRequest {
    #[validate(length(min = 1))]
    pub inbound_type: String,
    pub order_id: Option<i64>,
    pub supplier_id: Option<i64>,
    pub notes: Option<String>,
    pub pipes: Vec<InboundPipeItem>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct InboundPipeItem {
    #[validate(length(min = 1))]
    pub pipe_type: String,
    #[validate(range(min = 1))]
    pub pipe_id: i64,
}

#[derive(Debug, Deserialize)]
pub struct InboundFilter {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub inbound_type: Option<String>,
    pub approval_status: Option<String>,
    pub order_id: Option<i64>,
    pub q: Option<String>,
}

// ━━━ Outbound DTOs ━━━

#[derive(Debug, Deserialize, Validate)]
pub struct CreateOutboundRecordRequest {
    #[validate(length(min = 1))]
    pub outbound_type: String,
    pub order_id: Option<i64>,
    pub customer_id: Option<i64>,
    pub notes: Option<String>,
    pub pipes: Vec<OutboundPipeItem>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct OutboundPipeItem {
    #[validate(length(min = 1))]
    pub pipe_type: String,
    #[validate(range(min = 1))]
    pub pipe_id: i64,
}

#[derive(Debug, Deserialize)]
pub struct OutboundFilter {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub outbound_type: Option<String>,
    pub approval_status: Option<String>,
    pub order_id: Option<i64>,
    pub q: Option<String>,
}

// ━━━ Inventory DTOs ━━━

#[derive(Debug, Deserialize)]
pub struct InventoryFilter {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub grade: Option<String>,
    pub pipe_type: Option<String>,
    pub location_id: Option<i64>,
    pub q: Option<String>,
}

// ━━━ Location DTOs ━━━

#[derive(Debug, Deserialize, Validate)]
pub struct CreateLocationRequest {
    #[validate(length(min = 1))]
    pub zone_code: String,
    #[validate(length(min = 1))]
    pub shelf_code: String,
    #[validate(length(min = 1))]
    pub level_code: String,
    pub description: Option<String>,
    pub capacity: Option<i64>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateLocationRequest {
    pub description: Option<String>,
    pub capacity: Option<i64>,
    pub is_active: Option<bool>,
}

// ━━━ Check DTOs ━━━

#[derive(Debug, Deserialize, Validate)]
pub struct CreateCheckRequest {
    pub location_id: Option<i64>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct SubmitCheckItemRequest {
    #[validate(length(min = 1))]
    pub found_status: String,
    pub notes: Option<String>,
}

// ━━━ Approval DTOs ━━━

#[derive(Debug, Deserialize, Validate)]
pub struct ApproveRequest {
}

#[derive(Debug, Deserialize, Validate)]
pub struct RejectRequest {
    #[validate(length(min = 1))]
    pub reason: String,
}

// ━━━ ATP DTOs ━━━

#[derive(Debug, Deserialize, Validate)]
pub struct AtpQuery {
    pub pipe_type: Option<String>,
    pub grade: Option<String>,
    #[validate(range(min = 1, message = "Location ID must be positive"))]
    pub location_id: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct AtpItem {
    pub pipe_type: String,
    pub grade: String,
    pub quantity: i64,
    pub location_id: Option<i64>,
}

// ━━━ Location Assignment DTOs ━━━

#[derive(Debug, Deserialize, Validate)]
pub struct AssignLocationRequest {
    #[validate(length(min = 1))]
    pub pipe_type: String,
    #[validate(range(min = 1))]
    pub pipe_id: i64,
}

#[derive(Debug, Deserialize, Validate)]
pub struct TransferLocationRequest {
    #[validate(range(min = 1))]
    pub to_location_id: i64,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct BatchCreateInboundRequest {
    #[validate(length(min = 1))]
    pub records: Vec<CreateInboundRecordRequest>,
}
