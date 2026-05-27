use serde::{Deserialize, Serialize};
use validator::Validate;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Inbound DTOs
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Inbound record detail response DTO — record header + line items.
#[derive(Debug, Serialize)]
pub struct InboundRecordDetail {
    pub record: crate::models::inventory::InboundRecord,
    pub items: Vec<crate::models::inventory::InboundItem>,
}

/// Outbound record detail response DTO — record header + line items.
#[derive(Debug, Serialize)]
pub struct OutboundRecordDetail {
    pub record: crate::models::inventory::OutboundRecord,
    pub items: Vec<crate::models::inventory::OutboundItem>,
}

/// Create inbound record request DTO.
#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct CreateInboundRecordRequest {
    /// Inbound type: purchase / production / return / transfer.
    #[validate(length(min = 1))]
    pub inbound_type: String,
    /// Related order ID.
    pub order_id: Option<i64>,
    /// Supplier ID.
    pub supplier_id: Option<i64>,
    /// Notes.
    pub notes: Option<String>,
    /// List of inbound pipes.
    pub pipes: Vec<InboundPipeItem>,
}

/// Inbound pipe item DTO.
#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct InboundPipeItem {
    /// Pipe type: seamless or screen.
    #[validate(length(min = 1))]
    pub pipe_type: String,
    /// Pipe ID.
    #[validate(range(min = 1))]
    pub pipe_id: i64,
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
    /// Full-text search.
    pub q: Option<String>,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Outbound DTOs
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Create outbound record request DTO.
#[derive(Debug, Deserialize, Validate)]
pub struct CreateOutboundRecordRequest {
    /// Outbound type: sales / scrapped / transfer.
    #[validate(length(min = 1))]
    pub outbound_type: String,
    /// Related order ID.
    pub order_id: Option<i64>,
    /// Customer ID.
    pub customer_id: Option<i64>,
    /// Notes.
    pub notes: Option<String>,
    /// List of outbound pipes.
    pub pipes: Vec<OutboundPipeItem>,
}

/// Outbound pipe item DTO.
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct OutboundPipeItem {
    /// Pipe type: seamless or screen.
    #[validate(length(min = 1))]
    pub pipe_type: String,
    /// Pipe ID.
    #[validate(range(min = 1))]
    pub pipe_id: i64,
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
    /// Full-text search.
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
    /// Filter by steel grade.
    pub grade: Option<String>,
    /// Filter by pipe type.
    pub pipe_type: Option<String>,
    /// Filter by location.
    pub location_id: Option<i64>,
    /// Full-text search.
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
    /// Actual status found during check.
    #[validate(length(min = 1))]
    pub found_status: String,
    /// Notes.
    pub notes: Option<String>,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Approval DTOs
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Approve request DTO (empty body — just triggers the action).
#[derive(Debug, Deserialize, Validate)]
pub struct ApproveRequest {
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
    /// Filter by pipe type.
    pub pipe_type: Option<String>,
    /// Filter by steel grade.
    pub grade: Option<String>,
    /// Filter by location.
    #[validate(range(min = 1, message = "Location ID must be positive"))]
    pub location_id: Option<i64>,
}

/// ATP query result item DTO.
#[derive(Debug, Serialize)]
pub struct AtpItem {
    /// Pipe type.
    pub pipe_type: String,
    /// Steel grade.
    pub grade: String,
    /// Available quantity.
    pub quantity: i64,
    /// Location ID.
    pub location_id: Option<i64>,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Location Assignment DTOs
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Assign location request DTO.
#[derive(Debug, Deserialize, Validate)]
pub struct AssignLocationRequest {
    /// Pipe type: seamless or screen.
    #[validate(length(min = 1))]
    pub pipe_type: String,
    /// Pipe ID.
    #[validate(range(min = 1))]
    pub pipe_id: i64,
}

/// Transfer location request DTO.
#[derive(Debug, Deserialize, Validate)]
pub struct TransferLocationRequest {
    /// Target location ID.
    #[validate(range(min = 1))]
    pub to_location_id: i64,
    /// Notes.
    pub notes: Option<String>,
}

/// Batch create inbound records request DTO.
#[derive(Debug, Deserialize, Validate)]
pub struct BatchCreateInboundRequest {
    /// List of inbound records.
    #[validate(length(min = 1))]
    pub records: Vec<CreateInboundRecordRequest>,
}

/// Inventory statistics response DTO — total stock, breakdown by grade and location.
#[derive(Debug, Serialize)]
pub struct InventoryStatistics {
    pub total_in_stock: i64,
    pub by_grade: Vec<crate::repositories::inventory_repo::GradeCount>,
    pub by_location: Vec<crate::repositories::inventory_repo::LocationCount>,
}

/// Check record detail response DTO — record header + check items.
#[derive(Debug, Serialize)]
pub struct CheckRecordDetail {
    pub record: crate::models::inventory::InventoryCheckRecord,
    pub items: Vec<crate::models::inventory::InventoryCheckItem>,
}

/// Stock item DTO — unified row from seamless_pipes or screen_pipes.
#[derive(Debug, Serialize)]
pub struct StockItem {
    pub id: i64,
    pub pipe_number: String,
    pub grade: String,
    pub od: f64,
    pub wt: f64,
    pub pipe_type: String,
    pub status: String,
    pub location_id: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}
