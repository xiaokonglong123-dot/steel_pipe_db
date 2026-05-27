use serde::{Deserialize, Serialize};

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

impl OrderStatus {
    /// Parse a string into an order status. Returns `None` if the string's garbage.
    /// Valid values: `"draft" | "pending" | "approved" | "rejected" | "completed" | "cancelled"`
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "draft" => Some(Self::Draft),
            "pending" => Some(Self::Pending),
            "approved" => Some(Self::Approved),
            "rejected" => Some(Self::Rejected),
            "completed" => Some(Self::Completed),
            "cancelled" => Some(Self::Cancelled),
            _ => None,
        }
    }

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
