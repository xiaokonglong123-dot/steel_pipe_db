use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Purchase order DB row. Buying pipes from a supplier — the full PO header.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PurchaseOrder {
    pub id: i64,
    /// Purchase order number.
    pub order_no: String,
    /// Supplier ID we're buying from.
    pub supplier_id: i64,
    /// Order date.
    pub order_date: String,
    /// Status: draft / pending / approved / rejected / completed / cancelled.
    pub status: String,
    /// Total order amount.
    pub total_amount: Option<f64>,
    /// Free-form notes.
    pub notes: Option<String>,
    /// User ID who created this order.
    pub created_by: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

/// Purchase order item DB row. Line items — what pipes and how many.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PurchaseOrderItem {
    pub id: i64,
    /// FK back to the purchase order.
    pub order_id: i64,
    /// Pipe type: seamless or screen.
    pub pipe_type: String,
    /// Steel grade.
    pub grade: String,
    /// Outer diameter (mm).
    pub od: f64,
    /// Wall thickness (mm).
    pub wt: f64,
    /// Quantity ordered.
    pub quantity: i64,
    /// Quantity received so far.
    pub received_quantity: i64,
    /// Unit price.
    pub unit_price: Option<f64>,
    /// Total price for this line.
    pub total_price: Option<f64>,
    /// Free-form notes.
    pub notes: Option<String>,
    pub created_at: String,
}
