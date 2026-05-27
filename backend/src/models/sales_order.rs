use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Sales order DB row. Represents a sales order for selling pipes to a customer.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SalesOrder {
    pub id: i64,
    /// Sales order number.
    pub order_no: String,
    /// Customer ID we're selling to.
    pub customer_id: i64,
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

/// Sales order item DB row. Line items — what pipes and how many.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SalesOrderItem {
    pub id: i64,
    /// FK back to the sales order.
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
    /// Quantity delivered so far.
    pub delivered_quantity: i64,
    /// Unit price.
    pub unit_price: Option<f64>,
    /// Total price for this line.
    pub total_price: Option<f64>,
    /// Free-form notes.
    pub notes: Option<String>,
    pub created_at: String,
}
