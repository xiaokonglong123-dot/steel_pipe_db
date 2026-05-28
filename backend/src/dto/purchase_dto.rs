use serde::Deserialize;
use validator::Validate;

/// Approve purchase order request DTO.
#[derive(Debug, Deserialize, Validate)]
pub struct ApproveOrderRequest {
    /// Approval notes.
    pub notes: Option<String>,
}

/// Reject purchase order request DTO.
#[derive(Debug, Deserialize, Validate)]
pub struct RejectOrderRequest {
    /// Rejection reason.
    #[validate(length(min = 1))]
    pub reason: String,
}

/// Link inbound record request DTO.
#[derive(Debug, Deserialize, Validate)]
pub struct LinkInboundRequest {
    /// Inbound record ID.
    #[validate(range(min = 1))]
    pub inbound_record_id: i64,
}

/// Create purchase order request DTO.
#[derive(Debug, Deserialize, Validate)]
pub struct CreatePurchaseOrderRequest {
    /// Order number (auto-generated if empty).
    pub order_no: Option<String>,
    /// Supplier ID.
    #[validate(range(min = 1))]
    pub supplier_id: i64,
    /// Order date.
    #[validate(length(min = 1))]
    pub order_date: String,
    /// Notes.
    pub notes: Option<String>,
    /// List of purchase items.
    pub items: Vec<CreatePurchaseItemRequest>,
}

/// Update purchase order request DTO.
#[derive(Debug, Deserialize, Validate)]
pub struct UpdatePurchaseOrderRequest {
    /// Order date.
    pub order_date: Option<String>,
    /// Notes.
    pub notes: Option<String>,
}

/// Create purchase order item request DTO.
#[derive(Debug, Deserialize, Validate)]
pub struct CreatePurchaseItemRequest {
    /// Pipe type: seamless or screen.
    #[validate(length(min = 1))]
    pub pipe_type: String,
    /// Steel grade.
    #[validate(length(min = 1))]
    pub grade: String,
    /// Outer diameter (mm).
    #[validate(range(min = 0.0))]
    pub od: f64,
    /// Wall thickness (mm).
    #[validate(range(min = 0.0))]
    pub wt: f64,
    /// Quantity ordered.
    #[validate(range(min = 1))]
    pub quantity: i64,
    /// Unit price.
    pub unit_price: Option<f64>,
    /// Total price.
    pub total_price: Option<f64>,
    /// Notes.
    pub notes: Option<String>,
}

/// Update purchase order item request DTO.
///
/// Note: `total_price` is NOT client-writable — the server always computes it
/// as `quantity * unit_price` to prevent tampering.
#[derive(Debug, Deserialize, Validate)]
pub struct UpdatePurchaseItemRequest {
    pub pipe_type: Option<String>,
    pub grade: Option<String>,
    pub od: Option<f64>,
    pub wt: Option<f64>,
    pub quantity: Option<i64>,
    pub unit_price: Option<f64>,
    pub notes: Option<String>,
}

/// Purchase order list filter params.
#[derive(Debug, Deserialize)]
pub struct PurchaseOrderFilterParams {
    /// Full-text search.
    pub q: Option<String>,
    /// Filter by status.
    pub status: Option<String>,
    /// Filter by supplier.
    pub supplier_id: Option<i64>,
    /// Order date start.
    pub order_date_from: Option<String>,
    /// Order date end.
    pub order_date_to: Option<String>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

/// Purchase order status change request DTO.
#[derive(Debug, Deserialize, Validate)]
pub struct PurchaseOrderStatusTransitionRequest {
    /// Target status.
    #[validate(length(min = 1))]
    pub status: String,
}

/// Purchase order detail response DTO (includes order header + line items).
/// Used by GET `/api/v1/purchase-orders/{id}` to return a consistent ApiResponse shape.
#[derive(Debug, serde::Serialize)]
pub struct PurchaseOrderDetailResponse {
    pub order: crate::models::purchase_order::PurchaseOrder,
    pub items: Vec<crate::models::purchase_order::PurchaseOrderItem>,
}
