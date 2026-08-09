use serde::{Deserialize, Serialize};
use validator::Validate;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Inbound DTOs
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// One inbound line item — item + quantity.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InboundItemRequest {
    /// FK to the items master table.
    pub item_id: i64,
    /// Received quantity.
    pub quantity: f64,
}

/// Inbound record detail response DTO — record header + line items.
#[derive(Debug, Serialize)]
pub struct InboundRecordDetail {
    pub record: crate::models::inventory::InboundRecord,
    pub items: Vec<crate::models::inventory::InboundItem>,
}

/// Update inbound record request DTO — only editable fields.
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateInboundRecordRequest {
    /// Free-form notes.
    pub notes: Option<String>,
    /// Related order ID.
    pub order_id: Option<i64>,
    /// Supplier ID.
    pub supplier_id: Option<i64>,
}

/// Create inbound record request DTO.
#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct CreateInboundRecordRequest {
    /// Inbound type: purchase / production / return.
    #[validate(length(min = 1))]
    pub inbound_type: String,
    /// Related order ID.
    pub order_id: Option<i64>,
    /// Supplier ID.
    pub supplier_id: Option<i64>,
    /// Notes.
    pub notes: Option<String>,
    /// Line items (item_id + quantity). At least one required.
    #[validate(length(min = 1, message = "At least one item is required"))]
    pub items: Vec<InboundItemRequest>,
}

/// Inbound record list filter params.
#[derive(Debug, Deserialize)]
pub struct InboundFilter {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    /// Filter by inbound type.
    pub inbound_type: Option<String>,
    /// Filter by approval status.
    pub approval_status: Option<String>,
    /// Filter by related order ID.
    pub order_id: Option<i64>,
    /// Full-text search on inbound number.
    pub q: Option<String>,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Outbound DTOs
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Update outbound record request DTO — only editable fields.
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateOutboundRecordRequest {
    /// Free-form notes.
    pub notes: Option<String>,
    /// Related order ID.
    pub order_id: Option<i64>,
    /// Customer ID.
    pub customer_id: Option<i64>,
}

/// Outbound record detail response DTO — record header + line items.
#[derive(Debug, Serialize)]
pub struct OutboundRecordDetail {
    pub record: crate::models::inventory::OutboundRecord,
    pub items: Vec<crate::models::inventory::OutboundItem>,
}

/// One outbound line item — item + quantity.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OutboundItemRequest {
    /// FK to the items master table.
    pub item_id: i64,
    /// Shipped quantity.
    pub quantity: f64,
}

/// Create outbound record request DTO.
#[derive(Debug, Deserialize, Validate)]
pub struct CreateOutboundRecordRequest {
    /// Outbound type: sales / transfer / scrapped.
    #[validate(length(min = 1))]
    pub outbound_type: String,
    /// Related order ID.
    pub order_id: Option<i64>,
    /// Customer ID.
    pub customer_id: Option<i64>,
    /// Notes.
    pub notes: Option<String>,
    /// Line items (item_id + quantity). At least one required.
    #[validate(length(min = 1, message = "At least one item is required"))]
    pub items: Vec<OutboundItemRequest>,
}

/// Outbound record list filter params.
#[derive(Debug, Deserialize)]
pub struct OutboundFilter {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    /// Filter by outbound type.
    pub outbound_type: Option<String>,
    /// Filter by approval status.
    pub approval_status: Option<String>,
    /// Filter by related order ID.
    pub order_id: Option<i64>,
    /// Full-text search on outbound number.
    pub q: Option<String>,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Inventory DTOs
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Inventory filter params DTO.
#[derive(Debug, Deserialize)]
pub struct InventoryFilter {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    /// Filter by item master ID.
    pub item_id: Option<i64>,
    /// Filter by item category.
    pub category: Option<String>,
    /// Filter by location.
    pub location_id: Option<i64>,
    /// Full-text search on SKU / name / spec.
    pub q: Option<String>,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Location DTOs
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Create location request DTO.
#[derive(Debug, Deserialize, Validate)]
pub struct CreateLocationRequest {
    /// Zone code.
    #[validate(length(min = 1))]
    pub zone_code: String,
    /// Shelf code.
    #[validate(length(min = 1))]
    pub shelf_code: String,
    /// Level code.
    #[validate(length(min = 1))]
    pub level_code: String,
    /// Location description.
    pub description: Option<String>,
    /// Capacity (max quantity it can hold).
    pub capacity: Option<i64>,
}

/// Update location request DTO.
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateLocationRequest {
    /// Location description.
    pub description: Option<String>,
    /// Capacity.
    pub capacity: Option<i64>,
    /// Whether active.
    pub is_active: Option<bool>,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Check DTOs
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Create inventory check record request DTO.
#[derive(Debug, Deserialize, Validate)]
pub struct CreateCheckRequest {
    /// Location ID to check (empty = full inventory check).
    pub location_id: Option<i64>,
    /// Notes.
    pub notes: Option<String>,
}

/// Submit check item result DTO.
#[derive(Debug, Deserialize, Validate)]
pub struct SubmitCheckItemRequest {
    /// Actual quantity found during check.
    pub found_quantity: f64,
    /// Notes.
    pub notes: Option<String>,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Approval DTOs
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Approve request DTO — optionally includes an approval reason/comment.
#[derive(Debug, Deserialize, Validate)]
pub struct ApproveRequest {
    /// Optional approval reason or comment.
    pub reason: Option<String>,
}

/// Reject request DTO.
#[derive(Debug, Deserialize, Validate)]
pub struct RejectRequest {
    /// Rejection reason.
    #[validate(length(min = 1))]
    pub reason: String,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  ATP DTOs
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// ATP (Available-to-Promise) query params.
#[derive(Debug, Deserialize, Validate)]
pub struct AtpQuery {
    /// Filter by item master ID.
    #[validate(range(min = 1, message = "Item ID must be positive"))]
    pub item_id: Option<i64>,
    /// Filter by location.
    #[validate(range(min = 1, message = "Location ID must be positive"))]
    pub location_id: Option<i64>,
}

/// ATP query result item DTO.
#[derive(Debug, Serialize)]
pub struct AtpItem {
    /// Item master ID.
    pub item_id: i64,
    /// SKU.
    pub sku: Option<String>,
    /// Available quantity.
    pub quantity: f64,
    /// Location ID.
    pub location_id: Option<i64>,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Stock / Statistics DTOs
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Batch create inbound records request DTO.
#[derive(Debug, Deserialize, Validate)]
pub struct BatchCreateInboundRequest {
    /// List of inbound records.
    #[validate(length(min = 1))]
    pub records: Vec<CreateInboundRecordRequest>,
}

/// Inventory statistics response DTO — total stock, breakdown by category and location.
#[derive(Debug, Serialize)]
pub struct InventoryStatistics {
    pub total_in_stock: f64,
    pub by_category: Vec<crate::inventory::inventory_repo::CategoryCount>,
    pub by_location: Vec<crate::inventory::inventory_repo::LocationCount>,
}

/// Check record detail response DTO — record header + check items.
#[derive(Debug, Serialize)]
pub struct CheckRecordDetail {
    pub record: crate::models::inventory::InventoryCheckRecord,
    pub items: Vec<crate::models::inventory::InventoryCheckItem>,
}

/// Stock item DTO — item master row joined with computed on-hand quantity.
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct StockItem {
    pub id: i64,
    pub sku: String,
    pub name: String,
    pub category: Option<String>,
    pub unit: Option<String>,
    pub spec: Option<String>,
    pub status: String,
    /// Computed on-hand quantity from inventory_logs.
    pub quantity: f64,
}
