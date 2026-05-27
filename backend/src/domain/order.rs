use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Lifecycle state for purchase and sales orders.
///
/// Transitions:
/// `Draft → Pending → Approved → Completed`
/// `Draft → Cancelled`
/// `Pending → Rejected → Draft`
/// `Approved → Cancelled`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    Draft,
    Pending,
    Approved,
    Rejected,
    Completed,
    Cancelled,
}

impl FromStr for OrderStatus {
    type Err = ();

    /// Parse a string into an order status. Returns `Err(())` if the string's garbage.
    /// Valid values: `"draft" | "pending" | "approved" | "rejected" | "completed" | "cancelled"`
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "draft" => Ok(Self::Draft),
            "pending" => Ok(Self::Pending),
            "approved" => Ok(Self::Approved),
            "rejected" => Ok(Self::Rejected),
            "completed" => Ok(Self::Completed),
            "cancelled" => Ok(Self::Cancelled),
            _ => Err(()),
        }
    }
}

impl OrderStatus {

    /// Check whether transitioning from the current status to the target is kosher.
    /// Valid transition matrix:
    /// - Draft → Pending | Cancelled
    /// - Pending → Approved | Rejected
    /// - Rejected → Draft
    /// - Approved → Completed | Cancelled
    pub fn valid_transition(&self, target: &Self) -> bool {
        matches!(
            (self, target),
            (Self::Draft, Self::Pending)
                | (Self::Pending, Self::Approved)
                | (Self::Pending, Self::Rejected)
                | (Self::Rejected, Self::Draft)
                | (Self::Approved, Self::Completed)
                | (Self::Approved, Self::Cancelled)
                | (Self::Draft, Self::Cancelled)
        )
    }
}
