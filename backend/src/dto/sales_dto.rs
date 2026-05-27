use serde::Deserialize;
use validator::Validate;

/// Approve sales order request DTO.
#[derive(Debug, Deserialize, Validate)]
pub struct ApproveOrderRequest {
    /// Approval notes.
    pub notes: Option<String>,
}

/// Reject sales order request DTO.
#[derive(Debug, Deserialize, Validate)]
pub struct RejectOrderRequest {
    /// Rejection reason.
    #[validate(length(min = 1))]
    pub reason: String,
}

/// Link outbound record request DTO.
#[derive(Debug, Deserialize, Validate)]
pub struct LinkOutboundRequest {
    /// Outbound record ID.
    #[validate(range(min = 1))]
    pub outbound_record_id: i64,
}

/// Create sales order request DTO.
#[derive(Debug, Deserialize, Validate)]
pub struct CreateSalesOrderRequest {
    /// Order number (auto-generated if empty).
    pub order_no: Option<String>,
    /// Customer ID.
    #[validate(range(min = 1))]
    pub customer_id: i64,
    /// Order date.
    #[validate(length(min = 1))]
    pub order_date: String,
    /// Notes.
    pub notes: Option<String>,
    /// List of sales items.
    pub items: Vec<CreateSalesItemRequest>,
}

/// Update sales order request DTO.
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateSalesOrderRequest {
    /// Order date.
    pub order_date: Option<String>,
    /// Notes.
    pub notes: Option<String>,
}

/// Create sales order item request DTO.
#[derive(Debug, Deserialize, Validate)]
pub struct CreateSalesItemRequest {
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

/// Update sales order item request DTO.
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateSalesItemRequest {
    pub pipe_type: Option<String>,
    pub grade: Option<String>,
    pub od: Option<f64>,
    pub wt: Option<f64>,
    pub quantity: Option<i64>,
    pub unit_price: Option<f64>,
    pub total_price: Option<f64>,
    pub notes: Option<String>,
}

/// Sales order list filter params.
#[derive(Debug, Deserialize)]
pub struct SalesOrderFilterParams {
    /// Full-text search.
    pub q: Option<String>,
    /// Filter by status.
    pub status: Option<String>,
    /// Filter by customer.
    pub customer_id: Option<i64>,
    /// Order date start.
    pub order_date_from: Option<String>,
    /// Order date end.
    pub order_date_to: Option<String>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

/// Sales order status change request DTO.
#[derive(Debug, Deserialize, Validate)]
pub struct SalesOrderStatusTransitionRequest {
    /// Target status.
    #[validate(length(min = 1))]
    pub status: String,
}
