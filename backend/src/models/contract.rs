use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::domain::money::{to_decimal, to_decimal_opt};

/// Contract DB row. Purchase or sales contract — the big picture.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Contract {
    pub id: i64,
    /// Contract number.
    pub contract_no: String,
    /// Contract type: purchase or sales.
    pub contract_type: String,
    /// Contract title.
    pub title: String,
    /// Party A name.
    pub party_a: String,
    /// Party B name.
    pub party_b: String,
    /// Signing date.
    pub sign_date: Option<String>,
    /// Contract effective date.
    pub start_date: Option<String>,
    /// Contract expiry date.
    pub end_date: Option<String>,
    /// Total contract amount.
    pub total_amount: Option<f64>,
    /// Status: draft / active / completed / terminated / cancelled.
    pub status: String,
    /// Free-form notes.
    pub notes: Option<String>,
    /// User ID who created this contract.
    pub created_by: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

impl Contract {
    /// Returns `total_amount` as a `Decimal` for precise arithmetic.
    pub fn total_amount_decimal(&self) -> Option<Decimal> {
        to_decimal_opt(self.total_amount)
    }
}

impl ContractItem {
    /// Returns `unit_price` as a `Decimal` for precise arithmetic.
    pub fn unit_price_decimal(&self) -> Option<Decimal> {
        to_decimal_opt(self.unit_price)
    }

    /// Returns `total_price` as a `Decimal` for precise arithmetic.
    pub fn total_price_decimal(&self) -> Option<Decimal> {
        to_decimal_opt(self.total_price)
    }
}

impl ContractPayment {
    /// Returns `amount` as a `Decimal` for precise arithmetic.
    pub fn amount_decimal(&self) -> Decimal {
        to_decimal(self.amount)
    }
}

/// Contract item DB row. Pipe specs and quantities under a contract.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ContractItem {
    pub id: i64,
    /// FK back to the contract.
    pub contract_id: i64,
    /// Pipe type: seamless or screen.
    pub pipe_type: String,
    /// Steel grade.
    pub grade: String,
    /// Outer diameter (mm).
    pub od: f64,
    /// Wall thickness (mm).
    pub wt: f64,
    /// Quantity.
    pub quantity: i64,
    /// Unit price.
    pub unit_price: Option<f64>,
    /// Total price for this line.
    pub total_price: Option<f64>,
    /// Free-form notes.
    pub notes: Option<String>,
    pub created_at: String,
}

/// Contract payment milestone DB row. Payment schedule — when and how much.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ContractPayment {
    pub id: i64,
    /// FK back to the contract.
    pub contract_id: i64,
    /// Payment due date.
    pub due_date: String,
    /// Payment amount.
    pub amount: f64,
    /// Payment type: deposit / progress / final / retention.
    pub payment_type: String,
    /// Whether paid.
    pub is_paid: bool,
    /// Actual payment date.
    pub paid_date: Option<String>,
    /// Free-form notes.
    pub notes: Option<String>,
    pub created_at: String,
}
