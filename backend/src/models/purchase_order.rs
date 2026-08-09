use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::domain::money::to_decimal_opt;
use crate::domain::order::OrderStatus;
use std::str::FromStr;

/// Purchase order DB row. Buying items from a supplier — the full PO header.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PurchaseOrder {
    pub id: i64,
    /// Purchase order number.
    pub order_no: String,
    /// Supplier ID we're buying from.
    pub supplier_id: i64,
    /// Order date.
    pub order_date: DateTime<Utc>,
    /// Status stored as string in DB; use `order_status()` for typed access.
    pub status: String,
    /// Total order amount.
    pub total_amount: Option<f64>,
    /// Free-form notes.
    pub notes: Option<String>,
    /// User ID who created this order.
    pub created_by: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl PurchaseOrder {
    /// Returns the typed `OrderStatus` enum for this order.
    /// Returns `None` if the stored string is not a valid status value.
    pub fn order_status(&self) -> Option<OrderStatus> {
        FromStr::from_str(&self.status).ok()
    }

    /// Returns `total_amount` as a `Decimal` for precise arithmetic.
    pub fn total_amount_decimal(&self) -> Option<Decimal> {
        to_decimal_opt(self.total_amount)
    }
}

impl PurchaseOrderItem {
    /// Returns `unit_price` as a `Decimal` for precise arithmetic.
    pub fn unit_price_decimal(&self) -> Option<Decimal> {
        to_decimal_opt(self.unit_price)
    }

    /// Returns `total_price` as a `Decimal` for precise arithmetic.
    pub fn total_price_decimal(&self) -> Option<Decimal> {
        to_decimal_opt(self.total_price)
    }
}

/// Purchase order item DB row. Line items — what items and how many.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PurchaseOrderItem {
    pub id: i64,
    /// FK back to the purchase order.
    pub order_id: i64,
    /// FK to the item master (`items.id`).
    pub item_id: i64,
    /// Quantity ordered.
    pub quantity: f64,
    /// Quantity received so far.
    pub received_quantity: f64,
    /// Unit price.
    pub unit_price: Option<f64>,
    /// Total price for this line.
    pub total_price: Option<f64>,
    /// Free-form notes.
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}
